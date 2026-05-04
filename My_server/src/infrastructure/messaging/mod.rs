//! Messaging - 消息队列接口模块
//!
//! 提供消息队列的抽象接口，支持不同的消息队列实现。

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::errors::AppResult;

/// 消息 trait
pub trait Message: Send + Sync + Serialize + for<'de> Deserialize<'de> {
    /// 消息ID
    fn message_id(&self) -> &str;

    /// 消息类型
    fn message_type() -> &'static str;
}

/// 消息队列 trait
#[async_trait]
pub trait MessageQueue: Send + Sync {
    /// 发送消息
    async fn send<M: Message>(&self, queue: &str, message: M) -> AppResult<()>;

    /// 接收消息
    async fn receive<M: Message>(&self, queue: &str) -> AppResult<Option<M>>;

    /// 确认消息
    async fn ack(&self, queue: &str, message_id: &str) -> AppResult<()>;

    /// 拒绝消息
    async fn nack(&self, queue: &str, message_id: &str) -> AppResult<()>;
}

/// 消息队列配置
#[derive(Debug, Clone)]
pub struct MessageQueueConfig {
    /// 队列名称
    pub queue_name: String,
    /// 最大重试次数
    pub max_retries: u32,
    /// 重试延迟（毫秒）
    pub retry_delay_ms: u64,
    /// 死信队列名称
    pub dead_letter_queue: Option<String>,
}

impl Default for MessageQueueConfig {
    fn default() -> Self {
        Self {
            queue_name: "default".to_string(),
            max_retries: 3,
            retry_delay_ms: 1000,
            dead_letter_queue: None,
        }
    }
}

/// 内存消息队列实现
pub struct InMemoryMessageQueue {
    queues: Arc<tokio::sync::RwLock<std::collections::HashMap<String, Vec<Vec<u8>>>>>,
}

impl InMemoryMessageQueue {
    /// 创建新的内存消息队列
    pub fn new() -> Self {
        Self {
            queues: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
        }
    }
}

impl Default for InMemoryMessageQueue {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl MessageQueue for InMemoryMessageQueue {
    async fn send<M: Message>(&self, queue: &str, message: M) -> AppResult<()> {
        let payload = serde_json::to_vec(&message).map_err(|e| {
            crate::errors::AppError::internal_error(
                &format!("Failed to serialize message: {}", e),
                None,
            )
        })?;

        let mut queues = self.queues.write().await;
        queues
            .entry(queue.to_string())
            .or_insert_with(Vec::new)
            .push(payload);

        log::debug!("Message sent to queue: {}", queue);
        Ok(())
    }

    async fn receive<M: Message>(&self, queue: &str) -> AppResult<Option<M>> {
        let mut queues = self.queues.write().await;

        if let Some(messages) = queues.get_mut(queue) {
            if let Some(payload) = messages.first().cloned() {
                let message: M = serde_json::from_slice(&payload).map_err(|e| {
                    crate::errors::AppError::internal_error(
                        &format!("Failed to deserialize message: {}", e),
                        None,
                    )
                })?;
                messages.remove(0);
                return Ok(Some(message));
            }
        }

        Ok(None)
    }

    async fn ack(&self, _queue: &str, _message_id: &str) -> AppResult<()> {
        // 内存实现不需要确认
        Ok(())
    }

    async fn nack(&self, _queue: &str, _message_id: &str) -> AppResult<()> {
        // 内存实现不需要拒绝
        Ok(())
    }
}

/// 消息处理器 trait
#[async_trait]
pub trait MessageHandler<M: Message>: Send + Sync {
    /// 处理消息
    async fn handle(&self, message: M) -> AppResult<()>;

    /// 处理器名称
    fn name(&self) -> &str;
}

/// 消息消费者
pub struct MessageConsumer<M: Message> {
    queue: String,
    handler: Arc<dyn MessageHandler<M>>,
    queue_impl: Arc<InMemoryMessageQueue>,
}

impl<M: Message> MessageConsumer<M> {
    /// 创建新的消息消费者
    pub fn new(
        queue: impl Into<String>,
        handler: Arc<dyn MessageHandler<M>>,
        queue_impl: Arc<InMemoryMessageQueue>,
    ) -> Self {
        Self {
            queue: queue.into(),
            handler,
            queue_impl,
        }
    }

    /// 开始消费消息
    pub async fn start(&self) -> AppResult<()> {
        loop {
            match self.queue_impl.receive::<M>(&self.queue).await {
                Ok(Some(message)) => {
                    if let Err(e) = self.handler.handle(message).await {
                        log::error!("Message handler error: {}", e);
                    }
                }
                Ok(None) => {
                    // 没有消息，等待一段时间
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                }
                Err(e) => {
                    log::error!("Message receive error: {}", e);
                    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
                }
            }
        }
    }
}

/// 消息生产者
pub struct MessageProducer {
    queue_impl: Arc<InMemoryMessageQueue>,
}

impl MessageProducer {
    /// 创建新的消息生产者
    pub fn new(queue_impl: Arc<InMemoryMessageQueue>) -> Self {
        Self { queue_impl }
    }

    /// 发送消息
    pub async fn send<M: Message>(&self, queue: &str, message: M) -> AppResult<()> {
        self.queue_impl.send(queue, message).await
    }
}
