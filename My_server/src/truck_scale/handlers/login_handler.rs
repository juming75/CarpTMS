//! / 登录处理器
use crate::truck_scale::auth::{Authenticator, SessionManager};
use crate::truck_scale::db::TruckScaleDb;
use anyhow::Result;
use chrono::Utc;
use serde::{Deserialize, Serialize};

/// 登录请求
#[derive(Debug, Clone, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
    pub client_version: Option<String>,
}

/// 登录响应
#[derive(Debug, Clone, Serialize)]
pub struct LoginResponse {
    pub success: bool,
    pub session_id: Option<String>,
    pub user_info: Option<serde_json::Value>,
    pub server_time: String,
    pub message: String,
    // 前端兼容字段
    pub access_token: Option<String>,
    pub token: Option<String>,
    pub refresh_token: Option<String>,
    pub user: Option<serde_json::Value>,
}

/// 登录错误类型
#[derive(Debug, thiserror::Error)]
pub enum LoginError {
    #[error("用户名或密码错误")]
    InvalidCredentials,
    #[error("用户已被禁用")]
    UserDisabled,
    #[error("用户已过期")]
    UserExpired,
    #[error("会话创建失败")]
    SessionCreationFailed,
    #[error("数据库错误: {0}")]
    DatabaseError(String),
}

impl From<anyhow::Error> for LoginError {
    fn from(err: anyhow::Error) -> Self {
        LoginError::DatabaseError(err.to_string())
    }
}

/// 登录处理器
pub struct LoginHandler {
    db: TruckScaleDb,
    authenticator: Authenticator,
    session_manager: SessionManager,
}

impl LoginHandler {
    /// 创建新的登录处理器
    pub fn new(db: TruckScaleDb, session_manager: SessionManager) -> Self {
        Self {
            db,
            authenticator: Authenticator::new(),
            session_manager,
        }
    }

    /// 处理登录请求
    pub async fn handle_login(
        &self,
        request: LoginRequest,
        connection_id: &str,
        client_ip: Option<&str>,
    ) -> Result<LoginResponse, LoginError> {
        // 查询用户
        let user = self
            .db
            .query_user_by_name(&request.username)
            .await
            .map_err(|e| LoginError::DatabaseError(e.to_string()))?
            .ok_or(LoginError::InvalidCredentials)?;

        // 验证密码
        let password_hash = user["password"]
            .as_str()
            .ok_or_else(|| LoginError::DatabaseError("Missing password hash".to_string()))?;

        let is_valid = self
            .authenticator
            .verify_password(&request.password, password_hash)
            .map_err(|e| LoginError::DatabaseError(e.to_string()))?;

        if !is_valid {
            return Err(LoginError::InvalidCredentials);
        }

        // 检查用户状态
        let status = user["status"]
            .as_i64()
            .ok_or_else(|| LoginError::DatabaseError("Missing status".to_string()))?;

        if status != 0 {
            return Err(LoginError::UserDisabled);
        }

        // 检查过期时间(如果有)
        if let Some(expiration_time) = user["expiration_time"].as_str() {
            if !expiration_time.is_empty() {
                // 简单的日期比较,实际使用chrono::DateTime解析
                // 这里简化处理
            }
        }

        // 创建会话
        let user_id = user["user_id"]
            .as_str()
            .ok_or_else(|| LoginError::DatabaseError("Missing user_id".to_string()))?;

        let session = self
            .session_manager
            .create_session(
                user_id,
                connection_id,
                client_ip,
                request.client_version.as_deref(),
            )
            .await
            .map_err(|_| LoginError::SessionCreationFailed)?;

        let session_id_clone = session.session_id.clone();
        Ok(LoginResponse {
            success: true,
            session_id: Some(session_id_clone.clone()),
            user_info: Some(user.clone()),
            server_time: Utc::now().to_rfc3339(),
            message: "登录成功".to_string(),
            // 前端兼容字段
            access_token: Some(session_id_clone.clone()),
            token: Some(session_id_clone.clone()),
            refresh_token: Some(session_id_clone),
            user: Some(user),
        })
    }

    /// 处理登出请求
    pub async fn handle_logout(&self, session_id: &str) -> Result<LoginResponse, LoginError> {
        self.session_manager
            .logout_session(session_id)
            .await
            .map_err(|e| LoginError::DatabaseError(e.to_string()))?;

        Ok(LoginResponse {
            success: true,
            session_id: None,
            user_info: None,
            server_time: Utc::now().to_rfc3339(),
            message: "登出成功".to_string(),
            access_token: None,
            token: None,
            refresh_token: None,
            user: None,
        })
    }

    /// 处理心跳请求
    pub async fn handle_heartbeat(&self, session_id: &str) -> Result<LoginResponse, LoginError> {
        // 验证会话
        let is_valid = self
            .session_manager
            .is_session_valid(session_id)
            .await
            .map_err(|e| LoginError::DatabaseError(e.to_string()))?;

        if !is_valid {
            return Ok(LoginResponse {
                success: false,
                session_id: None,
                user_info: None,
                server_time: Utc::now().to_rfc3339(),
                message: "会话已过期或无效".to_string(),
                access_token: None,
                token: None,
                refresh_token: None,
                user: None,
            });
        }

        // 更新心跳时间
        self.session_manager
            .update_heartbeat(session_id)
            .await
            .map_err(|e| LoginError::DatabaseError(e.to_string()))?;

        Ok(LoginResponse {
            success: true,
            session_id: Some(session_id.to_string()),
            user_info: None,
            server_time: Utc::now().to_rfc3339(),
            message: "心跳更新成功".to_string(),
            access_token: None,
            token: None,
            refresh_token: None,
            user: None,
        })
    }

    /// 获取会话信息
    pub async fn get_session_info(
        &self,
        session_id: &str,
    ) -> Result<Option<LoginResponse>, LoginError> {
        let session = self
            .session_manager
            .get_session(session_id)
            .await
            .map_err(|e| LoginError::DatabaseError(e.to_string()))?;

        match session {
            Some(s) => {
                let user = self
                    .db
                    .query_user(&s.user_id)
                    .await
                    .map_err(|e| LoginError::DatabaseError(e.to_string()))?;

                Ok(Some(LoginResponse {
                    success: true,
                    session_id: Some(s.session_id.clone()),
                    user_info: user.clone(),
                    server_time: Utc::now().to_rfc3339(),
                    message: "获取会话信息成功".to_string(),
                    access_token: Some(s.session_id.clone()),
                    token: Some(s.session_id.clone()),
                    refresh_token: Some(s.session_id),
                    user,
                }))
            }
            None => Ok(None),
        }
    }
}
