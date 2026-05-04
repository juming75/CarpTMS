//! 认证仓库实现

use std::sync::Arc;

use chrono::Utc;
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
        Self { db }
    }

    /// 更新用户密码
    pub async fn update_password(&self, user_id: i32, new_password_hash: &str) -> AppResult<()> {
        sqlx::query(
            r#"UPDATE users
               SET password = $1,
                   update_time = $2,
                   password_changed_at = $3
               WHERE user_id = $4"#,
        )
        .bind(new_password_hash)
        .bind(Utc::now().naive_utc())
        .bind(Utc::now().naive_utc())
        .bind(user_id)
        .execute(&*self.db)
        .await
        .map_err(|e| AppError::db_error("Failed to update password", Some(&e.to_string())))?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl AuthRepository for AuthRepositoryImpl {
    async fn get_user_by_username(&self, username: &str) -> AppResult<Option<UserAuth>> {
        let user = sqlx::query_as::<_, UserAuth>(
            r#"SELECT
                user_id,
                user_name AS username,
                password AS password_hash,
                COALESCE(user_group_id, 0) AS user_group_id,
                password_changed_at
            FROM users
            WHERE user_name = $1"#,
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
                u.user_name AS username,
                u.real_name,
                COALESCE(g.group_name, '普通用户') AS role,
                COALESCE(u.user_group_id, 0) AS group_id,
                u.create_time,
                u.update_time
            FROM users u
            LEFT JOIN user_groups g ON u.user_group_id = g.group_id
            WHERE u.user_id = $1"#,
        )
        .bind(user_id)
        .fetch_optional(&*self.db)
        .await
        .map_err(|e| AppError::db_error("Failed to get user by id", Some(&e.to_string())))?;

        Ok(user)
    }

    async fn get_user_role(&self, user_id: i32) -> AppResult<(String, i32)> {
        let user_group_id: i32 =
            sqlx::query_scalar::<_, Option<i32>>("SELECT user_group_id FROM users WHERE user_id = $1")
                .bind(user_id)
                .fetch_one(&*self.db)
                .await
                .map_err(|e| {
                    AppError::db_error("Failed to get user group id", Some(&e.to_string()))
                })?
                .unwrap_or(0);

        let role = if user_group_id > 0 {
            sqlx::query_scalar::<_, String>(
                "SELECT group_name FROM user_groups WHERE group_id = $1",
            )
            .bind(user_group_id)
            .fetch_one(&*self.db)
            .await
            .map_err(|e| AppError::db_error("Failed to get user role", Some(&e.to_string())))?
        } else {
            "普通用户".to_string()
        };

        Ok((role, user_group_id))
    }

    async fn update_password(&self, user_id: i32, new_password_hash: &str) -> AppResult<()> {
        sqlx::query(
            r#"UPDATE users
               SET password = $1,
                   update_time = $2,
                   password_changed_at = $3
               WHERE user_id = $4"#,
        )
        .bind(new_password_hash)
        .bind(Utc::now().naive_utc())
        .bind(Utc::now().naive_utc())
        .bind(user_id)
        .execute(&*self.db)
        .await
        .map_err(|e| AppError::db_error("Failed to update password", Some(&e.to_string())))?;

        Ok(())
    }
}
