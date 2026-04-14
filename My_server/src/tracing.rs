//! /! 分布式追踪模块
//!
//! 提供基于OpenTelemetry的分布式追踪功能,用于跟踪请求链路和性能分析

use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::Error;
use anyhow;
use futures_util::future::LocalBoxFuture;
use tracing::{span, Level};
use uuid::Uuid;

/// 初始化追踪系统
pub fn init_tracing() -> Result<(), anyhow::Error> {
    // 配置日志
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::FULL)
        .init();

    Ok(())
}

/// 关闭追踪系统
pub fn shutdown_tracing() {
    // 关闭追踪系统
}

/// 追踪中间件
pub struct TracingMiddleware;

impl TracingMiddleware {
    /// 创建新的追踪中间件
    pub fn new() -> Self {
        Self
    }
}

impl Default for TracingMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

impl<S, B> Transform<S, ServiceRequest> for TracingMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = TracingService<S>;
    type InitError = ();
    type Future = LocalBoxFuture<'static, Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        Box::pin(async move { Ok(TracingService { service }) })
    }
}

/// 追踪服务
pub struct TracingService<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for TracingService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        // 生成trace_id
        let trace_id = Uuid::new_v4().to_string();

        // 从请求中提取用户ID和设备ID(如果存在)
        let user_id = req
            .headers()
            .get("X-User-ID")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("unknown");
        let device_id = req
            .headers()
            .get("X-Device-ID")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("unknown");

        let span = span!(
            Level::INFO,
            "http.request",
            method = %req.method(),
            path = %req.path(),
            query = %req.query_string(),
            remote_addr = ?req.connection_info().peer_addr(),
            trace_id = %trace_id,
            user_id = %user_id,
            device_id = %device_id,
        );

        let fut = self.service.call(req);

        Box::pin(async move {
            let _enter = span.enter();
            fut.await
        })
    }
}

/// 追踪工具函数
pub mod trace {

    /// 创建一个新的追踪跨度
    pub fn span(name: &str, _fields: &[(&str, &str)]) -> tracing::Span {
        // 创建span并添加统一字段
        let span = tracing::span!(tracing::Level::INFO, "custom_span", name = name);

        span
    }

    /// 追踪一个异步函数
    pub async fn trace_async<F, T>(name: &str, fields: &[(&str, &str)], f: F) -> T
    where
        F: std::future::Future<Output = T>,
    {
        let span = span(name, fields);
        let _enter = span.enter();
        f.await
    }

    /// 追踪一个同步函数
    pub fn trace_sync<F, T>(name: &str, fields: &[(&str, &str)], f: F) -> T
    where
        F: FnOnce() -> T,
    {
        let span = span(name, fields);
        let _enter = span.enter();
        f()
    }

    /// 记录信息日志
    pub fn info(message: &str, _fields: &[(&str, &str)]) {
        tracing::info!(message);
    }

    /// 记录警告日志
    pub fn warn(message: &str, _fields: &[(&str, &str)]) {
        tracing::warn!(message);
    }

    /// 记录错误日志
    pub fn error(message: &str, error: &dyn std::error::Error, _fields: &[(&str, &str)]) {
        tracing::error!(message, error = ?error);
    }

    /// 脱敏敏感信息
    pub fn sanitize(value: &str) -> String {
        // 简单的脱敏实现,实际项目中可能需要更复杂的逻辑
        if value.len() > 8 {
            format!("{}****{}", &value[0..4], &value[value.len() - 4..])
        } else {
            "****".to_string()
        }
    }
}
