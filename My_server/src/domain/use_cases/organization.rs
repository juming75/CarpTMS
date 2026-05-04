//! 组织单位用例

use std::sync::Arc;

use validator::Validate;

use crate::domain::entities::organization::{
    Organization, OrganizationCreateRequest, OrganizationQuery, OrganizationUpdateRequest,
};
use crate::errors::{AppError, AppResult};

/// 组织单位仓库接口
#[async_trait::async_trait]
pub trait OrganizationRepository: Send + Sync {
    /// 创建组织单位
    async fn create(&self, organization: &Organization) -> AppResult<Organization>;

    /// 根据ID获取组织单位
    async fn get_by_id(&self, unit_id: &str) -> AppResult<Option<Organization>>;

    /// 根据ID列表获取组织单位
    async fn get_by_ids(&self, unit_ids: &[String]) -> AppResult<Vec<Organization>>;

    /// 获取所有组织单位
    async fn get_all(&self, query: &OrganizationQuery) -> AppResult<(Vec<Organization>, i64)>;

    /// 更新组织单位
    async fn update(
        &self,
        unit_id: &str,
        organization: &OrganizationUpdateRequest,
    ) -> AppResult<Organization>;

    /// 更新组织单位状态
    async fn update_status(&self, unit_id: &str, status: &str) -> AppResult<Organization>;

    /// 删除组织单位
    async fn delete(&self, unit_id: &str) -> AppResult<bool>;

    /// 获取组织单位树
    async fn get_organization_tree(&self) -> AppResult<Vec<Organization>>;
}

/// 组织单位用例
#[derive(Clone)]
pub struct OrganizationUseCases {
    repository: Arc<dyn OrganizationRepository + Send + Sync>,
}

impl OrganizationUseCases {
    /// 创建组织单位用例
    pub fn new(repository: Arc<dyn OrganizationRepository + Send + Sync>) -> Self {
        Self { repository }
    }

    /// 创建组织单位
    pub async fn create(&self, request: OrganizationCreateRequest) -> AppResult<Organization> {
        // 验证请求
        request
            .validate()
            .map_err(|e| AppError::validation(&e.to_string()))?;

        // 创建组织单位实体
        let organization = Organization::new(
            request.unit_id,
            request.name,
            request.r#type,
            request.parent_id,
            request.description,
            request.contact_person,
            request.contact_phone,
            request.status.unwrap_or("active".to_string()),
        );

        // 保存到数据库
        self.repository.create(&organization).await
    }

    /// 根据ID获取组织单位
    pub async fn get_by_id(&self, unit_id: &str) -> AppResult<Option<Organization>> {
        self.repository.get_by_id(unit_id).await
    }

    /// 根据ID列表获取组织单位
    pub async fn get_by_ids(&self, unit_ids: &[String]) -> AppResult<Vec<Organization>> {
        self.repository.get_by_ids(unit_ids).await
    }

    /// 获取所有组织单位
    pub async fn get_all(&self, query: &OrganizationQuery) -> AppResult<(Vec<Organization>, i64)> {
        self.repository.get_all(query).await
    }

    /// 更新组织单位
    pub async fn update(
        &self,
        unit_id: &str,
        request: OrganizationUpdateRequest,
    ) -> AppResult<Organization> {
        self.repository.update(unit_id, &request).await
    }

    /// 更新组织单位状态
    pub async fn update_status(&self, unit_id: &str, status: &str) -> AppResult<Organization> {
        self.repository.update_status(unit_id, status).await
    }

    /// 删除组织单位
    pub async fn delete(&self, unit_id: &str) -> AppResult<bool> {
        self.repository.delete(unit_id).await
    }

    /// 获取组织单位树
    pub async fn get_organization_tree(&self) -> AppResult<Vec<Organization>> {
        self.repository.get_organization_tree().await
    }
}

/// 应用服务接口实现
#[async_trait::async_trait]
impl crate::domain::use_cases::application_service::ApplicationService for OrganizationUseCases {
    fn name(&self) -> &str {
        "organization_service"
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
        pub OrganizationRepositoryImpl {}
        #[async_trait::async_trait]
        impl OrganizationRepository for OrganizationRepositoryImpl {
            async fn create(&self, organization: &Organization) -> AppResult<Organization>;
            async fn get_by_id(&self, unit_id: &str) -> AppResult<Option<Organization>>;
            async fn get_by_ids(&self, unit_ids: &[String]) -> AppResult<Vec<Organization>>;
            async fn get_all(&self, query: &OrganizationQuery) -> AppResult<(Vec<Organization>, i64)>;
            async fn update(&self, unit_id: &str, organization: &OrganizationUpdateRequest) -> AppResult<Organization>;
            async fn update_status(&self, unit_id: &str, status: &str) -> AppResult<Organization>;
            async fn delete(&self, unit_id: &str) -> AppResult<bool>;
            async fn get_organization_tree(&self) -> AppResult<Vec<Organization>>;
        }
    }

    #[tokio::test]
    async fn test_create_organization() -> Result<(), anyhow::Error> {
        let mut mock_repo = MockOrganizationRepositoryImpl::new();

        let request = OrganizationCreateRequest {
            unit_id: "org-001".to_string(),
            name: "Test Organization".to_string(),
            r#type: "company".to_string(),
            parent_id: None,
            description: Some("Test organization".to_string()),
            contact_person: Some("John Doe".to_string()),
            contact_phone: Some("1234567890".to_string()),
            status: Some("active".to_string()),
        };

        let _expected_organization = Organization::new(
            "org-001".to_string(),
            "Test Organization".to_string(),
            "company".to_string(),
            None,
            Some("Test organization".to_string()),
            Some("John Doe".to_string()),
            Some("1234567890".to_string()),
            "active".to_string(),
        );

        mock_repo.expect_create().returning(|org| Ok(org.clone()));

        let use_cases = OrganizationUseCases::new(Arc::new(mock_repo));
        let result = use_cases.create(request).await;

        assert!(result.is_ok());
        let created_organization = result.unwrap();
        assert_eq!(created_organization.unit_id, "org-001");
        assert_eq!(created_organization.name, "Test Organization");

        Ok(())
    }
}
