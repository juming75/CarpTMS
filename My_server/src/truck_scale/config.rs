//! / Truck Scale 3.5 配置
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TruckScaleConfig {
    /// TCP 服务器监听地址
    pub server_addr: SocketAddr,

    /// 最大并发连接数
    pub max_connections: usize,

    /// 心跳间隔(秒)
    pub heartbeat_interval: u64,

    /// 心跳超时(秒)
    pub heartbeat_timeout: u64,

    /// 会话超时(秒)
    pub session_timeout: u64,

    /// 是否启用 Deflate 压缩
    pub enable_compression: bool,

    /// 字符编码
    pub encoding: String,

    /// 数据库连接字符串
    pub database_url: String,
}

impl Default for TruckScaleConfig {
    fn default() -> Self {
        Self {
            server_addr: "0.0.0.0:9809".parse().expect("hardcoded addr"),
            max_connections: 100,
            heartbeat_interval: 60,
            heartbeat_timeout: 180,
            session_timeout: 86400, // 24小时
            enable_compression: true,
            encoding: "gb2312".to_string(),
            database_url: String::new(),
        }
    }
}

impl TruckScaleConfig {
    pub fn from_env() -> anyhow::Result<Self> {
        let mut config = Self::default();

        if let Ok(addr) = std::env::var("TRUCK_SCALE_ADDR") {
            config.server_addr = addr.parse()?;
        }

        if let Ok(max_conn) = std::env::var("TRUCK_SCALE_MAX_CONNECTIONS") {
            config.max_connections = max_conn.parse()?;
        }

        if let Ok(db_url) = std::env::var("DATABASE_URL") {
            config.database_url = db_url;
        }

        Ok(config)
    }
}
