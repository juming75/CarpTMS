//! / 称重数据领域用例

use std::sync::Arc;

use crate::domain::entities::weighing_data::{
    WeighingData, WeighingDataCreate, WeighingDataQuery, WeighingDataUpdate,
};

/// 称重数据仓库接口
#[async_trait::async_trait]
pub trait WeighingDataRepository: Send + Sync {
    /// 获取称重数据列表
    async fn get_weighing_data_list(
        &self,
        query: WeighingDataQuery,
    ) -> Result<(Vec<WeighingData>, i64), anyhow::Error>;

    /// 获取单个称重数据
    async fn get_weighing_data(&self, id: i32) -> Result<Option<WeighingData>, anyhow::Error>;

    /// 创建称重数据
    async fn create_weighing_data(
        &self,
        weighing_data: WeighingDataCreate,
    ) -> Result<WeighingData, anyhow::Error>;

    /// 更新称重数据
    async fn update_weighing_data(
        &self,
        id: i32,
        weighing_data: WeighingDataUpdate,
    ) -> Result<Option<WeighingData>, anyhow::Error>;

    /// 删除称重数据
    async fn delete_weighing_data(&self, id: i32) -> Result<bool, anyhow::Error>;

    /// 按车辆获取称重数据统计
    async fn get_weighing_data_stats_by_vehicle(
        &self,
        vehicle_id: i32,
        start_time: chrono::NaiveDateTime,
        end_time: chrono::NaiveDateTime,
    ) -> Result<Vec<WeighingData>, anyhow::Error>;

    /// 按设备获取称重数据统计
    async fn get_weighing_data_stats_by_device(
        &self,
        device_id: &str,
        start_time: chrono::NaiveDateTime,
        end_time: chrono::NaiveDateTime,
    ) -> Result<Vec<WeighingData>, anyhow::Error>;
}

/// 称重数据用例结构
#[derive(Clone)]
pub struct WeighingDataUseCases {
    weighing_data_repository: Arc<dyn WeighingDataRepository>,
}

impl WeighingDataUseCases {
    /// 创建称重数据用例实例
    pub fn new(weighing_data_repository: Arc<dyn WeighingDataRepository>) -> Self {
        Self {
            weighing_data_repository,
        }
    }

    /// 获取称重数据列表用例
    pub async fn get_weighing_data_list(
        &self,
        query: WeighingDataQuery,
    ) -> Result<(Vec<WeighingData>, i64), anyhow::Error> {
        // 业务逻辑:可以在这里添加额外的业务规则,例如权限检查、数据验证等
        self.weighing_data_repository
            .get_weighing_data_list(query)
            .await
    }

    /// 获取单个称重数据用例
    pub async fn get_weighing_data_by_id(
        &self,
        id: i32,
    ) -> Result<Option<WeighingData>, anyhow::Error> {
        if id <= 0 {
            return Err(anyhow::anyhow!("称重数据ID必须大于0"));
        }
        self.weighing_data_repository.get_weighing_data(id).await
    }

    /// 创建称重数据用例
    pub async fn create_weighing_data(
        &self,
        weighing_data: WeighingDataCreate,
    ) -> Result<WeighingData, anyhow::Error> {
        // 业务逻辑:数据验证
        if weighing_data.gross_weight <= 0.0 {
            return Err(anyhow::anyhow!("毛重必须大于0"));
        }

        // 调用仓库创建称重数据
        self.weighing_data_repository
            .create_weighing_data(weighing_data)
            .await
    }

    /// 更新称重数据用例
    pub async fn update_weighing_data(
        &self,
        id: i32,
        weighing_data: WeighingDataUpdate,
    ) -> Result<Option<WeighingData>, anyhow::Error> {
        // 业务逻辑:数据验证
        if id <= 0 {
            return Err(anyhow::anyhow!("称重数据ID必须大于0"));
        }

        // 调用仓库更新称重数据
        self.weighing_data_repository
            .update_weighing_data(id, weighing_data)
            .await
    }

    /// 删除称重数据用例
    pub async fn delete_weighing_data(&self, id: i32) -> Result<bool, anyhow::Error> {
        // 业务逻辑:可以在这里添加额外的业务规则,例如权限检查等
        if id <= 0 {
            return Err(anyhow::anyhow!("称重数据ID必须大于0"));
        }

        // 调用仓库删除称重数据
        self.weighing_data_repository.delete_weighing_data(id).await
    }

    /// 按车辆获取称重数据统计用例
    pub async fn get_weighing_data_stats_by_vehicle(
        &self,
        vehicle_id: i32,
        start_time: chrono::NaiveDateTime,
        end_time: chrono::NaiveDateTime,
    ) -> Result<Vec<WeighingData>, anyhow::Error> {
        // 业务逻辑:数据验证
        if vehicle_id <= 0 {
            return Err(anyhow::anyhow!("车辆ID必须大于0"));
        }

        if start_time >= end_time {
            return Err(anyhow::anyhow!("开始时间必须小于结束时间"));
        }

        // 调用仓库获取统计数据
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
        // 业务逻辑:数据验证
        if device_id.is_empty() {
            return Err(anyhow::anyhow!("设备ID不能为空"));
        }

        if start_time >= end_time {
            return Err(anyhow::anyhow!("开始时间必须小于结束时间"));
        }

        // 调用仓库获取统计数据
        self.weighing_data_repository
            .get_weighing_data_stats_by_device(device_id, start_time, end_time)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::weighing_data::{
        WeighingData, WeighingDataCreate, WeighingDataQuery, WeighingDataUpdate,
    };
    use async_trait::async_trait;
    use chrono::NaiveDate;
    use std::sync::Arc;

    // 模拟WeighingDataRepository实现
    #[derive(Clone)]
    struct MockWeighingDataRepository {
        weighing_data: Vec<WeighingData>,
    }

    #[async_trait]
    impl WeighingDataRepository for MockWeighingDataRepository {
        async fn get_weighing_data_list(
            &self,
            _query: WeighingDataQuery,
        ) -> Result<(Vec<WeighingData>, i64), anyhow::Error> {
            // 这里可以根据query实现过滤逻辑,为了简单起见,我们只返回所有数据
            Ok((self.weighing_data.clone(), self.weighing_data.len() as i64))
        }

        async fn get_weighing_data(&self, id: i32) -> Result<Option<WeighingData>, anyhow::Error> {
            Ok(self
                .weighing_data
                .iter()
                .find(|&data| data.id == id)
                .cloned())
        }

        async fn create_weighing_data(
            &self,
            weighing_data: WeighingDataCreate,
        ) -> Result<WeighingData, anyhow::Error> {
            let now = chrono::Utc::now().naive_utc();
            Ok(WeighingData {
                id: self.weighing_data.len() as i32 + 1,
                vehicle_id: weighing_data.vehicle_id,
                device_id: weighing_data.device_id,
                weighing_time: weighing_data.weighing_time,
                gross_weight: weighing_data.gross_weight,
                tare_weight: weighing_data.tare_weight,
                net_weight: weighing_data.net_weight,
                axle_count: weighing_data.axle_count,
                speed: weighing_data.speed,
                lane_no: weighing_data.lane_no,
                site_id: weighing_data.site_id,
                status: 1,
                create_time: now,
                update_time: None,
            })
        }

        async fn update_weighing_data(
            &self,
            id: i32,
            weighing_data: WeighingDataUpdate,
        ) -> Result<Option<WeighingData>, anyhow::Error> {
            if let Some(mut data) = self
                .weighing_data
                .iter()
                .find(|&data| data.id == id)
                .cloned()
            {
                if let Some(vehicle_id) = weighing_data.vehicle_id {
                    data.vehicle_id = vehicle_id;
                }
                if let Some(device_id) = &weighing_data.device_id {
                    data.device_id = device_id.clone();
                }
                if let Some(weighing_time) = weighing_data.weighing_time {
                    data.weighing_time = weighing_time;
                }
                if let Some(gross_weight) = weighing_data.gross_weight {
                    data.gross_weight = gross_weight;
                }
                if let Some(tare_weight) = weighing_data.tare_weight {
                    data.tare_weight = Some(tare_weight);
                }
                if let Some(net_weight) = weighing_data.net_weight {
                    data.net_weight = net_weight;
                }
                if let Some(axle_count) = weighing_data.axle_count {
                    data.axle_count = Some(axle_count);
                }
                if let Some(speed) = weighing_data.speed {
                    data.speed = Some(speed);
                }
                if let Some(lane_no) = weighing_data.lane_no {
                    data.lane_no = Some(lane_no);
                }
                if let Some(site_id) = weighing_data.site_id {
                    data.site_id = Some(site_id);
                }
                if let Some(status) = weighing_data.status {
                    data.status = status;
                }
                data.update_time = Some(chrono::Utc::now().naive_utc());
                Ok(Some(data))
            } else {
                Ok(None)
            }
        }

        async fn delete_weighing_data(&self, id: i32) -> Result<bool, anyhow::Error> {
            Ok(self.weighing_data.iter().any(|data| data.id == id))
        }

        async fn get_weighing_data_stats_by_vehicle(
            &self,
            _vehicle_id: i32,
            _start_time: chrono::NaiveDateTime,
            _end_time: chrono::NaiveDateTime,
        ) -> Result<Vec<WeighingData>, anyhow::Error> {
            // 简单实现,返回所有数据
            Ok(self.weighing_data.clone())
        }

        async fn get_weighing_data_stats_by_device(
            &self,
            _device_id: &str,
            _start_time: chrono::NaiveDateTime,
            _end_time: chrono::NaiveDateTime,
        ) -> Result<Vec<WeighingData>, anyhow::Error> {
            // 简单实现,返回所有数据
            Ok(self.weighing_data.clone())
        }
    }

    #[tokio::test]
    async fn test_get_weighing_data_list() {
        // 创建模拟仓库
        let now = NaiveDate::from_ymd_opt(2023, 1, 1)
            .unwrap()
            .and_hms_opt(10, 0, 0)
            .unwrap();
        let mock_repo = Arc::new(MockWeighingDataRepository {
            weighing_data: vec![WeighingData {
                id: 1,
                vehicle_id: 1,
                device_id: "DEV001".to_string(),
                weighing_time: now,
                gross_weight: 10.5,
                tare_weight: Some(3.5),
                net_weight: 7.0,
                axle_count: Some(3),
                speed: Some(40.5),
                lane_no: Some(1),
                site_id: Some(1),
                status: 1,
                create_time: now,
                update_time: None,
            }],
        });

        // 创建WeighingDataUseCases实例
        let weighing_data_use_cases = WeighingDataUseCases::new(mock_repo);

        // 测试获取称重数据列表
        let query = WeighingDataQuery::default();
        let (data, total) = weighing_data_use_cases
            .get_weighing_data_list(query)
            .await
            .unwrap();
        assert_eq!(data.len(), 1);
        assert_eq!(total, 1);
    }

    #[tokio::test]
    async fn test_get_weighing_data_by_id() {
        // 创建模拟仓库
        let now = NaiveDate::from_ymd_opt(2023, 1, 1)
            .unwrap()
            .and_hms_opt(10, 0, 0)
            .unwrap();
        let mock_repo = Arc::new(MockWeighingDataRepository {
            weighing_data: vec![WeighingData {
                id: 1,
                vehicle_id: 1,
                device_id: "DEV001".to_string(),
                weighing_time: now,
                gross_weight: 10.5,
                tare_weight: Some(3.5),
                net_weight: 7.0,
                axle_count: Some(3),
                speed: Some(40.5),
                lane_no: Some(1),
                site_id: Some(1),
                status: 1,
                create_time: now,
                update_time: None,
            }],
        });

        // 创建WeighingDataUseCases实例
        let weighing_data_use_cases = WeighingDataUseCases::new(mock_repo);

        // 测试获取存在的称重数据
        let data = weighing_data_use_cases
            .get_weighing_data_by_id(1)
            .await
            .unwrap();
        assert!(data.is_some());
        assert_eq!(data.unwrap().id, 1);

        // 测试获取不存在的称重数据
        let data = weighing_data_use_cases
            .get_weighing_data_by_id(999)
            .await
            .unwrap();
        assert!(data.is_none());
    }

    #[tokio::test]
    async fn test_create_weighing_data() {
        // 创建模拟仓库
        let now = NaiveDate::from_ymd_opt(2023, 1, 1)
            .unwrap()
            .and_hms_opt(10, 0, 0)
            .unwrap();
        let mock_repo = Arc::new(MockWeighingDataRepository {
            weighing_data: vec![],
        });

        // 创建WeighingDataUseCases实例
        let weighing_data_use_cases = WeighingDataUseCases::new(mock_repo);

        // 测试创建称重数据
        let weighing_data_create = WeighingDataCreate {
            vehicle_id: 1,
            device_id: "DEV001".to_string(),
            weighing_time: now,
            gross_weight: 10.5,
            tare_weight: Some(3.5),
            net_weight: 7.0,
            axle_count: Some(3),
            speed: Some(40.5),
            lane_no: Some(1),
            site_id: Some(1),
            status: 1,
        };

        let result = weighing_data_use_cases
            .create_weighing_data(weighing_data_create)
            .await;
        assert!(result.is_ok());

        // 测试毛重必须大于0
        let weighing_data_create = WeighingDataCreate {
            vehicle_id: 1,
            device_id: "DEV001".to_string(),
            weighing_time: now,
            gross_weight: 0.0,
            tare_weight: Some(3.5),
            net_weight: 7.0,
            axle_count: Some(3),
            speed: Some(40.5),
            lane_no: Some(1),
            site_id: Some(1),
            status: 1,
        };

        let result = weighing_data_use_cases
            .create_weighing_data(weighing_data_create)
            .await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "毛重必须大于0");
    }

    #[tokio::test]
    async fn test_update_weighing_data() {
        // 创建模拟仓库
        let now = NaiveDate::from_ymd_opt(2023, 1, 1)
            .unwrap()
            .and_hms_opt(10, 0, 0)
            .unwrap();
        let mock_repo = Arc::new(MockWeighingDataRepository {
            weighing_data: vec![WeighingData {
                id: 1,
                vehicle_id: 1,
                device_id: "DEV001".to_string(),
                weighing_time: now,
                gross_weight: 10.5,
                tare_weight: Some(3.5),
                net_weight: 7.0,
                axle_count: Some(3),
                speed: Some(40.5),
                lane_no: Some(1),
                site_id: Some(1),
                status: 1,
                create_time: now,
                update_time: None,
            }],
        });

        // 创建WeighingDataUseCases实例
        let weighing_data_use_cases = WeighingDataUseCases::new(mock_repo);

        // 测试更新称重数据
        let weighing_data_update = WeighingDataUpdate {
            vehicle_id: Some(2),
            device_id: Some("DEV002".to_string()),
            weighing_time: None,
            gross_weight: None,
            tare_weight: None,
            net_weight: None,
            axle_count: None,
            speed: None,
            lane_no: None,
            site_id: None,
            status: None,
        };

        let result = weighing_data_use_cases
            .update_weighing_data(1, weighing_data_update.clone())
            .await;
        assert!(result.is_ok());

        // 测试ID必须大于0
        let result = weighing_data_use_cases
            .update_weighing_data(0, weighing_data_update.clone())
            .await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "称重数据ID必须大于0");
    }

    #[tokio::test]
    async fn test_delete_weighing_data() {
        // 创建模拟仓库
        let now = NaiveDate::from_ymd_opt(2023, 1, 1)
            .unwrap()
            .and_hms_opt(10, 0, 0)
            .unwrap();
        let mock_repo = Arc::new(MockWeighingDataRepository {
            weighing_data: vec![WeighingData {
                id: 1,
                vehicle_id: 1,
                device_id: "DEV001".to_string(),
                weighing_time: now,
                gross_weight: 10.5,
                tare_weight: Some(3.5),
                net_weight: 7.0,
                axle_count: Some(3),
                speed: Some(40.5),
                lane_no: Some(1),
                site_id: Some(1),
                status: 1,
                create_time: now,
                update_time: None,
            }],
        });

        // 创建WeighingDataUseCases实例
        let weighing_data_use_cases = WeighingDataUseCases::new(mock_repo);

        // 测试删除称重数据
        let result = weighing_data_use_cases
            .delete_weighing_data(1)
            .await
            .unwrap();
        assert!(result);

        // 测试删除不存在的称重数据
        let result = weighing_data_use_cases
            .delete_weighing_data(999)
            .await
            .unwrap();
        assert!(!result);

        // 测试ID小于等于0
        let result = weighing_data_use_cases.delete_weighing_data(0).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "称重数据ID必须大于0");
    }
}
