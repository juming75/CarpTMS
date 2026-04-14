//! /! 事件驱动架构模块
//!
//! 提供事件总线、事件发布订阅机制,实现模块间的松耦合

use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, RwLock};
use tokio::time::sleep;

// CQRS架构模块
pub mod cqrs;

// 事件存储服务(事件溯源)
pub mod event_store;

/// 事件总线
pub struct EventBus {
    /// 事件通道映射
    channels: Arc<RwLock<HashMap<String, broadcast::Sender<Vec<u8>>>>>,
    /// 事件存储引用
    event_store: Option<Arc<event_store::EventStore>>,
    /// 是否启用事件持久化
    enable_persistence: bool,
    /// 事件发布重试次数
    max_retries: usize,
    /// 重试间隔
    retry_interval: Duration,
}

impl EventBus {
    /// 创建新的事件总线
    pub fn new() -> Self {
        Self {
            channels: Arc::new(RwLock::new(HashMap::new())),
            event_store: None,
            enable_persistence: false,
            max_retries: 3,
            retry_interval: Duration::from_millis(100),
        }
    }

    /// 创建带配置的事件总线
    pub fn with_config(enable_persistence: bool, max_retries: usize, retry_interval: Duration) -> Self {
        Self {
            channels: Arc::new(RwLock::new(HashMap::new())),
            event_store: None,
            enable_persistence,
            max_retries,
            retry_interval,
        }
    }

    /// 设置事件存储
    pub fn with_event_store(self, event_store: Arc<event_store::EventStore>) -> Self {
        Self {
            event_store: Some(event_store),
            ..self
        }
    }

    /// 获取或创建事件通道
    async fn get_channel(&self, event_type: &str) -> broadcast::Sender<Vec<u8>> {
        let mut channels = self.channels.write().await;

        if !channels.contains_key(event_type) {
            let (tx, _) = broadcast::channel(1000);
            channels.insert(event_type.to_string(), tx.clone());
            info!("Created event channel: {}", event_type);
        }

        channels.get(event_type).expect("event channel should be registered").clone()
    }

    /// 发布事件
    pub async fn publish<T>(&self, event: T) -> Result<(), EventError>
    where
        T: Event + Serialize,
    {
        let event_type = T::event_type();

        debug!("Publishing event: {}", event_type);

        let payload = serde_json::to_vec(&event)
            .map_err(|e| EventError::Serialization(format!("Failed to serialize event: {}", e)))?;

        // 如果启用了持久化，保存事件到事件存储
        if self.enable_persistence {
            if let Some(_event_store) = &self.event_store {
                // 注意：这里需要确保 event 是 DomainEvent 类型
                // 暂时注释掉，因为当前的 Event trait 和 DomainEvent 可能不兼容
                // match event_store.save_event(&event).await {
                //     Ok(_) => {
                //         debug!("Event persisted successfully: {}", event_type);
                //     }
                //     Err(e) => {
                //         warn!("Failed to persist event {}: {}", event_type, e);
                //         // 不返回错误，因为事件持久化失败不应阻止事件发布
                //     }
                // }
            }
        }

        let channel = self.get_channel(event_type).await;

        // 实现重试机制
        let mut retries = 0;
        while retries <= self.max_retries {
            match channel.send(payload.clone()) {
                Ok(_) => {
                    debug!("Event published successfully: {}", event_type);
                    return Ok(());
                }
                Err(e) => {
                    if retries >= self.max_retries {
                        error!("Failed to publish event {} after {} retries: {}", event_type, self.max_retries, e);
                        return Err(EventError::Publish(format!("No receivers: {}", e)));
                    }
                    
                    warn!("Retrying event publish ({} of {}): {}", retries + 1, self.max_retries, event_type);
                    sleep(self.retry_interval).await;
                    retries += 1;
                }
            }
        }

        Err(EventError::Publish("Max retries exceeded".to_string()))
    }

    /// 订阅事件
    pub async fn subscribe<T>(&self) -> Result<EventSubscriber<T>, EventError>
    where
        T: Event + for<'de> Deserialize<'de>,
    {
        let event_type = T::event_type();
        let channel = self.get_channel(event_type).await;

        info!("Subscribing to event: {}", event_type);

        Ok(EventSubscriber {
            rx: channel.subscribe(),
            _phantom: std::marker::PhantomData,
        })
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

/// 事件订阅器
pub struct EventSubscriber<T> {
    rx: broadcast::Receiver<Vec<u8>>,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> EventSubscriber<T>
where
    T: Event + for<'de> Deserialize<'de>,
{
    /// 接收下一个事件(阻塞)
    pub async fn recv(&mut self) -> Result<T, EventError> {
        loop {
            match self.rx.recv().await {
                Ok(payload) => match serde_json::from_slice(&payload) {
                    Ok(event) => return Ok(event),
                    Err(e) => {
                        error!("Failed to deserialize event: {}", e);
                        return Err(EventError::Deserialization(format!("{}", e)));
                    }
                },
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    warn!("Subscriber lagged, {} events skipped", n);
                }
                Err(broadcast::error::RecvError::Closed) => {
                    return Err(EventError::ChannelClosed);
                }
            }
        }
    }
}

/// 事件错误类型
#[derive(Debug, thiserror::Error)]
pub enum EventError {
    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Deserialization error: {0}")]
    Deserialization(String),

    #[error("Publish error: {0}")]
    Publish(String),

    #[error("Channel closed")]
    ChannelClosed,

    #[error("Unknown error: {0}")]
    Other(String),
}

/// 事件trait
pub trait Event: Serialize {
    /// 事件类型标识符
    fn event_type() -> &'static str;

    /// 事件发生时间
    fn timestamp(&self) -> i64;

    /// 事件来源
    fn source(&self) -> &str;
}

/// 事件处理器trait
#[async_trait::async_trait]
pub trait EventHandler<T>: Send + Sync
where
    T: Event + Send + Sync,
{
    /// 处理事件
    async fn handle(&self, event: T) -> Result<(), EventError>;

    /// 获取处理器名称
    fn name(&self) -> &str;
}

/// 事件处理器注册表
pub struct EventHandlerRegistry {
    handlers: EventHandlerMap,
}

/// 事件处理器映射类型别名
type EventHandlerMap = Arc<RwLock<HashMap<String, Vec<Arc<dyn EventHandlerAny>>>>>;

/// 类型擦除的事件处理器
#[async_trait::async_trait]
pub trait EventHandlerAny: Send + Sync {
    async fn handle(&self, event_json: Vec<u8>) -> Result<(), EventError>;
    fn name(&self) -> &str;
}

/// 事件处理器注册表实现
impl EventHandlerRegistry {
    pub fn new() -> Self {
        Self {
            handlers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 注册事件处理器
    pub async fn register<T, H>(&self, handler: H)
    where
        T: Event + for<'de> Deserialize<'de> + Send + Sync + 'static,
        H: EventHandler<T> + 'static,
    {
        let event_type = T::event_type();

        // 检查是否需要创建类型
        if !self.handlers.read().await.contains_key(event_type) {
            let mut handlers = self.handlers.write().await;
            handlers.insert(event_type.to_string(), Vec::new());
        }

        let any_handler = Arc::new(AnyHandlerWrapper::new(handler));
        self.handlers
            .write()
            .await
            .get_mut(event_type)
            .expect("event handler map should contain key after insert")
            .push(any_handler.clone());

        info!(
            "Registered handler '{}' for event '{}'",
            any_handler.name(),
            event_type
        );
    }

    /// 获取事件的所有处理器
    pub async fn get_handlers(&self, event_type: &str) -> Vec<Arc<dyn EventHandlerAny>> {
        self.handlers
            .read()
            .await
            .get(event_type)
            .cloned()
            .unwrap_or_default()
    }
}

/// 事件处理器包装器
struct AnyHandlerWrapper<T, H> {
    handler: H,
    _phantom: std::marker::PhantomData<T>,
}

impl<T, H> AnyHandlerWrapper<T, H> {
    fn new(handler: H) -> Self {
        Self {
            handler,
            _phantom: std::marker::PhantomData,
        }
    }
}

#[async_trait::async_trait]
impl<T, H> EventHandlerAny for AnyHandlerWrapper<T, H>
where
    T: Event + for<'de> Deserialize<'de> + Send + Sync,
    H: EventHandler<T>,
{
    async fn handle(&self, event_json: Vec<u8>) -> Result<(), EventError> {
        let event: T = serde_json::from_slice(&event_json)
            .map_err(|e| EventError::Deserialization(format!("{}", e)))?;
        self.handler.handle(event).await
    }

    fn name(&self) -> &str {
        self.handler.name()
    }
}

/// 全局事件总线
static GLOBAL_EVENT_BUS: tokio::sync::OnceCell<EventBus> = tokio::sync::OnceCell::const_new();

/// 初始化全局事件总线
pub async fn init_global_event_bus() {
    GLOBAL_EVENT_BUS
        .get_or_init(|| async { EventBus::new() })
        .await;
}

/// 获取全局事件总线
pub fn global_event_bus() -> &'static EventBus {
    GLOBAL_EVENT_BUS.get().expect("Event bus not initialized")
}

/// 便捷函数:发布事件到全局总线
pub async fn publish_event<T>(event: T) -> Result<(), EventError>
where
    T: Event + Serialize,
{
    global_event_bus().publish(event).await
}

/// 便捷函数:订阅全局事件
pub async fn subscribe_event<T>() -> Result<EventSubscriber<T>, EventError>
where
    T: Event + for<'de> Deserialize<'de>,
{
    global_event_bus().subscribe().await
}

impl Default for EventHandlerRegistry {
    fn default() -> Self {
        Self::new()
    }
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
    /// 事件时间戳
    pub timestamp: i64,
    /// 事件版本
    pub version: String,
    /// 额外属性
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub attributes: HashMap<String, String>,
}

impl EventMetadata {
    pub fn new(event_type: &str, source: &str) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            event_type: event_type.to_string(),
            source: source.to_string(),
            timestamp: chrono::Utc::now().timestamp(),
            version: "1.0".to_string(),
            attributes: HashMap::new(),
        }
    }

    pub fn with_attribute(mut self, key: String, value: String) -> Self {
        self.attributes.insert(key, value);
        self
    }
}
