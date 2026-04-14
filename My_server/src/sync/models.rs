//! / 数据模型 - 旧服务器格式
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// LegacyDevice 是 LegacyVehicle 的别名(向后兼容)
pub type LegacyDevice = LegacyVehicle;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct LegacyVehicle {
    pub VehicleID: String,
    pub PlateNumber: String,
    pub DeviceID: String,
    pub VehicleType: String,
    pub Status: i32,
    pub Phone: Option<String>,
    pub SIMCard: Option<String>,
    pub InstallDate: Option<String>,
    pub ExpireDate: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct LegacyGpsData {
    pub DeviceID: String,
    pub Latitude: f64,
    pub Longitude: f64,
    pub Speed: f64,
    pub Direction: f64,
    pub Altitude: f64,
    pub GPSDateTime: String,
    pub Status: i32,
    pub SatelliteCount: i32,
    pub IOStatus: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct LegacyUser {
    pub UserID: String,
    pub Username: String,
    pub Password: String,             // 注意: 实际应该是加密的
    pub PasswordHash: Option<String>, // 密码哈希
    pub RealName: String,
    pub Phone: Option<String>,
    pub Email: Option<String>,
    pub Role: String,
    pub Department: Option<String>,
    pub GroupID: Option<i32>, // 用户组ID
    pub Status: Option<i32>,  // 状态
}

// 新服务器同步模型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncVehicle {
    pub vehicle_id: String,
    pub plate_number: String,
    pub device_id: Option<String>,
    pub vehicle_type: String,
    pub status: String,
    pub phone: Option<String>,
    pub sim_card: Option<String>,
    pub install_date: Option<DateTime<Utc>>,
    pub expire_date: Option<DateTime<Utc>>,
    pub source: String,
    pub legacy_vehicle_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncGpsData {
    pub device_id: String,
    pub latitude: f64,
    pub longitude: f64,
    pub speed: f64,
    pub direction: f64,
    pub altitude: f64,
    pub timestamp: DateTime<Utc>,
    pub status: i32,
    pub satellite_count: i32,
    pub io_status: Option<String>,
    pub source: String,
    pub received_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncUser {
    pub username: String,
    pub email: Option<String>,
    pub password_hash: Option<String>,
    pub role: String,
    pub phone: Option<String>,
    pub department: Option<String>,
    pub source: String,
    pub legacy_user_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// 辅助函数 - 解析日期时间
pub fn parse_date_utc(s: &Option<String>) -> Option<DateTime<Utc>> {
    match s {
        Some(date_str) => chrono::NaiveDateTime::parse_from_str(date_str, "%Y-%m-%d %H:%M:%S")
            .or_else(|_| {
                chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
                    .map(|d| d.and_hms_opt(0, 0, 0).expect("midnight always valid"))
            })
            .ok()
            .map(|dt| DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc)),
        None => None,
    }
}

pub fn parse_datetime(dt_str: &str) -> Result<DateTime<Utc>, String> {
    // 尝试多种日期格式
    let formats = [
        "%Y-%m-%d %H:%M:%S",
        "%Y-%m-%dT%H:%M:%S",
        "%Y-%m-%dT%H:%M:%SZ",
        "%Y/%m/%d %H:%M:%S",
        "%Y-%m-%d %H:%M:%S%.f",
        "%Y-%m-%dT%H:%M:%S%.f",
    ];

    for format in &formats {
        if let Ok(naive) = chrono::NaiveDateTime::parse_from_str(dt_str, format) {
            return Ok(DateTime::<Utc>::from_naive_utc_and_offset(naive, Utc));
        }
    }

    Err(format!("无法解析日期时间: {}", dt_str))
}
