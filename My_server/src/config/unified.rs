//! /! 统一配置管理
//!
//! 提供统一的配置加载、验证和管理功能

use crate::config::architecture::ArchitectureConfig;
use crate::errors::{AppError, AppResult};
use argon2::Params;
use log::{info, warn};
use serde::{Deserialize, Serialize};
use std::env;
use std::path::Path;
use tokio::fs;

/// 统一应用配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedConfig {
    /// 架构配置
    #[serde(default)]
    pub architecture: ArchitectureConfig,
    /// 服务器配置
    pub server: ServerConfig,
    /// 数据库配置
    pub database: DatabaseConfig,
    /// Redis配置
    pub redis: RedisConfig,
    /// 安全配置
    pub security: SecurityConfig,
    /// 监控配置
    pub monitoring: MonitoringConfig,
    /// 日志配置
    pub logging: LoggingConfig,
    /// 网关配置
    pub gateway: GatewayConfig,
    /// 视频配置
    pub video: VideoConfig,
    /// 同步配置
    pub sync: SyncConfig,
    /// 熔断器配置
    pub circuit_breaker: CircuitBreakerConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// 主机地址
    #[serde(default = "default_host")]
    pub host: String,
    /// 端口
    #[serde(default = "default_port")]
    pub port: u16,
    /// 启用TLS
    #[serde(default)]
    pub enable_tls: bool,
    /// TLS证书路径
    #[serde(default)]
    pub tls_cert_path: Option<String>,
    /// TLS私钥路径
    #[serde(default)]
    pub tls_key_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// 连接URL
    pub url: String,
    /// 最大连接数
    #[serde(default = "default_db_max_connections")]
    pub max_connections: u32,
    /// 最小连接数
    #[serde(default = "default_db_min_connections")]
    pub min_connections: u32,
    /// 连接超时(秒)
    #[serde(default = "default_db_connect_timeout")]
    pub connect_timeout: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisConfig {
    /// 连接URL
    pub url: String,
    /// 最大连接数
    #[serde(default = "default_redis_max_connections")]
    pub max_connections: u32,
    /// 默认过期时间(秒)
    #[serde(default = "default_redis_default_ttl")]
    pub default_ttl: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// JWT密钥
    pub jwt_secret: String,
    /// JWT过期时间(秒)
    #[serde(default = "default_jwt_expiration")]
    pub jwt_expiration: u64,
    /// Argon2内存成本
    #[serde(default = "default_argon2_memory")]
    pub argon2_memory: u32,
    /// Argon2时间成本
    #[serde(default = "default_argon2_time")]
    pub argon2_time: u32,
    /// Argon2并行度
    #[serde(default = "default_argon2_parallelism")]
    pub argon2_parallelism: u32,
    /// 启用HTTPS
    #[serde(default)]
    pub enable_https: bool,
    /// 允许的CORS来源
    #[serde(default = "default_cors_origins")]
    pub allowed_origins: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    /// 启用Prometheus
    #[serde(default = "default_enable_prometheus")]
    pub enable_prometheus: bool,
    /// 启用追踪
    #[serde(default = "default_enable_tracing")]
    pub enable_tracing: bool,
    /// 追踪服务URL
    #[serde(default)]
    pub tracing_service_url: Option<String>,
    /// 慢查询阈值(毫秒)
    #[serde(default = "default_slow_query_threshold")]
    pub slow_query_threshold: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// 日志级别
    #[serde(default = "default_log_level")]
    pub level: String,
    /// 日志格式(json/text)
    #[serde(default = "default_log_format")]
    pub format: String,
    /// 日志文件路径
    #[serde(default)]
    pub file_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayConfig {
    /// JT808网关地址
    #[serde(default = "default_jt808_addr")]
    pub jt808_address: String,
    /// WebSocket网关地址
    #[serde(default = "default_ws_addr")]
    pub websocket_address: String,
    /// 最大连接数
    #[serde(default = "default_max_connections")]
    pub max_connections: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoConfig {
    /// 启用视频服务
    #[serde(default)]
    pub enabled: bool,
    /// JT1078最大连接数
    #[serde(default = "default_jt1078_max_connections")]
    pub jt1078_max_connections: usize,
    /// GB28181 SIP端口
    #[serde(default = "default_gb28181_sip_port")]
    pub gb28181_sip_port: u16,
    /// 视频存储路径
    #[serde(default = "default_video_storage")]
    pub storage_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    /// 启用同步
    #[serde(default)]
    pub enabled: bool,
    /// 同步间隔(秒)
    #[serde(default = "default_sync_interval")]
    pub interval_seconds: u64,
    /// 旧服务器地址
    #[serde(default = "default_legacy_host")]
    pub legacy_host: String,
    /// 旧服务器端口
    #[serde(default = "default_legacy_port")]
    pub legacy_port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    /// 失败率阈值
    #[serde(default = "default_circuit_breaker_failure_threshold")]
    pub failure_threshold: f64,
    /// 请求数量阈值
    #[serde(default = "default_circuit_breaker_request_threshold")]
    pub request_threshold: u32,
    /// 打开超时时间(秒)
    #[serde(default = "default_circuit_breaker_open_timeout")]
    pub open_timeout_seconds: u64,
    /// 半开状态下允许的请求数
    #[serde(default = "default_circuit_breaker_half_open_requests")]
    pub half_open_requests: u32,
    /// 半开状态下成功请求数阈值
    #[serde(default = "default_circuit_breaker_half_open_success_threshold")]
    pub half_open_success_threshold: u32,
    /// 豁免路径
    #[serde(default = "default_circuit_breaker_exempt_paths")]
    pub exempt_paths: Vec<String>,
}

// 默认值函数
fn default_host() -> String {
    "0.0.0.0".to_string()
}
fn default_port() -> u16 {
    8082
}
fn default_db_max_connections() -> u32 {
    20
}
fn default_db_min_connections() -> u32 {
    5
}
fn default_db_connect_timeout() -> u64 {
    30
}
fn default_redis_max_connections() -> u32 {
    10
}
fn default_redis_default_ttl() -> u64 {
    300
}
fn default_jwt_expiration() -> u64 {
    86400
}
fn default_argon2_memory() -> u32 {
    65536
}
fn default_argon2_time() -> u32 {
    3
}
fn default_argon2_parallelism() -> u32 {
    4
}
fn default_cors_origins() -> Vec<String> {
    vec![
        "http://localhost:5173".to_string(),
        "http://127.0.0.1:5173".to_string(),
    ]
}
fn default_enable_prometheus() -> bool {
    true
}
fn default_enable_tracing() -> bool {
    true
}
fn default_slow_query_threshold() -> u64 {
    100
}
fn default_log_level() -> String {
    "info".to_string()
}
fn default_log_format() -> String {
    "json".to_string()
}
fn default_jt808_addr() -> String {
    "0.0.0.0:8988".to_string()
}
fn default_ws_addr() -> String {
    "0.0.0.0:8089".to_string()
}
fn default_max_connections() -> usize {
    10000
}
fn default_jt1078_max_connections() -> usize {
    1000
}
fn default_gb28181_sip_port() -> u16 {
    5060
}
fn default_video_storage() -> String {
    "./data/videos".to_string()
}
fn default_sync_interval() -> u64 {
    300
}
fn default_legacy_host() -> String {
    "127.0.0.1".to_string()
}
fn default_legacy_port() -> u16 {
    9808
}

// 熔断器默认值函数
fn default_circuit_breaker_failure_threshold() -> f64 {
    0.5
}

fn default_circuit_breaker_request_threshold() -> u32 {
    20
}

fn default_circuit_breaker_open_timeout() -> u64 {
    30
}

fn default_circuit_breaker_half_open_requests() -> u32 {
    5
}

fn default_circuit_breaker_half_open_success_threshold() -> u32 {
    3
}

fn default_circuit_breaker_exempt_paths() -> Vec<String> {
    vec![
        "/api/health".to_string(),
        "/api/health/ready".to_string(),
        "/api/health/live".to_string(),
        "/api/metrics".to_string(),
        "/api/auth/login".to_string(),
        "/api/auth/refresh".to_string(),
        "/ws".to_string(),
    ]
}

impl Default for UnifiedConfig {
    fn default() -> Self {
        Self {
            architecture: ArchitectureConfig::default(),
            server: ServerConfig {
                host: default_host(),
                port: default_port(),
                enable_tls: false,
                tls_cert_path: None,
                tls_key_path: None,
            },
            database: DatabaseConfig {
                url: "postgresql://postgres:${DB_PASSWORD}@localhost:5432/carptms_db".to_string(),
                max_connections: default_db_max_connections(),
                min_connections: default_db_min_connections(),
                connect_timeout: default_db_connect_timeout(),
            },
            redis: RedisConfig {
                url: "redis://localhost:6379/0".to_string(),
                max_connections: default_redis_max_connections(),
                default_ttl: default_redis_default_ttl(),
            },
            security: SecurityConfig {
                jwt_secret: "your-secret-key-change-in-production-123456".to_string(),
                jwt_expiration: default_jwt_expiration(),
                argon2_memory: default_argon2_memory(),
                argon2_time: default_argon2_time(),
                argon2_parallelism: default_argon2_parallelism(),
                enable_https: false,
                allowed_origins: default_cors_origins(),
            },
            monitoring: MonitoringConfig {
                enable_prometheus: default_enable_prometheus(),
                enable_tracing: default_enable_tracing(),
                tracing_service_url: None,
                slow_query_threshold: default_slow_query_threshold(),
            },
            logging: LoggingConfig {
                level: default_log_level(),
                format: default_log_format(),
                file_path: None,
            },
            gateway: GatewayConfig {
                jt808_address: default_jt808_addr(),
                websocket_address: default_ws_addr(),
                max_connections: default_max_connections(),
            },
            video: VideoConfig {
                enabled: false,
                jt1078_max_connections: default_jt1078_max_connections(),
                gb28181_sip_port: default_gb28181_sip_port(),
                storage_path: default_video_storage(),
            },
            sync: SyncConfig {
                enabled: true,
                interval_seconds: default_sync_interval(),
                legacy_host: default_legacy_host(),
                legacy_port: default_legacy_port(),
            },
            circuit_breaker: CircuitBreakerConfig {
                failure_threshold: default_circuit_breaker_failure_threshold(),
                request_threshold: default_circuit_breaker_request_threshold(),
                open_timeout_seconds: default_circuit_breaker_open_timeout(),
                half_open_requests: default_circuit_breaker_half_open_requests(),
                half_open_success_threshold: default_circuit_breaker_half_open_success_threshold(),
                exempt_paths: default_circuit_breaker_exempt_paths(),
            },
        }
    }
}

impl UnifiedConfig {
    /// 从文件加载配置
    pub async fn from_file<P: AsRef<Path>>(path: P) -> AppResult<Self> {
        let path = path.as_ref();

        info!("Loading configuration from: {}", path.display());

        let content = fs::read_to_string(path).await.map_err(|e| {
            AppError::internal_error(&format!("Failed to read config file: {}", e), None)
        })?;

        let config: Self = serde_yaml::from_str(&content).map_err(|e| {
            AppError::internal_error(&format!("Failed to parse config file: {}", e), None)
        })?;

        config.validate()?;

        info!("Configuration loaded successfully");
        Ok(config)
    }

    /// 从.env文件加载环境变量
    fn load_env_file() {
        // 尝试加载.env文件
        if let Ok(content) = std::fs::read_to_string(".env") {
            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                if let Some((key, value)) = line.split_once('=') {
                    let key = key.trim();
                    let value = value.trim().trim_matches('"').trim_matches('\'');
                    if std::env::var(key).is_err() {
                        std::env::set_var(key, value);
                    }
                }
            }
        }
    }

    /// 从环境变量覆盖配置
    fn override_from_env(mut self) -> Self {
        if let Ok(host) = env::var("HOST") {
            self.server.host = host;
        }
        if let Ok(port_str) = env::var("PORT") {
            if let Ok(port) = port_str.parse::<u16>() {
                self.server.port = port;
            }
        }
        // Allow DATABASE_URL environment variable to override the database URL
        if let Ok(url) = env::var("DATABASE_URL") {
            self.database.url = url;
        }

        if let Ok(url) = env::var("REDIS_URL") {
            self.redis.url = url;
        }
        if let Ok(secret) = env::var("JWT_SECRET") {
            self.security.jwt_secret = secret;
        }
        if let Ok(level) = env::var("RUST_LOG") {
            self.logging.level = level;
        }

        self
    }

    /// 验证配置
    fn validate(&self) -> AppResult<()> {
        // 验证服务器配置
        if self.server.port == 0 {
            return Err(AppError::validation("Server port cannot be 0"));
        }

        // 验证数据库URL
        if self.database.url.is_empty() {
            return Err(AppError::validation("Database URL cannot be empty"));
        }

        // 验证JWT密钥
        if self.security.jwt_secret.len() < 32 {
            warn!("JWT secret is less than 32 characters, this is insecure");
        }

        // 验证监控配置
        if self.monitoring.slow_query_threshold == 0 {
            return Err(AppError::validation("Slow query threshold cannot be 0"));
        }

        // 验证架构配置
        if let Err(e) = self.architecture.validate() {
            return Err(AppError::validation(&format!("Architecture config validation failed: {}", e)));
        }

        Ok(())
    }

    /// 保存配置到文件
    pub async fn save_to_file<P: AsRef<Path>>(&self, path: P) -> AppResult<()> {
        let path = path.as_ref();

        let content = serde_yaml::to_string(self).map_err(|e| {
            AppError::internal_error(&format!("Failed to serialize config: {}", e), None)
        })?;

        // 创建父目录
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await.map_err(|e| {
                AppError::internal_error(&format!("Failed to create config directory: {}", e), None)
            })?;
        }

        match fs::write(path, content).await {
            Ok(_) => {}
            Err(e) => {
                return Err(AppError::internal_error(
                    &format!("Failed to write config file: {}", e),
                    None,
                ))
            }
        }

        info!("Configuration saved to: {}", path.display());
        Ok(())
    }

    /// 获取数据库连接池配置
    pub fn get_pool_config(&self) -> sqlx::postgres::PgPoolOptions {
        sqlx::postgres::PgPoolOptions::new()
            .max_connections(self.database.max_connections)
            .min_connections(self.database.min_connections)
            .acquire_timeout(std::time::Duration::from_secs(
                self.database.connect_timeout,
            ))
    }

    /// 获取Argon2配置
    pub fn get_argon2_config(&self) -> Params {
        Params::new(
            self.security.argon2_memory,
            self.security.argon2_time,
            self.security.argon2_parallelism,
            None,
        )
        .expect("Invalid Argon2 parameters")
    }

    /// 动态重新加载配置
    pub async fn reload(&mut self) -> AppResult<()> {
        info!("Reloading configuration");

        let new_config = Self::load().await?;
        *self = new_config;

        info!("Configuration reloaded successfully");
        Ok(())
    }

    /// 从环境变量和默认值加载配置(异步版本)
    pub async fn load() -> AppResult<Self> {
        info!("Loading configuration from environment variables");

        // 加载.env文件
        Self::load_env_file();

        let config: Self = Self::default()
            .override_from_env()
            .override_from_config_file()
            .await?;

        config.validate()?;

        info!("Configuration loaded successfully");
        Ok(config)
    }

    /// 从配置文件覆盖配置(异步版本)
    async fn override_from_config_file(self) -> AppResult<Self> {
        if let Ok(config_file) = env::var("CONFIG_FILE") {
            let path = Path::new(&config_file);
            if path.exists() {
                return Self::from_file(path).await;
            }
        }

        // 尝试默认配置文件路径
        let default_paths = [
            "./config.yaml",
            "./config.yml",
            "./settings.yaml",
            "./settings.yml",
        ];

        for path_str in &default_paths {
            let path = Path::new(path_str);
            if path.exists() {
                return Self::from_file(path).await;
            }
        }

        Ok(self)
    }

    /// 从环境变量获取完整配置
    pub fn from_env() -> AppResult<Self> {
        let mut config = Self::default();

        // 服务器配置
        if let Ok(host) = env::var("HOST") {
            config.server.host = host;
        }
        if let Ok(port_str) = env::var("PORT") {
            if let Ok(port) = port_str.parse::<u16>() {
                config.server.port = port;
            }
        }
        if let Ok(enable_tls_str) = env::var("ENABLE_TLS") {
            if let Ok(enable_tls) = enable_tls_str.parse::<bool>() {
                config.server.enable_tls = enable_tls;
            }
        }

        // 数据库配置
        if let Ok(url) = env::var("DATABASE_URL") {
            config.database.url = url;
        }
        if let Ok(max_connections_str) = env::var("DATABASE_MAX_CONNECTIONS") {
            if let Ok(max_connections) = max_connections_str.parse::<u32>() {
                config.database.max_connections = max_connections;
            }
        }
        if let Ok(min_connections_str) = env::var("DATABASE_MIN_CONNECTIONS") {
            if let Ok(min_connections) = min_connections_str.parse::<u32>() {
                config.database.min_connections = min_connections;
            }
        }

        // Redis配置
        if let Ok(url) = env::var("REDIS_URL") {
            config.redis.url = url;
        }

        // 安全配置
        if let Ok(secret) = env::var("JWT_SECRET") {
            config.security.jwt_secret = secret;
        }
        if let Ok(expiration_str) = env::var("JWT_EXPIRATION") {
            if let Ok(expiration) = expiration_str.parse::<u64>() {
                config.security.jwt_expiration = expiration;
            }
        }

        // 日志配置
        if let Ok(level) = env::var("RUST_LOG") {
            config.logging.level = level;
        }

        config.validate()?;
        Ok(config)
    }
}

/// 配置管理模块
pub mod manager {
    use super::*;
    use once_cell::sync::Lazy;
    use std::sync::{
        atomic::{AtomicBool, Ordering},
        RwLock,
    };

    // 静态配置变量,初始值为空配置
    static CONFIG: Lazy<RwLock<UnifiedConfig>> =
        Lazy::new(|| RwLock::new(UnifiedConfig::default()));

    // 初始化标志,确保init_config()只能调用一次
    static INITIALIZED: AtomicBool = AtomicBool::new(false);

    /// 获取当前配置
    pub fn get_config() -> UnifiedConfig {
        CONFIG.read().expect("config should always be readable").clone()
    }

    /// 重新加载配置
    pub async fn reload_config() -> AppResult<()> {
        // 先执行异步操作,再获取锁
        let new_config = UnifiedConfig::load().await?;
        let mut config = CONFIG.write().expect("config should always be writable");
        *config = new_config;
        info!("Configuration reloaded successfully");
        Ok(())
    }

    /// 初始化配置
    pub async fn init_config() -> AppResult<()> {
        // 检查是否已经初始化
        if INITIALIZED
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_err()
        {
            info!("Configuration already initialized");
            return Ok(());
        }

        // 先执行异步操作,再获取锁
        let new_config = UnifiedConfig::load().await?;
        let mut config = CONFIG.write().expect("config should always be writable");
        *config = new_config;
        info!("Configuration initialized successfully");
        Ok(())
    }

    /// 重新加载配置(阻塞版本)
    pub fn reload_config_blocking() -> AppResult<()> {
        // 直接返回 Ok,避免在阻塞上下文中创建运行时
        warn!("reload_config_blocking() is not implemented, use reload_config() instead");
        Ok(())
    }

    /// 初始化配置(阻塞版本)
    pub fn init_config_blocking() -> AppResult<()> {
        // 直接返回 Ok,避免在阻塞上下文中创建运行时
        warn!("init_config_blocking() is not implemented, use init_config() instead");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = UnifiedConfig::default();
        assert_eq!(config.server.host, "0.0.0.0");
        assert_eq!(config.server.port, 8082);
    }

    #[test]
    fn test_config_validation() {
        let mut config = UnifiedConfig::default();
        config.server.port = 0;

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_override_from_env() {
        env::set_var("HOST", "127.0.0.1");
        env::set_var("PORT", "9000");

        let config = UnifiedConfig::default().override_from_env();

        assert_eq!(config.server.host, "127.0.0.1");
        assert_eq!(config.server.port, 9000);

        env::remove_var("HOST");
        env::remove_var("PORT");
    }
}
