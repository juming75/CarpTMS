//! 审计日志中间件
//! 自动记录所有API请求的审计信息

use actix_web::{
    body::MessageBody,
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use futures_util::future::LocalBoxFuture;
use std::rc::Rc;
use std::time::Instant;

use crate::utils::audit::{log_audit_event, AuditLogRecord};

/// 审计日志中间件配置
#[derive(Debug, Clone)]
pub struct AuditLoggerConfig {
    /// 要记录审计的路径前缀
    pub monitored_paths: Vec<String>,
    /// 是否记录请求体
    pub log_request_body: bool,
    /// 是否记录响应体
    pub log_response_body: bool,
    /// 是否跳过健康检查等路径
    pub skip_paths: Vec<String>,
}

impl Default for AuditLoggerConfig {
    fn default() -> Self {
        Self {
            monitored_paths: vec!["/api/".to_string()],
            log_request_body: false,
            log_response_body: false,
            skip_paths: vec![
                "/health".to_string(),
                "/metrics".to_string(),
                "/swagger-ui".to_string(),
                "/api-docs".to_string(),
                "/api/auth/login".to_string(),
            ],
        }
    }
}

pub struct AuditLogger {
    config: AuditLoggerConfig,
}

impl AuditLogger {
    pub fn new(config: AuditLoggerConfig) -> Self {
        Self { config }
    }
}

impl Default for AuditLogger {
    fn default() -> Self {
        Self::new(AuditLoggerConfig::default())
    }
}

impl<S, B> Transform<S, ServiceRequest> for AuditLogger
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = AuditLoggerService<S>;
    type InitError = ();
    type Future = std::future::Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        std::future::ready(Ok(AuditLoggerService {
            service: Rc::new(service),
            config: self.config.clone(),
        }))
    }
}

pub struct AuditLoggerService<S> {
    service: Rc<S>,
    config: AuditLoggerConfig,
}

impl<S, B> Service<ServiceRequest> for AuditLoggerService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();
        let config = self.config.clone();
        let start = Instant::now();
        let method = req.method().to_string();
        let path = req.path().to_string();
        let query = req.query_string().to_string();
        let client_ip = req
            .peer_addr()
            .map(|a| a.ip().to_string())
            .unwrap_or_default();
        let user_agent = req
            .headers()
            .get("User-Agent")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        // 检查是否需要跳过
        let should_skip = config.skip_paths.iter().any(|p| path.starts_with(p));
        let should_monitor = config.monitored_paths.iter().any(|p| path.starts_with(p));

        // 获取用户信息（由认证中间件设置）
        let user_id = req.extensions().get::<AuditUserInfo>().cloned();

        Box::pin(async move {
            let response = service.call(req).await;

            // 记录审计日志（仅对监控路径且非跳过路径）
            if !should_skip && should_monitor {
                let duration = start.elapsed();
                let status = match &response {
                    Ok(resp) => resp.status().as_u16() as i32,
                    Err(_) => 500,
                };

                // 异步写入审计日志
                let pool_opt = response
                    .as_ref()
                    .ok()
                    .and_then(|r| r.request().app_data::<actix_web::web::Data<sqlx::PgPool>>())
                    .map(|d| d.get_ref().clone());

                if let Some(pool) = pool_opt {
                    let audit_record = AuditLogRecord {
                        user_id: user_id.as_ref().map(|u| u.user_id).unwrap_or(0),
                        username: user_id
                            .as_ref()
                            .map(|u| u.username.clone())
                            .unwrap_or_default(),
                        action: format!("{} {}", method, path),
                        resource: path.clone(),
                        resource_id: Some(query),
                        request_data: None,
                        ip_address: Some(client_ip),
                        user_agent,
                        result: if status < 400 { 1 } else { 0 },
                        error_message: if status >= 400 {
                            Some(format!("HTTP {}", status))
                        } else {
                            None
                        },
                    };

                    // 后台记录审计日志
                    let pool = pool.clone();
                    tokio::spawn(async move {
                        log_audit_event(&pool, audit_record).await;
                    });
                }

                tracing::debug!(
                    "Audit: {} {} {} {}ms",
                    method,
                    path,
                    status,
                    duration.as_millis()
                );
            }

            response
        })
    }
}

#[derive(Debug, Clone)]
pub struct AuditUserInfo {
    pub user_id: i32,
    pub username: String,
}

/// 在认证中间件中调用此函数来设置审计用户信息
pub fn set_audit_user(req: &mut actix_web::dev::ServiceRequest, user_id: i32, username: String) {
    req.extensions_mut()
        .insert(AuditUserInfo { user_id, username });
}
