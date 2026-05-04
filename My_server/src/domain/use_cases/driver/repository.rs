//! 司机仓库接口

use crate::domain::entities::driver::{
    Driver, DriverCreateRequest, DriverQuery, DriverUpdateRequest,
};

/// 司机仓库接口
#[async_trait::async_trait]
pub trait DriverRepository: Send + Sync {
    /// 获取司机列表
    async fn find_all(
        &self,
        page: i32,
        page_size: i32,
        query: DriverQuery,
    ) -> Result<(Vec<Driver>, i64), anyhow::Error>;

    /// 获取单个司机
    async fn find_by_id(&self, driver_id: i32) -> Result<Option<Driver>, anyhow::Error>;

    /// 创建司机
    async fn create(&self, driver: DriverCreateRequest) -> Result<Driver, anyhow::Error>;

    /// 更新司机
    async fn update(
        &self,
        driver_id: i32,
        driver: DriverUpdateRequest,
    ) -> Result<Driver, anyhow::Error>;

    /// 删除司机
    async fn delete(&self, driver_id: i32) -> Result<(), anyhow::Error>;

    /// 检查司机是否有关联数据
    async fn has_related_data(&self, driver_id: i32) -> Result<bool, anyhow::Error>;

    /// 检查司机是否存在
    async fn exists(&self, driver_id: i32) -> Result<bool, anyhow::Error>;

    /// 根据名称统计司机数量
    async fn count_by_name(
        &self,
        name: &str,
        exclude_id: Option<i32>,
    ) -> Result<i64, anyhow::Error>;

    /// 统计司机下的车辆数量
    async fn count_vehicles(&self, driver_id: i32) -> Result<i64, anyhow::Error>;

    /// 统计司机下的订单数量
    async fn count_orders(&self, driver_id: i32) -> Result<i64, anyhow::Error>;
}
