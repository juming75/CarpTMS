//! 搜索车辆

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::{PagedResult, Query, QueryHandler};
use crate::application::dto::VehicleDto;
use crate::domain::entities::vehicle::VehicleQuery;
use crate::domain::repositories::{SqlxVehicleRepository, VehicleRepository};
use crate::errors::AppResult;

/// 搜索车辆查询
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchVehiclesQuery {
    /// 搜索关键词
    pub keyword: String,
    /// 页码
    pub page: Option<i32>,
    /// 每页大小
    pub page_size: Option<i32>,
    /// 车辆类型（可选过滤）
    pub vehicle_type: Option<String>,
    /// 状态（可选过滤）
    pub status: Option<i32>,
}

impl Query for SearchVehiclesQuery {
    fn query_type() -> &'static str {
        "search_vehicles"
    }
}

/// 搜索车辆查询处理器
pub struct SearchVehiclesQueryHandler {
    pool: sqlx::PgPool,
    repository: SqlxVehicleRepository,
}

impl SearchVehiclesQueryHandler {
    /// 创建处理器实例
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self {
            pool,
            repository: SqlxVehicleRepository::new(),
        }
    }
}

#[async_trait]
impl QueryHandler<SearchVehiclesQuery, PagedResult<VehicleDto>> for SearchVehiclesQueryHandler {
    async fn handle(&self, query: SearchVehiclesQuery) -> AppResult<PagedResult<VehicleDto>> {
        // 构建领域查询（使用关键词搜索车辆名称和车牌号）
        let vehicle_query = VehicleQuery {
            page: query.page,
            page_size: query.page_size,
            vehicle_name: Some(query.keyword.clone()),
            license_plate: Some(query.keyword.clone()),
            vehicle_type: query.vehicle_type,
            status: query.status,
        };

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

/// 高级搜索车辆查询
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedSearchVehiclesQuery {
    /// 搜索关键词
    pub keyword: Option<String>,
    /// 车辆名称
    pub vehicle_name: Option<String>,
    /// 车牌号
    pub license_plate: Option<String>,
    /// 车辆类型
    pub vehicle_type: Option<String>,
    /// 车辆品牌
    pub vehicle_brand: Option<String>,
    /// 状态
    pub status: Option<i32>,
    /// 车组ID
    pub group_id: Option<i32>,
    /// 运营状态
    pub operation_status: Option<i32>,
    /// 页码
    pub page: Option<i32>,
    /// 每页大小
    pub page_size: Option<i32>,
}

impl Query for AdvancedSearchVehiclesQuery {
    fn query_type() -> &'static str {
        "advanced_search_vehicles"
    }
}

/// 高级搜索车辆查询处理器
pub struct AdvancedSearchVehiclesQueryHandler {
    pool: sqlx::PgPool,
    repository: SqlxVehicleRepository,
}

impl AdvancedSearchVehiclesQueryHandler {
    /// 创建处理器实例
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self {
            pool,
            repository: SqlxVehicleRepository::new(),
        }
    }
}

#[async_trait]
impl QueryHandler<AdvancedSearchVehiclesQuery, PagedResult<VehicleDto>>
    for AdvancedSearchVehiclesQueryHandler
{
    async fn handle(
        &self,
        query: AdvancedSearchVehiclesQuery,
    ) -> AppResult<PagedResult<VehicleDto>> {
        // 构建领域查询
        let vehicle_query = VehicleQuery {
            page: query.page,
            page_size: query.page_size,
            vehicle_name: query.vehicle_name.or(query.keyword.clone()),
            license_plate: query.license_plate,
            vehicle_type: query.vehicle_type,
            status: query.status,
        };

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
