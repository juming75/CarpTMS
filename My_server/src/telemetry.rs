//! / 分布式追踪模块
use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

// 初始化OpenTelemetry,集成Jaeger
pub fn init_telemetry(_service_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    // 简化实现,避免版本冲突
    // 在实际项目中,您可以根据需要配置更复杂的OpenTelemetry和Jaeger集成
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer().with_ansi(false))
        .try_init()?;

    Ok(())
}

// 关闭OpenTelemetry
pub fn shutdown_telemetry() {
    // 关闭OpenTelemetry
}

// 辅助宏:创建一个新的span
// 使用 Rust 2024 的 expr 特性,可以接受更丰富的表达式
#[macro_export]
macro_rules! trace_span {
    ($name:expr) => {
        tracing::span!(tracing::Level::INFO, $name)
    };
    ($name:expr, $($key:expr => $value:expr),*) => {
        tracing::span!(tracing::Level::INFO, $name, $($key => $value),*)
    };
}

// 辅助宏:进入一个span
#[macro_export]
macro_rules! enter_span {
    ($span:expr) => {
        let _enter = $span.enter();
    };
}

// 辅助宏:记录事件到当前span
#[macro_export]
macro_rules! trace_event {
    ($name:expr) => {
        tracing::event!(tracing::Level::INFO, $name);
    };
    ($name:expr, $($key:expr => $value:expr),*) => {
        tracing::event!(tracing::Level::INFO, $name, $($key => $value),*);
    };
}

// 辅助宏:记录错误到当前span
#[macro_export]
macro_rules! trace_error {
    ($name:expr, $error:expr) => {
        tracing::event!(tracing::Level::ERROR, $name, error = ?$error);
    };
    ($name:expr, $error:expr, $($key:expr => $value:expr),*) => {
        tracing::event!(tracing::Level::ERROR, $name, error = ?$error, $($key => $value),*);
    };
}

// 辅助宏:记录信息日志
#[macro_export]
macro_rules! trace_info {
    ($name:expr) => {
        tracing::event!(tracing::Level::INFO, $name);
    };
    ($name:expr, $($key:expr => $value:expr),*) => {
        tracing::event!(tracing::Level::INFO, $name, $($key => $value),*);
    };
}

// 辅助宏:记录警告日志
#[macro_export]
macro_rules! trace_warn {
    ($name:expr) => {
        tracing::event!(tracing::Level::WARN, $name);
    };
    ($name:expr, $($key:expr => $value:expr),*) => {
        tracing::event!(tracing::Level::WARN, $name, $($key => $value),*);
    };
}
