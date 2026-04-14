//! Commands - 命令处理模块
//!
//! 命令代表对系统的写操作意图，遵循 CQRS 模式。
//! 每个命令都有对应的处理器负责执行。

pub mod create_order;
pub mod create_vehicle;
pub mod delete_vehicle;
pub mod update_order;
pub mod update_vehicle;

pub use create_order::*;
pub use create_vehicle::*;
pub use delete_vehicle::*;
pub use update_order::*;
pub use update_vehicle::*;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::errors::AppResult;

/// 命令 trait - 所有命令必须实现此 trait
pub trait Command: Send + Sync + Serialize + for<'de> Deserialize<'de> {
    /// 命令类型名称
    fn command_type() -> &'static str;
}

/// 命令处理器 trait
#[async_trait]
pub trait CommandHandler<C: Command>: Send + Sync {
    /// 处理命令
    async fn handle(&self, command: C) -> AppResult<CommandResponse>;
}

/// 命令响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandResponse {
    /// 是否成功
    pub success: bool,
    /// 受影响的记录ID
    pub affected_id: Option<i32>,
    /// 消息
    pub message: Option<String>,
}

impl CommandResponse {
    /// 创建成功响应
    pub fn success(affected_id: i32) -> Self {
        Self {
            success: true,
            affected_id: Some(affected_id),
            message: None,
        }
    }

    /// 创建成功响应（带消息）
    pub fn success_with_message(affected_id: i32, message: impl Into<String>) -> Self {
        Self {
            success: true,
            affected_id: Some(affected_id),
            message: Some(message.into()),
        }
    }

    /// 创建失败响应
    pub fn failure(message: impl Into<String>) -> Self {
        Self {
            success: false,
            affected_id: None,
            message: Some(message.into()),
        }
    }
}
