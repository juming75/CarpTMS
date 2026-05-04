//! 组织单位仓库实现

use std::sync::Arc;

use sqlx::PgPool;

use crate::domain::entities::organization::{
    Organization, OrganizationQuery, OrganizationUpdateRequest,
};
use crate::domain::use_cases::organization::OrganizationRepository;
use crate::errors::{AppError, AppResult};

/// 组织单位仓库实现
#[derive(Clone)]
pub struct OrganizationRepositoryImpl {
    db: Arc<PgPool>,
}

impl OrganizationRepositoryImpl {
    /// 创建组织单位仓库实例
    pub fn new(db: Arc<PgPool>) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl OrganizationRepository for OrganizationRepositoryImpl {
    async fn create(&self, organization: &Organization) -> AppResult<Organization> {
        let result = sqlx::query_as::<_, Organization>(
            r#"INSERT INTO organizations (
                unit_id, 
                name, 
                type, 
                parent_id, 
                description, 
                contact_person, 
                contact_phone, 
                status, 
                create_time, 
                update_time
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING *"#,
        )
        .bind(&organization.unit_id)
        .bind(&organization.name)
        .bind(&organization.r#type)
        .bind(organization.parent_id)
        .bind(&organization.description)
        .bind(&organization.contact_person)
        .bind(&organization.contact_phone)
        .bind(&organization.status)
        .bind(organization.create_time)
        .bind(organization.update_time)
        .fetch_one(&*self.db)
        .await
        .map_err(|e| AppError::db_error("Failed to create organization", Some(&e.to_string())))?;

        Ok(result)
    }

    async fn get_by_id(&self, unit_id: &str) -> AppResult<Option<Organization>> {
        let organization = sqlx::query_as::<_, Organization>(
            r#"SELECT 
                unit_id, 
                name, 
                type, 
                parent_id, 
                description, 
                contact_person, 
                contact_phone, 
                status, 
                create_time, 
                update_time
            FROM organizations 
            WHERE unit_id = $1"#,
        )
        .bind(unit_id)
        .fetch_optional(&*self.db)
        .await
        .map_err(|e| {
            AppError::db_error("Failed to get organization by id", Some(&e.to_string()))
        })?;

        Ok(organization)
    }

    async fn get_by_ids(&self, unit_ids: &[String]) -> AppResult<Vec<Organization>> {
        if unit_ids.is_empty() {
            return Ok(vec![]);
        }

        let placeholders = (1..=unit_ids.len())
            .map(|i| format!("${}", i))
            .collect::<Vec<_>>()
            .join(", ");

        let query = format!(
            r#"SELECT 
                unit_id, 
                name, 
                type, 
                parent_id, 
                description, 
                contact_person, 
                contact_phone, 
                status, 
                create_time, 
                update_time
            FROM organizations 
            WHERE unit_id IN ({})"#,
            placeholders
        );

        let mut query_builder = sqlx::query_as::<_, Organization>(&query);
        for unit_id in unit_ids {
            query_builder = query_builder.bind(unit_id);
        }

        let organizations = query_builder.fetch_all(&*self.db).await.map_err(|e| {
            AppError::db_error("Failed to get organizations by ids", Some(&e.to_string()))
        })?;

        Ok(organizations)
    }

    async fn get_all(&self, query: &OrganizationQuery) -> AppResult<(Vec<Organization>, i64)> {
        let page = query.page.unwrap_or(1);
        let page_size = query.page_size.unwrap_or(10);
        let offset = (page - 1) * page_size;

        log::info!(
            "Organization query: page={}, page_size={}, offset={}",
            page,
            page_size,
            offset
        );

        // 构建查询条件
        let mut where_clauses = vec![];
        let mut params = vec![];

        if let Some(name) = &query.name {
            let param_idx = params.len() + 1;
            where_clauses.push(format!("name LIKE ${}", param_idx));
            params.push(format!("%{}%", name));
        }

        if let Some(r#type) = &query.r#type {
            let param_idx = params.len() + 1;
            where_clauses.push(format!("type = ${}", param_idx));
            params.push(r#type.clone());
        }

        let where_clause = if where_clauses.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", where_clauses.join(" AND "))
        };

        // 构建计数查询
        let count_query = format!("SELECT COUNT(*) FROM organizations {}", where_clause);

        log::info!("Count query: {}", count_query);

        // 执行计数查询
        let mut count_query_builder = sqlx::query_scalar(&count_query);
        for param in &params {
            count_query_builder = count_query_builder.bind(param);
        }
        let total: i64 = count_query_builder
            .fetch_one(&*self.db)
            .await
            .map_err(|e| {
                log::error!("Count query failed: {:?}", e);
                AppError::db_error("Failed to count organizations", Some(&e.to_string()))
            })?;

        // 构建分页查询
        let param_count = params.len();
        let limit_param = param_count + 1;
        let offset_param = limit_param + 1;

        let pagination_query = format!(
            r#"SELECT 
                unit_id, 
                name, 
                type, 
                parent_id, 
                description, 
                contact_person, 
                contact_phone, 
                status, 
                create_time, 
                update_time
            FROM organizations 
            {} 
            ORDER BY create_time DESC 
            LIMIT ${} OFFSET ${}"#,
            where_clause, limit_param, offset_param
        );

        log::info!("Pagination query: {}", pagination_query);

        // 执行分页查询
        let mut query_builder = sqlx::query_as::<_, Organization>(&pagination_query);

        // 先绑定过滤参数
        for param in &params {
            query_builder = query_builder.bind(param);
        }

        // 再绑定分页参数
        query_builder = query_builder.bind(page_size);
        query_builder = query_builder.bind(offset);

        let organizations = query_builder.fetch_all(&*self.db).await.map_err(|e| {
            log::error!("Pagination query failed: {:?}", e);
            AppError::db_error("Failed to get organizations", Some(&e.to_string()))
        })?;

        log::info!(
            "Retrieved {} organizations, total={}",
            organizations.len(),
            total
        );

        Ok((organizations, total))
    }

    async fn update(
        &self,
        unit_id: &str,
        organization: &OrganizationUpdateRequest,
    ) -> AppResult<Organization> {
        let result = sqlx::query_as::<_, Organization>(
            r#"UPDATE organizations 
            SET name = COALESCE($1, name),
                type = COALESCE($2, type),
                parent_id = COALESCE($3, parent_id),
                description = COALESCE($4, description),
                contact_person = COALESCE($5, contact_person),
                contact_phone = COALESCE($6, contact_phone),
                update_time = $7
            WHERE unit_id = $8
            RETURNING *"#,
        )
        .bind(&organization.name)
        .bind(&organization.r#type)
        .bind(organization.parent_id)
        .bind(&organization.description)
        .bind(&organization.contact_person)
        .bind(&organization.contact_phone)
        .bind(chrono::Local::now().naive_utc())
        .bind(unit_id)
        .fetch_one(&*self.db)
        .await
        .map_err(|e| AppError::db_error("Failed to update organization", Some(&e.to_string())))?;

        Ok(result)
    }

    async fn update_status(&self, unit_id: &str, status: &str) -> AppResult<Organization> {
        let result = sqlx::query_as::<_, Organization>(
            r#"UPDATE organizations 
            SET status = $1, update_time = $2 
            WHERE unit_id = $3 
            RETURNING *"#,
        )
        .bind(status)
        .bind(chrono::Local::now().naive_utc())
        .bind(unit_id)
        .fetch_one(&*self.db)
        .await
        .map_err(|e| {
            AppError::db_error("Failed to update organization status", Some(&e.to_string()))
        })?;

        Ok(result)
    }

    async fn delete(&self, unit_id: &str) -> AppResult<bool> {
        let result = sqlx::query("DELETE FROM organizations WHERE unit_id = $1")
            .bind(unit_id)
            .execute(&*self.db)
            .await
            .map_err(|e| {
                AppError::db_error("Failed to delete organization", Some(&e.to_string()))
            })?;

        Ok(result.rows_affected() > 0)
    }

    async fn get_organization_tree(&self) -> AppResult<Vec<Organization>> {
        let organizations = sqlx::query_as::<_, Organization>(
            r#"SELECT 
                unit_id, 
                name, 
                type, 
                parent_id, 
                description, 
                contact_person, 
                contact_phone, 
                status, 
                create_time, 
                update_time
            FROM organizations 
            ORDER BY parent_id NULLS FIRST, name ASC"#,
        )
        .fetch_all(&*self.db)
        .await
        .map_err(|e| AppError::db_error("Failed to get organization tree", Some(&e.to_string())))?;

        Ok(organizations)
    }
}
