//! 查询订单列表

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::Row;

use super::{PagedResult, Query, QueryHandler};
use crate::application::dto::OrderDto;
use crate::domain::entities::order::OrderQuery;
use crate::errors::AppResult;

/// 获取订单列表查询
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListOrdersQuery {
    /// 页码
    pub page: Option<i32>,
    /// 每页大小
    pub page_size: Option<i32>,
    /// 订单编号
    pub order_no: Option<String>,
    /// 车辆ID
    pub vehicle_id: Option<i32>,
    /// 客户名称
    pub customer_name: Option<String>,
    /// 订单状态
    pub order_status: Option<i16>,
    /// 出发地
    pub origin: Option<String>,
    /// 目的地
    pub destination: Option<String>,
}

impl Query for ListOrdersQuery {
    fn query_type() -> &'static str {
        "list_orders"
    }
}

impl ListOrdersQuery {
    /// 转换为领域查询
    pub fn to_order_query(&self) -> OrderQuery {
        OrderQuery {
            page: self.page,
            page_size: self.page_size,
            order_no: self.order_no.clone(),
            vehicle_id: self.vehicle_id,
            customer_name: self.customer_name.clone(),
            order_status: self.order_status,
            origin: self.origin.clone(),
            destination: self.destination.clone(),
        }
    }
}

/// 获取订单列表查询处理器
pub struct ListOrdersQueryHandler {
    pool: sqlx::PgPool,
}

impl ListOrdersQueryHandler {
    /// 创建处理器实例
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl QueryHandler<ListOrdersQuery, PagedResult<OrderDto>> for ListOrdersQueryHandler {
    async fn handle(&self, query: ListOrdersQuery) -> AppResult<PagedResult<OrderDto>> {
        // 转换为领域查询
        let order_query = query.to_order_query();

        // 处理分页参数
        let page = order_query.page.unwrap_or(1);
        let page_size = order_query.page_size.unwrap_or(20);
        let offset = (page - 1) * page_size;

        // 构建动态查询条件
        let mut conditions = Vec::new();
        let mut param_index = 1;

        if let Some(order_no) = &order_query.order_no {
            if !order_no.is_empty() {
                conditions.push(format!("order_no LIKE ${}", param_index));
                param_index += 1;
            }
        }

        if let Some(_vehicle_id) = order_query.vehicle_id {
            conditions.push(format!("vehicle_id = ${}", param_index));
            param_index += 1;
        }

        if let Some(customer_name) = &order_query.customer_name {
            if !customer_name.is_empty() {
                conditions.push(format!("customer_name LIKE ${}", param_index));
                param_index += 1;
            }
        }

        if let Some(_order_status) = order_query.order_status {
            conditions.push(format!("order_status = ${}", param_index));
            param_index += 1;
        }

        if let Some(origin) = &order_query.origin {
            if !origin.is_empty() {
                conditions.push(format!("origin LIKE ${}", param_index));
                param_index += 1;
            }
        }

        if let Some(destination) = &order_query.destination {
            if !destination.is_empty() {
                conditions.push(format!("destination LIKE ${}", param_index));
                param_index += 1;
            }
        }

        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        };

        // 查询总数
        let count_sql = format!("SELECT COUNT(*) FROM orders {}", where_clause);
        let total: i64 = sqlx::query_scalar(&count_sql)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| {
                crate::errors::AppError::db_error(&format!("Failed to count orders: {}", e), None)
            })?;

        // 查询数据
        let data_sql = format!(
            r#"SELECT 
                order_id, order_no, vehicle_id, driver_id, customer_name, customer_phone,
                origin, destination, cargo_type, cargo_weight, cargo_volume,
                cargo_count, order_amount, order_status, departure_time,
                arrival_time, remark, create_user_id, create_time, update_time
            FROM orders {}
            ORDER BY order_id DESC
            LIMIT ${} OFFSET ${}"#,
            where_clause,
            param_index,
            param_index + 1
        );

        let rows = sqlx::query(&data_sql)
            .bind(page_size)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| {
                crate::errors::AppError::db_error(&format!("Failed to list orders: {}", e), None)
            })?;

        // 转换为DTO
        let items: Vec<OrderDto> = rows
            .iter()
            .map(|row| OrderDto {
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
            })
            .collect();

        Ok(PagedResult::new(items, total, page, page_size))
    }
}
