use actix_web::{Error, HttpMessage, HttpRequest, HttpResponse};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::env;

use crate::middleware::permission_checker::{role_from_str, PermissionChecker, Role};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub name: String,
    pub role: String,
    pub group_id: Option<i32>,
    pub exp: usize,
}

/// 检查是否处于生产模式
fn is_production() -> bool {
    env::var("PRODUCTION_MODE")
        .map(|v| v.eq_ignore_ascii_case("true") || v == "1")
        .unwrap_or(false)
}

/// 获取 JWT 密钥（生产环境强制验证）
fn get_jwt_secret() -> Result<String, Error> {
    let secret = env::var("JWT_SECRET")
        .map_err(|_| actix_web::error::ErrorInternalServerError("JWT_SECRET 环境变量未设置"))?;

    // 生产环境安全检查
    if is_production() {
        if secret.contains("dev") || secret.contains("change") || secret.contains("default") {
            log::error!("FATAL: 生产环境使用了不安全的 JWT_SECRET！请立即设置安全的密钥。");
            return Err(actix_web::error::ErrorInternalServerError(
                "生产环境使用了不安全的 JWT_SECRET，请联系管理员",
            ));
        }
        if secret.len() < 64 {
            log::error!("FATAL: 生产环境 JWT_SECRET 长度不足 64 字符！");
            return Err(actix_web::error::ErrorInternalServerError(
                "JWT_SECRET 长度不足，请使用至少 64 字符的安全密钥",
            ));
        }
        // 开发环境警告
    } else if secret.contains("dev") || secret.contains("change") {
        log::warn!("⚠️  使用默认 JWT_SECRET，仅限开发环境使用！");
    }

    Ok(secret)
}

/// 从请求中提取 token：优先从 HttpOnly cookie，fallback 到 Authorization header
fn extract_token(req: &HttpRequest) -> Option<String> {
    // 1. 从 HttpOnly cookie 读取（推荐方式）
    if let Some(cookie) = req.cookie("access_token") {
        let token = cookie.value().trim();
        if !token.is_empty() {
            return Some(token.to_string());
        }
    }

    // 2. Fallback：从 Authorization header 读取（兼容旧版前端）
    if let Some(auth_header) = req.headers().get("Authorization") {
        if let Ok(auth_value) = auth_header.to_str() {
            if let Some(token) = auth_value.strip_prefix("Bearer ") {
                return Some(token.to_string());
            }
        }
    }

    None
}

// JWT认证中间件
pub async fn jwt_auth_middleware(req: HttpRequest) -> Result<(), Error> {
    let token = match extract_token(&req) {
        Some(t) => t,
        None => {
            return Err(actix_web::error::ErrorUnauthorized(
                "缺少认证凭证（Cookie 或 Authorization 头）",
            ));
        }
    };

    let secret = get_jwt_secret()?;
    let decoding_key = DecodingKey::from_secret(secret.as_bytes());
    let validation = Validation::default();

    match decode::<Claims>(&token, &decoding_key, &validation) {
        Ok(claims) => {
            req.extensions_mut().insert(claims);
            Ok(())
        }
        Err(e) => {
            log::warn!("JWT 验证失败: {:?}", e);
            Err(actix_web::error::ErrorUnauthorized("无效的token"))
        }
    }
}

/// CSRF 防护中间件
pub async fn csrf_protection(req: HttpRequest) -> Result<HttpResponse, Error> {
    // 仅对状态修改请求进行 CSRF 检查
    let method = req.method();
    if method == "GET" || method == "HEAD" || method == "OPTIONS" {
        return Ok(HttpResponse::Ok().finish());
    }

    // 1. 检查 SameSite Cookie（浏览器自动防护）
    // 2. Double-Submit Cookie 验证
    let csrf_token = req.cookie("csrf_token").map(|c| c.value().to_string());

    let header_token = req
        .headers()
        .get("X-CSRF-Token")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());

    // 如果存在 CSRF token，两者必须匹配
    if let (Some(cookie_token), Some(header_token)) = (csrf_token, header_token) {
        if cookie_token != header_token {
            log::warn!("CSRF 验证失败");
            return Err(actix_web::error::ErrorForbidden("CSRF 验证失败"));
        }
    }

    Ok(HttpResponse::Ok().finish())
}

// 角色权限检查中间件
pub fn role_required(required_role: &'static str) -> impl Fn(HttpRequest) -> Result<(), Error> {
    move |req: HttpRequest| {
        let extensions = req.extensions();
        let claims = extensions.get::<Claims>();
        if claims.is_none() {
            return Err(actix_web::error::ErrorUnauthorized("未认证"));
        }

        let claims = claims.ok_or_else(|| actix_web::error::ErrorUnauthorized("认证信息缺失"))?;
        let user_role = role_from_str(&claims.role);
        let required_role_enum = role_from_str(required_role);

        let permission_checker = PermissionChecker::new(required_role_enum);
        if !permission_checker.check_role_permission(user_role) {
            return Err(actix_web::error::ErrorForbidden("权限不足"));
        }

        Ok(())
    }
}

// 资源权限检查中间件
pub fn resource_required(
    resource: &'static str,
    action: &'static str,
) -> impl Fn(HttpRequest) -> Result<(), Error> {
    move |req: HttpRequest| {
        let extensions = req.extensions();
        let claims = extensions.get::<Claims>();
        if claims.is_none() {
            return Err(actix_web::error::ErrorUnauthorized("未认证"));
        }

        let claims = claims.ok_or_else(|| actix_web::error::ErrorUnauthorized("认证信息缺失"))?;
        let user_role = role_from_str(&claims.role);

        let permission_checker = PermissionChecker::new(Role::Guest)
            .resource_str(resource)
            .action_str(action);

        match permission_checker.check_resource_permission(user_role) {
            Ok(_) => Ok(()),
            Err(err) => Err(actix_web::error::ErrorForbidden(err)),
        }
    }
}
