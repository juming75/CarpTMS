// WebSocket 批量推送优化
// 支持批量消息推送，减少网络往返次数

use actix::prelude::*;
use log::{debug, error, info, warn};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, RwLock};
use tokio::task::JoinHandle;

use crate::infrastructure::message_router::UnifiedMessage;

/// 消息批处理器配置
#[derive(Debug, Clone)]
pub struct MessageBatcherConfig {
    /// 每批次最大消息数量
    pub batch_size: usize,
    /// 批处理超时时间
    pub batch_timeout: Duration,
    /// 单个设备的最大待发送消息数
    pub max_pending_per_device: usize,
}

impl Default for MessageBatcherConfig {
    fn default() -> Self {
        Self {
            batch_size: 50,
            batch_timeout: Duration::from_millis(100),
            max_pending_per_device: 1000,
        }
    }
}

/// 批处理状态
#[derive(Debug, Clone)]
struct BatchState {
    messages: Vec<UnifiedMessage>,
    last_update: Instant,
}

/// 消息批处理器
pub struct MessageBatcher {
    config: MessageBatcherConfig,
    pending_messages: Arc<RwLock<HashMap<String, BatchState>>>,
    subscribers: Arc<RwLock<HashMap<String, Vec<Addr<crate::gateway::websocket_server::WebSocketSession>>>>>,
    flush_task: Option<JoinHandle<()>>,
}

impl MessageBatcher {
    /// 创建新的批处理器
    pub fn new(config: MessageBatcherConfig) -> Self {
        info!("Creating message batcher: batch_size={}, timeout={:?}",
              config.batch_size, config.batch_timeout);
        
        let batcher = Self {
            config: config.clone(),
            pending_messages: Arc::new(RwLock::new(HashMap::new())),
            subscribers: Arc::new(RwLock::new(HashMap::new())),
            flush_task: None,
        };

        batcher
    }

    /// 启动批处理器
    pub fn start_flush_task(&mut self, ctx: &mut Context<Self>) {
        let pending_messages = self.pending_messages.clone();
        let subscribers = self.subscribers.clone();
        let batch_timeout = self.config.batch_timeout;
        let batch_size = self.config.batch_size;

        self.flush_task = Some(tokio::spawn(async move {
            let mut interval = tokio::time::interval(batch_timeout);
            
            loop {
                interval.tick().await;
                
                // 检查超时的批次
                let mut batches_to_flush = Vec::new();
                {
                    let pending = pending_messages.read().await;
                    let now = Instant::now();
                    
                    for (device_id, state) in pending.iter() {
                        if now.duration_since(state.last_update) >= batch_timeout
                            || state.messages.len() >= batch_size
                        {
                            batches_to_flush.push(device_id.clone());
                        }
                    }
                }

                // 刷新超时的批次
                for device_id in batches_to_flush {
                    Self::flush_device(&device_id, &pending_messages, &subscribers).await;
                }
            }
        }));

        info!("Message batcher flush task started");
    }

    /// 添加消息到批处理队列
    pub async fn add_message(&self, device_id: String, msg: UnifiedMessage) {
        let mut pending = self.pending_messages.write().await;
        
        let state = pending.entry(device_id.clone()).or_insert_with(|| BatchState {
            messages: Vec::with_capacity(self.config.batch_size),
            last_update: Instant::now(),
        });

        state.messages.push(msg);
        state.last_update = Instant::now();

        // 检查是否达到批处理大小
        if state.messages.len() >= self.config.batch_size {
            drop(pending); // 释放写锁
            Self::flush_device(&device_id, &self.pending_messages, &self.subscribers).await;
        } else if state.messages.len() > self.config.max_pending_per_device {
            warn!("Device {} pending messages exceed limit, flushing early", device_id);
            drop(pending);
            Self::flush_device(&device_id, &self.pending_messages, &self.subscribers).await;
        }
    }

    /// 刷新指定设备的消息
    async fn flush_device(
        device_id: &str,
        pending: &Arc<RwLock<HashMap<String, BatchState>>>,
        subscribers: &Arc<RwLock<HashMap<String, Vec<Addr<crate::gateway::websocket_server::WebSocketSession>>>>>,
    ) {
        // 提取待发送消息
        let messages = {
            let mut pending_write = pending.write().await;
            pending_write.remove(device_id)
                .map(|state| state.messages)
        };

        if let Some(messages) = messages {
            if messages.is_empty() {
                return;
            }

            debug!("Flushing batch for device {}: {} messages", device_id, messages.len());

            // 创建批量消息
            let batch_msg = json!({
                "type": "batch",
                "device_id": device_id,
                "count": messages.len(),
                "messages": messages,
                "timestamp": chrono::Utc::now().to_rfc3339(),
            });

            // 发送给所有订阅者
            let subs = subscribers.read().await;
            if let Some(device_subs) = subs.get(device_id) {
                for session in device_subs {
                    // 发送批量消息
                    if let Err(e) = session.try_send(
                        crate::gateway::websocket_server::WsMessage::Text(batch_msg.to_string())
                    ) {
                        error!("Failed to send batch message to WebSocket session: {:?}", e);
                    }
                }
                debug!("Batch message sent to {} subscribers for device {}", 
                       device_subs.len(), device_id);
            }
        }
    }

    /// 注册订阅者
    pub async fn subscribe(&self, device_id: String, session: Addr<crate::gateway::websocket_server::WebSocketSession>) {
        let mut subs = self.subscribers.write().await;
        subs.entry(device_id).or_insert_with(Vec::new).push(session);
    }

    /// 取消订阅
    pub async fn unsubscribe(&self, device_id: String, session_addr: Addr<crate::gateway::websocket_server::WebSocketSession>) {
        let mut subs = self.subscribers.write().await;
        if let Some(device_subs) = subs.get_mut(&device_id) {
            device_subs.retain(|s| !s.same_actor(&session_addr));
            if device_subs.is_empty() {
                subs.remove(&device_id);
            }
        }
    }

    /// 强制刷新所有待发送消息
    pub async fn flush_all(&self) {
        let device_ids: Vec<String> = {
            let pending = self.pending_messages.read().await;
            pending.keys().cloned().collect()
        };

        for device_id in device_ids {
            Self::flush_device(&device_id, &self.pending_messages, &self.subscribers).await;
        }
    }
}

impl Actor for MessageBatcher {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        info!("MessageBatcher actor started");
        self.start_flush_task(ctx);
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        info!("MessageBatcher actor stopped");
        if let Some(task) = self.flush_task.take() {
            task.abort();
        }
    }
}

// 消息类型：添加消息
pub struct AddMessage {
    pub device_id: String,
    pub message: UnifiedMessage,
}

impl Message for AddMessage {
    type Result = ();
}

impl Handler<AddMessage> for MessageBatcher {
    type Result = ResponseActFuture<Self, ()>;

    fn handle(&mut self, msg: AddMessage, _ctx: &mut Self::Context) -> Self::Result {
        let batcher = self.clone();
        
        Box::pin(async move {
            batcher.add_message(msg.device_id, msg.message).await;
        }.into_actor(self))
    }
}

// 消息类型：强制刷新
pub struct FlushAll;

impl Message for FlushAll {
    type Result = ();
}

impl Handler<FlushAll> for MessageBatcher {
    type Result = ResponseActFuture<Self, ()>;

    fn handle(&mut self, _msg: FlushAll, _ctx: &mut Self::Context) -> Self::Result {
        let batcher = self.clone();
        
        Box::pin(async move {
            batcher.flush_all().await;
        }.into_actor(self))
    }
}

impl Clone for MessageBatcher {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            pending_messages: self.pending_messages.clone(),
            subscribers: self.subscribers.clone(),
            flush_task: None, // 不克隆任务句柄
        }
    }
}
