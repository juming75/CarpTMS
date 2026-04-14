//! / 监控中间件
use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error,
};
use futures::future::{ready, Future, Ready};
use std::pin::Pin;
use std::time::Instant;

use crate::metrics::{API_REQUESTS_TOTAL, API_REQUEST_DURATION};

// Metrics中间件
#[derive(Debug, Clone)]
pub struct MetricsMiddleware;

impl Default for MetricsMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricsMiddleware {
    pub fn new() -> Self {
        Self
    }
}

impl<S, B> Transform<S, ServiceRequest> for MetricsMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = MetricsMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(MetricsMiddlewareService { service }))
    }
}

#[derive(Debug, Clone)]
pub struct MetricsMiddlewareService<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for MetricsMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        // 记录请求开始时间
        let start_time = Instant::now();

        // 克隆请求信息用于异步回调
        let method = req.method().to_string();
        let path = req.path().to_string();

        // 处理请求
        let fut = self.service.call(req);

        // 异步处理响应
        Box::pin(async move {
            let result = fut.await;

            // 计算请求处理时间
            let duration = start_time.elapsed().as_secs_f64();

            // 获取状态码
            let status = match &result {
                Ok(res) => res.status().as_str().to_string(),
                Err(_) => "500".to_string(),
            };

            // 记录请求计数
            API_REQUESTS_TOTAL
                .with_label_values(&[&method, &path, &status])
                .inc();

            // 记录请求延迟
            API_REQUEST_DURATION
                .with_label_values(&[&method, &path, &status])
                .observe(duration);

            result
        })
    }
}
