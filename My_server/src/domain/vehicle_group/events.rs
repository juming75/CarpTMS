// VehicleGroup 领域事件
// 支持车组相关的领域事件

use serde::{Deserialize, Serialize};

/// 车组领域事件
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum VehicleGroupEvent {
    /// 车组创建
    Created {
        group_id: i32,
        group_name: String,
        parent_id: Option<i32>,
    },
    /// 车组更新
    Updated {
        group_id: i32,
        changes: Vec<FieldChange>,
    },
    /// 车组删除
    Deleted { group_id: i32, group_name: String },
    /// 车组关联车辆
    VehicleAssociated { group_id: i32, vehicle_id: i32 },
    /// 车组解绑车辆
    VehicleDissociated { group_id: i32, vehicle_id: i32 },
}

/// 字段变更记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldChange {
    pub field_name: String,
    pub old_value: String,
    pub new_value: String,
}

impl VehicleGroupEvent {
    pub fn get_group_id(&self) -> Option<i32> {
        match self {
            VehicleGroupEvent::Created { group_id, .. } => Some(*group_id),
            VehicleGroupEvent::Updated { group_id, .. } => Some(*group_id),
            VehicleGroupEvent::Deleted { group_id, .. } => Some(*group_id),
            VehicleGroupEvent::VehicleAssociated { group_id, .. } => Some(*group_id),
            VehicleGroupEvent::VehicleDissociated { group_id, .. } => Some(*group_id),
        }
    }
}
