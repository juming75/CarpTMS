//! /! 订单聚合根实现
//!
//! 基于 Event Sourcing 的订单聚合根

use crate::domain::ddd::{AggregateRoot, DomainEvent, Entity, EntityId, EventSourcedAggregate};
use crate::errors::AppResult;
use serde::{Deserialize, Serialize};

/// 订单ID
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct OrderId(pub i32);

impl EntityId for OrderId {
    fn type_name(&self) -> &'static str {
        "OrderId"
    }
}

impl std::fmt::Display for OrderId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// 订单状态
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderStatus {
    Pending,
    Confirmed,
    InTransit,
    Delivered,
    Cancelled,
}

/// 订单创建参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderCreateParams {
    pub order_no: String,
    pub vehicle_id: i32,
    pub customer_name: String,
    pub customer_phone: String,
    pub origin: String,
    pub destination: String,
    pub cargo_type: String,
    pub cargo_weight: f64,
}

/// 订单聚合根
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderAggregate {
    pub id: OrderId,
    pub order_no: String,
    pub vehicle_id: i32,
    pub driver_id: Option<i32>,
    pub customer_name: String,
    pub customer_phone: String,
    pub origin: String,
    pub destination: String,
    pub cargo_type: String,
    pub cargo_weight: f64,
    pub status: OrderStatus,
    pub order_amount: f64,
    pub version: u64,
    events: Vec<DomainEvent>,
}

impl OrderAggregate {
    /// 创建新订单
    pub fn new(params: OrderCreateParams) -> Self {
        let mut order = Self {
            id: OrderId(0),
            order_no: params.order_no,
            vehicle_id: params.vehicle_id,
            driver_id: None,
            customer_name: params.customer_name,
            customer_phone: params.customer_phone,
            origin: params.origin,
            destination: params.destination,
            cargo_type: params.cargo_type,
            cargo_weight: params.cargo_weight,
            status: OrderStatus::Pending,
            order_amount: 0.0,
            version: 0,
            events: Vec::new(),
        };
        order.raise_event("OrderCreated", serde_json::to_value(&order).expect("order aggregate should serialize"));
        order
    }

    /// 确认订单
    pub fn confirm(&mut self) -> AppResult<()> {
        if self.status != OrderStatus::Pending {
            return Err(crate::errors::AppError::validation(
                "Only pending orders can be confirmed",
            ));
        }
        self.status = OrderStatus::Confirmed;
        self.raise_event(
            "OrderConfirmed",
            serde_json::json!({ "order_id": self.id.0 }),
        );
        Ok(())
    }

    /// 开始运输
    pub fn start_transit(&mut self, driver_id: i32) -> AppResult<()> {
        if self.status != OrderStatus::Confirmed {
            return Err(crate::errors::AppError::validation(
                "Only confirmed orders can start transit",
            ));
        }
        self.status = OrderStatus::InTransit;
        self.driver_id = Some(driver_id);
        self.raise_event(
            "OrderTransitStarted",
            serde_json::json!({
                "order_id": self.id.0,
                "driver_id": driver_id
            }),
        );
        Ok(())
    }

    /// 送达
    pub fn deliver(&mut self) -> AppResult<()> {
        if self.status != OrderStatus::InTransit {
            return Err(crate::errors::AppError::validation(
                "Only in-transit orders can be delivered",
            ));
        }
        self.status = OrderStatus::Delivered;
        self.raise_event(
            "OrderDelivered",
            serde_json::json!({ "order_id": self.id.0 }),
        );
        Ok(())
    }

    /// 取消订单
    pub fn cancel(&mut self, reason: String) -> AppResult<()> {
        if self.status == OrderStatus::Delivered {
            return Err(crate::errors::AppError::validation(
                "Delivered orders cannot be cancelled",
            ));
        }
        self.status = OrderStatus::Cancelled;
        self.raise_event(
            "OrderCancelled",
            serde_json::json!({
                "order_id": self.id.0,
                "reason": reason
            }),
        );
        Ok(())
    }

    fn raise_event(&mut self, event_type: &str, data: serde_json::Value) {
        let event = DomainEvent::new(
            "Order",
            &self.id.to_string(),
            event_type,
            data,
            self.version as i32 + 1,
        );
        self.events.push(event);
        self.version += 1;
    }
}

impl Entity for OrderAggregate {
    fn id(&self) -> &impl EntityId {
        &self.id
    }
}

impl AggregateRoot for OrderAggregate {
    fn version(&self) -> u64 {
        self.version
    }
    fn events(&self) -> &[DomainEvent] {
        &self.events
    }
    fn clear_events(&mut self) {
        self.events.clear();
    }
}

impl EventSourcedAggregate for OrderAggregate {
    fn rebuild_from_events(&mut self, events: &[DomainEvent]) -> AppResult<()> {
        for event in events {
            self.apply_event(event);
        }
        Ok(())
    }
    fn get_uncommitted_events(&self) -> &[DomainEvent] {
        &self.events
    }
    fn mark_events_committed(&mut self) {
        self.events.clear();
    }
}

impl OrderAggregate {
    pub fn apply_event_in_place(&mut self, event: &DomainEvent) {
        self.apply_event(event);
    }

    fn apply_event(&mut self, event: &DomainEvent) {
        match event.event_type.as_str() {
            "OrderCreated" => {
                if let Ok(data) = serde_json::from_value::<Self>(event.event_data.clone()) {
                    *self = data;
                    self.version = event.version as u64;
                }
            }
            "OrderConfirmed" => {
                self.status = OrderStatus::Confirmed;
                self.version = event.version as u64;
            }
            "OrderTransitStarted" => {
                self.status = OrderStatus::InTransit;
                if let Some(obj) = event.event_data.as_object() {
                    if let Some(driver_id) = obj.get("driver_id").and_then(|v| v.as_i64()) {
                        self.driver_id = Some(driver_id as i32);
                    }
                }
                self.version = event.version as u64;
            }
            "OrderDelivered" => {
                self.status = OrderStatus::Delivered;
                self.version = event.version as u64;
            }
            "OrderCancelled" => {
                self.status = OrderStatus::Cancelled;
                self.version = event.version as u64;
            }
            _ => {}
        }
    }
}
