//! 车辆仓库接口
//!
//! 定义车辆数据的持久化接口

use crate::domain::entities::vehicle::{Vehicle, VehicleCreate, VehicleQuery, VehicleUpdate};

/// 车辆仓库接口
#[async_trait::async_trait]
pub trait VehicleRepository: Send + Sync {
    /// 获取车辆列表
    async fn get_vehicles(&self, query: VehicleQuery)
        -> Result<(Vec<Vehicle>, i64), anyhow::Error>;

    /// 获取单个车辆
    async fn get_vehicle(&self, vehicle_id: i32) -> Result<Option<Vehicle>, anyhow::Error>;

    /// 批量获取车辆信息 (数据库批量查询优化)
    async fn get_vehicles_batch(&self, vehicle_ids: &[i32]) -> Result<Vec<Vehicle>, anyhow::Error>;

    /// 创建车辆
    async fn create_vehicle(&self, vehicle: VehicleCreate) -> Result<Vehicle, anyhow::Error>;

    /// 更新车辆
    async fn update_vehicle(
        &self,
        vehicle_id: i32,
        vehicle: VehicleUpdate,
    ) -> Result<Option<Vehicle>, anyhow::Error>;

    /// 删除车辆
    async fn delete_vehicle(&self, vehicle_id: i32) -> Result<bool, anyhow::Error>;

    /// 检查车辆是否有关联数据
    async fn has_related_data(&self, vehicle_id: i32) -> Result<bool, anyhow::Error>;
}
