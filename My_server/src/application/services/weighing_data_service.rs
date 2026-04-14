use std::sync::Arc;
use chrono::NaiveDateTime;

use crate::domain::entities::weighing_data::{WeighingData, WeighingDataCreate, WeighingDataQuery, WeighingDataUpdate};
use crate::errors::AppResult;
use crate::domain::use_cases::weighing_data::WeighingDataRepository;

#[async_trait::async_trait]
pub trait WeighingDataService: Send + Sync {
    async fn get_weighing_data_list(&self, query: WeighingDataQuery) -> AppResult<(Vec<WeighingData>, i64)>;
    async fn get_weighing_data(&self, id: i32) -> AppResult<Option<WeighingData>>;
    async fn create_weighing_data(&self, data: WeighingDataCreate) -> AppResult<WeighingData>;
    async fn update_weighing_data(&self, id: i32, data: WeighingDataUpdate) -> AppResult<Option<WeighingData>>;
    async fn delete_weighing_data(&self, id: i32) -> AppResult<bool>;
    async fn get_weighing_data_stats_by_vehicle(&self, vehicle_id: i32, start_time: NaiveDateTime, end_time: NaiveDateTime) -> AppResult<Vec<WeighingData>>;
    async fn get_weighing_data_stats_by_device(&self, device_id: &str, start_time: NaiveDateTime, end_time: NaiveDateTime) -> AppResult<Vec<WeighingData>>;
}

pub struct WeighingDataServiceImpl {
    weighing_data_repository: Arc<dyn WeighingDataRepository>,
}

impl WeighingDataServiceImpl {
    pub fn new(weighing_data_repository: Arc<dyn WeighingDataRepository>) -> Self {
        Self {
            weighing_data_repository,
        }
    }
}

#[async_trait::async_trait]
impl WeighingDataService for WeighingDataServiceImpl {
    async fn get_weighing_data_list(&self, query: WeighingDataQuery) -> AppResult<(Vec<WeighingData>, i64)> {
        Ok(self.weighing_data_repository.get_weighing_data_list(query).await?)
    }

    async fn get_weighing_data(&self, id: i32) -> AppResult<Option<WeighingData>> {
        Ok(self.weighing_data_repository.get_weighing_data(id).await?)
    }

    async fn create_weighing_data(&self, data: WeighingDataCreate) -> AppResult<WeighingData> {
        Ok(self.weighing_data_repository.create_weighing_data(data).await?)
    }

    async fn update_weighing_data(&self, id: i32, data: WeighingDataUpdate) -> AppResult<Option<WeighingData>> {
        Ok(self.weighing_data_repository.update_weighing_data(id, data).await?)
    }

    async fn delete_weighing_data(&self, id: i32) -> AppResult<bool> {
        Ok(self.weighing_data_repository.delete_weighing_data(id).await?)
    }

    async fn get_weighing_data_stats_by_vehicle(&self, vehicle_id: i32, start_time: NaiveDateTime, end_time: NaiveDateTime) -> AppResult<Vec<WeighingData>> {
        Ok(self.weighing_data_repository.get_weighing_data_stats_by_vehicle(vehicle_id, start_time, end_time).await?)
    }

    async fn get_weighing_data_stats_by_device(&self, device_id: &str, start_time: NaiveDateTime, end_time: NaiveDateTime) -> AppResult<Vec<WeighingData>> {
        Ok(self.weighing_data_repository.get_weighing_data_stats_by_device(device_id, start_time, end_time).await?)
    }
}
