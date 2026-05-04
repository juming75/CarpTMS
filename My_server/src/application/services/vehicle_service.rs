//! 车辆应用服务

use async_trait::async_trait;

use crate::application::{
    dto::{VehicleDto, VehicleSummaryDto},
    ApplicationService, PagedResult,
};
use crate::domain::entities::vehicle::{VehicleCreate, VehicleQuery, VehicleUpdate};
use crate::domain::repositories::{SqlxVehicleRepository, VehicleRepository};
use crate::errors::AppResult;

/// 车辆应用服务
pub struct VehicleApplicationService {
    pool: sqlx::PgPool,
    repository: SqlxVehicleRepository,
}

impl VehicleApplicationService {
    /// 创建服务实例
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self {
            pool,
            repository: SqlxVehicleRepository::new(),
        }
    }

    /// 创建车辆
    pub async fn create_vehicle(&self, vehicle: VehicleCreate) -> AppResult<VehicleDto> {
        let vehicle = self.repository.create(&self.pool, vehicle).await?;
        Ok(VehicleDto::from(vehicle))
    }

    /// 更新车辆
    pub async fn update_vehicle(
        &self,
        vehicle_id: i32,
        vehicle: VehicleUpdate,
    ) -> AppResult<Option<VehicleDto>> {
        let result = self
            .repository
            .update(&self.pool, vehicle_id, vehicle)
            .await?;
        Ok(result.map(VehicleDto::from))
    }

    /// 删除车辆
    pub async fn delete_vehicle(&self, vehicle_id: i32) -> AppResult<bool> {
        self.repository.delete(&self.pool, vehicle_id).await
    }

    /// 获取单个车辆
    pub async fn get_vehicle(&self, vehicle_id: i32) -> AppResult<Option<VehicleDto>> {
        let result = self.repository.find_by_id(&self.pool, vehicle_id).await?;
        Ok(result.map(VehicleDto::from))
    }

    /// 获取车辆列表
    pub async fn list_vehicles(&self, query: VehicleQuery) -> AppResult<PagedResult<VehicleDto>> {
        let (vehicles, total) = self.repository.find_all(&self.pool, query.clone()).await?;

        let items: Vec<VehicleDto> = vehicles.into_iter().map(VehicleDto::from).collect();

        let page = query.page.unwrap_or(1);
        let page_size = query.page_size.unwrap_or(20);

        Ok(PagedResult::new(items, total, page, page_size))
    }

    /// 获取车辆简要列表
    pub async fn list_vehicles_summary(
        &self,
        query: VehicleQuery,
    ) -> AppResult<PagedResult<VehicleSummaryDto>> {
        let (vehicles, total) = self.repository.find_all(&self.pool, query.clone()).await?;

        let items: Vec<VehicleSummaryDto> =
            vehicles.into_iter().map(VehicleSummaryDto::from).collect();

        let page = query.page.unwrap_or(1);
        let page_size = query.page_size.unwrap_or(20);

        Ok(PagedResult::new(items, total, page, page_size))
    }

    /// 批量获取车辆
    pub async fn get_vehicles_batch(&self, vehicle_ids: &[i32]) -> AppResult<Vec<VehicleDto>> {
        let vehicles = self.repository.find_by_ids(&self.pool, vehicle_ids).await?;
        Ok(vehicles.into_iter().map(VehicleDto::from).collect())
    }
}

#[async_trait]
impl ApplicationService for VehicleApplicationService {
    fn name(&self) -> &str {
        "VehicleApplicationService"
    }

    async fn initialize(&self) -> AppResult<()> {
        // 初始化逻辑（如果需要）
        Ok(())
    }
}

/// 车辆服务 trait（用于依赖注入）
#[async_trait]
pub trait VehicleService: Send + Sync {
    /// 创建车辆
    async fn create(&self, vehicle: VehicleCreate) -> AppResult<VehicleDto>;

    /// 更新车辆
    async fn update(
        &self,
        vehicle_id: i32,
        vehicle: VehicleUpdate,
    ) -> AppResult<Option<VehicleDto>>;

    /// 删除车辆
    async fn delete(&self, vehicle_id: i32) -> AppResult<bool>;

    /// 获取单个车辆
    async fn get(&self, vehicle_id: i32) -> AppResult<Option<VehicleDto>>;

    /// 获取车辆列表
    async fn list(&self, query: VehicleQuery) -> AppResult<PagedResult<VehicleDto>>;
}

#[async_trait]
impl VehicleService for VehicleApplicationService {
    async fn create(&self, vehicle: VehicleCreate) -> AppResult<VehicleDto> {
        self.create_vehicle(vehicle).await
    }

    async fn update(
        &self,
        vehicle_id: i32,
        vehicle: VehicleUpdate,
    ) -> AppResult<Option<VehicleDto>> {
        self.update_vehicle(vehicle_id, vehicle).await
    }

    async fn delete(&self, vehicle_id: i32) -> AppResult<bool> {
        self.delete_vehicle(vehicle_id).await
    }

    async fn get(&self, vehicle_id: i32) -> AppResult<Option<VehicleDto>> {
        self.get_vehicle(vehicle_id).await
    }

    async fn list(&self, query: VehicleQuery) -> AppResult<PagedResult<VehicleDto>> {
        self.list_vehicles(query).await
    }
}
