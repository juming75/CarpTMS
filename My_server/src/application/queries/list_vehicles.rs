//! 查询车辆列表

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::{PagedResult, Query, QueryHandler};
use crate::application::dto::VehicleDto;
use crate::domain::entities::vehicle::VehicleQuery;
use crate::domain::repositories::{SqlxVehicleRepository, VehicleRepository};
use crate::errors::AppResult;

/// 获取车辆列表查询
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListVehiclesQuery {
    /// 页码
    pub page: Option<i32>,
    /// 每页大小
    pub page_size: Option<i32>,
    /// 车辆名称
    pub vehicle_name: Option<String>,
    /// 车牌号
    pub license_plate: Option<String>,
    /// 车辆类型
    pub vehicle_type: Option<String>,
    /// 状态
    pub status: Option<i32>,
}

impl Query for ListVehiclesQuery {
    fn query_type() -> &'static str {
        "list_vehicles"
    }
}

impl ListVehiclesQuery {
    /// 转换为领域查询
    pub fn to_vehicle_query(&self) -> VehicleQuery {
        VehicleQuery {
            page: self.page,
            page_size: self.page_size,
            vehicle_name: self.vehicle_name.clone(),
            license_plate: self.license_plate.clone(),
            vehicle_type: self.vehicle_type.clone(),
            status: self.status,
        }
    }
}

/// 获取车辆列表查询处理器
pub struct ListVehiclesQueryHandler {
    pool: sqlx::PgPool,
    repository: SqlxVehicleRepository,
}

impl ListVehiclesQueryHandler {
    /// 创建处理器实例
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self {
            pool,
            repository: SqlxVehicleRepository::new(),
        }
    }
}

#[async_trait]
impl QueryHandler<ListVehiclesQuery, PagedResult<VehicleDto>> for ListVehiclesQueryHandler {
    async fn handle(&self, query: ListVehiclesQuery) -> AppResult<PagedResult<VehicleDto>> {
        // 转换为领域查询
        let vehicle_query = query.to_vehicle_query();

        // 查询车辆列表
        let (vehicles, total) = self.repository.find_all(&self.pool, vehicle_query).await?;

        // 转换为DTO
        let items: Vec<VehicleDto> = vehicles.into_iter().map(VehicleDto::from).collect();

        // 计算分页信息
        let page = query.page.unwrap_or(1);
        let page_size = query.page_size.unwrap_or(20);

        Ok(PagedResult::new(items, total, page, page_size))
    }
}
