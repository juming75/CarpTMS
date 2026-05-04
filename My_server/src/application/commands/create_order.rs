//! 创建订单命令

use async_trait::async_trait;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::Row;

use super::{Command, CommandHandler, CommandResponse};
use crate::domain::entities::order::OrderCreate;
use crate::errors::{AppError, AppResult};

/// 创建订单命令
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateOrderCommand {
    /// 订单编号
    pub order_no: String,
    /// 车辆ID
    pub vehicle_id: i32,
    /// 司机ID
    pub driver_id: Option<i32>,
    /// 客户名称
    pub customer_name: String,
    /// 客户电话
    pub customer_phone: String,
    /// 出发地
    pub origin: String,
    /// 目的地
    pub destination: String,
    /// 货物类型
    pub cargo_type: String,
    /// 货物重量
    pub cargo_weight: f64,
    /// 货物体积
    pub cargo_volume: f64,
    /// 货物数量
    pub cargo_count: i32,
    /// 订单金额
    pub order_amount: f64,
    /// 订单状态
    pub order_status: i16,
    /// 出发时间
    pub departure_time: Option<NaiveDateTime>,
    /// 到达时间
    pub arrival_time: Option<NaiveDateTime>,
    /// 备注
    pub remark: Option<String>,
    /// 创建用户ID
    pub create_user_id: i32,
}

impl Command for CreateOrderCommand {
    fn command_type() -> &'static str {
        "create_order"
    }
}

impl CreateOrderCommand {
    /// 转换为领域实体
    pub fn to_order_create(&self) -> OrderCreate {
        OrderCreate {
            order_no: self.order_no.clone(),
            vehicle_id: self.vehicle_id,
            driver_id: self.driver_id,
            customer_name: self.customer_name.clone(),
            customer_phone: self.customer_phone.clone(),
            origin: self.origin.clone(),
            destination: self.destination.clone(),
            cargo_type: self.cargo_type.clone(),
            cargo_weight: self.cargo_weight,
            cargo_volume: self.cargo_volume,
            cargo_count: self.cargo_count,
            order_amount: self.order_amount,
            order_status: self.order_status,
            departure_time: self.departure_time,
            arrival_time: self.arrival_time,
            remark: self.remark.clone(),
            create_user_id: self.create_user_id,
        }
    }
}

/// 创建订单命令处理器
pub struct CreateOrderCommandHandler {
    pool: sqlx::PgPool,
}

impl CreateOrderCommandHandler {
    /// 创建处理器实例
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CommandHandler<CreateOrderCommand> for CreateOrderCommandHandler {
    async fn handle(&self, command: CreateOrderCommand) -> AppResult<CommandResponse> {
        // 验证命令
        if command.order_no.is_empty() {
            return Err(AppError::validation_error("订单编号不能为空", None));
        }

        if command.customer_name.is_empty() {
            return Err(AppError::validation_error("客户名称不能为空", None));
        }

        if command.customer_phone.is_empty() {
            return Err(AppError::validation_error("客户电话不能为空", None));
        }

        if command.origin.is_empty() {
            return Err(AppError::validation_error("出发地不能为空", None));
        }

        if command.destination.is_empty() {
            return Err(AppError::validation_error("目的地不能为空", None));
        }

        if command.cargo_weight <= 0.0 {
            return Err(AppError::validation_error("货物重量必须大于0", None));
        }

        if command.cargo_volume <= 0.0 {
            return Err(AppError::validation_error("货物体积必须大于0", None));
        }

        if command.cargo_count <= 0 {
            return Err(AppError::validation_error("货物数量必须大于0", None));
        }

        if command.order_amount <= 0.0 {
            return Err(AppError::validation_error("订单金额必须大于0", None));
        }

        // 转换为领域实体
        let order_create = command.to_order_create();

        // 插入数据库
        let row = sqlx::query(
            r#"INSERT INTO orders (
                order_no, vehicle_id, driver_id, customer_name, customer_phone,
                origin, destination, cargo_type, cargo_weight, cargo_volume,
                cargo_count, order_amount, order_status, departure_time,
                arrival_time, remark, create_user_id
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17
            ) RETURNING order_id"#,
        )
        .bind(&order_create.order_no)
        .bind(order_create.vehicle_id)
        .bind(order_create.driver_id)
        .bind(&order_create.customer_name)
        .bind(&order_create.customer_phone)
        .bind(&order_create.origin)
        .bind(&order_create.destination)
        .bind(&order_create.cargo_type)
        .bind(order_create.cargo_weight)
        .bind(order_create.cargo_volume)
        .bind(order_create.cargo_count)
        .bind(order_create.order_amount)
        .bind(order_create.order_status)
        .bind(order_create.departure_time)
        .bind(order_create.arrival_time)
        .bind(&order_create.remark)
        .bind(order_create.create_user_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::db_error(&format!("Failed to create order: {}", e), None))?;

        let order_id: i32 = row
            .try_get("order_id")
            .map_err(|e| AppError::db_error(&format!("Failed to get order_id: {}", e), None))?;

        Ok(CommandResponse::success_with_message(
            order_id,
            "订单创建成功",
        ))
    }
}
