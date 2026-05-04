//! Event Bus - 事件总线模块
//!
//! 提供事件发布订阅机制，实现模块间的松耦合通信。

pub mod in_memory;

pub use in_memory::*;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::errors::AppResult;

/// 事件 trait
pub trait Event: Send + Sync + Serialize + for<'de> Deserialize<'de> {
    /// 事件类型
    fn event_type() -> &'static str;

    /// 事件ID
    fn event_id(&self) -> &str;

    /// 事件时间戳
    fn timestamp(&self) -> i64;
}

/// 事件处理器 trait
#[async_trait]
pub trait EventHandler<E: Event>: Send + Sync {
    /// 处理事件
    async fn handle(&self, event: E) -> AppResult<()>;

    /// 处理器名称
    fn name(&self) -> &str;
}

/// 事件总线 trait
#[async_trait]
pub trait EventBus: Send + Sync {
    /// 发布事件
    async fn publish<E: Event>(&self, event: E) -> AppResult<()>;

    /// 订阅事件类型
    async fn subscribe<E: Event>(&self) -> AppResult<EventSubscription<E>>;
}

/// 事件订阅
pub struct EventSubscription<E: Event> {
    receiver: tokio::sync::broadcast::Receiver<Vec<u8>>,
    _phantom: std::marker::PhantomData<E>,
}

impl<E: Event> EventSubscription<E> {
    /// 创建事件订阅
    pub fn new(receiver: tokio::sync::broadcast::Receiver<Vec<u8>>) -> Self {
        Self {
            receiver,
            _phantom: std::marker::PhantomData,
        }
    }

    /// 接收下一个事件
    pub async fn recv(&mut self) -> AppResult<E> {
        loop {
            match self.receiver.recv().await {
                Ok(payload) => {
                    // 反序列化事件
                    let event: E = serde_json::from_slice(&payload).map_err(|e| {
                        crate::errors::AppError::internal_error(
                            &format!("Failed to deserialize event: {}", e),
                            None,
                        )
                    })?;
                    return Ok(event);
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                    log::warn!("Event subscription lagged, {} events skipped", n);
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                    return Err(crate::errors::AppError::internal_error(
                        "Event channel closed",
                        None,
                    ));
                }
            }
        }
    }
}

/// 事件信封（包装事件用于传输）
#[derive(Debug, Clone, Serialize)]
pub struct EventEnvelope<E: Event> {
    /// 事件
    pub event: E,
    /// 元数据
    pub metadata: EventMetadata,
}

/// 事件元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMetadata {
    /// 事件ID
    pub id: String,
    /// 事件类型
    pub event_type: String,
    /// 事件来源
    pub source: String,
    /// 时间戳
    pub timestamp: i64,
    /// 版本
    pub version: String,
}

impl Default for EventMetadata {
    fn default() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            event_type: String::new(),
            source: "unknown".to_string(),
            timestamp: chrono::Utc::now().timestamp(),
            version: "1.0".to_string(),
        }
    }
}
