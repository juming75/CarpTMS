//! / 用户仓库PostgreSQL实现

use sqlx::{PgPool, Row};
use std::sync::Arc;

use crate::domain::entities::user::{User, UserCreate, UserQuery, UserUpdate};
use crate::domain::use_cases::user::UserRepository;

/// 用户仓库PostgreSQL实现
pub struct PgUserRepository {
    pool: Arc<PgPool>,
}

impl PgUserRepository {
    /// 创建用户仓库实例
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl UserRepository for PgUserRepository {
    /// 获取用户列表
    async fn get_users(&self, query: UserQuery) -> Result<(Vec<User>, i64), anyhow::Error> {
        // 处理分页参数
        let page = query.page.unwrap_or(1);
        let page_size = query.page_size.unwrap_or(20);
        let offset = (page - 1) * page_size;

        // 构建动态查询条件
        let mut where_clauses = Vec::new();
        let mut params = Vec::new();
        let mut param_index = 1;

        if let Some(user_name) = &query.user_name {
            if !user_name.is_empty() {
                where_clauses.push(format!("user_name LIKE ${}", param_index));
                params.push(format!("%{}", user_name));
                param_index += 1;
            }
        }

        if let Some(full_name) = &query.full_name {
            if !full_name.is_empty() {
                where_clauses.push(format!("real_name LIKE ${}", param_index));
                params.push(format!("%{}", full_name));
                param_index += 1;
            }
        }

        if let Some(status) = query.status {
            where_clauses.push(format!("status = ${}", param_index));
            params.push(status.to_string());
            param_index += 1;
        }

        if let Some(user_group_id) = query.user_group_id {
            where_clauses.push(format!("user_group_id = ${}", param_index));
            params.push(user_group_id.to_string());
            param_index += 1;
        }

        // 构建完整查询
        let where_sql = if where_clauses.is_empty() {
            "".to_string()
        } else {
            format!("WHERE {}", where_clauses.join(" AND "))
        };

        // 查询总记录数
        let count_query = format!("SELECT COUNT(*) FROM users {}", where_sql);
        let mut count_sqlx_query = sqlx::query_scalar::<_, i64>(&count_query);

        // 绑定参数
        for param in &params {
            count_sqlx_query = count_sqlx_query.bind(param);
        }

        let total_count = count_sqlx_query.fetch_one(&*self.pool).await?;

        // 查询分页数据
        let data_query = format!(
            "SELECT * FROM users {}
             ORDER BY user_id DESC
             LIMIT ${} OFFSET ${}",
            where_sql,
            param_index,
            param_index + 1
        );

        // 使用query方法执行动态查询
        let mut sqlx_query = sqlx::query(&data_query);

        // 绑定参数
        for param in params {
            sqlx_query = sqlx_query.bind(param);
        }
        sqlx_query = sqlx_query.bind(page_size).bind(offset);

        let users = sqlx_query
            .fetch_all(&*self.pool)
            .await?
            .into_iter()
            .map(|row| User {
                user_id: row.try_get("user_id").unwrap_or(0),
                user_name: row.try_get("user_name").unwrap_or_default(),
                password: row.try_get("password").unwrap_or_default(),
                full_name: row.try_get("real_name").unwrap_or_default(),
                phone_number: row.try_get("phone").ok(),
                email: row.try_get("email").ok(),
                user_group_id: row.try_get("user_group_id").unwrap_or(1),
                status: row.try_get("status").unwrap_or(1),
                last_login_time: row.try_get("last_login_time").ok(),
                create_time: row
                    .try_get("create_time")
                    .unwrap_or_else(|_| chrono::Utc::now().naive_utc()),
                update_time: row.try_get("update_time").ok(),
            })
            .collect::<Vec<User>>();

        Ok((users, total_count))
    }

    async fn get_user(&self, user_id: i32) -> Result<Option<User>, anyhow::Error> {
        let user = sqlx::query(r#"SELECT * FROM users WHERE user_id = $1"#)
            .bind(user_id)
            .fetch_optional(&*self.pool)
            .await?
            .map(|row| User {
                user_id: row.try_get("user_id").unwrap_or(0),
                user_name: row.try_get("user_name").unwrap_or_default(),
                password: row.try_get("password").unwrap_or_default(),
                full_name: row.try_get("real_name").unwrap_or_default(),
                phone_number: row.try_get("phone").ok(),
                email: row.try_get("email").ok(),
                user_group_id: row.try_get("user_group_id").unwrap_or(1),
                status: row.try_get("status").unwrap_or(1),
                last_login_time: row.try_get("last_login_time").ok(),
                create_time: row
                    .try_get("create_time")
                    .unwrap_or_else(|_| chrono::Utc::now().naive_utc()),
                update_time: row.try_get("update_time").ok(),
            });

        Ok(user)
    }

    async fn get_user_by_name(&self, user_name: &str) -> Result<Option<User>, anyhow::Error> {
        let user = sqlx::query(r#"SELECT * FROM users WHERE user_name = $1"#)
            .bind(user_name)
            .fetch_optional(&*self.pool)
            .await?
            .map(|row| User {
                user_id: row.try_get("user_id").unwrap_or(0),
                user_name: row.try_get("user_name").unwrap_or_default(),
                password: row.try_get("password").unwrap_or_default(),
                full_name: row.try_get("real_name").unwrap_or_default(),
                phone_number: row.try_get("phone").ok(),
                email: row.try_get("email").ok(),
                user_group_id: row.try_get("user_group_id").unwrap_or(1),
                status: row.try_get("status").unwrap_or(1),
                last_login_time: row.try_get("last_login_time").ok(),
                create_time: row
                    .try_get("create_time")
                    .unwrap_or_else(|_| chrono::Utc::now().naive_utc()),
                update_time: row.try_get("update_time").ok(),
            });

        Ok(user)
    }

    async fn create_user(&self, user: UserCreate) -> Result<User, anyhow::Error> {
        let result = sqlx::query(
            r#"INSERT INTO users ( 
                user_name, password, real_name, phone, 
                email, user_group_id, status, create_time 
            ) VALUES ( 
                $1, $2, $3, $4, $5, $6, $7, CURRENT_TIMESTAMP 
            ) RETURNING *"#,
        )
        .bind(&user.user_name)
        .bind(&user.password)
        .bind(&user.full_name)
        .bind(&user.phone_number)
        .bind(&user.email)
        .bind(user.user_group_id)
        .bind(user.status)
        .fetch_one(&*self.pool)
        .await;

        let u = match result {
            Ok(row) => User {
                user_id: row.try_get("user_id").unwrap_or(0),
                user_name: row.try_get("user_name").unwrap_or_default(),
                password: row.try_get("password").unwrap_or_default(),
                full_name: row.try_get("real_name").unwrap_or_default(),
                phone_number: row.try_get("phone").ok(),
                email: row.try_get("email").ok(),
                user_group_id: row.try_get("user_group_id").unwrap_or(1),
                status: row.try_get("status").unwrap_or(1),
                last_login_time: row.try_get("last_login_time").ok(),
                create_time: row
                    .try_get("create_time")
                    .unwrap_or_else(|_| chrono::Utc::now().naive_utc()),
                update_time: row.try_get("update_time").ok(),
            },
            Err(e) => {
                return Err(anyhow::anyhow!("Failed to create user: {:?}", e));
            }
        };

        Ok(u)
    }

    async fn update_user(
        &self,
        user_id: i32,
        user: UserUpdate,
    ) -> Result<Option<User>, anyhow::Error> {
        let result = sqlx::query(
            r#"UPDATE users 
               SET 
                   password = COALESCE($1, password),
                   real_name = COALESCE($2, real_name),
                   phone = COALESCE($3, phone),
                   email = COALESCE($4, email),
                   user_group_id = COALESCE($5, user_group_id),
                   status = COALESCE($6, status),
                   last_login_time = COALESCE($7, last_login_time),
                   update_time = CURRENT_TIMESTAMP 
               WHERE user_id = $8 
               RETURNING *"#,
        )
        .bind(&user.password)
        .bind(&user.full_name)
        .bind(&user.phone_number)
        .bind(&user.email)
        .bind(user.user_group_id)
        .bind(user.status)
        .bind(user.last_login_time)
        .bind(user_id)
        .fetch_optional(&*self.pool)
        .await;

        match result {
            Ok(Some(row)) => {
                let user = User {
                    user_id: row.try_get("user_id").unwrap_or(0),
                    user_name: row.try_get("user_name").unwrap_or_default(),
                    password: row.try_get("password").unwrap_or_default(),
                    full_name: row.try_get("real_name").unwrap_or_default(),
                    phone_number: row.try_get("phone").ok(),
                    email: row.try_get("email").ok(),
                    user_group_id: row.try_get("user_group_id").unwrap_or(1),
                    status: row.try_get("status").unwrap_or(1),
                    last_login_time: row.try_get("last_login_time").ok(),
                    create_time: row
                        .try_get("create_time")
                        .unwrap_or_else(|_| chrono::Utc::now().naive_utc()),
                    update_time: row.try_get("update_time").ok(),
                };

                Ok(Some(user))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(anyhow::anyhow!("Failed to update user: {:?}", e)),
        }
    }

    /// 删除用户
    async fn delete_user(&self, user_id: i32) -> Result<bool, anyhow::Error> {
        let result = sqlx::query(r#"DELETE FROM users WHERE user_id = $1"#)
            .bind(user_id)
            .execute(&*self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }
}
