//! OpenAPI 平台用例

use std::sync::Arc;

use validator::Validate;

use crate::domain::entities::openapi_platform::{
    OpenapiPlatform, OpenapiPlatformCreateRequest, OpenapiPlatformQuery,
    OpenapiPlatformUpdateRequest,
};
use crate::errors::{AppError, AppResult};

/// OpenAPI 平台仓库接口
#[async_trait::async_trait]
pub trait OpenapiPlatformRepository: Send + Sync {
    /// 创建 OpenAPI 平台
    async fn create(&self, platform: &OpenapiPlatform) -> AppResult<OpenapiPlatform>;

    /// 根据ID获取 OpenAPI 平台
    async fn get_by_id(&self, id: i32) -> AppResult<Option<OpenapiPlatform>>;

    /// 获取所有 OpenAPI 平台
    async fn get_all(&self, query: &OpenapiPlatformQuery)
        -> AppResult<(Vec<OpenapiPlatform>, i64)>;

    /// 更新 OpenAPI 平台
    async fn update(
        &self,
        id: i32,
        platform: &OpenapiPlatformUpdateRequest,
    ) -> AppResult<OpenapiPlatform>;

    /// 更新 OpenAPI 平台状态
    async fn update_status(&self, id: i32, status: &str) -> AppResult<OpenapiPlatform>;

    /// 删除 OpenAPI 平台
    async fn delete(&self, id: i32) -> AppResult<bool>;
}

/// OpenAPI 平台用例
#[derive(Clone)]
pub struct OpenapiPlatformUseCases {
    repository: Arc<dyn OpenapiPlatformRepository + Send + Sync>,
}

impl OpenapiPlatformUseCases {
    /// 创建 OpenAPI 平台用例
    pub fn new(repository: Arc<dyn OpenapiPlatformRepository + Send + Sync>) -> Self {
        Self { repository }
    }

    /// 创建 OpenAPI 平台
    pub async fn create(
        &self,
        request: OpenapiPlatformCreateRequest,
    ) -> AppResult<OpenapiPlatform> {
        // 验证请求
        request
            .validate()
            .map_err(|e| AppError::validation(&e.to_string()))?;

        // 创建 OpenAPI 平台实体
        let platform = OpenapiPlatform::new(
            request.name,
            request.url,
            request.api_key,
            request.status.unwrap_or("active".to_string()),
        );

        // 保存到数据库
        self.repository.create(&platform).await
    }

    /// 根据ID获取 OpenAPI 平台
    pub async fn get_by_id(&self, id: i32) -> AppResult<Option<OpenapiPlatform>> {
        self.repository.get_by_id(id).await
    }

    /// 获取所有 OpenAPI 平台
    pub async fn get_all(
        &self,
        query: &OpenapiPlatformQuery,
    ) -> AppResult<(Vec<OpenapiPlatform>, i64)> {
        self.repository.get_all(query).await
    }

    /// 更新 OpenAPI 平台
    pub async fn update(
        &self,
        id: i32,
        request: OpenapiPlatformUpdateRequest,
    ) -> AppResult<OpenapiPlatform> {
        self.repository.update(id, &request).await
    }

    /// 更新 OpenAPI 平台状态
    pub async fn update_status(&self, id: i32, status: &str) -> AppResult<OpenapiPlatform> {
        self.repository.update_status(id, status).await
    }

    /// 删除 OpenAPI 平台
    pub async fn delete(&self, id: i32) -> AppResult<bool> {
        self.repository.delete(id).await
    }
}

/// 应用服务接口实现
#[async_trait::async_trait]
impl crate::domain::use_cases::application_service::ApplicationService for OpenapiPlatformUseCases {
    fn name(&self) -> &str {
        "openapi_platform_service"
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
