//! / 订单仓库PostgreSQL实现

use anyhow::Result;
use sqlx::{PgPool, Row};
use std::sync::Arc;

use crate::domain::entities::order::{
    Order, OrderCreate, OrderItem, OrderItemCreate, OrderItemUpdate, OrderQuery, OrderUpdate,
};
use crate::domain::use_cases::order::OrderRepository;

/// 订单仓库PostgreSQL实现
pub struct PgOrderRepository {
    pool: Arc<PgPool>,
}

impl PgOrderRepository {
    /// 创建订单仓库实例
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl OrderRepository for PgOrderRepository {
    /// 获取订单列表
    async fn get_orders(&self, query: OrderQuery) -> Result<(Vec<Order>, i64), anyhow::Error> {
        // 处理分页参数
        let page = query.page.unwrap_or(1);
        let page_size = query.page_size.unwrap_or(20);
        let offset = (page - 1) * page_size;

        // 构建动态查询条件
        let mut where_clauses = Vec::new();
        let mut params = Vec::new();
        let mut param_index = 1;

        if let Some(order_no) = &query.order_no {
            if !order_no.is_empty() {
                where_clauses.push(format!("order_no LIKE ${}", param_index));
                params.push(format!("%{}%", order_no));
                param_index += 1;
            }
        }

        if let Some(vehicle_id) = query.vehicle_id {
            where_clauses.push(format!("vehicle_id = ${}", param_index));
            params.push(vehicle_id.to_string());
            param_index += 1;
        }

        if let Some(customer_name) = &query.customer_name {
            if !customer_name.is_empty() {
                where_clauses.push(format!("customer_name LIKE ${}", param_index));
                params.push(format!("%{}%", customer_name));
                param_index += 1;
            }
        }

        if let Some(order_status) = query.order_status {
            where_clauses.push(format!("order_status = ${}", param_index));
            params.push(order_status.to_string());
            param_index += 1;
        }

        if let Some(origin) = &query.origin {
            if !origin.is_empty() {
                where_clauses.push(format!("origin LIKE ${}", param_index));
                params.push(format!("%{}%", origin));
                param_index += 1;
            }
        }

        if let Some(destination) = &query.destination {
            if !destination.is_empty() {
                where_clauses.push(format!("destination LIKE ${}", param_index));
                params.push(format!("%{}%", destination));
                param_index += 1;
            }
        }

        // 构建完整查询
        let where_sql = if where_clauses.is_empty() {
            "".to_string()
        } else {
            format!("WHERE {}", where_clauses.join(" AND "))
        };

        // 查询总记录数
        let count_query = format!("SELECT COUNT(*) FROM orders {}", where_sql);
        let mut count_sqlx_query = sqlx::query_scalar::<_, i64>(&count_query);

        // 绑定参数
        for param in &params {
            count_sqlx_query = count_sqlx_query.bind(param);
        }

        let total_count = count_sqlx_query.fetch_one(&*self.pool).await?;

        // 查询分页数据
        let data_query = format!(
            "SELECT * FROM orders {}
             ORDER BY order_id DESC
             LIMIT ${} OFFSET ${}",
            where_sql,
            param_index,
            param_index + 1
        );

        // 使用query方法执行动态查询
        let mut sqlx_query = sqlx::query(&data_query);

        // 绑定参数
        for param in params {
            sqlx_query = sqlx_query.bind(param);
        }
        sqlx_query = sqlx_query.bind(page_size).bind(offset);

        let orders = sqlx_query
            .fetch_all(&*self.pool)
            .await?
            .into_iter()
            .map(|row| Order {
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
            .collect::<Vec<Order>>();

        Ok((orders, total_count))
    }

    /// 获取单个订单
    async fn get_order(&self, order_id: i32) -> Result<Option<Order>, anyhow::Error> {
        let order = sqlx::query(r#"SELECT * FROM orders WHERE order_id = $1"#)
            .bind(order_id)
            .fetch_optional(&*self.pool)
            .await?
            .map(|row| Order {
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
            });

        Ok(order)
    }

    /// 创建订单
    async fn create_order(&self, order: OrderCreate) -> Result<Order, anyhow::Error> {
        let result = sqlx::query(
            r#"INSERT INTO orders (
                order_no, vehicle_id, driver_id, customer_name, customer_phone,
                origin, destination, cargo_type, cargo_weight, cargo_volume,
                cargo_count, order_amount, order_status, departure_time,
                arrival_time, remark, create_user_id
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17
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
        .fetch_one(&*self.pool)
        .await;

        let o = match result {
            Ok(row) => Order {
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
            },
            Err(e) => {
                return Err(anyhow::anyhow!("Failed to create order: {:?}", e));
            }
        };

        Ok(o)
    }

    /// 更新订单
    async fn update_order(
        &self,
        order_id: i32,
        order: OrderUpdate,
    ) -> Result<Option<Order>, anyhow::Error> {
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
        .fetch_optional(&*self.pool)
        .await;

        match result {
            Ok(Some(row)) => {
                let order = Order {
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
                };

                Ok(Some(order))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(anyhow::anyhow!("Failed to update order: {:?}", e)),
        }
    }

    /// 删除订单
    async fn delete_order(&self, order_id: i32) -> Result<bool, anyhow::Error> {
        // 先删除相关的订单项
        let _item_result = sqlx::query(r#"DELETE FROM order_items WHERE order_id = $1"#)
            .bind(order_id)
            .execute(&*self.pool)
            .await?;

        // 再删除订单
        let order_result = sqlx::query(r#"DELETE FROM orders WHERE order_id = $1"#)
            .bind(order_id)
            .execute(&*self.pool)
            .await?;

        Ok(order_result.rows_affected() > 0)
    }

    /// 获取订单项列表
    async fn get_order_items(&self, order_id: i32) -> Result<Vec<OrderItem>, anyhow::Error> {
        let items =
            sqlx::query(r#"SELECT * FROM order_items WHERE order_id = $1 ORDER BY item_id"#)
                .bind(order_id)
                .fetch_all(&*self.pool)
                .await?
                .into_iter()
                .map(|row| OrderItem {
                    item_id: row.try_get("item_id").unwrap_or(0),
                    order_id: row.try_get("order_id").unwrap_or(0),
                    item_name: row.try_get("item_name").unwrap_or_default(),
                    item_description: row.try_get("item_description").ok(),
                    quantity: row.try_get("quantity").unwrap_or(0),
                    unit_price: row.try_get("unit_price").unwrap_or(0.0),
                    total_price: row.try_get("total_price").unwrap_or(0.0),
                    create_time: row
                        .try_get("create_time")
                        .unwrap_or_else(|_| chrono::Utc::now().naive_utc()),
                    update_time: row.try_get("update_time").ok(),
                })
                .collect::<Vec<OrderItem>>();

        Ok(items)
    }

    /// 创建订单项
    async fn create_order_item(&self, item: OrderItemCreate) -> Result<OrderItem, anyhow::Error> {
        // 计算总价
        let total_price = item.quantity as f64 * item.unit_price;

        let result = sqlx::query(
            r#"INSERT INTO order_items (
                order_id, item_name, item_description, quantity, unit_price, total_price
            ) VALUES (
                $1, $2, $3, $4, $5, $6
            ) RETURNING *"#,
        )
        .bind(item.order_id)
        .bind(&item.item_name)
        .bind(&item.item_description)
        .bind(item.quantity)
        .bind(item.unit_price)
        .bind(total_price)
        .fetch_one(&*self.pool)
        .await;

        let i = match result {
            Ok(row) => OrderItem {
                item_id: row.try_get("item_id").unwrap_or(0),
                order_id: row.try_get("order_id").unwrap_or(0),
                item_name: row.try_get("item_name").unwrap_or_default(),
                item_description: row.try_get("item_description").ok(),
                quantity: row.try_get("quantity").unwrap_or(0),
                unit_price: row.try_get("unit_price").unwrap_or(0.0),
                total_price: row.try_get("total_price").unwrap_or(0.0),
                create_time: row
                    .try_get("create_time")
                    .unwrap_or_else(|_| chrono::Utc::now().naive_utc()),
                update_time: row.try_get("update_time").ok(),
            },
            Err(e) => {
                return Err(anyhow::anyhow!("Failed to create order item: {:?}", e));
            }
        };

        Ok(i)
    }

    /// 更新订单项
    async fn update_order_item(
        &self,
        item_id: i32,
        item: OrderItemUpdate,
    ) -> Result<Option<OrderItem>, anyhow::Error> {
        let result = sqlx::query(
            r#"UPDATE order_items 
               SET 
                   item_name = COALESCE($1, item_name),
                   item_description = COALESCE($2, item_description),
                   quantity = COALESCE($3, quantity),
                   unit_price = COALESCE($4, unit_price),
                   total_price = COALESCE($3, quantity) * COALESCE($4, unit_price),
                   update_time = CURRENT_TIMESTAMP 
               WHERE item_id = $5 
               RETURNING *"#,
        )
        .bind(&item.item_name)
        .bind(&item.item_description)
        .bind(item.quantity)
        .bind(item.unit_price)
        .bind(item_id)
        .fetch_optional(&*self.pool)
        .await;

        match result {
            Ok(Some(row)) => {
                let order_item = OrderItem {
                    item_id: row.try_get("item_id").unwrap_or(0),
                    order_id: row.try_get("order_id").unwrap_or(0),
                    item_name: row.try_get("item_name").unwrap_or_default(),
                    item_description: row.try_get("item_description").ok(),
                    quantity: row.try_get("quantity").unwrap_or(0),
                    unit_price: row.try_get("unit_price").unwrap_or(0.0),
                    total_price: row.try_get("total_price").unwrap_or(0.0),
                    create_time: row
                        .try_get("create_time")
                        .unwrap_or_else(|_| chrono::Utc::now().naive_utc()),
                    update_time: row.try_get("update_time").ok(),
                };

                Ok(Some(order_item))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(anyhow::anyhow!("Failed to update order item: {:?}", e)),
        }
    }

    /// 删除订单项
    async fn delete_order_item(&self, item_id: i32) -> Result<bool, anyhow::Error> {
        let result = sqlx::query(r#"DELETE FROM order_items WHERE item_id = $1"#)
            .bind(item_id)
            .execute(&*self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }
}
