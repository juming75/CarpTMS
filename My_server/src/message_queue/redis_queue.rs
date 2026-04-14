use crate::message_queue::{MessageQueue, MessageStream, MessageQueueConfig, QueueMessage};
use redis::{aio::ConnectionManager, AsyncCommands};
use serde_json::to_vec;
use std::sync::Arc;
use uuid::Uuid;

/// Redis消息队列实现
pub struct RedisMessageQueue {
    connection: Arc<ConnectionManager>,
    config: MessageQueueConfig,
}

impl RedisMessageQueue {
    /// 创建Redis消息队列
    pub async fn new(config: MessageQueueConfig) -> anyhow::Result<Self> {
        let client = redis::Client::open(config.url.clone())?;
        let connection = ConnectionManager::new(client).await?;
        
        Ok(Self {
            connection: Arc::new(connection),
            config,
        })
    }
}

#[async_trait::async_trait]
impl MessageQueue for RedisMessageQueue {
    async fn publish<T: serde::Serialize>(&self, topic: &str, message: &T) -> anyhow::Result<()> {
        let connection = self.connection.clone();
        let payload = to_vec(message)?;
        
        let queue_message = QueueMessage {
            id: Uuid::new_v4().to_string(),
            topic: topic.to_string(),
            payload: serde_json::from_slice(&payload)?,
            timestamp: chrono::Utc::now(),
            headers: std::collections::HashMap::new(),
        };
        
        let message_json = serde_json::to_string(&queue_message)?;
        connection.rpush(topic, message_json).await?;
        
        Ok(())
    }
    
    async fn subscribe(&self, topic: &str) -> anyhow::Result<Box<dyn MessageStream>> {
        Ok(Box::new(RedisMessageStream {
            connection: self.connection.clone(),
            topic: topic.to_string(),
        }))
    }
    
    async fn close(&self) -> anyhow::Result<()> {
        // Redis连接管理器会自动处理连接关闭
        Ok(())
    }
}

/// Redis消息流实现
pub struct RedisMessageStream {
    connection: Arc<ConnectionManager>,
    topic: String,
}

#[async_trait::async_trait]
impl MessageStream for RedisMessageStream {
    async fn receive(&mut self) -> anyhow::Result<Option<Vec<u8>>> {
        let connection = self.connection.clone();
        let topic = self.topic.clone();
        
        // 使用BLPOP命令阻塞等待消息
        let result: Option<(String, String)> = connection.blpop(&topic, 0).await?;
        
        match result {
            Some((_, message)) => Ok(Some(message.into_bytes())),
            None => Ok(None),
        }
    }
    
    async fn close(&mut self) -> anyhow::Result<()> {
        Ok(())
    }
}
