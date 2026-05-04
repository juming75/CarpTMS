//! 司机用例实现

use std::sync::Arc;

use crate::domain::entities::driver::{
    Driver, DriverCreateRequest, DriverQuery, DriverUpdateRequest,
};
use crate::domain::use_cases::driver::repository::DriverRepository;
use crate::redis::{del_cache_pattern, get_cache, set_cache};

/// 司机用例结构
#[derive(Clone)]
pub struct DriverUseCases {
    driver_repository: Arc<dyn DriverRepository + Send + Sync>,
}

impl DriverUseCases {
    /// 创建司机用例实例
    pub fn new(driver_repository: Arc<dyn DriverRepository>) -> Self {
        Self { driver_repository }
    }

    /// 获取司机列表用例
    pub async fn get_drivers(
        &self,
        query: DriverQuery,
    ) -> Result<(Vec<Driver>, i64), anyhow::Error> {
        // 构建缓存键
        let cache_key = format!(
            "drivers:list:name_{}:license_{}:status_{}:page_{}:size_{}",
            query.driver_name.as_deref().unwrap_or(""),
            query.license_number.as_deref().unwrap_or(""),
            query.status.unwrap_or(-1),
            query.page.unwrap_or(1),
            query.page_size.unwrap_or(20)
        );

        // 尝试从缓存获取
        if let Ok(Some(cached)) = get_cache::<(Vec<Driver>, i64)>(&cache_key).await {
            return Ok(cached);
        }

        // 从数据库获取
        let result = self
            .driver_repository
            .find_all(
                query.page.unwrap_or(1),
                query.page_size.unwrap_or(20),
                query,
            )
            .await?;

        // 缓存结果,过期时间30分钟
        let _ = set_cache(&cache_key, &result, 1800).await;

        Ok(result)
    }

    /// 获取单个司机用例
    pub async fn get_driver(&self, driver_id: i32) -> Result<Option<Driver>, anyhow::Error> {
        // 构建缓存键
        let cache_key = format!("driver:{}:details", driver_id);

        // 尝试从缓存获取
        if let Ok(Some(cached)) = get_cache::<Option<Driver>>(&cache_key).await {
            return Ok(cached);
        }

        // 从数据库获取
        let result = self.driver_repository.find_by_id(driver_id).await?;

        // 缓存结果,过期时间30分钟
        let _ = set_cache(&cache_key, &result, 1800).await;

        Ok(result)
    }

    /// 创建司机用例
    pub async fn create_driver(
        &self,
        driver: DriverCreateRequest,
    ) -> Result<Driver, anyhow::Error> {
        // 业务逻辑:数据验证
        if driver.driver_name.is_empty() {
            return Err(anyhow::anyhow!("司机名称不能为空"));
        }

        // 业务逻辑：检查司机名称是否已存在
        let existing_count = self
            .driver_repository
            .count_by_name(&driver.driver_name, None)
            .await?;
        if existing_count > 0 {
            return Err(anyhow::anyhow!("司机名称已存在"));
        }

        // 调用仓库创建司机
        let created_driver = self.driver_repository.create(driver).await?;

        // 清理相关缓存
        let _ = del_cache_pattern("drivers:list:*").await;

        Ok(created_driver)
    }

    /// 更新司机用例
    pub async fn update_driver(
        &self,
        driver_id: i32,
        driver: DriverUpdateRequest,
    ) -> Result<Option<Driver>, anyhow::Error> {
        // 业务逻辑:数据验证
        if let Some(driver_name) = &driver.driver_name {
            if driver_name.is_empty() {
                return Err(anyhow::anyhow!("司机名称不能为空"));
            }
        }

        // 业务逻辑：检查司机是否存在
        let existing = self.driver_repository.find_by_id(driver_id).await?;
        if existing.is_none() {
            return Ok(None);
        }

        // 业务逻辑：检查司机名称是否已被其他司机使用
        if let Some(driver_name) = &driver.driver_name {
            let existing_count = self
                .driver_repository
                .count_by_name(driver_name, Some(driver_id))
                .await?;
            if existing_count > 0 {
                return Err(anyhow::anyhow!("司机名称已存在"));
            }
        }

        // 调用仓库更新司机
        let updated_driver = self
            .driver_repository
            .update(driver_id, driver)
            .await
            .map(Some)?;

        // 清理相关缓存
        let _ = del_cache_pattern(&format!("driver:{}:*", driver_id)).await;
        let _ = del_cache_pattern("drivers:list:*").await;

        Ok(updated_driver)
    }

    /// 删除司机用例
    pub async fn delete_driver(&self, driver_id: i32) -> Result<bool, anyhow::Error> {
        // 业务逻辑:检查司机是否存在
        let existing = self.driver_repository.find_by_id(driver_id).await?;
        if existing.is_none() {
            return Ok(false);
        }

        // 业务逻辑:检查关联数据
        if let Ok(has_related) = self.driver_repository.has_related_data(driver_id).await {
            if has_related {
                return Err(anyhow::anyhow!("司机有关联数据，无法删除"));
            }
        }

        // 调用仓库删除司机
        self.driver_repository.delete(driver_id).await?;

        // 清理相关缓存
        let _ = del_cache_pattern(&format!("driver:{}:*", driver_id)).await;
        let _ = del_cache_pattern("drivers:list:*").await;

        Ok(true)
    }
}

// 实现 ApplicationService trait
#[async_trait::async_trait]
impl crate::domain::ddd::ApplicationService for DriverUseCases {
    fn name(&self) -> &str {
        "driver_service"
    }

    async fn initialize(&self) -> crate::errors::AppResult<()> {
        Ok(())
    }

    async fn execute(
        &self,
        command: crate::domain::ddd::Command,
    ) -> crate::errors::AppResult<crate::domain::ddd::CommandResult> {
        use crate::domain::ddd::{CommandResult, DomainEvent};
        use crate::errors::AppError;

        match command.command_type.as_str() {
            "get_drivers" => {
                let query = serde_json::from_value(command.data).map_err(|e| {
                    AppError::internal_error(&format!("Failed to parse command data: {}", e), None)
                })?;
                let (drivers, total) = self.get_drivers(query).await.map_err(|e| {
                    AppError::internal_error(&format!("Failed to get drivers: {}", e), None)
                })?;
                let data = serde_json::json!({ "drivers": drivers, "total": total });
                Ok(CommandResult::success_with_data(vec![], data))
            }
            "get_driver" => {
                let driver_id = serde_json::from_value(command.data).map_err(|e| {
                    AppError::internal_error(&format!("Failed to parse driver_id: {}", e), None)
                })?;
                let driver = self.get_driver(driver_id).await.map_err(|e| {
                    AppError::internal_error(&format!("Failed to get driver: {}", e), None)
                })?;
                let data = serde_json::json!({ "driver": driver });
                Ok(CommandResult::success_with_data(vec![], data))
            }
            "create_driver" => {
                let driver = serde_json::from_value(command.data).map_err(|e| {
                    AppError::internal_error(&format!("Failed to parse driver data: {}", e), None)
                })?;
                let created_driver = self.create_driver(driver).await.map_err(|e| {
                    AppError::internal_error(&format!("Failed to create driver: {}", e), None)
                })?;
                let data = serde_json::json!({ "driver": created_driver });
                let event = DomainEvent::new(
                    "Driver",
                    &created_driver.driver_id.to_string(),
                    "DriverCreated",
                    serde_json::to_value(created_driver).map_err(|e| {
                        AppError::internal_error(
                            &format!("Failed to serialize driver: {}", e),
                            None,
                        )
                    })?,
                    1,
                );
                Ok(CommandResult::success_with_data(vec![event], data))
            }
            "update_driver" => {
                let (driver_id, driver) = serde_json::from_value(command.data).map_err(|e| {
                    AppError::internal_error(&format!("Failed to parse update data: {}", e), None)
                })?;
                let updated_driver = self.update_driver(driver_id, driver).await.map_err(|e| {
                    AppError::internal_error(&format!("Failed to update driver: {}", e), None)
                })?;
                let data = serde_json::json!({ "driver": updated_driver });
                if let Some(driver) = updated_driver {
                    let event = DomainEvent::new(
                        "Driver",
                        &driver.driver_id.to_string(),
                        "DriverUpdated",
                        serde_json::to_value(driver).map_err(|e| {
                            AppError::internal_error(
                                &format!("Failed to serialize driver: {}", e),
                                None,
                            )
                        })?,
                        1,
                    );
                    Ok(CommandResult::success_with_data(vec![event], data))
                } else {
                    Ok(CommandResult::success_with_data(vec![], data))
                }
            }
            "delete_driver" => {
                let driver_id = serde_json::from_value(command.data).map_err(|e| {
                    AppError::internal_error(&format!("Failed to parse driver_id: {}", e), None)
                })?;
                let deleted = self.delete_driver(driver_id).await.map_err(|e| {
                    AppError::internal_error(&format!("Failed to delete driver: {}", e), None)
                })?;
                let data = serde_json::json!({ "deleted": deleted });
                if deleted {
                    let event = DomainEvent::new(
                        "Driver",
                        &driver_id.to_string(),
                        "DriverDeleted",
                        serde_json::json!({ "driver_id": driver_id }),
                        1,
                    );
                    Ok(CommandResult::success_with_data(vec![event], data))
                } else {
                    Ok(CommandResult::success_with_data(vec![], data))
                }
            }
            _ => Ok(CommandResult::error(format!(
                "Unknown command: {}",
                command.command_type
            ))),
        }
    }
}
