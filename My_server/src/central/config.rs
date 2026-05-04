//! / 中心服务配置模块
// 定义中心服务的配置结构

use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::path::PathBuf;

// 中心服务配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CentralConfig {
    // 服务器配置
    pub host: String,
    pub port: u16,

    // 网关配置
    pub gateway_config: GatewayConfig,

    // 数据库配置
    pub database_config: DatabaseConfig,

    // 日志配置
    pub log_config: LogConfig,

    // 安全配置
    pub security_config: SecurityConfig,

    // 监控配置
    pub monitor_config: MonitorConfig,
}

// 网关配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayConfig {
    // GPRS服务器配置
    pub gprs_server_addr: SocketAddr,

    // TCP服务器配置
    pub tcp_server_addr: SocketAddr,
    // 第二个TCP服务器配置(用于车载终端/无人机/物联网终端)
    pub tcp_server_addr2: SocketAddr,

    // UDP服务器配置
    pub udp_server_addr: SocketAddr,

    // 最大连接数
    pub max_connections: usize,

    // 心跳间隔(秒)
    pub heartbeat_interval: u64,
}

// 手动实现Default trait
impl Default for GatewayConfig {
    fn default() -> Self {
        Self {
            gprs_server_addr: "0.0.0.0:8888".parse().expect("hardcoded addr"),
            tcp_server_addr: "0.0.0.0:8988".parse().expect("hardcoded addr"),
            tcp_server_addr2: "0.0.0.0:8085".parse().expect("hardcoded addr"),
            udp_server_addr: "0.0.0.0:7777".parse().expect("hardcoded addr"),
            max_connections: 1000,
            heartbeat_interval: 30,
        }
    }
}

// 数据库配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DatabaseConfig {
    // 数据库类型
    pub db_type: String,

    // 数据库连接字符串
    pub connection_string: String,

    // 最大连接数
    pub max_connections: u32,

    // 连接超时(秒)
    pub connection_timeout: u64,
}

// 日志配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LogConfig {
    // 日志级别
    pub log_level: String,

    // 日志文件路径
    pub log_file: PathBuf,

    // 日志文件大小限制(MB)
    pub log_file_size: u64,

    // 日志文件保留天数
    pub log_file_retention_days: u32,
}

// 安全配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SecurityConfig {
    // 是否启用TLS
    pub enable_tls: bool,

    // TLS证书路径
    pub tls_cert_path: PathBuf,

    // TLS密钥路径
    pub tls_key_path: PathBuf,

    // 令牌过期时间(秒)
    pub token_expiration: u64,

    // 刷新令牌过期时间(秒)
    pub refresh_token_expiration: u64,
}

// 监控配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MonitorConfig {
    // 是否启用监控
    pub enable_monitor: bool,

    // 监控端口
    pub monitor_port: u16,

    // 监控更新间隔(秒)
    pub monitor_interval: u64,

    // 告警阈值配置
    pub alert_thresholds: AlertThresholds,
}

// 告警阈值配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AlertThresholds {
    // 系统指标阈值
    pub cpu_usage_threshold: f64,
    pub memory_usage_threshold: f64,
    pub disk_usage_threshold: f64,
    pub connection_count_threshold: usize,
    pub data_receive_rate_threshold: f64,
    pub data_send_rate_threshold: f64,

    // 数据库指标阈值
    pub db_connections_usage_threshold: f64, // 数据库连接使用率阈值(%)
    pub db_query_duration_threshold: f64,    // 数据库查询延迟阈值(秒)

    // API指标阈值
    pub api_error_rate_threshold: f64,       // API错误率阈值(%)
    pub api_request_duration_threshold: f64, // API请求延迟阈值(秒)

    // JWT指标阈值
    pub jwt_validation_failure_rate: f64, // JWT验证失败率阈值(%)

    // 业务指标阈值
    pub device_offline_rate_threshold: f64, // 设备离线率阈值(%)
    pub weighing_data_daily_threshold: f64, // 每日称重数据异常阈值
    pub orders_pending_threshold: f64,      // 待处理订单异常阈值

    // 服务注册中心指标阈值
    pub service_instances_unhealthy_threshold: f64, // 不健康服务实例阈值

    // 缓存指标阈值
    pub cache_hit_rate_threshold: f64, // 缓存命中率阈值(%)
}

// 从环境变量加载配置
impl CentralConfig {
    pub fn from_env() -> Result<Self, std::io::Error> {
        // 服务器配置
        let host = std::env::var("CENTRAL_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
        let port = std::env::var("CENTRAL_PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse()
            .map_err(|e| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Invalid CENTRAL_PORT: {}", e),
                )
            })?;

        // 网关配置
        let gprs_server_addr = std::env::var("GPRS_SERVER_ADDR")
            .unwrap_or_else(|_| "0.0.0.0:8089".to_string())
            .parse()
            .map_err(|e| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Invalid GPRS_SERVER_ADDR: {}", e),
                )
            })?;

        let tcp_server_addr = std::env::var("TCP_SERVER_ADDR")
            .unwrap_or_else(|_| "0.0.0.0:8988".to_string())
            .parse()
            .map_err(|e| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Invalid TCP_SERVER_ADDR: {}", e),
                )
            })?;

        let tcp_server_addr2 = std::env::var("TCP_SERVER_ADDR2")
            .unwrap_or_else(|_| "0.0.0.0:8085".to_string())
            .parse()
            .map_err(|e| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Invalid TCP_SERVER_ADDR2: {}", e),
                )
            })?;

        let udp_server_addr = std::env::var("UDP_SERVER_ADDR")
            .unwrap_or_else(|_| "0.0.0.0:4519".to_string())
            .parse()
            .map_err(|e| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Invalid UDP_SERVER_ADDR: {}", e),
                )
            })?;

        // 创建GatewayConfig实例
        let gateway_config = GatewayConfig {
            gprs_server_addr,
            tcp_server_addr,
            tcp_server_addr2,
            udp_server_addr,
            max_connections: std::env::var("MAX_CONNECTIONS")
                .unwrap_or_else(|_| "1000".to_string())
                .parse()
                .unwrap_or(1000),
            heartbeat_interval: std::env::var("HEARTBEAT_INTERVAL")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .unwrap_or(30),
        };

        // 创建完整的CentralConfig实例
        let config = CentralConfig {
            host,
            port,
            gateway_config,
            database_config: DatabaseConfig::default(),
            log_config: LogConfig::default(),
            security_config: SecurityConfig::default(),
            monitor_config: MonitorConfig::default(),
        };

        Ok(config)
    }
}
