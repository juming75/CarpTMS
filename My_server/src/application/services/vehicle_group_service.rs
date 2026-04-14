//! 车组应用服务

use std::sync::Arc;

use crate::domain::entities::vehicle_group::{VehicleGroup, VehicleGroupCreateRequest, VehicleGroupTreeNode, VehicleGroupUpdateRequest};
use crate::errors::AppError;
use crate::domain::use_cases::vehicle_group::VehicleGroupRepository;

/// 车组应用服务
pub struct VehicleGroupService {
    repository: Arc<dyn VehicleGroupRepository>,
}

impl VehicleGroupService {
    /// 创建新车组服务
    pub fn new(repository: Arc<dyn VehicleGroupRepository>) -> Self {
        Self { repository }
    }

    /// 获取车组列表
    pub async fn get_vehicle_groups(&self, page: i32, page_size: i32) -> Result<(Vec<VehicleGroup>, i64), AppError> {
        self.repository.find_all(page, page_size).await.map_err(|e| AppError::internal_error("Database error", Some(&e.to_string())))
    }

    /// 获取车组详情
    pub async fn get_vehicle_group(&self, group_id: i32) -> Result<Option<VehicleGroup>, AppError> {
        self.repository.find_by_id(group_id).await.map_err(|e| AppError::internal_error("Database error", Some(&e.to_string())))
    }

    /// 创建车组
    pub async fn create_vehicle_group(&self, request: VehicleGroupCreateRequest) -> Result<VehicleGroup, AppError> {
        // 检查车组名称是否已存在
        let existing_count = self.repository.count_by_name(&request.group_name, None).await.map_err(|e| AppError::internal_error("Database error", Some(&e.to_string())))?;
        if existing_count > 0 {
            return Err(AppError::business_error("Vehicle group name already exists", None));
        }

        self.repository.create(request).await.map_err(|e| AppError::internal_error("Database error", Some(&e.to_string())))
    }

    /// 更新车组
    pub async fn update_vehicle_group(
        &self,
        group_id: i32,
        request: VehicleGroupUpdateRequest,
    ) -> Result<VehicleGroup, AppError> {
        // 检查车组是否存在
        if !self.repository.exists(group_id).await.map_err(|e| AppError::internal_error("Database error", Some(&e.to_string())))? {
            return Err(AppError::not_found_error("Vehicle group not found".to_string()));
        }

        // 检查车组名称是否已被其他车组使用
        if let Some(ref group_name) = request.group_name {
            if !group_name.is_empty() {
                let duplicate_count = self.repository.count_by_name(group_name, Some(group_id)).await.map_err(|e| AppError::internal_error("Database error", Some(&e.to_string())))?;
                if duplicate_count > 0 {
                    return Err(AppError::business_error("Vehicle group name already exists", None));
                }
            }
        }

        self.repository.update(group_id, request).await.map_err(|e| AppError::internal_error("Database error", Some(&e.to_string())))
    }

    /// 删除车组
    pub async fn delete_vehicle_group(&self, group_id: i32) -> Result<(), AppError> {
        // 检查车组是否存在
        if !self.repository.exists(group_id).await.map_err(|e| AppError::internal_error("Database error", Some(&e.to_string())))? {
            return Err(AppError::not_found_error("Vehicle group not found".to_string()));
        }

        // 检查是否有车辆属于该车组
        let vehicle_count = self.repository.count_vehicles(group_id).await.map_err(|e| AppError::internal_error("Database error", Some(&e.to_string())))?;
        if vehicle_count > 0 {
            return Err(AppError::business_error(
                "Cannot delete vehicle group with vehicles",
                None,
            ));
        }

        // 检查是否有子车组
        let child_count = self.repository.count_children(group_id).await.map_err(|e| AppError::internal_error("Database error", Some(&e.to_string())))?;
        if child_count > 0 {
            return Err(AppError::business_error(
                "Cannot delete vehicle group with child groups",
                None,
            ));
        }

        self.repository.delete(group_id).await.map_err(|e| AppError::internal_error("Database error", Some(&e.to_string())))
    }

    /// 获取车组树结构
    pub async fn get_vehicle_group_tree(&self) -> Result<Vec<VehicleGroupTreeNode>, AppError> {
        self.repository.get_tree().await.map_err(|e| AppError::internal_error("Database error", Some(&e.to_string())))
    }
}
