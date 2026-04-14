use actix_web::{Error, HttpMessage, HttpRequest};
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
            return Err(actix_web::error::ErrorUnauthorized("缺少认证凭证（Cookie 或 Authorization 头）"));
        }
    };

    let secret = env::var("JWT_SECRET").unwrap_or_else(|_| "your-secret-key".to_string());
    let decoding_key = DecodingKey::from_secret(secret.as_bytes());
    let validation = Validation::default();

    match decode::<Claims>(&token, &decoding_key, &validation) {
        Ok(claims) => {
            req.extensions_mut().insert(claims);
            Ok(())
        }
        Err(_) => Err(actix_web::error::ErrorUnauthorized("无效的token")),
    }
}

// 角色权限检查中间件
pub fn role_required(required_role: &'static str) -> impl Fn(HttpRequest) -> Result<(), Error> {
    move |req: HttpRequest| {
        let extensions = req.extensions();
        let claims = extensions.get::<Claims>();
        if claims.is_none() {
            return Err(actix_web::error::ErrorUnauthorized("未认证"));
        }

        let claims = claims.expect("claims should be present after auth check");
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

        let claims = claims.expect("claims should be present after auth check");
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
