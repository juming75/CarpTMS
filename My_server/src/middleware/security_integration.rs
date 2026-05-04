//! 安全中间件集成模块
//! 加载安全配置并应用到所有路由

use actix_web::{
    body::EitherBody,
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpResponse,
};
use std::{
    future::{ready, Future, Ready},
    pin::Pin,
    rc::Rc,
    task::{Context, Poll},
};

use crate::security::security_config::SecurityConfig;

/// Cookie 安全配置
#[derive(Clone, Debug)]
pub struct CookieSecurityConfig {
    pub http_only: bool,
    pub secure: bool,
    pub same_site: String,
    pub max_age_secs: u64,
    pub path: String,
}

/// JWT 安全配置
#[derive(Clone, Debug)]
pub struct JwtSecurityConfig {
    pub access_token_expires: u64,
    pub refresh_token_expires: u64,
    pub refresh_threshold: u64,
    pub enable_blacklist: bool,
}

/// 密码安全配置
#[derive(Clone, Debug)]
pub struct PasswordSecurityConfig {
    pub min_length: u32,
    pub require_uppercase: bool,
    pub require_lowercase: bool,
    pub require_digit: bool,
    pub require_special: bool,
    pub history_count: u32,
    pub expire_days: u32,
    pub lockout_enabled: bool,
    pub lockout_max_attempts: u32,
    pub lockout_duration_mins: u32,
}

/// 审计日志安全配置
#[derive(Clone, Debug)]
pub struct AuditSecurityConfig {
    pub enabled: bool,
    pub level: String,
    pub retention_days: u32,
}

/// CSRF 安全配置
#[derive(Clone, Debug)]
pub struct CsrfSecurityConfig {
    pub enabled: bool,
    pub token_lifetime: u64,
}

/// 速率限制安全配置
#[derive(Clone, Debug)]
pub struct RateLimitSecurityConfig {
    pub enabled: bool,
    pub auth_per_minute: u32,
    pub api_per_minute: u32,
}

/// 完整安全配置
#[derive(Clone, Debug)]
pub struct AppSecurityConfig {
    pub level: String,
    pub production_enforce: bool,
    pub cookie: CookieSecurityConfig,
    pub jwt: JwtSecurityConfig,
    pub password: PasswordSecurityConfig,
    pub audit: AuditSecurityConfig,
    pub csrf: CsrfSecurityConfig,
    pub rate_limit: RateLimitSecurityConfig,
}

impl Default for AppSecurityConfig {
    fn default() -> Self {
        Self {
            level: "medium".to_string(),
            production_enforce: false,
            cookie: CookieSecurityConfig {
                http_only: true,
                secure: false, // 开发环境默认关闭
                same_site: "lax".to_string(),
                max_age_secs: 86400,
                path: "/".to_string(),
            },
            jwt: JwtSecurityConfig {
                access_token_expires: 86400,
                refresh_token_expires: 604800,
                refresh_threshold: 300,
                enable_blacklist: true,
            },
            password: PasswordSecurityConfig {
                min_length: 8,
                require_uppercase: true,
                require_lowercase: true,
                require_digit: true,
                require_special: true,
                history_count: 5,
                expire_days: 0,
                lockout_enabled: false,
                lockout_max_attempts: 5,
                lockout_duration_mins: 30,
            },
            audit: AuditSecurityConfig {
                enabled: true,
                level: "info".to_string(),
                retention_days: 180,
            },
            csrf: CsrfSecurityConfig {
                enabled: true,
                token_lifetime: 86400,
            },
            rate_limit: RateLimitSecurityConfig {
                enabled: true,
                auth_per_minute: 10,
                api_per_minute: 60,
            },
        }
    }
}

impl AppSecurityConfig {
    /// 从 TOML 配置文件加载
    pub async fn load_from_file(path: &str) -> Result<Self, anyhow::Error> {
        let content = tokio::fs::read_to_string(path).await?;
        Self::parse_from_toml(&content)
    }

    /// 从 TOML 字符串解析
    pub fn parse_from_toml(content: &str) -> Result<Self, anyhow::Error> {
        // 简化的 TOML 解析，实际使用中可用 toml crate
        let mut config = Self::default();

        // 解析安全级别
        if let Some(level) = extract_toml_value(content, "security.level") {
            config.level = level.trim_matches('"').to_string();
        }

        // 解析 Cookie 配置
        if let Some(value) = extract_toml_value(content, "cookie.access_token.http_only") {
            config.cookie.http_only = value == "true";
        }
        if let Some(value) = extract_toml_value(content, "cookie.access_token.secure") {
            config.cookie.secure = value == "true";
        }
        if let Some(value) = extract_toml_value(content, "cookie.access_token.same_site") {
            config.cookie.same_site = value.trim_matches('"').to_string();
        }
        if let Some(value) = extract_toml_value(content, "cookie.access_token.max_age_hours") {
            config.cookie.max_age_secs = value.parse::<u64>().unwrap_or(24) * 3600;
        }

        // 解析 JWT 配置
        if let Some(value) = extract_toml_value(content, "jwt.access_token_expires") {
            config.jwt.access_token_expires = value.parse().unwrap_or(86400);
        }
        if let Some(value) = extract_toml_value(content, "jwt.refresh_token_expires") {
            config.jwt.refresh_token_expires = value.parse().unwrap_or(604800);
        }
        if let Some(value) = extract_toml_value(content, "jwt.enable_blacklist") {
            config.jwt.enable_blacklist = value == "true";
        }

        // 解析密码策略
        if let Some(value) = extract_toml_value(content, "password.min_length") {
            config.password.min_length = value.parse().unwrap_or(8);
        }
        if let Some(value) = extract_toml_value(content, "password.require_uppercase") {
            config.password.require_uppercase = value == "true";
        }
        if let Some(value) = extract_toml_value(content, "password.require_lowercase") {
            config.password.require_lowercase = value == "true";
        }
        if let Some(value) = extract_toml_value(content, "password.require_digit") {
            config.password.require_digit = value == "true";
        }
        if let Some(value) = extract_toml_value(content, "password.require_special") {
            config.password.require_special = value == "true";
        }
        if let Some(value) = extract_toml_value(content, "password.history_count") {
            config.password.history_count = value.parse().unwrap_or(5);
        }
        if let Some(value) = extract_toml_value(content, "password.expire_days") {
            config.password.expire_days = value.parse().unwrap_or(0);
        }
        if let Some(value) = extract_toml_value(content, "password.lockout.enabled") {
            config.password.lockout_enabled = value == "true";
        }
        if let Some(value) = extract_toml_value(content, "password.lockout.max_attempts") {
            config.password.lockout_max_attempts = value.parse().unwrap_or(5);
        }
        if let Some(value) = extract_toml_value(content, "password.lockout.lockout_duration_minutes") {
            config.password.lockout_duration_mins = value.parse().unwrap_or(30);
        }

        // 解析审计配置
        if let Some(value) = extract_toml_value(content, "audit.enabled") {
            config.audit.enabled = value == "true";
        }
        if let Some(value) = extract_toml_value(content, "audit.level") {
            config.audit.level = value.trim_matches('"').to_string();
        }
        if let Some(value) = extract_toml_value(content, "audit.retention_days") {
            config.audit.retention_days = value.parse().unwrap_or(180);
        }

        // 解析 CSRF 配置
        if let Some(value) = extract_toml_value(content, "csrf.enabled") {
            config.csrf.enabled = value == "true";
        }
        if let Some(value) = extract_toml_value(content, "csrf.token_lifetime") {
            config.csrf.token_lifetime = value.parse().unwrap_or(86400);
        }

        // 解析速率限制配置
        if let Some(value) = extract_toml_value(content, "rate_limit.enabled") {
            config.rate_limit.enabled = value == "true";
        }
        if let Some(value) = extract_toml_value(content, "rate_limit.auth.requests_per_minute") {
            config.rate_limit.auth_per_minute = value.parse().unwrap_or(10);
        }
        if let Some(value) = extract_toml_value(content, "rate_limit.api.requests_per_minute") {
            config.rate_limit.api_per_minute = value.parse().unwrap_or(60);
        }

        Ok(config)
    }

    /// 是否为生产环境
    pub fn is_production(&self) -> bool {
        std::env::var("PRODUCTION_MODE")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(false)
    }

    /// 是否启用了高安全级别
    pub fn is_high_security(&self) -> bool {
        self.level == "high" || self.level == "critical" || self.production_enforce
    }

    /// 获取 Cookie 安全配置（生产环境硬化）
    pub fn get_production_cookie_config(&self) -> CookieSecurityConfig {
        if self.is_production() || self.is_high_security() {
            CookieSecurityConfig {
                http_only: true,
                secure: true,
                same_site: "strict".to_string(),
                max_age_secs: 86400,
                path: "/".to_string(),
            }
        } else {
            self.cookie.clone()
        }
    }
}

/// 从 TOML 内容中提取指定路径的值
fn extract_toml_value(content: &str, path: &str) -> Option<String> {
    let parts: Vec<&str> = path.split('.').collect();
    let mut current_content = content;

    for (i, part) in parts.iter().enumerate() {
        let search_pattern = if i == 0 {
            format!("{} =", part)
        } else {
            format!("\n{} =", part)
        };

        if let Some(start_pos) = current_content.find(&search_pattern) {
            let line_start = current_content[..start_pos].rfind('\n').map(|p| p + 1).unwrap_or(0);
            let line_end = current_content[start_pos..]
                .find('\n')
                .map(|p| start_pos + p)
                .unwrap_or(current_content.len());

            let line = &current_content[line_start..line_end];

            // 提取值
            if let Some(eq_pos) = line.find('=') {
                let value = line[eq_pos + 1..].trim();
                return Some(value.to_string());
            }
        }
    }

    None
}

/// 安全头中间件
pub struct SecurityHeadersMiddleware;

impl<S, B> Transform<S, ServiceRequest> for SecurityHeadersMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = SecurityHeadersMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(SecurityHeadersMiddlewareService {
            service: Rc::new(service),
        }))
    }
}

pub struct SecurityHeadersMiddlewareService<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for SecurityHeadersMiddlewareService<S>
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

        Box::pin(async move {
            let res = service.call(req).await?;

            // 添加安全头
            let mut res = res.map_body(|_, body| {
                EitherBody::new(body)
            });

            // 在响应中添加安全头
            let headers = res.headers_mut();
            headers.insert(
                actix_web::http::header::STRICT_TRANSPORT_SECURITY,
                actix_web::http::header::HeaderValue::from_static(
                    "max-age=31536000; includeSubDomains",
                ),
            );
            headers.insert(
                actix_web::http::header::X_CONTENT_TYPE_OPTIONS,
                actix_web::http::header::HeaderValue::from_static("nosniff"),
            );
            headers.insert(
                actix_web::http::header::X_FRAME_OPTIONS,
                actix_web::http::header::HeaderValue::from_static("DENY"),
            );
            headers.insert(
                actix_web::http::header::X_XSS_PROTECTION,
                actix_web::http::header::HeaderValue::from_static("1; mode=block"),
            );
            headers.insert(
                actix_web::http::header::REFERRER_POLICY,
                actix_web::http::header::HeaderValue::from_static(
                    "strict-origin-when-cross-origin",
                ),
            );
            headers.insert(
                "Content-Security-Policy",
                actix_web::http::header::HeaderValue::from_static(
                    "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; img-src 'self' data: https:; font-src 'self'; connect-src 'self' https:; frame-ancestors 'none';",
                ),
            );

            Ok(res)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_security_config() {
        let toml_content = r#"
[security]
level = "high"
production_enforce = true

[jwt]
access_token_expires = 86400
refresh_token_expires = 604800
enable_blacklist = true

[password]
min_length = 12
require_uppercase = true
require_lowercase = true
require_digit = true
require_special = true
history_count = 10
expire_days = 90

[password.lockout]
enabled = true
max_attempts = 5
lockout_duration_minutes = 30

[audit]
enabled = true
level = "info"
retention_days = 180
"#;

        let config = AppSecurityConfig::parse_from_toml(toml_content).unwrap();
        assert_eq!(config.level, "high");
        assert!(config.production_enforce);
        assert_eq!(config.password.min_length, 12);
        assert_eq!(config.password.expire_days, 90);
        assert!(config.password.lockout_enabled);
        assert!(config.audit.enabled);
        assert_eq!(config.audit.retention_days, 180);
    }
}
