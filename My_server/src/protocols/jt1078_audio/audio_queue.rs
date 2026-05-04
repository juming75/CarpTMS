//! 音频消息队列
//!
//! 用于管理音频数据的生产消费，实现客户端和终端之间的音频流转发
//! Rust的异步安全实现

use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex, RwLock};

/// 音频帧数据类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AudioFrameType {
    /// 来自客户端的音频数据（推送到终端）
    ClientToTerminal,
    /// 来自终端的音频数据（推送给客户端）
    TerminalToClient,
}

/// 音频帧消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioMessage {
    /// 音频数据
    pub data: Vec<u8>,
    /// 数据长度
    pub length: usize,
    /// 设备ID（终端设备标识）
    pub device_id: String,
    /// 客户端ID（发起对讲的客户端）
    pub client_id: String,
    /// 音频帧类型
    pub frame_type: AudioFrameType,
    /// 时间戳（微秒）
    pub timestamp: u64,
    /// 音频通道号
    pub channel: u8,
}

impl AudioMessage {
    /// 创建新的音频消息
    pub fn new(
        data: Vec<u8>,
        device_id: String,
        client_id: String,
        frame_type: AudioFrameType,
        channel: u8,
    ) -> Self {
        Self {
            length: data.len(),
            data,
            device_id,
            client_id,
            frame_type,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_micros() as u64,
            channel,
        }
    }
}

/// 音频队列配置
#[derive(Debug, Clone)]
pub struct AudioQueueConfig {
    /// 队列最大容量
    pub max_queue_size: usize,
    /// 音频帧最大大小（字节）
    pub max_frame_size: usize,
    /// 清理间隔（秒）
    pub cleanup_interval: u64,
}

impl Default for AudioQueueConfig {
    fn default() -> Self {
        Self {
            max_queue_size: 1000,
            max_frame_size: 10240,
            cleanup_interval: 60,
        }
    }
}

/// 设备连接信息
#[derive(Debug, Clone)]
pub struct DeviceConnection {
    /// 设备ID
    pub device_id: String,
    /// 连接时间戳
    pub connected_at: u64,
    /// 最后活跃时间
    pub last_active: u64,
    /// 是否处于对讲状态
    pub is_intercom_active: bool,
    /// 对讲客户端ID
    pub intercom_client_id: Option<String>,
}

/// 音频消息队列管理器
/// 对应C++实现中的message_queue、uv_mutex_lock/unlock、uv_async_send
pub struct AudioQueueManager {
    /// 设备ID到音频消息发送器的映射（用于转发到终端）
    device_senders: Arc<RwLock<HashMap<String, mpsc::UnboundedSender<AudioMessage>>>>,
    /// 客户端ID到音频消息发送器的映射（用于转发给客户端）
    client_senders: Arc<RwLock<HashMap<String, mpsc::UnboundedSender<AudioMessage>>>>,
    /// 设备连接信息
    device_connections: Arc<RwLock<HashMap<String, DeviceConnection>>>,
    /// 配置
    config: AudioQueueConfig,
    /// 队列统计信息
    stats: Arc<Mutex<QueueStats>>,
}

/// 队列统计信息
#[derive(Debug, Clone, Default)]
pub struct QueueStats {
    /// 总接收帧数
    pub total_frames_received: u64,
    /// 总转发帧数
    pub total_frames_forwarded: u64,
    /// 错误帧数
    pub error_frames: u64,
    /// 当前活跃对讲数
    pub active_intercoms: u64,
}

impl AudioQueueManager {
    /// 创建新的音频队列管理器
    pub fn new(config: AudioQueueConfig) -> Self {
        Self {
            device_senders: Arc::new(RwLock::new(HashMap::new())),
            client_senders: Arc::new(RwLock::new(HashMap::new())),
            device_connections: Arc::new(RwLock::new(HashMap::new())),
            config,
            stats: Arc::new(Mutex::new(QueueStats::default())),
        }
    }

    /// 注册设备连接（对应C++中的g_map_equpid_client）
    pub async fn register_device(
        &self,
        device_id: &str,
        sender: mpsc::UnboundedSender<AudioMessage>,
    ) -> Result<(), String> {
        let mut device_senders = self.device_senders.write().await;

        if device_senders.contains_key(device_id) {
            warn!(
                "Device {} already registered, replacing connection",
                device_id
            );
        }

        device_senders.insert(device_id.to_string(), sender);

        // 更新设备连接信息
        let mut connections = self.device_connections.write().await;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        connections.insert(
            device_id.to_string(),
            DeviceConnection {
                device_id: device_id.to_string(),
                connected_at: now,
                last_active: now,
                is_intercom_active: false,
                intercom_client_id: None,
            },
        );

        info!("Device {} registered for audio forwarding", device_id);
        Ok(())
    }

    /// 注销设备连接
    pub async fn unregister_device(&self, device_id: &str) {
        let mut device_senders = self.device_senders.write().await;
        device_senders.remove(device_id);

        let mut connections = self.device_connections.write().await;
        if let Some(conn) = connections.get(device_id) {
            if conn.is_intercom_active {
                let mut stats = self.stats.lock().await;
                stats.active_intercoms = stats.active_intercoms.saturating_sub(1);
            }
        }
        connections.remove(device_id);

        info!("Device {} unregistered from audio forwarding", device_id);
    }

    /// 注册客户端连接（用于接收终端音频）
    pub async fn register_client(
        &self,
        client_id: &str,
        sender: mpsc::UnboundedSender<AudioMessage>,
    ) -> Result<(), String> {
        let mut client_senders = self.client_senders.write().await;
        client_senders.insert(client_id.to_string(), sender);
        Ok(())
    }

    /// 注销客户端连接
    pub async fn unregister_client(&self, client_id: &str) {
        let mut client_senders = self.client_senders.write().await;
        client_senders.remove(client_id);
    }

    /// 写入音频数据到消息队列（对应C++中的writeAudioQueue）
    /// 从客户端接收音频，准备转发到终端
    pub async fn write_audio_to_queue(
        &self,
        buffer: Vec<u8>,
        device_id: String,
        client_id: String,
        channel: u8,
    ) -> Result<(), String> {
        if buffer.is_empty() || buffer.len() > self.config.max_frame_size {
            let mut stats = self.stats.lock().await;
            stats.error_frames += 1;
            return Err(format!("Invalid audio frame size: {}", buffer.len()));
        }

        let message = AudioMessage::new(
            buffer,
            device_id.clone(),
            client_id.clone(),
            AudioFrameType::ClientToTerminal,
            channel,
        );

        // 更新设备活跃时间
        self.update_device_activity(&device_id).await;

        // 查找设备连接并转发音频
        let device_senders = self.device_senders.read().await;
        if let Some(sender) = device_senders.get(&device_id) {
            if sender.send(message.clone()).is_err() {
                let mut stats = self.stats.lock().await;
                stats.error_frames += 1;
                return Err(format!("Failed to send audio to device {}", device_id));
            }

            let mut stats = self.stats.lock().await;
            stats.total_frames_received += 1;
            stats.total_frames_forwarded += 1;

            debug!(
                "Audio frame forwarded to device {}, size: {} bytes",
                device_id, message.length
            );
            Ok(())
        } else {
            let mut stats = self.stats.lock().await;
            stats.error_frames += 1;
            Err(format!("Device {} not connected", device_id))
        }
    }

    /// 写入终端音频到客户端队列
    /// 从终端接收音频，准备转发给客户端
    pub async fn write_terminal_audio_to_queue(
        &self,
        buffer: Vec<u8>,
        device_id: String,
        client_id: String,
        channel: u8,
    ) -> Result<(), String> {
        if buffer.is_empty() || buffer.len() > self.config.max_frame_size {
            let mut stats = self.stats.lock().await;
            stats.error_frames += 1;
            return Err(format!("Invalid audio frame size: {}", buffer.len()));
        }

        let message = AudioMessage::new(
            buffer,
            device_id.clone(),
            client_id.clone(),
            AudioFrameType::TerminalToClient,
            channel,
        );

        // 查找客户端连接并转发音频
        let client_senders = self.client_senders.read().await;
        if let Some(sender) = client_senders.get(&client_id) {
            if sender.send(message.clone()).is_err() {
                let mut stats = self.stats.lock().await;
                stats.error_frames += 1;
                return Err(format!("Failed to send audio to client {}", client_id));
            }

            let mut stats = self.stats.lock().await;
            stats.total_frames_received += 1;
            stats.total_frames_forwarded += 1;

            debug!(
                "Terminal audio forwarded to client {}, size: {} bytes",
                client_id, message.length
            );
            Ok(())
        } else {
            let mut stats = self.stats.lock().await;
            stats.error_frames += 1;
            Err(format!("Client {} not connected", client_id))
        }
    }

    /// 更新设备活跃时间
    async fn update_device_activity(&self, device_id: &str) {
        let mut connections = self.device_connections.write().await;
        if let Some(conn) = connections.get_mut(device_id) {
            conn.last_active = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
        }
    }

    /// 开始对讲会话
    pub async fn start_intercom_session(
        &self,
        device_id: &str,
        client_id: &str,
    ) -> Result<(), String> {
        let mut connections = self.device_connections.write().await;
        if let Some(conn) = connections.get_mut(device_id) {
            if conn.is_intercom_active {
                return Err(format!("Device {} already in intercom session", device_id));
            }

            conn.is_intercom_active = true;
            conn.intercom_client_id = Some(client_id.to_string());

            let mut stats = self.stats.lock().await;
            stats.active_intercoms += 1;

            info!(
                "Intercom session started: device={}, client={}",
                device_id, client_id
            );
            Ok(())
        } else {
            Err(format!("Device {} not connected", device_id))
        }
    }

    /// 停止对讲会话
    pub async fn stop_intercom_session(&self, device_id: &str) -> Result<(), String> {
        let mut connections = self.device_connections.write().await;
        if let Some(conn) = connections.get_mut(device_id) {
            if !conn.is_intercom_active {
                return Err(format!(
                    "Device {} is not in active intercom session",
                    device_id
                ));
            }

            let client_id = conn.intercom_client_id.clone();
            conn.is_intercom_active = false;
            conn.intercom_client_id = None;

            let mut stats = self.stats.lock().await;
            stats.active_intercoms = stats.active_intercoms.saturating_sub(1);

            info!(
                "Intercom session stopped: device={}, client={:?}",
                device_id, client_id
            );
            Ok(())
        } else {
            Err(format!("Device {} not connected", device_id))
        }
    }

    /// 检查设备是否在线
    pub async fn is_device_online(&self, device_id: &str) -> bool {
        let device_senders = self.device_senders.read().await;
        device_senders.contains_key(device_id)
    }

    /// 检查客户端是否在线
    pub async fn is_client_online(&self, client_id: &str) -> bool {
        let client_senders = self.client_senders.read().await;
        client_senders.contains_key(client_id)
    }

    /// 获取对讲会话信息
    pub async fn get_intercom_session(&self, device_id: &str) -> Option<(String, String)> {
        let connections = self.device_connections.read().await;
        if let Some(conn) = connections.get(device_id) {
            if conn.is_intercom_active {
                if let Some(client_id) = &conn.intercom_client_id {
                    return Some((conn.device_id.clone(), client_id.clone()));
                }
            }
        }
        None
    }

    /// 获取队列统计信息
    pub async fn get_stats(&self) -> QueueStats {
        self.stats.lock().await.clone()
    }

    /// 重置统计信息
    pub async fn reset_stats(&self) {
        let mut stats = self.stats.lock().await;
        *stats = QueueStats::default();
    }

    /// 清理不活跃的设备连接
    pub async fn cleanup_inactive_connections(&self, max_inactive_seconds: u64) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let mut to_remove = Vec::new();
        let connections = self.device_connections.read().await;

        for (device_id, conn) in connections.iter() {
            if !conn.is_intercom_active && (now - conn.last_active) > max_inactive_seconds {
                to_remove.push(device_id.clone());
            }
        }

        drop(connections);

        for device_id in to_remove {
            self.unregister_device(&device_id).await;
            info!("Cleaned up inactive device connection: {}", device_id);
        }
    }
}

impl Default for AudioQueueManager {
    fn default() -> Self {
        Self::new(AudioQueueConfig::default())
    }
}
