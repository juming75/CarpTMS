//! 订单仓库接口

use crate::domain::entities::order::{
    Order, OrderCreate, OrderItem, OrderItemCreate, OrderItemUpdate, OrderQuery, OrderUpdate,
};

/// 订单仓库接口
#[async_trait::async_trait]
pub trait OrderRepository: Send + Sync {
    /// 获取订单列表
    async fn get_orders(&self, query: OrderQuery) -> Result<(Vec<Order>, i64), anyhow::Error>;

    /// 获取单个订单
    async fn get_order(&self, order_id: i32) -> Result<Option<Order>, anyhow::Error>;

    /// 创建订单
    async fn create_order(&self, order: OrderCreate) -> Result<Order, anyhow::Error>;

    /// 更新订单
    async fn update_order(
        &self,
        order_id: i32,
        order: OrderUpdate,
    ) -> Result<Option<Order>, anyhow::Error>;

    /// 删除订单
    async fn delete_order(&self, order_id: i32) -> Result<bool, anyhow::Error>;

    /// 获取订单项列表
    async fn get_order_items(&self, order_id: i32) -> Result<Vec<OrderItem>, anyhow::Error>;

    /// 创建订单项
    async fn create_order_item(&self, item: OrderItemCreate) -> Result<OrderItem, anyhow::Error>;

    /// 更新订单项
    async fn update_order_item(
        &self,
        item_id: i32,
        item: OrderItemUpdate,
    ) -> Result<Option<OrderItem>, anyhow::Error>;

    /// 删除订单项
    async fn delete_order_item(&self, item_id: i32) -> Result<bool, anyhow::Error>;
}
