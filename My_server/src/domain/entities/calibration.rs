//! 载重标定领域实体

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 传感器标定实体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorCalibration {
    pub id: i32,
    pub sensor_no: i32,
    pub vehicle_id: i32,
    pub plate_no: String,
    pub sensor_side: String,
    pub sensor_group: Option<i32>,
    pub self_weight: Option<i32>,
    pub polynomial_json: Option<String>,
    pub linear_segments_json: Option<String>,
    pub is_calibrated: bool,
    pub create_time: DateTime<Utc>,
    pub update_time: Option<DateTime<Utc>>,
    // DDD 改造：支持标定点数组（用于多项式拟合计算）
    pub calibration_points: Option<String>, // JSON 数组
    pub pa_raw: Option<i32>,
    pub axle_number: Option<i32>,
    pub is_left_wheel: Option<bool>,
    pub turn_point: Option<i32>,
    pub polynomial_order: Option<i32>,
    pub r2_score: Option<f64>,
    pub rmse: Option<f64>,
    pub max_error: Option<f64>,
    pub point_count: Option<i32>,
    pub rated_total_weight: Option<f64>,
    pub tare_weight: Option<f64>,
}

impl SensorCalibration {
    /// 创建新的传感器标定实体
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        sensor_no: i32,
        vehicle_id: i32,
        plate_no: String,
        sensor_side: String,
        sensor_group: Option<i32>,
        self_weight: Option<i32>,
        polynomial_json: Option<String>,
        linear_segments_json: Option<String>,
        is_calibrated: bool,
    ) -> Self {
        Self {
            id: 0, // 数据库自增
            sensor_no,
            vehicle_id,
            plate_no,
            sensor_side,
            sensor_group,
            self_weight,
            polynomial_json,
            linear_segments_json,
            is_calibrated,
            create_time: Utc::now(),
            update_time: None,
            calibration_points: None,
            pa_raw: None,
            axle_number: None,
            is_left_wheel: None,
            turn_point: None,
            polynomial_order: None,
            r2_score: None,
            rmse: None,
            max_error: None,
            point_count: None,
            rated_total_weight: None,
            tare_weight: None,
        }
    }

    /// 标记为已标定
    pub fn mark_as_calibrated(&mut self) {
        self.is_calibrated = true;
        self.update_time = Some(Utc::now());
    }

    /// 更新多项式系数
    pub fn update_polynomial(&mut self, polynomial_json: String) {
        self.polynomial_json = Some(polynomial_json);
        self.update_time = Some(Utc::now());
    }
}

/// 标定历史记录实体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalibrationHistory {
    pub id: i32,
    pub sensor_no: i32,
    pub vehicle_id: i32,
    pub plate_no: String,
    pub polynomial_json: String,
    pub polynomial_order: i32,
    pub r2_score: f64,
    pub rmse: f64,
    pub max_error: f64,
    pub point_count: i32,
    pub operation_type: String,
    pub operation_type_name: Option<String>,
    pub operator: Option<String>,
    pub remark: Option<String>,
    pub is_valid: bool,
    pub create_time: DateTime<Utc>,
    pub update_time: Option<DateTime<Utc>>,
}

impl CalibrationHistory {
    /// 创建新的标定历史记录
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        sensor_no: i32,
        vehicle_id: i32,
        plate_no: String,
        polynomial_json: String,
        polynomial_order: i32,
        r2_score: f64,
        rmse: f64,
        max_error: f64,
        point_count: i32,
        operation_type: String,
        operation_type_name: Option<String>,
        operator: Option<String>,
        remark: Option<String>,
    ) -> Self {
        Self {
            id: 0, // 数据库自增
            sensor_no,
            vehicle_id,
            plate_no,
            polynomial_json,
            polynomial_order,
            r2_score,
            rmse,
            max_error,
            point_count,
            operation_type,
            operation_type_name,
            operator,
            remark,
            is_valid: true,
            create_time: Utc::now(),
            update_time: None,
        }
    }

    /// 标记为无效
    pub fn mark_as_invalid(&mut self) {
        self.is_valid = false;
        self.update_time = Some(Utc::now());
    }
}

/// 标定点数据实体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalibrationPoint {
    pub sensor_address: u16,
    pub calibration_time: DateTime<Utc>,
    pub actual_weight: f64,
    pub pa_value: u32,
    pub pa_raw: u32,
    pub temperature: f64,
    pub load_percentage: u8,
    pub is_manual: bool,
    pub record_time: DateTime<Utc>,
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
