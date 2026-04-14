//! / BFF统一数据模型

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// 车辆实时状态聚合模型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VehicleRealtimeStatus {
    /// 车辆基础信息
    pub vehicle: VehicleBaseInfo,

    /// GPS/北斗定位数据
    pub gps: Option<GpsData>,

    /// 传感器数据
    pub sensors: Option<SensorData>,

    /// 运营状态
    pub operation: Option<OperationStatus>,

    /// 数据来源标识
    pub source: DataSource,

    /// 数据接收时间
    pub received_at: DateTime<Utc>,
}

/// 车辆基础信息(精简版)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct VehicleBaseInfo {
    pub vehicle_id: i32,
    pub vehicle_name: String,
    pub license_plate: String,
    pub vehicle_type: String,
    pub vehicle_color: String,
    pub device_id: Option<String>,
    pub terminal_type: Option<String>,
    pub group_id: i32,
    pub group_name: Option<String>,
    pub status: i16,
}

impl From<crate::models::Vehicle> for VehicleBaseInfo {
    fn from(v: crate::models::Vehicle) -> Self {
        Self {
            // models.Vehicle.id (vehicle PK) → vehicle_id
            vehicle_id: v.id,
            // models.Vehicle.model (车型/型号) → vehicle_name (BFF 用 vehicle_name 表示车名/型号)
            vehicle_name: v.model.clone(),
            license_plate: v.plate_number.clone(),
            // models.Vehicle.vehicle_type → vehicle_type
            vehicle_type: v.vehicle_type.clone(),
            // models.Vehicle.brand → vehicle_color (BFF 无 brand 字段，复用到 vehicle_color)
            vehicle_color: v.brand.clone(),
            // models.Vehicle.vehicle_id 是设备号字符串（如 "DEV001"）→ device_id
            device_id: Some(v.vehicle_id),
            terminal_type: None,
            // models.Vehicle.owner_id (车主用户ID) → group_id (占位，BFF 无 owner_id 对应字段)
            group_id: v.owner_id,
            // 需要从 vehicle_groups 表联查
            group_name: None,
            // models.Vehicle.status 是 String → i16
            status: v.status.parse().unwrap_or(0),
        }
    }
}

/// GPS数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpsData {
    pub latitude: f64,
    pub longitude: f64,
    pub altitude: Option<f64>,
    pub speed: f64,
    pub direction: f64,
    pub gps_time: DateTime<Utc>,
    pub location_accuracy: Option<f64>,
    pub satellite_count: Option<i32>,
}

/// 传感器数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorData {
    /// 传感器类型和值映射 (sensor_type -> sensor_value)
    pub sensors: Vec<SensorReading>,
    /// 传感器数据接收时间
    pub collect_time: DateTime<Utc>,
}

/// 单个传感器读数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorReading {
    /// 传感器类型 (如: fuel, water_temp, oil_temp, engine_rpm, load_weight, mileage, io_status)
    pub sensor_type: String,
    /// 传感器值
    pub sensor_value: f64,
    /// 单位
    pub unit: Option<String>,
}

impl SensorData {
    /// 获取油量 (L)
    pub fn get_fuel(&self) -> Option<f64> {
        self.sensors
            .iter()
            .find(|s| s.sensor_type == "fuel")
            .map(|s| s.sensor_value)
    }

    /// 获取水温 (°C)
    pub fn get_water_temp(&self) -> Option<f64> {
        self.sensors
            .iter()
            .find(|s| s.sensor_type == "water_temp")
            .map(|s| s.sensor_value)
    }

    /// 获取油温 (°C)
    pub fn get_oil_temp(&self) -> Option<f64> {
        self.sensors
            .iter()
            .find(|s| s.sensor_type == "oil_temp")
            .map(|s| s.sensor_value)
    }

    /// 获取发动机转速 (RPM)
    pub fn get_engine_rpm(&self) -> Option<i32> {
        self.sensors
            .iter()
            .find(|s| s.sensor_type == "engine_rpm")
            .map(|s| s.sensor_value as i32)
    }

    /// 获取载重 (kg)
    pub fn get_load_weight(&self) -> Option<f64> {
        self.sensors
            .iter()
            .find(|s| s.sensor_type == "load_weight")
            .map(|s| s.sensor_value)
    }

    /// 获取里程 (km)
    pub fn get_mileage(&self) -> Option<f64> {
        self.sensors
            .iter()
            .find(|s| s.sensor_type == "mileage")
            .map(|s| s.sensor_value)
    }

    /// 获取IO状态
    pub fn get_io_status(&self) -> Option<u32> {
        self.sensors
            .iter()
            .find(|s| s.sensor_type == "io_status")
            .map(|s| s.sensor_value as u32)
    }
}

/// 运营状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationStatus {
    /// 状态: 1-在线, 2-离线, 3-行驶中, 4-停车中
    pub status: i16,
    /// 最后上线时间
    pub last_online_time: Option<DateTime<Utc>>,
    /// 累计行驶时间 (秒)
    pub total_driving_time: Option<i64>,
    /// 累计行驶里程 (km)
    pub total_mileage: Option<f64>,
    /// 当前司机
    pub current_driver: Option<String>,
}

/// 数据来源
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DataSource {
    /// 本地数据库
    LocalDB,
    /// 旧服务器
    LegacyTcp,
    /// WebSocket实时
    WebSocket,
    /// Redis缓存
    RedisCache,
    /// 第三方平台
    ThirdParty(String),
}

/// 车辆实时状态查询参数
#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct VehicleRealtimeQuery {
    /// 页码
    #[serde(default = "default_page")]
    pub page: u32,

    /// 每页大小
    #[serde(default = "default_size")]
    pub size: u32,

    /// 分组ID
    pub group_id: Option<i32>,

    /// 状态
    pub status: Option<i16>,

    /// 车辆类型
    pub vehicle_type: Option<String>,

    /// 车牌号
    pub license_plate: Option<String>,

    /// 是否只查询在线车辆
    pub online_only: Option<bool>,
}

fn default_page() -> u32 {
    1
}

fn default_size() -> u32 {
    20
}

/// 分页响应
#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub total: u64,
    pub page: u32,
    pub size: u32,
}

impl<T> PaginatedResponse<T> {
    pub fn new(items: Vec<T>, total: u64, page: u32, size: u32) -> Self {
        Self {
            items,
            total,
            page,
            size,
        }
    }
}

/// API统一响应格式
#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub code: i32,
    pub message: String,
    pub data: T,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            code: 200,
            message: "success".to_string(),
            data,
        }
    }

    pub fn error(code: i32, message: String, data: T) -> Self {
        Self {
            code,
            message,
            data,
        }
    }
}

/// 为serde_json::Value实现的便捷错误方法
impl ApiResponse<serde_json::Value> {
    pub fn error_with_null(code: i32, message: String) -> Self {
        Self {
            code,
            message,
            data: serde_json::Value::Null,
        }
    }
}

/// 批量查询请求
#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct BatchVehicleQuery {
    /// 车辆ID列表
    pub vehicle_ids: Vec<i32>,
}

/// GPS轨迹点
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct GpsTrackPoint {
    pub id: i64,
    pub vehicle_id: i32,
    pub latitude: f64,
    pub longitude: f64,
    pub altitude: Option<f64>,
    pub speed: f64,
    pub direction: f64,
    pub gps_time: DateTime<Utc>,
    pub location_accuracy: Option<f64>,
    pub satellite_count: Option<i32>,
    pub address: Option<String>,
}

/// 轨迹查询参数
#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct TrackQueryParams {
    /// 车辆ID
    pub vehicle_id: i32,
    /// 开始时间
    pub start_time: DateTime<Utc>,
    /// 结束时间
    pub end_time: DateTime<Utc>,
    /// 最大点数(用于降采样)
    #[serde(default = "default_max_points")]
    pub max_points: u32,
}

fn default_max_points() -> u32 {
    10000
}

/// 轨迹回放参数
#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct TrackPlaybackParams {
    /// 车辆ID
    pub vehicle_id: i32,
    /// 开始时间
    pub start_time: DateTime<Utc>,
    /// 结束时间
    pub end_time: DateTime<Utc>,
    /// 回放速度倍数(1=正常速度,2=2倍速)
    #[serde(default = "default_playback_speed")]
    pub speed_multiplier: f64,
    /// 播放间隔(毫秒)
    #[serde(default = "default_playback_interval")]
    pub interval_ms: u64,
}

fn default_playback_speed() -> f64 {
    1.0
}

fn default_playback_interval() -> u64 {
    1000
}

/// 传感器历史数据查询参数
#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct SensorHistoryQuery {
    /// 车辆ID
    pub vehicle_id: i32,
    /// 开始时间
    pub start_time: DateTime<Utc>,
    /// 结束时间
    pub end_time: DateTime<Utc>,
    /// 传感器类型(fuel, water_temp, oil_temp, engine_rpm等)
    pub sensor_type: Option<String>,
    /// 采样间隔(分钟)
    #[serde(default = "default_sample_interval")]
    pub sample_interval: u32,
}

fn default_sample_interval() -> u32 {
    1
}

// ============================================
// 报表相关模型
// ============================================

/// 报表请求参数(通用)
#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct ReportRequest {
    /// 开始时间
    pub start_time: DateTime<Utc>,
    /// 结束时间
    pub end_time: DateTime<Utc>,
    /// 车辆ID列表(可选,不传则查询所有车辆)
    pub vehicle_ids: Option<Vec<i32>>,
    /// 分组ID(可选)
    pub group_id: Option<i32>,
    /// 报表格式(json, excel)
    #[serde(default = "default_report_format")]
    pub report_format: String,
}

fn default_report_format() -> String {
    "json".to_string()
}

/// 车辆运营报表
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VehicleOperationReport {
    /// 报表生成时间
    pub generated_at: DateTime<Utc>,
    /// 统计时间范围
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    /// 车辆运营明细
    pub vehicles: Vec<VehicleOperationSummary>,
    /// 统计汇总
    pub summary: OperationSummary,
}

/// 车辆运营汇总
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VehicleOperationSummary {
    pub vehicle_id: i32,
    pub vehicle_name: String,
    pub license_plate: String,
    pub driver_name: Option<String>,
    /// 总行驶里程 (km)
    pub total_mileage: f64,
    /// 总行驶时长 (秒)
    pub total_duration: i64,
    /// 总燃油消耗 (L)
    pub total_fuel_consumption: Option<f64>,
    /// 平均速度 (km/h)
    pub average_speed: f64,
    /// 最高速度 (km/h)
    pub max_speed: f64,
    /// 在线时长 (秒)
    pub online_duration: i64,
    /// 离线时长 (秒)
    pub offline_duration: i64,
    /// GPS轨迹点数
    pub track_point_count: i64,
}

/// 运营汇总统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationSummary {
    /// 车辆总数
    pub total_vehicles: i64,
    /// 总行驶里程 (km)
    pub total_mileage: f64,
    /// 总行驶时长 (小时)
    pub total_duration_hours: f64,
    /// 总燃油消耗 (L)
    pub total_fuel_consumption: Option<f64>,
    /// 平均速度 (km/h)
    pub average_speed: f64,
    /// 最高速度 (km/h)
    pub max_speed: f64,
    /// 总在线时长 (小时)
    pub total_online_hours: f64,
}

/// 称重统计报表
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeighingStatisticsReport {
    /// 报表生成时间
    pub generated_at: DateTime<Utc>,
    /// 统计时间范围
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    /// 称重明细
    pub weighings: Vec<WeighingRecord>,
    /// 统计汇总
    pub summary: WeighingSummary,
}

/// 称重记录
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct WeighingRecord {
    pub weighing_id: i64,
    pub vehicle_id: i32,
    pub vehicle_name: String,
    pub license_plate: String,
    /// 毛重 (kg)
    pub gross_weight: f64,
    /// 皮重 (kg)
    pub tare_weight: f64,
    /// 净重 (kg)
    pub net_weight: f64,
    /// 称重时间
    pub weighing_time: DateTime<Utc>,
    /// 称重地点
    pub location: Option<String>,
    /// 物料类型
    pub material_type: Option<String>,
}

/// 称重汇总统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeighingSummary {
    /// 称重次数
    pub total_weighings: i64,
    /// 总毛重 (kg)
    pub total_gross_weight: f64,
    /// 总皮重 (kg)
    pub total_tare_weight: f64,
    /// 总净重 (kg)
    pub total_net_weight: f64,
    /// 平均净重 (kg)
    pub average_net_weight: f64,
    /// 最大净重 (kg)
    pub max_net_weight: f64,
    /// 最小净重 (kg)
    pub min_net_weight: f64,
}

/// 报警分析报表
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlarmAnalysisReport {
    /// 报表生成时间
    pub generated_at: DateTime<Utc>,
    /// 统计时间范围
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    /// 报警明细
    pub alarms: Vec<AlarmRecord>,
    /// 按类型统计
    pub by_type: Vec<AlarmTypeStatistics>,
    /// 按车辆统计
    pub by_vehicle: Vec<AlarmVehicleStatistics>,
    /// 汇总统计
    pub summary: AlarmSummary,
}

/// 报警记录
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AlarmRecord {
    pub alarm_id: i64,
    pub vehicle_id: i32,
    pub vehicle_name: String,
    pub license_plate: String,
    /// 报警类型
    pub alarm_type: String,
    /// 报警级别(1-低,2-中,3-高,4-紧急)
    pub alarm_level: i32,
    /// 报警消息
    pub alarm_message: String,
    /// 报警时间
    pub alarm_time: DateTime<Utc>,
    /// 报警位置
    pub location: Option<String>,
    /// 是否已处理
    pub is_handled: bool,
    /// 处理时间
    pub handled_time: Option<DateTime<Utc>>,
    /// 处理人
    pub handler: Option<String>,
}

/// 按类型统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlarmTypeStatistics {
    /// 报警类型
    pub alarm_type: String,
    /// 报警次数
    pub count: i64,
    /// 占比
    pub percentage: f64,
}

/// 按车辆统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlarmVehicleStatistics {
    pub vehicle_id: i32,
    pub vehicle_name: String,
    pub license_plate: String,
    /// 报警次数
    pub alarm_count: i64,
    /// 紧急报警次数
    pub urgent_count: i64,
    /// 高级报警次数
    pub high_count: i64,
}

/// 报警汇总统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlarmSummary {
    /// 总报警次数
    pub total_alarms: i64,
    /// 已处理次数
    pub handled_alarms: i64,
    /// 未处理次数
    pub unhandled_alarms: i64,
    /// 处理率
    pub handling_rate: f64,
    /// 紧急报警次数
    pub urgent_alarms: i64,
    /// 高级报警次数
    pub high_alarms: i64,
}

/// 报表导出查询参数
#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct ExportQuery {
    /// 导出格式: excel, pdf
    pub format: String,
}
