//! 订单应用服务

use async_trait::async_trait;
use sqlx::Row;

use crate::application::{
    dto::{OrderDto, OrderItemDto},
    ApplicationService, PagedResult,
};
use crate::domain::entities::order::{OrderCreate, OrderItemCreate, OrderQuery, OrderUpdate};
use crate::errors::{AppError, AppResult};

/// 订单应用服务
pub struct OrderApplicationService {
    pool: sqlx::PgPool,
}

impl OrderApplicationService {
    /// 创建服务实例
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }

    /// 创建订单
    pub async fn create_order(&self, order: OrderCreate) -> AppResult<OrderDto> {
        // 验证
        if order.order_no.is_empty() {
            return Err(AppError::validation_error("订单编号不能为空", None));
        }

        if order.customer_name.is_empty() {
            return Err(AppError::validation_error("客户名称不能为空", None));
        }

        let row = sqlx::query(
            r#"INSERT INTO orders (
                order_no, vehicle_id, driver_id, customer_name, customer_phone,
                origin, destination, cargo_type, cargo_weight, cargo_volume, cargo_count,
                order_amount, order_status, departure_time, arrival_time, remark,
                create_time, update_time, create_user_id
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16,
                CURRENT_TIMESTAMP, NULL, $17
            ) RETURNING *"#,
        )
        .bind(&order.order_no)
        .bind(order.vehicle_id)
        .bind(order.driver_id)
        .bind(&order.customer_name)
        .bind(&order.customer_phone)
        .bind(&order.origin)
        .bind(&order.destination)
        .bind(&order.cargo_type)
        .bind(order.cargo_weight)
        .bind(order.cargo_volume)
        .bind(order.cargo_count)
        .bind(order.order_amount)
        .bind(order.order_status)
        .bind(order.departure_time)
        .bind(order.arrival_time)
        .bind(&order.remark)
        .bind(order.create_user_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::db_error(&format!("Failed to create order: {}", e), None))?;

        Ok(self.row_to_dto(&row))
    }

    /// 更新订单
    pub async fn update_order(
        &self,
        order_id: i32,
        order: OrderUpdate,
    ) -> AppResult<Option<OrderDto>> {
        let result = sqlx::query(
            r#"UPDATE orders 
               SET 
                   order_no = COALESCE($1, order_no),
                   vehicle_id = COALESCE($2, vehicle_id),
                   driver_id = COALESCE($3, driver_id),
                   customer_name = COALESCE($4, customer_name),
                   customer_phone = COALESCE($5, customer_phone),
                   origin = COALESCE($6, origin),
                   destination = COALESCE($7, destination),
                   cargo_type = COALESCE($8, cargo_type),
                   cargo_weight = COALESCE($9, cargo_weight),
                   cargo_volume = COALESCE($10, cargo_volume),
                   cargo_count = COALESCE($11, cargo_count),
                   order_amount = COALESCE($12, order_amount),
                   order_status = COALESCE($13, order_status),
                   departure_time = COALESCE($14, departure_time),
                   arrival_time = COALESCE($15, arrival_time),
                   remark = COALESCE($16, remark),
                   update_time = CURRENT_TIMESTAMP 
               WHERE order_id = $17 
               RETURNING *"#,
        )
        .bind(&order.order_no)
        .bind(order.vehicle_id)
        .bind(order.driver_id)
        .bind(&order.customer_name)
        .bind(&order.customer_phone)
        .bind(&order.origin)
        .bind(&order.destination)
        .bind(&order.cargo_type)
        .bind(order.cargo_weight)
        .bind(order.cargo_volume)
        .bind(order.cargo_count)
        .bind(order.order_amount)
        .bind(order.order_status)
        .bind(order.departure_time)
        .bind(order.arrival_time)
        .bind(&order.remark)
        .bind(order_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::db_error(&format!("Failed to update order: {}", e), None))?;

        Ok(result.as_ref().map(|r| self.row_to_dto(r)))
    }

    /// 删除订单
    pub async fn delete_order(&self, order_id: i32) -> AppResult<bool> {
        // 先删除订单项
        sqlx::query("DELETE FROM order_items WHERE order_id = $1")
            .bind(order_id)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::db_error(&format!("Failed to delete order items: {}", e), None))?;

        // 删除订单
        let result = sqlx::query("DELETE FROM orders WHERE order_id = $1")
            .bind(order_id)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::db_error(&format!("Failed to delete order: {}", e), None))?;

        Ok(result.rows_affected() > 0)
    }

    /// 获取单个订单
    pub async fn get_order(&self, order_id: i32) -> AppResult<Option<OrderDto>> {
        let result = sqlx::query(
            r#"SELECT * FROM orders WHERE order_id = $1"#,
        )
        .bind(order_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::db_error(&format!("Failed to get order: {}", e), None))?;

        Ok(result.as_ref().map(|r| self.row_to_dto(r)))
    }

    /// 获取订单列表
    pub async fn list_orders(&self, query: OrderQuery) -> AppResult<PagedResult<OrderDto>> {
        let page = query.page.unwrap_or(1);
        let page_size = query.page_size.unwrap_or(20);
        let offset = (page - 1) * page_size;

        // 构建查询条件
        let mut conditions = Vec::new();
        if let Some(order_no) = &query.order_no {
            if !order_no.is_empty() {
                conditions.push(format!("order_no LIKE '%{}%'", order_no.replace("'", "''")));
            }
        }
        if let Some(vehicle_id) = query.vehicle_id {
            conditions.push(format!("vehicle_id = {}", vehicle_id));
        }
        if let Some(customer_name) = &query.customer_name {
            if !customer_name.is_empty() {
                conditions.push(format!("customer_name LIKE '%{}%'", customer_name.replace("'", "''")));
            }
        }
        if let Some(order_status) = query.order_status {
            conditions.push(format!("order_status = {}", order_status));
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
            .map_err(|e| AppError::db_error(&format!("Failed to count orders: {}", e), None))?;

        // 查询数据
        let data_sql = format!(
            "SELECT * FROM orders {} ORDER BY order_id DESC LIMIT {} OFFSET {}",
            where_clause, page_size, offset
        );

        let rows = sqlx::query(&data_sql)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AppError::db_error(&format!("Failed to list orders: {}", e), None))?;

        let items: Vec<OrderDto> = rows.iter().map(|r| self.row_to_dto(r)).collect();

        Ok(PagedResult::new(items, total, page, page_size))
    }

    /// 获取订单项列表
    pub async fn get_order_items(&self, order_id: i32) -> AppResult<Vec<OrderItemDto>> {
        let rows = sqlx::query(
            r#"SELECT * FROM order_items WHERE order_id = $1 ORDER BY item_id"#,
        )
        .bind(order_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::db_error(&format!("Failed to get order items: {}", e), None))?;

        let items: Vec<OrderItemDto> = rows
            .iter()
            .map(|r| OrderItemDto {
                item_id: r.try_get("item_id").unwrap_or(0),
                order_id: r.try_get("order_id").unwrap_or(0),
                item_name: r.try_get("item_name").unwrap_or_default(),
                item_description: r.try_get("item_description").ok(),
                quantity: r.try_get("quantity").unwrap_or(0),
                unit_price: r.try_get("unit_price").unwrap_or(0.0),
                total_price: r.try_get("total_price").unwrap_or(0.0),
                create_time: r.try_get("create_time").unwrap_or_else(|_| chrono::Utc::now().naive_utc()),
                update_time: r.try_get("update_time").ok(),
            })
            .collect();

        Ok(items)
    }

    /// 创建订单项
    pub async fn create_order_item(&self, item: OrderItemCreate) -> AppResult<OrderItemDto> {
        let total_price = item.quantity as f64 * item.unit_price;

        let row = sqlx::query(
            r#"INSERT INTO order_items (
                order_id, item_name, item_description, quantity, unit_price, total_price
            ) VALUES ($1, $2, $3, $4, $5, $6) RETURNING *"#,
        )
        .bind(item.order_id)
        .bind(&item.item_name)
        .bind(&item.item_description)
        .bind(item.quantity)
        .bind(item.unit_price)
        .bind(total_price)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::db_error(&format!("Failed to create order item: {}", e), None))?;

        Ok(OrderItemDto {
            item_id: row.try_get("item_id").unwrap_or(0),
            order_id: row.try_get("order_id").unwrap_or(0),
            item_name: row.try_get("item_name").unwrap_or_default(),
            item_description: row.try_get("item_description").ok(),
            quantity: row.try_get("quantity").unwrap_or(0),
            unit_price: row.try_get("unit_price").unwrap_or(0.0),
            total_price: row.try_get("total_price").unwrap_or(0.0),
            create_time: row.try_get("create_time").unwrap_or_else(|_| chrono::Utc::now().naive_utc()),
            update_time: row.try_get("update_time").ok(),
        })
    }

    /// 更新订单项
    pub async fn update_order_item(
        &self,
        item_id: i32,
        item_name: Option<String>,
        item_description: Option<String>,
        quantity: Option<i32>,
        unit_price: Option<f64>,
    ) -> AppResult<Option<OrderItemDto>> {
        // 先获取当前订单项
        let current = sqlx::query(r#"SELECT * FROM order_items WHERE item_id = $1"#)
            .bind(item_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AppError::db_error(&format!("Failed to get order item: {}", e), None))?;

        let current = match current {
            Some(row) => row,
            None => return Ok(None),
        };

        let current_qty: i32 = current.try_get("quantity").unwrap_or(0);
        let current_price: f64 = current.try_get("unit_price").unwrap_or(0.0);

        let new_qty = quantity.unwrap_or(current_qty);
        let new_price = unit_price.unwrap_or(current_price);
        let new_total = new_qty as f64 * new_price;

        let row = sqlx::query(
            r#"UPDATE order_items 
               SET 
                   item_name = COALESCE($1, item_name),
                   item_description = COALESCE($2, item_description),
                   quantity = COALESCE($3, quantity),
                   unit_price = COALESCE($4, unit_price),
                   total_price = $5,
                   update_time = CURRENT_TIMESTAMP
               WHERE item_id = $6 
               RETURNING *"#,
        )
        .bind(&item_name)
        .bind(&item_description)
        .bind(quantity)
        .bind(unit_price)
        .bind(new_total)
        .bind(item_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::db_error(&format!("Failed to update order item: {}", e), None))?;

        Ok(row.as_ref().map(|r| OrderItemDto {
            item_id: r.try_get("item_id").unwrap_or(0),
            order_id: r.try_get("order_id").unwrap_or(0),
            item_name: r.try_get("item_name").unwrap_or_default(),
            item_description: r.try_get("item_description").ok(),
            quantity: r.try_get("quantity").unwrap_or(0),
            unit_price: r.try_get("unit_price").unwrap_or(0.0),
            total_price: r.try_get("total_price").unwrap_or(0.0),
            create_time: r.try_get("create_time").unwrap_or_else(|_| chrono::Utc::now().naive_utc()),
            update_time: r.try_get("update_time").ok(),
        }))
    }

    /// 删除订单项
    pub async fn delete_order_item(&self, item_id: i32) -> AppResult<bool> {
        let result = sqlx::query(r#"DELETE FROM order_items WHERE item_id = $1"#)
            .bind(item_id)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::db_error(&format!("Failed to delete order item: {}", e), None))?;

        Ok(result.rows_affected() > 0)
    }

    /// 创建物流跟踪记录
    #[allow(clippy::too_many_arguments)]
    pub async fn create_logistics_track(
        &self,
        order_id: i32,
        vehicle_id: i32,
        track_time: chrono::NaiveDateTime,
        latitude: f64,
        longitude: f64,
        address: String,
        status: i16,
        remark: Option<String>,
    ) -> AppResult<crate::models::LogisticsTrack> {
        let row = sqlx::query(
            r#"INSERT INTO logistics_tracks (
                order_id, vehicle_id, track_time, latitude, longitude, address, status, remark
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING *"#,
        )
        .bind(order_id)
        .bind(vehicle_id)
        .bind(track_time)
        .bind(latitude)
        .bind(longitude)
        .bind(&address)
        .bind(status)
        .bind(&remark)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::db_error(&format!("Failed to create logistics track: {}", e), None))?;

        // 更新订单状态
        sqlx::query(r#"UPDATE orders SET order_status = $1 WHERE order_id = $2"#)
            .bind(status)
            .bind(order_id)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::db_error(&format!("Failed to update order status: {}", e), None))?;

        Ok(crate::models::LogisticsTrack {
            track_id: row.try_get("track_id").unwrap_or(0),
            order_id: row.try_get("order_id").unwrap_or(0),
            vehicle_id: row.try_get("vehicle_id").unwrap_or(0),
            track_time: row.try_get::<Option<chrono::DateTime<chrono::Utc>>, _>("track_time").ok().flatten(),
            latitude: row.try_get("latitude").unwrap_or(0.0),
            longitude: row.try_get("longitude").unwrap_or(0.0),
            address: row.try_get("address").ok(),
            status: row.try_get("status").unwrap_or(1),
            remark: row.try_get("remark").ok(),
            create_time: row.try_get::<Option<chrono::DateTime<chrono::Utc>>, _>("create_time").ok().flatten().unwrap_or_else(chrono::Utc::now),
            created_at: chrono::Utc::now(),
        })
    }

    /// 更新物流跟踪记录
    #[allow(clippy::too_many_arguments)]
    pub async fn update_logistics_track(
        &self,
        track_id: i32,
        vehicle_id: Option<i32>,
        track_time: Option<chrono::NaiveDateTime>,
        latitude: Option<f64>,
        longitude: Option<f64>,
        address: Option<String>,
        status: Option<i16>,
        remark: Option<String>,
    ) -> AppResult<Option<crate::models::LogisticsTrack>> {
        let row = sqlx::query(
            r#"UPDATE logistics_tracks 
               SET 
                   vehicle_id = COALESCE($1, vehicle_id),
                   track_time = COALESCE($2, track_time),
                   latitude = COALESCE($3, latitude),
                   longitude = COALESCE($4, longitude),
                   address = COALESCE($5, address),
                   status = COALESCE($6, status),
                   remark = COALESCE($7, remark)
               WHERE track_id = $8 
               RETURNING *"#,
        )
        .bind(vehicle_id)
        .bind(track_time)
        .bind(latitude)
        .bind(longitude)
        .bind(&address)
        .bind(status)
        .bind(&remark)
        .bind(track_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::db_error(&format!("Failed to update logistics track: {}", e), None))?;

        if let Some(ref r) = row {
            let order_id: i32 = r.try_get("order_id").unwrap_or(0);
            if let Some(s) = status {
                sqlx::query(r#"UPDATE orders SET order_status = $1 WHERE order_id = $2"#)
                    .bind(s)
                    .bind(order_id)
                    .execute(&self.pool)
                    .await
                    .map_err(|e| AppError::db_error(&format!("Failed to update order status: {}", e), None))?;
            }
        }

        Ok(row.as_ref().map(|r| crate::models::LogisticsTrack {
            track_id: r.try_get("track_id").unwrap_or(0),
            order_id: r.try_get("order_id").unwrap_or(0),
            vehicle_id: r.try_get("vehicle_id").unwrap_or(0),
            track_time: r.try_get::<Option<chrono::DateTime<chrono::Utc>>, _>("track_time").ok().flatten(),
            latitude: r.try_get("latitude").unwrap_or(0.0),
            longitude: r.try_get("longitude").unwrap_or(0.0),
            address: r.try_get("address").ok(),
            status: r.try_get("status").unwrap_or(1),
            remark: r.try_get("remark").ok(),
            create_time: r.try_get::<Option<chrono::DateTime<chrono::Utc>>, _>("create_time").ok().flatten().unwrap_or_else(chrono::Utc::now),
            created_at: chrono::Utc::now(),
        }))
    }

    /// 删除物流跟踪记录
    pub async fn delete_logistics_track(&self, track_id: i32) -> AppResult<bool> {
        let result = sqlx::query(r#"DELETE FROM logistics_tracks WHERE track_id = $1"#)
            .bind(track_id)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::db_error(&format!("Failed to delete logistics track: {}", e), None))?;

        Ok(result.rows_affected() > 0)
    }

    /// 批量创建物流轨迹
    pub async fn create_logistics_tracks_batch(
        &self,
        order_id: i32,
        tracks: Vec<crate::schemas::LogisticsTrackCreate>,
    ) -> AppResult<Vec<crate::models::LogisticsTrack>> {
        let mut results = Vec::with_capacity(tracks.len());
        
        // 获取最后一个轨迹的状态，用于更新订单状态
        let last_track_status = tracks.last().map(|t| t.status);

        for track in tracks {
            let row = sqlx::query(
                r#"INSERT INTO logistics_tracks (
                    order_id, vehicle_id, track_time, latitude, longitude, address, status, remark
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING *"#,
            )
            .bind(order_id)
            .bind(track.vehicle_id)
            .bind(track.track_time)
            .bind(track.latitude)
            .bind(track.longitude)
            .bind(&track.address)
            .bind(track.status)
            .bind(&track.remark)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AppError::db_error(&format!("Failed to create logistics track: {}", e), None))?;

            results.push(crate::models::LogisticsTrack {
                track_id: row.try_get("track_id").unwrap_or(0),
                order_id: row.try_get("order_id").unwrap_or(0),
                vehicle_id: row.try_get("vehicle_id").unwrap_or(0),
                track_time: row.try_get::<Option<chrono::DateTime<chrono::Utc>>, _>("track_time").ok().flatten(),
                latitude: row.try_get("latitude").unwrap_or(0.0),
                longitude: row.try_get("longitude").unwrap_or(0.0),
                address: row.try_get("address").ok(),
                status: row.try_get("status").unwrap_or(1),
                remark: row.try_get("remark").ok(),
                create_time: row.try_get::<Option<chrono::DateTime<chrono::Utc>>, _>("create_time").ok().flatten().unwrap_or_else(chrono::Utc::now),
                created_at: chrono::Utc::now(),
            });
        }

        // 更新订单状态为最新轨迹的状态
        if let Some(status) = last_track_status {
            sqlx::query(r#"UPDATE orders SET order_status = $1 WHERE order_id = $2"#)
                .bind(status)
                .bind(order_id)
                .execute(&self.pool)
                .await
                .map_err(|e| AppError::db_error(&format!("Failed to update order status: {}", e), None))?;
        }

        Ok(results)
    }

    /// 获取车辆轨迹数据
    pub async fn get_vehicle_tracks(
        &self,
        vehicle_id: i32,
        start_time: chrono::NaiveDateTime,
        end_time: chrono::NaiveDateTime,
        page: i32,
        page_size: i32,
    ) -> AppResult<crate::application::PagedResult<crate::models::LogisticsTrack>> {
        let offset = (page - 1) * page_size;
        let is_paginated = page > 0 && page_size > 0;

        // 查询总数
        let total: i64 = sqlx::query_scalar(
            r#"SELECT COUNT(*) FROM logistics_tracks 
               WHERE vehicle_id = $1 AND track_time BETWEEN $2 AND $3"#,
        )
        .bind(vehicle_id)
        .bind(start_time)
        .bind(end_time)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::db_error(&format!("Failed to count tracks: {}", e), None))?;

        // 查询数据
        let rows = if is_paginated {
            sqlx::query(
                r#"SELECT * FROM logistics_tracks 
                   WHERE vehicle_id = $1 AND track_time BETWEEN $2 AND $3 
                   ORDER BY track_time ASC 
                   LIMIT $4 OFFSET $5"#,
            )
            .bind(vehicle_id)
            .bind(start_time)
            .bind(end_time)
            .bind(page_size as i64)
            .bind(offset as i64)
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query(
                r#"SELECT * FROM logistics_tracks 
                   WHERE vehicle_id = $1 AND track_time BETWEEN $2 AND $3 
                   ORDER BY track_time ASC"#,
            )
            .bind(vehicle_id)
            .bind(start_time)
            .bind(end_time)
            .fetch_all(&self.pool)
            .await
        }
        .map_err(|e| AppError::db_error(&format!("Failed to get vehicle tracks: {}", e), None))?;

        let items: Vec<crate::models::LogisticsTrack> = rows
            .iter()
            .map(|r| crate::models::LogisticsTrack {
                track_id: r.try_get("track_id").unwrap_or(0),
                order_id: r.try_get("order_id").unwrap_or(0),
                vehicle_id: r.try_get("vehicle_id").unwrap_or(0),
                track_time: r.try_get::<Option<chrono::DateTime<chrono::Utc>>, _>("track_time").ok().flatten(),
                latitude: r.try_get("latitude").unwrap_or(0.0),
                longitude: r.try_get("longitude").unwrap_or(0.0),
                address: r.try_get("address").ok(),
                status: r.try_get("status").unwrap_or(1),
                remark: r.try_get("remark").ok(),
                create_time: r.try_get::<Option<chrono::DateTime<chrono::Utc>>, _>("create_time").ok().flatten().unwrap_or_else(chrono::Utc::now),
                created_at: chrono::Utc::now(),
            })
            .collect();

        Ok(crate::application::PagedResult::new(items, total, page, page_size))
    }

    /// 获取车辆信息
    pub async fn get_vehicle_info(&self, vehicle_id: i32) -> AppResult<Option<(String, String)>> {
        let row = sqlx::query(
            r#"SELECT vehicle_name, license_plate FROM vehicles WHERE vehicle_id = $1"#,
        )
        .bind(vehicle_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::db_error(&format!("Failed to get vehicle info: {}", e), None))?;

        Ok(row.map(|r| {
            (
                r.try_get::<String, _>("vehicle_name").unwrap_or_default(),
                r.try_get::<String, _>("license_plate").unwrap_or_default(),
            )
        }))
    }

    /// 获取司机名称
    pub async fn get_driver_name(&self, driver_id: i32) -> AppResult<Option<String>> {
        let row = sqlx::query(r#"SELECT driver_name FROM drivers WHERE driver_id = $1"#)
            .bind(driver_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AppError::db_error(&format!("Failed to get driver name: {}", e), None))?;

        Ok(row.and_then(|r| r.try_get::<Option<String>, _>("driver_name").unwrap_or(None)))
    }

    /// 将数据库行转换为 DTO
    fn row_to_dto(&self, row: &sqlx::postgres::PgRow) -> OrderDto {
        OrderDto {
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
            create_time: row.try_get::<chrono::DateTime<chrono::Utc>, _>("create_time").map(|t| t.naive_utc()).unwrap_or_else(|_| chrono::Utc::now().naive_utc()),
            update_time: row.try_get::<Option<chrono::DateTime<chrono::Utc>>, _>("update_time").map(|t| t.map(|t| t.naive_utc())).unwrap_or(None),
        }
    }
}

#[async_trait]
impl ApplicationService for OrderApplicationService {
    fn name(&self) -> &str {
        "OrderApplicationService"
    }

    async fn initialize(&self) -> AppResult<()> {
        Ok(())
    }
}
