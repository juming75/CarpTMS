//! /! 速率限制中间件

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error,
    HttpMessage,
};
use futures::future::{LocalBoxFuture, Ready, ready};
use log::warn;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// 速率限制配置
#[derive(Clone)]
pub struct RateLimitConfig {
    /// 时间窗口(秒)
    pub window_size: u64,
    /// 窗口内允许的最大请求数
    pub max_requests: u32,
    /// 速率限制策略
    pub strategy: RateLimitStrategy,
}

/// 速率限制策略
#[derive(Clone, Debug)]
pub enum RateLimitStrategy {
    /// 基于IP地址
    Ip,
    /// 基于用户ID
    User,
    /// 基于API路径
    Path,
}

/// 速率限制数据
struct RateLimitData {
    /// 最后请求时间
    last_request: Instant,
    /// 窗口内的请求次数
    request_count: u32,
}

/// 速率限制中间件
#[derive(Clone)]
pub struct RateLimitMiddleware {
    config: RateLimitConfig,
    /// 速率限制存储
    rate_limits: Arc<Mutex<HashMap<String, RateLimitData>>>,
}

impl RateLimitMiddleware {
    /// 创建新的速率限制中间件
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            config,
            rate_limits: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// 创建默认配置的速率限制中间件
    pub fn default() -> Self {
        Self::new(RateLimitConfig {
            window_size: 60, // 1分钟
            max_requests: 100, // 100次请求
            strategy: RateLimitStrategy::Ip,
        })
    }

    /// 创建IP-based速率限制中间件
    pub fn ip_based(max_requests: u32, window_size: u64) -> Self {
        Self::new(RateLimitConfig {
            window_size,
            max_requests,
            strategy: RateLimitStrategy::Ip,
        })
    }

    /// 创建User-based速率限制中间件
    pub fn user_based(max_requests: u32, window_size: u64) -> Self {
        Self::new(RateLimitConfig {
            window_size,
            max_requests,
            strategy: RateLimitStrategy::User,
        })
    }

    /// 创建Path-based速率限制中间件
    pub fn path_based(max_requests: u32, window_size: u64) -> Self {
        Self::new(RateLimitConfig {
            window_size,
            max_requests,
            strategy: RateLimitStrategy::Path,
        })
    }

    /// 获取速率限制键
    fn get_rate_limit_key(&self, req: &ServiceRequest) -> String {
        match self.config.strategy {
            RateLimitStrategy::Ip => {
                // 从请求中获取IP地址
                req.connection_info()
                    .realip_remote_addr()
                    .unwrap_or("unknown")
                    .to_string()
            }
            RateLimitStrategy::User => {
                // 从请求扩展中获取用户ID
                if let Some(claims) = req.extensions().get::<crate::utils::jwt::Claims>() {
                    format!("user:{}", claims.sub)
                } else {
                    "anonymous".to_string()
                }
            }
            RateLimitStrategy::Path => {
                // 从请求中获取路径
                req.path().to_string()
            }
        }
    }

    /// 检查速率限制
    fn check_rate_limit(&self, key: &str) -> bool {
        let Ok(mut rate_limits) = self.rate_limits.lock() else { return true };
        let now = Instant::now();
        let window_duration = Duration::from_secs(self.config.window_size);

        if let Some(data) = rate_limits.get_mut(key) {
            if now - data.last_request < window_duration {
                data.request_count += 1;
                data.last_request = now;
                data.request_count <= self.config.max_requests
            } else {
                *data = RateLimitData {
                    last_request: now,
                    request_count: 1,
                };
                true
            }
        } else {
            rate_limits.insert(
                key.to_string(),
                RateLimitData {
                    last_request: now,
                    request_count: 1,
                },
            );
            true
        }
    }
}

// 中间件转换实现
impl<S, B> Transform<S, ServiceRequest> for RateLimitMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = RateLimitMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RateLimitMiddlewareService {
            service: Arc::new(service),
            rate_limiter: self.clone(),
        }))
    }
}

// 中间件服务结构体
pub struct RateLimitMiddlewareService<S> {
    service: Arc<S>,
    rate_limiter: RateLimitMiddleware,
}

impl<S, B> Service<ServiceRequest> for RateLimitMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = Arc::clone(&self.service);
        let rate_limiter = self.rate_limiter.clone();

        Box::pin(async move {
            // 获取速率限制键
            let key = rate_limiter.get_rate_limit_key(&req);

            // 检查速率限制
            if !rate_limiter.check_rate_limit(&key) {
                warn!("Rate limit exceeded for key: {}", key);
                return Err(actix_web::error::ErrorTooManyRequests("Rate limit exceeded"));
            }

            // 继续处理请求
            let res = service.call(req).await?;
            Ok(res)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::test::TestRequest;

    #[test]
    fn test_get_rate_limit_key_ip() {
        let middleware = RateLimitMiddleware::ip_based(100, 60);
        let req = TestRequest::default().to_srv_request();
        let key = middleware.get_rate_limit_key(&req);
        assert_eq!(key, "unknown");
    }

    #[test]
    fn test_check_rate_limit() {
        let middleware = RateLimitMiddleware::default();
        let key = "test_ip";

        // 测试首次请求
        assert!(middleware.check_rate_limit(key));

        // 测试在限制内的请求
        for _ in 1..100 {
            assert!(middleware.check_rate_limit(key));
        }

        // 测试超出限制的请求
        assert!(!middleware.check_rate_limit(key));
    }
}





