//! OpenAPI 平台仓库实现

use std::sync::Arc;

use async_trait::async_trait;
use sqlx::PgPool;

use crate::domain::entities::openapi_platform::{OpenapiPlatform, OpenapiPlatformUpdateRequest, OpenapiPlatformQuery};
use crate::domain::use_cases::openapi_platform::OpenapiPlatformRepository;
use crate::errors::{AppError, AppResult};

/// OpenAPI 平台仓库实现
#[derive(Clone)]
pub struct OpenapiPlatformRepositoryImpl {
    db: Arc<PgPool>,
}

impl OpenapiPlatformRepositoryImpl {
    /// 创建 OpenAPI 平台仓库实例
    pub fn new(db: Arc<PgPool>) -> Self {
        Self {
            db,
        }
    }
}

#[async_trait]
impl OpenapiPlatformRepository for OpenapiPlatformRepositoryImpl {
    async fn create(&self, platform: &OpenapiPlatform) -> AppResult<OpenapiPlatform> {
        let result = sqlx::query_as::<_, OpenapiPlatform>(
            r#"INSERT INTO openapi_platforms (
                name, 
                url, 
                api_key, 
                status, 
                created_at, 
                updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *"#
        )
        .bind(&platform.name)
        .bind(&platform.url)
        .bind(&platform.api_key)
        .bind(&platform.status)
        .bind(platform.created_at)
        .bind(platform.updated_at)
        .fetch_one(&*self.db)
        .await
        .map_err(|e| AppError::db_error("Failed to create openapi platform", Some(&e.to_string())))?;

        Ok(result)
    }

    async fn get_by_id(&self, id: i32) -> AppResult<Option<OpenapiPlatform>> {
        let platform = sqlx::query_as::<_, OpenapiPlatform>(
            r#"SELECT 
                id, 
                name, 
                url, 
                api_key, 
                status, 
                created_at, 
                updated_at
            FROM openapi_platforms 
            WHERE id = $1"#
        )
        .bind(id)
        .fetch_optional(&*self.db)
        .await
        .map_err(|e| AppError::db_error("Failed to get openapi platform by id", Some(&e.to_string())))?;

        Ok(platform)
    }

    async fn get_all(&self, query: &OpenapiPlatformQuery) -> AppResult<(Vec<OpenapiPlatform>, i64)> {
        let page = query.page.unwrap_or(1);
        let page_size = query.page_size.unwrap_or(10);
        let offset = (page - 1) * page_size;

        // 构建查询条件
        let mut where_clauses = vec![];
        let mut params = vec![];

        if let Some(name) = &query.name {
            let param_idx = params.len() + 1;
            where_clauses.push(format!("name LIKE ${}", param_idx));
            params.push(format!("%{}%", name));
        }

        if let Some(status) = &query.status {
            let param_idx = params.len() + 1;
            where_clauses.push(format!("status = ${}", param_idx));
            params.push(status.clone());
        }

        let where_clause = if where_clauses.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", where_clauses.join(" AND "))
        };

        // 构建计数查询
        let count_query = format!(
            "SELECT COUNT(*) FROM openapi_platforms {}",
            where_clause
        );

        // 执行计数查询
        let mut count_query_builder = sqlx::query_scalar(&count_query);
        for param in &params {
            count_query_builder = count_query_builder.bind(param);
        }
        let total: i64 = count_query_builder
            .fetch_one(&*self.db)
            .await
            .map_err(|e| AppError::db_error("Failed to count openapi platforms", Some(&e.to_string())))?;

        // 构建分页查询
        let param_count = params.len();
        let limit_param = param_count + 1;
        let offset_param = limit_param + 1;
        
        let pagination_query = format!(
            r#"SELECT 
                id, 
                name, 
                url, 
                api_key, 
                status, 
                created_at, 
                updated_at
            FROM openapi_platforms 
            {} 
            ORDER BY created_at DESC 
            LIMIT ${} OFFSET ${}"#,
            where_clause,
            limit_param,
            offset_param
        );

        // 执行分页查询
        let mut query_builder = sqlx::query_as::<_, OpenapiPlatform>(&pagination_query);
        
        // 先绑定过滤参数
        for param in &params {
            query_builder = query_builder.bind(param);
        }
        
        // 再绑定分页参数
        query_builder = query_builder.bind(page_size);
        query_builder = query_builder.bind(offset);

        let platforms = query_builder
            .fetch_all(&*self.db)
            .await
            .map_err(|e| AppError::db_error("Failed to get openapi platforms", Some(&e.to_string())))?;

        Ok((platforms, total))
    }

    async fn update(&self, id: i32, platform: &OpenapiPlatformUpdateRequest) -> AppResult<OpenapiPlatform> {
        let result = sqlx::query_as::<_, OpenapiPlatform>(
            r#"UPDATE openapi_platforms 
            SET name = COALESCE($1, name), 
                url = COALESCE($2, url), 
                api_key = COALESCE($3, api_key), 
                status = COALESCE($4, status), 
                updated_at = $5 
            WHERE id = $6 
            RETURNING *"#
        )
        .bind(platform.name.clone())
        .bind(platform.url.clone())
        .bind(platform.api_key.clone())
        .bind(platform.status.clone())
        .bind(chrono::Utc::now())
        .bind(id)
        .fetch_one(&*self.db)
        .await
        .map_err(|e| AppError::db_error("Failed to update openapi platform", Some(&e.to_string())))?;

        Ok(result)
    }

    async fn update_status(&self, id: i32, status: &str) -> AppResult<OpenapiPlatform> {
        let result = sqlx::query_as::<_, OpenapiPlatform>(
            r#"UPDATE openapi_platforms 
            SET status = $1, updated_at = $2 
            WHERE id = $3 
            RETURNING *"#
        )
        .bind(status)
        .bind(chrono::Utc::now())
        .bind(id)
        .fetch_one(&*self.db)
        .await
        .map_err(|e| AppError::db_error("Failed to update openapi platform status", Some(&e.to_string())))?;

        Ok(result)
    }

    async fn delete(&self, id: i32) -> AppResult<bool> {
        let result = sqlx::query(
            "DELETE FROM openapi_platforms WHERE id = $1"
        )
        .bind(id)
        .execute(&*self.db)
        .await
        .map_err(|e| AppError::db_error("Failed to delete openapi platform", Some(&e.to_string())))?;

        Ok(result.rows_affected() > 0)
    }
}
