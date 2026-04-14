//! Roles Application Service
//!
//! Encapsulates all SQL for role/permission management.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use utoipa::ToSchema;

use crate::errors::{AppError, AppResult};

#[derive(Debug, Deserialize, ToSchema)]
pub struct RoleCreateRequest {
    pub role_name: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct RoleUpdateRequest {
    pub role_name: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct RoleResponse {
    pub role_id: i32,
    pub role_name: String,
    pub description: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

fn row_to_role_response(row: &sqlx::postgres::PgRow) -> RoleResponse {
    RoleResponse {
        role_id: row.get(0),
        role_name: row.get(1),
        description: None,
        created_at: row
            .get::<Option<chrono::NaiveDateTime>, _>(2)
            .map(|t| t.to_string())
            .unwrap_or_default(),
        updated_at: row
            .get::<Option<chrono::NaiveDateTime>, _>(3)
            .map(|t| t.to_string())
            .unwrap_or_default(),
    }
}

pub struct RoleApplicationService {
    pool: PgPool,
}

impl RoleApplicationService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_roles(&self) -> AppResult<Vec<RoleResponse>> {
        let rows = sqlx::query(
            "SELECT group_id, group_name, create_time, update_time FROM user_groups",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.iter().map(row_to_role_response).collect())
    }

    pub async fn get_role(&self, role_id: i32) -> AppResult<Option<RoleResponse>> {
        let row = sqlx::query(
            "SELECT group_id, group_name, create_time, update_time FROM user_groups WHERE group_id = $1",
        )
        .bind(role_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.as_ref().map(row_to_role_response))
    }

    pub async fn create_role(&self, request: RoleCreateRequest) -> AppResult<RoleResponse> {
        let now = Utc::now().naive_utc();
        let row = sqlx::query(
            "INSERT INTO user_groups (group_name, create_time, update_time)
             VALUES ($1, $2, $3)
             RETURNING group_id, group_name, create_time, update_time",
        )
        .bind(&request.role_name)
        .bind(now)
        .bind(now)
        .fetch_one(&self.pool)
        .await?;

        Ok(row_to_role_response(&row))
    }

    pub async fn update_role(
        &self,
        role_id: i32,
        request: RoleUpdateRequest,
    ) -> AppResult<RoleResponse> {
        let count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM user_groups WHERE group_id = $1")
                .bind(role_id)
                .fetch_one(&self.pool)
                .await?;

        if count == 0 {
            return Err(AppError::not_found_error("Role not found".to_string()));
        }

        let row = sqlx::query(
            "UPDATE user_groups
             SET group_name = COALESCE($1, group_name),
                 update_time = $2
             WHERE group_id = $3
             RETURNING group_id, group_name, create_time, update_time",
        )
        .bind(&request.role_name)
        .bind(Utc::now().naive_utc())
        .bind(role_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(row_to_role_response(&row))
    }

    pub async fn delete_role(&self, role_id: i32) -> AppResult<()> {
        let count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM user_groups WHERE group_id = $1")
                .bind(role_id)
                .fetch_one(&self.pool)
                .await?;

        if count == 0 {
            return Err(AppError::not_found_error("Role not found".to_string()));
        }

        let user_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE user_group_id = $1")
                .bind(role_id)
                .fetch_one(&self.pool)
                .await?;

        if user_count > 0 {
            return Err(AppError::business_error(
                "Cannot delete role with existing users",
                None,
            ));
        }

        sqlx::query("DELETE FROM user_groups WHERE group_id = $1")
            .bind(role_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}
