//! 订单用例集成测试
//!
//! 独立的集成测试，不依赖内嵌测试模块

use std::sync::Arc;
use carptms::domain::use_cases::order::OrderUseCases;
use carptms::domain::use_cases::order::repository::OrderRepository;
use carptms::domain::entities::order::{
    Order, OrderCreate, OrderItem, OrderItemCreate, OrderItemUpdate, OrderQuery, OrderUpdate,
};
use async_trait::async_trait;
use chrono::NaiveDateTime;

#[allow(dead_code)]
struct MockOrderRepo {
    orders: Vec<Order>,
    order_items: Vec<OrderItem>,
}

#[async_trait]
impl OrderRepository for MockOrderRepo {
    async fn get_orders(&self, _query: OrderQuery) -> Result<(Vec<Order>, i64), anyhow::Error> {
        Ok((self.orders.clone(), self.orders.len() as i64))
    }

    async fn get_order(&self, order_id: i32) -> Result<Option<Order>, anyhow::Error> {
        Ok(self.orders.iter().find(|o| o.order_id == order_id).cloned())
    }

    async fn create_order(&self, order: OrderCreate) -> Result<Order, anyhow::Error> {
        let now =
            NaiveDateTime::parse_from_str("2026-01-13 10:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
        Ok(Order {
            order_id: self.orders.len() as i32 + 1,
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
            create_time: now,
            update_time: None,
        })
    }

    async fn update_order(
        &self,
        order_id: i32,
        order: OrderUpdate,
    ) -> Result<Option<Order>, anyhow::Error> {
        if let Some(mut existing) = self.get_order(order_id).await? {
            if let Some(status) = order.order_status {
                existing.order_status = status;
            }
            Ok(Some(existing))
        } else {
            Ok(None)
        }
    }

    async fn delete_order(&self, order_id: i32) -> Result<bool, anyhow::Error> {
        Ok(self.orders.iter().any(|o| o.order_id == order_id))
    }

    async fn get_order_items(&self, order_id: i32) -> Result<Vec<OrderItem>, anyhow::Error> {
        Ok(self
            .order_items
            .iter()
            .filter(|i| i.order_id == order_id)
            .cloned()
            .collect())
    }

    async fn create_order_item(&self, item: OrderItemCreate) -> Result<OrderItem, anyhow::Error> {
        let now =
            NaiveDateTime::parse_from_str("2026-01-13 10:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
        Ok(OrderItem {
            item_id: self.order_items.len() as i32 + 1,
            order_id: item.order_id,
            item_name: item.item_name,
            item_description: item.item_description,
            quantity: item.quantity,
            unit_price: item.unit_price,
            total_price: item.quantity as f64 * item.unit_price,
            create_time: now,
            update_time: None,
        })
    }

    async fn update_order_item(
        &self,
        item_id: i32,
        item: OrderItemUpdate,
    ) -> Result<Option<OrderItem>, anyhow::Error> {
        if let Some(mut existing) = self
            .order_items
            .iter()
            .find(|i| i.item_id == item_id)
            .cloned()
        {
            if let Some(name) = item.item_name {
                existing.item_name = name;
            }
            if let Some(desc) = item.item_description {
                existing.item_description = Some(desc);
            }
            if let Some(qty) = item.quantity {
                existing.quantity = qty;
                existing.total_price = qty as f64 * existing.unit_price;
            }
            if let Some(price) = item.unit_price {
                existing.unit_price = price;
                existing.total_price = existing.quantity as f64 * price;
            }
            Ok(Some(existing))
        } else {
            Ok(None)
        }
    }

    async fn delete_order_item(&self, item_id: i32) -> Result<bool, anyhow::Error> {
        Ok(self.order_items.iter().any(|i| i.item_id == item_id))
    }
}

#[tokio::test]
async fn test_create_order_success() {
    let order_create = OrderCreate {
        order_no: "ORD20260113001".to_string(),
        vehicle_id: 1,
        driver_id: Some(1),
        customer_name: "测试客户".to_string(),
        customer_phone: "13800138000".to_string(),
        origin: "北京".to_string(),
        destination: "上海".to_string(),
        cargo_type: "普通货物".to_string(),
        cargo_weight: 10.5,
        cargo_volume: 5.2,
        cargo_count: 100,
        order_amount: 10000.0,
        order_status: 1,
        departure_time: None,
        arrival_time: None,
        remark: None,
        create_user_id: 1,
    };

    let mock_repo = Arc::new(MockOrderRepo {
        orders: vec![],
        order_items: vec![],
    });
    let use_cases = OrderUseCases::new(mock_repo);
    let result = use_cases.create_order(order_create).await;

    assert!(result.is_ok());
    let order = result.unwrap();
    assert_eq!(order.order_no, "ORD20260113001");
    assert_eq!(order.customer_name, "测试客户");
}

#[tokio::test]
async fn test_create_order_empty_order_no() {
    let order_create = OrderCreate {
        order_no: "".to_string(),
        vehicle_id: 1,
        driver_id: Some(1),
        customer_name: "测试客户".to_string(),
        customer_phone: "13800138000".to_string(),
        origin: "北京".to_string(),
        destination: "上海".to_string(),
        cargo_type: "普通货物".to_string(),
        cargo_weight: 10.5,
        cargo_volume: 5.2,
        cargo_count: 100,
        order_amount: 10000.0,
        order_status: 1,
        departure_time: None,
        arrival_time: None,
        remark: None,
        create_user_id: 1,
    };

    let mock_repo = Arc::new(MockOrderRepo {
        orders: vec![],
        order_items: vec![],
    });
    let use_cases = OrderUseCases::new(mock_repo);
    let result: Result<Order, anyhow::Error> = use_cases.create_order(order_create).await;

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().to_string(), "订单编号不能为空");
}

#[tokio::test]
async fn test_create_order_zero_weight() {
    let order_create = OrderCreate {
        order_no: "ORD20260113001".to_string(),
        vehicle_id: 1,
        driver_id: Some(1),
        customer_name: "测试客户".to_string(),
        customer_phone: "13800138000".to_string(),
        origin: "北京".to_string(),
        destination: "上海".to_string(),
        cargo_type: "普通货物".to_string(),
        cargo_weight: 0.0,
        cargo_volume: 5.2,
        cargo_count: 100,
        order_amount: 10000.0,
        order_status: 1,
        departure_time: None,
        arrival_time: None,
        remark: None,
        create_user_id: 1,
    };

    let mock_repo = Arc::new(MockOrderRepo {
        orders: vec![],
        order_items: vec![],
    });
    let use_cases = OrderUseCases::new(mock_repo);
    let result: Result<Order, anyhow::Error> = use_cases.create_order(order_create).await;

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().to_string(), "货物重量必须大于0");
}

#[tokio::test]
async fn test_create_order_item_success() {
    let item_create = OrderItemCreate {
        order_id: 1,
        item_name: "商品1".to_string(),
        item_description: None,
        quantity: 10,
        unit_price: 100.0,
    };

    let mock_repo = Arc::new(MockOrderRepo {
        orders: vec![],
        order_items: vec![],
    });
    let use_cases = OrderUseCases::new(mock_repo);
    let result: Result<OrderItem, anyhow::Error> = use_cases.create_order_item(item_create).await;

    assert!(result.is_ok());
    let item = result.unwrap();
    assert_eq!(item.item_name, "商品1");
    assert_eq!(item.total_price, 1000.0);
}

#[tokio::test]
async fn test_create_order_item_empty_name() {
    let item_create = OrderItemCreate {
        order_id: 1,
        item_name: "".to_string(),
        item_description: None,
        quantity: 10,
        unit_price: 100.0,
    };

    let mock_repo = Arc::new(MockOrderRepo {
        orders: vec![],
        order_items: vec![],
    });
    let use_cases = OrderUseCases::new(mock_repo);
    let result: Result<OrderItem, anyhow::Error> = use_cases.create_order_item(item_create).await;

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().to_string(), "商品名称不能为空");
}
