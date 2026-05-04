//! 称重数据统计服务

use std::sync::Arc;

use crate::domain::entities::weighing_data::WeighingData;

use super::repository::WeighingDataRepository;

/// 称重数据统计用例
#[derive(Clone)]
pub struct WeighingDataStatsUseCases {
    weighing_data_repository: Arc<dyn WeighingDataRepository>,
}

impl WeighingDataStatsUseCases {
    /// 创建统计用例实例
    pub fn new(weighing_data_repository: Arc<dyn WeighingDataRepository>) -> Self {
        Self {
            weighing_data_repository,
        }
    }

    /// 按车辆获取称重数据统计用例
    pub async fn get_weighing_data_stats_by_vehicle(
        &self,
        vehicle_id: i32,
        start_time: chrono::NaiveDateTime,
        end_time: chrono::NaiveDateTime,
    ) -> Result<Vec<WeighingData>, anyhow::Error> {
        // 数据验证
        if vehicle_id <= 0 {
            return Err(anyhow::anyhow!("车辆ID必须大于0"));
        }

        if start_time >= end_time {
            return Err(anyhow::anyhow!("开始时间必须小于结束时间"));
        }

        self.weighing_data_repository
            .get_weighing_data_stats_by_vehicle(vehicle_id, start_time, end_time)
            .await
    }

    /// 按设备获取称重数据统计用例
    pub async fn get_weighing_data_stats_by_device(
        &self,
        device_id: &str,
        start_time: chrono::NaiveDateTime,
        end_time: chrono::NaiveDateTime,
    ) -> Result<Vec<WeighingData>, anyhow::Error> {
        // 数据验证
        if device_id.is_empty() {
            return Err(anyhow::anyhow!("设备ID不能为空"));
        }

        if start_time >= end_time {
            return Err(anyhow::anyhow!("开始时间必须小于结束时间"));
        }

        self.weighing_data_repository
            .get_weighing_data_stats_by_device(device_id, start_time, end_time)
            .await
    }
}
