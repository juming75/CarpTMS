//! 认证仓库实现

use std::sync::Arc;

use sqlx::PgPool;

use crate::domain::entities::auth::{UserAuth, UserInfo};
use crate::domain::use_cases::auth::AuthRepository;
use crate::errors::{AppError, AppResult};

/// 认证仓库实现
#[derive(Clone)]
pub struct AuthRepositoryImpl {
    db: Arc<PgPool>,
}

impl AuthRepositoryImpl {
    /// 创建认证仓库实例
    pub fn new(db: Arc<PgPool>) -> Self {
        Self {
            db,
        }
    }
}

#[async_trait::async_trait]
impl AuthRepository for AuthRepositoryImpl {
    async fn get_user_by_username(&self, username: &str) -> AppResult<Option<UserAuth>> {
        let user = sqlx::query_as::<_, UserAuth>(
            r#"SELECT 
                user_id, 
                user_name as username, 
                password as password_hash, 
                user_group_id 
            FROM users 
            WHERE user_name = $1"#
        )
        .bind(username)
        .fetch_optional(&*self.db)
        .await
        .map_err(|e| AppError::db_error("Failed to get user by username", Some(&e.to_string())))?;

        Ok(user)
    }

    async fn get_user_by_id(&self, user_id: i32) -> AppResult<Option<UserInfo>> {
        let user = sqlx::query_as::<_, UserInfo>(
            r#"SELECT 
                u.user_id, 
                u.user_name as username, 
                NULL as real_name, 
                g.group_name as role, 
                u.user_group_id as group_id, 
                u.create_time, 
                u.update_time 
            FROM users u 
            LEFT JOIN user_groups g ON u.user_group_id = g.group_id 
            WHERE u.user_id = $1"#
        )
        .bind(user_id)
        .fetch_optional(&*self.db)
        .await
        .map_err(|e| AppError::db_error("Failed to get user by id", Some(&e.to_string())))?;

        Ok(user)
    }

    async fn get_user_role(&self, user_id: i32) -> AppResult<(String, i32)> {
        // 从users表获取user_group_id
        let user_group_id = sqlx::query_scalar::<_, i32>(
            "SELECT user_group_id FROM users WHERE user_id = $1"
        )
        .bind(user_id)
        .fetch_one(&*self.db)
        .await
        .map_err(|e| AppError::db_error("Failed to get user group id", Some(&e.to_string())))?;
        
        // 从user_groups表获取group_name
        let role = sqlx::query_scalar::<_, String>(
            "SELECT group_name FROM user_groups WHERE group_id = $1"
        )
        .bind(user_group_id)
        .fetch_one(&*self.db)
        .await
        .map_err(|e| AppError::db_error("Failed to get user role", Some(&e.to_string())))?;
        
        Ok((role, user_group_id))
    }
}
