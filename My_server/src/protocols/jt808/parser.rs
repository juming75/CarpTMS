//! / JT808协议解析器
// 专门用于解析车载终端上传的GPS+传感器融合数据

use super::models::*;
use chrono::{DateTime, TimeZone, Utc};
use log::{debug, error};

/// 传感器类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SensorType {
    /// 液压传感器(如SJ-1000)
    /// - 精度:2位小数
    /// - 压力值需要 /100 转换为实际值
    Hydraulic,

    /// 电感传感器
    /// - 精度:根据厂家不同(通常1-3位小数)
    Inductive,

    /// 应变片传感器
    /// - 精度:根据厂家不同(通常1-3位小数)
    StrainGauge,

    /// 其他类型
    Other,
}

impl SensorType {
    /// 获取传感器类型的精度(小数位数)
    pub fn precision(&self) -> u32 {
        match self {
            SensorType::Hydraulic => 2,   // 2位小数,如SJ-1000
            SensorType::Inductive => 1,   // 默认1位小数
            SensorType::StrainGauge => 1, // 默认1位小数
            SensorType::Other => 2,       // 默认2位小数
        }
    }

    /// 获取除数(用于将原始值转换为实际值)
    /// 例如:2位小数 -> /100,1位小数 -> /10
    pub fn divisor(&self) -> f64 {
        10f64.powi(self.precision() as i32)
    }

    /// 获取传感器类型名称
    pub fn name(&self) -> &'static str {
        match self {
            SensorType::Hydraulic => "液压传感器",
            SensorType::Inductive => "电感传感器",
            SensorType::StrainGauge => "应变片传感器",
            SensorType::Other => "其他传感器",
        }
    }
}

/// 传感器标定参数(每个传感器独立标定)
#[derive(Debug, Clone)]
pub struct SensorCalibration {
    pub address: u16,                            // 传感器编号
    pub sensor_type: SensorType,                 // 传感器类型
    pub pressure_coefficient: f64,               // 压力系数
    pub initial_value: f64,                      // 初始值/零点值
    pub calibration_date: Option<DateTime<Utc>>, // 标定日期
}

impl SensorCalibration {
    /// 创建液压传感器校准参数(如SJ-1000)
    pub fn hydraulic(address: u16, coefficient: f64, initial_value: f64) -> Self {
        Self {
            address,
            sensor_type: SensorType::Hydraulic,
            pressure_coefficient: coefficient,
            initial_value,
            calibration_date: Some(Utc::now()),
        }
    }

    /// 创建电感传感器校准参数
    pub fn inductive(address: u16, coefficient: f64, initial_value: f64) -> Self {
        Self {
            address,
            sensor_type: SensorType::Inductive,
            pressure_coefficient: coefficient,
            initial_value,
            calibration_date: Some(Utc::now()),
        }
    }

    /// 创建应变片传感器校准参数
    pub fn strain_gauge(address: u16, coefficient: f64, initial_value: f64) -> Self {
        Self {
            address,
            sensor_type: SensorType::StrainGauge,
            pressure_coefficient: coefficient,
            initial_value,
            calibration_date: Some(Utc::now()),
        }
    }

    /// 计算实际重量(根据传感器类型自动处理精度)
    ///
    /// 公式: 重量 = (P / 精度除数) * 压力系数 + 初始值
    ///
    /// 例如:
    /// - SJ-1000液压传感器:P=62324,精度2位 -> 623.24
    /// - 电感传感器:P=62324,精度1位 -> 6232.4
    pub fn calculate_weight(&self, pressure: u32) -> f64 {
        let divisor = self.sensor_type.divisor();
        (pressure as f64 / divisor) * self.pressure_coefficient + self.initial_value
    }

    /// 将原始压力值转换为实际压力值(根据传感器精度)
    ///
    /// 例如:
    /// - SJ-1000液压传感器:62324 -> 623.24
    /// - 电感传感器:62324 -> 6232.4
    pub fn raw_to_pressure(&self, raw_pressure: u32) -> f64 {
        raw_pressure as f64 / self.sensor_type.divisor()
    }
}

/// 板簧重量计算模型
///
/// 理论基础:
/// - 单层板簧:y(x) = ax + b,其中x为弹簧形变(等同于传感器压力)
/// - 多层板簧:带转折点的一阶函数
/// - 老化板簧:二阶或三阶函数
///
/// 注意:此模型支持不同类型的传感器(液压/电感/应变片),
/// 通过sensor_type自动处理不同的精度要求
#[derive(Debug, Clone)]
pub enum LeafSpringModel {
    /// 线性模型(单层板簧)
    /// 公式: 重量 = (压力 - 零点) / 精度除数 × 系数 + 常数
    Linear {
        sensor_type: SensorType, // 传感器类型(决定精度)
        zero_point: f64,         // 零点压力值
        coefficient: f64,        // 校准系数
        constant: f64,           // 压力常数
    },

    /// 分段线性模型(多层板簧,带转折点)
    /// 公式: 重量 = (压力 - 零点) / 精度除数 × 系数 + 常数(根据压力范围选择不同系数)
    PiecewiseLinear {
        sensor_type: SensorType,      // 传感器类型(决定精度)
        zero_point: f64,              // 零点压力值
        segments: Vec<LinearSegment>, // 分段线性区间
    },

    /// 二阶多项式模型(老化板簧)
    /// 公式: 重量 = a×x² + b×x + c,其中x = (压力-零点)/精度除数
    Quadratic {
        sensor_type: SensorType, // 传感器类型(决定精度)
        zero_point: f64,         // 零点压力值
        a: f64,                  // 二次项系数
        b: f64,                  // 一次项系数
        c: f64,                  // 常数项
    },

    /// 三阶多项式模型(严重老化板簧)
    /// 公式: 重量 = a×x³ + b×x² + c×x + d,其中x = (压力-零点)/精度除数
    Cubic {
        sensor_type: SensorType, // 传感器类型(决定精度)
        zero_point: f64,         // 零点压力值
        a: f64,                  // 三次项系数
        b: f64,                  // 二次项系数
        c: f64,                  // 一次项系数
        d: f64,                  // 常数项
    },
}

/// 线性分段区间
#[derive(Debug, Clone)]
pub struct LinearSegment {
    pub pressure_min: f64, // 压力范围最小值
    pub pressure_max: f64, // 压力范围最大值
    pub coefficient: f64,  // 该区间校准系数
    pub constant: f64,     // 该区间常数
}

impl LeafSpringModel {
    /// 创建单层板簧线性模型(液压传感器SJ-1000,2位小数)
    pub fn linear_hydraulic(zero_point: f64, coefficient: f64, constant: f64) -> Self {
        LeafSpringModel::Linear {
            sensor_type: SensorType::Hydraulic,
            zero_point,
            coefficient,
            constant,
        }
    }

    /// 创建单层板簧线性模型(电感传感器,1位小数)
    pub fn linear_inductive(zero_point: f64, coefficient: f64, constant: f64) -> Self {
        LeafSpringModel::Linear {
            sensor_type: SensorType::Inductive,
            zero_point,
            coefficient,
            constant,
        }
    }

    /// 创建单层板簧线性模型(应变片传感器,1位小数)
    pub fn linear_strain_gauge(zero_point: f64, coefficient: f64, constant: f64) -> Self {
        LeafSpringModel::Linear {
            sensor_type: SensorType::StrainGauge,
            zero_point,
            coefficient,
            constant,
        }
    }

    /// 创建单层板簧线性模型(指定传感器类型)
    pub fn linear(
        sensor_type: SensorType,
        zero_point: f64,
        coefficient: f64,
        constant: f64,
    ) -> Self {
        LeafSpringModel::Linear {
            sensor_type,
            zero_point,
            coefficient,
            constant,
        }
    }

    /// 创建多层板簧分段线性模型
    pub fn piecewise_linear(
        sensor_type: SensorType,
        zero_point: f64,
        segments: Vec<LinearSegment>,
    ) -> Self {
        LeafSpringModel::PiecewiseLinear {
            sensor_type,
            zero_point,
            segments,
        }
    }

    /// 创建老化板簧二阶模型
    pub fn quadratic(sensor_type: SensorType, zero_point: f64, a: f64, b: f64, c: f64) -> Self {
        LeafSpringModel::Quadratic {
            sensor_type,
            zero_point,
            a,
            b,
            c,
        }
    }

    /// 创建严重老化板簧三阶模型
    pub fn cubic(sensor_type: SensorType, zero_point: f64, a: f64, b: f64, c: f64, d: f64) -> Self {
        LeafSpringModel::Cubic {
            sensor_type,
            zero_point,
            a,
            b,
            c,
            d,
        }
    }

    /// 获取传感器类型
    pub fn sensor_type(&self) -> SensorType {
        match self {
            LeafSpringModel::Linear { sensor_type, .. } => *sensor_type,
            LeafSpringModel::PiecewiseLinear { sensor_type, .. } => *sensor_type,
            LeafSpringModel::Quadratic { sensor_type, .. } => *sensor_type,
            LeafSpringModel::Cubic { sensor_type, .. } => *sensor_type,
        }
    }

    /// 计算重量
    ///
    /// 参数:
    /// - pressure: 传感器压力原始值
    ///
    /// 返回:
    /// - 计算后的重量(kg)
    ///
    /// 注意:根据传感器类型自动处理精度
    /// - 液压传感器(SJ-1000):/100(2位小数)
    /// - 电感传感器:/10(1位小数)
    /// - 应变片传感器:/10(1位小数)
    pub fn calculate_weight(&self, pressure: u32) -> f64 {
        let pressure_f = pressure as f64;

        match self {
            LeafSpringModel::Linear {
                sensor_type,
                zero_point,
                coefficient,
                constant,
            } => {
                // 单层板簧线性模型
                // 重量 = (压力 - 零点) / 精度除数 × 系数 + 常数
                let x = (pressure_f - zero_point) / sensor_type.divisor();
                x * coefficient + constant
            }

            LeafSpringModel::PiecewiseLinear {
                sensor_type,
                zero_point,
                segments,
            } => {
                // 多层板簧分段线性模型
                let x = (pressure_f - zero_point) / sensor_type.divisor();

                // 找到对应的分段
                for segment in segments {
                    if x >= segment.pressure_min && x <= segment.pressure_max {
                        return x * segment.coefficient + segment.constant;
                    }
                }

                // 如果没有找到对应分段,使用最后一个分段
                if let Some(last) = segments.last() {
                    x * last.coefficient + last.constant
                } else {
                    0.0
                }
            }

            LeafSpringModel::Quadratic {
                sensor_type,
                zero_point,
                a,
                b,
                c,
            } => {
                // 老化板簧二阶模型
                // 重量 = a×x² + b×x + c,其中x = (压力-零点)/精度除数
                let x = (pressure_f - zero_point) / sensor_type.divisor();
                a * x * x + b * x + c
            }

            LeafSpringModel::Cubic {
                sensor_type,
                zero_point,
                a,
                b,
                c,
                d,
            } => {
                // 严重老化板簧三阶模型
                // 重量 = a×x³ + b×x² + c×x + d,其中x = (压力-零点)/精度除数
                let x = (pressure_f - zero_point) / sensor_type.divisor();
                a * x * x * x + b * x * x + c * x + d
            }
        }
    }

    /// 获取模型类型描述
    pub fn model_type(&self) -> &'static str {
        match self {
            LeafSpringModel::Linear { .. } => "单层板簧线性模型",
            LeafSpringModel::PiecewiseLinear { .. } => "多层板簧分段线性模型",
            LeafSpringModel::Quadratic { .. } => "老化板簧二阶模型",
            LeafSpringModel::Cubic { .. } => "严重老化板簧三阶模型",
        }
    }
}

/// 车辆重量校准配置(包含所有传感器的校准参数)
#[derive(Debug, Clone)]
pub struct VehicleWeightCalibration {
    pub vehicle_id: String,                        // 车辆ID/车牌号
    pub plate_number: String,                      // 车牌号
    pub sensor_models: Vec<(u8, LeafSpringModel)>, // 传感器校准模型列表 (传感器地址, 模型)
    pub total_weight_formula: TotalWeightFormula,  // 总重量计算公式
    pub calibration_date: DateTime<Utc>,           // 校准日期
    pub valid_until: Option<DateTime<Utc>>,        // 有效期至
}

/// 总重量计算公式
#[derive(Debug, Clone)]
pub enum TotalWeightFormula {
    /// 简单求和
    Sum,
    /// 加权平均
    WeightedAverage(Vec<f64>), // 各传感器的权重
    /// 自定义公式
    Custom(fn(&[f64]) -> f64),
}

impl VehicleWeightCalibration {
    /// 计算单颗传感器的重量
    pub fn calculate_sensor_weight(&self, sensor_address: u8, pressure: u32) -> Option<f64> {
        for (addr, model) in &self.sensor_models {
            if *addr == sensor_address {
                return Some(model.calculate_weight(pressure));
            }
        }
        None
    }

    /// 计算车辆总重量
    pub fn calculate_total_weight(&self, sensor_pressures: &[(u8, u32)]) -> f64 {
        let mut weights = Vec::new();

        for (addr, pressure) in sensor_pressures {
            if let Some(weight) = self.calculate_sensor_weight(*addr, *pressure) {
                weights.push(weight);
            }
        }

        match &self.total_weight_formula {
            TotalWeightFormula::Sum => weights.iter().sum(),
            TotalWeightFormula::WeightedAverage(coefficients) => {
                if weights.len() == coefficients.len() {
                    weights
                        .iter()
                        .zip(coefficients.iter())
                        .map(|(w, c)| w * c)
                        .sum()
                } else {
                    weights.iter().sum()
                }
            }
            TotalWeightFormula::Custom(func) => func(&weights),
        }
    }
}

/// 称重传感器文本数据(格式: Addr=3755,T=1926,P=64390)
///
/// 字段说明:
/// - address: 传感器编号/地址
/// - temperature: 传感器温度,单位0.01度(如1411表示14.11度)
/// - pressure: 传感器压力值(AD值),非直接重量
///
/// 重量计算公式(待标定):
/// 重量 = (P / 100) * 压力系数 + 初始值
/// 其中压力系数和初始值需要通过标定计算得到
#[derive(Debug, Clone)]
pub struct WeightTextSensor {
    pub address: u16,     // 传感器编号
    pub temperature: u16, // 温度值(单位0.01度)
    pub pressure: u32,    // 压力值(AD值,需转换)
}

/// 称重传感器数据结构1(18字节,川J53757格式)
///
/// 数据格式:
/// - 字节0: 传感器序号
/// - 字节1-3: 压力原始值 (3字节大端)
/// - 字节4-6: 温度值 (3字节大端,单位0.01℃)
/// - 字节7-8: X轴加速度 (mg)
/// - 字节9-10: Y轴加速度 (mg)
/// - 字节11-12: Z轴加速度 (mg)
/// - 字节13-14: 水平倾斜角 (0.01度)
/// - 字节15-16: 垂直倾斜角 (0.01度)
/// - 字节17: 分隔符 ';' (0x3B)
#[derive(Debug, Clone)]
pub struct WeightSensorV2 {
    pub address: u8,          // 传感器序号 (2-7)
    pub pressure: u32,        // 压力原始值 (3字节大端)
    pub temperature: u32,     // 温度值 (3字节大端,单位0.01℃)
    pub accel_x: u16,         // X轴加速度 (mg)
    pub accel_y: u16,         // Y轴加速度 (mg)
    pub accel_z: u16,         // Z轴加速度 (mg)
    pub tilt_horizontal: u16, // 水平倾斜角 (0.01度)
    pub tilt_vertical: u16,   // 垂直倾斜角 (0.01度)
}

/// 称重扩展信息(0xE8)
#[derive(Debug, Clone)]
pub struct WeightExtension {
    pub sensor_positions: u16, // 传感器位置位图
    pub received_sensors: u16, // 接收到的传感器位图
}

pub struct JT808Parser;

impl Clone for JT808Parser {
    fn clone(&self) -> Self {
        Self
    }
}

impl JT808Parser {
    /// 解析JT808协议帧
    pub fn parse_frame(data: &[u8]) -> Result<JT808Frame, ParseError> {
        debug!("Parsing JT808 frame, data length: {}", data.len());

        // 去除转义字符
        let unescaped = Self::unescape(data)?;

        // 验证起始和结束标识
        if unescaped.is_empty() || unescaped[0] != 0x7E || unescaped.last() != Some(&0x7E) {
            return Err(ParseError::InvalidFrameHeader);
        }

        // 最小帧长度: 起始符(1) + 消息ID(2) + 属性(2) + 手机号(6) + 流水号(2) + 校验(1) + 结束符(1) = 15
        if unescaped.len() < 15 {
            return Err(ParseError::InvalidLength);
        }

        // 解析帧头
        let msg_id = u16::from_be_bytes([unescaped[1], unescaped[2]]);
        let msg_attr = u16::from_be_bytes([unescaped[3], unescaped[4]]);
        let phone = Self::parse_bcd_phone(&unescaped[5..11]);
        let flow_no = u16::from_be_bytes([unescaped[11], unescaped[12]]);
        let body_len = (msg_attr & 0x03FF) as usize;

        debug!("Message ID: 0x{:04X}, Body length: {}", msg_id, body_len);

        // 解析消息体
        let body_start = 12;
        let body_end = body_start + body_len;

        if body_end + 2 > unescaped.len() {
            return Err(ParseError::InvalidLength);
        }

        let body = &unescaped[body_start..body_end];

        // 验证校验码
        let crc_pos = body_end;
        let checksum = unescaped[crc_pos];

        if !Self::verify_checksum(&unescaped[1..crc_pos], checksum) {
            error!("Checksum verification failed");
            return Err(ParseError::ChecksumError);
        }

        Ok(JT808Frame {
            msg_id,
            msg_attr,
            phone,
            flow_no,
            body: body.to_vec(),
            checksum,
        })
    }

    /// 解析消息体(根据消息ID分发)
    pub fn parse_body(msg_id: u16, body: &[u8]) -> Result<LocationReport, ParseError> {
        match msg_id {
            0x0200 => Self::parse_0x0200(body),
            0x0201 => Self::parse_0x0201(body),
            0x0704 => Self::parse_0x0704(body),
            0x1201 => Self::parse_0x1201(body),
            _ => {
                debug!("Unsupported message ID: 0x{:04X}", msg_id);
                Err(ParseError::UnknownCommand)
            }
        }
    }

    /// 解析0x0201位置数据批量上报
    pub fn parse_0x0201(body: &[u8]) -> Result<LocationReport, ParseError> {
        debug!("Parsing 0x0201 batch location report");

        // 简化实现:只取第一个位置数据
        if body.len() < 28 {
            return Err(ParseError::InvalidLength);
        }

        // 前几个字节与0x0200格式相同
        Self::parse_0x0200(body)
    }

    /// 解析0x0704数据上传应答
    pub fn parse_0x0704(body: &[u8]) -> Result<LocationReport, ParseError> {
        debug!("Parsing 0x0704 data upload response");

        if body.len() < 3 {
            return Err(ParseError::InvalidLength);
        }

        let data_type = u16::from_be_bytes([body[0], body[1]]);
        let result = body[2];

        debug!(
            "Data upload response: type=0x{:04X}, result={}",
            data_type, result
        );

        // 返回一个简化的 LocationReport
        Ok(LocationReport {
            alarm_flag: 0,
            status: 0,
            latitude: 0.0,
            longitude: 0.0,
            altitude: 0.0,
            speed: 0.0,
            direction: 0.0,
            timestamp: chrono::Utc::now(),
            sensor_data: SensorData::new(),
        })
    }

    /// 解析0x1201报警附件上传消息
    pub fn parse_0x1201(body: &[u8]) -> Result<LocationReport, ParseError> {
        debug!("Parsing 0x1201 alarm upload");

        if body.len() < 30 {
            return Err(ParseError::InvalidLength);
        }

        let mut offset = 0;

        // 报警ID (4字节)
        let alarm_id = u32::from_be_bytes([
            body[offset],
            body[offset + 1],
            body[offset + 2],
            body[offset + 3],
        ]);
        offset += 4;

        // 报警标志 (4字节)
        let alarm_flag = u32::from_be_bytes([
            body[offset],
            body[offset + 1],
            body[offset + 2],
            body[offset + 3],
        ]);
        offset += 4;

        // 状态 (4字节)
        let status = u32::from_be_bytes([
            body[offset],
            body[offset + 1],
            body[offset + 2],
            body[offset + 3],
        ]);
        offset += 4;

        // 纬度 (4字节)
        let latitude = i32::from_be_bytes([
            body[offset],
            body[offset + 1],
            body[offset + 2],
            body[offset + 3],
        ]) as f64
            / 1_000_000.0;
        offset += 4;

        // 经度 (4字节)
        let longitude = i32::from_be_bytes([
            body[offset],
            body[offset + 1],
            body[offset + 2],
            body[offset + 3],
        ]) as f64
            / 1_000_000.0;
        offset += 4;

        // 时间 (6字节 BCD)
        let time_str = Self::bcd_to_string(&body[offset..offset + 6]);
        let timestamp = Self::parse_bcd_time(&time_str)?;

        debug!(
            "Alarm {}: lat={:.6}, lon={:.6}, flag=0x{:08X}",
            alarm_id, latitude, longitude, alarm_flag
        );

        Ok(LocationReport {
            alarm_flag,
            status,
            latitude,
            longitude,
            altitude: 0.0,
            speed: 0.0,
            direction: 0.0,
            timestamp,
            sensor_data: SensorData::new(),
        })
    }

    /// 解析0x0200位置信息汇报(核心方法)
    pub fn parse_0x0200(body: &[u8]) -> Result<LocationReport, ParseError> {
        debug!(
            "Parsing 0x0200 location report, body length: {}",
            body.len()
        );

        // 最小长度: 报警(4) + 状态(4) + 纬度(4) + 经度(4) + 高程(2) + 速度(2) + 方向(2) + 时间(6) = 28
        if body.len() < 28 {
            return Err(ParseError::InvalidLength);
        }

        let mut offset = 0;

        // 报警标志 (4字节)
        let alarm_flag = u32::from_be_bytes([
            body[offset],
            body[offset + 1],
            body[offset + 2],
            body[offset + 3],
        ]);
        offset += 4;

        // 状态 (4字节)
        let status = u32::from_be_bytes([
            body[offset],
            body[offset + 1],
            body[offset + 2],
            body[offset + 3],
        ]);
        offset += 4;

        // 纬度 (4字节, 1/1000000度)
        let latitude = i32::from_be_bytes([
            body[offset],
            body[offset + 1],
            body[offset + 2],
            body[offset + 3],
        ]) as f64
            / 1_000_000.0;
        offset += 4;

        // 经度 (4字节, 1/1000000度)
        let longitude = i32::from_be_bytes([
            body[offset],
            body[offset + 1],
            body[offset + 2],
            body[offset + 3],
        ]) as f64
            / 1_000_000.0;
        offset += 4;

        // 高程 (2字节, 米)
        let altitude = u16::from_be_bytes([body[offset], body[offset + 1]]) as f64;
        offset += 2;

        // 速度 (2字节, 1/10km/h)
        let speed = u16::from_be_bytes([body[offset], body[offset + 1]]) as f64 / 10.0;
        offset += 2;

        // 方向 (2字节, 0-360度)
        let direction = u16::from_be_bytes([body[offset], body[offset + 1]]) as f64;
        offset += 2;

        // 时间 (BCD码 6字节)
        let time_str = Self::bcd_to_string(&body[offset..offset + 6]);
        offset += 6;
        let timestamp = Self::parse_bcd_time(&time_str)?;

        debug!(
            "GPS: lat={:.6}, lon={:.6}, speed={:.1}",
            latitude, longitude, speed
        );

        // 解析附加数据(传感器数据)
        let sensor_data = Self::parse_additional_data(&body[offset..])?;

        debug!(
            "Parsed {} sensor data items",
            sensor_data.analog_inputs.len()
        );

        Ok(LocationReport {
            alarm_flag,
            status,
            latitude,
            longitude,
            altitude,
            speed,
            direction,
            timestamp,
            sensor_data,
        })
    }

    /// 解析附加数据(传感器数据核心)
    pub fn parse_additional_data(data: &[u8]) -> Result<SensorData, ParseError> {
        let mut sensor = SensorData::new();
        let mut offset = 0;

        while offset + 2 <= data.len() {
            let item_id = data[offset];
            let item_len = data[offset + 1] as usize;
            offset += 2;

            if offset + item_len > data.len() {
                debug!("Invalid item length for ID 0x{:02X}", item_id);
                break;
            }

            let item_data = &data[offset..offset + item_len];

            match item_id {
                // 里程 (1/10km)
                0x01 if item_len >= 4 => {
                    let mileage = u32::from_be_bytes([
                        item_data[0],
                        item_data[1],
                        item_data[2],
                        item_data[3],
                    ]) as f64
                        / 10.0;
                    sensor.mileage = Some(mileage);
                    debug!("Mileage: {:.1} km", mileage);
                }

                // 油量 (1/10L)
                0x02 if item_len >= 2 => {
                    let fuel = u16::from_be_bytes([item_data[0], item_data[1]]) as f64 / 10.0;
                    sensor.fuel = Some(fuel);
                    debug!("Fuel: {:.1} L", fuel);
                }

                // 温度
                0x03 if item_len >= 1 => {
                    let temp = item_data[0] as i8 as f64;
                    sensor.water_temp = Some(temp);
                    debug!("Temperature: {:.1}°C", temp);
                }

                // 速度1 (可能是发动机转速相关)
                0x04 if item_len >= 1 => {
                    sensor.engine_rpm = Some(item_data[0] as i32);
                    debug!("Engine RPM: {}", item_data[0]);
                }

                // IO状态
                0x25 if item_len >= 2 => {
                    let io_status = u16::from_be_bytes([item_data[0], item_data[1]]) as u32;
                    sensor.io_status = Some(io_status);
                    debug!("IO Status: 0x{:04X}", io_status);
                }

                // 模拟量
                0x2E if item_len >= 3 => {
                    let analog_id = item_data[0];
                    let analog_value = u16::from_be_bytes([item_data[1], item_data[2]]) as f64;
                    sensor.analog_inputs.push((analog_id, analog_value));
                    debug!("Analog {}: {:.1}", analog_id, analog_value);
                }

                // 载重
                0x31 if item_len >= 2 => {
                    let load = u16::from_be_bytes([item_data[0], item_data[1]]) as f64;
                    sensor.load_weight = Some(load);
                    debug!("Load Weight: {:.1} kg", load);
                }

                // 扩展指令 (E7 02) - JT-2011增强版
                0xE7 if item_len >= 2 && item_data[0] == 0x02 => {
                    debug!("Parsing JT-2011 extended command E7 02");
                    Self::parse_extended_command(item_data, &mut sensor)?;
                }

                // 称重信息扩展(4字节)
                0xE8 if item_len >= 4 => {
                    let sensor_positions = u16::from_be_bytes([item_data[0], item_data[1]]);
                    let received_sensors = u16::from_be_bytes([item_data[2], item_data[3]]);
                    debug!(
                        "Weight extension: positions=0x{:04X}, received=0x{:04X}",
                        sensor_positions, received_sensors
                    );
                }

                // 称重传感器数据(0xE9-0xFF)
                0xE9..=0xFF => {
                    // 首先尝试解析二进制格式(18字节)
                    let binary_sensors = Self::parse_weight_sensors_v2(item_data);
                    if !binary_sensors.is_empty() {
                        debug!(
                            "Parsed {} weight sensors (binary format)",
                            binary_sensors.len()
                        );
                        for sensor_v2 in &binary_sensors {
                            sensor
                                .sensors
                                .push((sensor_v2.address, sensor_v2.pressure as f64));
                        }
                    } else {
                        // 尝试解析文本格式(Addr=xxx,T=xxx,P=xxx;)
                        let text_sensors = Self::parse_weight_text_sensors(item_data);
                        if !text_sensors.is_empty() {
                            debug!("Parsed {} weight sensors (text format)", text_sensors.len());
                            for text_sensor in &text_sensors {
                                sensor
                                    .sensors
                                    .push((text_sensor.address as u8, text_sensor.pressure as f64));
                            }
                        } else {
                            debug!(
                                "Unknown weight data item: 0x{:02X} (length: {})",
                                item_id, item_len
                            );
                        }
                    }
                }

                _ => {
                    debug!(
                        "Unknown additional data item: 0x{:02X} (length: {})",
                        item_id, item_len
                    );
                }
            }

            offset += item_len;
        }

        // 解析报警
        sensor.alarms = Self::parse_alarms(0); // 需要从LocationReport传入alarm_flag

        Ok(sensor)
    }

    /// 解析扩展指令 (E7 02) - JT-2011增强版
    pub fn parse_extended_command(data: &[u8], sensor: &mut SensorData) -> Result<(), ParseError> {
        // 跳过扩展指令标识 (02)
        let mut offset = 1;

        if offset + 2 > data.len() {
            return Err(ParseError::InvalidLength);
        }

        // 传感器长度
        let sensor_length = u16::from_be_bytes([data[offset], data[offset + 1]]) as usize;
        offset += 2;

        debug!(
            "Sensor length: {}, data[offset-2]: 0x{:02X}, data[offset-1]: 0x{:02X}",
            sensor_length,
            data[offset - 2],
            data[offset - 1]
        );

        debug!("Extended command sensor length: {}", sensor_length);

        // 解析传感器数据
        // 传感器长度是从传感器长度字段之后开始计算的
        let sensor_end = offset + sensor_length;
        while offset + 18 <= sensor_end {
            // 传感器序号
            let sensor_id = data[offset];
            offset += 1;

            // 传感器压力值 (3字节)
            let pressure = if offset + 3 <= sensor_end {
                u32::from_be_bytes([0, data[offset], data[offset + 1], data[offset + 2]])
            } else {
                0
            };
            offset += 3;

            // 传感器温度 (3字节)
            let temperature = if offset + 3 <= sensor_end {
                u32::from_be_bytes([0, data[offset], data[offset + 1], data[offset + 2]])
            } else {
                0
            };
            offset += 3;

            // G-sensor X-Axis (2字节)
            let accel_x = if offset + 2 <= sensor_end {
                i16::from_be_bytes([data[offset], data[offset + 1]])
            } else {
                0
            };
            offset += 2;

            // G-sensor Y-Axis (2字节)
            let accel_y = if offset + 2 <= sensor_end {
                i16::from_be_bytes([data[offset], data[offset + 1]])
            } else {
                0
            };
            offset += 2;

            // G-sensor Z-Axis (2字节)
            let accel_z = if offset + 2 <= sensor_end {
                i16::from_be_bytes([data[offset], data[offset + 1]])
            } else {
                0
            };
            offset += 2;

            // Horizontal Angle (2字节)
            let tilt_horizontal = if offset + 2 <= sensor_end {
                u16::from_be_bytes([data[offset], data[offset + 1]])
            } else {
                0
            };
            offset += 2;

            // Vertical Angle (2字节)
            let tilt_vertical = if offset + 2 <= sensor_end {
                u16::from_be_bytes([data[offset], data[offset + 1]])
            } else {
                0
            };
            offset += 2;

            // 传感器分割标识 (1字节: 3B)
            if offset < sensor_end && data[offset] == 0x3B {
                offset += 1;
            }

            // 存储传感器数据,使用压力值作为主要数值
            sensor.sensors.push((sensor_id, pressure as f64));

            // 转换温度为摄氏度
            let temp_c = temperature as f64 / 100.0;
            // 转换倾斜角为度
            let tilt_h = tilt_horizontal as f64 / 100.0;
            let tilt_v = tilt_vertical as f64 / 100.0;

            debug!(
                "Sensor {}: pressure={}, temp={:.2}°C, accel=({},{},{})mg, tilt=({:.2}°, {:.2}°)",
                sensor_id, pressure, temp_c, accel_x, accel_y, accel_z, tilt_h, tilt_v
            );
        }

        // 确保offset不超过sensor_end
        if offset > sensor_end {
            offset = sensor_end;
        }

        // 传感器检验码 (2字节)
        if offset + 2 <= data.len() {
            let checksum = u16::from_be_bytes([data[offset], data[offset + 1]]);
            offset += 2;
            debug!("Sensor checksum: 0x{:04X}", checksum);
        }

        // 解析总重数据
        // 总重 (单位:1kg) - F9 04 00 00 10 CE
        // F9 = 总重标识, 04 = 长度(4字节), 00 00 10 CE = 总重数值
        if offset + 6 <= data.len() && data[offset] == 0xF9 {
            let weight_len = data[offset + 1] as usize;
            offset += 2;
            if offset + weight_len <= data.len() {
                let total_weight_kg = match weight_len {
                    1 => data[offset] as f64,
                    2 => u16::from_be_bytes([data[offset], data[offset + 1]]) as f64,
                    4 => u32::from_be_bytes([
                        data[offset],
                        data[offset + 1],
                        data[offset + 2],
                        data[offset + 3],
                    ]) as f64,
                    _ => 0.0,
                };
                sensor.total_weight_kg = Some(total_weight_kg);
                debug!("Total weight (1kg): {:.1} kg", total_weight_kg);
                offset += weight_len;
            }
        }

        // 总重 (单位:0.1kg) - 55 08 00 00 00 00 00 00 A8 0C
        // 55 = 总重标识, 08 = 长度(8字节), 00 00 00 00 00 00 A8 0C = 总重数值
        // 08表示后面8个字节,直接使用完整的8字节数据
        if offset + 3 <= data.len() && data[offset] == 0x55 {
            let weight_len = data[offset + 1] as usize;
            offset += 2;
            if offset + weight_len <= data.len() {
                let total_weight_01kg = match weight_len {
                    1 => data[offset] as f64 / 10.0,
                    2 => u16::from_be_bytes([data[offset], data[offset + 1]]) as f64 / 10.0,
                    4 => {
                        u32::from_be_bytes([
                            data[offset],
                            data[offset + 1],
                            data[offset + 2],
                            data[offset + 3],
                        ]) as f64
                            / 10.0
                    }
                    8 => {
                        u64::from_be_bytes([
                            data[offset],
                            data[offset + 1],
                            data[offset + 2],
                            data[offset + 3],
                            data[offset + 4],
                            data[offset + 5],
                            data[offset + 6],
                            data[offset + 7],
                        ]) as f64
                            / 10.0
                    }
                    _ => 0.0,
                };
                sensor.total_weight_01kg = Some(total_weight_01kg);
                debug!("Total weight (0.1kg): {:.1} kg", total_weight_01kg);
            }
        }

        Ok(())
    }

    /// 解析称重传感器文本数据
    /// 格式: Addr=3755,T=1926,P=64390;Addr=3575,T=1411,P=59355;...
    fn parse_weight_text_sensors(data: &[u8]) -> Vec<WeightTextSensor> {
        let mut sensors = Vec::new();

        // 将字节数据转换为字符串
        let text = match String::from_utf8(data.to_vec()) {
            Ok(s) => s,
            Err(_) => return sensors,
        };

        // 按分号分隔多个传感器
        for sensor_str in text.split(';') {
            let sensor_str = sensor_str.trim();
            if sensor_str.is_empty() {
                continue;
            }

            // 解析 Addr=xxx,T=xxx,P=xxx
            let mut address: Option<u16> = None;
            let mut temperature: Option<u16> = None;
            let mut pressure: Option<u32> = None;

            for part in sensor_str.split(',') {
                let part = part.trim();
                if let Some(val) = part.strip_prefix("Addr=") {
                    address = val.parse().ok();
                } else if let Some(val) = part.strip_prefix("T=") {
                    temperature = val.parse().ok();
                } else if let Some(val) = part.strip_prefix("P=") {
                    pressure = val.parse().ok();
                }
            }

            if let (Some(addr), Some(temp), Some(press)) = (address, temperature, pressure) {
                sensors.push(WeightTextSensor {
                    address: addr,
                    temperature: temp,
                    pressure: press,
                });
            }
        }

        sensors
    }

    /// 解析称重传感器数据V2(18字节格式)
    ///
    /// 数据格式(18字节):
    /// - 字节0: 传感器序号
    /// - 字节1-3: 压力原始值 (3字节大端)
    /// - 字节4-6: 温度值 (3字节大端,单位0.01℃)
    /// - 字节7-8: X轴加速度 (mg)
    /// - 字节9-10: Y轴加速度 (mg)
    /// - 字节11-12: Z轴加速度 (mg)
    /// - 字节13-14: 水平倾斜角 (0.01度)
    /// - 字节15-16: 垂直倾斜角 (0.01度)
    /// - 字节17: 分隔符 ';' (0x3B)
    fn parse_weight_sensors_v2(data: &[u8]) -> Vec<WeightSensorV2> {
        let mut sensors = Vec::new();
        let mut i = 0;

        while i + 18 <= data.len() {
            // 检查是否是传感器数据(以地址02-07开头)
            let address = data[i];
            if !(2..=7).contains(&address) {
                i += 1;
                continue;
            }

            // 解析传感器数据(18字节)
            let pressure =
                ((data[i + 1] as u32) << 16) | ((data[i + 2] as u32) << 8) | (data[i + 3] as u32);
            let temperature =
                ((data[i + 4] as u32) << 16) | ((data[i + 5] as u32) << 8) | (data[i + 6] as u32);
            let accel_x = u16::from_be_bytes([data[i + 7], data[i + 8]]);
            let accel_y = u16::from_be_bytes([data[i + 9], data[i + 10]]);
            let accel_z = u16::from_be_bytes([data[i + 11], data[i + 12]]);
            let tilt_horizontal = u16::from_be_bytes([data[i + 13], data[i + 14]]);
            let tilt_vertical = u16::from_be_bytes([data[i + 15], data[i + 16]]);

            sensors.push(WeightSensorV2 {
                address,
                pressure,
                temperature,
                accel_x,
                accel_y,
                accel_z,
                tilt_horizontal,
                tilt_vertical,
            });

            i += 18;
        }

        sensors
    }

    /// 解析报警标志
    fn parse_alarms(alarm_flag: u32) -> Vec<AlarmType> {
        let mut alarms = Vec::new();

        if alarm_flag & 0x01 != 0 {
            alarms.push(AlarmType::Overspeed);
        }
        if alarm_flag & 0x02 != 0 {
            alarms.push(AlarmType::FatigueDriving);
        }
        if alarm_flag & 0x04 != 0 {
            alarms.push(AlarmType::EmergencyBrake);
        }
        if alarm_flag & 0x08 != 0 {
            alarms.push(AlarmType::FuelLeakage);
        }
        if alarm_flag & 0x10 != 0 {
            alarms.push(AlarmType::TemperatureHigh);
        }
        if alarm_flag & 0x20 != 0 {
            alarms.push(AlarmType::IOStateChanged);
        }

        alarms
    }

    /// 提取GPS状态
    pub fn extract_gps_status(status: u32) -> GpsStatus {
        // 根据状态位判断GPS有效性
        if status & 0x00000002 != 0 {
            GpsStatus::Valid
        } else if status & 0x00000001 != 0 {
            GpsStatus::Invalid
        } else {
            GpsStatus::Unknown
        }
    }

    /// 提取卫星数量
    pub fn extract_satellite_count(status: u32) -> i32 {
        ((status >> 16) & 0xFF) as i32
    }

    /// 去除转义字符
    fn unescape(data: &[u8]) -> Result<Vec<u8>, ParseError> {
        let mut result = Vec::new();
        let mut i = 0;

        while i < data.len() {
            if data[i] == 0x7D && i + 1 < data.len() {
                match data[i + 1] {
                    0x01 => {
                        result.push(0x7D);
                        i += 2;
                    }
                    0x02 => {
                        result.push(0x7E);
                        i += 2;
                    }
                    _ => {
                        return Err(ParseError::InvalidEscape);
                    }
                }
            } else {
                result.push(data[i]);
                i += 1;
            }
        }

        Ok(result)
    }

    /// 验证校验码
    fn verify_checksum(data: &[u8], checksum: u8) -> bool {
        let mut sum: u8 = 0;
        for &byte in data {
            sum = sum.wrapping_add(byte);
        }
        sum == checksum
    }

    /// BCD码转字符串
    fn bcd_to_string(data: &[u8]) -> String {
        data.iter().map(|b| format!("{:02X}", b)).collect()
    }

    /// BCD码时间转DateTime
    fn parse_bcd_time(s: &str) -> Result<chrono::DateTime<chrono::Utc>, ParseError> {
        if s.len() < 12 {
            return Err(ParseError::InvalidBCD);
        }

        let year = 2000 + u32::from_str_radix(&s[0..2], 16).unwrap_or(0);
        let month = u32::from_str_radix(&s[2..4], 16).unwrap_or(0);
        let day = u32::from_str_radix(&s[4..6], 16).unwrap_or(0);
        let hour = u32::from_str_radix(&s[6..8], 16).unwrap_or(0);
        let minute = u32::from_str_radix(&s[8..10], 16).unwrap_or(0);
        let second = u32::from_str_radix(&s[10..12], 16).unwrap_or(0);

        chrono::Utc
            .with_ymd_and_hms(year as i32, month, day, hour, minute, second)
            .single()
            .ok_or(ParseError::InvalidDateTime)
    }

    /// 解析BCD编码的手机号
    fn parse_bcd_phone(data: &[u8]) -> String {
        let mut result = String::new();
        for &byte in data {
            let high = (byte >> 4) & 0x0F;
            let low = byte & 0x0F;
            result.push_str(&format!("{}{}", high, low));
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_0x0200() {
        // 构造一个简单的0x0200消息体
        let mut body = vec![0u8; 28];

        // 报警标志
        body[0..4].copy_from_slice(&0x00000000u32.to_be_bytes());
        // 状态
        body[4..8].copy_from_slice(&0x00000002u32.to_be_bytes());
        // 纬度 (22.5431度)
        body[8..12].copy_from_slice(&(22543100i32).to_be_bytes());
        // 经度 (114.0579度)
        body[12..16].copy_from_slice(&(114057900i32).to_be_bytes());
        // 高程 (50米)
        body[16..18].copy_from_slice(&50u16.to_be_bytes());
        // 速度 (65.5km/h)
        body[18..20].copy_from_slice(&655u16.to_be_bytes());
        // 方向 (180度)
        body[20..22].copy_from_slice(&180u16.to_be_bytes());
        // 时间 (2026-01-18 10:30:00)
        body[22..28].copy_from_slice(&[0x26, 0x01, 0x18, 0x10, 0x30, 0x00]);

        let report = JT808Parser::parse_0x0200(&body).unwrap();

        assert_eq!(report.latitude, 22.5431);
        assert_eq!(report.longitude, 114.0579);
        assert_eq!(report.altitude, 50.0);
        assert_eq!(report.speed, 65.5);
        assert_eq!(report.direction, 180.0);
    }

    #[test]
    fn test_parse_jt2011_extended() {
        // JT-2011增强版协议示例数据
        let hex_data = "7E020000A40142601900010962000000000004000301DD87120646F3C0000002800052260222092237010400712969E702006C0200F3740007FC000000000000123456783B0300EC00000653000000000000123456783B0400E19D00063E000000000000123456783B0500D999000643000000000000123456783B0600E09F000672000000000000123456783B0700B868000877000000000000123456783BD803F904000040A555080000000000028672F57E";

        // 转换为字节数组
        let mut data = Vec::new();
        for i in (0..hex_data.len()).step_by(2) {
            let byte = u8::from_str_radix(&hex_data[i..i + 2], 16).unwrap();
            data.push(byte);
        }

        // 解析帧
        let frame = JT808Parser::parse_frame(&data).unwrap();
        assert_eq!(frame.msg_id, 0x0200);

        // 解析消息体
        let report = JT808Parser::parse_body(frame.msg_id, &frame.body).unwrap();

        // 验证基本数据
        assert_eq!(report.alarm_flag, 0x09620000);
        assert_eq!(report.status, 0x00000004);
        assert!((report.latitude - 30.6707).abs() < 0.0001); // 30.6707度
        assert!((report.longitude - 104.0668).abs() < 0.0001); // 104.0668度

        // 验证传感器数据
        assert_eq!(report.sensor_data.sensors.len(), 6);
        assert!(report.sensor_data.total_weight_kg.is_some());
        assert!(report.sensor_data.total_weight_01kg.is_some());

        // 验证总重数据
        let total_weight_kg = report.sensor_data.total_weight_kg.unwrap();
        let total_weight_01kg = report.sensor_data.total_weight_01kg.unwrap();
        println!("Total weight (1kg): {:.1} kg", total_weight_kg);
        println!("Total weight (0.1kg): {:.1} kg", total_weight_01kg);
        println!("Sensors: {:?}", report.sensor_data.sensors);
    }
}
