use crate::domain::ddd::DomainEvent;
use crate::message_queue::{MessageQueue, EventPublisher, EventConsumer};
use serde_json::to_string;
use std::sync::Arc;

/// 事件总线
pub struct EventBus {
    message_queue: Arc<dyn MessageQueue>,
}

impl EventBus {
    /// 创建事件总线
    pub fn new(message_queue: Arc<dyn MessageQueue>) -> Self {
        Self {
            message_queue,
        }
    }
    
    /// 启动事件处理器
    pub async fn start_event_processor<T: EventConsumer + Send + Sync>(
        &self, 
        consumer: Arc<T>,
        event_types: Vec<String>,
    ) {
        for event_type in event_types {
            tokio::spawn(Self::process_events(
                self.message_queue.clone(),
                consumer.clone(),
                event_type,
            ));
        }
    }
    
    /// 处理事件
    async fn process_events<T: EventConsumer + Send + Sync>(
        message_queue: Arc<dyn MessageQueue>,
        consumer: Arc<T>,
        event_type: String,
    ) {
        loop {
            match message_queue.subscribe(&event_type).await {
                Ok(mut stream) => {
                    while let Ok(Some(message)) = stream.receive().await {
                        if let Ok(event_json) = serde_json::from_slice::<serde_json::Value>(&message) {
                            // 这里需要根据事件类型创建具体的事件实例
                            // 由于事件类型是动态的，这里需要一个事件工厂
                            // 暂时简化处理
                            println!("Received event: {:?}", event_json);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error subscribing to event {}: {:?}", event_type, e);
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                }
            }
        }
    }
}

#[async_trait::async_trait]
impl EventPublisher for EventBus {
    async fn publish_event(&self, event: &dyn DomainEvent) -> anyhow::Result<()> {
        let event_type = event.event_type();
        let event_json = to_string(event)?;
        
        self.message_queue.publish(&event_type, &event_json).await?;
        Ok(())
    }
}

/// 事件处理器
pub struct EventProcessor {
    event_bus: Arc<EventBus>,
    consumers: Vec<Arc<dyn EventConsumer>>,
}

impl EventProcessor {
    /// 创建事件处理器
    pub fn new(event_bus: Arc<EventBus>) -> Self {
        Self {
            event_bus,
            consumers: vec!(),
        }
    }
    
    /// 添加事件消费者
    pub fn add_consumer(&mut self, consumer: Arc<dyn EventConsumer>) {
        self.consumers.push(consumer);
    }
    
    /// 启动处理器
    pub async fn start(&self, event_types: Vec<String>) {
        for consumer in &self.consumers {
            self.event_bus.start_event_processor(consumer.clone(), event_types.clone()).await;
        }
    }
}
