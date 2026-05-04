//! 称重数据仓库接口

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
    async fn get_weighing_data(&self, id: i64) -> Result<Option<WeighingData>, anyhow::Error>;

    /// 创建称重数据
    async fn create_weighing_data(
        &self,
        weighing_data: WeighingDataCreate,
    ) -> Result<WeighingData, anyhow::Error>;

    /// 更新称重数据
    async fn update_weighing_data(
        &self,
        id: i64,
        weighing_data: WeighingDataUpdate,
    ) -> Result<Option<WeighingData>, anyhow::Error>;

    /// 删除称重数据
    async fn delete_weighing_data(&self, id: i64) -> Result<bool, anyhow::Error>;

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
