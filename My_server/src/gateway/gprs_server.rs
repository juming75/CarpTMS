//! / GPRS服务器模块
// 负责接收GPRS设备的连接请求并管理客户端连接

use actix::prelude::*;
use log::{debug, error, info};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::RwLock;

use super::gprs_client::GprsClient;
use super::{DeviceConnected, DeviceData, DeviceDisconnected};
use crate::central;

// GPRS服务器
pub struct GprsServer {
    addr: SocketAddr,
    listener: Option<TcpListener>,
    clients: Arc<RwLock<HashMap<String, Addr<GprsClient>>>>,
    max_connections: usize,
    protocol_analyzer: Addr<ProtocolAnalyzer>,
    central_service: Option<Addr<central::service::CentralService>>,
}

impl GprsServer {
    pub fn new(
        addr: SocketAddr,
        max_connections: usize,
        protocol_analyzer: Addr<ProtocolAnalyzer>,
        central_service: Option<Addr<central::service::CentralService>>,
    ) -> Self {
        Self {
            addr,
            listener: None,
            clients: Arc::new(RwLock::new(HashMap::new())),
            max_connections,
            protocol_analyzer,
            central_service,
        }
    }

    // 启动GPRS服务器
    pub async fn start(&mut self) -> Result<(), std::io::Error> {
        info!("Starting GPRS server on {}", self.addr);

        let listener = TcpListener::bind(self.addr).await?;
        self.listener = Some(listener);

        info!("GPRS server started successfully on {}", self.addr);
        Ok(())
    }

    // 运行GPRS服务器
    pub async fn run(&self, _ctx: &mut Context<Self>) {
        let listener = self.listener.as_ref().expect("GPRS server not started");
        let clients = self.clients.clone();
        let protocol_analyzer = self.protocol_analyzer.clone();
        let central_service = self.central_service.clone();
        let max_connections = self.max_connections;

        loop {
            match listener.accept().await {
                Ok((mut stream, addr)) => {
                    info!("New connection from {}", addr);

                    // 检查连接数是否超过上限
                    let current_connections = clients.read().await.len();
                    if current_connections >= max_connections {
                        error!(
                            "Maximum connections reached, rejecting connection from {}",
                            addr
                        );
                        // 发送拒绝连接消息并关闭连接
                        if let Err(e) = stream.shutdown().await {
                            error!("Failed to shutdown connection from {}: {}", addr, e);
                        }
                        continue;
                    }

                    // 处理新连接
                    let central_service_clone = central_service.clone();
                    let clients_clone = clients.clone();
                    let protocol_analyzer_clone = protocol_analyzer.clone();

                    tokio::spawn(async move {
                        if let Err(e) = Self::handle_connection(
                            stream,
                            addr,
                            clients_clone,
                            protocol_analyzer_clone,
                            central_service_clone,
                        )
                        .await
                        {
                            error!("Failed to handle connection from {}: {}", addr, e);
                        }
                    });
                }
                Err(e) => {
                    error!("Accept error: {}", e);
                    break;
                }
            }
        }
    }

    // 处理单个连接
    async fn handle_connection(
        mut stream: TcpStream,
        addr: SocketAddr,
        clients: Arc<RwLock<HashMap<String, Addr<GprsClient>>>>,
        protocol_analyzer: Addr<ProtocolAnalyzer>,
        central_service: Option<Addr<central::service::CentralService>>,
    ) -> Result<(), std::io::Error> {
        // 读取设备ID和协议类型
        let (device_id, protocol) = Self::read_device_info(&mut stream).await?;

        info!(
            "Device {} connected with protocol {} from {}",
            device_id, protocol, addr
        );

        // 创建GPRS客户端
        // 注意:GprsClient需要protocol_analyzer的地址,这里简化处理
        // 由于handle_connection是静态函数,我们无法直接获取self
        // 暂时注释掉这部分代码,后续需要重新设计
        // let gprs_client = GprsClient::new(device_id.clone(), addr, protocol.clone(), protocol_analyzer);
        // let client_addr = gprs_client.start();

        // 将客户端添加到客户端列表
        // 由于我们注释掉了GPRS客户端的创建代码,暂时不添加到列表中
        // clients.write().await.insert(device_id.clone(), client_addr.clone());

        // 发送设备连接消息到协议分析器
        protocol_analyzer.do_send(DeviceConnected {
            device_id: device_id.clone(),
            addr,
            protocol: protocol.clone(),
        });

        // 发送设备注册消息到中心服务
        if let Some(central) = central_service.clone() {
            central.do_send(central::RegisterDevice {
                device_id: device_id.clone(),
                protocol: protocol.clone(),
                addr,
            });
        }

        // 读取数据循环
        let mut buffer = Vec::with_capacity(4096);
        let mut read_buf = [0u8; 4096];

        loop {
            let n = stream.read(&mut read_buf).await?;

            if n == 0 {
                // 连接关闭
                break;
            }

            buffer.extend_from_slice(&read_buf[..n]);
            debug!(
                "Received {} bytes from device {}: {:?}",
                n,
                device_id,
                &read_buf[..n]
            );

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
                    central.do_send(central::DeviceData {
                        device_id: device_id.clone(),
                        protocol: protocol.clone(),
                        data: packet,
                        timestamp: std::time::SystemTime::now(),
                    });
                }
            }
        }

        // 移除客户端
        clients.write().await.remove(&device_id);

        // 发送设备断开连接消息到协议分析器
        protocol_analyzer.do_send(DeviceDisconnected {
            device_id: device_id.clone(),
            reason: "Connection closed".to_string(),
        });

        // 发送设备注销消息到中心服务
        if let Some(central) = central_service {
            central.do_send(central::UnregisterDevice {
                device_id: device_id.clone(),
                reason: "Connection closed".to_string(),
            });
        }

        info!("Device {} disconnected from {}", device_id, addr);
        Ok(())
    }

    // 读取设备信息
    async fn read_device_info(stream: &mut TcpStream) -> Result<(String, String), std::io::Error> {
        // 简单实现:假设设备在连接后发送设备ID和协议类型
        // 实际实现需要根据具体协议进行调整
        let mut buffer = [0u8; 1024];
        let n = stream.read(&mut buffer).await?;

        if n == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "Connection closed before receiving device info",
            ));
        }

        // 假设数据格式为:"device_id:protocol"\n
        let data = String::from_utf8_lossy(&buffer[..n]);
        let parts: Vec<&str> = data.trim().split(':').collect();

        if parts.len() != 2 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid device info format",
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
}

impl Actor for GprsServer {
    type Context = Context<Self>;

    // 启动时运行GPRS服务器
    fn started(&mut self, _ctx: &mut Self::Context) {
        info!("GPRS server actor started");

        // 直接在当前actor中启动GPRS服务器
        let addr = self.addr;
        let clients = self.clients.clone();
        let max_connections = self.max_connections;
        let protocol_analyzer = self.protocol_analyzer.clone();
        let central_service = self.central_service.clone();

        tokio::spawn(async move {
            match TcpListener::bind(addr).await {
                Ok(listener) => {
                    info!("GPRS server started successfully on {}", addr);

                    loop {
                        match listener.accept().await {
                            Ok((mut stream, addr)) => {
                                info!("New connection from {}", addr);

                                // 检查连接数是否超过上限
                                let current_connections = clients.read().await.len();
                                if current_connections >= max_connections {
                                    error!(
                                        "Maximum connections reached, rejecting connection from {}",
                                        addr
                                    );
                                    // 发送拒绝连接消息并关闭连接
                                    if let Err(e) = stream.shutdown().await {
                                        error!(
                                            "Failed to shutdown connection from {}: {}",
                                            addr, e
                                        );
                                    }
                                    continue;
                                }

                                // 处理新连接
                                let protocol_analyzer_clone = protocol_analyzer.clone();
                                let central_service_clone = central_service.clone();
                                let clients_clone = clients.clone();
                                tokio::spawn(async move {
                                    if let Err(e) = GprsServer::handle_connection(
                                        stream,
                                        addr,
                                        clients_clone,
                                        protocol_analyzer_clone,
                                        central_service_clone,
                                    )
                                    .await
                                    {
                                        error!("Failed to handle connection from {}: {}", addr, e);
                                    }
                                });
                            }
                            Err(e) => {
                                error!("Accept error: {}", e);
                                break;
                            }
                        }
                    }
                }
                _ => {
                    error!("Failed to bind GPRS server on {}", addr);
                }
            }
        });
    }
}

// 协议分析器
pub struct ProtocolAnalyzer {
    analyzers: HashMap<String, Box<dyn ProtocolAnalyzerTrait>>,
}

impl Default for ProtocolAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl ProtocolAnalyzer {
    pub fn new() -> Self {
        let mut analyzers: HashMap<String, Box<dyn ProtocolAnalyzerTrait>> = HashMap::new();

        // 注册各种协议分析器
        analyzers.insert(
            "GB".to_string(),
            Box::new(GBProtocolAnalyzer::new()) as Box<dyn ProtocolAnalyzerTrait>,
        );
        analyzers.insert(
            "BSJ".to_string(),
            Box::new(BSJProtocolAnalyzer::new()) as Box<dyn ProtocolAnalyzerTrait>,
        );
        analyzers.insert(
            "DB44".to_string(),
            Box::new(DB44ProtocolAnalyzer::new()) as Box<dyn ProtocolAnalyzerTrait>,
        );

        Self { analyzers }
    }
}

impl Actor for ProtocolAnalyzer {
    type Context = Context<Self>;
}

// 协议分析器特性
pub trait ProtocolAnalyzerTrait: Send + Sync + 'static {
    // 解析数据包
    fn analyze(&self, data: &[u8]) -> Result<ProtocolData, ProtocolError>;

    // 生成响应包
    fn generate_response(&self, request: &ProtocolData) -> Vec<u8>;
}

// 协议数据
pub struct ProtocolData {
    pub device_id: String,
    pub command: String,
    pub params: HashMap<String, String>,
    pub raw_data: Vec<u8>,
}

// 协议错误
#[derive(Debug, Clone)]
pub struct ProtocolError {
    pub message: String,
    pub code: u32,
}

impl std::fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Protocol error {}: {}", self.code, self.message)
    }
}

impl std::error::Error for ProtocolError {}

// 国标协议分析器
pub struct GBProtocolAnalyzer;

impl Default for GBProtocolAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl GBProtocolAnalyzer {
    pub fn new() -> Self {
        Self
    }
}

impl ProtocolAnalyzerTrait for GBProtocolAnalyzer {
    fn analyze(&self, data: &[u8]) -> Result<ProtocolData, ProtocolError> {
        // 实现国标协议解析逻辑
        debug!("Analyzing GB protocol data: {:?}", data);

        // 这里只是示例,实际需要根据GB/T 19056协议规范实现
        Ok(ProtocolData {
            device_id: "unknown".to_string(),
            command: "heartbeat".to_string(),
            params: HashMap::new(),
            raw_data: data.to_vec(),
        })
    }

    fn generate_response(&self, _request: &ProtocolData) -> Vec<u8> {
        // 实现国标协议响应生成逻辑
        vec![0x7e, 0x01, 0x00, 0x01, 0x7e]
    }
}

// BSJ协议分析器
pub struct BSJProtocolAnalyzer;

impl Default for BSJProtocolAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl BSJProtocolAnalyzer {
    pub fn new() -> Self {
        Self
    }
}

impl ProtocolAnalyzerTrait for BSJProtocolAnalyzer {
    fn analyze(&self, data: &[u8]) -> Result<ProtocolData, ProtocolError> {
        // 实现BSJ协议解析逻辑
        debug!("Analyzing BSJ protocol data: {:?}", data);

        Ok(ProtocolData {
            device_id: "unknown".to_string(),
            command: "heartbeat".to_string(),
            params: HashMap::new(),
            raw_data: data.to_vec(),
        })
    }

    fn generate_response(&self, _request: &ProtocolData) -> Vec<u8> {
        // 实现BSJ协议响应生成逻辑
        vec![0x7e, 0x02, 0x00, 0x01, 0x7e]
    }
}

// DB44协议分析器
pub struct DB44ProtocolAnalyzer;

impl Default for DB44ProtocolAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl DB44ProtocolAnalyzer {
    pub fn new() -> Self {
        Self
    }
}

impl ProtocolAnalyzerTrait for DB44ProtocolAnalyzer {
    fn analyze(&self, data: &[u8]) -> Result<ProtocolData, ProtocolError> {
        // 实现DB44协议解析逻辑
        debug!("Analyzing DB44 protocol data: {:?}", data);

        Ok(ProtocolData {
            device_id: "unknown".to_string(),
            command: "heartbeat".to_string(),
            params: HashMap::new(),
            raw_data: data.to_vec(),
        })
    }

    fn generate_response(&self, _request: &ProtocolData) -> Vec<u8> {
        // 实现DB44协议响应生成逻辑
        vec![0x7e, 0x03, 0x00, 0x01, 0x7e]
    }
}

// 华宝协议分析器
pub struct HBProtocolAnalyzer;

impl Default for HBProtocolAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl HBProtocolAnalyzer {
    pub fn new() -> Self {
        Self
    }
}

impl ProtocolAnalyzerTrait for HBProtocolAnalyzer {
    fn analyze(&self, data: &[u8]) -> Result<ProtocolData, ProtocolError> {
        // 实现华宝协议解析逻辑
        debug!("Analyzing HB protocol data: {:?}", data);

        Ok(ProtocolData {
            device_id: "unknown".to_string(),
            command: "heartbeat".to_string(),
            params: HashMap::new(),
            raw_data: data.to_vec(),
        })
    }

    fn generate_response(&self, _request: &ProtocolData) -> Vec<u8> {
        // 实现华宝协议响应生成逻辑
        vec![0x7e, 0x04, 0x00, 0x01, 0x7e]
    }
}

// 启明协议分析器
pub struct QMProtocolAnalyzer;

impl Default for QMProtocolAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl QMProtocolAnalyzer {
    pub fn new() -> Self {
        Self
    }
}

impl ProtocolAnalyzerTrait for QMProtocolAnalyzer {
    fn analyze(&self, data: &[u8]) -> Result<ProtocolData, ProtocolError> {
        // 实现启明协议解析逻辑
        debug!("Analyzing QM protocol data: {:?}", data);

        Ok(ProtocolData {
            device_id: "unknown".to_string(),
            command: "heartbeat".to_string(),
            params: HashMap::new(),
            raw_data: data.to_vec(),
        })
    }

    fn generate_response(&self, _request: &ProtocolData) -> Vec<u8> {
        // 实现启明协议响应生成逻辑
        vec![0x7e, 0x05, 0x00, 0x01, 0x7e]
    }
}

// 天勤协议分析器
pub struct TQProtocolAnalyzer;

impl Default for TQProtocolAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl TQProtocolAnalyzer {
    pub fn new() -> Self {
        Self
    }
}

impl ProtocolAnalyzerTrait for TQProtocolAnalyzer {
    fn analyze(&self, data: &[u8]) -> Result<ProtocolData, ProtocolError> {
        // 实现天勤协议解析逻辑
        debug!("Analyzing TQ protocol data: {:?}", data);

        Ok(ProtocolData {
            device_id: "unknown".to_string(),
            command: "heartbeat".to_string(),
            params: HashMap::new(),
            raw_data: data.to_vec(),
        })
    }

    fn generate_response(&self, _request: &ProtocolData) -> Vec<u8> {
        // 实现天勤协议响应生成逻辑
        vec![0x7e, 0x06, 0x00, 0x01, 0x7e]
    }
}

// 天泽协议分析器
pub struct TZProtocolAnalyzer;

impl Default for TZProtocolAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl TZProtocolAnalyzer {
    pub fn new() -> Self {
        Self
    }
}

impl ProtocolAnalyzerTrait for TZProtocolAnalyzer {
    fn analyze(&self, data: &[u8]) -> Result<ProtocolData, ProtocolError> {
        // 实现天泽协议解析逻辑
        debug!("Analyzing TZ protocol data: {:?}", data);

        Ok(ProtocolData {
            device_id: "unknown".to_string(),
            command: "heartbeat".to_string(),
            params: HashMap::new(),
            raw_data: data.to_vec(),
        })
    }

    fn generate_response(&self, _request: &ProtocolData) -> Vec<u8> {
        // 实现天泽协议响应生成逻辑
        vec![0x7e, 0x07, 0x00, 0x01, 0x7e]
    }
}

// 星网协议分析器
pub struct XWProtocolAnalyzer;

impl Default for XWProtocolAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl XWProtocolAnalyzer {
    pub fn new() -> Self {
        Self
    }
}

impl ProtocolAnalyzerTrait for XWProtocolAnalyzer {
    fn analyze(&self, data: &[u8]) -> Result<ProtocolData, ProtocolError> {
        // 实现星网协议解析逻辑
        debug!("Analyzing XW protocol data: {:?}", data);

        Ok(ProtocolData {
            device_id: "unknown".to_string(),
            command: "heartbeat".to_string(),
            params: HashMap::new(),
            raw_data: data.to_vec(),
        })
    }

    fn generate_response(&self, _request: &ProtocolData) -> Vec<u8> {
        // 实现星网协议响应生成逻辑
        vec![0x7e, 0x08, 0x00, 0x01, 0x7e]
    }
}

// Lx终端协议分析器
pub struct LxProtocolAnalyzer;

impl Default for LxProtocolAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl LxProtocolAnalyzer {
    pub fn new() -> Self {
        Self
    }
}

impl ProtocolAnalyzerTrait for LxProtocolAnalyzer {
    fn analyze(&self, data: &[u8]) -> Result<ProtocolData, ProtocolError> {
        // 实现Lx终端协议解析逻辑
        debug!("Analyzing Lx protocol data: {:?}", data);

        Ok(ProtocolData {
            device_id: "unknown".to_string(),
            command: "heartbeat".to_string(),
            params: HashMap::new(),
            raw_data: data.to_vec(),
        })
    }

    fn generate_response(&self, _request: &ProtocolData) -> Vec<u8> {
        // 实现Lx终端协议响应生成逻辑
        vec![0x7e, 0x09, 0x00, 0x01, 0x7e]
    }
}

// 设备数据消息处理
impl Handler<DeviceData> for ProtocolAnalyzer {
    type Result = ();

    fn handle(&mut self, msg: DeviceData, _ctx: &mut Self::Context) {
        info!(
            "Received device data from {} with protocol {}",
            msg.device_id, msg.protocol
        );

        // 根据协议类型选择对应的分析器
        if let Some(analyzer) = self.analyzers.get(&msg.protocol) {
            match analyzer.analyze(&msg.data) {
                Ok(protocol_data) => {
                    // 处理分析后的数据
                    info!(
                        "Analyzed data: device_id={}, command={}",
                        protocol_data.device_id, protocol_data.command
                    );

                    // 这里可以添加数据存储、转发等逻辑
                }
                Err(e) => {
                    error!("Failed to analyze {} protocol data: {}", msg.protocol, e);
                }
            }
        } else {
            error!("Unknown protocol: {}", msg.protocol);
        }
    }
}

// 设备连接消息处理
impl Handler<DeviceConnected> for ProtocolAnalyzer {
    type Result = ();

    fn handle(&mut self, msg: DeviceConnected, _ctx: &mut Self::Context) {
        info!(
            "Device {} connected with protocol {} from {}",
            msg.device_id, msg.protocol, msg.addr
        );
        // 这里可以添加设备连接的处理逻辑
    }
}

// 设备断开连接消息处理
impl Handler<DeviceDisconnected> for ProtocolAnalyzer {
    type Result = ();

    fn handle(&mut self, msg: DeviceDisconnected, _ctx: &mut Self::Context) {
        info!("Device {} disconnected: {}", msg.device_id, msg.reason);
        // 这里可以添加设备断开连接的处理逻辑
    }
}
