//! / 日志配置模块
use std::io;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use tokio::sync::RwLock;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Layer};
use uuid::Uuid;

// 日志配置选项
pub struct LogConfig {
    pub log_directory: PathBuf,
    pub rotation: Rotation,
    pub enable_json: bool,
    pub enable_console: bool,
    pub log_level: String,
    pub max_log_files: usize,
    pub enable_sampling: bool,
    pub sampling_ratio: f64,
    pub enable_request_tracing: bool,
    pub enable_structured_logging: bool,
    pub enable_performance_logging: bool,
}

/// 日志上下文管理器
#[derive(Debug, Clone)]
pub struct LogContextManager {
    context: Arc<RwLock<Option<LogContext>>>,
}

/// 日志上下文
#[derive(Debug, Clone)]
pub struct LogContext {
    pub request_id: String,
    pub user_id: Option<String>,
    pub endpoint: Option<String>,
    pub method: Option<String>,
    pub client_ip: Option<String>,
    pub correlation_id: Option<String>,
}

impl LogContext {
    /// 创建新的日志上下文
    pub fn new() -> Self {
        Self {
            request_id: Uuid::new_v4().to_string(),
            user_id: None,
            endpoint: None,
            method: None,
            client_ip: None,
            correlation_id: None,
        }
    }

    /// 从请求ID创建日志上下文
    pub fn with_request_id(request_id: &str) -> Self {
        Self {
            request_id: request_id.to_string(),
            user_id: None,
            endpoint: None,
            method: None,
            client_ip: None,
            correlation_id: None,
        }
    }
}

impl Default for LogContext {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for LogContextManager {
    fn default() -> Self {
        Self {
            context: Arc::new(RwLock::new(None)),
        }
    }
}

impl LogContextManager {
    /// 创建新的日志上下文管理器
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置日志上下文
    pub async fn set_context(&self, context: LogContext) {
        let mut ctx = self.context.write().await;
        *ctx = Some(context);
    }

    /// 获取当前日志上下文
    pub async fn get_context(&self) -> Option<LogContext> {
        let ctx = self.context.read().await;
        ctx.clone()
    }

    /// 清除日志上下文
    pub async fn clear_context(&self) {
        let mut ctx = self.context.write().await;
        *ctx = None;
    }

    /// 创建带上下文的日志记录器
    pub async fn create_span(&self, _name: &str) -> tracing::Span {
        let context = self.get_context().await;
        if let Some(ctx) = context {
            tracing::info_span!(
                "request",
                request_id = ctx.request_id,
                user_id = ctx.user_id.unwrap_or("unknown".to_string()),
                endpoint = ctx.endpoint.unwrap_or("unknown".to_string()),
                method = ctx.method.unwrap_or("unknown".to_string()),
                client_ip = ctx.client_ip.unwrap_or("unknown".to_string()),
                correlation_id = ctx.correlation_id.unwrap_or("unknown".to_string()),
            )
        } else {
            tracing::info_span!("request")
        }
    }
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            log_directory: PathBuf::from("./logs"),
            rotation: Rotation::DAILY,
            enable_json: true, // 默认启用JSON格式,便于结构化日志分析
            enable_console: true,
            log_level: "info".to_string(),     // 默认日志级别
            max_log_files: 30,                 // 默认保留30天的日志文件
            enable_sampling: false,            // 默认禁用采样
            sampling_ratio: 1.0,               // 默认采样率为100%
            enable_request_tracing: true,      // 默认启用请求追踪
            enable_structured_logging: true,   // 默认启用结构化日志
            enable_performance_logging: false, // 默认禁用性能日志
        }
    }
}

/// 初始化日志系统
pub async fn init_logging(config: LogConfig) -> io::Result<()> {
    // 创建日志目录
    fs::create_dir_all(&config.log_directory).await?;

    // 创建滚动日志文件输出
    let rotation = config.rotation;
    let log_directory = config.log_directory;
    let file_appender = RollingFileAppender::new(rotation.clone(), &log_directory, "tms-server");

    // 创建错误日志文件输出
    let error_appender =
        RollingFileAppender::new(rotation.clone(), &log_directory, "tms-server-error");

    // 创建性能日志文件输出(如果启用)
    let performance_appender = if config.enable_performance_logging {
        Some(RollingFileAppender::new(
            rotation.clone(),
            &log_directory,
            "tms-server-performance",
        ))
    } else {
        None
    };

    // 创建基础日志格式
    let file_layer = if config.enable_json {
        tracing_subscriber::fmt::layer()
            .with_writer(file_appender)
            .with_ansi(false)
            .with_timer(tracing_subscriber::fmt::time::UtcTime::rfc_3339())
            .with_level(true)
            .with_target(true)
            .with_thread_ids(true)
            .with_thread_names(true)
            .json()
            .boxed()
    } else {
        tracing_subscriber::fmt::layer()
            .with_writer(file_appender)
            .with_ansi(false)
            .with_timer(tracing_subscriber::fmt::time::UtcTime::rfc_3339())
            .with_level(true)
            .with_target(true)
            .with_thread_ids(true)
            .with_thread_names(true)
            .pretty()
            .boxed()
    };

    // 创建错误日志格式
    let error_layer = if config.enable_json {
        tracing_subscriber::fmt::layer()
            .with_writer(error_appender)
            .with_ansi(false)
            .with_timer(tracing_subscriber::fmt::time::UtcTime::rfc_3339())
            .with_level(true)
            .with_target(true)
            .with_thread_ids(true)
            .with_thread_names(true)
            .json()
            .with_filter(tracing_subscriber::filter::LevelFilter::ERROR)
            .boxed()
    } else {
        tracing_subscriber::fmt::layer()
            .with_writer(error_appender)
            .with_ansi(false)
            .with_timer(tracing_subscriber::fmt::time::UtcTime::rfc_3339())
            .with_level(true)
            .with_target(true)
            .with_thread_ids(true)
            .with_thread_names(true)
            .pretty()
            .with_filter(tracing_subscriber::filter::LevelFilter::ERROR)
            .boxed()
    };

    // 创建性能日志格式(如果启用)
    let performance_layer = performance_appender.map(|appender| {
        tracing_subscriber::fmt::layer()
            .with_writer(appender)
            .with_ansi(false)
            .with_timer(tracing_subscriber::fmt::time::UtcTime::rfc_3339())
            .with_level(true)
            .with_target(true)
            .boxed()
    });

    // 创建控制台输出层
    let console_layer = tracing_subscriber::fmt::layer()
        .with_ansi(true)
        .with_timer(tracing_subscriber::fmt::time::UtcTime::rfc_3339())
        .boxed();

    // 创建环境过滤器
    let env_filter = EnvFilter::new(format!(
        "{},actix_web={},sqlx={}",
        config.log_level,
        if config.log_level == "debug" {
            "debug"
        } else {
            "info"
        },
        if config.log_level == "debug" {
            "debug"
        } else {
            "info"
        }
    ));

    // 创建订阅者
    let subscriber = tracing_subscriber::registry()
        .with(env_filter)
        .with(file_layer)
        .with(error_layer)
        .with(performance_layer.unwrap_or_else(|| tracing_subscriber::fmt::layer().boxed()))
        .with(if config.enable_console {
            console_layer
        } else {
            tracing_subscriber::fmt::layer().boxed()
        });

    // 初始化订阅者
    subscriber.try_init().map_err(io::Error::other)?;

    // 记录初始化信息
    tracing::info!("Logging initialized successfully");
    tracing::info!("Log directory: {:?}", log_directory);
    tracing::info!("Rotation: {:?}", rotation);
    tracing::info!("JSON format: {:?}", config.enable_json);
    tracing::info!("Console output: {:?}", config.enable_console);
    tracing::info!("Log level: {:?}", config.log_level);
    tracing::info!("Request tracing: {:?}", config.enable_request_tracing);
    tracing::info!("Structured logging: {:?}", config.enable_structured_logging);
    tracing::info!(
        "Performance logging: {:?}",
        config.enable_performance_logging
    );

    Ok(())
}

/// 从环境变量加载日志配置
pub fn load_log_config_from_env() -> LogConfig {
    let mut config = LogConfig::default();

    // 从环境变量加载日志目录
    if let Ok(dir) = std::env::var("LOG_DIRECTORY") {
        config.log_directory = PathBuf::from(dir);
    }

    // 从环境变量加载日志轮转策略
    if let Ok(rotation) = std::env::var("LOG_ROTATION") {
        config.rotation = match rotation.to_lowercase().as_str() {
            "hourly" => Rotation::HOURLY,
            "daily" => Rotation::DAILY,
            "never" => Rotation::NEVER,
            _ => Rotation::DAILY,
        };
    }

    // 从环境变量加载是否启用JSON格式
    if let Ok(enable_json) = std::env::var("LOG_JSON") {
        config.enable_json = enable_json.to_lowercase() == "true";
    }

    // 从环境变量加载是否启用控制台输出
    if let Ok(enable_console) = std::env::var("LOG_CONSOLE") {
        config.enable_console = enable_console.to_lowercase() == "true";
    }

    // 从环境变量加载日志级别
    if let Ok(log_level) = std::env::var("LOG_LEVEL") {
        config.log_level = log_level;
    }

    // 从环境变量加载最大日志文件数
    if let Ok(max_files) = std::env::var("LOG_MAX_FILES") {
        if let Ok(max_files) = max_files.parse::<usize>() {
            config.max_log_files = max_files;
        }
    }

    // 从环境变量加载是否启用采样
    if let Ok(enable_sampling) = std::env::var("LOG_SAMPLING_ENABLED") {
        config.enable_sampling = enable_sampling.to_lowercase() == "true";
    }

    // 从环境变量加载采样率
    if let Ok(sampling_ratio) = std::env::var("LOG_SAMPLING_RATIO") {
        if let Ok(sampling_ratio) = sampling_ratio.parse::<f64>() {
            config.sampling_ratio = sampling_ratio;
        }
    }

    // 从环境变量加载请求追踪设置
    if let Ok(enable_tracing) = std::env::var("LOG_ENABLE_REQUEST_TRACING") {
        config.enable_request_tracing = enable_tracing.to_lowercase() == "true";
    }

    // 从环境变量加载结构化日志设置
    if let Ok(enable_structured) = std::env::var("LOG_ENABLE_STRUCTURED_LOGGING") {
        config.enable_structured_logging = enable_structured.to_lowercase() == "true";
    }

    // 从环境变量加载性能日志设置
    if let Ok(enable_performance) = std::env::var("LOG_ENABLE_PERFORMANCE_LOGGING") {
        config.enable_performance_logging = enable_performance.to_lowercase() == "true";
    }

    config
}

/// 便捷的日志记录函数
pub mod logging {

    /// 记录信息级日志
    #[macro_export]
    macro_rules! info {
        ($($arg:tt)*) => {
            tracing::info!($($arg)*);
        };
    }

    /// 记录调试级日志
    #[macro_export]
    macro_rules! debug {
        ($($arg:tt)*) => {
            tracing::debug!($($arg)*);
        };
    }

    /// 记录警告级日志
    #[macro_export]
    macro_rules! warn {
        ($($arg:tt)*) => {
            tracing::warn!($($arg)*);
        };
    }

    /// 记录错误级日志
    #[macro_export]
    macro_rules! error {
        ($($arg:tt)*) => {
            tracing::error!($($arg)*);
        };
    }

    /// 记录跟踪级日志
    #[macro_export]
    macro_rules! trace {
        ($($arg:tt)*) => {
            tracing::trace!($($arg)*);
        };
    }

    /// 记录性能日志
    pub fn performance(message: &str, duration_ms: u64, operation: &str) {
        tracing::info!(
            message,
            duration_ms = duration_ms,
            operation = operation,
            log_type = "performance"
        );
    }

    /// 记录请求日志
    pub fn request(method: &str, path: &str, status: u16, duration_ms: u64, client_ip: &str) {
        tracing::info!(
            "Request completed: {} {} {} {}ms {} {}",
            method,
            path,
            status,
            duration_ms,
            client_ip,
            "request"
        );
    }

    /// 记录数据库操作日志
    pub fn database_operation(operation: &str, table: &str, duration_ms: u64, success: bool) {
        tracing::info!(
            "Database operation: {} {} {}ms {} {}",
            operation,
            table,
            duration_ms,
            success,
            "database"
        );
    }

    /// 记录缓存操作日志
    pub fn cache_operation(operation: &str, key: &str, duration_ms: u64, success: bool) {
        tracing::info!(
            "Cache operation: {} {} {}ms {} {}",
            operation,
            key,
            duration_ms,
            success,
            "cache"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    // 测试日志配置默认值
    #[test]
    fn test_log_config_default() {
        let config = LogConfig::default();
        assert_eq!(config.log_directory, PathBuf::from("./logs"));
        assert_eq!(config.rotation, Rotation::DAILY);
        assert!(config.enable_json); // 默认启用JSON格式
        assert!(config.enable_console);
        assert_eq!(config.log_level, "info");
        assert_eq!(config.max_log_files, 30);
        assert!(!config.enable_sampling);
        assert_eq!(config.sampling_ratio, 1.0);
    }

    // 测试从环境变量加载日志配置
    #[test]
    fn test_load_log_config_from_env() {
        // 设置环境变量
        unsafe {
            env::set_var("LOG_DIRECTORY", "/tmp/test_logs");
            env::set_var("LOG_ROTATION", "hourly");
            env::set_var("LOG_JSON", "true");
            env::set_var("LOG_CONSOLE", "false");
        }

        let config = load_log_config_from_env();

        assert_eq!(config.log_directory, PathBuf::from("/tmp/test_logs"));
        assert_eq!(config.rotation, Rotation::HOURLY);
        assert!(config.enable_json);
        assert!(!config.enable_console);

        // 清理环境变量
        unsafe {
            env::remove_var("LOG_DIRECTORY");
            env::remove_var("LOG_ROTATION");
            env::remove_var("LOG_JSON");
            env::remove_var("LOG_CONSOLE");
        }
    }

    // 测试日志初始化
    #[tokio::test]
    async fn test_init_logging() {
        let config = LogConfig {
            log_directory: PathBuf::from("./test_logs"),
            rotation: Rotation::DAILY,
            enable_json: false,
            enable_console: true,
            log_level: "info".to_string(),
            max_log_files: 30,
            enable_sampling: false,
            sampling_ratio: 1.0,
            enable_performance_logging: false,
            enable_request_tracing: false,
            enable_structured_logging: false,
        };

        // 执行测试:初始化日志系统
        let result = init_logging(config).await;

        // 验证:日志系统应该成功初始化
        assert!(result.is_ok());

        // 清理测试日志目录
        std::fs::remove_dir_all("./test_logs").unwrap_or(());
    }
}
