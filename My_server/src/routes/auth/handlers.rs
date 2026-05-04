//! 认证业务处理器
//!
//! 包含登录、刷新令牌、登出、获取用户信息、修改密码等处理器

use actix_web::{web, HttpRequest, HttpResponse};
use log::{error, info};
use std::sync::Arc;

use crate::domain::entities::auth::{LoginRequest, PasswordChangeRequest, RefreshTokenRequest};
use crate::domain::use_cases::auth::AuthUseCases;
use crate::errors::AppError;
use crate::errors::AppResult;
use crate::routes::auth::cookies::{
    build_access_cookie, build_auth_check_cookie, build_refresh_cookie,
};

use super::secure_cookie;

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

// ==================== Token 提取 ====================

/// 从请求中提取 Bearer Token
pub fn extract_token_from_request(req: &HttpRequest) -> Option<String> {
    req.headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|auth| auth.strip_prefix("Bearer ").map(|s| s.to_string()))
}

/// 从请求中提取 refresh_token：优先从 HttpOnly Cookie，其次从请求体
pub fn extract_refresh_token(
    req: &HttpRequest,
    body: &web::Json<serde_json::Value>,
) -> Option<String> {
    if let Some(cookie) = req.cookie("refresh_token") {
        let token = cookie.value().trim();
        if !token.is_empty() {
            return Some(token.to_string());
        }
    }
    if let Some(token) = body.get("refresh_token").and_then(|v| v.as_str()) {
        if !token.is_empty() {
            return Some(token.to_string());
        }
    }
    None
}

// ==================== 登录 ====================

/// POST /api/auth/login - 用户登录
pub async fn login(
    req: web::Json<LoginRequest>,
    auth_service: web::Data<Arc<AuthUseCases>>,
) -> AppResult<HttpResponse> {
    let username = req.username.clone();
    info!("Login attempt for user: {}", username);

    match auth_service.get_ref().login(req.into_inner()).await {
        Ok(token_response) => {
            let claims = crate::utils::jwt::verify_token(&token_response.access_token).ok();
            let user_info = if let Some(c) = &claims {
                let uid = c.sub.parse::<i32>().unwrap_or(0);
                let user_detail = auth_service.get_ref().get_user_info(uid).await.ok();
                if let Some(detail) = user_detail {
                    serde_json::json!({
                        "user_id": detail.user_id,
                        "username": detail.username,
                        "role": detail.role
                    })
                } else {
                    serde_json::json!({
                        "user_id": uid,
                        "username": username.clone(),
                        "role": c.role.clone()
                    })
                }
            } else {
                serde_json::json!({
                    "user_id": 0,
                    "username": username.clone(),
                    "role": "user"
                })
            };

            let response_data = serde_json::json!({
                "access_token": token_response.access_token,
                "refresh_token": token_response.refresh_token,
                "password_required": token_response.password_required,
                "user": user_info
            });

            let access_cookie = build_access_cookie(&token_response.access_token);
            let refresh_cookie = build_refresh_cookie(&token_response.refresh_token);
            let auth_check = build_auth_check_cookie();

            if token_response.password_required {
                let password_required_cookie =
                    actix_web::cookie::Cookie::build("password_required", "1")
                        .path("/")
                        .http_only(false)
                        .secure(secure_cookie())
                        .same_site(actix_web::cookie::SameSite::Lax)
                        .max_age(actix_web::cookie::time::Duration::hours(1))
                        .finish();

                info!(
                    "Login successful for user: {} (password change required)",
                    username
                );
                return Ok(HttpResponse::Ok()
                    .insert_header(("Set-Cookie", access_cookie.to_string()))
                    .insert_header(("Set-Cookie", refresh_cookie.to_string()))
                    .insert_header(("Set-Cookie", auth_check.to_string()))
                    .insert_header(("Set-Cookie", password_required_cookie.to_string()))
                    .json(response_data));
            }

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

// ==================== 刷新令牌 ====================

/// POST /api/auth/refresh - 刷新访问令牌
pub async fn refresh_token(
    http_req: HttpRequest,
    body: web::Json<serde_json::Value>,
    auth_service: web::Data<Arc<AuthUseCases>>,
) -> AppResult<HttpResponse> {
    let refresh_token_str = match extract_refresh_token(&http_req, &body) {
        Some(t) => t,
        None => {
            error!(
                "Token refresh failed: missing refresh_token (neither in cookie nor request body)"
            );
            return Err(AppError::permission_error("缺少刷新令牌"));
        }
    };

    let request = RefreshTokenRequest {
        refresh_token: refresh_token_str,
    };

    match auth_service.get_ref().refresh_token(request).await {
        Ok(token_response) => {
            let response_data = serde_json::json!({
                "access_token": token_response.access_token,
                "refresh_token": token_response.refresh_token,
                "password_required": token_response.password_required
            });

            let access_cookie = build_access_cookie(&token_response.access_token);
            let refresh_cookie = build_refresh_cookie(&token_response.refresh_token);

            if token_response.password_required {
                return Ok(HttpResponse::Ok()
                    .insert_header(("Set-Cookie", access_cookie.to_string()))
                    .insert_header(("Set-Cookie", refresh_cookie.to_string()))
                    .json(response_data));
            }

            Ok(HttpResponse::Ok()
                .insert_header(("Set-Cookie", access_cookie.to_string()))
                .insert_header(("Set-Cookie", refresh_cookie.to_string()))
                .json(response_data))
        }
        Err(e) => {
            error!("Token refresh failed: {:?}", e);
            Err(AppError::permission_error("刷新令牌无效或已过期"))
        }
    }
}

// ==================== 登出 ====================

use crate::security::jwt_blacklist::{calculate_ttl_seconds, JwtBlacklist};

/// POST /api/auth/logout - 用户登出
pub async fn logout(req: HttpRequest, _body: web::Json<serde_json::Value>) -> HttpResponse {
    use actix_web::cookie::CookieBuilder;

    // 提取 Token 并加入黑名单
    if let Some(token) = extract_token_from_request(&req) {
        if let Ok(claims) = crate::utils::jwt::verify_token(&token) {
            let ttl = calculate_ttl_seconds(claims.exp);
            if ttl > 0 {
                let blacklist = JwtBlacklist::from_env();
                match blacklist.add_to_blacklist(&token, ttl).await {
                    Ok(_) => info!("Access token 已加入黑名单"),
                    Err(e) => error!("Failed to add token to blacklist: {}", e),
                }
            }
        }
    }

    let clear_access = CookieBuilder::new("access_token", "")
        .path("/")
        .http_only(true)
        .secure(secure_cookie())
        .same_site(actix_web::cookie::SameSite::Lax)
        .max_age(actix_web::cookie::time::Duration::ZERO)
        .finish();

    let clear_refresh = CookieBuilder::new("refresh_token", "")
        .path("/")
        .http_only(true)
        .secure(secure_cookie())
        .same_site(actix_web::cookie::SameSite::Lax)
        .max_age(actix_web::cookie::time::Duration::ZERO)
        .finish();

    let clear_auth = CookieBuilder::new("auth_check", "")
        .path("/")
        .http_only(false)
        .secure(secure_cookie())
        .same_site(actix_web::cookie::SameSite::Lax)
        .max_age(actix_web::cookie::time::Duration::ZERO)
        .finish();

    HttpResponse::Ok()
        .insert_header(("Set-Cookie", clear_access.to_string()))
        .insert_header(("Set-Cookie", clear_refresh.to_string()))
        .insert_header(("Set-Cookie", clear_auth.to_string()))
        .json(serde_json::json!({
            "code": 200,
            "message": "登出成功"
        }))
}

// ==================== 用户信息 ====================

/// GET /api/auth/user/{id} - 根据路径 ID 获取用户信息
pub async fn get_current_user(
    path: web::Path<i32>,
    auth_service: web::Data<Arc<AuthUseCases>>,
) -> AppResult<HttpResponse> {
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

/// GET /api/auth/user - 从 Token 获取当前用户信息
pub async fn get_current_user_by_token(
    http_req: HttpRequest,
    auth_service: web::Data<Arc<AuthUseCases>>,
) -> AppResult<HttpResponse> {
    let token = extract_token_from_request(&http_req).ok_or_else(|| {
        error!("Get current user failed: missing authentication token");
        AppError::permission_error("缺少认证凭证")
    })?;

    let claims = crate::utils::jwt::verify_token(&token).map_err(|e| {
        error!("Get current user failed: invalid token: {:?}", e);
        AppError::permission_error("无效的认证凭证")
    })?;

    let user_id = claims.sub.parse::<i32>().map_err(|e| {
        error!("Failed to parse user_id from token: {:?}", e);
        AppError::permission_error("无效的用户身份")
    })?;

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

// ==================== 修改密码 ====================

/// POST /api/auth/change-password - 修改密码
pub async fn change_password(
    http_req: HttpRequest,
    req: web::Json<PasswordChangeRequest>,
    auth_service: web::Data<Arc<AuthUseCases>>,
) -> AppResult<HttpResponse> {
    use actix_web::cookie::CookieBuilder;

    let token = extract_token_from_request(&http_req).ok_or_else(|| {
        error!("Password change failed: missing authentication token");
        AppError::permission_error("缺少认证凭证")
    })?;

    let claims = crate::utils::jwt::verify_token(&token).map_err(|e| {
        error!("Password change failed: invalid token: {:?}", e);
        AppError::permission_error("无效的认证凭证")
    })?;

    let user_id = claims.sub.parse::<i32>().map_err(|e| {
        error!("Failed to parse user_id from token: {:?}", e);
        AppError::permission_error("无效的用户身份")
    })?;

    info!("Password change request for user_id: {}", user_id);

    match auth_service
        .get_ref()
        .change_password(user_id, req.into_inner())
        .await
    {
        Ok(_) => {
            // 清除 password_required cookie
            let clear_password_required = CookieBuilder::new("password_required", "")
                .path("/")
                .http_only(false)
                .secure(secure_cookie())
                .same_site(actix_web::cookie::SameSite::Lax)
                .max_age(actix_web::cookie::time::Duration::ZERO)
                .finish();

            info!("Password changed successfully for user_id: {}", user_id);
            let mut response = crate::errors::success_response(serde_json::json!({
                "message": "密码修改成功"
            }));
            response.headers_mut().insert(
                actix_web::http::header::SET_COOKIE,
                actix_web::http::header::HeaderValue::from_str(
                    &clear_password_required.to_string(),
                )
                .unwrap_or_else(|_| actix_web::http::header::HeaderValue::from_static("")),
            );
            Ok(response)
        }
        Err(e) => {
            error!("Password change failed for user_id: {}: {:?}", user_id, e);
            Err(e)
        }
    }
}
