//! 认证领域实体

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// 登录请求
#[derive(Debug, Clone, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

/// 刷新令牌请求
#[derive(Debug, Clone, Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

/// 令牌响应
#[derive(Debug, Clone, Serialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
}

/// 用户信息
#[derive(Debug, Clone, Serialize, FromRow)]
pub struct UserInfo {
    pub user_id: i32,
    pub username: String,
    pub real_name: Option<String>,
    pub role: String,
    pub group_id: i32,
    pub create_time: NaiveDateTime,
    pub update_time: Option<NaiveDateTime>,
}

/// 用户认证信息
#[derive(Debug, Clone, FromRow)]
pub struct UserAuth {
    pub user_id: i32,
    pub username: String,
    pub password_hash: String,
    pub user_group_id: i32,
}
