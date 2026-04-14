//! / JT808协议数据模型

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// JT808协议帧
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JT808Frame {
    pub msg_id: u16,
    pub msg_attr: u16,
    pub phone: String,
    pub flow_no: u16,
    pub body: Vec<u8>,
    pub checksum: u8,
}

/// 0x0200 位置信息汇报消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationReport {
    pub alarm_flag: u32,
    pub status: u32,
    pub latitude: f64,
    pub longitude: f64,
    pub altitude: f64,
    pub speed: f64,
    pub direction: f64,
    pub timestamp: DateTime<Utc>,
    pub sensor_data: SensorData,
}

/// 传感器数据(从JT808附加数据提取)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorData {
    /// 设备ID
    pub device_id: String,

    /// 油量 (L)
    pub fuel: Option<f64>,

    /// 水温 (°C)
    pub water_temp: Option<f64>,

    /// 油温 (°C)
    pub oil_temp: Option<f64>,

    /// 发动机转速 (RPM)
    pub engine_rpm: Option<i32>,

    /// 载重 (kg)
    pub load_weight: Option<f64>,

    /// 总重 (kg) - 单位:1kg
    pub total_weight_kg: Option<f64>,

    /// 总重 (0.1kg) - 单位:0.1kg
    pub total_weight_01kg: Option<f64>,

    /// 里程 (km)
    pub mileage: Option<f64>,

    /// IO状态(车门、货箱等)
    pub io_status: Option<u32>,

    /// 模拟量(自定义传感器)
    pub analog_inputs: Vec<(u8, f64)>,

    /// 传感器数据(序号+数值)
    pub sensors: Vec<(u8, f64)>,

    /// 报警列表
    pub alarms: Vec<AlarmType>,

    /// 数据采集时间
    pub timestamp: DateTime<Utc>,
}

impl SensorData {
    pub fn new() -> Self {
        Self {
            device_id: String::new(),
            fuel: None,
            water_temp: None,
            oil_temp: None,
            engine_rpm: None,
            load_weight: None,
            total_weight_kg: None,
            total_weight_01kg: None,
            mileage: None,
            io_status: None,
            analog_inputs: Vec::new(),
            sensors: Vec::new(),
            alarms: Vec::new(),
            timestamp: Utc::now(),
        }
    }
}

impl Default for SensorData {
    fn default() -> Self {
        Self::new()
    }
}

/// 报警类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlarmType {
    Overspeed,       // 超速报警
    FatigueDriving,  // 疲劳驾驶
    EmergencyBrake,  // 紧急制动
    FuelLeakage,     // 油量异常
    TemperatureHigh, // 温度过高
    IOStateChanged,  // IO状态变化
    GpsLost,         // GPS丢失
    Custom(String),  // 自定义报警
}

/// GPS状态枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum GpsStatus {
    Valid,   // 有效
    Invalid, // 无效
    Unknown, // 未知
}

/// 附加数据项ID定义
#[allow(dead_code)]
pub enum AdditionalDataItem {
    Mileage = 0x01,     // 里程
    Fuel = 0x02,        // 油量
    Temperature = 0x03, // 温度
    Speed1 = 0x04,      // 速度
    VideoStatus = 0x05, // 录像状态
    Overspeed = 0x11,   // 超速报警
    IOStatus = 0x25,    // IO状态
    Analog = 0x2E,      // 模拟量
    LoadWeight = 0x31,  // 载重
}

/// 数据来源
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DataSource {
    JT808Terminal, // JT808终端直接上传
    LegacyServer,  // 旧服务器
    ThirdPartyA,   // 第三方平台A
    ThirdPartyB,   // 第三方平台B
}

/// 车辆实时状态(聚合数据)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VehicleRealtimeStatus {
    /// 车辆基础信息
    pub vehicle: VehicleBaseInfo,

    /// GPS/北斗定位数据
    pub gps: GpsData,

    /// 传感器数据
    pub sensors: SensorData,

    /// 数据来源标识
    pub source: DataSource,

    /// 数据接收时间
    pub received_at: DateTime<Utc>,
}

/// 车辆基础信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VehicleBaseInfo {
    pub vehicle_id: String,
    pub plate_number: String,
    pub device_id: String,
    pub vehicle_type: String,
    pub phone: String,
    pub sim_card: String,
}

/// GPS/北斗定位数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpsData {
    pub device_id: String,
    pub latitude: f64,
    pub longitude: f64,
    pub altitude: f64,
    pub speed: f64,
    pub direction: f64,
    pub timestamp: DateTime<Utc>,
    pub satellite_count: i32,
    pub gps_status: GpsStatus,
}

/// 轨迹点(包含传感器数据)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VehicleTrackPoint {
    pub vehicle_id: String,
    pub latitude: f64,
    pub longitude: f64,
    pub speed: Option<f64>,
    pub direction: Option<f64>,
    pub timestamp: DateTime<Utc>,
    pub sensor_data: Option<serde_json::Value>,
}

/// 解析错误
#[derive(Debug, Clone)]
pub enum ParseError {
    InvalidFrameHeader,
    ChecksumError,
    InvalidEscape,
    InvalidDateTime,
    InvalidBCD,
    InvalidLength,
    UnknownCommand,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::InvalidFrameHeader => write!(f, "Invalid frame header"),
            ParseError::ChecksumError => write!(f, "Checksum error"),
            ParseError::InvalidEscape => write!(f, "Invalid escape sequence"),
            ParseError::InvalidDateTime => write!(f, "Invalid date time"),
            ParseError::InvalidBCD => write!(f, "Invalid BCD encoding"),
            ParseError::InvalidLength => write!(f, "Invalid data length"),
            ParseError::UnknownCommand => write!(f, "Unknown command"),
        }
    }
}

impl std::error::Error for ParseError {}
