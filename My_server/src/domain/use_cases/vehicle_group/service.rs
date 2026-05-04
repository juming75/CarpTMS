//! 车组服务

use std::sync::Arc;

use crate::domain::entities::vehicle_group::{
    VehicleGroup, VehicleGroupCreateRequest, VehicleGroupQuery, VehicleGroupTreeNode,
    VehicleGroupUpdateRequest,
};
use crate::domain::use_cases::vehicle_group::repository::VehicleGroupRepository;
use crate::redis::{del_cache_pattern, get_cache, set_cache};

/// 车组用例结构
#[derive(Clone)]
pub struct VehicleGroupUseCases {
    vehicle_group_repository: Arc<dyn VehicleGroupRepository + Send + Sync>,
}

impl VehicleGroupUseCases {
    /// 创建车组用例实例
    pub fn new(vehicle_group_repository: Arc<dyn VehicleGroupRepository>) -> Self {
        Self {
            vehicle_group_repository,
        }
    }

    /// 获取车组列表用例
    pub async fn get_vehicle_groups(
        &self,
        query: VehicleGroupQuery,
    ) -> Result<(Vec<VehicleGroup>, i64), anyhow::Error> {
        let cache_key = format!(
            "vehicle_groups:list:name_{}:page_{}:size_{}",
            query.group_name.as_deref().unwrap_or(""),
            query.page.unwrap_or(1),
            query.page_size.unwrap_or(20)
        );

        if let Ok(Some(cached)) = get_cache::<(Vec<VehicleGroup>, i64)>(&cache_key).await {
            return Ok(cached);
        }

        let result = self
            .vehicle_group_repository
            .find_all(query.page.unwrap_or(1), query.page_size.unwrap_or(20))
            .await?;

        let _ = set_cache(&cache_key, &result, 1800).await;
        Ok(result)
    }

    /// 获取单个车组用例
    pub async fn get_vehicle_group(
        &self,
        group_id: i32,
    ) -> Result<Option<VehicleGroup>, anyhow::Error> {
        let cache_key = format!("vehicle_group:{}:details", group_id);

        if let Ok(Some(cached)) = get_cache::<Option<VehicleGroup>>(&cache_key).await {
            return Ok(cached);
        }

        let result = self.vehicle_group_repository.find_by_id(group_id).await?;
        let _ = set_cache(&cache_key, &result, 1800).await;
        Ok(result)
    }

    /// 创建车组用例
    pub async fn create_vehicle_group(
        &self,
        group: VehicleGroupCreateRequest,
    ) -> Result<VehicleGroup, anyhow::Error> {
        if group.group_name.is_empty() {
            return Err(anyhow::anyhow!("车组名称不能为空"));
        }

        let existing_count = self
            .vehicle_group_repository
            .count_by_name(&group.group_name, None)
            .await?;
        if existing_count > 0 {
            return Err(anyhow::anyhow!("车组名称已存在"));
        }

        let created_group = self.vehicle_group_repository.create(group).await?;

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
        if let Some(group_name) = &group.group_name {
            if group_name.is_empty() {
                return Err(anyhow::anyhow!("车组名称不能为空"));
            }
        }

        let existing = self.vehicle_group_repository.find_by_id(group_id).await?;
        if existing.is_none() {
            return Ok(None);
        }

        if let Some(group_name) = &group.group_name {
            let existing_count = self
                .vehicle_group_repository
                .count_by_name(group_name, Some(group_id))
                .await?;
            if existing_count > 0 {
                return Err(anyhow::anyhow!("车组名称已存在"));
            }
        }

        let updated_group = self
            .vehicle_group_repository
            .update(group_id, group)
            .await
            .map(Some)?;

        if updated_group.is_some() {
            let _ = del_cache_pattern(&format!("vehicle_group:{}:*", group_id)).await;
            let _ = del_cache_pattern("vehicle_groups:list:*").await;
            let _ = del_cache_pattern("vehicle_group:tree").await;
        }

        Ok(updated_group)
    }

    /// 删除车组用例
    pub async fn delete_vehicle_group(&self, group_id: i32) -> Result<bool, anyhow::Error> {
        let existing = self.vehicle_group_repository.find_by_id(group_id).await?;
        if existing.is_none() {
            return Ok(false);
        }

        if let Ok(has_related) = self
            .vehicle_group_repository
            .has_related_data(group_id)
            .await
        {
            if has_related {
                return Err(anyhow::anyhow!("车组有关联数据，无法删除"));
            }
        }

        self.vehicle_group_repository.delete(group_id).await?;

        let _ = del_cache_pattern(&format!("vehicle_group:{}:*", group_id)).await;
        let _ = del_cache_pattern("vehicle_groups:list:*").await;
        let _ = del_cache_pattern("vehicle_group:tree").await;

        Ok(true)
    }

    /// 获取车组树结构用例
    pub async fn get_vehicle_group_tree(&self) -> Result<Vec<VehicleGroupTreeNode>, anyhow::Error> {
        let cache_key = "vehicle_group:tree";

        if let Ok(Some(cached)) = get_cache::<Vec<VehicleGroupTreeNode>>(cache_key).await {
            return Ok(cached);
        }

        let result = self.vehicle_group_repository.get_tree().await?;
        let _ = set_cache(cache_key, &result, 1800).await;
        Ok(result)
    }
}
