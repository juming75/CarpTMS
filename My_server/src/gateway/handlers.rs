//! 处理器封装模块
//!
//! 提供统一的请求处理器接口，支持：
//! - 本地处理：直接调用 application 层服务
//! - 远程处理：通过 gRPC 调用微服务
//! - 响应转换：统一响应格式

use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Duration;
use tokio::time::timeout;

use crate::config::ArchitectureMode;
use crate::errors::{AppError, AppResult};

/// 请求上下文
#[derive(Debug, Clone)]
pub struct RequestContext {
    /// 请求ID
    pub request_id: String,
    /// 用户ID（如果已认证）
    pub user_id: Option<i32>,
    /// 组织ID
    pub organization_id: Option<i32>,
    /// 架构模式
    pub mode: ArchitectureMode,
    /// 额外的元数据
    pub metadata: std::collections::HashMap<String, String>,
}

impl RequestContext {
    /// 创建新的请求上下文
    pub fn new(request_id: String, mode: ArchitectureMode) -> Self {
        Self {
            request_id,
            user_id: None,
            organization_id: None,
            mode,
            metadata: std::collections::HashMap::new(),
        }
    }

    /// 设置用户ID
    pub fn with_user_id(mut self, user_id: i32) -> Self {
        self.user_id = Some(user_id);
        self
    }

    /// 设置组织ID
    pub fn with_organization_id(mut self, org_id: i32) -> Self {
        self.organization_id = Some(org_id);
        self
    }

    /// 添加元数据
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

/// 统一响应格式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T: Serialize> {
    /// 是否成功
    pub success: bool,
    /// 响应数据
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    /// 错误信息
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ApiError>,
    /// 请求ID
    pub request_id: String,
    /// 时间戳
    pub timestamp: i64,
}

/// API 错误
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError {
    /// 错误码
    pub code: String,
    /// 错误消息
    pub message: String,
    /// 详细信息
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<Value>,
}

impl<T: Serialize> ApiResponse<T> {
    /// 创建成功响应
    pub fn success(data: T, request_id: String) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            request_id,
            timestamp: chrono::Utc::now().timestamp(),
        }
    }

    /// 创建成功响应（无数据）
    pub fn success_empty(request_id: String) -> Self
    where
        T: Default,
    {
        Self {
            success: true,
            data: Some(T::default()),
            error: None,
            request_id,
            timestamp: chrono::Utc::now().timestamp(),
        }
    }

    /// 创建错误响应
    pub fn error(code: &str, message: &str, request_id: String) -> Self
    where
        T: Default,
    {
        Self {
            success: false,
            data: Some(T::default()),
            error: Some(ApiError {
                code: code.to_string(),
                message: message.to_string(),
                details: None,
            }),
            request_id,
            timestamp: chrono::Utc::now().timestamp(),
        }
    }

    /// 创建带详细信息的错误响应
    pub fn error_with_details(code: &str, message: &str, details: Value, request_id: String) -> Self
    where
        T: Default,
    {
        Self {
            success: false,
            data: Some(T::default()),
            error: Some(ApiError {
                code: code.to_string(),
                message: message.to_string(),
                details: Some(details),
            }),
            request_id,
            timestamp: chrono::Utc::now().timestamp(),
        }
    }

    /// 转换为 HTTP 响应
    pub fn to_http_response(&self, status: actix_web::http::StatusCode) -> HttpResponse {
        HttpResponse::build(status).json(self)
    }
}

/// 处理器 trait - 定义统一的请求处理接口
#[async_trait::async_trait]
pub trait RequestHandler: Send + Sync {
    /// 处理 GET 请求
    async fn handle_get(
        &self,
        ctx: &RequestContext,
        path: &str,
        query: web::Query<Value>,
    ) -> AppResult<HttpResponse>;

    /// 处理 POST 请求
    async fn handle_post(
        &self,
        ctx: &RequestContext,
        path: &str,
        body: web::Json<Value>,
    ) -> AppResult<HttpResponse>;

    /// 处理 PUT 请求
    async fn handle_put(
        &self,
        ctx: &RequestContext,
        path: &str,
        body: web::Json<Value>,
    ) -> AppResult<HttpResponse>;

    /// 处理 DELETE 请求
    async fn handle_delete(&self, ctx: &RequestContext, path: &str) -> AppResult<HttpResponse>;

    /// 处理 PATCH 请求
    async fn handle_patch(
        &self,
        ctx: &RequestContext,
        path: &str,
        body: web::Json<Value>,
    ) -> AppResult<HttpResponse>;
}

/// 处理器工厂
pub struct HandlerFactory {
    /// 数据库连接池
    pool: sqlx::PgPool,
    /// 架构模式
    mode: ArchitectureMode,
}

impl HandlerFactory {
    /// 创建新的处理器工厂
    pub fn new(pool: sqlx::PgPool, mode: ArchitectureMode) -> Self {
        Self { pool, mode }
    }

    /// 创建车辆处理器
    pub fn create_vehicle_handler(&self) -> Box<dyn RequestHandler> {
        // 根据架构模式选择处理器
        match self.mode {
            ArchitectureMode::MonolithDDD => {
                Box::new(MonolithVehicleHandler::new(self.pool.clone()))
            }
            ArchitectureMode::MicroDDD => Box::new(MicroserviceVehicleHandler::new()),
        }
    }

    /// 创建订单处理器
    pub fn create_order_handler(&self) -> Box<dyn RequestHandler> {
        match self.mode {
            ArchitectureMode::MonolithDDD => Box::new(MonolithOrderHandler::new(self.pool.clone())),
            ArchitectureMode::MicroDDD => Box::new(MicroserviceOrderHandler::new()),
        }
    }
}

// ============= 单体模式处理器 =============

/// 单体模式 - 车辆处理器
#[allow(dead_code)]
pub struct MonolithVehicleHandler {
    pool: sqlx::PgPool,
}

impl MonolithVehicleHandler {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl RequestHandler for MonolithVehicleHandler {
    async fn handle_get(
        &self,
        ctx: &RequestContext,
        path: &str,
        query: web::Query<Value>,
    ) -> AppResult<HttpResponse> {
        // 解析路径，提取资源ID
        let parts: Vec<&str> = path.trim_start_matches('/').split('/').collect();

        match parts.len() {
            2 => {
                // GET /api/vehicles - 列表
                let page = query.get("page").and_then(|v| v.as_i64()).unwrap_or(1) as i32;
                let page_size = query
                    .get("page_size")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(20) as i32;

                // 这里应该调用 application 层服务
                let response = ApiResponse::success(
                    serde_json::json!({
                        "items": [],
                        "total": 0,
                        "page": page,
                        "page_size": page_size
                    }),
                    ctx.request_id.clone(),
                );

                Ok(response.to_http_response(actix_web::http::StatusCode::OK))
            }
            3 => {
                // GET /api/vehicles/{id} - 详情
                let id: i32 = parts[2]
                    .parse()
                    .map_err(|_| AppError::validation_error("Invalid vehicle ID", None))?;

                // 调用 application 层查询服务
                let response = ApiResponse::success(
                    serde_json::json!({
                        "vehicle_id": id,
                        "message": "Vehicle details would be fetched from database"
                    }),
                    ctx.request_id.clone(),
                );

                Ok(response.to_http_response(actix_web::http::StatusCode::OK))
            }
            _ => {
                let response = ApiResponse::<()>::error(
                    "NOT_FOUND",
                    "Resource not found",
                    ctx.request_id.clone(),
                );
                Ok(response.to_http_response(actix_web::http::StatusCode::NOT_FOUND))
            }
        }
    }

    async fn handle_post(
        &self,
        ctx: &RequestContext,
        _path: &str,
        body: web::Json<Value>,
    ) -> AppResult<HttpResponse> {
        // POST /api/vehicles - 创建
        let response = ApiResponse::success(
            serde_json::json!({
                "message": "Vehicle creation processed",
                "data": body.0
            }),
            ctx.request_id.clone(),
        );

        Ok(response.to_http_response(actix_web::http::StatusCode::CREATED))
    }

    async fn handle_put(
        &self,
        ctx: &RequestContext,
        path: &str,
        body: web::Json<Value>,
    ) -> AppResult<HttpResponse> {
        let parts: Vec<&str> = path.trim_start_matches('/').split('/').collect();

        if parts.len() < 3 {
            let response = ApiResponse::<()>::error(
                "BAD_REQUEST",
                "Vehicle ID is required",
                ctx.request_id.clone(),
            );
            return Ok(response.to_http_response(actix_web::http::StatusCode::BAD_REQUEST));
        }

        let response = ApiResponse::success(
            serde_json::json!({
                "message": "Vehicle update processed",
                "data": body.0
            }),
            ctx.request_id.clone(),
        );

        Ok(response.to_http_response(actix_web::http::StatusCode::OK))
    }

    async fn handle_delete(&self, ctx: &RequestContext, path: &str) -> AppResult<HttpResponse> {
        let parts: Vec<&str> = path.trim_start_matches('/').split('/').collect();

        if parts.len() < 3 {
            let response = ApiResponse::<()>::error(
                "BAD_REQUEST",
                "Vehicle ID is required",
                ctx.request_id.clone(),
            );
            return Ok(response.to_http_response(actix_web::http::StatusCode::BAD_REQUEST));
        }

        let response = ApiResponse::<()>::success_empty(ctx.request_id.clone());
        Ok(response.to_http_response(actix_web::http::StatusCode::NO_CONTENT))
    }

    async fn handle_patch(
        &self,
        ctx: &RequestContext,
        _path: &str,
        body: web::Json<Value>,
    ) -> AppResult<HttpResponse> {
        let response = ApiResponse::success(
            serde_json::json!({
                "message": "Vehicle patch processed",
                "data": body.0
            }),
            ctx.request_id.clone(),
        );

        Ok(response.to_http_response(actix_web::http::StatusCode::OK))
    }
}

/// 单体模式 - 订单处理器
#[allow(dead_code)]
pub struct MonolithOrderHandler {
    pool: sqlx::PgPool,
}

impl MonolithOrderHandler {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl RequestHandler for MonolithOrderHandler {
    async fn handle_get(
        &self,
        ctx: &RequestContext,
        _path: &str,
        _query: web::Query<Value>,
    ) -> AppResult<HttpResponse> {
        let response = ApiResponse::success(
            serde_json::json!({
                "message": "Order list from monolith handler"
            }),
            ctx.request_id.clone(),
        );

        Ok(response.to_http_response(actix_web::http::StatusCode::OK))
    }

    async fn handle_post(
        &self,
        ctx: &RequestContext,
        _path: &str,
        _body: web::Json<Value>,
    ) -> AppResult<HttpResponse> {
        let response = ApiResponse::success(
            serde_json::json!({
                "message": "Order created from monolith handler"
            }),
            ctx.request_id.clone(),
        );

        Ok(response.to_http_response(actix_web::http::StatusCode::CREATED))
    }

    async fn handle_put(
        &self,
        ctx: &RequestContext,
        _path: &str,
        _body: web::Json<Value>,
    ) -> AppResult<HttpResponse> {
        let response = ApiResponse::success(
            serde_json::json!({
                "message": "Order updated from monolith handler"
            }),
            ctx.request_id.clone(),
        );

        Ok(response.to_http_response(actix_web::http::StatusCode::OK))
    }

    async fn handle_delete(&self, ctx: &RequestContext, _path: &str) -> AppResult<HttpResponse> {
        let response = ApiResponse::<()>::success_empty(ctx.request_id.clone());
        Ok(response.to_http_response(actix_web::http::StatusCode::NO_CONTENT))
    }

    async fn handle_patch(
        &self,
        ctx: &RequestContext,
        _path: &str,
        _body: web::Json<Value>,
    ) -> AppResult<HttpResponse> {
        let response = ApiResponse::success(
            serde_json::json!({
                "message": "Order patched from monolith handler"
            }),
            ctx.request_id.clone(),
        );

        Ok(response.to_http_response(actix_web::http::StatusCode::OK))
    }
}

// ============= 微服务模式处理器 =============

/// 微服务模式 - 车辆处理器（通过 gRPC 调用）
pub struct MicroserviceVehicleHandler {
    // gRPC 客户端应该在初始化时注入
}

impl Default for MicroserviceVehicleHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl MicroserviceVehicleHandler {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait::async_trait]
impl RequestHandler for MicroserviceVehicleHandler {
    async fn handle_get(
        &self,
        ctx: &RequestContext,
        _path: &str,
        _query: web::Query<Value>,
    ) -> AppResult<HttpResponse> {
        // 通过 gRPC 调用 vehicle-service
        let response = ApiResponse::success(
            serde_json::json!({
                "message": "Vehicle data from microservice via gRPC",
                "service": "vehicle-service"
            }),
            ctx.request_id.clone(),
        );

        Ok(response.to_http_response(actix_web::http::StatusCode::OK))
    }

    async fn handle_post(
        &self,
        ctx: &RequestContext,
        _path: &str,
        _body: web::Json<Value>,
    ) -> AppResult<HttpResponse> {
        let response = ApiResponse::success(
            serde_json::json!({
                "message": "Vehicle created via gRPC",
                "service": "vehicle-service"
            }),
            ctx.request_id.clone(),
        );

        Ok(response.to_http_response(actix_web::http::StatusCode::CREATED))
    }

    async fn handle_put(
        &self,
        ctx: &RequestContext,
        _path: &str,
        _body: web::Json<Value>,
    ) -> AppResult<HttpResponse> {
        let response = ApiResponse::success(
            serde_json::json!({
                "message": "Vehicle updated via gRPC",
                "service": "vehicle-service"
            }),
            ctx.request_id.clone(),
        );

        Ok(response.to_http_response(actix_web::http::StatusCode::OK))
    }

    async fn handle_delete(&self, ctx: &RequestContext, _path: &str) -> AppResult<HttpResponse> {
        let response = ApiResponse::<()>::success_empty(ctx.request_id.clone());
        Ok(response.to_http_response(actix_web::http::StatusCode::NO_CONTENT))
    }

    async fn handle_patch(
        &self,
        ctx: &RequestContext,
        _path: &str,
        _body: web::Json<Value>,
    ) -> AppResult<HttpResponse> {
        let response = ApiResponse::success(
            serde_json::json!({
                "message": "Vehicle patched via gRPC",
                "service": "vehicle-service"
            }),
            ctx.request_id.clone(),
        );

        Ok(response.to_http_response(actix_web::http::StatusCode::OK))
    }
}

/// 微服务模式 - 订单处理器（通过 gRPC 调用）
pub struct MicroserviceOrderHandler {}

impl Default for MicroserviceOrderHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl MicroserviceOrderHandler {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait::async_trait]
impl RequestHandler for MicroserviceOrderHandler {
    async fn handle_get(
        &self,
        ctx: &RequestContext,
        _path: &str,
        _query: web::Query<Value>,
    ) -> AppResult<HttpResponse> {
        let response = ApiResponse::success(
            serde_json::json!({
                "message": "Order data from microservice via gRPC",
                "service": "cargo-service"
            }),
            ctx.request_id.clone(),
        );

        Ok(response.to_http_response(actix_web::http::StatusCode::OK))
    }

    async fn handle_post(
        &self,
        ctx: &RequestContext,
        _path: &str,
        _body: web::Json<Value>,
    ) -> AppResult<HttpResponse> {
        let response = ApiResponse::success(
            serde_json::json!({
                "message": "Order created via gRPC",
                "service": "cargo-service"
            }),
            ctx.request_id.clone(),
        );

        Ok(response.to_http_response(actix_web::http::StatusCode::CREATED))
    }

    async fn handle_put(
        &self,
        ctx: &RequestContext,
        _path: &str,
        _body: web::Json<Value>,
    ) -> AppResult<HttpResponse> {
        let response = ApiResponse::success(
            serde_json::json!({
                "message": "Order updated via gRPC",
                "service": "cargo-service"
            }),
            ctx.request_id.clone(),
        );

        Ok(response.to_http_response(actix_web::http::StatusCode::OK))
    }

    async fn handle_delete(&self, ctx: &RequestContext, _path: &str) -> AppResult<HttpResponse> {
        let response = ApiResponse::<()>::success_empty(ctx.request_id.clone());
        Ok(response.to_http_response(actix_web::http::StatusCode::NO_CONTENT))
    }

    async fn handle_patch(
        &self,
        ctx: &RequestContext,
        _path: &str,
        _body: web::Json<Value>,
    ) -> AppResult<HttpResponse> {
        let response = ApiResponse::success(
            serde_json::json!({
                "message": "Order patched via gRPC",
                "service": "cargo-service"
            }),
            ctx.request_id.clone(),
        );

        Ok(response.to_http_response(actix_web::http::StatusCode::OK))
    }
}

/// 带超时的处理器调用
pub async fn with_timeout<F, T>(duration: Duration, future: F) -> AppResult<T>
where
    F: std::future::Future<Output = AppResult<T>>,
{
    match timeout(duration, future).await {
        Ok(result) => result,
        Err(_) => Err(AppError::internal_error("Request timeout", None)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_response_success() {
        let response = ApiResponse::success(serde_json::json!({ "id": 1 }), "req-123".to_string());

        assert!(response.success);
        assert!(response.data.is_some());
        assert!(response.error.is_none());
    }

    #[test]
    fn test_api_response_error() {
        let response =
            ApiResponse::<()>::error("NOT_FOUND", "Resource not found", "req-123".to_string());

        assert!(!response.success);
        assert!(response.data.is_none());
        assert!(response.error.is_some());
    }

    #[test]
    fn test_request_context() {
        let ctx = RequestContext::new("req-123".to_string(), ArchitectureMode::MonolithDDD)
            .with_user_id(1)
            .with_organization_id(100);

        assert_eq!(ctx.request_id, "req-123");
        assert_eq!(ctx.user_id, Some(1));
        assert_eq!(ctx.organization_id, Some(100));
    }
}
