//! /! 车联网通讯服务器
//! 负责监听端口并处理设备连接

use actix::prelude::*;
use anyhow::Result;
use log::{error, info};
use sqlx::PgPool;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::AsyncReadExt;
use tokio::net::TcpListener;
use tokio::sync::Semaphore;
use tokio::time::timeout;

use super::protocol::ProtocolParser;
use super::router::MessageRouter;
use super::session::SessionManager;

/// 车联网通讯服务器
#[derive(Debug, Clone)]
pub struct VehicleCommServer {
    /// 服务器地址
    addr: SocketAddr,
    /// 最大连接数
    max_connections: usize,
    /// 协议解析器
    protocol_parser: ProtocolParser,
    /// 会话管理器
    session_manager: SessionManager,
    /// 消息路由器
    message_router: MessageRouter,
    /// 连接信号量
    connection_semaphore: Arc<Semaphore>,
}

/// 启动服务器消息
#[derive(Message)]
#[rtype(result = "Result<()>")]
pub struct StartServer;

/// 获取服务器状态消息
#[derive(Message)]
#[rtype(result = "Result<ServerStatus>")]
pub struct GetServerStatus;

impl VehicleCommServer {
    /// 创建新的服务器
    pub fn new(addr: SocketAddr, max_connections: usize, pool: Option<PgPool>) -> Self {
        Self {
            addr,
            max_connections,
            protocol_parser: ProtocolParser::new(),
            session_manager: SessionManager::with_default_timeout(),
            message_router: MessageRouter::new(pool.clone()),
            connection_semaphore: Arc::new(Semaphore::new(max_connections)),
        }
    }

    /// 运行服务器(带监听器)
    async fn run_with_listener(&self, listener: TcpListener) -> Result<()> {
        loop {
            match listener.accept().await {
                Ok((stream, addr)) => {
                    info!("New connection from {}", addr);

                    // 检查连接数限制
                    if self.connection_semaphore.clone().try_acquire().is_err() {
                        error!(
                            "Maximum connections reached, rejecting connection from {}",
                            addr
                        );
                        continue;
                    }

                    let permit = self.connection_semaphore.clone().acquire_owned().await?;
                    let protocol_parser = self.protocol_parser.clone();
                    let session_manager = self.session_manager.clone();
                    let message_router = self.message_router.clone();

                    // 使用actix::spawn而不是tokio::spawn
                    actix::spawn(async move {
                        if let Err(e) = Self::handle_connection(
                            stream,
                            addr,
                            protocol_parser,
                            session_manager,
                            message_router,
                        )
                        .await
                        {
                            error!("Connection handler error: {}", e);
                        }

                        // 释放连接许可
                        drop(permit);
                    });
                }
                Err(e) => {
                    error!("Accept error: {}", e);
                    break;
                }
            }
        }

        Ok(())
    }

    /// 处理单个连接
    async fn handle_connection(
        mut stream: tokio::net::TcpStream,
        addr: SocketAddr,
        protocol_parser: ProtocolParser,
        _session_manager: SessionManager,
        message_router: MessageRouter,
    ) -> Result<()> {
        info!("Handling connection from {}", addr);

        // 读取数据
        let mut buffer = Vec::new();
        loop {
            let mut chunk = vec![0; 4096];
            match timeout(Duration::from_secs(30), stream.read(&mut chunk)).await {
                Ok(Ok(0)) => {
                    // 连接关闭
                    info!("Connection closed by {}", addr);
                    break;
                }
                Ok(Ok(n)) => {
                    buffer.extend_from_slice(&chunk[..n]);

                    // 尝试解析数据
                    if let Ok(message) = protocol_parser.parse(&buffer) {
                        // 处理消息
                        if let Err(e) = message_router.handle_message(message).await {
                            error!("Message handling error: {}", e);
                        }
                        // 清空缓冲区
                        buffer.clear();
                    }
                }
                Ok(Err(e)) => {
                    error!("Read error: {}", e);
                    break;
                }
                Err(_) => {
                    // 超时
                    info!("Read timeout from {}", addr);
                    break;
                }
            }
        }

        Ok(())
    }
}

/// 实现Actor trait
impl Actor for VehicleCommServer {
    type Context = Context<Self>;

    /// 启动时的初始化
    fn started(&mut self, _ctx: &mut Self::Context) {
        info!("VehicleCommServer started");
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        info!("VehicleCommServer stopped");
    }
}

/// 实现Handler<StartServer> trait
impl Handler<StartServer> for VehicleCommServer {
    type Result = Result<()>;

    fn handle(&mut self, _msg: StartServer, _ctx: &mut Self::Context) -> Self::Result {
        // 在Actor上下文中启动服务器
        let addr = self.addr;
        let server = self.clone();

        // 使用actix::spawn在Actor上下文中运行服务器
        actix::spawn(async move {
            let listener = TcpListener::bind(&addr)
                .await
                .expect("Failed to bind to address");
            info!(
                "Vehicle Communication Server started successfully on {}",
                addr
            );

            if let Err(e) = server.run_with_listener(listener).await {
                error!("Server error: {}", e);
            }
        });

        Ok(())
    }
}

/// 实现Handler<GetServerStatus> trait
impl Handler<GetServerStatus> for VehicleCommServer {
    type Result = Result<ServerStatus>;

    fn handle(&mut self, _msg: GetServerStatus, _ctx: &mut Self::Context) -> Self::Result {
        let available = self.connection_semaphore.available_permits();
        let active_connections = self.max_connections - available;

        Ok(ServerStatus {
            is_running: true,
            server_addr: self.addr,
            max_connections: self.max_connections,
            active_connections,
            available_connections: available,
            session_count: self.session_manager.session_count(),
        })
    }
}

/// 服务器状态
#[derive(Debug, Clone)]
pub struct ServerStatus {
    pub is_running: bool,
    pub server_addr: SocketAddr,
    pub max_connections: usize,
    pub active_connections: usize,
    pub available_connections: usize,
    pub session_count: usize,
}
