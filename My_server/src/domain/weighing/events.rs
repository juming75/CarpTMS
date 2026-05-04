//! 称重领域事件
//!
//! 定义称重领域的领域事件，支持事件溯源

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

/// 称重领域事件枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum WeighingEvent {
    /// 称重数据创建
    Created {
        vehicle_id: i32,
        device_id: String,
        gross_weight: f64,
        net_weight: f64,
        weighing_time: NaiveDateTime,
    },

    /// 称重数据更新
    Updated {
        weighing_id: i64,
        changes: Vec<WeighingFieldChange>,
    },

    /// 称重数据删除
    Deleted { weighing_id: i64, vehicle_id: i32 },

    /// 称重状态变更
    StatusChanged {
        weighing_id: i64,
        old_status: i32,
        new_status: i32,
    },

    /// 异常告警
    AbnormalDetected {
        weighing_id: i64,
        vehicle_id: i32,
        abnormal_type: AbnormalType,
        description: String,
    },

    /// 超载告警
    OverloadWarning {
        weighing_id: i64,
        vehicle_id: i32,
        gross_weight: f64,
        max_weight: f64,
    },
}

/// 字段变更记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeighingFieldChange {
    pub field_name: String,
    pub old_value: String,
    pub new_value: String,
}

/// 异常类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AbnormalType {
    /// 超速
    Overspeed,
    /// 超载
    Overload,
    /// 重复称重
    DuplicateWeighing,
    /// 数据异常
    DataAbnormal,
    /// 设备故障
    DeviceFault,
}

impl WeighingEvent {
    /// 获取事件发生的聚合根ID
    pub fn get_aggregate_id(&self) -> Option<i64> {
        match self {
            WeighingEvent::Created { .. } => None,
            WeighingEvent::Updated { weighing_id, .. } => Some(*weighing_id),
            WeighingEvent::Deleted { weighing_id, .. } => Some(*weighing_id),
            WeighingEvent::StatusChanged { weighing_id, .. } => Some(*weighing_id),
            WeighingEvent::AbnormalDetected { weighing_id, .. } => Some(*weighing_id),
            WeighingEvent::OverloadWarning { weighing_id, .. } => Some(*weighing_id),
        }
    }
}
