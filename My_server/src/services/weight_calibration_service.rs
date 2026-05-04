//! 载重标定参数计算服务
//!
//! 提供传感器标定、重量计算、标定表管理等功能

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info, warn};

/// 多项式系数（3阶）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolynomialCoefficients {
    pub coef_3: f64,   // 3阶系数
    pub coef_2: f64,   // 2阶系数
    pub coef_1: f64,   // 1阶系数
    pub constant: f64, // 常数
}

impl PolynomialCoefficients {
    /// 计算多项式值: y = coef_3 * x^3 + coef_2 * x^2 + coef_1 * x + constant
    pub fn calculate(&self, x: f64) -> f64 {
        self.coef_3 * x.powi(3) + self.coef_2 * x.powi(2) + self.coef_1 * x + self.constant
    }
}

impl Default for PolynomialCoefficients {
    fn default() -> Self {
        Self {
            coef_3: 0.0,
            coef_2: 0.0,
            coef_1: 1.0, // 默认线性关系
            constant: 0.0,
        }
    }
}

/// 分段系数（转折前/后）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentCoefficients {
    pub before_turning: PolynomialCoefficients, // 转折前
    pub after_turning: PolynomialCoefficients,  // 转折后
    pub turning_point: f64,                     // 转折点(AD值)
}

impl Default for SegmentCoefficients {
    fn default() -> Self {
        Self {
            before_turning: PolynomialCoefficients::default(),
            after_turning: PolynomialCoefficients::default(),
            turning_point: 50000.0, // 默认转折点
        }
    }
}

/// 标定点数据参数（简化版）
#[derive(Debug, Clone)]
pub struct CalibrationPointParams {
    pub plate_number: String,
    pub axle_number: u8,
    pub is_left_wheel: bool,
    pub pa_value: u32,
    pub actual_weight: f64,
    pub temperature: f64,
    pub load_percentage: u8,
}

/// 标定点数据参数（完整版，带标定时间）
#[derive(Debug, Clone)]
pub struct CalibrationPointWithTimeParams {
    pub plate_number: String,
    pub axle_number: u8,
    pub is_left_wheel: bool,
    pub calibration_time: DateTime<Utc>,
    pub pa_raw: u32,
    pub actual_weight: f64,
    pub temperature: f64,
    pub load_percentage: u8,
}

/// 轴传感器系数（左右轮）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AxleSensorCoefficients {
    pub axle_number: u8,                  // 轴号(1-4)
    pub left_wheel: SegmentCoefficients,  // 左侧轮
    pub right_wheel: SegmentCoefficients, // 右侧轮
}

/// 传感器标定参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorCalibration {
    pub sensor_address: u16,               // 传感器地址/编号
    pub sensor_name: String,               // 传感器名称(如"左前轮")
    pub axle_number: u8,                   // 所属轴号
    pub is_left_wheel: bool,               // 是否左侧轮
    pub coefficients: SegmentCoefficients, // 分段系数
    pub calibration_date: DateTime<Utc>,   // 标定日期
    pub is_calibrated: bool,               // 是否已标定
}

/// 车辆标定表
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VehicleCalibrationTable {
    pub vehicle_id: String,                                     // 车辆ID
    pub plate_number: String,                                   // 车牌号
    pub rated_total_weight: f64,                                // 额定总重(kg)
    pub tare_weight: f64,                                       // 自重/空车重量(kg)
    pub axle_count: u8,                                         // 轴数
    pub sensor_calibrations: HashMap<u16, SensorCalibration>,   // 传感器标定参数(按地址索引)
    pub axle_coefficients: HashMap<u8, AxleSensorCoefficients>, // 轴系数(按轴号索引)
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 标定点数据(用于计算系数)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalibrationPoint {
    pub sensor_address: u16,             // 传感器编号(Addr)
    pub calibration_time: DateTime<Utc>, // 标定时间(用于抓取对应时间的传感器数据)
    pub actual_weight: f64,              // 实际轮重(kg)
    pub pa_value: u32,                   // 压力值Pa (Pa原始值/100,取整)
    pub pa_raw: u32,                     // Pa原始值
    pub temperature: f64,                // 温度(°C)
    pub load_percentage: u8,             // 负载百分比(0=空车, 25, 50, 75, 100)
    pub is_manual: bool,                 // 是否为手工修改
    pub record_time: DateTime<Utc>,      // 记录时间
}

impl CalibrationPoint {
    /// 从原始Pa值计算取整Pa值 (Pa原始值/100,取整)
    pub fn calculate_pa_from_raw(raw_pa: u32) -> u32 {
        (raw_pa as f64 / 100.0).round() as u32
    }

    /// 设置Pa值(自动计算)
    pub fn set_pa_auto(&mut self, raw_pa: u32) {
        self.pa_raw = raw_pa;
        self.pa_value = Self::calculate_pa_from_raw(raw_pa);
        self.is_manual = false;
    }

    /// 手动设置Pa值
    pub fn set_pa_manual(&mut self, pa_value: u32) {
        self.pa_value = pa_value;
        self.is_manual = true;
    }
}

/// 多项式阶数
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PolynomialOrder {
    First = 1,  // 一阶: W(x) = ax + b
    Second = 2, // 二阶: W(x) = ax² + bx + c
    Third = 3,  // 三阶: W(x) = ax³ + bx² + cx + d
}

impl PolynomialOrder {
    pub fn as_str(&self) -> &'static str {
        match self {
            PolynomialOrder::First => "一阶",
            PolynomialOrder::Second => "二阶",
            PolynomialOrder::Third => "三阶",
        }
    }

    pub fn required_points(&self) -> usize {
        match self {
            PolynomialOrder::First => 2,
            PolynomialOrder::Second => 3,
            PolynomialOrder::Third => 4,
        }
    }
}

/// 标定阶段
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CalibrationStage {
    Empty = 0,        // 空车轮重
    Load25 = 25,      // 25%满载
    Load50 = 50,      // 50%满载
    Load75 = 75,      // 75%满载
    Load100 = 100,    // 100%满载
    Additional = 255, // 增补数据
}

impl CalibrationStage {
    pub fn from_percentage(pct: u8) -> Self {
        match pct {
            0 => CalibrationStage::Empty,
            25 => CalibrationStage::Load25,
            50 => CalibrationStage::Load50,
            75 => CalibrationStage::Load75,
            100 => CalibrationStage::Load100,
            _ => CalibrationStage::Additional,
        }
    }

    pub fn as_percentage(&self) -> u8 {
        match self {
            CalibrationStage::Empty => 0,
            CalibrationStage::Load25 => 25,
            CalibrationStage::Load50 => 50,
            CalibrationStage::Load75 => 75,
            CalibrationStage::Load100 => 100,
            CalibrationStage::Additional => 255,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            CalibrationStage::Empty => "空车轮重",
            CalibrationStage::Load25 => "25%满载",
            CalibrationStage::Load50 => "50%满载",
            CalibrationStage::Load75 => "75%满载",
            CalibrationStage::Load100 => "100%满载",
            CalibrationStage::Additional => "增补数据",
        }
    }
}

/// 轮子标定记录(单轮)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WheelCalibrationRecord {
    pub sensor_address: u16,                                  // 传感器地址
    pub wheel_position: String,                               // 轮子位置(如"左前轮")
    pub calibration_points: Vec<CalibrationPoint>,            // 标定点数据
    pub calculated_coefficients: Option<SegmentCoefficients>, // 计算出的系数
}

/// 轴标定记录(左右轮)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AxleCalibrationRecord {
    pub axle_number: u8,                     // 轴号(1-6)
    pub left_wheel: WheelCalibrationRecord,  // 左轮记录
    pub right_wheel: WheelCalibrationRecord, // 右轮记录
}

/// 车辆完整标定表
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VehicleCalibrationSheet {
    pub plate_number: String,                             // 车牌号
    pub rated_total_weight: f64,                          // 额定总重(kg)
    pub tare_weight: f64,                                 // 空车重量(kg)
    pub axle_count: u8,                                   // 轴数(1-6)
    pub axle_records: HashMap<u8, AxleCalibrationRecord>, // 各轴标定记录
    pub is_completed: bool,                               // 是否完成所有标定
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 标定进度报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalibrationProgress {
    pub plate_number: String,            // 车牌号
    pub axle_count: u8,                  // 轴数
    pub total_wheels: u8,                // 总轮数
    pub completed_wheels: u8,            // 已完成标定的轮数
    pub total_calibration_points: usize, // 总标定点数
    pub is_completed: bool,              // 是否全部完成
}

/// 载重计算结果
#[derive(Debug, Clone)]
pub struct WeightCalculationResult {
    pub sensor_address: u16,
    pub sensor_name: String,
    pub ad_value: u32,
    pub raw_weight: f64,        // 原始计算重量
    pub calibrated_weight: f64, // 标定后重量
    pub temperature: f64,
}

/// 车辆总重计算结果
#[derive(Debug, Clone)]
pub struct VehicleWeightResult {
    pub plate_number: String,
    pub timestamp: DateTime<Utc>,
    pub sensor_weights: Vec<WeightCalculationResult>,
    pub total_sensor_weight: f64, // 传感器总重
    pub total_weight: f64,        // 总重(含标定)
    pub load_weight: f64,         // 载重
    pub rated_total_weight: f64,  // 额定总重
    pub overload: bool,           // 是否超载
    pub overload_amount: f64,     // 超载量
}

pub struct WeightCalibrationService {
    calibration_tables: HashMap<String, VehicleCalibrationTable>, // 按车牌号索引(旧版)
    calibration_sheets: HashMap<String, VehicleCalibrationSheet>, // 新版标定表
}

impl WeightCalibrationService {
    pub fn new() -> Self {
        Self {
            calibration_tables: HashMap::new(),
            calibration_sheets: HashMap::new(),
        }
    }

    // ==================== 新版标定表管理 ====================

    /// 创建新的标定表
    pub fn create_calibration_sheet(
        &mut self,
        plate_number: &str,
        rated_total_weight: f64,
        tare_weight: f64,
        axle_count: u8,
    ) -> Result<&VehicleCalibrationSheet, String> {
        if axle_count == 0 || axle_count > 6 {
            return Err("轴数必须在1-6之间".to_string());
        }

        let mut axle_records = HashMap::new();

        // 初始化各轴记录
        for axle_num in 1..=axle_count {
            let left_addr = 3754 + axle_num as u16 * 2; // 左轮地址: 3756, 3758, ...
            let right_addr = 3779 + axle_num as u16; // 右轮地址: 3780, 3781, ...

            let axle_record = AxleCalibrationRecord {
                axle_number: axle_num,
                left_wheel: WheelCalibrationRecord {
                    sensor_address: left_addr,
                    wheel_position: format!("左{}轮", axle_num),
                    calibration_points: Vec::new(),
                    calculated_coefficients: None,
                },
                right_wheel: WheelCalibrationRecord {
                    sensor_address: right_addr,
                    wheel_position: format!("右{}轮", axle_num),
                    calibration_points: Vec::new(),
                    calculated_coefficients: None,
                },
            };
            axle_records.insert(axle_num, axle_record);
        }

        let sheet = VehicleCalibrationSheet {
            plate_number: plate_number.to_string(),
            rated_total_weight,
            tare_weight,
            axle_count,
            axle_records,
            is_completed: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        self.calibration_sheets
            .insert(plate_number.to_string(), sheet);
        info!(
            "创建车辆标定表: {}, 轴数: {}, 额定总重: {}kg",
            plate_number, axle_count, rated_total_weight
        );

        self.calibration_sheets
            .get(plate_number)
            .ok_or_else(|| format!("标定表不存在: {}", plate_number))
    }

    /// 添加标定点数据（简化版）
    pub fn add_calibration_point(&mut self, params: CalibrationPointParams) -> Result<(), String> {
        let sheet = self
            .calibration_sheets
            .get_mut(&params.plate_number)
            .ok_or_else(|| format!("标定表不存在: {}", params.plate_number))?;

        let axle_record = sheet
            .axle_records
            .get_mut(&params.axle_number)
            .ok_or_else(|| format!("轴号不存在: {}", params.axle_number))?;

        let wheel_record = if params.is_left_wheel {
            &mut axle_record.left_wheel
        } else {
            &mut axle_record.right_wheel
        };

        let sensor_address = wheel_record.sensor_address;

        // 检查是否已存在相同负载百分比的记录，如果存在则更新
        if let Some(existing) = wheel_record
            .calibration_points
            .iter_mut()
            .find(|p| p.load_percentage == params.load_percentage)
        {
            existing.pa_value = params.pa_value;
            existing.pa_raw = params.pa_value * 100; // 反向计算原始值
            existing.actual_weight = params.actual_weight;
            existing.temperature = params.temperature;
            existing.record_time = Utc::now();
            existing.is_manual = true;
            info!(
                "更新标定点: {} 轴{} {}%负载 Pa={}",
                params.plate_number, params.axle_number, params.load_percentage, params.pa_value
            );
        } else {
            let point = CalibrationPoint {
                sensor_address,
                calibration_time: Utc::now(),
                actual_weight: params.actual_weight,
                pa_value: params.pa_value,
                pa_raw: params.pa_value * 100,
                temperature: params.temperature,
                load_percentage: params.load_percentage,
                is_manual: true,
                record_time: Utc::now(),
            };
            wheel_record.calibration_points.push(point);
            info!(
                "添加标定点: {} 轴{} {}%负载 Pa={}",
                params.plate_number, params.axle_number, params.load_percentage, params.pa_value
            );
        }

        sheet.updated_at = Utc::now();
        Ok(())
    }

    /// 添加标定点数据（完整版，带标定时间）
    pub fn add_calibration_point_with_time(
        &mut self,
        params: CalibrationPointWithTimeParams,
    ) -> Result<(), String> {
        let sheet = self
            .calibration_sheets
            .get_mut(&params.plate_number)
            .ok_or_else(|| format!("标定表不存在: {}", params.plate_number))?;

        let axle_record = sheet
            .axle_records
            .get_mut(&params.axle_number)
            .ok_or_else(|| format!("轴号不存在: {}", params.axle_number))?;

        let wheel_record = if params.is_left_wheel {
            &mut axle_record.left_wheel
        } else {
            &mut axle_record.right_wheel
        };

        let sensor_address = wheel_record.sensor_address;
        let pa_value = CalibrationPoint::calculate_pa_from_raw(params.pa_raw);

        // 检查是否已存在相同负载百分比的记录，如果存在则更新
        if let Some(existing) = wheel_record
            .calibration_points
            .iter_mut()
            .find(|p| p.load_percentage == params.load_percentage)
        {
            existing.calibration_time = params.calibration_time;
            existing.pa_raw = params.pa_raw;
            existing.pa_value = pa_value;
            existing.actual_weight = params.actual_weight;
            existing.temperature = params.temperature;
            existing.record_time = Utc::now();
            existing.is_manual = false;
            info!(
                "更新标定点: {} 轴{} {}%负载 原始Pa={} 取整Pa={}",
                params.plate_number,
                params.axle_number,
                params.load_percentage,
                params.pa_raw,
                pa_value
            );
        } else {
            let point = CalibrationPoint {
                sensor_address,
                calibration_time: params.calibration_time,
                actual_weight: params.actual_weight,
                pa_value,
                pa_raw: params.pa_raw,
                temperature: params.temperature,
                load_percentage: params.load_percentage,
                is_manual: false,
                record_time: Utc::now(),
            };
            wheel_record.calibration_points.push(point);
            info!(
                "添加标定点: {} 轴{} {}%负载 原始Pa={} 取整Pa={}",
                params.plate_number,
                params.axle_number,
                params.load_percentage,
                params.pa_raw,
                pa_value
            );
        }

        sheet.updated_at = Utc::now();
        Ok(())
    }

    /// 获取标定表
    pub fn get_calibration_sheet(&self, plate_number: &str) -> Option<&VehicleCalibrationSheet> {
        self.calibration_sheets.get(plate_number)
    }

    /// 获取所有标定表
    pub fn get_all_sheets(&self) -> &HashMap<String, VehicleCalibrationSheet> {
        &self.calibration_sheets
    }

    /// 检查标定是否完成
    pub fn check_calibration_complete(&self, plate_number: &str) -> bool {
        if let Some(sheet) = self.calibration_sheets.get(plate_number) {
            for axle_num in 1..=sheet.axle_count {
                if let Some(axle) = sheet.axle_records.get(&axle_num) {
                    // 检查左轮是否有至少2个标定点(空车+满载)
                    if axle.left_wheel.calibration_points.len() < 2 {
                        return false;
                    }
                    // 检查右轮是否有至少2个标定点
                    if axle.right_wheel.calibration_points.len() < 2 {
                        return false;
                    }
                }
            }
            true
        } else {
            false
        }
    }

    /// 计算并更新传感器系数
    pub fn calculate_sensor_coefficients(
        &mut self,
        plate_number: &str,
        axle_number: u8,
        is_left_wheel: bool,
        turning_point: f64,
    ) -> Result<SegmentCoefficients, String> {
        let sheet = self
            .calibration_sheets
            .get(plate_number)
            .ok_or_else(|| format!("标定表不存在: {}", plate_number))?;

        let axle_record = sheet
            .axle_records
            .get(&axle_number)
            .ok_or_else(|| format!("轴号不存在: {}", axle_number))?;

        let wheel_record = if is_left_wheel {
            &axle_record.left_wheel
        } else {
            &axle_record.right_wheel
        };

        if wheel_record.calibration_points.len() < 4 {
            return Err("至少需要4个标定点才能计算系数".to_string());
        }

        // 按Pa值排序
        let mut points: Vec<_> = wheel_record.calibration_points.clone();
        points.sort_by_key(|a| a.pa_value);

        // 分割为转折前和转折后两组
        let before_turning: Vec<_> = points
            .iter()
            .filter(|p| (p.pa_value as f64) < turning_point)
            .cloned()
            .collect();

        let after_turning: Vec<_> = points
            .iter()
            .filter(|p| (p.pa_value as f64) >= turning_point)
            .cloned()
            .collect();

        // 计算两段的系数
        let before_coeffs = if before_turning.len() >= 2 {
            Self::calculate_first_order_coefficients(&before_turning)?
        } else {
            PolynomialCoefficients::default()
        };

        let after_coeffs = if after_turning.len() >= 2 {
            Self::calculate_first_order_coefficients(&after_turning)?
        } else {
            PolynomialCoefficients::default()
        };

        let segment_coeffs = SegmentCoefficients {
            before_turning: before_coeffs,
            after_turning: after_coeffs,
            turning_point,
        };

        // 更新到标定表
        let sheet = self
            .calibration_sheets
            .get_mut(plate_number)
            .ok_or_else(|| format!("标定表不存在: {}", plate_number))?;
        let axle = sheet
            .axle_records
            .get_mut(&axle_number)
            .ok_or_else(|| format!("轴号不存在: {}", axle_number))?;
        let wheel = if is_left_wheel {
            &mut axle.left_wheel
        } else {
            &mut axle.right_wheel
        };
        wheel.calculated_coefficients = Some(segment_coeffs.clone());
        sheet.updated_at = Utc::now();

        info!(
            "计算传感器系数完成: {} 轴{} {} 转折点={}",
            plate_number,
            axle_number,
            if is_left_wheel { "左轮" } else { "右轮" },
            turning_point
        );

        Ok(segment_coeffs)
    }

    /// 计算一阶系数 W(x) = ax + b
    /// 使用最小二乘法拟合
    pub fn calculate_first_order_coefficients(
        points: &[CalibrationPoint],
    ) -> Result<PolynomialCoefficients, String> {
        if points.len() < 2 {
            return Err("至少需要2个标定点计算一阶系数".to_string());
        }

        // 使用最小二乘法拟合直线 y = ax + b
        let n = points.len() as f64;
        let sum_x: f64 = points.iter().map(|p| p.pa_value as f64).sum();
        let sum_y: f64 = points.iter().map(|p| p.actual_weight).sum();
        let sum_xy: f64 = points
            .iter()
            .map(|p| p.pa_value as f64 * p.actual_weight)
            .sum();
        let sum_x2: f64 = points.iter().map(|p| (p.pa_value as f64).powi(2)).sum();

        let denominator = n * sum_x2 - sum_x * sum_x;
        if denominator.abs() < 1e-10 {
            return Err("无法计算系数：分母为零".to_string());
        }

        let a = (n * sum_xy - sum_x * sum_y) / denominator;
        let b = (sum_y * sum_x2 - sum_x * sum_xy) / denominator;

        info!(
            "计算一阶系数: a={:.6}, b={:.2} (基于{}个标定点)",
            a,
            b,
            points.len()
        );

        Ok(PolynomialCoefficients {
            coef_3: 0.0,
            coef_2: 0.0,
            coef_1: a,
            constant: b,
        })
    }

    /// 计算二阶系数 W(x) = ax² + bx + c
    /// 使用最小二乘法拟合
    pub fn calculate_second_order_coefficients(
        points: &[CalibrationPoint],
    ) -> Result<PolynomialCoefficients, String> {
        if points.len() < 3 {
            return Err("至少需要3个标定点计算二阶系数".to_string());
        }

        // 最小二乘法拟合二次曲线
        let n = points.len() as f64;
        let sum_x: f64 = points.iter().map(|p| p.pa_value as f64).sum();
        let sum_x2: f64 = points.iter().map(|p| (p.pa_value as f64).powi(2)).sum();
        let sum_x3: f64 = points.iter().map(|p| (p.pa_value as f64).powi(3)).sum();
        let sum_x4: f64 = points.iter().map(|p| (p.pa_value as f64).powi(4)).sum();
        let sum_y: f64 = points.iter().map(|p| p.actual_weight).sum();
        let sum_xy: f64 = points
            .iter()
            .map(|p| p.pa_value as f64 * p.actual_weight)
            .sum();
        let sum_x2y: f64 = points
            .iter()
            .map(|p| (p.pa_value as f64).powi(2) * p.actual_weight)
            .sum();

        // 构建正规方程矩阵并求解 (简化实现)
        // [ n      sum_x    sum_x2  ] [ c ]   [ sum_y   ]
        // [ sum_x  sum_x2   sum_x3  ] [ b ] = [ sum_xy  ]
        // [ sum_x2 sum_x3   sum_x4  ] [ a ]   [ sum_x2y ]

        let a_mat = [
            [n, sum_x, sum_x2],
            [sum_x, sum_x2, sum_x3],
            [sum_x2, sum_x3, sum_x4],
        ];
        let b_vec = [sum_y, sum_xy, sum_x2y];

        // 使用克莱默法则求解3x3线性方程组
        let det = Self::determinant_3x3(&a_mat);
        if det.abs() < 1e-10 {
            return Err("无法计算二阶系数：矩阵奇异".to_string());
        }

        let a_mat_c = [
            [b_vec[0], a_mat[0][1], a_mat[0][2]],
            [b_vec[1], a_mat[1][1], a_mat[1][2]],
            [b_vec[2], a_mat[2][1], a_mat[2][2]],
        ];
        let a_mat_b = [
            [a_mat[0][0], b_vec[0], a_mat[0][2]],
            [a_mat[1][0], b_vec[1], a_mat[1][2]],
            [a_mat[2][0], b_vec[2], a_mat[2][2]],
        ];
        let a_mat_a = [
            [a_mat[0][0], a_mat[0][1], b_vec[0]],
            [a_mat[1][0], a_mat[1][1], b_vec[1]],
            [a_mat[2][0], a_mat[2][1], b_vec[2]],
        ];

        let c = Self::determinant_3x3(&a_mat_c) / det;
        let b = Self::determinant_3x3(&a_mat_b) / det;
        let a = Self::determinant_3x3(&a_mat_a) / det;

        info!(
            "计算二阶系数: a={:.10}, b={:.6}, c={:.2} (基于{}个标定点)",
            a,
            b,
            c,
            points.len()
        );

        Ok(PolynomialCoefficients {
            coef_3: 0.0,
            coef_2: a,
            coef_1: b,
            constant: c,
        })
    }

    /// 计算三阶系数 W(x) = ax³ + bx² + cx + d
    /// 使用最小二乘法拟合
    pub fn calculate_third_order_coefficients(
        points: &[CalibrationPoint],
    ) -> Result<PolynomialCoefficients, String> {
        if points.len() < 4 {
            return Err("至少需要4个标定点计算三阶系数".to_string());
        }

        // 对于三阶，使用简化的方法：先计算二阶，如果点足够多再拟合三阶
        // 实际生产环境应该使用更稳定的数值算法（如QR分解或SVD）
        warn!("三阶系数计算使用简化实现，建议实现完整的矩阵求解算法");

        // 这里返回二阶结果作为近似
        let coeffs = Self::calculate_second_order_coefficients(points)?;

        Ok(PolynomialCoefficients {
            coef_3: 0.0, // 简化处理，实际应该计算
            coef_2: coeffs.coef_2,
            coef_1: coeffs.coef_1,
            constant: coeffs.constant,
        })
    }

    /// 计算3x3矩阵行列式
    fn determinant_3x3(m: &[[f64; 3]; 3]) -> f64 {
        m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])
            - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
            + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0])
    }

    /// 根据指定阶数计算系数
    pub fn calculate_coefficients(
        points: &[CalibrationPoint],
        order: PolynomialOrder,
    ) -> Result<PolynomialCoefficients, String> {
        match order {
            PolynomialOrder::First => Self::calculate_first_order_coefficients(points),
            PolynomialOrder::Second => Self::calculate_second_order_coefficients(points),
            PolynomialOrder::Third => Self::calculate_third_order_coefficients(points),
        }
    }

    /// 计算单个传感器的重量
    /// W(x) = ax³ + bx² + cx + d (x为Pa值)
    pub fn calculate_sensor_weight(coeffs: &PolynomialCoefficients, pa_value: u32) -> f64 {
        let x = pa_value as f64;
        coeffs.coef_3 * x.powi(3) + coeffs.coef_2 * x.powi(2) + coeffs.coef_1 * x + coeffs.constant
    }

    /// 计算车辆总重
    /// 总重 = W(x1) + W(x2) + ... + W(xn)
    /// 其中x1, x2, ...为各传感器的Pa值
    pub fn calculate_total_weight(
        sensor_coeffs: &[(u16, PolynomialCoefficients)], // (传感器地址, 系数)
        sensor_pa_values: &[(u16, u32)],                 // (传感器地址, Pa值)
    ) -> f64 {
        let mut total_weight = 0.0;

        for (addr, pa_value) in sensor_pa_values {
            if let Some((_, coeffs)) = sensor_coeffs.iter().find(|(a, _)| a == addr) {
                let weight = Self::calculate_sensor_weight(coeffs, *pa_value);
                total_weight += weight;
                debug!("传感器{}: Pa={}, 重量={:.2}kg", addr, pa_value, weight);
            }
        }

        info!("计算总重: {:.2}kg", total_weight);
        total_weight
    }

    /// 计算载重
    /// 载重 = 总重 - 自重
    pub fn calculate_load_weight(total_weight: f64, tare_weight: f64) -> f64 {
        let load = total_weight - tare_weight;
        info!(
            "计算载重: {:.2}kg (总重{:.2}kg - 自重{:.2}kg)",
            load, total_weight, tare_weight
        );
        load
    }

    /// 为车辆所有传感器生成系数(默认一阶)
    pub fn generate_vehicle_coefficients(
        &mut self,
        plate_number: &str,
        order: PolynomialOrder,
    ) -> Result<HashMap<u16, PolynomialCoefficients>, String> {
        let sheet = self
            .calibration_sheets
            .get(plate_number)
            .ok_or_else(|| format!("标定表不存在: {}", plate_number))?;

        let mut coefficients = HashMap::new();

        for axle_num in 1..=sheet.axle_count {
            if let Some(axle) = sheet.axle_records.get(&axle_num) {
                // 左轮
                if axle.left_wheel.calibration_points.len() >= order.required_points() {
                    let coeffs =
                        Self::calculate_coefficients(&axle.left_wheel.calibration_points, order)?;
                    coefficients.insert(axle.left_wheel.sensor_address, coeffs);
                }

                // 右轮
                if axle.right_wheel.calibration_points.len() >= order.required_points() {
                    let coeffs =
                        Self::calculate_coefficients(&axle.right_wheel.calibration_points, order)?;
                    coefficients.insert(axle.right_wheel.sensor_address, coeffs);
                }
            }
        }

        info!(
            "为{}生成{}系数完成，共{}个传感器",
            plate_number,
            order.as_str(),
            coefficients.len()
        );

        Ok(coefficients)
    }

    /// 根据标定时间抓取传感器Pa值
    /// 从传感器历史数据中找到对应时间点的Pa值
    pub fn fetch_pa_value_at_time(
        sensor_address: u16,
        target_time: DateTime<Utc>,
        sensor_history: &[(DateTime<Utc>, u32)], // (时间, Pa原始值)
        tolerance_seconds: i64,
    ) -> Option<u32> {
        // 在容忍时间范围内查找最接近的数据点
        let closest = sensor_history
            .iter()
            .min_by_key(|(time, _)| (*time - target_time).num_seconds().abs());

        if let Some((time, raw_pa)) = closest {
            let diff_seconds = (*time - target_time).num_seconds().abs();
            if diff_seconds <= tolerance_seconds {
                let pa_value = CalibrationPoint::calculate_pa_from_raw(*raw_pa);
                info!(
                    "抓取传感器{}在{}的Pa值: 原始={}, 取整={} (时间差{}秒)",
                    sensor_address, target_time, raw_pa, pa_value, diff_seconds
                );
                return Some(pa_value);
            }
        }

        warn!("未找到传感器{}在{}附近的数据", sensor_address, target_time);
        None
    }

    /// 获取标定进度报告
    pub fn get_calibration_progress(&self, plate_number: &str) -> Option<CalibrationProgress> {
        let sheet = self.calibration_sheets.get(plate_number)?;

        let mut total_points = 0;
        let mut completed_wheels = 0;
        let mut total_wheels = 0;

        for axle_num in 1..=sheet.axle_count {
            if let Some(axle) = sheet.axle_records.get(&axle_num) {
                total_wheels += 2;

                // 左轮
                let left_points = axle.left_wheel.calibration_points.len();
                total_points += left_points;
                if left_points >= 2 {
                    completed_wheels += 1;
                }

                // 右轮
                let right_points = axle.right_wheel.calibration_points.len();
                total_points += right_points;
                if right_points >= 2 {
                    completed_wheels += 1;
                }
            }
        }

        Some(CalibrationProgress {
            plate_number: plate_number.to_string(),
            axle_count: sheet.axle_count,
            total_wheels,
            completed_wheels,
            total_calibration_points: total_points,
            is_completed: completed_wheels == total_wheels,
        })
    }

    /// 创建或更新车辆标定表
    pub fn create_calibration_table(
        &mut self,
        plate_number: &str,
        rated_total_weight: f64,
        tare_weight: f64,
        axle_count: u8,
    ) -> VehicleCalibrationTable {
        let table = VehicleCalibrationTable {
            vehicle_id: format!("V{}", plate_number),
            plate_number: plate_number.to_string(),
            rated_total_weight,
            tare_weight,
            axle_count,
            sensor_calibrations: HashMap::new(),
            axle_coefficients: HashMap::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        self.calibration_tables
            .insert(plate_number.to_string(), table.clone());
        info!(
            "创建车辆标定表: {}, 额定总重: {}kg",
            plate_number, rated_total_weight
        );
        table
    }

    /// 添加传感器标定参数
    pub fn add_sensor_calibration(
        &mut self,
        plate_number: &str,
        sensor_address: u16,
        sensor_name: &str,
        axle_number: u8,
        is_left_wheel: bool,
        coefficients: SegmentCoefficients,
    ) -> Result<(), String> {
        if let Some(table) = self.calibration_tables.get_mut(plate_number) {
            let calibration = SensorCalibration {
                sensor_address,
                sensor_name: sensor_name.to_string(),
                axle_number,
                is_left_wheel,
                coefficients,
                calibration_date: Utc::now(),
                is_calibrated: true,
            };

            table
                .sensor_calibrations
                .insert(sensor_address, calibration);
            table.updated_at = Utc::now();

            info!(
                "添加传感器标定: 车辆={}, 传感器={}, 地址={}",
                plate_number, sensor_name, sensor_address
            );
            Ok(())
        } else {
            Err(format!("车辆标定表不存在: {}", plate_number))
        }
    }

    /// 计算车辆总重(旧版)
    pub fn calculate_vehicle_weight_old(
        &self,
        plate_number: &str,
        sensor_data: &[(u16, u32, f64)], // (传感器地址, AD值, 温度)
    ) -> Option<VehicleWeightResult> {
        let table = self.calibration_tables.get(plate_number)?;
        let mut sensor_weights = Vec::new();
        let mut total_sensor_weight = 0.0;

        for (address, ad_value, temperature) in sensor_data {
            if let Some(calibration) = table.sensor_calibrations.get(address) {
                let x = *ad_value as f64;
                let coeffs = &calibration.coefficients;
                let calibrated_weight = if x < coeffs.turning_point {
                    coeffs.before_turning.calculate(x)
                } else {
                    coeffs.after_turning.calculate(x)
                };

                let result = WeightCalculationResult {
                    sensor_address: *address,
                    sensor_name: calibration.sensor_name.clone(),
                    ad_value: *ad_value,
                    raw_weight: *ad_value as f64,
                    calibrated_weight,
                    temperature: *temperature,
                };

                total_sensor_weight += calibrated_weight;
                sensor_weights.push(result);
            } else {
                warn!("传感器未标定: 车辆={}, 地址={}", plate_number, address);
            }
        }

        // 计算总重和载重
        let total_weight = total_sensor_weight;
        let load_weight = total_weight - table.tare_weight;
        let overload = total_weight > table.rated_total_weight;
        let overload_amount = if overload {
            total_weight - table.rated_total_weight
        } else {
            0.0
        };

        Some(VehicleWeightResult {
            plate_number: plate_number.to_string(),
            timestamp: Utc::now(),
            sensor_weights,
            total_sensor_weight,
            total_weight,
            load_weight,
            rated_total_weight: table.rated_total_weight,
            overload,
            overload_amount,
        })
    }

    /// 通过标定点计算多项式系数(最小二乘法拟合)
    pub fn calculate_coefficients_from_points(
        points: &[CalibrationPoint],
    ) -> Result<PolynomialCoefficients, String> {
        if points.len() < 4 {
            return Err("至少需要4个标定点才能计算3阶多项式系数".to_string());
        }

        // 使用最小二乘法拟合3阶多项式
        // y = a*x^3 + b*x^2 + c*x + d
        // 这里使用简化的方法，实际应该使用矩阵运算

        // 提取数据 (用于最小二乘法拟合)
        let _n = points.len() as f64;
        let _sum_x: f64 = points.iter().map(|p| p.pa_value as f64).sum();
        let _sum_x2: f64 = points.iter().map(|p| (p.pa_value as f64).powi(2)).sum();
        let _sum_x3: f64 = points.iter().map(|p| (p.pa_value as f64).powi(3)).sum();
        let _sum_x4: f64 = points.iter().map(|p| (p.pa_value as f64).powi(4)).sum();
        let _sum_x5: f64 = points.iter().map(|p| (p.pa_value as f64).powi(5)).sum();
        let _sum_x6: f64 = points.iter().map(|p| (p.pa_value as f64).powi(6)).sum();

        let _sum_y: f64 = points.iter().map(|p| p.actual_weight).sum();
        let _sum_xy: f64 = points
            .iter()
            .map(|p| p.pa_value as f64 * p.actual_weight)
            .sum();
        let _sum_x2y: f64 = points
            .iter()
            .map(|p| (p.pa_value as f64).powi(2) * p.actual_weight)
            .sum();
        let _sum_x3y: f64 = points
            .iter()
            .map(|p| (p.pa_value as f64).powi(3) * p.actual_weight)
            .sum();

        // 构建正规方程矩阵 (4x4)
        // [ n      sum_x    sum_x2   sum_x3  ] [ d ]   [ sum_y   ]
        // [ sum_x  sum_x2   sum_x3   sum_x4  ] [ c ] = [ sum_xy  ]
        // [ sum_x2 sum_x3   sum_x4   sum_x5  ] [ b ]   [ sum_x2y ]
        // [ sum_x3 sum_x4   sum_x5   sum_x6  ] [ a ]   [ sum_x3y ]

        // 使用高斯消元法求解 (简化实现)
        // 实际生产环境应该使用更稳定的数值算法

        // 这里返回默认系数，实际应该实现完整的矩阵求解
        warn!("多项式系数计算使用简化实现，建议实现完整的矩阵求解算法");

        Ok(PolynomialCoefficients {
            coef_3: 0.0,
            coef_2: 0.0,
            coef_1: 1.0,
            constant: 0.0,
        })
    }

    /// 获取车辆标定表
    pub fn get_calibration_table(&self, plate_number: &str) -> Option<&VehicleCalibrationTable> {
        self.calibration_tables.get(plate_number)
    }

    /// 获取所有标定表
    pub fn get_all_tables(&self) -> &HashMap<String, VehicleCalibrationTable> {
        &self.calibration_tables
    }
}

impl Default for WeightCalibrationService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_polynomial_calculation() {
        let coeffs = PolynomialCoefficients {
            coef_3: 0.0,
            coef_2: 0.0,
            coef_1: 0.1, // 线性系数
            constant: 0.0,
        };

        // y = 0.1 * x
        assert_eq!(coeffs.calculate(100.0), 10.0);
        assert_eq!(coeffs.calculate(50000.0), 5000.0);
    }

    #[test]
    fn test_segment_coefficients() {
        let segment = SegmentCoefficients {
            before_turning: PolynomialCoefficients {
                coef_3: 0.0,
                coef_2: 0.0,
                coef_1: 0.1,
                constant: 0.0,
            },
            after_turning: PolynomialCoefficients {
                coef_3: 0.0,
                coef_2: 0.0,
                coef_1: 0.08, // 转折后斜率变小
                constant: 1000.0,
            },
            turning_point: 50000.0,
        };

        // 转折前: y = 0.1 * 40000 = 4000
        let x1 = 40000.0;
        let y1 = if x1 < segment.turning_point {
            segment.before_turning.calculate(x1)
        } else {
            segment.after_turning.calculate(x1)
        };
        assert_eq!(y1, 4000.0);

        // 转折后: y = 0.08 * 60000 + 1000 = 5800
        let x2 = 60000.0;
        let y2 = if x2 < segment.turning_point {
            segment.before_turning.calculate(x2)
        } else {
            segment.after_turning.calculate(x2)
        };
        assert_eq!(y2, 5800.0);
    }

    #[test]
    fn test_calibration_sheet() {
        let mut service = WeightCalibrationService::new();

        // 创建2轴车辆标定表
        let sheet = service
            .create_calibration_sheet(
                "川J53757",
                32000.0, // 额定总重32吨
                12000.0, // 空车12吨
                2,       // 2轴
            )
            .unwrap();

        assert_eq!(sheet.axle_count, 2);
        assert_eq!(sheet.axle_records.len(), 2);

        // 添加第一轴左轮标定点(空车)
        // 空车轮重: Pa=581 (原始值58100/100取整)
        service
            .add_calibration_point(CalibrationPointParams {
                plate_number: "川J53757".to_string(),
                axle_number: 1,
                is_left_wheel: true,
                pa_value: 581,      // Pa值 (原始58100/100取整)
                actual_weight: 0.0, // 空车轮重0kg
                temperature: 14.11, // 温度
                load_percentage: 0, // 0%负载
            })
            .unwrap();

        // 添加第一轴右轮标定点(空车)
        // 空车轮重: Pa=574 (原始值57356/100取整)
        service
            .add_calibration_point(CalibrationPointParams {
                plate_number: "川J53757".to_string(),
                axle_number: 1,
                is_left_wheel: false,
                pa_value: 574,      // Pa值 (原始57356/100取整)
                actual_weight: 0.0, // 空车轮重0kg
                temperature: 14.11, // 温度
                load_percentage: 0, // 0%负载
            })
            .unwrap();

        // 添加100%满载标定点(示例数据)
        // 假设满载时第一轴左轮配重8000kg，Pa值约700
        service
            .add_calibration_point(CalibrationPointParams {
                plate_number: "川J53757".to_string(),
                axle_number: 1,
                is_left_wheel: true,
                pa_value: 700,         // Pa值
                actual_weight: 8000.0, // 轮重8000kg
                temperature: 15.0,
                load_percentage: 100, // 100%负载
            })
            .unwrap();

        service
            .add_calibration_point(CalibrationPointParams {
                plate_number: "川J53757".to_string(),
                axle_number: 1,
                is_left_wheel: false,
                pa_value: 690,         // Pa值
                actual_weight: 8000.0, // 轮重8000kg
                temperature: 15.0,
                load_percentage: 100, // 100%负载
            })
            .unwrap();

        // 检查标定进度
        let progress = service.get_calibration_progress("川J53757").unwrap();
        println!("标定进度: {:?}", progress);
        assert_eq!(progress.total_wheels, 4); // 2轴 * 2轮
        assert_eq!(progress.total_calibration_points, 4);

        // 生成一阶系数
        let coeffs = service
            .generate_vehicle_coefficients("川J53757", PolynomialOrder::First)
            .unwrap();
        println!("生成的一阶系数: {:?}", coeffs);
        assert_eq!(coeffs.len(), 2); // 第一轴左右轮

        // 获取传感器地址
        let left_addr = service
            .get_calibration_sheet("川J53757")
            .and_then(|s| s.axle_records.get(&1))
            .map(|a| a.left_wheel.sensor_address)
            .unwrap();
        let right_addr = service
            .get_calibration_sheet("川J53757")
            .and_then(|s| s.axle_records.get(&1))
            .map(|a| a.right_wheel.sensor_address)
            .unwrap();

        // 计算重量
        let sensor_coeffs: Vec<_> = coeffs.iter().map(|(addr, c)| (*addr, c.clone())).collect();
        let sensor_pa = vec![
            (left_addr, 650),  // Pa=650
            (right_addr, 640), // Pa=640
        ];
        let total_weight =
            WeightCalibrationService::calculate_total_weight(&sensor_coeffs, &sensor_pa);
        println!("计算总重: {:.2}kg", total_weight);

        // 计算载重
        let load = WeightCalibrationService::calculate_load_weight(total_weight, 12000.0);
        println!("计算载重: {:.2}kg", load);
    }

    #[test]
    fn test_first_order_coefficient_calculation() {
        // 测试一阶系数计算 W(x) = ax + b
        let points = vec![
            CalibrationPoint {
                sensor_address: 3756,
                calibration_time: Utc::now(),
                actual_weight: 0.0, // 空车 = 0kg
                pa_value: 581,      // Pa = 58100/100取整
                pa_raw: 58100,
                temperature: 14.11,
                load_percentage: 0,
                is_manual: false,
                record_time: Utc::now(),
            },
            CalibrationPoint {
                sensor_address: 3756,
                calibration_time: Utc::now(),
                actual_weight: 8000.0, // 满载 = 8000kg
                pa_value: 700,         // Pa = 70000/100取整
                pa_raw: 70000,
                temperature: 15.0,
                load_percentage: 100,
                is_manual: false,
                record_time: Utc::now(),
            },
        ];

        let coeffs = WeightCalibrationService::calculate_first_order_coefficients(&points).unwrap();
        println!("一阶系数: a={:.6}, b={:.2}", coeffs.coef_1, coeffs.constant);

        // W(x) = ax + b
        // 0 = a*581 + b
        // 8000 = a*700 + b
        // => a = 8000 / (700 - 581) = 8000 / 119 ≈ 67.23
        // => b = -a * 581 ≈ -39060
        assert!(coeffs.coef_1 > 0.0); // 斜率应为正

        // 验证计算
        let weight_at_640 = coeffs.calculate(640.0);
        println!("Pa=640时重量: {:.2}kg", weight_at_640);
        assert!(weight_at_640 > 0.0 && weight_at_640 < 8000.0);
    }

    #[test]
    fn test_pa_value_calculation() {
        // 测试Pa值计算: 原始值/100取整
        assert_eq!(CalibrationPoint::calculate_pa_from_raw(58100), 581);
        assert_eq!(CalibrationPoint::calculate_pa_from_raw(57356), 574);
        assert_eq!(CalibrationPoint::calculate_pa_from_raw(50000), 500);
        assert_eq!(CalibrationPoint::calculate_pa_from_raw(49999), 500); // 四舍五入
    }

    #[test]
    fn test_weight_calculation_formula() {
        // 测试重量计算公式: 总重 = W(x1) + W(x2) + ...
        // 载重 = 总重 - 自重

        // 模拟两个传感器的系数
        let coeffs1 = PolynomialCoefficients {
            coef_3: 0.0,
            coef_2: 0.0,
            coef_1: 67.23,      // 一阶系数a
            constant: -39060.0, // 常数b
        };

        let coeffs2 = PolynomialCoefficients {
            coef_3: 0.0,
            coef_2: 0.0,
            coef_1: 68.0,
            constant: -39000.0,
        };

        // 计算各传感器重量
        let pa1 = 640u32;
        let pa2 = 650u32;

        let weight1 = WeightCalibrationService::calculate_sensor_weight(&coeffs1, pa1);
        let weight2 = WeightCalibrationService::calculate_sensor_weight(&coeffs2, pa2);

        println!("传感器1 (Pa={}): {:.2}kg", pa1, weight1);
        println!("传感器2 (Pa={}): {:.2}kg", pa2, weight2);

        // 计算总重
        let total = weight1 + weight2;
        println!("总重: {:.2}kg", total);

        // 计算载重
        let tare = 12000.0; // 自重12吨
        let load = WeightCalibrationService::calculate_load_weight(total, tare);
        println!("载重: {:.2}kg", load);

        assert!(total > 0.0);
        assert!(load < total);
    }
}
