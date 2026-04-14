//! / 健康检查模块
// 提供服务健康检查端点,复用已有 PgPool 连接池,
// 避免每次 /health 调用都创建新连接。

pub mod handlers;

// 导出 handler 函数,供 main.rs 和 OpenAPI 宏使用
pub use handlers::health_check;
pub use handlers::liveness_check;
pub use handlers::metrics_endpoint;
pub use handlers::readiness_check;

// re-export __path_* so utoipa OpenAPI macro can resolve the paths
pub use handlers::__path_health_check;
pub use handlers::__path_liveness_check;
pub use handlers::__path_metrics_endpoint;
pub use handlers::__path_readiness_check;
