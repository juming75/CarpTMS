//! /! 全局错误处理中间件
//!
//! 为整个应用提供统一的错误处理和日志记录

use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::{web, Error, HttpRequest, HttpResponse};
use futures::future::LocalBoxFuture;
use log::error;
use std::future::{ready, Ready};
use std::rc::Rc;
use std::task::{Context, Poll};

/// 自定义错误处理中间件
pub struct ErrorHandler;

impl<S, B> Transform<S, ServiceRequest> for ErrorHandler
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = ErrorHandlerMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(ErrorHandlerMiddleware {
            service: Rc::new(service),
        }))
    }
}

pub struct ErrorHandlerMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for ErrorHandlerMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();
        let path = req.path().to_string();
        let method = req.method().to_string();

        Box::pin(async move {
            match service.call(req).await {
                Ok(response) => Ok(response),
                Err(err) => {
                    error!("Request failed: {} {} - {}", method, path, err);
                    Err(err)
                }
            }
        })
    }
}

/// 处理404错误
pub async fn not_found_handler(req: HttpRequest) -> HttpResponse {
    let path = req.path().to_string();
    error!("Resource not found: {}", path);

    HttpResponse::NotFound().json(serde_json::json!({
        "code": "NOT_FOUND",
        "message": format!("Resource not found: {}", path)
    }))
}

/// 处理通用错误
pub fn handle_error(err: Error) -> HttpResponse {
    error!("Unhandled error: {}", err);

    HttpResponse::InternalServerError().json(serde_json::json!({
        "code": "INTERNAL_ERROR",
        "message": "Internal server error",
        "details": err.to_string()
    }))
}

/// 为Actix-web应用配置错误处理
pub fn configure_error_handlers(cfg: &mut web::ServiceConfig) {
    cfg.default_service(web::route().to(not_found_handler));
}







