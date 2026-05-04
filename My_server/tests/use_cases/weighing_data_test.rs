//! 称重数据用例集成测试
//!
//! 独立的集成测试，不依赖内嵌测试模块

use carptms::domain::entities::weighing_data::{
    WeighingData, WeighingDataCreate, WeighingDataQuery, WeighingDataUpdate,
};
use carptms::domain::use_cases::weighing_data::repository::WeighingDataRepository;
use carptms::domain::use_cases::weighing_data::service::WeighingDataUseCases;
use async_trait::async_trait;
use chrono::NaiveDate;
use std::sync::Arc;

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
        Ok((self.weighing_data.clone(), self.weighing_data.len() as i64))
    }

    async fn get_weighing_data(&self, id: i64) -> Result<Option<WeighingData>, anyhow::Error> {
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
            id: self.weighing_data.len() as i64 + 1,
            vehicle_id: weighing_data.vehicle_id,
            vehicle_name: "测试车辆".to_string(),
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
        id: i64,
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

    async fn delete_weighing_data(&self, id: i64) -> Result<bool, anyhow::Error> {
        Ok(self.weighing_data.iter().any(|data| data.id == id))
    }

    async fn get_weighing_data_stats_by_vehicle(
        &self,
        _vehicle_id: i32,
        _start_time: chrono::NaiveDateTime,
        _end_time: chrono::NaiveDateTime,
    ) -> Result<Vec<WeighingData>, anyhow::Error> {
        Ok(self.weighing_data.clone())
    }

    async fn get_weighing_data_stats_by_device(
        &self,
        _device_id: &str,
        _start_time: chrono::NaiveDateTime,
        _end_time: chrono::NaiveDateTime,
    ) -> Result<Vec<WeighingData>, anyhow::Error> {
        Ok(self.weighing_data.clone())
    }
}

fn create_test_data() -> Vec<WeighingData> {
    let now = NaiveDate::from_ymd_opt(2023, 1, 1)
        .unwrap()
        .and_hms_opt(10, 0, 0)
        .unwrap();
    vec![WeighingData {
        id: 1,
        vehicle_id: 1,
        vehicle_name: "测试车辆".to_string(),
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
    }]
}

#[tokio::test]
async fn test_get_weighing_data_list() {
    let mock_repo = Arc::new(MockWeighingDataRepository {
        weighing_data: create_test_data(),
    });
    let use_cases = WeighingDataUseCases::new(mock_repo);

    let query = WeighingDataQuery::default();
    let (data, total) = use_cases.get_weighing_data_list(query).await.unwrap();
    assert_eq!(data.len(), 1);
    assert_eq!(total, 1);
}

#[tokio::test]
async fn test_get_weighing_data_by_id() {
    let mock_repo = Arc::new(MockWeighingDataRepository {
        weighing_data: create_test_data(),
    });
    let use_cases = WeighingDataUseCases::new(mock_repo);

    let data = use_cases.get_weighing_data_by_id(1).await.unwrap();
    assert!(data.is_some());
    assert_eq!(data.unwrap().id, 1);

    let data = use_cases.get_weighing_data_by_id(999).await.unwrap();
    assert!(data.is_none());
}

#[tokio::test]
async fn test_create_weighing_data() {
    let mock_repo = Arc::new(MockWeighingDataRepository {
        weighing_data: vec![],
    });
    let use_cases = WeighingDataUseCases::new(mock_repo);

    let now = NaiveDate::from_ymd_opt(2023, 1, 1)
        .unwrap()
        .and_hms_opt(10, 0, 0)
        .unwrap();
    let create_data = WeighingDataCreate {
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

    let result = use_cases.create_weighing_data(create_data).await;
    assert!(result.is_ok());

    // 测试毛重必须大于0
    let invalid_data = WeighingDataCreate {
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
    let result = use_cases.create_weighing_data(invalid_data).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().to_string(), "毛重必须大于0");
}

#[tokio::test]
async fn test_update_weighing_data() {
    let mock_repo = Arc::new(MockWeighingDataRepository {
        weighing_data: create_test_data(),
    });
    let use_cases = WeighingDataUseCases::new(mock_repo);

    let update_data = WeighingDataUpdate {
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

    let result = use_cases.update_weighing_data(1, update_data.clone()).await;
    assert!(result.is_ok());

    // 测试ID必须大于0
    let result = use_cases.update_weighing_data(0, update_data).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().to_string(), "称重数据ID必须大于0");
}

#[tokio::test]
async fn test_delete_weighing_data() {
    let mock_repo = Arc::new(MockWeighingDataRepository {
        weighing_data: create_test_data(),
    });
    let use_cases = WeighingDataUseCases::new(mock_repo);

    let result = use_cases.delete_weighing_data(1).await.unwrap();
    assert!(result);

    // 测试删除不存在的称重数据
    let result = use_cases.delete_weighing_data(999).await.unwrap();
    assert!(!result);

    // 测试ID小于等于0
    let result = use_cases.delete_weighing_data(0).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().to_string(), "称重数据ID必须大于0");
}
