//! / TCP 服务器
use super::connection::ConnectionHandler;
use crate::truck_scale::config::TruckScaleConfig;
use anyhow::Result;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Semaphore;

/// Truck Scale TCP 服务器
pub struct TruckScaleServer {
    config: Arc<TruckScaleConfig>,
    pool: Arc<PgPool>,
    listener: Option<TcpListener>,
    connection_semaphore: Arc<Semaphore>,
}

impl TruckScaleServer {
    /// 创建新的服务器实例
    pub fn new(config: TruckScaleConfig, pool: Arc<PgPool>) -> Self {
        let config = Arc::new(config);
        let connection_semaphore = Arc::new(Semaphore::new(config.max_connections));

        Self {
            config,
            pool,
            listener: None,
            connection_semaphore,
        }
    }

    /// 启动服务器
    pub async fn start(&mut self) -> Result<()> {
        let listener = TcpListener::bind(&self.config.server_addr).await?;
        println!(
            "Truck Scale server listening on {}",
            self.config.server_addr
        );

        self.listener = Some(listener);

        self.run().await
    }

    /// 运行服务器主循环
    async fn run(&self) -> Result<()> {
        let listener = self
            .listener
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Server not started"))?;

        loop {
            // 接受新连接
            match listener.accept().await {
                Ok((stream, addr)) => {
                    // 检查连接数限制
                    if self.connection_semaphore.clone().try_acquire().is_err() {
                        println!(
                            "Connection rejected: maximum connections reached from {}",
                            addr
                        );
                        drop(stream);
                        continue;
                    }

                    let permit = self.connection_semaphore.clone().acquire_owned().await?;
                    let config = self.config.clone();
                    let pool = self.pool.clone();

                    // 启动连接处理器
                    tokio::spawn(async move {
                        let handler = ConnectionHandler::new(stream, addr, config, pool);

                        if let Err(e) = handler.handle().await {
                            eprintln!("Connection handler error: {}", e);
                        }

                        // 释放连接许可
                        drop(permit);
                    });
                }
                Err(e) => {
                    eprintln!("Error accepting connection: {}", e);
                }
            }
        }
    }

    /// 获取服务器状态
    pub fn status(&self) -> ServerStatus {
        let available = self.connection_semaphore.available_permits();
        let max_connections = self.config.max_connections;
        let active_connections = max_connections - available;

        ServerStatus {
            is_running: self.listener.is_some(),
            server_addr: self.config.server_addr,
            max_connections,
            active_connections,
            available_connections: available,
        }
    }
}

/// 服务器状态
#[derive(Debug, Clone)]
pub struct ServerStatus {
    pub is_running: bool,
    pub server_addr: std::net::SocketAddr,
    pub max_connections: usize,
    pub active_connections: usize,
    pub available_connections: usize,
}
