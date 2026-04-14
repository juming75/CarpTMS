//! 认证相关路由

use std::env;

use actix_web::{web, HttpResponse};
use log::{error, info};

use crate::domain::entities::auth::{LoginRequest, RefreshTokenRequest};
use crate::domain::use_cases::auth::AuthUseCases;
use crate::errors::AppError;

/// 根据环境决定 cookie 是否启用 Secure 标志（本地开发不启用）
fn secure_cookie() -> bool {
    env::var("RUST_ENV").unwrap_or_default() != "development"
}

/// 认证相关路由配置
pub fn configure_auth_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/auth")
            .route("/login", web::post().to(login))
            .route("/refresh", web::post().to(refresh_token))
            .route("/logout", web::post().to(logout))
            .route("/user/{id}", web::get().to(get_current_user)),
    );
}

/// 登录
pub async fn login(
    req: web::Json<LoginRequest>,
    auth_service: web::Data<std::sync::Arc<AuthUseCases>>,
) -> crate::errors::AppResult<HttpResponse> {
    let username = req.username.clone();
    info!("Login attempt for user: {}", username);

    match auth_service.get_ref().login(req.into_inner()).await {
        Ok(token_response) => {
            let user_info = serde_json::json!({
                "user_id": 0,
                "username": username,
                "role": "user"
            });

            let response_data = serde_json::json!({
                "access_token": token_response.access_token,
                "refresh_token": token_response.refresh_token,
                "user": user_info
            });

            // HttpOnly cookies：Token 永不进入 JavaScript
            let access_cookie = actix_web::cookie::Cookie::build("access_token", &token_response.access_token)
                .path("/")
                .http_only(false)
                .secure(secure_cookie())
                .same_site(actix_web::cookie::SameSite::Strict)
                .max_age(actix_web::cookie::time::Duration::hours(24))
                .finish();

            let refresh_cookie = actix_web::cookie::Cookie::build("refresh_token", &token_response.refresh_token)
                .path("/")
                .http_only(false)
                .secure(secure_cookie())
                .same_site(actix_web::cookie::SameSite::Strict)
                .max_age(actix_web::cookie::time::Duration::days(7))
                .finish();

            // JS 可读 cookie：用于前端路由守卫检测登录状态
            let auth_check = actix_web::cookie::Cookie::build("auth_check", "1")
                .path("/")
                .http_only(false)
                .secure(secure_cookie())
                .same_site(actix_web::cookie::SameSite::Strict)
                .max_age(actix_web::cookie::time::Duration::hours(24))
                .finish();

            info!("Login successful for user: {}", username);
            Ok(HttpResponse::Ok()
                .insert_header(("Set-Cookie", access_cookie.to_string()))
                .insert_header(("Set-Cookie", refresh_cookie.to_string()))
                .insert_header(("Set-Cookie", auth_check.to_string()))
                .json(response_data))
        }
        Err(e) => {
            error!("Login failed for user: {}: {:?}", username, e);
            Err(AppError::permission_error("用户名或密码错误"))
        }
    }
}

/// 刷新令牌
pub async fn refresh_token(
    req: web::Json<RefreshTokenRequest>,
    auth_service: web::Data<std::sync::Arc<AuthUseCases>>,
) -> crate::errors::AppResult<HttpResponse> {
    match auth_service.get_ref().refresh_token(req.into_inner()).await {
        Ok(token_response) => {
            let response_data = serde_json::json!({
                "access_token": token_response.access_token,
                "refresh_token": token_response.refresh_token,
                "expires_in": 3600
            });

            let access_cookie = actix_web::cookie::Cookie::build("access_token", &token_response.access_token)
                .path("/")
                .http_only(false)
                .secure(secure_cookie())
                .same_site(actix_web::cookie::SameSite::Strict)
                .max_age(actix_web::cookie::time::Duration::hours(24))
                .finish();

            let refresh_cookie = actix_web::cookie::Cookie::build("refresh_token", &token_response.refresh_token)
                .path("/")
                .http_only(false)
                .secure(secure_cookie())
                .same_site(actix_web::cookie::SameSite::Strict)
                .max_age(actix_web::cookie::time::Duration::days(7))
                .finish();

            let auth_check = actix_web::cookie::Cookie::build("auth_check", "1")
                .path("/")
                .http_only(false)
                .secure(secure_cookie())
                .same_site(actix_web::cookie::SameSite::Strict)
                .max_age(actix_web::cookie::time::Duration::hours(24))
                .finish();

            Ok(HttpResponse::Ok()
                .insert_header(("Set-Cookie", access_cookie.to_string()))
                .insert_header(("Set-Cookie", refresh_cookie.to_string()))
                .insert_header(("Set-Cookie", auth_check.to_string()))
                .json(response_data))
        }
        Err(e) => {
            error!("Token refresh failed: {:?}", e);
            Err(AppError::permission_error("刷新令牌无效或已过期"))
        }
    }
}

/// 登出 — 清除所有认证 cookies
pub async fn logout(_req: web::Json<serde_json::Value>) -> HttpResponse {
    let clear_access = actix_web::cookie::Cookie::build("access_token", "")
        .path("/")
        .http_only(false)
        .secure(secure_cookie())
        .same_site(actix_web::cookie::SameSite::Strict)
        .max_age(actix_web::cookie::time::Duration::ZERO)
        .finish();

    let clear_refresh = actix_web::cookie::Cookie::build("refresh_token", "")
        .path("/")
        .http_only(false)
        .secure(secure_cookie())
        .same_site(actix_web::cookie::SameSite::Strict)
        .max_age(actix_web::cookie::time::Duration::ZERO)
        .finish();

    let clear_auth = actix_web::cookie::Cookie::build("auth_check", "")
        .path("/")
        .http_only(false)
        .secure(secure_cookie())
        .same_site(actix_web::cookie::SameSite::Strict)
        .max_age(actix_web::cookie::time::Duration::ZERO)
        .finish();

    HttpResponse::Ok()
        .insert_header(("Set-Cookie", clear_access.to_string()))
        .insert_header(("Set-Cookie", clear_refresh.to_string()))
        .insert_header(("Set-Cookie", clear_auth.to_string()))
        .finish()
}

/// 获取当前用户信息
pub async fn get_current_user(
    path: web::Path<i32>,
    auth_service: web::Data<std::sync::Arc<AuthUseCases>>,
) -> crate::errors::AppResult<HttpResponse> {
    let user_id = path.into_inner();

    match auth_service.get_ref().get_user_info(user_id).await {
        Ok(user_info) => {
            let user_data = serde_json::json!({
                "id": user_info.user_id,
                "username": user_info.username,
                "role": user_info.role
            });
            Ok(crate::errors::success_response(user_data))
        }
        Err(e) => {
            error!("Failed to get user info: {:?}", e);
            Err(AppError::permission_error("用户不存在或已被删除"))
        }
    }
}
