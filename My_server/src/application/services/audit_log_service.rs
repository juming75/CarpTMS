use crate::domain::entities::audit_log::{AuditLog, CreateAuditLog};
use crate::infrastructure::repositories::audit_log_repository::AuditLogRepository;
use anyhow::Result;

pub struct AuditLogService {
    repository: AuditLogRepository,
}

impl AuditLogService {
    pub fn new(repository: AuditLogRepository) -> Self {
        Self { repository }
    }

    pub async fn create_log(&self, log: CreateAuditLog) -> Result<AuditLog> {
        self.repository.create(log).await
    }

    pub async fn get_log_by_id(&self, id: i32) -> Result<Option<AuditLog>> {
        self.repository.get_by_id(id).await
    }

    pub async fn get_all_logs(&self, limit: i32, offset: i32) -> Result<Vec<AuditLog>> {
        self.repository.get_all(limit, offset).await
    }

    pub async fn get_logs_by_user_id(&self, user_id: i32, limit: i32, offset: i32) -> Result<Vec<AuditLog>> {
        self.repository.get_by_user_id(user_id, limit, offset).await
    }

    pub async fn get_logs_by_resource_type(&self, resource_type: &str, limit: i32, offset: i32) -> Result<Vec<AuditLog>> {
        self.repository.get_by_resource_type(resource_type, limit, offset).await
    }

    // 辅助方法，用于记录设置变更
    pub async fn log_settings_change(
        &self,
        user_id: Option<i32>,
        setting_type: &str,
        old_value: Option<serde_json::Value>,
        new_value: Option<serde_json::Value>,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<AuditLog> {
        let log = CreateAuditLog {
            user_id,
            action_type: "UPDATE".to_string(),
            resource_type: format!("SETTING_{}", setting_type.to_uppercase()),
            resource_id: None,
            old_value,
            new_value,
            ip_address,
            user_agent,
        };

        self.create_log(log).await
    }
}