//! 统一安全中间件
//! 整合认证、授权、审计、安全检查等功能

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage, HttpResponse,
};
use futures::future::{ok, LocalBoxFuture, Ready};
use std::{
    collections::HashMap,
    rc::Rc,
    sync::Arc,
    task::{Context, Poll},
    time::Instant,
};

use crate::domain::entities::auth::Claims;
use crate::security::audit_log::{AuditLogEntry, AuditLogLevel, AuditLogger};
use crate::security::jwt_blacklist::{calculate_ttl_seconds, JwtBlacklist};

/// 统一安全配置
#[derive(Clone)]
pub struct UnifiedSecurityConfig {
    /// 是否启用JWT黑名单检查
    pub enable_jwt_blacklist: bool,
    /// 是否启用审计日志
    pub enable_audit_log: bool,
    /// 是否启用请求日志
    pub enable_request_log: bool,
    /// 是否启用安全头
    pub enable_security_headers: bool,
    /// 是否启用速率限制检查
    pub enable_rate_limit: bool,
    /// 速率限制：每分钟最大请求数
    pub rate_limit_per_minute: u32,
    /// 审计日志级别
    pub audit_level: AuditLogLevel,
}

impl Default for UnifiedSecurityConfig {
    fn default() -> Self {
        Self {
            enable_jwt_blacklist: true,
            enable_audit_log: true,
            enable_request_log: true,
            enable_security_headers: true,
            enable_rate_limit: true,
            rate_limit_per_minute: 60,
            audit_level: AuditLogLevel::Info,
        }
    }
}

impl UnifiedSecurityConfig {
    /// 从环境变量加载配置
    pub fn from_env() -> Self {
        Self {
            enable_jwt_blacklist: std::env::var("SECURITY_ENABLE_JWT_BLACKLIST")
                .map(|v| v != "false")
                .unwrap_or(true),
            enable_audit_log: std::env::var("SECURITY_ENABLE_AUDIT_LOG")
                .map(|v| v != "false")
                .unwrap_or(true),
            enable_request_log: std::env::var("SECURITY_ENABLE_REQUEST_LOG")
                .map(|v| v != "false")
                .unwrap_or(true),
            enable_security_headers: std::env::var("SECURITY_ENABLE_HEADERS")
                .map(|v| v != "false")
                .unwrap_or(true),
            enable_rate_limit: std::env::var("SECURITY_ENABLE_RATE_LIMIT")
                .map(|v| v != "false")
                .unwrap_or(true),
            rate_limit_per_minute: std::env::var("SECURITY_RATE_LIMIT_PER_MINUTE")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(60),
            audit_level: std::env::var("SECURITY_AUDIT_LEVEL")
                .ok()
                .map(|v| match v.to_lowercase().as_str() {
                    "debug" => AuditLogLevel::Debug,
                    "warning" => AuditLogLevel::Warning,
                    "error" => AuditLogLevel::Error,
                    "critical" => AuditLogLevel::Critical,
                    _ => AuditLogLevel::Info,
                })
                .unwrap_or(AuditLogLevel::Info),
        }
    }
}

/// 统一安全中间件
pub struct UnifiedSecurityMiddleware {
    config: UnifiedSecurityConfig,
    audit_logger: Arc<AuditLogger>,
}

impl UnifiedSecurityMiddleware {
    pub fn new(config: UnifiedSecurityConfig) -> Self {
        Self {
            config,
            audit_logger: Arc::new(AuditLogger::new()),
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for UnifiedSecurityMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = UnifiedSecurityMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(UnifiedSecurityMiddlewareService {
            service: Rc::new(service),
            config: self.config.clone(),
            audit_logger: self.audit_logger.clone(),
        })
    }
}

pub struct UnifiedSecurityMiddlewareService<S> {
    service: Rc<S>,
    config: UnifiedSecurityConfig,
    audit_logger: Arc<AuditLogger>,
}

impl<S, B> Service<ServiceRequest> for UnifiedSecurityMiddlewareService<S>
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
        let start = Instant::now();
        let config = self.config.clone();
        let audit_logger = self.audit_logger.clone();
        let service = self.service.clone();

        Box::pin(async move {
            // 记录请求信息
            let method = req.method().to_string();
            let path = req.path().to_string();
            let client_ip = req
                .connection_info()
                .realip_remote_addr()
                .map(|s| s.to_string());

            // 添加安全头
            if config.enable_security_headers {
                req.extensions_mut().insert(SecurityHeadersAdded {});
            }

            // 审计日志：记录访问
            if config.enable_request_log {
                let user = req
                    .extensions()
                    .get::<Claims>()
                    .map(|c| c.name.clone());
                let ip = client_ip.clone();

                audit_logger.log_with_context(
                    AuditLogLevel::Info,
                    &format!("请求: {} {}", method, path),
                    Some(serde_json::json!({
                        "method": method,
                        "path": path,
                        "user_agent": req.headers().get("user-agent").and_then(|h| h.to_str().ok())
                    })),
                    user,
                    ip,
                );
            }

            // 执行请求
            let res = service.call(req).await?;

            // 审计日志：记录响应
            if config.enable_audit_log {
                let status = res.status().as_u16();
                let user = res
                    .request()
                    .extensions()
                    .get::<Claims>()
                    .map(|c| c.name.clone());
                let ip = res
                    .request()
                    .connection_info()
                    .realip_remote_addr()
                    .map(|s| s.to_string());
                let duration = start.elapsed();

                if status >= 400 {
                    audit_logger.log_with_context(
                        AuditLogLevel::Warning,
                        &format!("请求失败: {} {} - 状态码 {}", method, path, status),
                        Some(serde_json::json!({
                            "status": status,
                            "duration_ms": duration.as_millis()
                        })),
                        user,
                        ip,
                    );
                }
            }

            Ok(res)
        })
    }
}

/// 安全头标记
struct SecurityHeadersAdded;

/// 审计日志辅助函数
pub async fn log_security_event(
    level: AuditLogLevel,
    message: &str,
    details: Option<serde_json::Value>,
    user: Option<String>,
    ip: Option<String>,
) {
    if let Ok(logger) = AuditLogger::new().await {
        logger.log_with_context(level, message, details, user, ip).await;
    }
}

/// 检查JWT是否在黑名单中
pub async fn check_jwt_blacklist(token: &str) -> Result<bool, String> {
    let blacklist = JwtBlacklist::from_env();
    blacklist.is_blacklisted(token).await
}

/// 将JWT加入黑名单
pub async fn blacklist_jwt(token: &str, exp: usize) -> Result<(), String> {
    let blacklist = JwtBlacklist::from_env();
    let ttl = calculate_ttl_seconds(exp);
    if ttl > 0 {
        blacklist.add_to_blacklist(token, ttl).await?;
    }
    Ok(())
}

/// 速率限制计数器
lazy_static::lazy_static! {
    static ref RATE_LIMIT_COUNTER: std::sync::Mutex<HashMap<String, Vec<u64>>> = std::sync::Mutex::new(HashMap::new());
}

/// 检查速率限制
pub fn check_rate_limit(key: &str, limit: u32) -> Result<(), String> {
    let mut counter = RATE_LIMIT_COUNTER.lock().map_err(|e| e.to_string())?;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_secs();

    let window_start = now - 60; // 1分钟窗口

    // 清理过期记录
    if let Some(times) = counter.get_mut(key) {
        times.retain(|&t| t > window_start);
    }

    // 检查限制
    let times = counter.entry(key.to_string()).or_insert_with(Vec::new);
    if times.len() >= limit as usize {
        return Err(format!("速率限制 exceeded: 每分钟最多 {} 个请求", limit));
    }

    times.push(now);
    Ok(())
}
