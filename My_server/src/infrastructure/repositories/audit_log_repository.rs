use crate::domain::entities::audit_log::{AuditLog, CreateAuditLog};
use anyhow::Result;
use sqlx::PgPool;
use std::sync::Arc;

pub struct AuditLogRepository {
    pool: Arc<PgPool>,
}

impl AuditLogRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    pub async fn create(&self, log: CreateAuditLog) -> Result<AuditLog> {
        let audit_log = sqlx::query_as::<_, AuditLog>(
            r#"
            INSERT INTO audit_logs (
                user_id, 
                action_type, 
                resource_type, 
                resource_id, 
                old_value, 
                new_value, 
                ip_address, 
                user_agent
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING 
                id, 
                user_id, 
                action_type, 
                resource_type, 
                resource_id, 
                old_value, 
                new_value, 
                ip_address, 
                user_agent, 
                created_at
            "#,
        )
        .bind(log.user_id)
        .bind(log.action_type)
        .bind(log.resource_type)
        .bind(log.resource_id)
        .bind(log.old_value)
        .bind(log.new_value)
        .bind(log.ip_address)
        .bind(log.user_agent)
        .fetch_one(&*self.pool)
        .await?;

        Ok(audit_log)
    }

    pub async fn get_by_id(&self, id: i32) -> Result<Option<AuditLog>> {
        let audit_log = sqlx::query_as::<_, AuditLog>(
            r#"
            SELECT 
                id, 
                user_id, 
                action_type, 
                resource_type, 
                resource_id, 
                old_value, 
                new_value, 
                ip_address, 
                user_agent, 
                created_at
            FROM audit_logs
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&*self.pool)
        .await?;

        Ok(audit_log)
    }

    pub async fn get_all(&self, limit: i32, offset: i32) -> Result<Vec<AuditLog>> {
        let audit_logs = sqlx::query_as::<_, AuditLog>(
            r#"
            SELECT 
                id, 
                user_id, 
                action_type, 
                resource_type, 
                resource_id, 
                old_value, 
                new_value, 
                ip_address, 
                user_agent, 
                created_at
            FROM audit_logs
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&*self.pool)
        .await?;

        Ok(audit_logs)
    }

    pub async fn get_by_user_id(
        &self,
        user_id: i32,
        limit: i32,
        offset: i32,
    ) -> Result<Vec<AuditLog>> {
        let audit_logs = sqlx::query_as::<_, AuditLog>(
            r#"
            SELECT 
                id, 
                user_id, 
                action_type, 
                resource_type, 
                resource_id, 
                old_value, 
                new_value, 
                ip_address, 
                user_agent, 
                created_at
            FROM audit_logs
            WHERE user_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(user_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&*self.pool)
        .await?;

        Ok(audit_logs)
    }

    pub async fn get_by_resource_type(
        &self,
        resource_type: &str,
        limit: i32,
        offset: i32,
    ) -> Result<Vec<AuditLog>> {
        let audit_logs = sqlx::query_as::<_, AuditLog>(
            r#"
            SELECT 
                id, 
                user_id, 
                action_type, 
                resource_type, 
                resource_id, 
                old_value, 
                new_value, 
                ip_address, 
                user_agent, 
                created_at
            FROM audit_logs
            WHERE resource_type = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(resource_type)
        .bind(limit)
        .bind(offset)
        .fetch_all(&*self.pool)
        .await?;

        Ok(audit_logs)
    }
}
