use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    Error,
};
use futures::future::LocalBoxFuture;
use log::{error, info, warn};
use std::future::{ready, Ready};
use std::sync::Arc;

use crate::errors::AppError;

// 请求头验证中间件结构体
pub struct RequestHeaderMiddleware {
    require_json: bool,
    allow_multipart: bool,
}

// 中间件工厂实现
impl Default for RequestHeaderMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

impl RequestHeaderMiddleware {
    pub fn new() -> Self {
        Self {
            require_json: true,
            allow_multipart: false,
        }
    }

    // 设置是否需要JSON Content-Type
    pub fn require_json(mut self, require_json: bool) -> Self {
        self.require_json = require_json;
        self
    }

    // 设置是否允许multipart请求
    pub fn allow_multipart(mut self, allow_multipart: bool) -> Self {
        self.allow_multipart = allow_multipart;
        self
    }

    // 快捷方法:允许multipart请求
    pub fn with_multipart() -> Self {
        Self::new().allow_multipart(true)
    }

    // 快捷方法:不要求JSON Content-Type
    pub fn without_json_requirement() -> Self {
        Self::new().require_json(false)
    }
}

// 中间件转换实现
impl<S, B> Transform<S, ServiceRequest> for RequestHeaderMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = RequestHeaderMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RequestHeaderMiddlewareService {
            service: Arc::new(service),
            require_json: self.require_json,
            allow_multipart: self.allow_multipart,
        }))
    }
}

// 中间件服务结构体
pub struct RequestHeaderMiddlewareService<S> {
    service: Arc<S>,
    require_json: bool,
    allow_multipart: bool,
}

// 中间件服务实现
impl<S, B> Service<ServiceRequest> for RequestHeaderMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    // 转发准备状态
    fn poll_ready(
        &self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = Arc::clone(&self.service);
        let require_json = self.require_json;
        let allow_multipart = self.allow_multipart;
        let path = req.path().to_string();
        let method = req.method().to_string();

        Box::pin(async move {
            // 对于登录请求和WebSocket请求,跳过请求头验证
            if path == "/api/auth/login" || path == "/ws" {
                info!("Skipping header validation for login or WebSocket request");
                let res = service.call(req).await?;
                return Ok(res);
            }

            // 验证请求头
            if let Err(err) = validate_headers(&req, require_json, allow_multipart).await {
                error!(
                    "Request header validation failed for path: {}, method: {}, error: {:?}",
                    path, method, err
                );
                return Err(actix_web::error::ErrorBadRequest(err));
            }

            info!(
                "Request header validation passed for path: {}, method: {}",
                path, method
            );

            // 继续处理请求
            let res = service.call(req).await?;
            Ok(res)
        })
    }
}

// 验证请求头
async fn validate_headers(
    req: &ServiceRequest,
    require_json: bool,
    allow_multipart: bool,
) -> Result<(), AppError> {
    let method = req.method();
    let path = req.path();

    // 检查Content-Type
    if require_json && (method == "POST" || method == "PUT" || method == "PATCH") {
        let content_type = req.headers().get("Content-Type");

        if let Some(ct) = content_type {
            let ct_str = ct.to_str().map_err(|e| {
                AppError::validation(&format!("Invalid Content-Type header: {}", e))
            })?;

            // 允许JSON或multipart(如果配置允许)
            if !ct_str.contains("application/json") {
                if allow_multipart && ct_str.contains("multipart/form-data") {
                    // 允许multipart请求
                    info!(
                        "Allowed multipart request for path: {}, method: {}",
                        path, method
                    );
                } else {
                    warn!(
                        "Invalid Content-Type: {} for path: {}, method: {}",
                        ct_str, path, method
                    );
                    return Err(AppError::validation(&format!(
                        "Invalid Content-Type: {}, expected application/json",
                        ct_str
                    )));
                }
            }
        } else {
            warn!(
                "Missing Content-Type header for path: {}, method: {}",
                path, method
            );
            return Err(AppError::validation(
                "Missing Content-Type header, expected application/json",
            ));
        }
    }

    // 检查Accept头(仅对非GET请求强制要求)
    if method != "GET" {
        let accept = req.headers().get("Accept");
        if let Some(accept) = accept {
            let accept_str = accept
                .to_str()
                .map_err(|e| AppError::validation(&format!("Invalid Accept header: {}", e)))?;
            if !accept_str.contains("application/json") && !accept_str.contains("*/*") {
                warn!(
                    "Invalid Accept header: {} for path: {}, method: {}",
                    accept_str, path, method
                );
                return Err(AppError::validation(&format!(
                    "Invalid Accept header: {}, expected application/json",
                    accept_str
                )));
            }
        }
    }

    // 检查User-Agent头
    if req.headers().get("User-Agent").is_none() {
        warn!(
            "Missing User-Agent header for path: {}, method: {}",
            path, method
        );
        // 不强制要求User-Agent,但记录警告
    }

    // 检查X-Request-ID头(可选,但建议使用)
    if req.headers().get("X-Request-ID").is_none() {
        info!(
            "Missing X-Request-ID header for path: {}, method: {}",
            path, method
        );
        // 不强制要求X-Request-ID,但记录信息
    }

    Ok(())
}

// 导出常用的中间件配置
pub fn default_header_middleware() -> RequestHeaderMiddleware {
    RequestHeaderMiddleware::new()
}

pub fn multipart_header_middleware() -> RequestHeaderMiddleware {
    RequestHeaderMiddleware::with_multipart()
}

pub fn relaxed_header_middleware() -> RequestHeaderMiddleware {
    RequestHeaderMiddleware::without_json_requirement()
}

#[cfg(test)]
mod tests {
    use super::*;

    // 测试中间件构造函数
    #[test]
    fn test_request_header_middleware_constructors() {
        // 测试默认构造函数
        let middleware = RequestHeaderMiddleware::new();
        assert!(middleware.require_json);
        assert!(!middleware.allow_multipart);

        // 测试with_multipart方法
        let middleware = RequestHeaderMiddleware::with_multipart();
        assert!(middleware.require_json);
        assert!(middleware.allow_multipart);

        // 测试without_json_requirement方法
        let middleware = RequestHeaderMiddleware::without_json_requirement();
        assert!(!middleware.require_json);
        assert!(!middleware.allow_multipart);

        // 测试链式调用
        let middleware = RequestHeaderMiddleware::new()
            .require_json(false)
            .allow_multipart(true);
        assert!(!middleware.require_json);
        assert!(middleware.allow_multipart);
    }

    // 测试导出的中间件配置函数
    #[test]
    fn test_exported_middleware_functions() {
        // 测试default_header_middleware
        let middleware = default_header_middleware();
        assert!(middleware.require_json);
        assert!(!middleware.allow_multipart);

        // 测试multipart_header_middleware
        let middleware = multipart_header_middleware();
        assert!(middleware.require_json);
        assert!(middleware.allow_multipart);

        // 测试relaxed_header_middleware
        let middleware = relaxed_header_middleware();
        assert!(!middleware.require_json);
        assert!(!middleware.allow_multipart);
    }
}
