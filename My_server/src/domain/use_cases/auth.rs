//! 认证用例

use std::sync::Arc;

use crate::domain::entities::auth::{
    LoginRequest, PasswordChangeRequest, RefreshTokenRequest, TokenResponse, UserAuth, UserInfo,
};
use crate::errors::{AppError, AppResult};
use bcrypt;
use tracing::info;

/// 认证仓库接口
#[async_trait::async_trait]
pub trait AuthRepository: Send + Sync {
    /// 根据用户名获取用户
    async fn get_user_by_username(&self, username: &str) -> AppResult<Option<UserAuth>>;

    /// 根据ID获取用户信息
    async fn get_user_by_id(&self, user_id: i32) -> AppResult<Option<UserInfo>>;

    /// 获取用户角色
    async fn get_user_role(&self, user_id: i32) -> AppResult<(String, i32)>;

    /// 更新用户密码
    async fn update_password(&self, user_id: i32, new_password_hash: &str) -> AppResult<()>;
}

/// 认证用例
#[derive(Clone)]
pub struct AuthUseCases {
    repository: Arc<dyn AuthRepository + Send + Sync>,
}

impl AuthUseCases {
    /// 创建认证用例
    pub fn new(repository: Arc<dyn AuthRepository + Send + Sync>) -> Self {
        Self { repository }
    }

    /// 登录
    pub async fn login(&self, request: LoginRequest) -> AppResult<TokenResponse> {
        info!("Attempting to get user by username: {}", request.username);

        // 从仓库获取用户
        let user = self
            .repository
            .get_user_by_username(&request.username)
            .await?
            .ok_or_else(|| AppError::permission_error("User not found"))?;

        info!(
            "User found: user_id={}, username={}, password_hash={}",
            user.user_id, user.username, user.password_hash
        );

        // 验证密码
        let is_valid = if user.password_hash.starts_with("$argon2id$") {
            // 验证 Argon2 哈希密码
            crate::utils::password::verify_password(&request.password, &user.password_hash)?
        } else if user.password_hash.starts_with("$2a$")
            || user.password_hash.starts_with("$2b$")
            || user.password_hash.starts_with("$2y$")
        {
            // 验证 bcrypt 哈希密码
            bcrypt::verify(&request.password, &user.password_hash).map_err(|e| {
                AppError::internal_error("bcrypt verification failed", Some(&e.to_string()))
            })?
        } else {
            // 验证明文密码（用于开发环境）
            info!(
                "Checking plaintext password: input={}, stored={}",
                request.password, user.password_hash
            );
            request.password == user.password_hash
        };

        info!("Password validation result: {}", is_valid);

        if !is_valid {
            return Err(AppError::permission_error("Invalid password"));
        }

        // 获取用户角色
        info!("Getting user role for user_id: {}", user.user_id);
        let (role, group_id) = self.repository.get_user_role(user.user_id).await?;

        info!("User role found: role={}, group_id={}", role, group_id);

        // 生成令牌
        info!("Generating access token");
        let access_token = crate::utils::jwt::generate_access_token(user.user_id, &role, group_id)
            .map_err(|e| {
                AppError::internal_error("Failed to generate access token", Some(&e.to_string()))
            })?;

        info!("Generating refresh token");
        let refresh_token =
            crate::utils::jwt::generate_refresh_token(user.user_id).map_err(|e| {
                AppError::internal_error("Failed to generate refresh token", Some(&e.to_string()))
            })?;

        // 检查是否需要修改密码（password_changed_at 为 NULL 表示首次登录或未修改过）
        let password_required = user.password_changed_at.is_none();
        log::info!("Password change required: {}", password_required);

        log::info!("Login successful");
        Ok(TokenResponse {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: 3600,
            password_required,
        })
    }

    /// 刷新令牌
    pub async fn refresh_token(&self, request: RefreshTokenRequest) -> AppResult<TokenResponse> {
        // 验证刷新令牌
        let claims = crate::utils::jwt::verify_refresh_token(&request.refresh_token)
            .map_err(|e| AppError::permission_error(&e.to_string()))?;

        let user_id = claims
            .sub
            .parse::<i32>()
            .map_err(|e| AppError::validation(&e.to_string()))?;

        // 获取用户角色
        let (role, group_id) = self.repository.get_user_role(user_id).await?;

        let user_info = self.repository.get_user_by_id(user_id).await?;
        let password_required = match user_info {
            Some(info) => {
                let user_auth = self.repository.get_user_by_username(&info.username).await?;
                user_auth
                    .map(|u| u.password_changed_at.is_none())
                    .unwrap_or(false)
            }
            None => false,
        };

        // 生成新令牌
        let access_token = crate::utils::jwt::generate_access_token(user_id, &role, group_id)
            .map_err(|e| {
                AppError::internal_error("Failed to generate access token", Some(&e.to_string()))
            })?;

        let refresh_token = crate::utils::jwt::generate_refresh_token(user_id).map_err(|e| {
            AppError::internal_error("Failed to generate refresh token", Some(&e.to_string()))
        })?;

        Ok(TokenResponse {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: 3600,
            password_required,
        })
    }

    /// 修改密码
    pub async fn change_password(
        &self,
        user_id: i32,
        request: PasswordChangeRequest,
    ) -> AppResult<()> {
        log::info!("Changing password for user_id: {}", user_id);

        // 获取当前用户
        let user = self
            .repository
            .get_user_by_id(user_id)
            .await?
            .ok_or_else(|| AppError::resource_not_found("User not found"))?;

        // 验证旧密码
        let user_auth = self
            .repository
            .get_user_by_username(&user.username)
            .await?
            .ok_or_else(|| AppError::resource_not_found("User not found"))?;

        // 验证旧密码是否正确
        let is_valid = if user_auth.password_hash.starts_with("$argon2id$") {
            crate::utils::password::verify_password(
                &request.old_password,
                &user_auth.password_hash,
            )?
        } else if user_auth.password_hash.starts_with("$2a$")
            || user_auth.password_hash.starts_with("$2b$")
            || user_auth.password_hash.starts_with("$2y$")
        {
            bcrypt::verify(&request.old_password, &user_auth.password_hash).map_err(|e| {
                AppError::internal_error("bcrypt verification failed", Some(&e.to_string()))
            })?
        } else {
            request.old_password == user_auth.password_hash
        };

        if !is_valid {
            return Err(AppError::permission_error("旧密码不正确"));
        }

        // 验证新密码强度
        let strength = crate::utils::password::validate_password_strength(&request.new_password);
        if !strength.is_valid {
            let error_msg = strength.errors.join("; ");
            return Err(AppError::validation(&error_msg));
        }

        // 不能与旧密码相同
        if request.old_password == request.new_password {
            return Err(AppError::validation("新密码不能与旧密码相同"));
        }

        // 生成新密码哈希
        let new_hash =
            crate::utils::password::hash_password(&request.new_password).map_err(|e| {
                AppError::internal_error("Failed to hash password", Some(&e.to_string()))
            })?;

        // 更新密码
        self.repository.update_password(user_id, &new_hash).await?;

        log::info!("Password changed successfully for user_id: {}", user_id);
        Ok(())
    }

    /// 获取用户信息
    pub async fn get_user_info(&self, user_id: i32) -> AppResult<UserInfo> {
        self.repository
            .get_user_by_id(user_id)
            .await?
            .ok_or_else(|| AppError::resource_not_found("User not found"))
    }
}

/// 应用服务接口实现
#[async_trait::async_trait]
impl crate::domain::use_cases::application_service::ApplicationService for AuthUseCases {
    fn name(&self) -> &str {
        "auth_service"
    }

    fn initialize(&self) -> anyhow::Result<()> {
        // 初始化逻辑（如果需要）
        Ok(())
    }

    async fn execute(&self, _input: serde_json::Value) -> anyhow::Result<serde_json::Value> {
        // 通用执行方法（如果需要）
        Ok(serde_json::Value::Null)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use mockall::mock;

    mock! {
        pub AuthRepositoryImpl {}
        #[async_trait::async_trait]
        impl AuthRepository for AuthRepositoryImpl {
            async fn get_user_by_username(&self, username: &str) -> AppResult<Option<UserAuth>>;
            async fn get_user_by_id(&self, user_id: i32) -> AppResult<Option<UserInfo>>;
            async fn get_user_role(&self, user_id: i32) -> AppResult<(String, i32)>;
            async fn update_password(&self, user_id: i32, new_password_hash: &str) -> AppResult<()>;
        }
    }

    #[tokio::test]
    async fn test_login() -> Result<(), anyhow::Error> {
        let mut mock_repo = MockAuthRepositoryImpl::new();

        let request = LoginRequest {
            username: "admin".to_string(),
            password: "password".to_string(),
        };

        // 用户已修改过密码，不需要强制修改
        let user_auth = UserAuth {
            user_id: 1,
            username: "admin".to_string(),
            password_hash: "$argon2id$v=19$m=19456,t=2,p=1$Ne5P0SURb5RPBbGtEc6opw$fVIF2d6MrHLXn4t71m3KitDin3/8YTJ7ZZsYvEEXw7w".to_string(),
            user_group_id: 1,
            password_changed_at: Some(Utc::now().naive_utc()), // 已修改过密码
        };

        mock_repo
            .expect_get_user_by_username()
            .returning(move |_| Ok(Some(user_auth.clone())));

        mock_repo
            .expect_get_user_role()
            .returning(|_| Ok(("admin".to_string(), 1)));

        let use_cases = AuthUseCases::new(Arc::new(mock_repo));
        let result = use_cases.login(request).await;

        assert!(result.is_ok());
        let token_response = result.unwrap();
        assert!(!token_response.access_token.is_empty());
        assert!(!token_response.refresh_token.is_empty());
        assert!(!token_response.password_required); // 不需要修改密码

        Ok(())
    }

    #[tokio::test]
    async fn test_login_password_required() -> Result<(), anyhow::Error> {
        let mut mock_repo = MockAuthRepositoryImpl::new();

        let request = LoginRequest {
            username: "admin".to_string(),
            password: "admin123".to_string(),
        };

        // 用户首次登录，password_changed_at 为 None，需要强制修改密码
        let user_auth = UserAuth {
            user_id: 1,
            username: "admin".to_string(),
            password_hash: "$argon2id$v=19$m=19456,t=2,p=1$Ne5P0SURb5RPBbGtEc6opw$fVIF2d6MrHLXn4t71m3KitDin3/8YTJ7ZZsYvEEXw7w".to_string(),
            user_group_id: 1,
            password_changed_at: None, // 首次登录，从未修改过密码
        };

        mock_repo
            .expect_get_user_by_username()
            .returning(move |_| Ok(Some(user_auth.clone())));

        mock_repo
            .expect_get_user_role()
            .returning(|_| Ok(("admin".to_string(), 1)));

        let use_cases = AuthUseCases::new(Arc::new(mock_repo));
        let result = use_cases.login(request).await;

        assert!(result.is_ok());
        let token_response = result.unwrap();
        assert!(!token_response.access_token.is_empty());
        assert!(token_response.password_required); // 需要修改密码

        Ok(())
    }
}
