//! 司机领域用例

use std::sync::Arc;

use crate::domain::entities::driver::{Driver, DriverCreateRequest, DriverUpdateRequest, DriverQuery};
use crate::redis::{del_cache_pattern, get_cache, set_cache};

/// 司机仓库接口
#[async_trait::async_trait]
pub trait DriverRepository: Send + Sync {
    /// 获取司机列表
    async fn find_all(&self, page: i32, page_size: i32, query: DriverQuery) -> Result<(Vec<Driver>, i64), anyhow::Error>;

    /// 获取单个司机
    async fn find_by_id(&self, driver_id: i32) -> Result<Option<Driver>, anyhow::Error>;

    /// 创建司机
    async fn create(&self, driver: DriverCreateRequest) -> Result<Driver, anyhow::Error>;

    /// 更新司机
    async fn update(&self, driver_id: i32, driver: DriverUpdateRequest) -> Result<Driver, anyhow::Error>;

    /// 删除司机
    async fn delete(&self, driver_id: i32) -> Result<(), anyhow::Error>;
    
    /// 检查司机是否有关联数据
    async fn has_related_data(&self, driver_id: i32) -> Result<bool, anyhow::Error>;
    
    /// 检查司机是否存在
    async fn exists(&self, driver_id: i32) -> Result<bool, anyhow::Error>;
    
    /// 根据名称统计司机数量
    async fn count_by_name(&self, name: &str, exclude_id: Option<i32>) -> Result<i64, anyhow::Error>;
    
    /// 统计司机下的车辆数量
    async fn count_vehicles(&self, driver_id: i32) -> Result<i64, anyhow::Error>;
    
    /// 统计司机下的订单数量
    async fn count_orders(&self, driver_id: i32) -> Result<i64, anyhow::Error>;
}

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
        let result = self.driver_repository.find_all(
            query.page.unwrap_or(1),
            query.page_size.unwrap_or(20),
            query
        ).await?;

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
    pub async fn create_driver(&self, driver: DriverCreateRequest) -> Result<Driver, anyhow::Error> {
        // 业务逻辑:数据验证
        if driver.driver_name.is_empty() {
            return Err(anyhow::anyhow!("司机名称不能为空"));
        }

        // 业务逻辑：检查司机名称是否已存在
        let existing_count = self.driver_repository.count_by_name(&driver.driver_name, None).await?;
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
            let existing_count = self.driver_repository.count_by_name(driver_name, Some(driver_id)).await?;
            if existing_count > 0 {
                return Err(anyhow::anyhow!("司机名称已存在"));
            }
        }

        // 调用仓库更新司机
        let updated_driver = self.driver_repository.update(driver_id, driver).await.map(Some)?;

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
        // 初始化逻辑，如果需要的话
        Ok(())
    }

    async fn execute(&self, command: crate::domain::ddd::Command) -> crate::errors::AppResult<crate::domain::ddd::CommandResult> {
        use crate::domain::ddd::{CommandResult, DomainEvent};
        use crate::errors::AppError;
        
        match command.command_type.as_str() {
            "get_drivers" => {
                let query = serde_json::from_value(command.data)
                    .map_err(|e| AppError::internal_error(&format!("Failed to parse command data: {}", e), None))?;
                let (drivers, total) = self.get_drivers(query).await
                    .map_err(|e| AppError::internal_error(&format!("Failed to get drivers: {}", e), None))?;
                let data = serde_json::json!({ "drivers": drivers, "total": total });
                Ok(CommandResult::success_with_data(vec![], data))
            }
            "get_driver" => {
                let driver_id = serde_json::from_value(command.data)
                    .map_err(|e| AppError::internal_error(&format!("Failed to parse driver_id: {}", e), None))?;
                let driver = self.get_driver(driver_id).await
                    .map_err(|e| AppError::internal_error(&format!("Failed to get driver: {}", e), None))?;
                let data = serde_json::json!({ "driver": driver });
                Ok(CommandResult::success_with_data(vec![], data))
            }
            "create_driver" => {
                let driver = serde_json::from_value(command.data)
                    .map_err(|e| AppError::internal_error(&format!("Failed to parse driver data: {}", e), None))?;
                let created_driver = self.create_driver(driver).await
                    .map_err(|e| AppError::internal_error(&format!("Failed to create driver: {}", e), None))?;
                let data = serde_json::json!({ "driver": created_driver });
                let event = DomainEvent::new(
                    "Driver",
                    &created_driver.driver_id.to_string(),
                    "DriverCreated",
                    serde_json::to_value(created_driver)
                        .map_err(|e| AppError::internal_error(&format!("Failed to serialize driver: {}", e), None))?,
                    1
                );
                Ok(CommandResult::success_with_data(vec![event], data))
            }
            "update_driver" => {
                let (driver_id, driver) = serde_json::from_value(command.data)
                    .map_err(|e| AppError::internal_error(&format!("Failed to parse update data: {}", e), None))?;
                let updated_driver = self.update_driver(driver_id, driver).await
                    .map_err(|e| AppError::internal_error(&format!("Failed to update driver: {}", e), None))?;
                let data = serde_json::json!({ "driver": updated_driver });
                if let Some(driver) = updated_driver {
                    let event = DomainEvent::new(
                        "Driver",
                        &driver.driver_id.to_string(),
                        "DriverUpdated",
                        serde_json::to_value(driver)
                            .map_err(|e| AppError::internal_error(&format!("Failed to serialize driver: {}", e), None))?,
                        1
                    );
                    Ok(CommandResult::success_with_data(vec![event], data))
                } else {
                    Ok(CommandResult::success_with_data(vec![], data))
                }
            }
            "delete_driver" => {
                let driver_id = serde_json::from_value(command.data)
                    .map_err(|e| AppError::internal_error(&format!("Failed to parse driver_id: {}", e), None))?;
                let deleted = self.delete_driver(driver_id).await
                    .map_err(|e| AppError::internal_error(&format!("Failed to delete driver: {}", e), None))?;
                let data = serde_json::json!({ "deleted": deleted });
                if deleted {
                    let event = DomainEvent::new(
                        "Driver",
                        &driver_id.to_string(),
                        "DriverDeleted",
                        serde_json::json!({ "driver_id": driver_id }),
                        1
                    );
                    Ok(CommandResult::success_with_data(vec![event], data))
                } else {
                    Ok(CommandResult::success_with_data(vec![], data))
                }
            }
            _ => {
                Ok(CommandResult::error(format!("Unknown command: {}", command.command_type)))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use std::sync::Arc;

    struct MockDriverRepo {
        drivers: Vec<Driver>,
        has_related_data: bool,
    }

    #[async_trait::async_trait]
    impl DriverRepository for MockDriverRepo {
        async fn find_all(&self, _page: i32, _page_size: i32, _query: DriverQuery) -> Result<(Vec<Driver>, i64), anyhow::Error> {
            Ok((self.drivers.clone(), self.drivers.len() as i64))
        }

        async fn find_by_id(&self, driver_id: i32) -> Result<Option<Driver>, anyhow::Error> {
            Ok(self
                .drivers
                .iter()
                .find(|d| d.driver_id == driver_id)
                .cloned())
        }

        async fn create(&self, driver: DriverCreateRequest) -> Result<Driver, anyhow::Error> {
            let now = Utc::now();
            let new_driver = Driver {
                driver_id: self.drivers.len() as i32 + 1,
                driver_name: driver.driver_name,
                license_number: driver.license_number,
                phone_number: driver.phone_number,
                email: driver.email,
                status: driver.status,
                create_time: now,
                update_time: None,
            };
            Ok(new_driver)
        }

        async fn update(&self, driver_id: i32, driver: DriverUpdateRequest) -> Result<Driver, anyhow::Error> {
            if let Some(mut existing_driver) = self.find_by_id(driver_id).await? {
                if let Some(driver_name) = driver.driver_name {
                    existing_driver.driver_name = driver_name;
                }
                if driver.license_number.is_some() {
                    existing_driver.license_number = driver.license_number;
                }
                if driver.phone_number.is_some() {
                    existing_driver.phone_number = driver.phone_number;
                }
                if driver.email.is_some() {
                    existing_driver.email = driver.email;
                }
                if let Some(status) = driver.status {
                    existing_driver.status = status;
                }
                existing_driver.update_time = Some(Utc::now());

                Ok(existing_driver)
            } else {
                Err(anyhow::anyhow!("司机不存在"))
            }
        }

        async fn delete(&self, driver_id: i32) -> Result<(), anyhow::Error> {
            if self.drivers.iter().any(|d| d.driver_id == driver_id) {
                Ok(())
            } else {
                Err(anyhow::anyhow!("司机不存在"))
            }
        }
        
        async fn has_related_data(&self, _driver_id: i32) -> Result<bool, anyhow::Error> {
            Ok(self.has_related_data)
        }
        
        async fn exists(&self, driver_id: i32) -> Result<bool, anyhow::Error> {
            Ok(self.drivers.iter().any(|d| d.driver_id == driver_id))
        }
        
        async fn count_by_name(&self, name: &str, exclude_id: Option<i32>) -> Result<i64, anyhow::Error> {
            Ok(self.drivers.iter()
                .filter(|d| d.driver_name == name && exclude_id.map(|id| d.driver_id != id).unwrap_or(true))
                .count() as i64)
        }
        
        async fn count_vehicles(&self, _driver_id: i32) -> Result<i64, anyhow::Error> {
            Ok(0)
        }
        
        async fn count_orders(&self, _driver_id: i32) -> Result<i64, anyhow::Error> {
            Ok(0)
        }
    }

    // 测试用例:获取司机列表
    #[tokio::test]
    async fn test_get_drivers() {
        // 准备测试数据
        let now = Utc::now();
        let drivers = vec![
            Driver {
                driver_id: 1,
                driver_name: "测试司机1".to_string(),
                license_number: Some("123456".to_string()),
                phone_number: Some("13800138000".to_string()),
                email: Some("driver1@example.com".to_string()),
                status: 1,
                create_time: now,
                update_time: None,
            },
            Driver {
                driver_id: 2,
                driver_name: "测试司机2".to_string(),
                license_number: Some("654321".to_string()),
                phone_number: Some("13900139000".to_string()),
                email: Some("driver2@example.com".to_string()),
                status: 1,
                create_time: now,
                update_time: None,
            },
        ];

        // 创建模拟仓库
        let mock_repo = Arc::new(MockDriverRepo {
            drivers: drivers.clone(),
            has_related_data: false,
        });

        // 创建用例实例
        let use_cases = DriverUseCases::new(mock_repo);

        // 执行测试
        let query = DriverQuery { page: None, page_size: None, driver_name: None, license_number: None, status: None };
        let result = use_cases.get_drivers(query).await;

        // 验证结果
        assert!(result.is_ok());
        let (result_drivers, result_total) = result.ok_or_else(|| AppError::resource_not_found("Failed to get drivers"))?;
        assert_eq!(result_drivers, drivers);
        assert_eq!(result_total, 2);
    }

    // 测试用例:获取单个司机
    #[tokio::test]
    async fn test_get_driver() {
        // 准备测试数据
        let now = Utc::now();
        let driver = Driver {
            driver_id: 1,
            driver_name: "测试司机".to_string(),
            license_number: Some("123456".to_string()),
            phone_number: Some("13800138000".to_string()),
            email: Some("driver@example.com".to_string()),
            status: 1,
            create_time: now,
            update_time: None,
        };

        // 创建模拟仓库
        let mock_repo = Arc::new(MockDriverRepo {
            drivers: vec![driver.clone()],
            has_related_data: false,
        });

        // 创建用例实例
        let use_cases = DriverUseCases::new(mock_repo);

        // 执行测试
        let result = use_cases.get_driver(1).await;

        // 验证结果
        assert!(result.is_ok());
        assert_eq!(result.ok_or_else(|| AppError::resource_not_found("Driver not found"))?, Some(driver));
    }

    // 测试用例:创建司机成功
    #[tokio::test]
    async fn test_create_driver_success() {
        // 创建模拟仓库
        let mock_repo = Arc::new(MockDriverRepo {
            drivers: vec![],
            has_related_data: false,
        });

        // 创建用例实例
        let use_cases = DriverUseCases::new(mock_repo);

        // 准备测试数据
        let request = DriverCreateRequest {
            driver_name: "新司机".to_string(),
            license_number: Some("123456".to_string()),
            phone_number: Some("13800138000".to_string()),
            email: Some("newdriver@example.com".to_string()),
            status: 1,
        };

        // 执行测试
        let result = use_cases.create_driver(request).await;

        // 验证结果
        assert!(result.is_ok());
        let created_driver = result.ok_or_else(|| AppError::resource_not_found("Failed to create driver"))?;
        assert_eq!(created_driver.driver_name, "新司机");
    }

    // 测试用例:创建司机失败 - 名称已存在
    #[tokio::test]
    async fn test_create_driver_invalid_name() {
        // 准备测试数据
        let now = Utc::now();
        let existing_driver = Driver {
            driver_id: 1,
            driver_name: "已存在司机".to_string(),
            license_number: Some("123456".to_string()),
            phone_number: Some("13800138000".to_string()),
            email: Some("existing@example.com".to_string()),
            status: 1,
            create_time: now,
            update_time: None,
        };

        // 创建模拟仓库
        let mock_repo = Arc::new(MockDriverRepo {
            drivers: vec![existing_driver],
            has_related_data: false,
        });

        // 创建用例实例
        let use_cases = DriverUseCases::new(mock_repo);

        // 准备测试数据 - 使用已存在的名称
        let request = DriverCreateRequest {
            driver_name: "已存在司机".to_string(),
            license_number: Some("654321".to_string()),
            phone_number: Some("13900139000".to_string()),
            email: Some("new@example.com".to_string()),
            status: 1,
        };

        // 执行测试
        let result = use_cases.create_driver(request).await;

        // 验证结果
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("司机名称已存在"));
    }

    // 测试用例:更新司机
    #[tokio::test]
    async fn test_update_driver() {
        // 准备测试数据
        let now = Utc::now();
        let existing_driver = Driver {
            driver_id: 1,
            driver_name: "旧司机名称".to_string(),
            license_number: Some("123456".to_string()),
            phone_number: Some("13800138000".to_string()),
            email: Some("old@example.com".to_string()),
            status: 1,
            create_time: now,
            update_time: None,
        };

        // 创建模拟仓库
        let mock_repo = Arc::new(MockDriverRepo {
            drivers: vec![existing_driver],
            has_related_data: false,
        });

        // 创建用例实例
        let use_cases = DriverUseCases::new(mock_repo);

        // 准备测试数据
        let request = DriverUpdateRequest {
            driver_name: Some("新司机名称".to_string()),
            license_number: Some("654321".to_string()),
            phone_number: Some("13900139000".to_string()),
            email: Some("new@example.com".to_string()),
            status: Some(0),
        };

        // 执行测试
        let result = use_cases.update_driver(1, request).await;

        // 验证结果
        assert!(result.is_ok());
        let updated_driver = result.ok_or_else(|| AppError::resource_not_found("Failed to update driver"))?.ok_or_else(|| AppError::resource_not_found("Driver not found"))?;
        assert_eq!(updated_driver.driver_name, "新司机名称");
        assert_eq!(updated_driver.status, 0);
    }

    // 测试用例:删除司机成功
    #[tokio::test]
    async fn test_delete_driver_success() {
        // 准备测试数据
        let now = Utc::now();
        let driver = Driver {
            driver_id: 1,
            driver_name: "测试司机".to_string(),
            license_number: Some("123456".to_string()),
            phone_number: Some("13800138000".to_string()),
            email: Some("driver@example.com".to_string()),
            status: 1,
            create_time: now,
            update_time: None,
        };

        // 创建模拟仓库
        let mock_repo = Arc::new(MockDriverRepo {
            drivers: vec![driver],
            has_related_data: false,
        });

        // 创建用例实例
        let use_cases = DriverUseCases::new(mock_repo);

        // 执行测试
        let result = use_cases.delete_driver(1).await;

        // 验证结果
        assert!(result.is_ok());
        assert!(result.ok_or_else(|| AppError::resource_not_found("Failed to delete driver"))?);
    }

    // 测试用例:删除司机失败 - 有关联数据
    #[tokio::test]
    async fn test_delete_driver_with_related_data() {
        // 准备测试数据
        let now = Utc::now();
        let driver = Driver {
            driver_id: 1,
            driver_name: "测试司机".to_string(),
            license_number: Some("123456".to_string()),
            phone_number: Some("13800138000".to_string()),
            email: Some("driver@example.com".to_string()),
            status: 1,
            create_time: now,
            update_time: None,
        };

        // 创建模拟仓库 - 设置有关联数据
        let mock_repo = Arc::new(MockDriverRepo {
            drivers: vec![driver],
            has_related_data: true,
        });

        // 创建用例实例
        let use_cases = DriverUseCases::new(mock_repo);

        // 执行测试
        let result = use_cases.delete_driver(1).await;

        // 验证结果
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("司机有关联数据，无法删除"));
    }
}
