//! 车组统计服务

use std::sync::Arc;

use crate::domain::use_cases::vehicle_group::repository::VehicleGroupRepository;

/// 车组统计用例
#[derive(Clone)]
pub struct VehicleGroupStatsUseCases {
    vehicle_group_repository: Arc<dyn VehicleGroupRepository + Send + Sync>,
}

impl VehicleGroupStatsUseCases {
    pub fn new(vehicle_group_repository: Arc<dyn VehicleGroupRepository>) -> Self {
        Self {
            vehicle_group_repository,
        }
    }

    /// 获取车组总数
    pub async fn count_total(&self) -> Result<i64, anyhow::Error> {
        let (_, total) = self.vehicle_group_repository.find_all(1, 1).await?;
        Ok(total)
    }

    /// 获取子车组数量
    pub async fn count_children(&self, group_id: i32) -> Result<i64, anyhow::Error> {
        self.vehicle_group_repository.count_children(group_id).await
    }

    /// 获取车组下的车辆数量
    pub async fn count_vehicles(&self, group_id: i32) -> Result<i64, anyhow::Error> {
        self.vehicle_group_repository.count_vehicles(group_id).await
    }
}
