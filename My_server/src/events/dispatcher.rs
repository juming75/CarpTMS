//! /! 事件调度器
//!
//! 负责事件的调度和处理器调用

use std::sync::Arc;
use tokio::sync::RwLock;
use log::{info, error, debug};
use super::{EventBus, EventHandlerRegistry, publish_event, EventError};
use super::handlers::*;

/// 事件调度器
pub struct EventDispatcher {
    event_bus: Arc<EventBus>,
    registry: Arc<EventHandlerRegistry>,
    running: Arc<RwLock<bool>>,
}

impl EventDispatcher {
    pub fn new(event_bus: Arc<EventBus>) -> Self {
        Self {
            event_bus,
            registry: Arc::new(EventHandlerRegistry::new()),
            running: Arc::new(RwLock::new(false)),
        }
    }

    /// 注册所有事件处理器
    pub async fn register_all_handlers(&self) {
        info!("Registering event handlers...");

        // 注册车辆相关处理器
        self.registry.register(VehicleLocationCacheHandler).await;

        // 注册订单相关处理器
        self.registry.register(OrderStatusNotificationHandler).await;

        // 注册设备相关处理器
        self.registry.register(DeviceMonitoringHandler).await;

        // 注册警报处理器
        self.registry.register(AlertHandler).await;

        // 注册称重数据处理器
        self.registry.register(WeighingDataHandler).await;

        // 注册同步完成处理器
        self.registry.register(SyncCompletedHandler).await;

        info!("Event handlers registered successfully");
    }

    /// 启动事件调度器
    pub async fn start(&self) -> Result<(), EventError> {
        let mut running = self.running.write().await;
        if *running {
            return Err(EventError::Other("Event dispatcher already running".to_string()));
        }
        *running = true;
        drop(running);

        info!("Starting event dispatcher...");

        // 注册所有处理器
        self.register_all_handlers().await;

        Ok(())
    }

    /// 停止事件调度器
    pub async fn stop(&self) {
        let mut running = self.running.write().await;
        *running = false;
        info!("Event dispatcher stopped");
    }

    /// 调度事件到对应的处理器
    pub async fn dispatch(&self, event_json: Vec<u8>, event_type: &str) -> Result<(), EventError> {
        debug!("Dispatching event: {}", event_type);

        let handlers = self.registry.get_handlers(event_type).await;

        if handlers.is_empty() {
            debug!("No handlers registered for event: {}", event_type);
            return Ok(());
        }

        // 并发调用所有处理器
        let futures: Vec<_> = handlers
            .iter()
            .map(|handler| {
                let handler = handler.clone();
                let event_json = event_json.clone();
                async move {
                    handler.handle(event_json).await
                }
            })
            .collect();

        let results = futures::future::join_all(futures).await;

        for (handler, result) in handlers.iter().zip(results.iter()) {
            match result {
                Ok(_) => debug!("Handler {} processed event successfully", handler.name()),
                Err(e) => error!("Handler {} failed: {}", handler.name(), e),
            }
        }

        Ok(())
    }
}

/// 启动事件系统
pub async fn start_event_system(event_bus: Arc<EventBus>) -> Result<Arc<EventDispatcher>, EventError> {
    let dispatcher = Arc::new(EventDispatcher::new(event_bus));
    dispatcher.start().await?;
    Ok(dispatcher)
}







