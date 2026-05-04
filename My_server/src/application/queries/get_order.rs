//! 查询单个订单

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::Row;

use super::{Query, QueryHandler};
use crate::application::dto::OrderDto;
use crate::errors::{AppError, AppResult};

/// 获取单个订单查询
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetOrderQuery {
    /// 订单ID
    pub order_id: i32,
}

impl Query for GetOrderQuery {
    fn query_type() -> &'static str {
        "get_order"
    }
}

/// 获取单个订单查询处理器
pub struct GetOrderQueryHandler {
    pool: sqlx::PgPool,
}

impl GetOrderQueryHandler {
    /// 创建处理器实例
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl QueryHandler<GetOrderQuery, Option<OrderDto>> for GetOrderQueryHandler {
    async fn handle(&self, query: GetOrderQuery) -> AppResult<Option<OrderDto>> {
        // 验证查询
        if query.order_id <= 0 {
            return Err(AppError::validation_error("订单ID无效", None));
        }

        // 查询订单
        let result = sqlx::query(
            r#"SELECT 
                order_id, order_no, vehicle_id, driver_id, customer_name, customer_phone,
                origin, destination, cargo_type, cargo_weight, cargo_volume,
                cargo_count, order_amount, order_status, departure_time,
                arrival_time, remark, create_user_id, create_time, update_time
            FROM orders WHERE order_id = $1"#,
        )
        .bind(query.order_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::db_error(&format!("Failed to get order: {}", e), None))?;

        // 转换为DTO
        Ok(result.map(|row| OrderDto {
            order_id: row.try_get("order_id").unwrap_or(0),
            order_no: row.try_get("order_no").unwrap_or_default(),
            vehicle_id: row.try_get("vehicle_id").unwrap_or(0),
            driver_id: row.try_get("driver_id").ok(),
            customer_name: row.try_get("customer_name").unwrap_or_default(),
            customer_phone: row.try_get("customer_phone").unwrap_or_default(),
            origin: row.try_get("origin").unwrap_or_default(),
            destination: row.try_get("destination").unwrap_or_default(),
            cargo_type: row.try_get("cargo_type").unwrap_or_default(),
            cargo_weight: row.try_get("cargo_weight").unwrap_or(0.0),
            cargo_volume: row.try_get("cargo_volume").unwrap_or(0.0),
            cargo_count: row.try_get("cargo_count").unwrap_or(0),
            order_amount: row.try_get("order_amount").unwrap_or(0.0),
            order_status: row.try_get("order_status").unwrap_or(1),
            departure_time: row.try_get("departure_time").ok(),
            arrival_time: row.try_get("arrival_time").ok(),
            remark: row.try_get("remark").ok(),
            create_user_id: row.try_get("create_user_id").unwrap_or(1),
            create_time: row
                .try_get("create_time")
                .unwrap_or_else(|_| chrono::Utc::now().naive_utc()),
            update_time: row.try_get("update_time").ok(),
        }))
    }
}
