//! 更新订单命令

use async_trait::async_trait;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use super::{Command, CommandHandler, CommandResponse};
use crate::domain::entities::order::OrderUpdate;
use crate::errors::{AppError, AppResult};

/// 更新订单命令
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateOrderCommand {
    /// 订单ID
    pub order_id: i32,
    /// 订单编号
    pub order_no: Option<String>,
    /// 车辆ID
    pub vehicle_id: Option<i32>,
    /// 司机ID
    pub driver_id: Option<i32>,
    /// 客户名称
    pub customer_name: Option<String>,
    /// 客户电话
    pub customer_phone: Option<String>,
    /// 出发地
    pub origin: Option<String>,
    /// 目的地
    pub destination: Option<String>,
    /// 货物类型
    pub cargo_type: Option<String>,
    /// 货物重量
    pub cargo_weight: Option<f64>,
    /// 货物体积
    pub cargo_volume: Option<f64>,
    /// 货物数量
    pub cargo_count: Option<i32>,
    /// 订单金额
    pub order_amount: Option<f64>,
    /// 订单状态
    pub order_status: Option<i16>,
    /// 出发时间
    pub departure_time: Option<NaiveDateTime>,
    /// 到达时间
    pub arrival_time: Option<NaiveDateTime>,
    /// 备注
    pub remark: Option<String>,
}

impl Command for UpdateOrderCommand {
    fn command_type() -> &'static str {
        "update_order"
    }
}

impl UpdateOrderCommand {
    /// 转换为领域实体
    pub fn to_order_update(&self) -> OrderUpdate {
        OrderUpdate {
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
        }
    }
}

/// 更新订单命令处理器
pub struct UpdateOrderCommandHandler {
    pool: sqlx::PgPool,
}

impl UpdateOrderCommandHandler {
    /// 创建处理器实例
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CommandHandler<UpdateOrderCommand> for UpdateOrderCommandHandler {
    async fn handle(&self, command: UpdateOrderCommand) -> AppResult<CommandResponse> {
        // 验证命令
        if command.order_id <= 0 {
            return Err(AppError::validation_error("订单ID无效", None));
        }

        // 验证数值字段
        if let Some(cargo_weight) = command.cargo_weight {
            if cargo_weight <= 0.0 {
                return Err(AppError::validation_error("货物重量必须大于0", None));
            }
        }

        if let Some(cargo_volume) = command.cargo_volume {
            if cargo_volume <= 0.0 {
                return Err(AppError::validation_error("货物体积必须大于0", None));
            }
        }

        if let Some(cargo_count) = command.cargo_count {
            if cargo_count <= 0 {
                return Err(AppError::validation_error("货物数量必须大于0", None));
            }
        }

        if let Some(order_amount) = command.order_amount {
            if order_amount <= 0.0 {
                return Err(AppError::validation_error("订单金额必须大于0", None));
            }
        }

        // 转换为领域实体
        let order_update = command.to_order_update();

        // 更新数据库
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
               RETURNING order_id"#,
        )
        .bind(&order_update.order_no)
        .bind(order_update.vehicle_id)
        .bind(order_update.driver_id)
        .bind(&order_update.customer_name)
        .bind(&order_update.customer_phone)
        .bind(&order_update.origin)
        .bind(&order_update.destination)
        .bind(&order_update.cargo_type)
        .bind(order_update.cargo_weight)
        .bind(order_update.cargo_volume)
        .bind(order_update.cargo_count)
        .bind(order_update.order_amount)
        .bind(order_update.order_status)
        .bind(order_update.departure_time)
        .bind(order_update.arrival_time)
        .bind(&order_update.remark)
        .bind(command.order_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::db_error(&format!("Failed to update order: {}", e), None))?;

        match result {
            Some(_) => Ok(CommandResponse::success_with_message(
                command.order_id,
                "订单更新成功",
            )),
            None => Err(AppError::not_found_error("订单不存在".to_string())),
        }
    }
}
