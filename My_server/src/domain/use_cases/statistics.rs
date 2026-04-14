//! 统计分析领域用例
//! 
//! 包含系统统计分析的核心业务逻辑，包括车辆统计、设备统计、称重数据统计等

use std::sync::Arc;
use chrono::DateTime;

use anyhow::Result;

use crate::domain::entities::statistics::{StatisticsQuery, VehicleStatistics, DeviceStatistics, WeighingStatistics, CustomStatistics};

// 统计仓库接口
#[async_trait::async_trait]
pub trait StatisticsRepository: Send + Sync {
    // 车辆统计
    async fn get_vehicle_statistics(&self) -> Result<VehicleStatistics, anyhow::Error>;
    
    // 设备统计
    async fn get_device_statistics(&self) -> Result<DeviceStatistics, anyhow::Error>;
    
    // 称重数据统计
    async fn get_weighing_statistics(&self, start_time: DateTime<chrono::Utc>, end_time: DateTime<chrono::Utc>) -> Result<WeighingStatistics, anyhow::Error>;
    
    // 安全指数排行
    async fn get_safety_ranking(&self) -> Result<Vec<serde_json::Value>, anyhow::Error>;
    
    // 自定义统计
    async fn get_custom_statistics(&self, start_time: DateTime<chrono::Utc>, end_time: DateTime<chrono::Utc>) -> Result<CustomStatistics, anyhow::Error>;
}

// 统计用例
pub struct StatisticsUseCases {
    repository: Arc<dyn StatisticsRepository>,
}

impl StatisticsUseCases {
    pub fn new(repository: Arc<dyn StatisticsRepository>) -> Self {
        Self {
            repository,
        }
    }
    
    pub async fn get_vehicle_statistics(&self) -> Result<VehicleStatistics> {
        self.repository.get_vehicle_statistics().await
    }
    
    pub async fn get_device_statistics(&self) -> Result<DeviceStatistics> {
        self.repository.get_device_statistics().await
    }
    
    pub async fn get_weighing_statistics(&self, query: StatisticsQuery) -> Result<WeighingStatistics> {
        // 业务逻辑：设置默认时间范围为最近30天
        let end_time = query.end_time.unwrap_or(chrono::Utc::now());
        let start_time = query.start_time.unwrap_or(end_time - chrono::Duration::days(30));
        
        self.repository.get_weighing_statistics(start_time, end_time).await
    }
    
    pub async fn get_safety_ranking(&self) -> Result<Vec<serde_json::Value>> {
        self.repository.get_safety_ranking().await
    }
    
    pub async fn get_custom_statistics(&self, query: StatisticsQuery) -> Result<CustomStatistics> {
        // 业务逻辑：设置默认时间范围为最近7天
        let end_time = query.end_time.unwrap_or(chrono::Utc::now());
        let start_time = query.start_time.unwrap_or(end_time - chrono::Duration::days(7));
        
        self.repository.get_custom_statistics(start_time, end_time).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    
    // 模拟统计仓库
    struct MockStatisticsRepository;
    
    impl StatisticsRepository for MockStatisticsRepository {
        async fn get_vehicle_statistics(&self) -> Result<VehicleStatistics> {
            Ok(VehicleStatistics {
                total_vehicles: 100,
                active_vehicles: 80,
                inactive_vehicles: 20,
                vehicles_by_type: vec![],
                vehicles_by_group: vec![],
                vehicles_with_devices: 70,
            })
        }
        
        async fn get_device_statistics(&self) -> Result<DeviceStatistics> {
            Ok(DeviceStatistics {
                total_devices: 50,
                online_devices: 45,
                offline_devices: 5,
                devices_by_type: vec![],
                devices_by_manufacturer: vec![],
                devices_with_vehicles: 40,
            })
        }
        
        async fn get_weighing_statistics(&self, _start_time: DateTime<Utc>, _end_time: DateTime<Utc>) -> Result<WeighingStatistics> {
            Ok(WeighingStatistics {
                total_weight: 10000.0,
                total_records: 100,
                average_weight: 100.0,
                daily_weighing: vec![],
                vehicles_by_weight: vec![],
                devices_by_weight: vec![],
            })
        }
        
        async fn get_safety_ranking(&self) -> Result<Vec<serde_json::Value>> {
            Ok(vec![
                serde_json::json!({"id": 1, "name": "车队1队", "department": "车队1队", "score": 95}),
                serde_json::json!({"id": 2, "name": "车队2队", "department": "车队2队", "score": 92}),
            ])
        }
        
        async fn get_custom_statistics(&self, _start_time: DateTime<Utc>, _end_time: DateTime<Utc>) -> Result<StatisticsResponse> {
            Ok(StatisticsResponse {
                total: 5000.0,
                count: 50,
                average: 100.0,
                min: 50.0,
                max: 150.0,
                data: vec![],
                message: "Custom statistics fetched successfully".to_string(),
            })
        }
    }
    
    #[tokio::test]
    async fn test_get_vehicle_statistics() {
        let repository = Arc::new(MockStatisticsRepository);
        let use_cases = StatisticsUseCases::new(repository);
        
        let result = use_cases.get_vehicle_statistics().await;
        assert!(result.is_ok());
        let stats = result.unwrap();
        assert_eq!(stats.total_vehicles, 100);
        assert_eq!(stats.active_vehicles, 80);
    }
    
    #[tokio::test]
    async fn test_get_device_statistics() {
        let repository = Arc::new(MockStatisticsRepository);
        let use_cases = StatisticsUseCases::new(repository);
        
        let result = use_cases.get_device_statistics().await;
        assert!(result.is_ok());
        let stats = result.unwrap();
        assert_eq!(stats.total_devices, 50);
        assert_eq!(stats.online_devices, 45);
    }
}
