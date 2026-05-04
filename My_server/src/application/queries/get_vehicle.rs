//! 查询单个车辆

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::{Query, QueryHandler};
use crate::application::dto::VehicleDto;
use crate::domain::repositories::{SqlxVehicleRepository, VehicleRepository};
use crate::errors::{AppError, AppResult};

/// 获取单个车辆查询
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetVehicleQuery {
    /// 车辆ID
    pub vehicle_id: i32,
}

impl Query for GetVehicleQuery {
    fn query_type() -> &'static str {
        "get_vehicle"
    }
}

/// 获取单个车辆查询处理器
pub struct GetVehicleQueryHandler {
    pool: sqlx::PgPool,
    repository: SqlxVehicleRepository,
}

impl GetVehicleQueryHandler {
    /// 创建处理器实例
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self {
            pool,
            repository: SqlxVehicleRepository::new(),
        }
    }
}

#[async_trait]
impl QueryHandler<GetVehicleQuery, Option<VehicleDto>> for GetVehicleQueryHandler {
    async fn handle(&self, query: GetVehicleQuery) -> AppResult<Option<VehicleDto>> {
        // 验证查询
        if query.vehicle_id <= 0 {
            return Err(AppError::validation_error("车辆ID无效", None));
        }

        // 查询车辆
        let result = self
            .repository
            .find_by_id(&self.pool, query.vehicle_id)
            .await?;

        // 转换为DTO
        Ok(result.map(VehicleDto::from))
    }
}
