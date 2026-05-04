//! 音频转发管理器
//!
//! 负责音频流的接收、转换和转发
//! 对应C++示例中的write_audio_to_client和write_data函数
//! 使用Rust异步模型替代libuv的uv_write

use log::{debug, error, info, warn};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

use super::audio_queue::{AudioMessage, AudioQueueManager};

/// 终端TCP连接信息
pub struct TerminalConnection {
    /// 设备ID
    pub device_id: String,
    /// TCP写入通道（用于向终端发送音频）
    pub writer: mpsc::UnboundedSender<Vec<u8>>,
    /// 连接是否活跃
    pub is_active: bool,
}

/// 客户端WebSocket连接信息
pub struct ClientConnection {
    /// 客户端ID
    pub client_id: String,
    /// 关联的设备ID
    pub device_id: String,
    /// WebSocket写入通道（用于向客户端发送终端音频）
    pub writer: mpsc::UnboundedSender<AudioMessage>,
    /// 音频通道号
    pub channel: u8,
    /// 连接是否活跃
    pub is_active: bool,
}

/// 音频转发管理器
/// 核心功能：
/// 1. 接收客户端音频流，通过TCP转发给终端
/// 2. 接收终端音频流，通过WebSocket转发给客户端
pub struct AudioForwarder {
    /// 音频队列管理器
    queue_manager: Arc<AudioQueueManager>,
    /// 终端连接池（对应C++中的g_map_equpid_client）
    terminal_connections: Arc<RwLock<HashMap<String, TerminalConnection>>>,
    /// 客户端连接池
    client_connections: Arc<RwLock<HashMap<String, ClientConnection>>>,
    /// 统计信息
    stats: Arc<RwLock<ForwarderStats>>,
}

/// 转发器统计信息
#[derive(Debug, Clone, Default)]
pub struct ForwarderStats {
    /// 从客户端转发到终端的字节数
    pub client_to_terminal_bytes: u64,
    /// 从终端转发到客户端的字节数
    pub terminal_to_client_bytes: u64,
    /// 转发错误次数
    pub forward_errors: u64,
    /// 当前活跃转发会话数
    pub active_sessions: u64,
}

impl AudioForwarder {
    /// 创建新的音频转发器
    pub fn new(queue_manager: Arc<AudioQueueManager>) -> Self {
        Self {
            queue_manager,
            terminal_connections: Arc::new(RwLock::new(HashMap::new())),
            client_connections: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(ForwarderStats::default())),
        }
    }

    /// 注册终端连接（对应C++中的accept终端连接）
    pub async fn register_terminal(
        &self,
        device_id: String,
        writer: mpsc::UnboundedSender<Vec<u8>>,
    ) -> Result<(), String> {
        let connection = TerminalConnection {
            device_id: device_id.clone(),
            writer,
            is_active: true,
        };

        // 注册到队列管理器
        let (tx, rx) = mpsc::unbounded_channel();
        self.queue_manager.register_device(&device_id, tx).await?;

        // 保存连接
        let mut terminals = self.terminal_connections.write().await;
        terminals.insert(device_id.clone(), connection);

        // 启动音频转发任务（对应C++中的write_audio_to_client）
        self.start_terminal_audio_forwarding(device_id.clone(), rx)
            .await;

        info!("Terminal {} registered for audio forwarding", device_id);
        Ok(())
    }

    /// 注销终端连接
    pub async fn unregister_terminal(&self, device_id: &str) {
        let mut terminals = self.terminal_connections.write().await;
        if let Some(_conn) = terminals.remove(device_id) {
            let mut stats = self.stats.write().await;
            stats.active_sessions = stats.active_sessions.saturating_sub(1);
            info!("Terminal {} unregistered", device_id);
        }
        self.queue_manager.unregister_device(device_id).await;
    }

    /// 注册客户端连接（对应C++中的accept客户端连接）
    pub async fn register_client(
        &self,
        client_id: String,
        device_id: String,
        writer: mpsc::UnboundedSender<AudioMessage>,
        channel: u8,
    ) -> Result<(), String> {
        let connection = ClientConnection {
            client_id: client_id.clone(),
            device_id: device_id.clone(),
            writer,
            channel,
            is_active: true,
        };

        // 注册到队列管理器
        let (tx, rx) = mpsc::unbounded_channel();
        self.queue_manager.register_client(&client_id, tx).await?;

        // 保存连接
        let mut clients = self.client_connections.write().await;
        clients.insert(client_id.clone(), connection);

        // 启动客户端音频转发任务
        self.start_client_audio_forwarding(client_id.clone(), rx, device_id.clone(), channel)
            .await;

        info!(
            "Client {} registered for audio forwarding (device: {}, channel: {})",
            client_id, device_id, channel
        );
        Ok(())
    }

    /// 注销客户端连接
    pub async fn unregister_client(&self, client_id: &str) {
        let mut clients = self.client_connections.write().await;
        if let Some(conn) = clients.remove(client_id) {
            // 停止对讲会话
            let _ = self
                .queue_manager
                .stop_intercom_session(&conn.device_id)
                .await;

            let mut stats = self.stats.write().await;
            stats.active_sessions = stats.active_sessions.saturating_sub(1);
            info!("Client {} unregistered", client_id);
        }
        self.queue_manager.unregister_client(client_id).await;
    }

    /// 启动终端音频转发任务
    /// 对应C++中的write_audio_to_client函数
    /// 从消息队列接收音频并写入终端TCP连接
    async fn start_terminal_audio_forwarding(
        &self,
        device_id: String,
        mut receiver: mpsc::UnboundedReceiver<AudioMessage>,
    ) {
        let terminal_connections = self.terminal_connections.clone();
        let stats = self.stats.clone();

        tokio::spawn(async move {
            while let Some(message) = receiver.recv().await {
                // 查找终端连接（对应C++中的g_map_equpid_client.find）
                let terminals = terminal_connections.read().await;
                if let Some(terminal) = terminals.get(&device_id) {
                    if !terminal.is_active {
                        warn!("Terminal {} is not active, dropping audio frame", device_id);
                        continue;
                    }

                    // 转发音频数据到终端（对应C++中的write_data）
                    if terminal.writer.send(message.data.clone()).is_err() {
                        error!("Failed to send audio to terminal {}", device_id);
                        let mut stats = stats.write().await;
                        stats.forward_errors += 1;
                    } else {
                        debug!(
                            "Forwarded {} bytes audio to terminal {}",
                            message.data.len(),
                            device_id
                        );

                        let mut stats = stats.write().await;
                        stats.client_to_terminal_bytes += message.data.len() as u64;
                    }
                } else {
                    warn!("Terminal {} not found, dropping audio frame", device_id);
                    let mut stats = stats.write().await;
                    stats.forward_errors += 1;
                }
            }

            info!("Terminal audio forwarding task stopped for {}", device_id);
        });
    }

    /// 启动客户端音频转发任务
    /// 从终端接收音频并转发给客户端
    async fn start_client_audio_forwarding(
        &self,
        client_id: String,
        mut receiver: mpsc::UnboundedReceiver<AudioMessage>,
        _device_id: String,
        _channel: u8,
    ) {
        let client_connections = self.client_connections.clone();
        let stats = self.stats.clone();

        tokio::spawn(async move {
            while let Some(message) = receiver.recv().await {
                // 查找客户端连接
                let clients = client_connections.read().await;
                if let Some(client) = clients.get(&client_id) {
                    if !client.is_active {
                        warn!("Client {} is not active, dropping audio frame", client_id);
                        continue;
                    }

                    // 转发音频数据给客户端
                    if client.writer.send(message.clone()).is_err() {
                        error!("Failed to send audio to client {}", client_id);
                        let mut stats = stats.write().await;
                        stats.forward_errors += 1;
                    } else {
                        debug!(
                            "Forwarded {} bytes terminal audio to client {}",
                            message.data.len(),
                            client_id
                        );

                        let mut stats = stats.write().await;
                        stats.terminal_to_client_bytes += message.data.len() as u64;
                    }
                } else {
                    warn!("Client {} not found, dropping audio frame", client_id);
                    let mut stats = stats.write().await;
                    stats.forward_errors += 1;
                }
            }

            info!("Client audio forwarding task stopped for {}", client_id);
        });
    }

    /// 处理客户端音频数据
    /// 对应C++中的writeAudioQueue函数
    /// 接收客户端推送的音频，写入消息队列等待转发到终端
    pub async fn handle_client_audio(
        &self,
        client_id: &str,
        audio_data: Vec<u8>,
    ) -> Result<(), String> {
        let clients = self.client_connections.read().await;
        if let Some(client) = clients.get(client_id) {
            self.queue_manager
                .write_audio_to_queue(
                    audio_data,
                    client.device_id.clone(),
                    client_id.to_string(),
                    client.channel,
                )
                .await
        } else {
            Err(format!("Client {} not found", client_id))
        }
    }

    /// 处理终端音频数据
    /// 接收终端上传的音频，写入消息队列等待转发给客户端
    pub async fn handle_terminal_audio(
        &self,
        device_id: &str,
        audio_data: Vec<u8>,
        channel: u8,
    ) -> Result<(), String> {
        // 查找对讲会话中的客户端
        if let Some((_, client_id)) = self.queue_manager.get_intercom_session(device_id).await {
            self.queue_manager
                .write_terminal_audio_to_queue(
                    audio_data,
                    device_id.to_string(),
                    client_id,
                    channel,
                )
                .await
        } else {
            Err(format!(
                "No active intercom session for device {}",
                device_id
            ))
        }
    }

    /// 开始对讲会话
    /// 对应流程：平台下发0x9101请求后调用
    pub async fn start_intercom_session(
        &self,
        device_id: &str,
        client_id: &str,
    ) -> Result<(), String> {
        // 检查设备是否在线
        if !self.queue_manager.is_device_online(device_id).await {
            return Err(format!("Device {} is not connected", device_id));
        }

        // 检查客户端是否在线
        if !self.queue_manager.is_client_online(client_id).await {
            return Err(format!("Client {} is not connected", client_id));
        }

        // 开始对讲会话
        self.queue_manager
            .start_intercom_session(device_id, client_id)
            .await?;

        let mut stats = self.stats.write().await;
        stats.active_sessions += 1;

        info!(
            "Intercom session started: device={}, client={}",
            device_id, client_id
        );
        Ok(())
    }

    /// 停止对讲会话
    pub async fn stop_intercom_session(&self, device_id: &str) -> Result<(), String> {
        self.queue_manager.stop_intercom_session(device_id).await?;

        let mut stats = self.stats.write().await;
        stats.active_sessions = stats.active_sessions.saturating_sub(1);

        info!("Intercom session stopped for device {}", device_id);
        Ok(())
    }

    /// 获取统计信息
    pub async fn get_stats(&self) -> ForwarderStats {
        self.stats.read().await.clone()
    }

    /// 检查设备是否有活跃对讲会话
    pub async fn has_active_intercom(&self, device_id: &str) -> bool {
        self.queue_manager
            .get_intercom_session(device_id)
            .await
            .is_some()
    }
}
