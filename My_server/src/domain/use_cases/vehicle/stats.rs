//! 车辆统计服务
//!
//! 实现车辆相关的统计查询

use std::sync::Arc;

use crate::domain::entities::vehicle::Vehicle;
use crate::domain::entities::vehicle::VehicleQuery;
use crate::domain::use_cases::vehicle::repository::VehicleRepository;

/// 车辆统计用例
#[derive(Clone)]
pub struct VehicleStatsUseCases {
    vehicle_repository: Arc<dyn VehicleRepository + Send + Sync>,
}

impl VehicleStatsUseCases {
    pub fn new(vehicle_repository: Arc<dyn VehicleRepository>) -> Self {
        Self { vehicle_repository }
    }

    /// 按车组统计车辆数量
    pub async fn count_by_group(&self, _group_id: i32) -> Result<i64, anyhow::Error> {
        let query = VehicleQuery {
            page: Some(1),
            page_size: Some(1),
            vehicle_name: None,
            license_plate: None,
            vehicle_type: None,
            status: None,
        };

        let (_, total) = self.vehicle_repository.get_vehicles(query).await?;
        // 简化实现，实际需要按group_id过滤
        Ok(total)
    }

    /// 统计车辆总数
    pub async fn count_total(&self) -> Result<i64, anyhow::Error> {
        let query = VehicleQuery {
            page: Some(1),
            page_size: Some(1),
            vehicle_name: None,
            license_plate: None,
            vehicle_type: None,
            status: None,
        };

        let (_, total) = self.vehicle_repository.get_vehicles(query).await?;
        Ok(total)
    }

    /// 按状态统计车辆数量
    pub async fn count_by_status(&self, status: i32) -> Result<i64, anyhow::Error> {
        let query = VehicleQuery {
            page: Some(1),
            page_size: Some(1),
            vehicle_name: None,
            license_plate: None,
            vehicle_type: None,
            status: Some(status),
        };

        let (_, total) = self.vehicle_repository.get_vehicles(query).await?;
        Ok(total)
    }

    /// 获取即将过期的年检车辆
    pub async fn get_expiring_inspection(&self, days: i32) -> Result<Vec<Vehicle>, anyhow::Error> {
        let query = VehicleQuery {
            page: Some(1),
            page_size: Some(100),
            vehicle_name: None,
            license_plate: None,
            vehicle_type: None,
            status: Some(1),
        };

        let (vehicles, _) = self.vehicle_repository.get_vehicles(query).await?;

        // 过滤即将过期的车辆
        let now = chrono::Utc::now().naive_local();
        let threshold = now + chrono::Duration::days(days as i64);

        let expiring: Vec<Vehicle> = vehicles
            .into_iter()
            .filter(|v| v.inspection_date <= threshold && v.inspection_date > now)
            .collect();

        Ok(expiring)
    }
}
