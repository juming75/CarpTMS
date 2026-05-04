//! 订单用例实现

use std::sync::Arc;

use crate::domain::entities::order::{
    Order, OrderCreate, OrderItem, OrderItemCreate, OrderItemUpdate, OrderQuery, OrderUpdate,
};
use crate::domain::use_cases::order::repository::OrderRepository;

/// 订单用例结构
#[derive(Clone)]
pub struct OrderUseCases {
    order_repository: Arc<dyn OrderRepository + Send + Sync>,
}

impl OrderUseCases {
    pub fn new(order_repository: Arc<dyn OrderRepository>) -> Self {
        Self { order_repository }
    }

    /// 获取订单列表
    pub async fn get_orders(&self, query: OrderQuery) -> Result<(Vec<Order>, i64), anyhow::Error> {
        self.order_repository.get_orders(query).await
    }

    /// 获取单个订单
    pub async fn get_order(&self, order_id: i32) -> Result<Option<Order>, anyhow::Error> {
        self.order_repository.get_order(order_id).await
    }

    /// 创建订单
    pub async fn create_order(&self, order: OrderCreate) -> Result<Order, anyhow::Error> {
        // 数据验证
        if order.order_no.is_empty() {
            return Err(anyhow::anyhow!("订单编号不能为空"));
        }
        if order.customer_name.is_empty() {
            return Err(anyhow::anyhow!("客户名称不能为空"));
        }
        if order.customer_phone.is_empty() {
            return Err(anyhow::anyhow!("客户电话不能为空"));
        }
        if order.origin.is_empty() {
            return Err(anyhow::anyhow!("出发地不能为空"));
        }
        if order.destination.is_empty() {
            return Err(anyhow::anyhow!("目的地不能为空"));
        }
        if order.cargo_weight <= 0.0 {
            return Err(anyhow::anyhow!("货物重量必须大于0"));
        }
        if order.cargo_volume <= 0.0 {
            return Err(anyhow::anyhow!("货物体积必须大于0"));
        }
        if order.cargo_count <= 0 {
            return Err(anyhow::anyhow!("货物数量必须大于0"));
        }
        if order.order_amount <= 0.0 {
            return Err(anyhow::anyhow!("订单金额必须大于0"));
        }

        self.order_repository.create_order(order).await
    }

    /// 更新订单
    pub async fn update_order(
        &self,
        order_id: i32,
        order: OrderUpdate,
    ) -> Result<Option<Order>, anyhow::Error> {
        self.order_repository.update_order(order_id, order).await
    }

    /// 删除订单
    pub async fn delete_order(&self, order_id: i32) -> Result<bool, anyhow::Error> {
        self.order_repository.delete_order(order_id).await
    }

    /// 获取订单项列表
    pub async fn get_order_items(&self, order_id: i32) -> Result<Vec<OrderItem>, anyhow::Error> {
        self.order_repository.get_order_items(order_id).await
    }

    /// 创建订单项
    pub async fn create_order_item(
        &self,
        item: OrderItemCreate,
    ) -> Result<OrderItem, anyhow::Error> {
        if item.item_name.is_empty() {
            return Err(anyhow::anyhow!("商品名称不能为空"));
        }
        if item.quantity <= 0 {
            return Err(anyhow::anyhow!("数量必须大于0"));
        }
        if item.unit_price <= 0.0 {
            return Err(anyhow::anyhow!("单价必须大于0"));
        }
        self.order_repository.create_order_item(item).await
    }

    /// 更新订单项
    pub async fn update_order_item(
        &self,
        item_id: i32,
        item: OrderItemUpdate,
    ) -> Result<Option<OrderItem>, anyhow::Error> {
        self.order_repository.update_order_item(item_id, item).await
    }

    /// 删除订单项
    pub async fn delete_order_item(&self, item_id: i32) -> Result<bool, anyhow::Error> {
        self.order_repository.delete_order_item(item_id).await
    }
}
