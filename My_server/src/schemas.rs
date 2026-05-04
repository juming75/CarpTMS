use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::ValidationError;

use crate::domain::entities::calibration::SensorCalibration;

fn validate_optional_password(password: &str) -> Result<(), ValidationError> {
    if !password.is_empty() && password.len() < 8 {
        let mut err = ValidationError::new("length");
        err.message = Some("Password must be between 8 and 100 characters".into());
        return Err(err);
    }
    Ok(())
}

// 车组相关的请求/响应模式
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct VehicleGroupCreate {
    pub group_name: String,
    pub parent_id: Option<i32>,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct VehicleGroupUpdate {
    pub group_name: Option<String>,
    pub parent_id: Option<i32>,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct VehicleGroupResponse {
    pub group_id: i32,
    pub group_name: String,
    pub parent_id: Option<i32>,
    pub parent_name: Option<String>,
    pub description: Option<String>,
    pub vehicle_count: i32,
    pub create_time: DateTime<Utc>,
    pub update_time: Option<DateTime<Utc>>,
}

// 车辆相关的请求/响应模式
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct VehicleCreate {
    // 基本信息
    pub vehicle_name: String,
    pub license_plate: String,
    pub vehicle_type: String,
    pub vehicle_color: String,
    pub vehicle_brand: String,
    pub vehicle_model: String,
    pub engine_no: String,
    pub frame_no: String,
    pub register_date: DateTime<Utc>,
    pub inspection_date: DateTime<Utc>,
    pub insurance_date: DateTime<Utc>,
    pub seating_capacity: i32,
    pub load_capacity: f64,
    pub vehicle_length: f64,
    pub vehicle_width: f64,
    pub vehicle_height: f64,

    // 终端信息
    pub device_id: Option<String>,
    pub terminal_type: Option<String>,
    pub communication_type: Option<String>,
    pub sim_card_no: Option<String>,
    pub install_date: Option<DateTime<Utc>>,
    pub install_address: Option<String>,
    pub install_technician: Option<String>,

    // 车主信息
    pub own_no: Option<String>,
    pub own_name: Option<String>,
    pub own_phone: Option<String>,
    pub own_id_card: Option<String>,
    pub own_address: Option<String>,
    pub own_email: Option<String>,

    // 运营信息
    pub group_id: i32,
    pub operation_status: i32,
    pub operation_route: Option<String>,
    pub operation_area: Option<String>,
    pub operation_company: Option<String>,
    pub driver_name: Option<String>,
    pub driver_phone: Option<String>,
    pub driver_license_no: Option<String>,

    // 财务信息
    pub purchase_price: Option<f64>,
    pub annual_fee: Option<f64>,
    pub insurance_fee: Option<f64>,

    // 其他信息
    pub remark: Option<String>,
    pub create_user_id: i32,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct VehicleUpdate {
    // 基本信息
    pub vehicle_name: Option<String>,
    pub license_plate: Option<String>,
    pub vehicle_type: Option<String>,
    pub vehicle_color: Option<String>,
    pub vehicle_brand: Option<String>,
    pub vehicle_model: Option<String>,
    pub engine_no: Option<String>,
    pub frame_no: Option<String>,
    pub register_date: Option<DateTime<Utc>>,
    pub inspection_date: Option<DateTime<Utc>>,
    pub insurance_date: Option<DateTime<Utc>>,
    pub seating_capacity: Option<i32>,
    pub load_capacity: Option<f64>,
    pub vehicle_length: Option<f64>,
    pub vehicle_width: Option<f64>,
    pub vehicle_height: Option<f64>,

    // 终端信息
    pub device_id: Option<String>,
    pub terminal_type: Option<String>,
    pub communication_type: Option<String>,
    pub sim_card_no: Option<String>,
    pub install_date: Option<DateTime<Utc>>,
    pub install_address: Option<String>,
    pub install_technician: Option<String>,

    // 车主信息
    pub own_no: Option<String>,
    pub own_name: Option<String>,
    pub own_phone: Option<String>,
    pub own_id_card: Option<String>,
    pub own_address: Option<String>,
    pub own_email: Option<String>,

    // 运营信息
    pub group_id: Option<i32>,
    pub operation_status: Option<i32>,
    pub operation_route: Option<String>,
    pub operation_area: Option<String>,
    pub operation_company: Option<String>,
    pub driver_name: Option<String>,
    pub driver_phone: Option<String>,
    pub driver_license_no: Option<String>,

    // 财务信息
    pub purchase_price: Option<f64>,
    pub annual_fee: Option<f64>,
    pub insurance_fee: Option<f64>,

    // 其他信息
    pub remark: Option<String>,
    pub status: Option<i32>,
    pub update_user_id: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct VehicleResponse {
    // 基本信息
    pub vehicle_id: i32,
    pub vehicle_name: String,
    pub license_plate: String,
    pub vehicle_type: String,
    pub vehicle_color: String,
    pub vehicle_brand: String,
    pub vehicle_model: String,
    pub engine_no: String,
    pub frame_no: String,
    pub register_date: NaiveDateTime,
    pub inspection_date: NaiveDateTime,
    pub insurance_date: NaiveDateTime,
    pub seating_capacity: i32,
    pub load_capacity: f64,
    pub vehicle_length: f64,
    pub vehicle_width: f64,
    pub vehicle_height: f64,

    // 终端信息
    pub device_id: Option<String>,
    pub terminal_type: Option<String>,
    pub communication_type: Option<String>,
    pub sim_card_no: Option<String>,
    pub install_date: Option<NaiveDateTime>,
    pub install_address: Option<String>,
    pub install_technician: Option<String>,

    // 车主信息
    pub own_no: Option<String>,
    pub own_name: Option<String>,
    pub own_phone: Option<String>,
    pub own_id_card: Option<String>,
    pub own_address: Option<String>,
    pub own_email: Option<String>,

    // 运营信息
    pub group_id: i32,
    pub operation_status: i32,
    pub operation_route: Option<String>,
    pub operation_area: Option<String>,
    pub operation_company: Option<String>,
    pub driver_name: Option<String>,
    pub driver_phone: Option<String>,
    pub driver_license_no: Option<String>,

    // 财务信息
    pub purchase_price: Option<f64>,
    pub annual_fee: Option<f64>,
    pub insurance_fee: Option<f64>,

    // 其他信息
    pub remark: Option<String>,
    pub status: i32,
    pub create_time: NaiveDateTime,
    pub update_time: Option<NaiveDateTime>,
    pub create_user_id: i32,
    pub update_user_id: Option<i32>,
}

// 称重数据相关的请求/响应模式
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct WeighingDataCreate {
    pub vehicle_id: i32,
    pub device_id: String,
    pub weighing_time: NaiveDateTime,
    pub gross_weight: f64,
    pub tare_weight: Option<f64>,
    pub net_weight: f64,
    pub axle_count: Option<i32>,
    pub speed: Option<f64>,
    pub lane_no: Option<i32>,
    pub site_id: Option<i32>,
    pub status: i32,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct WeighingDataResponse {
    pub id: i64,
    pub vehicle_id: i32,
    pub vehicle_name: String,
    pub device_id: String,
    pub weighing_time: DateTime<Utc>,
    pub gross_weight: f64,
    pub tare_weight: Option<f64>,
    pub net_weight: f64,
    pub axle_count: Option<i32>,
    pub speed: Option<f64>,
    pub lane_no: Option<i32>,
    pub site_id: Option<i32>,
    pub status: i32,
    pub create_time: DateTime<Utc>,
    pub update_time: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Default)]
pub struct WeighingHistoryQuery {
    pub start_time: Option<NaiveDateTime>,
    pub end_time: Option<NaiveDateTime>,
    pub vehicle_id: Option<i32>,
    pub site_id: Option<i32>,
    #[serde(default = "default_page")]
    pub page: i32,
    #[serde(default = "default_page_size")]
    pub page_size: i32,
}

fn default_page() -> i32 {
    1
}

fn default_page_size() -> i32 {
    20
}

// 报表相关的请求/响应模式
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ReportGenerateRequest {
    pub template_id: i32,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub period_type: i16,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ReportDataResponse {
    pub id: i32,
    pub template_id: i32,
    pub template_name: String,
    pub report_time: NaiveDateTime,
    pub period_type: i16,
    pub period_value: String,
    pub data: serde_json::Value,
    pub status: i16,
    pub create_time: DateTime<Utc>,
    pub update_time: Option<DateTime<Utc>>,
}

// 数据同步相关的请求/响应模式
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SyncUploadRequest {
    pub table_name: String,
    pub data: Vec<serde_json::Value>,
    pub sync_time: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SyncDownloadRequest {
    pub table_name: String,
    pub last_sync_time: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SyncRequest {
    pub sync_type: String, // upload, download, full_sync
    pub tables: Vec<String>,
    pub last_sync_time: Option<NaiveDateTime>,
    pub data: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SyncResponse {
    pub status: String,
    pub synced_count: i32,
    pub message: String,
    pub sync_id: Option<String>,
}

/// 同步状态 - 匹配数据库sync_status表结构
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SyncStatus {
    pub id: Option<i64>,
    pub sync_type: String,
    pub source_type: String,
    pub status: String,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub total_records: i32,
    pub processed_records: i32,
    pub failed_records: i32,
    pub error_message: Option<String>,
    pub last_sync_time: Option<DateTime<Utc>>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SyncStatusResponse {
    pub last_sync_time: DateTime<Utc>,
    pub sync_count: i32,
    pub success_count: i32,
    pub fail_count: i32,
    pub status: String,
}

// 用户相关的请求/响应模式
#[derive(Debug, Serialize, Deserialize, validator::Validate, ToSchema)]
pub struct UserCreate {
    #[validate(length(
        min = 3,
        max = 50,
        message = "Username must be between 3 and 50 characters"
    ))]
    pub username: String,
    #[validate(length(
        min = 8,
        max = 100,
        message = "Password must be between 8 and 100 characters"
    ))]
    pub password: String,
    #[validate(length(
        min = 2,
        max = 100,
        message = "Full name must be between 2 and 100 characters"
    ))]
    pub full_name: String,
    #[validate(length(
        min = 0,
        max = 20,
        message = "Phone number must be less than 20 characters"
    ))]
    pub phone_number: Option<String>,
    #[validate(email(message = "Invalid email format"))]
    pub email: Option<String>,
    pub user_group_id: i32,
    pub department_id: Option<i32>,
    pub organization_id: Option<String>,
    pub status: i16,
}

#[derive(Debug, Serialize, Deserialize, validator::Validate, ToSchema)]
pub struct UserUpdate {
    #[validate(length(
        min = 3,
        max = 50,
        message = "Username must be between 3 and 50 characters"
    ))]
    pub username: Option<String>,
    #[validate(custom(function = "validate_optional_password"))]
    pub password: Option<String>,
    #[validate(length(
        min = 2,
        max = 100,
        message = "Full name must be between 2 and 100 characters"
    ))]
    pub full_name: Option<String>,
    #[validate(length(
        min = 0,
        max = 20,
        message = "Phone number must be less than 20 characters"
    ))]
    pub phone_number: Option<String>,
    #[validate(email(message = "Invalid email format"))]
    pub email: Option<String>,
    pub user_group_id: Option<i32>,
    pub department_id: Option<i32>,
    pub organization_id: Option<String>,
    pub status: Option<i16>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserResponse {
    pub id: i32,
    pub username: String,
    pub full_name: String,
    pub phone_number: Option<String>,
    pub email: Option<String>,
    pub user_group_id: i32,
    pub user_group_name: Option<String>,
    pub department_id: Option<i32>,
    pub department_name: Option<String>,
    pub organization_id: Option<String>,
    pub organization_name: Option<String>,
    pub status: i16,
    pub last_login_time: Option<DateTime<Utc>>,
    pub create_time: DateTime<Utc>,
    pub update_time: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserQuery {
    pub page: Option<i32>,
    pub page_size: Option<i32>,
    pub username: Option<String>,
    pub full_name: Option<String>,
    pub status: Option<i16>,
    pub user_group_id: Option<i32>,
}

// 设备相关的请求/响应模式
#[derive(Debug, Serialize, Deserialize, validator::Validate, ToSchema)]
pub struct DeviceCreate {
    #[validate(length(min = 1, max = 50))]
    pub device_id: String,
    #[validate(length(min = 1, max = 100))]
    pub device_name: String,
    #[validate(length(min = 1, max = 50))]
    pub device_type: String,
    #[validate(length(min = 1, max = 50))]
    pub device_model: String,
    #[validate(length(min = 1, max = 100))]
    pub manufacturer: String,
    #[validate(length(min = 1, max = 50))]
    pub serial_number: String,
    #[validate(length(min = 1, max = 50))]
    pub communication_type: String,
    pub sim_card_no: Option<String>,
    pub ip_address: Option<String>,
    pub port: Option<i32>,
    pub mac_address: Option<String>,
    pub install_date: Option<NaiveDateTime>,
    pub install_address: Option<String>,
    pub install_technician: Option<String>,
    pub status: i16,
    pub remark: Option<String>,
    pub create_user_id: i32,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct DeviceUpdate {
    pub device_name: Option<String>,
    pub device_type: Option<String>,
    pub device_model: Option<String>,
    pub manufacturer: Option<String>,
    pub serial_number: Option<String>,
    pub communication_type: Option<String>,
    pub sim_card_no: Option<String>,
    pub ip_address: Option<String>,
    pub port: Option<i32>,
    pub mac_address: Option<String>,
    pub install_date: Option<NaiveDateTime>,
    pub install_address: Option<String>,
    pub install_technician: Option<String>,
    pub status: Option<i16>,
    pub remark: Option<String>,
    pub update_user_id: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct DeviceResponse {
    pub device_id: String,
    pub device_name: String,
    pub device_type: String,
    pub device_model: String,
    pub manufacturer: String,
    pub serial_number: String,
    pub communication_type: String,
    pub sim_card_no: Option<String>,
    pub ip_address: Option<String>,
    pub port: Option<i32>,
    pub mac_address: Option<String>,
    pub install_date: Option<DateTime<Utc>>,
    pub install_address: Option<String>,
    pub install_technician: Option<String>,
    pub status: i16,
    pub remark: Option<String>,
    pub create_time: DateTime<Utc>,
    pub update_time: Option<DateTime<Utc>>,
    pub create_user_id: i32,
    pub update_user_id: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct DeviceQuery {
    pub page: Option<i32>,
    pub page_size: Option<i32>,
    pub device_id: Option<String>,
    pub device_name: Option<String>,
    pub device_type: Option<String>,
    pub status: Option<i16>,
    pub manufacturer: Option<String>,
}

// 数据统计相关的请求/响应模式
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct StatisticsQuery {
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub time_range: Option<String>, // day, week, month, year
    pub group_by: Option<String>,   // device, vehicle, group, etc.
    pub device_id: Option<String>,
    pub vehicle_id: Option<i32>,
    pub group_id: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct StatisticsResponse {
    pub total: f64,
    pub count: i64,
    pub average: f64,
    pub min: f64,
    pub max: f64,
    pub data: Vec<StatisticsItem>,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct StatisticsItem {
    pub label: String,
    pub value: f64,
    pub count: i64,
    pub timestamp: Option<DateTime<Utc>>,
    pub details: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct VehicleStatistics {
    pub total_vehicles: i64,
    pub active_vehicles: i64,
    pub inactive_vehicles: i64,
    pub vehicles_by_type: Vec<StatisticsItem>,
    pub vehicles_by_group: Vec<StatisticsItem>,
    pub vehicles_with_devices: i64,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct DeviceStatistics {
    pub total_devices: i64,
    pub online_devices: i64,
    pub offline_devices: i64,
    pub devices_by_type: Vec<StatisticsItem>,
    pub devices_by_manufacturer: Vec<StatisticsItem>,
    pub devices_with_vehicles: i64,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct WeighingStatistics {
    pub total_weight: f64,
    pub total_records: i64,
    pub average_weight: f64,
    pub daily_weighing: Vec<StatisticsItem>,
    pub vehicles_by_weight: Vec<StatisticsItem>,
    pub devices_by_weight: Vec<StatisticsItem>,
}

// 通用响应模式 - 从 bff 模块导入
pub use crate::bff::ApiResponse;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PagedResponse<T> {
    pub list: Vec<T>,
    pub total: i64,
    pub page: i32,
    pub page_size: i32,
    pub pages: i32,
}

// 订单相关的请求/响应模式
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct OrderCreate {
    pub vehicle_id: i32,
    pub driver_id: Option<i32>,
    pub customer_name: String,
    pub customer_phone: String,
    pub origin: String,
    pub destination: String,
    pub cargo_type: String,
    pub cargo_weight: f64,
    pub cargo_volume: Option<f64>,
    pub cargo_count: Option<i32>,
    pub order_amount: f64,
    pub remark: Option<String>,
    pub create_user_id: i32,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct OrderUpdate {
    pub vehicle_id: Option<i32>,
    pub driver_id: Option<i32>,
    pub customer_name: Option<String>,
    pub customer_phone: Option<String>,
    pub origin: Option<String>,
    pub destination: Option<String>,
    pub cargo_type: Option<String>,
    pub cargo_weight: Option<f64>,
    pub cargo_volume: Option<f64>,
    pub cargo_count: Option<i32>,
    pub order_amount: Option<f64>,
    pub order_status: Option<i16>,
    pub departure_time: Option<NaiveDateTime>,
    pub arrival_time: Option<NaiveDateTime>,
    pub remark: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct OrderResponse {
    pub order_id: i32,
    pub order_no: String,
    pub vehicle_id: i32,
    pub vehicle_name: String,
    pub license_plate: String,
    pub driver_id: Option<i32>,
    pub driver_name: Option<String>,
    pub customer_name: String,
    pub customer_phone: String,
    pub origin: String,
    pub destination: String,
    pub cargo_type: String,
    pub cargo_weight: f64,
    pub cargo_volume: Option<f64>,
    pub cargo_count: Option<i32>,
    pub order_amount: f64,
    pub order_status: i16,
    pub order_status_text: String,
    pub departure_time: Option<DateTime<Utc>>,
    pub arrival_time: Option<DateTime<Utc>>,
    pub remark: Option<String>,
    pub create_user_id: i32,
    pub create_time: DateTime<Utc>,
    pub update_time: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct OrderItemCreate {
    pub order_id: i32,
    pub item_name: String,
    pub item_description: Option<String>,
    pub quantity: i32,
    pub unit_price: f64,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct OrderItemUpdate {
    pub item_name: Option<String>,
    pub item_description: Option<String>,
    pub quantity: Option<i32>,
    pub unit_price: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct OrderItemResponse {
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

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct LogisticsTrackCreate {
    pub order_id: i32,
    pub vehicle_id: i32,
    pub track_time: NaiveDateTime,
    pub latitude: f64,
    pub longitude: f64,
    pub address: String,
    pub status: i16,
    pub remark: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct LogisticsTrackUpdate {
    pub vehicle_id: i32,
    pub track_time: NaiveDateTime,
    pub latitude: f64,
    pub longitude: f64,
    pub address: String,
    pub status: i16,
    pub remark: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct LogisticsTrackCreateBatch {
    pub tracks: Vec<LogisticsTrackCreate>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct LogisticsTrackResponse {
    pub track_id: i32,
    pub order_id: i32,
    pub order_no: String,
    pub vehicle_id: i32,
    pub vehicle_name: String,
    pub track_time: DateTime<Utc>,
    pub latitude: f64,
    pub longitude: f64,
    pub address: String,
    pub status: i16,
    pub status_text: String,
    pub remark: Option<String>,
    pub create_time: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct OrderQuery {
    pub page: Option<i32>,
    pub page_size: Option<i32>,
    pub order_no: Option<String>,
    pub vehicle_id: Option<i32>,
    pub driver_id: Option<i32>,
    pub customer_name: Option<String>,
    pub order_status: Option<i16>,
    pub start_time: Option<NaiveDateTime>,
    pub end_time: Option<NaiveDateTime>,
}

// 司机相关的请求/响应模式
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct DriverCreate {
    pub driver_name: String,
    pub license_number: Option<String>,
    pub phone_number: Option<String>,
    pub email: Option<String>,
    pub status: Option<i32>,
    pub license_no: Option<String>,
    pub license_type: Option<String>,
    pub license_expiry: Option<String>,
    pub id_card: Option<String>,
    pub address: Option<String>,
    pub emergency_contact: Option<String>,
    pub emergency_phone: Option<String>,
    pub hire_date: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct DriverUpdate {
    pub driver_name: Option<String>,
    pub license_number: Option<String>,
    pub phone_number: Option<String>,
    pub email: Option<String>,
    pub status: Option<i32>,
    pub license_no: Option<String>,
    pub license_type: Option<String>,
    pub license_expiry: Option<String>,
    pub id_card: Option<String>,
    pub address: Option<String>,
    pub emergency_contact: Option<String>,
    pub emergency_phone: Option<String>,
    pub hire_date: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct DriverResponse {
    pub driver_id: i32,
    pub driver_name: String,
    pub license_number: String,
    pub phone_number: Option<String>,
    pub email: Option<String>,
    pub status: i32,
    pub create_time: String,
    pub update_time: Option<String>,
    pub license_no: Option<String>,
    pub license_type: Option<String>,
    pub license_expiry: Option<String>,
    pub id_card: Option<String>,
    pub address: Option<String>,
    pub emergency_contact: Option<String>,
    pub emergency_phone: Option<String>,
    pub hire_date: Option<String>,
}

// 财务相关的请求/响应模式
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct FinanceCostCreate {
    pub cost_type: String,
    pub amount: f64,
    pub description: String,
    pub cost_date: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct FinanceCostUpdate {
    pub cost_type: Option<String>,
    pub amount: Option<f64>,
    pub description: Option<String>,
    pub cost_date: Option<NaiveDateTime>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct FinanceCostResponse {
    pub cost_id: i32,
    pub cost_type: String,
    pub amount: f64,
    pub description: String,
    pub cost_date: String,
    pub create_time: DateTime<Utc>,
    pub update_time: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct FinanceInvoiceCreate {
    pub invoice_number: String,
    pub amount: f64,
    pub invoice_date: NaiveDateTime,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct FinanceInvoiceUpdate {
    pub invoice_number: Option<String>,
    pub amount: Option<f64>,
    pub invoice_date: Option<NaiveDateTime>,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct FinanceInvoiceResponse {
    pub invoice_id: i32,
    pub invoice_number: String,
    pub amount: f64,
    pub invoice_date: String,
    pub description: String,
    pub create_time: DateTime<Utc>,
    pub update_time: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct FinanceStatisticsResponse {
    pub total_cost: f64,
    pub total_invoice: f64,
    pub balance: f64,
}

// 载重标定相关的请求/响应模式
#[derive(Debug, Serialize, Deserialize, validator::Validate, ToSchema)]
pub struct CalibrationCreate {
    pub sensor_no: i32,
    pub vehicle_id: i32,
    pub plate_no: String,
    pub sensor_side: String,
    pub sensor_group: Option<i32>,
    pub self_weight: Option<i32>,
    pub polynomial_json: String,
    pub linear_segments_json: Option<String>,
    pub is_calibrated: bool,
    // DDD 改造：支持标定点和多项式拟合计算
    pub calibration_points: Option<serde_json::Value>, // 前端提交的标定点数组
    pub pa_raw: Option<i32>,                           // 原始 AD 值
    pub axle_number: Option<i32>,                      // 轴号 1-3
    pub is_left_wheel: Option<bool>,                   // 是否左侧轮
    pub turn_point: Option<i32>,                       // 转折点 AD 值（默认 50000）
    pub polynomial_order: Option<i32>,                 // 多项式阶数：1=线性，2=二阶
    pub rated_total_weight: Option<f64>,               // 额定总重 kg
    pub tare_weight: Option<f64>,                      // 空车自重 kg
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CalibrationUpdate {
    pub vehicle_id: Option<i32>,
    pub plate_no: Option<String>,
    pub sensor_side: Option<String>,
    pub sensor_group: Option<i32>,
    pub self_weight: Option<i32>,
    pub polynomial_json: Option<String>,
    pub linear_segments_json: Option<String>,
    pub is_calibrated: Option<bool>,
    pub calibration_points: Option<serde_json::Value>,
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

impl From<SensorCalibration> for CalibrationResponse {
    fn from(c: SensorCalibration) -> Self {
        let calibration_points_value = c
            .calibration_points
            .as_ref()
            .and_then(|s| serde_json::from_str(s).ok());

        CalibrationResponse {
            id: c.id,
            sensor_no: c.sensor_no,
            vehicle_id: c.vehicle_id,
            plate_no: c.plate_no,
            sensor_side: c.sensor_side,
            sensor_group: c.sensor_group,
            self_weight: c.self_weight,
            polynomial_json: c.polynomial_json,
            linear_segments_json: c.linear_segments_json,
            is_calibrated: c.is_calibrated,
            create_time: c.create_time,
            update_time: c.update_time,
            calibration_points: calibration_points_value,
            pa_raw: c.pa_raw,
            axle_number: c.axle_number,
            is_left_wheel: c.is_left_wheel,
            turn_point: c.turn_point,
            polynomial_order: c.polynomial_order,
            polynomial_coefs_json: None,
            r2_score: c.r2_score,
            rmse: c.rmse,
            max_error: c.max_error,
            point_count: c.point_count,
            rated_total_weight: c.rated_total_weight,
            tare_weight: c.tare_weight,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CalibrationResponse {
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
    // DDD 改造字段
    pub calibration_points: Option<serde_json::Value>,
    pub pa_raw: Option<i32>,
    pub axle_number: Option<i32>,
    pub is_left_wheel: Option<bool>,
    pub turn_point: Option<i32>,
    pub polynomial_order: Option<i32>,
    pub polynomial_coefs_json: Option<String>,
    pub r2_score: Option<f64>,
    pub rmse: Option<f64>,
    pub max_error: Option<f64>,
    pub point_count: Option<i32>,
    pub rated_total_weight: Option<f64>,
    pub tare_weight: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CalibrationQuery {
    pub page: Option<i32>,
    pub page_size: Option<i32>,
    pub sensor_no: Option<i32>,
    pub vehicle_id: Option<i32>,
    pub plate_no: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CalibrationHistoryResponse {
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

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct VehicleTracksQuery {
    pub vehicle_id: i32,
    pub start_time: String,
    pub end_time: String,
    pub page: Option<i32>,
    pub page_size: Option<i32>,
}
