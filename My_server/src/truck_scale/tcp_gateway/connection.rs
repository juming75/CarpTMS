//! / 连接处理器
use crate::truck_scale::config::TruckScaleConfig;
use crate::truck_scale::db::TruckScaleDb;
use crate::truck_scale::protocol::builder::ProtocolBuilder;
use crate::truck_scale::protocol::parser::ProtocolParser;
use anyhow::Result;
use sqlx::PgPool;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

/// 连接处理器
pub struct ConnectionHandler {
    stream: TcpStream,
    addr: SocketAddr,
    _config: Arc<TruckScaleConfig>,
    _db: TruckScaleDb,
}

impl ConnectionHandler {
    /// 创建新的连接处理器
    pub fn new(
        stream: TcpStream,
        addr: SocketAddr,
        config: Arc<TruckScaleConfig>,
        pool: Arc<PgPool>,
    ) -> Self {
        Self {
            stream,
            addr,
            _config: config,
            _db: TruckScaleDb::new(pool),
        }
    }

    /// 处理连接
    pub async fn handle(mut self) -> Result<()> {
        println!("New connection from {}", self.addr);

        let mut buffer = vec![0u8; 8192];
        let parser = ProtocolParser::new();
        let builder = ProtocolBuilder::new();

        loop {
            // 读取数据
            let n = match self.stream.read(&mut buffer).await {
                Ok(0) => {
                    println!("Connection closed by client: {}", self.addr);
                    break;
                }
                Ok(n) => n,
                Err(e) => {
                    eprintln!("Read error from {}: {}", self.addr, e);
                    break;
                }
            };

            // 解析协议
            let data = &buffer[..n];
            match parser.parse(data) {
                Ok(message) => {
                    println!("Received message from {}: {:?}", self.addr, message);

                    // 处理消息
                    // 这里可以使用 self.db 来执行数据库操作

                    // 构建响应
                    let response = builder.build_response(&message)?;

                    // 发送响应
                    if let Err(e) = self.stream.write_all(&response).await {
                        eprintln!("Write error to {}: {}", self.addr, e);
                        break;
                    }
                }
                Err(e) => {
                    eprintln!("Parse error from {}: {}", self.addr, e);
                    // TODO: 发送错误响应
                }
            }
        }

        println!("Closing connection: {}", self.addr);
        Ok(())
    }
}
