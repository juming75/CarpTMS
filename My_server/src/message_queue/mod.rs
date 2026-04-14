use crate::domain::ddd::DomainEvent;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// 消息队列接口
#[async_trait]
pub trait MessageQueue {
    /// 发布消息
    async fn publish<T: Serialize>(&self, topic: &str, message: &T) -> anyhow::Result<()>;
    
    /// 订阅消息
    async fn subscribe(&self, topic: &str) -> anyhow::Result<Box<dyn MessageStream>>;
    
    /// 关闭连接
    async fn close(&self) -> anyhow::Result<()>;
}

/// 消息流接口
#[async_trait]
pub trait MessageStream: Send + Sync {
    /// 接收消息
    async fn receive(&mut self) -> anyhow::Result<Option<Vec<u8>>>;
    
    /// 关闭流
    async fn close(&mut self) -> anyhow::Result<()>;
}

/// 事件发布者
#[async_trait]
pub trait EventPublisher {
    /// 发布领域事件
    async fn publish_event(&self, event: &dyn DomainEvent) -> anyhow::Result<()>;
}

/// 事件消费者
#[async_trait]
pub trait EventConsumer {
    /// 消费领域事件
    async fn consume_event(&self, event: &dyn DomainEvent) -> anyhow::Result<()>;
}

/// 消息队列配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MessageQueueConfig {
    pub url: String,
    pub max_retries: usize,
    pub retry_interval: Duration,
    pub queue_size: usize,
}

/// 消息类型
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct QueueMessage {
    pub id: String,
    pub topic: String,
    pub payload: serde_json::Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub headers: std::collections::HashMap<String, String>,
}

/// 消息队列错误
#[derive(Debug, thiserror::Error)]
pub enum MessageQueueError {
    #[error("发布消息失败: {0}")]
    PublishError(#[from] anyhow::Error),
    
    #[error("订阅消息失败: {0}")]
    SubscribeError(#[from] anyhow::Error),
    
    #[error("接收消息失败: {0}")]
    ReceiveError(#[from] anyhow::Error),
    
    #[error("连接失败: {0}")]
    ConnectionError(#[from] anyhow::Error),
    
    #[error("序列化失败: {0}")]
    SerializationError(#[from] serde_json::Error),
}

pub mod redis_queue;
pub mod event_bus;
pub mod consistency;
