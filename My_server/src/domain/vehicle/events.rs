// Vehicle 领域事件
// 支持车辆相关的领域事件，用于事件日志和后续事件溯源

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

/// 车辆领域事件
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum VehicleEvent {
    /// 车辆创建
    Created {
        vehicle_id: i32,
        vehicle_name: String,
        license_plate: String,
        vehicle_type: String,
    },
    /// 车辆更新
    Updated {
        vehicle_id: i32,
        changes: Vec<FieldChange>,
    },
    /// 车辆删除
    Deleted {
        vehicle_id: i32,
        vehicle_name: String,
    },
    /// 车辆状态变更
    StatusChanged {
        vehicle_id: i32,
        old_status: i32,
        new_status: i32,
    },
    /// 车辆年检过期告警
    InspectionExpired {
        vehicle_id: i32,
        license_plate: String,
        expired_date: NaiveDateTime,
    },
    /// 车辆保险过期告警
    InsuranceExpired {
        vehicle_id: i32,
        license_plate: String,
        expired_date: NaiveDateTime,
    },
}

/// 字段变更记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldChange {
    pub field_name: String,
    pub old_value: String,
    pub new_value: String,
}

impl VehicleEvent {
    /// 获取车辆ID
    pub fn get_vehicle_id(&self) -> Option<i32> {
        match self {
            VehicleEvent::Created { vehicle_id, .. } => Some(*vehicle_id),
            VehicleEvent::Updated { vehicle_id, .. } => Some(*vehicle_id),
            VehicleEvent::Deleted { vehicle_id, .. } => Some(*vehicle_id),
            VehicleEvent::StatusChanged { vehicle_id, .. } => Some(*vehicle_id),
            VehicleEvent::InspectionExpired { vehicle_id, .. } => Some(*vehicle_id),
            VehicleEvent::InsuranceExpired { vehicle_id, .. } => Some(*vehicle_id),
        }
    }
}
