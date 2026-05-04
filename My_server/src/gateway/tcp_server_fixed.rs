// TCP服务器模块（修复版）
// 修复了连接管理、流管理、消息发送等问题

use actix::prelude::*;
use log::{debug, error, info, warn};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, Mutex, RwLock};

use super::{DeviceConnected, DeviceData, DeviceDisconnected};

// 消息发送任务
type MessageSender = mpsc::UnboundedSender<Vec<u8>>;

// TCP客户端状态（修复版）
pub struct TcpClientState {
    pub device_id: String,
    pub addr: SocketAddr,
    pub protocol: String,
    pub last_activity: Mutex<Instant>,
    pub message_sender: Mutex<Option<MessageSender>>, // 使用channel发送消息
    pub buffer: Mutex<Vec<u8>>,
    pub connection_time: Instant,
    pub total_data_received: Mutex<u64>,
    pub total_data_sent: Mutex<u64>,
}

impl TcpClientState {
    pub fn new(device_id: String, addr: SocketAddr, protocol: String) -> Self {
        Self {
            device_id,
            addr,
            protocol,
            last_activity: Mutex::new(Instant::now()),
            message_sender: Mutex::new(None),
            buffer: Mutex::new(Vec::with_capacity(4096)),
            connection_time: Instant::now(),
            total_data_received: Mutex::new(0),
            total_data_sent: Mutex::new(0),
        }
    }

    // 更新活动时间
    pub async fn update_activity(&self) {
        *self.last_activity.lock().await = Instant::now();
    }

    // 获取活动时间
    pub async fn get_last_activity(&self) -> Instant {
        *self.last_activity.lock().await
    }

    // 发送消息到客户端
    pub async fn send_message(&self, data: Vec<u8>) -> Result<(), String> {
        let sender = self.message_sender.lock().await;
        if let Some(sender) = sender.as_ref() {
            if let Err(e) = sender.send(data) {
                return Err(format!("Failed to send message: {}", e));
            }
            Ok(())
        } else {
            Err("Message sender not available".to_string())
        }
    }

    // 设置消息发送器
    pub async fn set_message_sender(&self, sender: MessageSender) {
        *self.message_sender.lock().await = Some(sender);
    }

    // 获取连接时长
    pub fn get_connection_duration(&self) -> Duration {
        self.connection_time.elapsed()
    }
}

// TCP服务器配置
#[derive(Clone)]
pub struct TcpServerConfig {
    pub addr: SocketAddr,
    pub max_connections: usize,
    pub cleanup_interval: Duration,
    pub client_timeout: Duration,
    pub buffer_size: usize,
}

impl Default for TcpServerConfig {
    fn default() -> Self {
        Self {
            addr: "0.0.0.0:9999".parse().unwrap(),
            max_connections: 1000,
            cleanup_interval: Duration::from_secs(60),
            client_timeout: Duration::from_secs(300),
            buffer_size: 4096,
        }
    }
}

// TCP服务器（修复版）
pub struct TcpServer {
    config: TcpServerConfig,
    listener: Option<TcpListener>,
    clients: Arc<RwLock<HashMap<String, Arc<TcpClientState>>>>,
    protocol_analyzer: Addr<super::gprs_server::ProtocolAnalyzer>,
    central_service: Option<Addr<super::super::central::service::CentralService>>,
}

impl TcpServer {
    pub fn new(
        config: TcpServerConfig,
        protocol_analyzer: Addr<super::gprs_server::ProtocolAnalyzer>,
        central_service: Option<Addr<super::super::central::service::CentralService>>,
    ) -> Self {
        Self {
            config,
            listener: None,
            clients: Arc::new(RwLock::new(HashMap::new())),
            protocol_analyzer,
            central_service,
        }
    }

    // 启动TCP服务器
    pub async fn start(&mut self) -> Result<(), std::io::Error> {
        info!("Starting TCP server on {}", self.config.addr);

        let listener = TcpListener::bind(self.config.addr).await?;
        self.listener = Some(listener);

        info!("TCP server started successfully on {}", self.config.addr);
        Ok(())
    }

    // 获取客户端连接数
    pub async fn get_client_count(&self) -> usize {
        self.clients.read().await.len()
    }

    // 获取指定客户端
    pub async fn get_client(&self, device_id: &str) -> Option<Arc<TcpClientState>> {
        self.clients.read().await.get(device_id).cloned()
    }

    // 发送数据到指定设备
    pub async fn send_data(&self, device_id: &str, data: &[u8]) -> Result<(), String> {
        let client = self.get_client(device_id).await
            .ok_or_else(|| format!("Device {} not found", device_id))?;

        client.send_message(data.to_vec()).await?;
        debug!("TCP sent {} bytes to device {}", data.len(), device_id);

        // 更新发送统计
        *client.total_data_sent.lock().await += data.len() as u64;

        Ok(())
    }

    // 广播数据到所有客户端
    pub async fn broadcast_data(&self, data: &[u8]) -> usize {
        let clients = self.clients.read().await;
        let mut success_count = 0;

        for (device_id, client) in clients.iter() {
            if client.send_message(data.to_vec()).await.is_ok() {
                success_count += 1;
                *client.total_data_sent.lock().await += data.len() as u64;
            } else {
                warn!("Failed to broadcast data to device {}", device_id);
            }
        }

        debug!("TCP broadcasted data to {}/{} clients", success_count, clients.len());
        success_count
    }

    // 获取统计信息
    pub async fn get_stats(&self) -> TcpServerStats {
        let clients = self.clients.read().await;
        let mut total_data_received = 0u64;
        let mut total_data_sent = 0u64;

        for client in clients.values() {
            total_data_received += *client.total_data_received.lock().await;
            total_data_sent += *client.total_data_sent.lock().await;
        }

        TcpServerStats {
            total_clients: clients.len(),
            total_data_received,
            total_data_sent,
            max_connections: self.config.max_connections,
            client_timeout: self.config.client_timeout,
        }
    }
}

// TCP服务器统计信息
#[derive(Debug, Clone)]
pub struct TcpServerStats {
    pub total_clients: usize,
    pub total_data_received: u64,
    pub total_data_sent: u64,
    pub max_connections: usize,
    pub client_timeout: Duration,
}

// 处理单个TCP连接（修复版）
async fn handle_connection(
    mut stream: TcpStream,
    addr: SocketAddr,
    clients: Arc<RwLock<HashMap<String, Arc<TcpClientState>>>>,
    protocol_analyzer: Addr<super::gprs_server::ProtocolAnalyzer>,
    central_service: Option<Addr<super::super::central::service::CentralService>>,
    buffer_size: usize,
    client_timeout: Duration,
) -> Result<(), std::io::Error> {
    // 读取设备ID和协议类型
    let (device_id, protocol) = TcpServer::read_device_info(&mut stream).await?;

    info!(
        "TCP device {} connected with protocol {} from {}",
        device_id, protocol, addr
    );

    // 创建消息发送channel
    let (sender, mut receiver) = mpsc::unbounded_channel::<Vec<u8>>();

    // 创建客户端状态
    let client_state = Arc::new(TcpClientState::new(
        device_id.clone(),
        addr,
        protocol.clone(),
    ));
    client_state.set_message_sender(sender).await;

    // 添加客户端到列表
    {
        let mut clients_write = clients.write().await;
        clients_write.insert(device_id.clone(), client_state.clone());
    }

    // 发送设备连接消息
    protocol_analyzer.do_send(DeviceConnected {
        device_id: device_id.clone(),
        addr,
        protocol: protocol.clone(),
    });

    if let Some(central) = central_service.clone() {
        central.do_send(super::super::central::RegisterDevice {
            device_id: device_id.clone(),
            protocol: protocol.clone(),
            addr,
        });
    }

    // 分离读写任务
    let stream_clone = stream.try_clone()?;
    let device_id_clone = device_id.clone();
    let protocol_clone = protocol.clone();
    let clients_clone = clients.clone();
    let protocol_analyzer_clone = protocol_analyzer.clone();
    let central_service_clone = central_service.clone();

    // 读取任务
    let read_task = tokio::spawn(async move {
        let mut read_buf = vec![0u8; buffer_size];
        let mut buffer = Vec::with_capacity(buffer_size);

        loop {
            match stream.read(&mut read_buf).await {
                Ok(0) => {
                    info!("TCP connection closed by device {}", device_id);
                    break;
                }
                Ok(n) => {
                    debug!("TCP received {} bytes from device {}", n, device_id);

                    // 更新客户端状态
                    if let Some(client) = clients_clone.read().await.get(&device_id) {
                        client.update_activity().await;
                        let mut client_buffer = client.buffer.lock().await;
                        client_buffer.extend_from_slice(&read_buf[..n]);
                        *client.total_data_received.lock().await += n as u64;
                    }

                    buffer.extend_from_slice(&read_buf[..n]);

                    // 解析数据包
                    while let Some(packet) = TcpServer::parse_packet(&mut buffer) {
                        debug!(
                            "TCP parsed packet from device {}: {:?}",
                            device_id, packet
                        );

                        // 发送数据到协议分析器
                        protocol_analyzer_clone.do_send(DeviceData {
                            device_id: device_id.clone(),
                            data: packet.clone(),
                            protocol: protocol.clone(),
                        });

                        // 发送数据到中心服务
                        if let Some(central) = central_service_clone.clone() {
                            central.do_send(super::super::central::DeviceData {
                                device_id: device_id.clone(),
                                protocol: protocol.clone(),
                                data: packet,
                                timestamp: std::time::SystemTime::now(),
                            });
                        }
                    }
                }
                Err(e) => {
                    error!("TCP read error from device {}: {}", device_id, e);
                    break;
                }
            }
        }
    });

    // 写入任务
    let write_task = tokio::spawn(async move {
        while let Some(data) = receiver.recv().await {
            if let Err(e) = stream_clone.write_all(&data).await {
                error!("TCP write error to device {}: {}", device_id_clone, e);
                break;
            }
            debug!("TCP sent {} bytes to device {}", data.len(), device_id_clone);
        }
    });

    // 等待任务完成
    tokio::select! {
        _ = read_task => {},
        _ = write_task => {},
    }

    // 清理客户端
    {
        let mut clients_write = clients.write().await;
        clients_write.remove(&device_id);
    }

    // 发送断开连接消息
    protocol_analyzer.do_send(DeviceDisconnected {
        device_id: device_id.clone(),
        reason: "TCP connection closed".to_string(),
    });

    if let Some(central) = central_service {
        central.do_send(super::super::central::UnregisterDevice {
            device_id: device_id.clone(),
            reason: "TCP connection closed".to_string(),
        });
    }

    info!("TCP device {} disconnected", device_id);
    Ok(())
}

impl TcpServer {
    // 读取设备信息
    async fn read_device_info(stream: &mut TcpStream) -> Result<(String, String), std::io::Error> {
        let mut buffer = vec![0u8; 1024];
        let n = stream.read(&mut buffer).await?;

        if n == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "TCP connection closed before receiving device info",
            ));
        }

        let data = String::from_utf8_lossy(&buffer[..n]);
        let parts: Vec<&str> = data.trim().split(':').collect();

        if parts.len() != 2 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid TCP device info format",
            ));
        }

        Ok((parts[0].to_string(), parts[1].to_string()))
    }

    // 解析数据包
    fn parse_packet(buffer: &mut Vec<u8>) -> Option<Vec<u8>> {
        // 简单的协议帧解析：假设每个完整的数据包以0x7e开头和结尾
        let start_pos = buffer.iter().position(|&b| b == 0x7e);
        let end_pos = buffer.iter().rposition(|&b| b == 0x7e);

        if let (Some(start), Some(end)) = (start_pos, end_pos) {
            if start < end {
                let packet = buffer[start..=end].to_vec();
                buffer.drain(0..=end);
                Some(packet)
            } else {
                None
            }
        } else {
            None
        }
    }

    // 清理过期连接
    pub async fn cleanup_stale_connections(&self) {
        let now = Instant::now();
        let timeout = self.config.client_timeout;
        let mut stale_device_ids = Vec::new();

        {
            let clients_read = self.clients.read().await;
            for (device_id, client) in clients_read.iter() {
                let last_activity = client.get_last_activity().await;
                if now.duration_since(last_activity) > timeout {
                    stale_device_ids.push(device_id.clone());
                    info!(
                        "TCP device {} is stale: last activity {:.2}s ago",
                        device_id,
                        now.duration_since(last_activity).as_secs_f64()
                    );
                }
            }
        }

        if !stale_device_ids.is_empty() {
            let mut clients_write = self.clients.write().await;
            for device_id in stale_device_ids {
                if let Some(client) = clients_write.remove(&device_id) {
                    info!("Cleaning up stale TCP connection for device {}", device_id);

                    // 发送断开连接消息
                    self.protocol_analyzer.do_send(DeviceDisconnected {
                        device_id: device_id.clone(),
                        reason: "Connection timeout".to_string(),
                    });

                    if let Some(central) = &self.central_service {
                        central.do_send(super::super::central::UnregisterDevice {
                            device_id,
                            reason: "Connection timeout".to_string(),
                        });
                    }
                }
            }
        }
    }
}

impl Actor for TcpServer {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        info!("TCP server actor started");

        let config = self.config.clone();
        let clients = self.clients.clone();
        let protocol_analyzer = self.protocol_analyzer.clone();
        let central_service = self.central_service.clone();

        tokio::spawn(async move {
            // 启动TCP监听器
            let listener = match TcpListener::bind(config.addr).await {
                Ok(listener) => {
                    info!("TCP server listening on {}", config.addr);
                    listener
                }
                Err(e) => {
                    error!("Failed to bind TCP server: {}", e);
                    return;
                }
            };

            // 启动连接清理任务
            let cleanup_clients = clients.clone();
            tokio::spawn(async move {
                loop {
                    tokio::time::sleep(config.cleanup_interval).await;
                    let mut clients_guard = cleanup_clients.write().await;
                    let now = Instant::now();
                    let mut stale_ids = Vec::new();

                    for (device_id, client) in clients_guard.iter() {
                        let last_activity = client.get_last_activity().await;
                        if now.duration_since(last_activity) > config.client_timeout {
                            stale_ids.push(device_id.clone());
                        }
                    }

                    for device_id in stale_ids {
                        if let Some(client) = clients_guard.remove(&device_id) {
                            info!("Cleaned up stale connection: {}", device_id);
                        }
                    }
                }
            });

            // 主监听循环
            loop {
                match listener.accept().await {
                    Ok((stream, addr)) => {
                        info!("New TCP connection from {}", addr);

                        // 检查连接数
                        let current_connections = clients.read().await.len();
                        if current_connections >= config.max_connections {
                            warn!(
                                "Max TCP connections reached ({}), rejecting {}",
                                config.max_connections, addr
                            );
                            let _ = stream.shutdown().await;
                            continue;
                        }

                        // 处理连接
                        tokio::spawn(handle_connection(
                            stream,
                            addr,
                            clients.clone(),
                            protocol_analyzer.clone(),
                            central_service.clone(),
                            config.buffer_size,
                            config.client_timeout,
                        ));
                    }
                    Err(e) => {
                        error!("TCP accept error: {}", e);
                        tokio::time::sleep(Duration::from_secs(1)).await;
                    }
                }
            }
        });
    }
}

// 发送TCP数据消息
#[derive(Message)]
#[rtype(result = "Result<(), String>")]
pub struct SendTcpData {
    pub device_id: String,
    pub data: Vec<u8>,
}

impl Handler<SendTcpData> for TcpServer {
    type Result = ResponseFuture<Result<(), String>>;

    fn handle(&mut self, msg: SendTcpData, _ctx: &mut Self::Context) -> Self::Result {
        let clients = self.clients.clone();
        let device_id = msg.device_id;
        let data = msg.data;

        Box::pin(async move {
            let client = clients.read().await
                .get(&device_id)
                .cloned()
                .ok_or_else(|| format!("Device {} not found", device_id))?;

            client.send_message(data).await
        })
    }
}

// 获取TCP服务器统计信息
#[derive(Message)]
#[rtype(result = "TcpServerStats")]
pub struct GetTcpStats;

impl Handler<GetTcpStats> for TcpServer {
    type Result = ResponseFuture<TcpServerStats>;

    fn handle(&mut self, _msg: GetTcpStats, _ctx: &mut Self::Context) -> Self::Result {
        let server = self.clone();
        Box::pin(async move { server.get_stats().await })
    }
}
