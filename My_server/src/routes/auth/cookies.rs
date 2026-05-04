//! Cookie 工具函数
//!
//! 提供认证相关的 Cookie 构建函数

use actix_web::cookie::{time::Duration, Cookie, SameSite};
use actix_web::HttpRequest;

/// 根据环境决定 cookie 是否启用 Secure 标志（本地开发不启用）
pub fn secure_cookie() -> bool {
    use std::env;

    let is_dev = env::var("RUST_ENV")
        .map(|v| v == "development")
        .unwrap_or(false);
    let is_not_prod = env::var("PRODUCTION_MODE")
        .map(|v| v != "true" && v != "1")
        .unwrap_or(true);
    !is_dev && !is_not_prod
}

/// 构建 access_token Cookie（HttpOnly，24小时）
pub fn build_access_cookie(token: &str) -> Cookie<'static> {
    Cookie::build("access_token", token.to_owned())
        .path("/")
        .http_only(true)
        .secure(secure_cookie())
        .same_site(SameSite::Lax)
        .max_age(Duration::hours(24))
        .finish()
}

/// 构建 refresh_token Cookie（HttpOnly，7天）
pub fn build_refresh_cookie(token: &str) -> Cookie<'static> {
    Cookie::build("refresh_token", token.to_owned())
        .path("/")
        .http_only(true)
        .secure(secure_cookie())
        .same_site(SameSite::Lax)
        .max_age(Duration::days(7))
        .finish()
}

/// 构建 auth_check Cookie（非 HttpOnly，1小时）
pub fn build_auth_check_cookie() -> Cookie<'static> {
    Cookie::build("auth_check", "1")
        .path("/")
        .http_only(false)
        .secure(secure_cookie())
        .same_site(SameSite::Lax)
        .max_age(Duration::hours(1))
        .finish()
}

/// 从请求中提取指定 Cookie 值
pub fn extract_cookie(req: &HttpRequest, name: &str) -> Option<String> {
    req.cookie(name)
        .map(|c| c.value().trim().to_string())
        .filter(|s| !s.is_empty())
}
