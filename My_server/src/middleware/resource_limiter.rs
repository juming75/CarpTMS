//! /! 资源限制中间件
//!
//! 提供请求资源限制功能，包括：
//! - 请求体大小限制
//! - 并发连接数限制
//! - 请求超时处理

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    http::header,
    Error,
};
use futures::future::{ready, Future, Ready};
use std::collections::HashSet;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::time::Duration;

// 引入自定义错误类型
use crate::errors::AppError;

/// 资源限制配置
#[derive(Debug, Clone)]
pub struct ResourceLimiterConfig {
    /// 最大请求体大小(字节)
    pub max_body_size: usize,
    /// 最大并发连接数
    pub max_concurrent_connections: usize,
    /// 请求超时时间
    pub request_timeout: Duration,
    /// 允许的最大请求头大小
    pub max_header_size: usize,
    /// 允许的最大查询参数大小
    pub max_query_size: usize,
}

impl Default for ResourceLimiterConfig {
    fn default() -> Self {
        Self {
            max_body_size: 10 * 1024 * 1024, // 10MB
            max_concurrent_connections: 1000,
            request_timeout: Duration::from_secs(30),
            max_header_size: 8192, // 8KB
            max_query_size: 4096,  // 4KB
        }
    }
}

/// 资源限制中间件
#[derive(Debug, Clone)]
pub struct ResourceLimiterMiddleware {
    config: ResourceLimiterConfig,
    concurrent_connections: Arc<Mutex<HashSet<SocketAddr>>>,
}

impl ResourceLimiterMiddleware {
    pub fn new(config: ResourceLimiterConfig) -> Self {
        Self {
            config,
            concurrent_connections: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    pub fn with_max_body_size(max_body_size: usize) -> Self {
        Self {
            config: ResourceLimiterConfig {
                max_body_size,
                ..Default::default()
            },
            concurrent_connections: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    pub fn with_max_concurrent_connections(max_concurrent_connections: usize) -> Self {
        Self {
            config: ResourceLimiterConfig {
                max_concurrent_connections,
                ..Default::default()
            },
            concurrent_connections: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    pub fn with_request_timeout(request_timeout: Duration) -> Self {
        Self {
            config: ResourceLimiterConfig {
                request_timeout,
                ..Default::default()
            },
            concurrent_connections: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    /// 检查请求体大小
    fn check_body_size(&self, req: &actix_web::HttpRequest) -> Result<(), AppError> {
        if let Some(content_length) = req.headers().get(header::CONTENT_LENGTH) {
            if let Ok(length_str) = content_length.to_str() {
                if let Ok(length) = length_str.parse::<usize>() {
                    if length > self.config.max_body_size {
                        return Err(AppError::bad_request(&format!(
                            "Request body too large. Maximum size is {} bytes",
                            self.config.max_body_size
                        )));
                    }
                }
            }
        }
        Ok(())
    }

    /// 检查请求头大小
    fn check_header_size(&self, req: &actix_web::HttpRequest) -> Result<(), AppError> {
        let headers = req.headers();
        let mut total_size = 0;
        for (name, value) in headers {
            total_size += name.as_str().len() + value.as_bytes().len() + 2; // +2 for colon and space
        }
        if total_size > self.config.max_header_size {
            return Err(AppError::bad_request(&format!(
                "Request headers too large. Maximum size is {} bytes",
                self.config.max_header_size
            )));
        }
        Ok(())
    }

    /// 检查查询参数大小
    fn check_query_size(&self, req: &actix_web::HttpRequest) -> Result<(), AppError> {
        if let Some(query) = req.uri().query() {
            if query.len() > self.config.max_query_size {
                return Err(AppError::bad_request(&format!(
                    "Query parameters too large. Maximum size is {} bytes",
                    self.config.max_query_size
                )));
            }
        }
        Ok(())
    }

    /// 检查并发连接数
    fn check_concurrent_connections(&self, req: &actix_web::HttpRequest) -> Result<(), AppError> {
        if let Some(addr) = req.connection_info().realip_remote_addr() {
            if let Ok(socket_addr) = addr.parse::<SocketAddr>() {
                if let Ok(mut connections) = self.concurrent_connections.lock() {
                    if connections.contains(&socket_addr) {
                        // 已经有一个连接来自这个地址
                    }
                    if connections.len() >= self.config.max_concurrent_connections {
                        return Err(AppError::service_unavailable_error(
                            "Too many concurrent connections",
                            None,
                        ));
                    }
                    connections.insert(socket_addr);
                }
            }
        }
        Ok(())
    }

    /// 移除并发连接
    fn remove_connection(&self, req: &actix_web::HttpRequest) {
        if let Some(addr) = req.connection_info().realip_remote_addr() {
            if let Ok(socket_addr) = addr.parse::<SocketAddr>() {
                if let Ok(mut connections) = self.concurrent_connections.lock() {
                    connections.remove(&socket_addr);
                }
            }
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for ResourceLimiterMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = ResourceLimiterMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(ResourceLimiterMiddlewareService {
            service: std::sync::Arc::new(service),
            middleware: self.clone(),
        }))
    }
}

#[derive(Debug)]
pub struct ResourceLimiterMiddlewareService<S> {
    service: std::sync::Arc<S>,
    middleware: ResourceLimiterMiddleware,
}

impl<S, B> Service<ServiceRequest> for ResourceLimiterMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();
        let middleware = self.middleware.clone();

        Box::pin(async move {
            // 将ServiceRequest转换为HttpRequest
            let http_req = req.request();

            // 检查请求体大小
            if let Err(e) = middleware.check_body_size(http_req) {
                return Err(e.into());
            }

            // 检查请求头大小
            if let Err(e) = middleware.check_header_size(http_req) {
                return Err(e.into());
            }

            // 检查查询参数大小
            if let Err(e) = middleware.check_query_size(http_req) {
                return Err(e.into());
            }

            // 检查并发连接数
            if let Err(e) = middleware.check_concurrent_connections(http_req) {
                return Err(e.into());
            }

            // 处理请求
            let result = service.call(req).await;

            // 移除并发连接
            if let Ok(ref res) = result {
                middleware.remove_connection(res.request());
            }

            result
        })
    }
}

/// 资源限制中间件的辅助函数
pub fn resource_limiter_middleware() -> ResourceLimiterMiddleware {
    ResourceLimiterMiddleware::new(ResourceLimiterConfig::default())
}

pub fn resource_limiter_middleware_with_config(
    config: ResourceLimiterConfig,
) -> ResourceLimiterMiddleware {
    ResourceLimiterMiddleware::new(config)
}
