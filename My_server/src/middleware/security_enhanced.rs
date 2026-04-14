//! / 安全增强中间件
// 提供更完善的安全检查和权限验证
// 注意:此中间件暂时禁用,需要重新设计泛型类型
// 问题:ServiceResponse的泛型类型B与实际返回类型ServiceResponse<BoxBody>不匹配

use actix_web::{
    body::MessageBody,
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
};
use std::collections::HashSet;
use std::sync::Arc;
use std::task::{Context, Poll};

/// 安全配置
#[derive(Clone)]
pub struct SecurityConfig {
    /// 允许的来源
    pub allowed_origins: HashSet<String>,
    /// 允许的方法
    pub allowed_methods: HashSet<String>,
    /// 允许的头部
    pub allowed_headers: HashSet<String>,
    /// 最大请求体大小(字节)
    pub max_body_size: usize,
    /// 是否启用 CSRF 保护
    pub enable_csrf: bool,
    /// 是否启用速率限制
    pub enable_rate_limit: bool,
    /// 每分钟最大请求数
    pub max_requests_per_minute: u32,
    /// 是否启用 XSS 保护
    pub enable_xss_protection: bool,
    /// 是否启用内容安全策略
    pub enable_csp: bool,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        let mut allowed_origins = HashSet::new();
        allowed_origins.insert("http://localhost:5173".to_string());
        allowed_origins.insert("http://localhost:3000".to_string());
        allowed_origins.insert("https://*.CarpTMS.com".to_string());

        let mut allowed_methods = HashSet::new();
        allowed_methods.insert("GET".to_string());
        allowed_methods.insert("POST".to_string());
        allowed_methods.insert("PUT".to_string());
        allowed_methods.insert("DELETE".to_string());
        allowed_methods.insert("OPTIONS".to_string());

        let mut allowed_headers = HashSet::new();
        allowed_headers.insert("Content-Type".to_string());
        allowed_headers.insert("Authorization".to_string());
        allowed_headers.insert("X-Requested-With".to_string());

        Self {
            allowed_origins,
            allowed_methods,
            allowed_headers,
            max_body_size: 10 * 1024 * 1024, // 10MB
            enable_csrf: true,
            enable_rate_limit: true,
            max_requests_per_minute: 60,
            enable_xss_protection: true,
            enable_csp: true,
        }
    }
}

/// 安全增强中间件
pub struct SecurityEnhanced {
    #[allow(dead_code)]
    config: Arc<SecurityConfig>,
}

impl SecurityEnhanced {
    pub fn new(config: SecurityConfig) -> Self {
        Self {
            config: Arc::new(config),
        }
    }
}

use std::pin::Pin;

impl<S, B> Transform<S, ServiceRequest> for SecurityEnhanced
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error> + 'static + Clone,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = actix_web::Error;
    type Transform = SecurityEnhancedMiddleware<S>;
    type InitError = ();
    type Future = std::future::Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        std::future::ready(Ok(SecurityEnhancedMiddleware {
            service,
            config: self.config.clone(),
        }))
    }
}

pub struct SecurityEnhancedMiddleware<S> {
    service: S,
    config: Arc<SecurityConfig>,
}

impl<S, B> Service<ServiceRequest> for SecurityEnhancedMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error> + 'static + Clone,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = actix_web::Error;
    type Future = Pin<Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();
        let config = self.config.clone();

        Box::pin(async move {
            // 这里可以添加安全检查逻辑
            // 例如：请求体大小检查、速率限制等
            
            // 调用下一个服务
            service.call(req).await
        })
    }
}




