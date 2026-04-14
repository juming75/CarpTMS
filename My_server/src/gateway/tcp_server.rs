//! / TCP服务器模块
// 负责处理来自外部设备的TCP连接和数据

use actix::prelude::*;
use log::{debug, error, info};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{Mutex, RwLock};

use super::{DeviceConnected, DeviceData, DeviceDisconnected};
use crate::infrastructure::message_router::router::MessageRouter;
use crate::infrastructure::message_router::types::TcpDeviceMessage;

use crate::protocols::jt808::session::{DeviceConnect, DeviceDisconnect, Jt808SessionManager};

// TCP客户端状态 - 使用分离的读写半部
pub struct TcpClientState {
    pub device_id: String,
    pub addr: SocketAddr,
    pub protocol: String,
    pub last_activity: Mutex<Instant>, // 使用Mutex保护可变字段
    pub write_half: Arc<Mutex<tokio::net::tcp::OwnedWriteHalf>>, // 可共享的写入半部
    pub buffer: Mutex<Vec<u8>>,
    pub connection_time: Instant,
    pub total_data_received: Mutex<u64>, // 使用Mutex保护可变字段
    pub total_data_sent: Mutex<u64>,     // 使用Mutex保护可变字段
}

// TCP服务器
pub struct TcpServer {
    addr: SocketAddr,
    listener: Option<TcpListener>,
    clients: Arc<RwLock<HashMap<String, Arc<TcpClientState>>>>,
    max_connections: usize,
    protocol_analyzer: Addr<super::gprs_server::ProtocolAnalyzer>,
    central_service: Option<Addr<super::super::central::service::CentralService>>,
    message_router: Option<Addr<MessageRouter>>,
    session_manager: Option<Addr<Jt808SessionManager>>,
    cleanup_interval: Duration,
    client_timeout: Duration,
}

impl TcpServer {
    pub fn new(
        addr: SocketAddr,
        max_connections: usize,
        protocol_analyzer: Addr<super::gprs_server::ProtocolAnalyzer>,
        central_service: Option<Addr<super::super::central::service::CentralService>>,
        message_router: Option<Addr<MessageRouter>>,
        session_manager: Option<Addr<Jt808SessionManager>>,
    ) -> Self {
        Self {
            addr,
            listener: None,
            clients: Arc::new(RwLock::new(HashMap::new())),
            max_connections,
            protocol_analyzer,
            central_service,
            message_router,
            session_manager,
            cleanup_interval: Duration::from_secs(60),
            client_timeout: Duration::from_secs(300),
        }
    }

    // 设置消息路由器
    pub fn with_message_router(mut self, message_router: Addr<MessageRouter>) -> Self {
        self.message_router = Some(message_router);
        self
    }

    // 设置会话管理器
    pub fn with_session_manager(mut self, session_manager: Addr<Jt808SessionManager>) -> Self {
        self.session_manager = Some(session_manager);
        self
    }

    // 启动TCP服务器
    pub async fn start(&mut self) -> Result<(), std::io::Error> {
        info!("Starting TCP server on {}", self.addr);

        let listener = TcpListener::bind(self.addr).await?;
        self.listener = Some(listener);

        info!("TCP server started successfully on {}", self.addr);
        Ok(())
    }

    // 运行TCP服务器
    pub async fn run(&self, _ctx: &mut Context<Self>) {
        let listener = self.listener.as_ref().expect("TCP server not started");
        let clients = self.clients.clone();
        let protocol_analyzer = self.protocol_analyzer.clone();
        let central_service = self.central_service.clone();
        let max_connections = self.max_connections;

        loop {
            match listener.accept().await {
                Ok((mut stream, addr)) => {
                    info!("New TCP connection from {}", addr);

                    // 检查连接数是否超过上限
                    let current_connections = clients.read().await.len();
                    if current_connections >= max_connections {
                        error!(
                            "Maximum TCP connections reached, rejecting connection from {}",
                            addr
                        );
                        // 发送拒绝连接消息并关闭连接
                        if let Err(e) = stream.shutdown().await {
                            error!("Failed to shutdown TCP connection from {}: {}", addr, e);
                        }
                        continue;
                    }

                    // 处理新连接
                    let central_service_clone = central_service.clone();
                    let clients_clone = clients.clone();
                    let protocol_analyzer_clone = protocol_analyzer.clone();
                    let message_router_clone = self.message_router.clone();
                    let session_manager_clone = self.session_manager.clone();

                    tokio::spawn(async move {
                        if let Err(e) = Self::handle_connection(
                            stream,
                            addr,
                            clients_clone,
                            protocol_analyzer_clone,
                            central_service_clone,
                            message_router_clone,
                            session_manager_clone,
                        )
                        .await
                        {
                            error!("Failed to handle TCP connection from {}: {}", addr, e);
                        }
                    });
                }
                Err(e) => {
                    error!("TCP accept error: {}", e);
                    break;
                }
            }
        }
    }

    // 处理单个TCP连接
    async fn handle_connection(
        stream: TcpStream,
        addr: SocketAddr,
        clients: Arc<RwLock<HashMap<String, Arc<TcpClientState>>>>,
        protocol_analyzer: Addr<super::gprs_server::ProtocolAnalyzer>,
        central_service: Option<Addr<super::super::central::service::CentralService>>,
        message_router: Option<Addr<MessageRouter>>,
        session_manager: Option<Addr<Jt808SessionManager>>,
    ) -> Result<(), std::io::Error> {
        // 分离读写半部
        let (mut read_half, write_half) = stream.into_split();

        // 读取设备ID和协议类型
        let (device_id, protocol) = Self::read_device_info(&mut read_half).await?;

        info!(
            "TCP device {} connected with protocol {} from {}",
            device_id, protocol, addr
        );

        // 创建客户端状态 - 将写入半部包装在 Arc<Mutex<>> 中以支持共享
        let client_state = Arc::new(TcpClientState {
            device_id: device_id.clone(),
            addr,
            protocol: protocol.clone(),
            last_activity: Mutex::new(Instant::now()),
            write_half: Arc::new(Mutex::new(write_half)), // 保存可共享的写入半部
            buffer: Mutex::new(Vec::with_capacity(4096)),
            connection_time: Instant::now(),
            total_data_received: Mutex::new(0),
            total_data_sent: Mutex::new(0),
        });

        // 添加客户端到列表
        {
            let mut clients_write = clients.write().await;
            clients_write.insert(device_id.clone(), client_state.clone());
        }

        // 发送设备连接消息到协议分析器
        protocol_analyzer.do_send(DeviceConnected {
            device_id: device_id.clone(),
            addr,
            protocol: protocol.clone(),
        });

        // 发送设备注册消息到中心服务
        if let Some(central) = central_service.clone() {
            central.do_send(super::super::central::RegisterDevice {
                device_id: device_id.clone(),
                protocol: protocol.clone(),
                addr,
            });
        }

        // 如果是 JT808 协议,通知会话管理器
        if protocol == "jt808" || protocol.is_empty() {
            if let Some(session_mgr) = session_manager.clone() {
                session_mgr.do_send(DeviceConnect {
                    device_id: device_id.clone(),
                    phone: device_id.clone(), // 简化处理,使用 device_id 作为 phone
                });
                info!(
                    "JT808 device {} connected, session manager notified",
                    device_id
                );
            }
        }

        // 读取数据循环 - 使用 read_half
        let mut read_buf = [0u8; 4096];
        let mut buffer = Vec::with_capacity(4096);

        loop {
            let n = match read_half.read(&mut read_buf).await {
                Ok(n) => n,
                Err(e) => {
                    error!("TCP read error from device {}: {}", device_id, e);
                    break;
                }
            };

            if n == 0 {
                // 连接关闭
                break;
            }

            buffer.extend_from_slice(&read_buf[..n]);
            debug!(
                "TCP received {} bytes from device {}: {:?}",
                n,
                device_id,
                &read_buf[..n]
            );

            // 更新客户端状态
            {
                let clients_read = clients.read().await;
                if let Some(client) = clients_read.get(&device_id) {
                    // 更新最后活动时间
                    *client.last_activity.lock().await = Instant::now();
                    // 更新数据接收统计
                    let mut buffer_lock = client.buffer.lock().await;
                    buffer_lock.extend_from_slice(&read_buf[..n]);
                    *client.total_data_received.lock().await += n as u64;
                }
            }

            // 解析数据包
            while let Some(packet) = Self::parse_packet(&mut buffer) {
                // 发送设备数据消息到协议分析器
                protocol_analyzer.do_send(DeviceData {
                    device_id: device_id.clone(),
                    data: packet.clone(),
                    protocol: protocol.clone(),
                });

                // 发送设备数据消息到中心服务
                if let Some(central) = central_service.clone() {
                    central.do_send(super::super::central::DeviceData {
                        device_id: device_id.clone(),
                        protocol: protocol.clone(),
                        data: packet.clone(),
                        timestamp: std::time::SystemTime::now(),
                    });
                }

                // 尝试使用 JT808 解析器解析数据
                if protocol == "jt808" || protocol.is_empty() {
                    match crate::protocols::jt808::JT808Parser::parse_frame(&packet) {
                        Ok(jt808_msg) => {
                            debug!(
                                "Successfully parsed JT808 message: msg_id=0x{:04X}",
                                jt808_msg.msg_id
                            );

                            // 将 JT808 消息发送到消息路由器
                            if let Some(router) = message_router.clone() {
                                // 转换为 UnifiedMessage(简化版本)
                                let _unified_json =
                                    Self::jt808_to_unified(&device_id, &protocol, &jt808_msg);
                                router.do_send(TcpDeviceMessage {
                                    device_id: device_id.clone(),
                                    protocol: protocol.clone(),
                                    raw_data: packet.clone(),
                                    unified_msg: None,
                                    received_at: chrono::Utc::now(),
                                });
                            }
                        }
                        Err(e) => {
                            debug!("Failed to parse as JT808: {:?}", e);
                        }
                    }
                }

                // 发送设备数据到消息路由器(非 JT808 或解析失败)
                if let Some(router) = message_router.clone() {
                    router.do_send(TcpDeviceMessage {
                        device_id: device_id.clone(),
                        protocol: protocol.clone(),
                        raw_data: packet,
                        unified_msg: None,
                        received_at: chrono::Utc::now(),
                    });
                }
            }
        }

        // 清理客户端资源
        {
            let mut clients_write = clients.write().await;
            clients_write.remove(&device_id);
        }

        // 发送设备断开连接消息到协议分析器
        protocol_analyzer.do_send(DeviceDisconnected {
            device_id: device_id.clone(),
            reason: "TCP connection closed".to_string(),
        });

        // 发送设备注销消息到中心服务
        if let Some(central) = central_service {
            central.do_send(super::super::central::UnregisterDevice {
                device_id: device_id.clone(),
                reason: "TCP connection closed".to_string(),
            });
        }

        // 如果是 JT808 协议,通知会话管理器设备断开
        if protocol == "jt808" || protocol.is_empty() {
            if let Some(session_mgr) = session_manager.clone() {
                session_mgr.do_send(DeviceDisconnect {
                    device_id: device_id.clone(),
                    reason: "TCP connection closed".to_string(),
                });
                info!(
                    "JT808 device {} disconnected, session manager notified",
                    device_id
                );
            }
        }

        info!("TCP device {} disconnected from {}", device_id, addr);
        Ok(())
    }

    // 读取设备信息
    async fn read_device_info(
        stream: &mut tokio::net::tcp::OwnedReadHalf,
    ) -> Result<(String, String), std::io::Error> {
        // 简单实现:假设设备在连接后发送设备ID和协议类型
        // 实际实现需要根据具体协议进行调整
        let mut buffer = [0u8; 1024];
        let n = stream.read(&mut buffer).await?;

        if n == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "TCP connection closed before receiving device info",
            ));
        }

        // 假设数据格式为:"device_id:protocol"\n
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
        // 简单的协议帧解析:假设每个完整的数据包以0x7e开头和结尾
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

    // 将 JT808 消息转换为 UnifiedMessage JSON 字符串
    fn jt808_to_unified(
        device_id: &str,
        protocol: &str,
        msg: &crate::protocols::jt808::JT808Frame,
    ) -> String {
        use serde_json::json;

        let unified = json!({
            "device_id": device_id,
            "protocol": protocol,
            "message_type": "jt808",
            "msg_id": format!("0x{:04X}", msg.msg_id),
            "msg_sn": msg.flow_no,
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "data": {
                "phone": msg.phone.clone(),
                "msg_id": msg.msg_id,
            }
        });

        unified.to_string()
    }

    // 清理过期连接
    async fn cleanup_stale_connections(
        clients: Arc<RwLock<HashMap<String, Arc<TcpClientState>>>>,
        timeout: Duration,
    ) {
        let now = Instant::now();
        let mut stale_device_ids = Vec::new();

        // 找出所有过期的连接
        {
            let clients_read = clients.read().await;
            for (device_id, client) in clients_read.iter() {
                if now.duration_since(*client.last_activity.lock().await) > timeout {
                    stale_device_ids.push(device_id.clone());
                }
            }
        }

        // 清理过期连接
        if !stale_device_ids.is_empty() {
            let mut clients_write = clients.write().await;
            for device_id in stale_device_ids {
                if let Some(client) = clients_write.remove(&device_id) {
                    info!("Cleaning up stale TCP connection for device {}", device_id);

                    // 关闭写入半部
                    let mut write_half_lock = client.write_half.lock().await;
                    if let Err(e) = write_half_lock.shutdown().await {
                        error!(
                            "Failed to shutdown stale TCP connection for device {}: {}",
                            device_id, e
                        );
                    }
                }
            }
        }
    }

    // 发送数据到指定设备
    pub async fn send_data(&self, device_id: &str, data: &[u8]) -> Result<(), std::io::Error> {
        let clients_read = self.clients.read().await;
        if let Some(client) = clients_read.get(device_id) {
            // 获取客户端写入半部并发送数据
            let mut write_half_lock = client.write_half.lock().await;
            write_half_lock.write_all(data).await?;

            // 更新发送统计
            debug!("TCP sent {} bytes to device {}", data.len(), device_id);
            Ok(())
        } else {
            error!("Device {} not found in TCP clients", device_id);
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Device not found",
            ))
        }
    }
}

impl Actor for TcpServer {
    type Context = Context<Self>;

    // 启动时运行TCP服务器
    fn started(&mut self, _ctx: &mut Self::Context) {
        info!("TCP server actor started");

        // 直接在当前actor中启动TCP服务器
        let addr = self.addr;
        let clients = self.clients.clone();
        let protocol_analyzer = self.protocol_analyzer.clone();
        let max_connections = self.max_connections;
        let central_service = self.central_service.clone();
        let cleanup_interval = self.cleanup_interval;
        let client_timeout = self.client_timeout;

        tokio::spawn(async move {
            // 启动TCP监听器
            let listener = match TcpListener::bind(addr).await {
                Ok(listener) => {
                    info!("TCP server started successfully on {}", addr);
                    listener
                }
                Err(e) => {
                    error!("Failed to bind TCP server on {}: {}", addr, e);
                    return;
                }
            };

            // 启动连接清理任务
            tokio::spawn({
                let clients = clients.clone();
                async move {
                    loop {
                        tokio::time::sleep(cleanup_interval).await;
                        TcpServer::cleanup_stale_connections(clients.clone(), client_timeout).await;
                    }
                }
            });

            // 主监听循环
            loop {
                match listener.accept().await {
                    Ok((stream, addr)) => {
                        info!("New TCP connection from {}", addr);

                        // 检查连接数是否超过上限
                        let current_connections = clients.read().await.len();
                        if current_connections >= max_connections {
                            error!(
                                "Maximum TCP connections reached, rejecting connection from {}",
                                addr
                            );
                            // 发送拒绝连接消息并关闭连接
                            let mut stream_clone = stream;
                            if let Err(e) = stream_clone.shutdown().await {
                                error!("Failed to shutdown TCP connection from {}: {}", addr, e);
                            }
                            continue;
                        }

                        // 处理新连接
                        let protocol_analyzer_clone = protocol_analyzer.clone();
                        let central_service_clone = central_service.clone();
                        let clients_clone = clients.clone();
                        tokio::spawn(async move {
                            if let Err(e) = TcpServer::handle_connection(
                                stream,
                                addr,
                                clients_clone,
                                protocol_analyzer_clone,
                                central_service_clone,
                                None,
                                None,
                            )
                            .await
                            {
                                error!("Failed to handle TCP connection from {}: {}", addr, e);
                            }
                        });
                    }
                    Err(e) => {
                        error!("TCP accept error: {}", e);
                        // 短暂休眠后继续尝试
                        tokio::time::sleep(Duration::from_secs(1)).await;
                        continue;
                    }
                }
            }
        });
    }
}

// 发送TCP数据消息
#[derive(Message)]
#[rtype(result = "Result<(), std::io::Error>")]
pub struct SendTcpData {
    pub device_id: String,
    pub data: Vec<u8>,
}

// 发送TCP数据消息处理
impl Handler<SendTcpData> for TcpServer {
    type Result = ResponseFuture<Result<(), std::io::Error>>;

    fn handle(&mut self, msg: SendTcpData, _ctx: &mut Self::Context) -> Self::Result {
        let device_id = msg.device_id;
        let data = msg.data;
        let clients = self.clients.clone();

        Box::pin(async move {
            let clients_read = clients.read().await;
            if let Some(client) = clients_read.get(&device_id) {
                // 获取客户端写入半部并发送数据
                let mut write_half_lock = client.write_half.lock().await;
                write_half_lock.write_all(&data).await?;
                debug!(
                    "TCP sent {} bytes to device {}: {:?}",
                    data.len(),
                    device_id,
                    data
                );
                Ok(())
            } else {
                error!("Device {} not found in TCP clients", device_id);
                Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Device not found",
                ))
            }
        })
    }
}
