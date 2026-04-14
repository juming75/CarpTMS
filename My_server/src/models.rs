use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

// 车辆模型
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Vehicle {
    pub id: i32,
    pub vehicle_id: String,
    pub plate_number: String,
    pub vehicle_type: String,
    pub brand: String,
    pub model: String,
    pub year: i32,
    pub vin: String,
    pub engine_number: String,
    pub owner_id: i32,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// 司机模型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Driver {
    pub id: i32,
    pub driver_id: String,
    pub name: String,
    pub license_number: String,
    pub phone: String,
    pub address: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// 订单模型
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Order {
    pub order_id: i32,
    pub order_no: String,
    pub vehicle_id: i32,
    pub driver_id: Option<i32>,
    pub customer_name: String,
    pub customer_phone: String,
    pub origin: String,
    pub destination: String,
    pub cargo_type: String,
    pub cargo_weight: f64,
    pub cargo_volume: f64,
    pub cargo_count: i32,
    pub order_amount: f64,
    pub order_status: i16,
    pub departure_time: Option<DateTime<Utc>>,
    pub arrival_time: Option<DateTime<Utc>>,
    pub remark: Option<String>,
    pub create_user_id: i32,
    pub create_time: DateTime<Utc>,
    pub update_time: Option<DateTime<Utc>>,
}

// 订单明细模型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderItem {
    pub item_id: i32,
    pub order_id: i32,
    pub item_name: String,
    pub item_description: Option<String>,
    pub quantity: i32,
    pub unit_price: f64,
    pub total_price: f64,
    pub create_time: DateTime<Utc>,
    pub update_time: Option<DateTime<Utc>>,
}

// 车辆实时位置模型
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct VehicleRealtimeLocation {
    pub id: i64,
    pub vehicle_id: i32,
    pub latitude: f64,
    pub longitude: f64,
    pub speed: f64,
    pub direction: i32,
    pub altitude: f64,
    pub accuracy: Option<f64>,
    pub status: i32,
    pub timestamp: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

// 告警模型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alarm {
    pub id: i32,
    pub vehicle_id: i32,
    pub alarm_type: String,
    pub alarm_level: String,
    pub alarm_content: String,
    pub status: String,
    pub timestamp: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// 用户模型
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub password_hash: String,
    pub name: String,
    pub role: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}



// 物流轨迹模型
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct LogisticsTrack {
    pub track_id: i32,
    pub order_id: i32,
    pub vehicle_id: i32,
    pub track_time: Option<chrono::DateTime<chrono::Utc>>,
    pub latitude: f64,
    pub longitude: f64,
    pub address: Option<String>,
    pub status: i32,
    pub remark: Option<String>,
    pub create_time: chrono::DateTime<chrono::Utc>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

// 报表模板模型
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ReportTemplate {
    pub id: i32,
    pub name: String,
    pub template_type: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// 审计日志模型
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AuditLog {
    pub id: i64,
    pub user_id: i32,
    pub action: String,
    pub resource: String,
    pub ip_address: String,
    pub user_agent: String,
    pub created_at: DateTime<Utc>,
}
