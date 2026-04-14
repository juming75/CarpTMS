//! / WebSocket 推送优化模块
// 实现消息批处理、压缩、连接池等优化
#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, RwLock, Semaphore};
// 移除未使用的序列化导入 - Vec<u8>不需要序列化
use anyhow::Result;

/// WebSocket 消息
/// 使用 Arc<Vec<u8>> 存储 payload，避免广播时的多次 clone
#[derive(Debug, Clone)]
pub struct WsOptimizedMessage {
    pub msg_id: String,
    pub msg_type: String,
    pub payload: Arc<Vec<u8>>, // 使用 Arc 避免广播时多次 clone 大数据
    pub timestamp: u64,
}

impl WsOptimizedMessage {
    /// 创建新的消息，自动将 payload 包装为 Arc
    pub fn new(msg_id: String, msg_type: String, payload: Vec<u8>, timestamp: u64) -> Self {
        Self {
            msg_id,
            msg_type,
            payload: Arc::new(payload),
            timestamp,
        }
    }
    
    /// 获取 payload 的引用
    pub fn payload_bytes(&self) -> &[u8] {
        &self.payload
    }
}

/// 消息批处理配置
#[derive(Debug, Clone)]
pub struct BatchConfig {
    pub max_batch_size: usize,    // 最大批处理大小
    pub max_batch_time: Duration, // 最大批处理时间
    pub enable_compression: bool, // 是否启用压缩
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            max_batch_size: 100,
            max_batch_time: Duration::from_millis(100),
            enable_compression: true,
        }
    }
}

/// 消息批处理器
pub struct MessageBatcher {
    config: BatchConfig,
    messages: Vec<WsOptimizedMessage>,
    start_time: Instant,
    sender: mpsc::Sender<Vec<WsOptimizedMessage>>,
}

impl MessageBatcher {
    pub fn new(config: BatchConfig, sender: mpsc::Sender<Vec<WsOptimizedMessage>>) -> Self {
        let max_batch_size = config.max_batch_size;
        Self {
            config: config.clone(),
            messages: Vec::with_capacity(max_batch_size),
            start_time: Instant::now(),
            sender,
        }
    }

    /// 添加消息到批处理器
    pub async fn add_message(&mut self, message: WsOptimizedMessage) -> Result<()> {
        self.messages.push(message);

        // 检查是否需要刷新
        if self.should_flush() {
            self.flush().await?;
        }

        Ok(())
    }

    /// 判断是否应该刷新
    fn should_flush(&self) -> bool {
        self.messages.len() >= self.config.max_batch_size
            || self.start_time.elapsed() >= self.config.max_batch_time
    }

    /// 刷新批处理的消息
    async fn flush(&mut self) -> Result<()> {
        if self.messages.is_empty() {
            return Ok(());
        }

        // 压缩消息(如果启用)
        if self.config.enable_compression {
            self.compress_messages()?;
        }

        // 发送批处理消息
        let batch = std::mem::take(&mut self.messages);
        self.start_time = Instant::now();
        self.sender.send(batch).await?;

        Ok(())
    }

    /// 压缩消息
    fn compress_messages(&mut self) -> Result<()> {
        // TODO: 添加 flate2 依赖后启用压缩
        // use flate2::write::GzEncoder;
        // use flate2::Compression;
        //
        // for msg in &mut self.messages {
        //     let mut encoder = GzEncoder::new(Vec::new(), Compression::fast());
        //     encoder.write_all(&msg.payload)?;
        //     msg.payload = encoder.finish()?;
        // }

        Ok(())
    }
}

/// WebSocket 连接池
pub struct WsConnectionPool {
    connections: Arc<RwLock<HashMap<String, mpsc::Sender<WsOptimizedMessage>>>>,
    max_connections: usize,
    semaphore: Arc<Semaphore>,
}

impl WsConnectionPool {
    pub fn new(max_connections: usize) -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            max_connections,
            semaphore: Arc::new(Semaphore::new(max_connections)),
        }
    }

    /// 添加连接
    pub async fn add_connection(
        &self,
        client_id: String,
        sender: mpsc::Sender<WsOptimizedMessage>,
    ) -> Result<()> {
        // 获取信号量
        let _permit = self.semaphore.acquire().await?;

        let mut connections = self.connections.write().await;
        connections.insert(client_id, sender);

        Ok(())
    }

    /// 移除连接
    pub async fn remove_connection(&self, client_id: &str) {
        let mut connections = self.connections.write().await;
        connections.remove(client_id);
    }

    /// 广播消息到所有连接
    /// 优化：WsOptimizedMessage 的 payload 已经是 Arc<Vec<u8>>，
    /// 所以 clone 消息时只需 clone Arc（原子引用计数，极低成本）
    pub async fn broadcast(&self, message: WsOptimizedMessage) -> usize {
        let connections = self.connections.read().await;

        let mut sent_count = 0;
        for sender in connections.values() {
            // 现在 WsOptimizedMessage.clone() 只会 clone Arc（O(1)）而不是 Vec（O(n)）
            if sender.send(message.clone()).await.is_ok() {
                sent_count += 1;
            }
        }

        sent_count
    }

    /// 获取连接数
    pub async fn connection_count(&self) -> usize {
        self.connections.read().await.len()
    }
}

/// 心跳管理器
pub struct HeartbeatManager {
    client_last_seen: Arc<RwLock<HashMap<String, Instant>>>,
    heartbeat_interval: Duration,
    timeout_threshold: Duration,
}

impl HeartbeatManager {
    pub fn new(heartbeat_interval: Duration, timeout_threshold: Duration) -> Self {
        Self {
            client_last_seen: Arc::new(RwLock::new(HashMap::new())),
            heartbeat_interval,
            timeout_threshold,
        }
    }

    /// 更新客户端最后活跃时间
    pub async fn update_last_seen(&self, client_id: &str) {
        let mut last_seen = self.client_last_seen.write().await;
        last_seen.insert(client_id.to_string(), Instant::now());
    }

    /// 检查超时的客户端
    pub async fn check_timeouts(&self) -> Vec<String> {
        let now = Instant::now();
        let last_seen = self.client_last_seen.read().await;

        last_seen
            .iter()
            .filter(|(_, &time)| now.duration_since(time) > self.timeout_threshold)
            .map(|(client_id, _)| client_id.clone())
            .collect()
    }

    /// 移除客户端
    pub async fn remove_client(&self, client_id: &str) {
        let mut last_seen = self.client_last_seen.write().await;
        last_seen.remove(client_id);
    }

    /// 启动心跳检查
    pub async fn start_heartbeat_check<F>(&self, timeout_callback: F)
    where
        F: Fn(Vec<String>) + Send + 'static,
    {
        let interval = self.heartbeat_interval;
        let timeout_threshold = self.timeout_threshold;
        let last_seen = self.client_last_seen.clone();

        tokio::spawn(async move {
            let mut timer = tokio::time::interval(interval);

            loop {
                timer.tick().await;

                let now = Instant::now();
                let mut timed_out_clients = Vec::new();

                {
                    let last_seen_read = last_seen.read().await;
                    for (client_id, &time) in last_seen_read.iter() {
                        if now.duration_since(time) > timeout_threshold {
                            timed_out_clients.push(client_id.clone());
                        }
                    }
                }

                if !timed_out_clients.is_empty() {
                    timeout_callback(timed_out_clients);
                }
            }
        });
    }
}

/// WebSocket 推送优化器
pub struct WsPushOptimizer {
    batch_config: BatchConfig,
    connection_pool: WsConnectionPool,
    heartbeat_manager: HeartbeatManager,
}

impl WsPushOptimizer {
    pub fn new(
        max_connections: usize,
        heartbeat_interval: Duration,
        timeout_threshold: Duration,
    ) -> Self {
        Self {
            batch_config: BatchConfig::default(),
            connection_pool: WsConnectionPool::new(max_connections),
            heartbeat_manager: HeartbeatManager::new(heartbeat_interval, timeout_threshold),
        }
    }

    /// 创建批处理器
    pub fn create_batcher(&self) -> (MessageBatcher, mpsc::Receiver<Vec<WsOptimizedMessage>>) {
        let (sender, receiver) = mpsc::channel(1000);
        let batcher = MessageBatcher::new(self.batch_config.clone(), sender);

        (batcher, receiver)
    }

    /// 添加连接
    pub async fn add_connection(
        &self,
        client_id: String,
        sender: mpsc::Sender<WsOptimizedMessage>,
    ) -> Result<()> {
        self.connection_pool
            .add_connection(client_id.clone(), sender)
            .await?;
        self.heartbeat_manager.update_last_seen(&client_id).await;
        Ok(())
    }

    /// 移除连接
    pub async fn remove_connection(&self, client_id: &str) {
        self.connection_pool.remove_connection(client_id).await;
        self.heartbeat_manager.remove_client(client_id).await;
    }

    /// 广播消息
    pub async fn broadcast(&self, message: WsOptimizedMessage) -> usize {
        self.connection_pool.broadcast(message).await
    }

    /// 更新心跳
    pub async fn update_heartbeat(&self, client_id: &str) {
        self.heartbeat_manager.update_last_seen(client_id).await;
    }

    /// 获取连接数
    pub async fn connection_count(&self) -> usize {
        self.connection_pool.connection_count().await
    }

    /// 启动心跳检查
    pub async fn start_heartbeat_check<F>(&self, callback: F)
    where
        F: Fn(Vec<String>) + Send + 'static,
    {
        self.heartbeat_manager.start_heartbeat_check(callback).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_batcher_flush() {
        let (tx, mut rx) = mpsc::channel(100);
        let mut batcher = MessageBatcher::new(BatchConfig::default(), tx);

        // 使用新的构造函数创建消息
        let msg = WsOptimizedMessage::new(
            "1".to_string(),
            "test".to_string(),
            Vec::new(), // payload 会被自动包装为 Arc
            0,
        );

        batcher.add_message(msg).await.unwrap();

        // 强制刷新
        batcher.flush().await.unwrap();

        let received = rx.recv().await.unwrap();
        assert_eq!(received.len(), 1);
    }

    #[tokio::test]
    async fn test_connection_pool() {
        let pool = WsConnectionPool::new(10);
        let (tx, _) = mpsc::channel(100);

        pool.add_connection("client1".to_string(), tx)
            .await
            .unwrap();

        assert_eq!(pool.connection_count().await, 1);

        pool.remove_connection("client1").await;

        assert_eq!(pool.connection_count().await, 0);
    }
}






