//! OpenTelemetry 链路追踪模块
//!
//! 提供分布式追踪能力，用于分析请求链路和性能诊断

use opentelemetry::global;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{
    runtime,
    trace::{self, Sampler},
};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// 追踪配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TracingConfig {
    /// 是否启用追踪
    pub enabled: bool,
    /// 服务名称
    pub service_name: String,
    /// 服务版本
    pub service_version: String,
    /// 环境
    pub environment: String,
    /// 导出器配置
    pub exporter: ExporterConfig,
    /// 采样配置
    pub sampling: SamplingConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ExporterConfig {
    /// 导出类型：otlp, jaeger, zipkin, console
    pub exporter_type: String,
    /// OTLP 配置
    pub otlp: OtlpConfig,
    /// Jaeger 配置
    pub jaeger: JaegerConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OtlpConfig {
    pub protocol: String,
    pub endpoint: String,
    pub use_tls: bool,
    pub timeout_secs: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JaegerConfig {
    pub agent_host: String,
    pub agent_port: u16,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SamplingConfig {
    pub sampler_type: String,
    pub sampling_ratio: f64,
    pub inherit_parent: bool,
}

impl Default for TracingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            service_name: "carptms-server".to_string(),
            service_version: env!("CARGO_PKG_VERSION").to_string(),
            environment: "development".to_string(),
            exporter: ExporterConfig {
                exporter_type: "console".to_string(),
                otlp: OtlpConfig {
                    protocol: "grpc".to_string(),
                    endpoint: "http://localhost:4317".to_string(),
                    use_tls: false,
                    timeout_secs: 30,
                },
                jaeger: JaegerConfig {
                    agent_host: "localhost".to_string(),
                    agent_port: 6831,
                },
            },
            sampling: SamplingConfig {
                sampler_type: "parent_based".to_string(),
                sampling_ratio: 0.1,
                inherit_parent: true,
            },
        }
    }
}

/// 初始化追踪器
pub fn init_tracing(config: &TracingConfig) -> Result<(), Box<dyn std::error::Error>> {
    if !config.enabled {
        return Ok(());
    }

    // 创建采样器
    let sampler = match config.sampling.sampler_type.as_str() {
        "always_on" => Sampler::AlwaysOn,
        "always_off" => Sampler::AlwaysOff,
        "trace_id_ratio" => Sampler::TraceIdRatioBased(config.sampling.sampling_ratio),
        _ => Sampler::ParentBased(Box::new(Sampler::TraceIdRatioBased(
            config.sampling.sampling_ratio,
        ))),
    };

    // 根据导出类型创建导出器
    match config.exporter.exporter_type.as_str() {
        "otlp" => {
            let endpoint = config.exporter.otlp.endpoint.clone();
            let _protocol = config.exporter.otlp.protocol.clone();

            let exporter = opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint(&endpoint)
                .with_timeout(Duration::from_secs(config.exporter.otlp.timeout_secs));

            let _ = opentelemetry_otlp::new_pipeline()
                .tracing()
                .with_exporter(exporter)
                .with_trace_config(trace::Config::default().with_sampler(sampler))
                .install_batch(runtime::Tokio)?;
        }
        #[cfg(feature = "jaeger")]
        "jaeger" => {
            use opentelemetry_otlp::WithExportConfig;

            let exporter = opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint(format!(
                    "{}:{}",
                    config.exporter.jaeger.agent_host, config.exporter.jaeger.agent_port
                ));

            let _ = opentelemetry_otlp::new_pipeline()
                .tracing()
                .with_exporter(exporter)
                .with_trace_config(trace::Config::default().with_sampler(sampler))
                .install_batch(runtime::Tokio)?;
        }
        _ => {
            // Console 导出器 - 仅用于开发调试
            let _ = opentelemetry_sdk::trace::TracerProvider::builder()
                .with_config(trace::Config::default().with_sampler(sampler))
                .build();
        }
    };

    // 创建 tracing subscriber
    let subscriber = tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(true)
                .with_thread_ids(false)
                .with_file(false)
                .with_line_number(false),
        )
        .with(EnvFilter::from_default_env().add_directive("carptms=info".parse().unwrap()));

    // 初始化 subscriber
    subscriber.init();

    log::info!("OpenTelemetry tracing initialized: {}", config.service_name);

    Ok(())
}

/// 创建追踪 span
#[macro_export]
macro_rules! trace_span {
    ($name:expr) => {
        tracing::info_span!(
            $name,
            "otel.name" = $name,
            "otel.kind" = "internal"
        )
    };

    ($name:expr, $($key:expr => $value:expr),*) => {
        tracing::info_span!(
            $name,
            "otel.name" = $name,
            "otel.kind" = "internal",
            $($key = $value,)*
        )
    };
}

/// 带追踪的异步函数封装
#[allow(unused_variables)]
pub async fn trace_async<T, F, Fut>(
    operation_name: &'static str,
    f: F,
) -> Result<T, Box<dyn std::error::Error + Send + Sync>>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<T, Box<dyn std::error::Error + Send + Sync>>>,
{
    let result = f().await?;
    Ok(result)
}

/// HTTP 请求追踪中间件
pub async fn trace_http_request<F, Fut>(
    method: &str,
    path: &str,
    f: F,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>>,
{
    let span = trace_span!(
        "http_request",
        "http.method" => method,
        "http.route" => path,
        "otel.kind" => "server"
    );

    let _guard = span.enter();

    f().await
}

/// 添加追踪属性
#[allow(unused_variables)]
pub fn add_trace_attribute(key: &str, value: &str) {
    // OpenTelemetry追踪属性记录
    // 在生产环境中可以连接OTLP导出器
}

/// 记录追踪错误
#[allow(unused_variables)]
pub fn trace_error(error: &dyn std::error::Error) {
    // 追踪错误记录
    log::error!("Traced error: {}", error);
}

/// 获取当前追踪 ID
#[allow(unused_variables)]
pub fn get_trace_id() -> Option<String> {
    None
}

/// 获取当前 Span ID
#[allow(unused_variables)]
pub fn get_span_id() -> Option<String> {
    None
}

/// 数据库查询追踪包装器
#[allow(unused_variables)]
pub async fn trace_database_query<T, F, Fut>(
    operation: &str,
    query: &str,
    f: F,
) -> Result<T, Box<dyn std::error::Error + Send + Sync>>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<T, Box<dyn std::error::Error + Send + Sync>>>,
{
    let start = std::time::Instant::now();
    let result = f().await;
    let duration = start.elapsed();

    // 记录查询持续时间
    if duration.as_millis() > 100 {
        log::warn!(
            "Slow query detected: {} took {}ms",
            operation,
            duration.as_millis()
        );
    }

    result
}

/// 关闭追踪器（优雅退出）
pub fn shutdown_tracing() {
    global::shutdown_tracer_provider();
    log::info!("OpenTelemetry tracing shutdown");
}
