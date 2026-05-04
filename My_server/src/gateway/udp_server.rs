//! / UDP服务器模块
// 负责处理来自外部设备的UDP数据

use actix::prelude::*;
use log::{debug, error, info};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::net::UdpSocket;
use tokio::sync::RwLock;

use super::DeviceData;

// UDP客户端状态
pub struct UdpClientState {
    pub device_id: String,
    pub addr: SocketAddr,
    pub protocol: String,
    pub last_activity: Instant,
    pub total_data_received: u64,
    pub total_data_sent: u64,
    pub first_seen: Instant,
}

// UDP服务器
pub struct UdpServer {
    addr: SocketAddr,
    socket: Option<UdpSocket>,
    clients: Arc<RwLock<HashMap<String, UdpClientState>>>,
    protocol_analyzer: Addr<super::gprs_server::ProtocolAnalyzer>,
    central_service: Option<Addr<super::super::central::service::CentralService>>,
    cleanup_interval: Duration,
    client_timeout: Duration,
}

impl UdpServer {
    pub fn new(
        addr: SocketAddr,
        protocol_analyzer: Addr<super::gprs_server::ProtocolAnalyzer>,
        central_service: Option<Addr<super::super::central::service::CentralService>>,
    ) -> Self {
        Self {
            addr,
            socket: None,
            clients: Arc::new(RwLock::new(HashMap::new())),
            protocol_analyzer,
            central_service,
            cleanup_interval: Duration::from_secs(60),
            client_timeout: Duration::from_secs(300),
        }
    }

    // 启动UDP服务器
    pub async fn start(&mut self) -> Result<(), std::io::Error> {
        info!("Starting UDP server on {}", self.addr);

        let socket = UdpSocket::bind(self.addr).await?;
        self.socket = Some(socket);

        info!("UDP server started successfully on {}", self.addr);
        Ok(())
    }

    // 运行UDP服务器
    pub async fn run(&self, _ctx: &mut Context<Self>) {
        let socket = self.socket.as_ref().expect("UDP server not started");
        let clients = self.clients.clone();
        let protocol_analyzer = self.protocol_analyzer.clone();
        let central_service = self.central_service.clone();

        let mut buf = [0u8; 4096];

        loop {
            match socket.recv_from(&mut buf).await {
                Ok((n, addr)) => {
                    let data = &buf[..n];
                    info!("Received UDP data from {}: {} bytes", addr, n);
                    debug!("UDP data: {:?}", data);

                    // 解析设备ID和协议类型
                    if let Some((device_id, protocol)) = self.parse_device_info(data) {
                        // 更新客户端状态
                        {
                            let mut clients_write = clients.write().await;
                            clients_write.insert(
                                device_id.clone(),
                                UdpClientState {
                                    device_id: device_id.clone(),
                                    addr,
                                    protocol: protocol.clone(),
                                    last_activity: Instant::now(),
                                    total_data_received: 0,
                                    total_data_sent: 0,
                                    first_seen: Instant::now(),
                                },
                            );
                        }

                        // 发送设备数据消息到协议分析器
                        protocol_analyzer.do_send(DeviceData {
                            device_id: device_id.clone(),
                            data: data.to_vec(),
                            protocol: protocol.clone(),
                        });

                        // 发送设备数据消息到中心服务
                        if let Some(central) = central_service.clone() {
                            central.do_send(super::super::central::DeviceData {
                                device_id: device_id.clone(),
                                protocol: protocol.clone(),
                                data: data.to_vec(),
                                timestamp: std::time::SystemTime::now(),
                            });
                        }

                        // 处理响应(如果需要)
                        self.handle_response(socket, addr, data, &device_id, &protocol)
                            .await;
                    } else {
                        error!("Failed to parse device info from UDP data");
                    }
                }
                Err(e) => {
                    error!("UDP recv error: {}", e);
                    break;
                }
            }
        }
    }

    // 解析设备信息
    fn parse_device_info(&self, data: &[u8]) -> Option<(String, String)> {
        // 简单实现:假设数据以"device_id:protocol:"开头
        // 实际实现需要根据具体协议进行调整
        let data_str = String::from_utf8_lossy(data);
        if data_str.starts_with("device_id:") {
            let parts: Vec<&str> = data_str.split(':').collect();
            if parts.len() >= 3 {
                let device_id = parts[1].to_string();
                let protocol = parts[2].to_string();
                return Some((device_id, protocol));
            }
        }

        // 默认返回一个示例设备ID和协议
        Some(("unknown_udp".to_string(), "default".to_string()))
    }

    // 处理响应
    async fn handle_response(
        &self,
        socket: &UdpSocket,
        addr: SocketAddr,
        _data: &[u8],
        device_id: &str,
        protocol: &str,
    ) {
        // 简单实现:根据协议类型生成响应
        // 实际实现需要根据具体协议进行调整
        let response = match protocol {
            "GB" => {
                // 生成GB协议响应
                vec![0x7e, 0x01, 0x00, 0x01, 0x7e]
            }
            "BSJ" => {
                // 生成BSJ协议响应
                vec![0x7e, 0x02, 0x00, 0x01, 0x7e]
            }
            _ => {
                // 生成默认响应
                vec![0x7e, 0x00, 0x00, 0x01, 0x7e]
            }
        };

        if let Err(e) = socket.send_to(&response, addr).await {
            error!(
                "Failed to send UDP response to {} for device {}: {}",
                addr, device_id, e
            );
        } else {
            // 更新客户端发送统计
            let mut clients_write = self.clients.write().await;
            if let Some(client) = clients_write.get_mut(device_id) {
                client.total_data_sent += response.len() as u64;
            }
        }
    }

    // 发送UDP数据
    pub async fn send_data(&self, device_id: &str, data: &[u8]) -> Result<(), std::io::Error> {
        let clients = self.clients.read().await;
        if let Some(client) = clients.get(device_id) {
            if let Some(socket) = &self.socket {
                socket.send_to(data, client.addr).await?;

                // 更新客户端发送统计
                let mut clients_write = self.clients.write().await;
                if let Some(client) = clients_write.get_mut(device_id) {
                    client.total_data_sent += data.len() as u64;
                    client.last_activity = Instant::now();
                }

                info!("Sent UDP data to {}: {:?}", client.addr, data);
                Ok(())
            } else {
                Err(std::io::Error::new(
                    std::io::ErrorKind::NotConnected,
                    "UDP socket not initialized",
                ))
            }
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Device not found",
            ))
        }
    }

    // 清理过期连接
    async fn cleanup_stale_connections(
        clients: Arc<RwLock<HashMap<String, UdpClientState>>>,
        timeout: Duration,
    ) {
        let now = Instant::now();
        let mut stale_device_ids = Vec::new();

        // 找出所有过期的连接
        {
            let clients_read = clients.read().await;
            for (device_id, client) in clients_read.iter() {
                if now.duration_since(client.last_activity) > timeout {
                    stale_device_ids.push(device_id.clone());
                }
            }
        }

        // 清理过期连接
        if !stale_device_ids.is_empty() {
            let mut clients_write = clients.write().await;
            for device_id in stale_device_ids {
                if clients_write.remove(&device_id).is_some() {
                    info!("Cleaned up stale UDP connection for device {}", device_id);
                }
            }
        }
    }
}

impl Actor for UdpServer {
    type Context = Context<Self>;

    // 启动时运行UDP服务器
    fn started(&mut self, _ctx: &mut Self::Context) {
        info!("UDP server actor started");

        // 直接在当前actor中启动UDP服务器
        let addr = self.addr;
        let clients = self.clients.clone();
        let protocol_analyzer = self.protocol_analyzer.clone();
        let _central_service = self.central_service.clone();
        let cleanup_interval = self.cleanup_interval;
        let client_timeout = self.client_timeout;

        tokio::spawn(async move {
            let socket = match UdpSocket::bind(addr).await {
                Ok(socket) => {
                    info!("UDP server started successfully on {}", addr);
                    socket
                }
                Err(e) => {
                    error!("Failed to bind UDP server on {}: {}", addr, e);
                    return;
                }
            };

            // 启动连接清理任务
            tokio::spawn({
                let clients = clients.clone();
                async move {
                    loop {
                        tokio::time::sleep(cleanup_interval).await;
                        UdpServer::cleanup_stale_connections(clients.clone(), client_timeout).await;
                    }
                }
            });

            let mut buf = [0u8; 4096];
            loop {
                match socket.recv_from(&mut buf).await {
                    Ok((n, addr)) => {
                        let data = &buf[..n];
                        info!("Received UDP data from {}: {} bytes", addr, n);
                        debug!("UDP data: {:?}", data);

                        // 解析设备ID和协议类型
                        // 这里简化处理,实际应该根据协议解析
                        // 假设数据前10个字节是设备ID,接下来2个字节是协议类型
                        if let Some((device_id, protocol)) = {
                            if data.len() >= 12 {
                                let device_id = String::from_utf8_lossy(&data[..10]).to_string();
                                let protocol = String::from_utf8_lossy(&data[10..12]).to_string();
                                Some((device_id, protocol))
                            } else {
                                None
                            }
                        } {
                            // 更新客户端状态
                            let now = Instant::now();
                            {
                                let mut clients_write = clients.write().await;
                                let client_state = clients_write
                                    .entry(device_id.clone())
                                    .or_insert_with(|| UdpClientState {
                                        device_id: device_id.clone(),
                                        addr,
                                        protocol: protocol.clone(),
                                        last_activity: now,
                                        total_data_received: 0,
                                        total_data_sent: 0,
                                        first_seen: now,
                                    });
                                client_state.last_activity = now;
                                client_state.total_data_received += n as u64;
                                client_state.protocol = protocol.clone();
                                client_state.addr = addr;
                            }

                            // 发送设备数据消息
                            protocol_analyzer.do_send(DeviceData {
                                device_id: device_id.clone(),
                                data: data.to_vec(),
                                protocol: protocol.clone(),
                            });

                            // 处理响应(如果需要)
                            tokio::spawn(async move {
                                info!(
                                    "Handling response for device {} with protocol {}",
                                    device_id, protocol
                                );
                                // 发送确认响应
                                let response = format!("ACK:{}", device_id);
                                // 创建新的socket来发送响应
                                match UdpSocket::bind("0.0.0.0:0").await {
                                    Ok(response_socket) => {
                                        if let Err(e) =
                                            response_socket.send_to(response.as_bytes(), addr).await
                                        {
                                            error!("Failed to send UDP response: {}", e);
                                        }
                                    }
                                    _ => {
                                        error!("Failed to create UDP socket for response");
                                    }
                                }
                            });
                        } else {
                            error!("Failed to parse device info from UDP data");
                        }
                    }
                    Err(e) => {
                        error!("UDP recv error: {}", e);
                        // 短暂休眠后继续尝试
                        tokio::time::sleep(Duration::from_secs(1)).await;
                        continue;
                    }
                }
            }
        });
    }
}

// 发送UDP数据消息
#[derive(Message)]
#[rtype(result = "Result<(), std::io::Error>")]
pub struct SendUdpData {
    pub device_id: String,
    pub data: Vec<u8>,
}

// 发送UDP数据消息处理
impl Handler<SendUdpData> for UdpServer {
    type Result = ResponseFuture<Result<(), std::io::Error>>;

    fn handle(&mut self, msg: SendUdpData, _ctx: &mut Self::Context) -> Self::Result {
        let device_id = msg.device_id;
        let data = msg.data;
        let clients = self.clients.clone();

        // 由于UdpSocket不支持clone,我们需要重新绑定或使用其他机制发送数据
        Box::pin(async move {
            let clients = clients.read().await;
            if let Some(client) = clients.get(&device_id) {
                // 尝试直接发送数据
                match UdpSocket::bind("0.0.0.0:0").await {
                    Ok(socket) => {
                        socket.send_to(&data, client.addr).await?;
                        info!("Sent UDP data to {}: {:?}", client.addr, data);
                        Ok(())
                    }
                    _ => Err(std::io::Error::new(
                        std::io::ErrorKind::NotConnected,
                        "Failed to create UDP socket",
                    )),
                }
            } else {
                Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Device not found",
                ))
            }
        })
    }
}
