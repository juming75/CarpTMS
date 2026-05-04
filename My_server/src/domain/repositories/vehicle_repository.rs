//! / 车辆仓库接口定义

use crate::domain::entities::vehicle::{Vehicle, VehicleCreate, VehicleQuery, VehicleUpdate};

/// 车辆仓库接口
#[async_trait::async_trait]
pub trait VehicleRepository: Send + Sync {
    /// 获取车辆列表
    async fn find_all(
        &self,
        pool: &sqlx::PgPool,
        query: VehicleQuery,
    ) -> crate::errors::AppResult<(Vec<Vehicle>, i64)>;

    /// 根据ID获取单个车辆
    async fn find_by_id(
        &self,
        pool: &sqlx::PgPool,
        vehicle_id: i32,
    ) -> crate::errors::AppResult<Option<Vehicle>>;

    /// 批量获取车辆信息
    async fn find_by_ids(
        &self,
        pool: &sqlx::PgPool,
        vehicle_ids: &[i32],
    ) -> crate::errors::AppResult<Vec<Vehicle>>;

    /// 创建车辆
    async fn create(
        &self,
        pool: &sqlx::PgPool,
        vehicle: VehicleCreate,
    ) -> crate::errors::AppResult<Vehicle>;

    /// 更新车辆
    async fn update(
        &self,
        pool: &sqlx::PgPool,
        vehicle_id: i32,
        vehicle: VehicleUpdate,
    ) -> crate::errors::AppResult<Option<Vehicle>>;

    /// 删除车辆
    async fn delete(&self, pool: &sqlx::PgPool, vehicle_id: i32) -> crate::errors::AppResult<bool>;
}
