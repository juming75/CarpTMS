//! / WebSocket 消息队列优化
// 使用消息队列解耦消息生产者和消费者

use actix::prelude::*;
use log::{debug, error, info};
use serde_json::json;
use std::collections::{BinaryHeap, HashMap};
use std::sync::Arc;
use tokio::sync::mpsc;
use std::cmp::Ordering;

use crate::infrastructure::message_router::UnifiedMessage;

/// 消息优先级
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MessagePriority {
    /// 低优先级:常规位置数据
    Low = 0,
    /// 中优先级:设备状态变更
    Medium = 1,
    /// 高优先级:报警消息
    High = 2,
    /// 紧急优先级:紧急报警
    Critical = 3,
}

impl MessagePriority {
    pub fn from_msg_type(msg_type: &str) -> Self {
        match msg_type {
            "alarm" | "emergency" => MessagePriority::Critical,
            "alert" | "warning" => MessagePriority::High,
            "status" | "heartbeat" => MessagePriority::Medium,
            _ => MessagePriority::Low,
        }
    }
}

/// 优先级消息包装器
#[derive(Debug, Clone)]
pub struct PrioritizedMessage {
    pub priority: MessagePriority,
    pub sequence: u64, // 用于同优先级的排序
    pub message: UnifiedMessage,
    pub device_id: String,
}

impl PartialEq for PrioritizedMessage {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority && self.sequence == other.sequence
    }
}

impl Eq for PrioritizedMessage {}

impl PartialOrd for PrioritizedMessage {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PrioritizedMessage {
    fn cmp(&self, other: &Self) -> Ordering {
        // 优先级高的排在前面(BinaryHeap 是最大堆)
        match self.priority.cmp(&other.priority) {
            Ordering::Equal => {
                // 同优先级按序列号排序(先发的先处理)
                other.sequence.cmp(&self.sequence)
            }
            other => other,
        }
    }
}

/// 消息队列配置
#[derive(Debug, Clone)]
pub struct MessageQueueConfig {
    /// 队列容量
    pub queue_size: usize,
    /// 工作线程数
    pub workers: usize,
    /// 每个工作线程的批处理大小
    pub batch_size: usize,
}

impl Default for MessageQueueConfig {
    fn default() -> Self {
        Self {
            queue_size: 10000,
            workers: 4,
            batch_size: 100,
        }
    }
}

/// WebSocket 消息队列
pub struct WebSocketMessageQueue {
    sender: mpsc::Sender<PrioritizedMessage>,
    workers: Vec<JoinHandle<()>>,
    config: MessageQueueConfig,
    next_sequence: Arc<std::sync::atomic::AtomicU64>,
}

impl WebSocketMessageQueue {
    /// 创建新的消息队列
    pub fn new(config: MessageQueueConfig) -> Self {
        info!("Creating WebSocket message queue: size={}, workers={}",
              config.queue_size, config.workers);

        let (sender, receiver) = mpsc::channel(config.queue_size);
        let mut workers = Vec::new();
        let next_sequence = Arc::new(std::sync::atomic::AtomicU64::new(0));

        // 启动工作线程
        for worker_id in 0..config.workers {
            let receiver_clone = receiver.clone();
            let next_sequence_clone = Arc::clone(&next_sequence);
            let batch_size = config.batch_size;

            let worker = tokio::spawn(async move {
                info!("WebSocket message queue worker {} started", worker_id);
                
                // 使用优先级队列
                let mut priority_queue: BinaryHeap<PrioritizedMessage> = BinaryHeap::new();
                let mut batch = Vec::with_capacity(batch_size);

                loop {
                    // 从通道接收消息
                    match receiver_clone.recv().await {
                        Ok(prioritized_msg) => {
                            // 添加到优先级队列
                            priority_queue.push(prioritized_msg);
                        }
                        Err(_) => {
                            // 通道关闭,处理剩余消息后退出
                            info!("Message queue worker {} shutting down", worker_id);
                            break;
                        }
                    }

                    // 处理优先级队列中的消息
                    while let Some(prioritized_msg) = priority_queue.pop() {
                        batch.push(prioritized_msg);

                        // 达到批处理大小,发送
                        if batch.len() >= batch_size {
                            Self::process_batch(&batch).await;
                            batch.clear();
                        }
                    }
                }

                // 处理剩余消息
                if !batch.is_empty() {
                    Self::process_batch(&batch).await;
                }

                info!("Message queue worker {} stopped", worker_id);
            });

            workers.push(worker);
        }

        info!("WebSocket message queue created with {} workers", workers.len());

        Self {
            sender,
            workers,
            config,
            next_sequence,
        }
    }

    /// 发送消息到队列
    pub async fn send(&self, device_id: String, message: UnifiedMessage, priority: MessagePriority) -> Result<(), mpsc::error::SendError<PrioritizedMessage>> {
        let sequence = self.next_sequence.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        let prioritized_msg = PrioritizedMessage {
            priority,
            sequence,
            message,
            device_id,
        };

        self.sender.send(prioritized_msg).await
    }

    /// 处理一批消息
    async fn process_batch(batch: &[PrioritizedMessage]) {
        if batch.is_empty() {
            return;
        }

        debug!("Processing message batch: {} messages", batch.len());

        // 统计优先级分布
        let mut counts = [0usize; 4];
        for msg in batch {
            counts[msg.priority as usize] += 1;
        }

        debug!("Message priority distribution: Critical={}, High={}, Medium={}, Low={}",
               counts[3], counts[2], counts[1], counts[0]);

        // TODO: 实际的消息发送逻辑
        // 这里应该调用 WebSocketSessionRegistry 来发送消息
        // 暂时只记录日志
        for msg in batch {
            debug!("Processing message for device {}: priority={:?}, msg_type={:?}",
                   msg.device_id, msg.priority, msg.message.message_type);
        }
    }

    /// 获取队列统计信息
    pub fn get_stats(&self) -> MessageQueueStats {
        MessageQueueStats {
            queue_size: self.config.queue_size,
            workers: self.config.workers,
            batch_size: self.config.batch_size,
            current_sequence: self.next_sequence.load(std::sync::atomic::Ordering::SeqCst),
        }
    }
}

impl Drop for WebSocketMessageQueue {
    fn drop(&mut self) {
        info!("Dropping WebSocket message queue");
        for worker in &self.workers {
            worker.abort();
        }
    }
}

/// 消息队列统计信息
#[derive(Debug, Clone, serde::Serialize)]
pub struct MessageQueueStats {
    pub queue_size: usize,
    pub workers: usize,
    pub batch_size: usize,
    pub current_sequence: u64,
}

// 消息类型:发送消息
pub struct QueueMessage {
    pub device_id: String,
    pub message: UnifiedMessage,
    pub priority: MessagePriority,
}

impl Message for QueueMessage {
    type Result = Result<(), String>;
}

/// Actor 包装器,用于在 Actor 系统中使用消息队列
pub struct MessageQueueActor {
    queue: Option<WebSocketMessageQueue>,
}

impl MessageQueueActor {
    pub fn new(config: MessageQueueConfig) -> Self {
        Self {
            queue: Some(WebSocketMessageQueue::new(config)),
        }
    }
}

impl Actor for MessageQueueActor {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        info!("MessageQueueActor started");
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        info!("MessageQueueActor stopped");
    }
}

impl Handler<QueueMessage> for MessageQueueActor {
    type Result = ResponseActFuture<Self, Result<(), String>>;

    fn handle(&mut self, msg: QueueMessage, _ctx: &mut Self::Context) -> Self::Result {
        let queue = if let Some(q) = &self.queue {
            q.clone()
        } else {
            return Box::pin(async move {
                Err("Message queue not initialized".to_string())
            }.into_actor(self));
        };

        Box::pin(async move {
            queue.send(msg.device_id, msg.message, msg.priority).await
                .map_err(|e| format!("Send error: {}", e))
        }.into_actor(self))
    }
}

impl Clone for WebSocketMessageQueue {
    fn clone(&self) -> Self {
        // 不完全克隆,只克隆 sender 和配置
        // 注意:这不会克隆工作线程
        Self {
            sender: self.sender.clone(),
            workers: Vec::new(),
            config: self.config.clone(),
            next_sequence: Arc::clone(&self.next_sequence),
        }
    }
}






