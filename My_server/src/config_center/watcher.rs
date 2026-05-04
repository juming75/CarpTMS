//! /! 配置变更监听模块
//!
//! 实现配置变更的监听和通知功能

use super::models::{ConfigChangeEvent, ConfigChangeEventType, ConfigKey, ConfigValue};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use tokio::sync::broadcast;
use tokio::time::sleep;

/// 配置变更监听函数类型
pub type ConfigChangeCallback = Arc<dyn Fn(ConfigChangeEvent) + Send + Sync + 'static>;

/// 配置监听器
pub struct ConfigWatcher {
    /// 事件广播器
    tx: broadcast::Sender<ConfigChangeEvent>,
    /// 监听器存储
    listeners: Arc<RwLock<Vec<ConfigChangeCallback>>>,
    /// 监听间隔
    check_interval: Duration,
}

impl ConfigWatcher {
    /// 创建新的配置监听器
    pub fn new(check_interval: Duration) -> Self {
        let (tx, _rx) = broadcast::channel(100);
        Self {
            tx,
            listeners: Arc::new(RwLock::new(Vec::new())),
            check_interval,
        }
    }

    /// 发送配置变更事件
    pub fn send_event(&self, event: ConfigChangeEvent) {
        let _ = self.tx.send(event);
    }

    /// 订阅配置变更事件
    pub fn subscribe(&self) -> broadcast::Receiver<ConfigChangeEvent> {
        self.tx.subscribe()
    }

    /// 添加配置变更监听器
    pub fn add_listener(&self, listener: ConfigChangeCallback) {
        if let Ok(mut listeners) = self.listeners.write() {
            listeners.push(listener);
        }
    }

    /// 移除配置变更监听器
    pub fn remove_listener(&self, listener: &ConfigChangeCallback) {
        if let Ok(mut listeners) = self.listeners.write() {
            listeners.retain(|l| !Arc::ptr_eq(l, listener));
        }
    }

    /// 通知所有监听器
    pub fn notify_listeners(&self, event: ConfigChangeEvent) {
        if let Ok(listeners) = self.listeners.read() {
            for listener in listeners.iter() {
                listener(event.clone());
            }
        }
    }

    /// 广播配置变更事件
    pub async fn broadcast_event(&self, event: ConfigChangeEvent) {
        // 发送广播事件
        let _ = self.tx.send(event.clone());

        // 通知所有监听器
        self.notify_listeners(event);
    }

    /// 启动配置变更监听循环
    pub async fn start_watching(&self) {
        loop {
            sleep(self.check_interval).await;
            // 这里可以添加主动检查配置变更的逻辑
            // 例如,定期检查配置文件或数据库中的配置变化
        }
    }

    /// 创建配置变更事件
    pub fn create_event(
        event_type: ConfigChangeEventType,
        key: ConfigKey,
        old_value: Option<ConfigValue>,
        new_value: Option<ConfigValue>,
        reason: Option<String>,
        changed_by: Option<String>,
    ) -> ConfigChangeEvent {
        ConfigChangeEvent {
            event_type,
            key,
            old_value,
            new_value,
            timestamp: Instant::now(),
            reason,
            changed_by,
        }
    }

    /// 获取监听间隔
    pub fn check_interval(&self) -> Duration {
        self.check_interval
    }

    /// 设置监听间隔
    pub fn set_check_interval(&mut self, interval: Duration) {
        self.check_interval = interval;
    }

    /// 获取监听器数量
    pub fn listener_count(&self) -> usize {
        self.listeners.read().ok().map(|l| l.len()).unwrap_or(0)
    }

    /// 清除所有监听器
    pub fn clear_listeners(&self) {
        if let Ok(mut listeners) = self.listeners.write() {
            listeners.clear();
        }
    }
}

/// 配置变更监听管理器
pub struct ConfigWatcherManager {
    /// 配置监听器
    watcher: Arc<ConfigWatcher>,
    /// 监听任务句柄
    watch_task: Option<tokio::task::JoinHandle<()>>,
}

impl ConfigWatcherManager {
    /// 创建新的配置变更监听管理器
    pub fn new(check_interval: Duration) -> Self {
        let watcher = Arc::new(ConfigWatcher::new(check_interval));
        Self {
            watcher,
            watch_task: None,
        }
    }

    /// 启动监听
    pub fn start(&mut self) {
        if self.watch_task.is_none() {
            let watcher = self.watcher.clone();
            let task = tokio::spawn(async move {
                watcher.start_watching().await;
            });
            self.watch_task = Some(task);
        }
    }

    /// 停止监听
    pub async fn stop(&mut self) {
        if let Some(task) = self.watch_task.take() {
            task.abort();
            let _ = task.await;
        }
    }

    /// 获取配置监听器
    pub fn watcher(&self) -> Arc<ConfigWatcher> {
        self.watcher.clone()
    }

    /// 发送配置变更事件
    pub fn send_event(&self, event: ConfigChangeEvent) {
        self.watcher.send_event(event);
    }

    /// 广播配置变更事件
    pub async fn broadcast_event(&self, event: ConfigChangeEvent) {
        self.watcher.broadcast_event(event).await;
    }
}
