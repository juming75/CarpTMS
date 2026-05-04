//! 称重命令对象 (CQRS - Write Side)
//!
//! 定义所有写操作命令

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

/// 创建称重记录命令
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateWeighingCommand {
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
    /// 最大载重（用于超载检测）
    pub max_weight: Option<f64>,
    /// 最大速度（用于超速检测）
    pub max_speed: Option<f64>,
}

/// 更新称重记录命令
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateWeighingCommand {
    pub id: i64,
    pub gross_weight: Option<f64>,
    pub tare_weight: Option<f64>,
    pub net_weight: Option<f64>,
    pub status: Option<i32>,
}

/// 删除称重记录命令
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteWeighingCommand {
    pub id: i64,
    /// 是否软删除
    pub soft_delete: bool,
}

/// 批量创建称重记录命令（用于物联网设备高频数据）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchCreateWeighingCommand {
    pub items: Vec<CreateWeighingCommand>,
    /// 批次ID（用于幂等）
    pub batch_id: String,
}

/// 状态变更命令
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeStatusCommand {
    pub id: i64,
    pub new_status: i32,
    pub reason: Option<String>,
}

/// 结算命令
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettleWeighingCommand {
    pub id: i64,
    pub settlement_amount: Option<f64>,
    pub settlement_note: Option<String>,
}

impl CreateWeighingCommand {
    /// 验证命令合法性
    pub fn validate(&self) -> Result<(), String> {
        if self.vehicle_id <= 0 {
            return Err("车辆ID必须大于0".to_string());
        }
        if self.gross_weight <= 0.0 {
            return Err("毛重必须大于0".to_string());
        }
        if self.net_weight <= 0.0 {
            return Err("净重必须大于0".to_string());
        }
        if let Some(tare) = self.tare_weight {
            if tare > self.gross_weight {
                return Err("皮重不能大于毛重".to_string());
            }
        }
        if let Some(speed) = self.speed {
            if speed < 0.0 {
                return Err("速度不能为负".to_string());
            }
        }
        Ok(())
    }
}
