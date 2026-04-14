//! 车组领域用例

use std::sync::Arc;

use crate::domain::entities::vehicle_group::{VehicleGroup, VehicleGroupCreateRequest, VehicleGroupUpdateRequest, VehicleGroupQuery, VehicleGroupTreeNode};
use crate::redis::{del_cache_pattern, get_cache, set_cache};

/// 车组仓库接口
#[async_trait::async_trait]
pub trait VehicleGroupRepository: Send + Sync {
    /// 获取车组列表
    async fn find_all(&self, page: i32, page_size: i32) -> Result<(Vec<VehicleGroup>, i64), anyhow::Error>;

    /// 获取单个车组
    async fn find_by_id(&self, group_id: i32) -> Result<Option<VehicleGroup>, anyhow::Error>;

    /// 创建车组
    async fn create(&self, group: VehicleGroupCreateRequest) -> Result<VehicleGroup, anyhow::Error>;

    /// 更新车组
    async fn update(&self, group_id: i32, group: VehicleGroupUpdateRequest) -> Result<VehicleGroup, anyhow::Error>;

    /// 删除车组
    async fn delete(&self, group_id: i32) -> Result<(), anyhow::Error>;
    
    /// 检查车组是否有关联数据
    async fn has_related_data(&self, group_id: i32) -> Result<bool, anyhow::Error>;
    
    /// 获取车组树结构
    async fn get_tree(&self) -> Result<Vec<VehicleGroupTreeNode>, anyhow::Error>;
    
    /// 检查车组是否存在
    async fn exists(&self, group_id: i32) -> Result<bool, anyhow::Error>;
    
    /// 根据名称统计车组数量
    async fn count_by_name(&self, name: &str, exclude_id: Option<i32>) -> Result<i64, anyhow::Error>;
    
    /// 统计车组下的车辆数量
    async fn count_vehicles(&self, group_id: i32) -> Result<i64, anyhow::Error>;
    
    /// 统计子车组数量
    async fn count_children(&self, group_id: i32) -> Result<i64, anyhow::Error>;
}

/// 车组用例结构
#[derive(Clone)]
pub struct VehicleGroupUseCases {
    vehicle_group_repository: Arc<dyn VehicleGroupRepository + Send + Sync>,
}

impl VehicleGroupUseCases {
    /// 创建车组用例实例
    pub fn new(vehicle_group_repository: Arc<dyn VehicleGroupRepository>) -> Self {
        Self { vehicle_group_repository }
    }

    /// 获取车组列表用例
    pub async fn get_vehicle_groups(
        &self,
        query: VehicleGroupQuery,
    ) -> Result<(Vec<VehicleGroup>, i64), anyhow::Error> {
        // 构建缓存键
        let cache_key = format!(
            "vehicle_groups:list:name_{}:page_{}:size_{}",
            query.group_name.as_deref().unwrap_or(""),
            query.page.unwrap_or(1),
            query.page_size.unwrap_or(20)
        );

        // 尝试从缓存获取
        if let Ok(Some(cached)) = get_cache::<(Vec<VehicleGroup>, i64)>(&cache_key).await {
            return Ok(cached);
        }

        // 从数据库获取
        let result = self.vehicle_group_repository.find_all(query.page.unwrap_or(1), query.page_size.unwrap_or(20)).await?;

        // 缓存结果,过期时间30分钟
        let _ = set_cache(&cache_key, &result, 1800).await;

        Ok(result)
    }

    /// 获取单个车组用例
    pub async fn get_vehicle_group(&self, group_id: i32) -> Result<Option<VehicleGroup>, anyhow::Error> {
        // 构建缓存键
        let cache_key = format!("vehicle_group:{}:details", group_id);

        // 尝试从缓存获取
        if let Ok(Some(cached)) = get_cache::<Option<VehicleGroup>>(&cache_key).await {
            return Ok(cached);
        }

        // 从数据库获取
        let result = self.vehicle_group_repository.find_by_id(group_id).await?;

        // 缓存结果,过期时间30分钟
        let _ = set_cache(&cache_key, &result, 1800).await;

        Ok(result)
    }

    /// 创建车组用例
    pub async fn create_vehicle_group(&self, group: VehicleGroupCreateRequest) -> Result<VehicleGroup, anyhow::Error> {
        // 业务逻辑:数据验证
        if group.group_name.is_empty() {
            return Err(anyhow::anyhow!("车组名称不能为空"));
        }

        // 业务逻辑：检查车组名称是否已存在
        let existing_count = self.vehicle_group_repository.count_by_name(&group.group_name, None).await?;
        if existing_count > 0 {
            return Err(anyhow::anyhow!("车组名称已存在"));
        }

        // 调用仓库创建车组
        let created_group = self.vehicle_group_repository.create(group).await?;

        // 清理相关缓存
        let _ = del_cache_pattern("vehicle_groups:list:*").await;
        let _ = del_cache_pattern("vehicle_group:tree").await;

        Ok(created_group)
    }

    /// 更新车组用例
    pub async fn update_vehicle_group(
        &self,
        group_id: i32,
        group: VehicleGroupUpdateRequest,
    ) -> Result<Option<VehicleGroup>, anyhow::Error> {
        // 业务逻辑:数据验证
        if let Some(group_name) = &group.group_name {
            if group_name.is_empty() {
                return Err(anyhow::anyhow!("车组名称不能为空"));
            }
        }

        // 业务逻辑：检查车组是否存在
        let existing = self.vehicle_group_repository.find_by_id(group_id).await?;
        if existing.is_none() {
            return Ok(None);
        }

        // 业务逻辑：检查车组名称是否已被其他车组使用
        if let Some(group_name) = &group.group_name {
            let existing_count = self.vehicle_group_repository.count_by_name(group_name, Some(group_id)).await?;
            if existing_count > 0 {
                return Err(anyhow::anyhow!("车组名称已存在"));
            }
        }

        // 调用仓库更新车组
        let updated_group = self.vehicle_group_repository
            .update(group_id, group)
            .await.map(Some)?;

        // 清理相关缓存
        if updated_group.is_some() {
            let _ = del_cache_pattern(&format!("vehicle_group:{}:*", group_id)).await;
            let _ = del_cache_pattern("vehicle_groups:list:*").await;
            let _ = del_cache_pattern("vehicle_group:tree").await;
        }

        Ok(updated_group)
    }

    /// 删除车组用例
    pub async fn delete_vehicle_group(&self, group_id: i32) -> Result<bool, anyhow::Error> {
        // 业务逻辑:检查车组是否存在
        let existing = self.vehicle_group_repository.find_by_id(group_id).await?;
        if existing.is_none() {
            return Ok(false);
        }

        // 业务逻辑:检查关联数据
        if let Ok(has_related) = self.vehicle_group_repository.has_related_data(group_id).await {
            if has_related {
                return Err(anyhow::anyhow!("车组有关联数据，无法删除"));
            }
        }

        // 调用仓库删除车组
        self.vehicle_group_repository.delete(group_id).await?;

        // 清理相关缓存
        let _ = del_cache_pattern(&format!("vehicle_group:{}:*", group_id)).await;
        let _ = del_cache_pattern("vehicle_groups:list:*").await;
        let _ = del_cache_pattern("vehicle_group:tree").await;

        Ok(true)
    }
    
    /// 获取车组树结构用例
    pub async fn get_vehicle_group_tree(&self) -> Result<Vec<VehicleGroupTreeNode>, anyhow::Error> {
        // 构建缓存键
        let cache_key = "vehicle_group:tree";

        // 尝试从缓存获取
        if let Ok(Some(cached)) = get_cache::<Vec<VehicleGroupTreeNode>>(cache_key).await {
            return Ok(cached);
        }

        // 从数据库获取
        let result = self.vehicle_group_repository.get_tree().await?;

        // 缓存结果,过期时间30分钟
        let _ = set_cache(cache_key, &result, 1800).await;

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{NaiveDateTime, Utc};
    use std::sync::Arc;

    struct MockVehicleGroupRepo {
        groups: Vec<VehicleGroup>,
        has_related_data: bool,
    }

    #[async_trait::async_trait]
    impl VehicleGroupRepository for MockVehicleGroupRepo {
        async fn find_all(&self, _page: i32, _page_size: i32) -> Result<(Vec<VehicleGroup>, i64), anyhow::Error> {
            Ok((self.groups.clone(), self.groups.len() as i64))
        }

        async fn find_by_id(&self, group_id: i32) -> Result<Option<VehicleGroup>, anyhow::Error> {
            Ok(self
                .groups
                .iter()
                .find(|g| g.group_id == group_id)
                .cloned())
        }

        async fn create(&self, group: VehicleGroupCreateRequest) -> Result<VehicleGroup, anyhow::Error> {
            let now = Utc::now();
            let new_group = VehicleGroup {
                group_id: self.groups.len() as i32 + 1,
                group_name: group.group_name,
                parent_id: group.parent_id,
                parent_name: None,
                description: group.description,
                vehicle_count: 0,
                create_time: now,
                update_time: None,
            };
            Ok(new_group)
        }

        async fn update(&self, group_id: i32, group: VehicleGroupUpdateRequest) -> Result<VehicleGroup, anyhow::Error> {
            if let Some(mut existing_group) = self.find_by_id(group_id).await? {
                if let Some(group_name) = group.group_name {
                    existing_group.group_name = group_name;
                }
                if let Some(parent_id) = group.parent_id {
                    existing_group.parent_id = Some(parent_id);
                }
                if let Some(description) = group.description {
                    existing_group.description = Some(description);
                }
                existing_group.update_time = Some(Utc::now());

                Ok(existing_group)
            } else {
                Err(anyhow::anyhow!("车组不存在"))
            }
        }

        async fn delete(&self, group_id: i32) -> Result<(), anyhow::Error> {
            if self.groups.iter().any(|g| g.group_id == group_id) {
                Ok(())
            } else {
                Err(anyhow::anyhow!("车组不存在"))
            }
        }
        
        async fn has_related_data(&self, _group_id: i32) -> Result<bool, anyhow::Error> {
            Ok(self.has_related_data)
        }
        
        async fn get_tree(&self) -> Result<Vec<VehicleGroupTreeNode>, anyhow::Error> {
            Ok(Vec::new())
        }
        
        async fn exists(&self, group_id: i32) -> Result<bool, anyhow::Error> {
            Ok(self.groups.iter().any(|g| g.group_id == group_id))
        }
        
        async fn count_by_name(&self, name: &str, exclude_id: Option<i32>) -> Result<i64, anyhow::Error> {
            Ok(self.groups.iter()
                .filter(|g| g.group_name == name && exclude_id.map(|id| g.group_id != id).unwrap_or(true))
                .count() as i64)
        }
        
        async fn count_vehicles(&self, _group_id: i32) -> Result<i64, anyhow::Error> {
            Ok(0)
        }
        
        async fn count_children(&self, _group_id: i32) -> Result<i64, anyhow::Error> {
            Ok(0)
        }
    }

    // 测试用例:获取车组列表
    #[tokio::test]
    async fn test_get_vehicle_groups() {
        // 准备测试数据
        let now = Utc::now();
        let groups = vec![
            VehicleGroup {
                group_id: 1,
                group_name: "测试车组1".to_string(),
                parent_id: None,
                parent_name: None,
                description: Some("测试车组1描述".to_string()),
                vehicle_count: 5,
                create_time: now,
                update_time: None,
            },
            VehicleGroup {
                group_id: 2,
                group_name: "测试车组2".to_string(),
                parent_id: Some(1),
                parent_name: Some("测试车组1".to_string()),
                description: Some("测试车组2描述".to_string()),
                vehicle_count: 3,
                create_time: now,
                update_time: None,
            },
        ];

        // 创建模拟仓库
        let mock_repo = Arc::new(MockVehicleGroupRepo {
            groups: groups.clone(),
            has_related_data: false,
        });

        // 创建用例实例
        let use_cases = VehicleGroupUseCases::new(mock_repo);

        // 执行测试
        let query = VehicleGroupQuery { page: None, page_size: None, group_name: None };
        let result = use_cases.get_vehicle_groups(query).await;

        // 验证结果
        assert!(result.is_ok());
        let (result_groups, result_total) = result.unwrap();
        assert_eq!(result_groups, groups);
        assert_eq!(result_total, 2);
    }

    // 测试用例:获取单个车组
    #[tokio::test]
    async fn test_get_vehicle_group() {
        // 准备测试数据
        let now = Utc::now();
        let group = VehicleGroup {
            group_id: 1,
            group_name: "测试车组".to_string(),
            parent_id: None,
            parent_name: None,
            description: Some("测试车组描述".to_string()),
            vehicle_count: 5,
            create_time: now,
            update_time: None,
        };

        // 创建模拟仓库
        let mock_repo = Arc::new(MockVehicleGroupRepo {
            groups: vec![group.clone()],
            has_related_data: false,
        });

        // 创建用例实例
        let use_cases = VehicleGroupUseCases::new(mock_repo);

        // 执行测试
        let result = use_cases.get_vehicle_group(1).await;

        // 验证结果
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some(group));
    }

    // 测试用例:创建车组成功
    #[tokio::test]
    async fn test_create_vehicle_group_success() {
        // 准备测试数据
        let group_create = VehicleGroupCreateRequest {
            group_name: "新车组".to_string(),
            parent_id: None,
            description: Some("新车组描述".to_string()),
        };

        // 创建模拟仓库
        let mock_repo = Arc::new(MockVehicleGroupRepo {
            groups: Vec::new(),
            has_related_data: false,
        });

        // 创建用例实例
        let use_cases = VehicleGroupUseCases::new(mock_repo);

        // 执行测试
        let result = use_cases.create_vehicle_group(group_create).await;

        // 验证结果
        assert!(result.is_ok());
        let group = result.unwrap();
        assert_eq!(group.group_name, "新车组");
    }

    // 测试用例:创建车组失败 - 名称为空
    #[tokio::test]
    async fn test_create_vehicle_group_invalid_name() {
        // 准备测试数据
        let group_create = VehicleGroupCreateRequest {
            group_name: "".to_string(),
            parent_id: None,
            description: Some("新车组描述".to_string()),
        };

        // 创建模拟仓库
        let mock_repo = Arc::new(MockVehicleGroupRepo {
            groups: Vec::new(),
            has_related_data: false,
        });

        // 创建用例实例
        let use_cases = VehicleGroupUseCases::new(mock_repo);

        // 执行测试
        let result = use_cases.create_vehicle_group(group_create).await;

        // 验证结果
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "车组名称不能为空");
    }

    // 测试用例:更新车组
    #[tokio::test]
    async fn test_update_vehicle_group() {
        // 准备测试数据
        let now = Utc::now();
        let group = VehicleGroup {
            group_id: 1,
            group_name: "测试车组".to_string(),
            parent_id: None,
            parent_name: None,
            description: Some("测试车组描述".to_string()),
            vehicle_count: 5,
            create_time: now,
            update_time: None,
        };

        let group_update = VehicleGroupUpdateRequest {
            group_name: Some("更新车组".to_string()),
            parent_id: None,
            description: Some("更新车组描述".to_string()),
        };

        // 创建模拟仓库
        let mock_repo = Arc::new(MockVehicleGroupRepo {
            groups: vec![group.clone()],
            has_related_data: false,
        });

        // 创建用例实例
        let use_cases = VehicleGroupUseCases::new(mock_repo);

        // 执行测试
        let result = use_cases.update_vehicle_group(1, group_update).await;

        // 验证结果
        assert!(result.is_ok());
        let updated_group = result.unwrap().unwrap();
        assert_eq!(updated_group.group_name, "更新车组");
        assert_eq!(updated_group.description, Some("更新车组描述".to_string()));
    }

    // 测试用例:删除车组成功
    #[tokio::test]
    async fn test_delete_vehicle_group_success() {
        // 准备测试数据
        let now = Utc::now();
        let group = VehicleGroup {
            group_id: 1,
            group_name: "测试车组".to_string(),
            parent_id: None,
            parent_name: None,
            description: Some("测试车组描述".to_string()),
            vehicle_count: 5,
            create_time: now,
            update_time: None,
        };

        // 创建模拟仓库 (无关联数据)
        let mock_repo = Arc::new(MockVehicleGroupRepo {
            groups: vec![group.clone()],
            has_related_data: false,
        });

        // 创建用例实例
        let use_cases = VehicleGroupUseCases::new(mock_repo);

        // 执行测试
        let result = use_cases.delete_vehicle_group(1).await;

        // 验证结果
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    // 测试用例:删除车组失败 - 有关联数据
    #[tokio::test]
    async fn test_delete_vehicle_group_with_related_data() {
        // 准备测试数据
        let now = Utc::now();
        let group = VehicleGroup {
            group_id: 1,
            group_name: "测试车组".to_string(),
            parent_id: None,
            parent_name: None,
            description: Some("测试车组描述".to_string()),
            vehicle_count: 5,
            create_time: now,
            update_time: None,
        };

        // 创建模拟仓库 (有关联数据)
        let mock_repo = Arc::new(MockVehicleGroupRepo {
            groups: vec![group.clone()],
            has_related_data: true,
        });

        // 创建用例实例
        let use_cases = VehicleGroupUseCases::new(mock_repo);

        // 执行测试
        let result = use_cases.delete_vehicle_group(1).await;

        // 验证结果
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "车组有关联数据，无法删除");
    }
}