//! 认证用例

use std::sync::Arc;

use crate::domain::entities::auth::{LoginRequest, RefreshTokenRequest, TokenResponse, UserInfo, UserAuth};
use crate::errors::{AppError, AppResult};

/// 认证仓库接口
#[async_trait::async_trait]
pub trait AuthRepository: Send + Sync {
    /// 根据用户名获取用户
    async fn get_user_by_username(&self, username: &str) -> AppResult<Option<UserAuth>>;
    
    /// 根据ID获取用户信息
    async fn get_user_by_id(&self, user_id: i32) -> AppResult<Option<UserInfo>>;
    
    /// 获取用户角色
    async fn get_user_role(&self, user_id: i32) -> AppResult<(String, i32)>;
}

/// 认证用例
#[derive(Clone)]
pub struct AuthUseCases {
    repository: Arc<dyn AuthRepository + Send + Sync>,
}

impl AuthUseCases {
    /// 创建认证用例
    pub fn new(repository: Arc<dyn AuthRepository + Send + Sync>) -> Self {
        Self {
            repository,
        }
    }
    
    /// 登录
    pub async fn login(&self, request: LoginRequest) -> AppResult<TokenResponse> {
        log::info!("Attempting to get user by username: {}", request.username);
        
        // 从仓库获取用户
        let user = self.repository.get_user_by_username(&request.username)
            .await?
            .ok_or_else(|| AppError::permission_error("User not found"))?;
        
        log::info!("User found: user_id={}, username={}", user.user_id, user.username);
        
        // 验证密码
        let is_valid = if user.password_hash.starts_with("$argon2id$") {
            // 验证哈希密码
            crate::utils::password::verify_password(&request.password, &user.password_hash)?
        } else {
            // 验证明文密码（用于开发环境）
            request.password == user.password_hash
        };
        
        log::info!("Password validation result: {}", is_valid);
        
        if !is_valid {
            return Err(AppError::permission_error("Invalid password"));
        }
        
        // 获取用户角色
        log::info!("Getting user role for user_id: {}", user.user_id);
        let (role, group_id) = self.repository.get_user_role(user.user_id)
            .await?;
        
        log::info!("User role found: role={}, group_id={}", role, group_id);
        
        // 生成令牌
        log::info!("Generating access token");
        let access_token = crate::utils::jwt::generate_access_token(user.user_id, &role, group_id)
            .map_err(|e| AppError::internal_error("Failed to generate access token", Some(&e.to_string())))?;
        
        log::info!("Generating refresh token");
        let refresh_token = crate::utils::jwt::generate_refresh_token(user.user_id)
            .map_err(|e| AppError::internal_error("Failed to generate refresh token", Some(&e.to_string())))?;
        
        log::info!("Login successful");
        Ok(TokenResponse {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: 3600,
        })
    }
    
    /// 刷新令牌
    pub async fn refresh_token(&self, request: RefreshTokenRequest) -> AppResult<TokenResponse> {
        // 验证刷新令牌
        let claims = crate::utils::jwt::verify_refresh_token(&request.refresh_token)
            .map_err(|e| AppError::permission_error(&e.to_string()))?;
        
        let user_id = claims.sub.parse::<i32>()
            .map_err(|e| AppError::validation(&e.to_string()))?;
        
        // 获取用户角色
        let (role, group_id) = self.repository.get_user_role(user_id)
            .await?;
        
        // 生成新令牌
        let access_token = crate::utils::jwt::generate_access_token(user_id, &role, group_id)
            .map_err(|e| AppError::internal_error("Failed to generate access token", Some(&e.to_string())))?;
        
        let refresh_token = crate::utils::jwt::generate_refresh_token(user_id)
            .map_err(|e| AppError::internal_error("Failed to generate refresh token", Some(&e.to_string())))?;
        
        Ok(TokenResponse {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: 3600,
        })
    }
    
    /// 获取用户信息
    pub async fn get_user_info(&self, user_id: i32) -> AppResult<UserInfo> {
        self.repository.get_user_by_id(user_id)
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
    use mockall::mock;
    
    mock! {
        pub AuthRepositoryImpl {}
        #[async_trait::async_trait]
        impl AuthRepository for AuthRepositoryImpl {
            async fn get_user_by_username(&self, username: &str) -> AppResult<Option<UserAuth>>;
            async fn get_user_by_id(&self, user_id: i32) -> AppResult<Option<UserInfo>>;
            async fn get_user_role(&self, user_id: i32) -> AppResult<(String, i32)>;
        }
    }
    
    #[tokio::test]
    async fn test_login() -> Result<(), anyhow::Error> {
        let mut mock_repo = MockAuthRepositoryImpl::new();
        
        let request = LoginRequest {
            username: "admin".to_string(),
            password: "password".to_string(),
        };
        
        let user_auth = UserAuth {
            user_id: 1,
            username: "admin".to_string(),
            password_hash: "$2b$10$EixZaYVK1fsbw1ZfbX3OXePaWxn96p36WQoeG6Lruj3vjPGga31lW".to_string(), // password
            user_group_id: 1,
        };
        
        mock_repo
            .expect_get_user_by_username()
            .returning(|_| Ok(Some(user_auth)));
        
        mock_repo
            .expect_get_user_role()
            .returning(|_| Ok(("admin".to_string(), 1)));
        
        let use_cases = AuthUseCases::new(Arc::new(mock_repo));
        let result = use_cases.login(request).await;
        
        assert!(result.is_ok());
        let token_response = result.ok_or_else(|| AppError::resource_not_found("Failed to login"))?;
        assert!(!token_response.access_token.is_empty());
        assert!(!token_response.refresh_token.is_empty());
        
        Ok(())
    }
}
