use chrono::NaiveDateTime;
use std::sync::Arc;

use crate::domain::entities::weighing_data::{
    WeighingData, WeighingDataCreate, WeighingDataQuery, WeighingDataUpdate,
};
use crate::domain::event_logger::{DomainEventLog, DomainEventLogger}; // 统一事件日志
use crate::domain::use_cases::weighing_data::{
    WeighingDataRepository, // 来自 repository.rs
};
use crate::errors::AppResult;

#[async_trait::async_trait]
pub trait WeighingDataService: Send + Sync {
    async fn get_weighing_data_list(
        &self,
        query: WeighingDataQuery,
    ) -> AppResult<(Vec<WeighingData>, i64)>;
    async fn get_weighing_data(&self, id: i64) -> AppResult<Option<WeighingData>>;
    async fn create_weighing_data(&self, data: WeighingDataCreate) -> AppResult<WeighingData>;
    async fn update_weighing_data(
        &self,
        id: i64,
        data: WeighingDataUpdate,
    ) -> AppResult<Option<WeighingData>>;
    async fn delete_weighing_data(&self, id: i64) -> AppResult<bool>;
    async fn get_weighing_data_stats_by_vehicle(
        &self,
        vehicle_id: i32,
        start_time: NaiveDateTime,
        end_time: NaiveDateTime,
    ) -> AppResult<Vec<WeighingData>>;
    async fn get_weighing_data_stats_by_device(
        &self,
        device_id: &str,
        start_time: NaiveDateTime,
        end_time: NaiveDateTime,
    ) -> AppResult<Vec<WeighingData>>;
}

pub struct WeighingDataServiceImpl {
    weighing_data_repository: Arc<dyn WeighingDataRepository>,
    event_logger: Arc<DomainEventLogger>, // 统一事件日志
}

impl WeighingDataServiceImpl {
    pub fn new(
        weighing_data_repository: Arc<dyn WeighingDataRepository>,
        event_logger: Arc<DomainEventLogger>,
    ) -> Self {
        Self {
            weighing_data_repository,
            event_logger,
        }
    }

    /// 记录领域事件
    fn log_event(&self, event: DomainEventLog) {
        self.event_logger.log(event);
    }
}

#[async_trait::async_trait]
impl WeighingDataService for WeighingDataServiceImpl {
    async fn get_weighing_data_list(
        &self,
        query: WeighingDataQuery,
    ) -> AppResult<(Vec<WeighingData>, i64)> {
        Ok(self
            .weighing_data_repository
            .get_weighing_data_list(query)
            .await?)
    }

    async fn get_weighing_data(&self, id: i64) -> AppResult<Option<WeighingData>> {
        Ok(self.weighing_data_repository.get_weighing_data(id).await?)
    }

    async fn create_weighing_data(&self, data: WeighingDataCreate) -> AppResult<WeighingData> {
        let result = self
            .weighing_data_repository
            .create_weighing_data(data)
            .await?;

        // 记录创建事件
        let event = DomainEventLog::new(
            "Weighing",
            result.id,
            "Created",
            &serde_json::to_string(&result).unwrap_or_default(),
            "API",
            None,
        );
        self.log_event(event);

        Ok(result)
    }

    async fn update_weighing_data(
        &self,
        id: i64,
        data: WeighingDataUpdate,
    ) -> AppResult<Option<WeighingData>> {
        let result = self
            .weighing_data_repository
            .update_weighing_data(id, data)
            .await?;

        // 记录更新事件
        if let Some(ref weighing) = result {
            let event = DomainEventLog::new(
                "Weighing",
                weighing.id,
                "Updated",
                &serde_json::to_string(&weighing).unwrap_or_default(),
                "API",
                None,
            );
            self.log_event(event);
        }

        Ok(result)
    }

    async fn delete_weighing_data(&self, id: i64) -> AppResult<bool> {
        // 获取删除前的数据用于日志
        let old_data = self.weighing_data_repository.get_weighing_data(id).await?;

        let result = self
            .weighing_data_repository
            .delete_weighing_data(id)
            .await?;

        // 记录删除事件
        if result {
            let event = DomainEventLog::new(
                "Weighing",
                id,
                "Deleted",
                &serde_json::to_string(&old_data).unwrap_or_default(),
                "API",
                None,
            );
            self.log_event(event);
        }

        Ok(result)
    }

    async fn get_weighing_data_stats_by_vehicle(
        &self,
        vehicle_id: i32,
        start_time: NaiveDateTime,
        end_time: NaiveDateTime,
    ) -> AppResult<Vec<WeighingData>> {
        Ok(self
            .weighing_data_repository
            .get_weighing_data_stats_by_vehicle(vehicle_id, start_time, end_time)
            .await?)
    }

    async fn get_weighing_data_stats_by_device(
        &self,
        device_id: &str,
        start_time: NaiveDateTime,
        end_time: NaiveDateTime,
    ) -> AppResult<Vec<WeighingData>> {
        Ok(self
            .weighing_data_repository
            .get_weighing_data_stats_by_device(device_id, start_time, end_time)
            .await?)
    }
}
