//! 订单 DTO

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::domain::entities::order::{Order, OrderItem};

/// 订单 DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderDto {
    /// 订单ID
    pub order_id: i32,
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
    /// 创建时间
    pub create_time: NaiveDateTime,
    /// 更新时间
    pub update_time: Option<NaiveDateTime>,
}

/// 从领域实体转换为 DTO
impl From<Order> for OrderDto {
    fn from(order: Order) -> Self {
        Self {
            order_id: order.order_id,
            order_no: order.order_no,
            vehicle_id: order.vehicle_id,
            driver_id: order.driver_id,
            customer_name: order.customer_name,
            customer_phone: order.customer_phone,
            origin: order.origin,
            destination: order.destination,
            cargo_type: order.cargo_type,
            cargo_weight: order.cargo_weight,
            cargo_volume: order.cargo_volume,
            cargo_count: order.cargo_count,
            order_amount: order.order_amount,
            order_status: order.order_status,
            departure_time: order.departure_time,
            arrival_time: order.arrival_time,
            remark: order.remark,
            create_user_id: order.create_user_id,
            create_time: order.create_time,
            update_time: order.update_time,
        }
    }
}

/// 订单项 DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderItemDto {
    /// 订单项ID
    pub item_id: i32,
    /// 订单ID
    pub order_id: i32,
    /// 商品名称
    pub item_name: String,
    /// 商品描述
    pub item_description: Option<String>,
    /// 数量
    pub quantity: i32,
    /// 单价
    pub unit_price: f64,
    /// 总价
    pub total_price: f64,
    /// 创建时间
    pub create_time: NaiveDateTime,
    /// 更新时间
    pub update_time: Option<NaiveDateTime>,
}

impl From<OrderItem> for OrderItemDto {
    fn from(item: OrderItem) -> Self {
        Self {
            item_id: item.item_id,
            order_id: item.order_id,
            item_name: item.item_name,
            item_description: item.item_description,
            quantity: item.quantity,
            unit_price: item.unit_price,
            total_price: item.total_price,
            create_time: item.create_time,
            update_time: item.update_time,
        }
    }
}

/// 订单详情 DTO（包含订单项）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderDetailDto {
    /// 订单信息
    pub order: OrderDto,
    /// 订单项列表
    pub items: Vec<OrderItemDto>,
}

/// 订单简要 DTO（用于列表显示）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderSummaryDto {
    /// 订单ID
    pub order_id: i32,
    /// 订单编号
    pub order_no: String,
    /// 客户名称
    pub customer_name: String,
    /// 出发地
    pub origin: String,
    /// 目的地
    pub destination: String,
    /// 订单金额
    pub order_amount: f64,
    /// 订单状态
    pub order_status: i16,
    /// 创建时间
    pub create_time: NaiveDateTime,
}

impl From<Order> for OrderSummaryDto {
    fn from(order: Order) -> Self {
        Self {
            order_id: order.order_id,
            order_no: order.order_no,
            customer_name: order.customer_name,
            origin: order.origin,
            destination: order.destination,
            order_amount: order.order_amount,
            order_status: order.order_status,
            create_time: order.create_time,
        }
    }
}
