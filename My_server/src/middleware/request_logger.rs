use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use futures::future::LocalBoxFuture;
use log::{info, warn};
use std::future::{ready, Ready};
use std::sync::Arc;
use tracing::info_span;
use uuid::Uuid;

// 日志中间件结构体
pub struct RequestLogger;

// 中间件工厂实现
impl RequestLogger {
    pub fn new() -> Self {
        Self
    }
}

// 添加Default实现
impl Default for RequestLogger {
    fn default() -> Self {
        Self::new()
    }
}

// 中间件转换实现
impl<S, B> Transform<S, ServiceRequest> for RequestLogger
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = RequestLoggerService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RequestLoggerService {
            service: Arc::new(service),
        }))
    }
}

// 中间件服务结构体
pub struct RequestLoggerService<S> {
    service: Arc<S>,
}

// 中间件服务实现
impl<S, B> Service<ServiceRequest> for RequestLoggerService<S>
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

        // 生成请求ID
        let request_id = Uuid::new_v4().to_string();

        // 获取请求信息
        let method = req.method().clone();
        let path = req.path().to_string();
        let remote_addr = req
            .connection_info()
            .realip_remote_addr()
            .unwrap_or("unknown")
            .to_string();

        // 创建日志span,包含请求ID和基本信息
        let span = info_span!("request",
            request_id = %request_id,
            method = %method,
            path = %path,
            remote_addr = %remote_addr
        );

        let _enter = span.enter();

        // 记录请求开始
        info!("Request started");

        Box::pin(async move {
            // 将请求ID添加到请求扩展
            req.extensions_mut().insert(request_id.clone());

            // 继续处理请求
            let mut res = service.call(req).await;

            // 记录响应
            match &mut res {
                Ok(res) => {
                    let status = res.status();
                    info!("Request completed with status: {}", status);

                    // 将请求ID添加到响应头
                    res.headers_mut().insert(
                        actix_web::http::header::HeaderName::from_static("x-request-id"),
                        actix_web::http::header::HeaderValue::from_str(&request_id).expect("request_id is valid ASCII"),
                    );
                }
                Err(e) => {
                    warn!("Request failed with error: {:?}", e);
                }
            }

            res
        })
    }
}
