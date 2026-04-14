//! / API文档模块

pub mod openapi;

use crate::errors::AppResult;

/// 健康检查响应
#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct HealthCheckResponse {
    pub status: String,
    pub version: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub services: ServiceStatus,
}

/// 服务状态
#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct ServiceStatus {
    pub database: bool,
    pub redis: bool,
    pub websocket: bool,
}

/// 通用成功响应
#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct SuccessResponse<T> {
    pub success: bool,
    pub data: T,
}

/// 通用错误响应
#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct ErrorResponse {
    pub success: bool,
    pub error: String,
    pub error_type: String,
}

/// 分页响应
#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct PaginationResponse<T> {
    pub success: bool,
    pub data: Vec<T>,
    pub pagination: PaginationInfo,
}

/// 分页信息
#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct PaginationInfo {
    pub page: u32,
    pub page_size: u32,
    pub total: u64,
    pub total_pages: u32,
}

// 为健康检查端点添加OpenAPI文档
#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "系统健康", body = HealthCheckResponse)
    ),
    tag = "健康检查"
)]
pub async fn health_check() -> impl actix_web::Responder {
    actix_web::HttpResponse::Ok().json(HealthCheckResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        timestamp: chrono::Utc::now(),
        services: ServiceStatus {
            database: true,
            redis: true,
            websocket: true,
        },
    })
}

// 为指标端点添加OpenAPI文档
#[utoipa::path(
    get,
    path = "/metrics",
    responses(
        (status = 200, description = "Prometheus指标", content_type = "text/plain")
    ),
    tag = "健康检查"
)]
pub async fn metrics_endpoint() -> impl actix_web::Responder {
    use prometheus::Encoder;
    let encoder = prometheus::TextEncoder::new();
    let metric_families = prometheus::default_registry().gather();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer).unwrap();
    
    actix_web::HttpResponse::Ok()
        .content_type("text/plain; version=0.0.4")
        .body(buffer)
}






