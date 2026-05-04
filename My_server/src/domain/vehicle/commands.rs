// Vehicle CQRS 命令对象
// 定义车辆写操作命令

use chrono::NaiveDate;

/// 创建车辆命令
#[derive(Debug, Clone)]
pub struct CreateVehicleCommand {
    pub vehicle_name: String,
    pub license_plate: String,
    pub vehicle_type: String,
    pub vehicle_color: Option<String>,
    pub vehicle_brand: Option<String>,
    pub vehicle_model: Option<String>,
    pub engine_no: Option<String>,
    pub frame_no: Option<String>,
    pub register_date: NaiveDate,
    pub inspection_date: NaiveDate,
    pub insurance_date: NaiveDate,
    pub load_capacity: Option<f64>,
    pub status: i32,
    pub create_user_id: Option<i32>,
}

/// 更新车辆命令
#[derive(Debug, Clone)]
pub struct UpdateVehicleCommand {
    pub vehicle_id: i32,
    pub vehicle_name: Option<String>,
    pub license_plate: Option<String>,
    pub inspection_date: Option<NaiveDate>,
    pub insurance_date: Option<NaiveDate>,
    pub status: Option<i32>,
    pub update_user_id: Option<i32>,
}

/// 删除车辆命令
#[derive(Debug, Clone)]
pub struct DeleteVehicleCommand {
    pub vehicle_id: i32,
    pub force: bool,
}

/// 批量更新车辆状态命令
#[derive(Debug, Clone)]
pub struct BatchUpdateStatusCommand {
    pub vehicle_ids: Vec<i32>,
    pub new_status: i32,
    pub update_user_id: Option<i32>,
}
