// VehicleGroup CQRS 命令对象
// 定义车组写操作命令

/// 创建车组命令
#[derive(Debug, Clone)]
pub struct CreateVehicleGroupCommand {
    pub group_name: String,
    pub parent_id: Option<i32>,
    pub description: Option<String>,
}

/// 更新车组命令
#[derive(Debug, Clone)]
pub struct UpdateVehicleGroupCommand {
    pub group_id: i32,
    pub group_name: Option<String>,
    pub description: Option<String>,
}

/// 删除车组命令
#[derive(Debug, Clone)]
pub struct DeleteVehicleGroupCommand {
    pub group_id: i32,
    pub force: bool,
}

/// 移动车组命令
#[derive(Debug, Clone)]
pub struct MoveVehicleGroupCommand {
    pub group_id: i32,
    pub new_parent_id: Option<i32>,
}

/// 批量绑定车辆命令
#[derive(Debug, Clone)]
pub struct BatchBindVehiclesCommand {
    pub group_id: i32,
    pub vehicle_ids: Vec<i32>,
}
