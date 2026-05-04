//! 内存事件总线实现

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

use super::{Event, EventBus};
use crate::errors::{AppError, AppResult};

/// 内存事件总线
pub struct InMemoryEventBus {
    /// 事件通道映射
    channels: Arc<RwLock<HashMap<String, broadcast::Sender<Vec<u8>>>>>,
    /// 默认通道容量
    capacity: usize,
}

impl InMemoryEventBus {
    /// 创建新的内存事件总线
    pub fn new() -> Self {
        Self {
            channels: Arc::new(RwLock::new(HashMap::new())),
            capacity: 1000,
        }
    }

    /// 创建指定容量的内存事件总线
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            channels: Arc::new(RwLock::new(HashMap::new())),
            capacity,
        }
    }

    /// 获取或创建事件通道
    async fn get_channel<E: Event>(&self) -> broadcast::Sender<Vec<u8>> {
        let event_type = E::event_type();

        // 先尝试读取
        {
            let channels = self.channels.read().await;
            if let Some(sender) = channels.get(event_type) {
                return sender.clone();
            }
        }

        // 需要创建新通道
        let mut channels = self.channels.write().await;
        let (sender, _) = broadcast::channel(self.capacity);
        channels.insert(event_type.to_string(), sender.clone());
        log::info!("Created event channel for: {}", event_type);
        sender
    }
}

impl Default for InMemoryEventBus {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl EventBus for InMemoryEventBus {
    async fn publish<E: Event>(&self, event: E) -> AppResult<()> {
        let event_type = E::event_type();

        // 序列化事件
        let payload = serde_json::to_vec(&event).map_err(|e| {
            AppError::internal_error(&format!("Failed to serialize event: {}", e), None)
        })?;

        // 获取通道
        let channel = self.get_channel::<E>().await;

        // 发送事件
        match channel.send(payload) {
            Ok(_) => {
                log::debug!("Event published: {}", event_type);
                Ok(())
            }
            Err(e) => {
                log::error!("Failed to publish event {}: {}", event_type, e);
                Err(AppError::internal_error(
                    &format!("Failed to publish event: {}", e),
                    None,
                ))
            }
        }
    }

    async fn subscribe<E: Event>(&self) -> AppResult<super::EventSubscription<E>> {
        let channel = self.get_channel::<E>().await;
        log::info!("Subscribed to event: {}", E::event_type());

        Ok(super::EventSubscription::new(channel.subscribe()))
    }
}

/// 全局事件总线实例
static GLOBAL_EVENT_BUS: tokio::sync::OnceCell<InMemoryEventBus> =
    tokio::sync::OnceCell::const_new();

/// 初始化全局事件总线
pub async fn init_global_event_bus() {
    GLOBAL_EVENT_BUS
        .get_or_init(|| async { InMemoryEventBus::new() })
        .await;
}

/// 获取全局事件总线
pub fn global_event_bus() -> &'static InMemoryEventBus {
    GLOBAL_EVENT_BUS
        .get()
        .expect("Event bus not initialized. Call init_global_event_bus() first.")
}

/// 发布事件到全局总线
pub async fn publish_event<E: Event>(event: E) -> AppResult<()> {
    global_event_bus().publish(event).await
}

/// 订阅全局事件
pub async fn subscribe_event<E: Event>() -> AppResult<super::EventSubscription<E>> {
    global_event_bus().subscribe::<E>().await
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct TestEvent {
        id: String,
        data: String,
        timestamp: i64,
    }

    impl Event for TestEvent {
        fn event_type() -> &'static str {
            "test_event"
        }

        fn event_id(&self) -> &str {
            &self.id
        }

        fn timestamp(&self) -> i64 {
            self.timestamp
        }
    }

    #[tokio::test]
    async fn test_publish_and_subscribe() {
        let bus = InMemoryEventBus::new();

        // 订阅事件
        let mut subscription = bus.subscribe::<TestEvent>().await.unwrap();

        // 发布事件
        let event = TestEvent {
            id: "test-1".to_string(),
            data: "hello".to_string(),
            timestamp: chrono::Utc::now().timestamp(),
        };
        bus.publish(event.clone()).await.unwrap();

        // 接收事件
        let received = subscription.recv().await.unwrap();
        assert_eq!(received.id, event.id);
        assert_eq!(received.data, event.data);
    }
}
