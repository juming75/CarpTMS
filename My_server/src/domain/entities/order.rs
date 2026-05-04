//! / 订单领域实体

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

/// 订单实体
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Order {
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

/// 订单项实体
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OrderItem {
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

/// 订单创建实体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderCreate {
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

/// 订单项创建实体
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OrderItemCreate {
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
}

/// 订单更新实体
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OrderUpdate {
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

/// 订单项更新实体
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OrderItemUpdate {
    /// 商品名称
    pub item_name: Option<String>,
    /// 商品描述
    pub item_description: Option<String>,
    /// 数量
    pub quantity: Option<i32>,
    /// 单价
    pub unit_price: Option<f64>,
}

/// 订单查询条件实体
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OrderQuery {
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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn test_order_create_to_order() {
        // 创建OrderCreate实例
        let order_create = OrderCreate {
            order_no: "ORDER001".to_string(),
            vehicle_id: 1,
            driver_id: Some(1),
            customer_name: "Test Customer".to_string(),
            customer_phone: "13800138000".to_string(),
            origin: "北京".to_string(),
            destination: "上海".to_string(),
            cargo_type: "普通货物".to_string(),
            cargo_weight: 10.5,
            cargo_volume: 5.0,
            cargo_count: 100,
            order_amount: 10000.0,
            order_status: 1,
            departure_time: Some(
                NaiveDate::from_ymd_opt(2023, 1, 1)
                    .unwrap()
                    .and_hms_opt(10, 0, 0)
                    .unwrap(),
            ),
            arrival_time: None,
            remark: Some("Test remark".to_string()),
            create_user_id: 1,
        };

        // 手动转换为Order实例
        let now = NaiveDate::from_ymd_opt(2023, 1, 1)
            .unwrap()
            .and_hms_opt(10, 0, 0)
            .unwrap();
        let order = Order {
            order_id: 1,
            order_no: order_create.order_no,
            vehicle_id: order_create.vehicle_id,
            driver_id: order_create.driver_id,
            customer_name: order_create.customer_name,
            customer_phone: order_create.customer_phone,
            origin: order_create.origin,
            destination: order_create.destination,
            cargo_type: order_create.cargo_type,
            cargo_weight: order_create.cargo_weight,
            cargo_volume: order_create.cargo_volume,
            cargo_count: order_create.cargo_count,
            order_amount: order_create.order_amount,
            order_status: order_create.order_status,
            departure_time: order_create.departure_time,
            arrival_time: order_create.arrival_time,
            remark: order_create.remark,
            create_user_id: order_create.create_user_id,
            create_time: now,
            update_time: None,
        };

        // 验证转换结果
        assert_eq!(order.order_no, "ORDER001");
        assert_eq!(order.vehicle_id, 1);
        assert_eq!(order.driver_id, Some(1));
        assert_eq!(order.customer_name, "Test Customer");
        assert_eq!(order.cargo_weight, 10.5);
    }

    #[test]
    fn test_order_update() {
        // 创建初始Order实例
        let now = NaiveDate::from_ymd_opt(2023, 1, 1)
            .unwrap()
            .and_hms_opt(10, 0, 0)
            .unwrap();
        let mut order = Order {
            order_id: 1,
            order_no: "ORDER001".to_string(),
            vehicle_id: 1,
            driver_id: Some(1),
            customer_name: "Test Customer".to_string(),
            customer_phone: "13800138000".to_string(),
            origin: "北京".to_string(),
            destination: "上海".to_string(),
            cargo_type: "普通货物".to_string(),
            cargo_weight: 10.5,
            cargo_volume: 5.0,
            cargo_count: 100,
            order_amount: 10000.0,
            order_status: 1,
            departure_time: Some(now),
            arrival_time: None,
            remark: Some("Test remark".to_string()),
            create_user_id: 1,
            create_time: now,
            update_time: None,
        };

        // 创建OrderUpdate实例
        let order_update = OrderUpdate {
            order_no: Some("ORDER002".to_string()),
            vehicle_id: Some(2),
            driver_id: None,
            customer_name: Some("Updated Customer".to_string()),
            customer_phone: None,
            origin: None,
            destination: Some("广州".to_string()),
            cargo_type: None,
            cargo_weight: Some(15.5),
            cargo_volume: None,
            cargo_count: Some(150),
            order_amount: Some(15000.0),
            order_status: Some(2),
            departure_time: None,
            arrival_time: Some(
                NaiveDate::from_ymd_opt(2023, 1, 2)
                    .unwrap()
                    .and_hms_opt(10, 0, 0)
                    .unwrap(),
            ),
            remark: Some("Updated remark".to_string()),
        };

        // 手动应用更新
        if let Some(order_no) = order_update.order_no {
            order.order_no = order_no;
        }
        if let Some(vehicle_id) = order_update.vehicle_id {
            order.vehicle_id = vehicle_id;
        }
        if let Some(driver_id) = order_update.driver_id {
            order.driver_id = Some(driver_id);
        }
        if let Some(customer_name) = order_update.customer_name {
            order.customer_name = customer_name;
        }
        if let Some(destination) = order_update.destination {
            order.destination = destination;
        }
        if let Some(cargo_weight) = order_update.cargo_weight {
            order.cargo_weight = cargo_weight;
        }
        if let Some(cargo_count) = order_update.cargo_count {
            order.cargo_count = cargo_count;
        }
        if let Some(order_amount) = order_update.order_amount {
            order.order_amount = order_amount;
        }
        if let Some(order_status) = order_update.order_status {
            order.order_status = order_status;
        }
        if let Some(arrival_time) = order_update.arrival_time {
            order.arrival_time = Some(arrival_time);
        }
        if let Some(remark) = order_update.remark {
            order.remark = Some(remark);
        }

        // 验证更新结果
        assert_eq!(order.order_no, "ORDER002");
        assert_eq!(order.vehicle_id, 2);
        assert_eq!(order.driver_id, Some(1)); // 没有更新,保持原值
        assert_eq!(order.customer_name, "Updated Customer");
        assert_eq!(order.destination, "广州");
        assert_eq!(order.cargo_weight, 15.5);
        assert_eq!(order.order_status, 2);
        assert_eq!(order.remark, Some("Updated remark".to_string()));
    }

    #[test]
    fn test_order_item_create_to_order_item() {
        // 创建OrderItemCreate实例
        let item_create = OrderItemCreate {
            order_id: 1,
            item_name: "Test Item".to_string(),
            item_description: Some("Test description".to_string()),
            quantity: 10,
            unit_price: 100.0,
        };

        // 手动转换为OrderItem实例
        let now = NaiveDate::from_ymd_opt(2023, 1, 1)
            .unwrap()
            .and_hms_opt(10, 0, 0)
            .unwrap();
        let item = OrderItem {
            item_id: 1,
            order_id: item_create.order_id,
            item_name: item_create.item_name,
            item_description: item_create.item_description,
            quantity: item_create.quantity,
            unit_price: item_create.unit_price,
            total_price: item_create.quantity as f64 * item_create.unit_price,
            create_time: now,
            update_time: None,
        };

        // 验证转换结果
        assert_eq!(item.item_name, "Test Item");
        assert_eq!(item.quantity, 10);
        assert_eq!(item.unit_price, 100.0);
        assert_eq!(item.total_price, 1000.0);
    }

    #[test]
    fn test_order_item_update() {
        // 创建初始OrderItem实例
        let now = NaiveDate::from_ymd_opt(2023, 1, 1)
            .unwrap()
            .and_hms_opt(10, 0, 0)
            .unwrap();
        let mut item = OrderItem {
            item_id: 1,
            order_id: 1,
            item_name: "Test Item".to_string(),
            item_description: Some("Test description".to_string()),
            quantity: 10,
            unit_price: 100.0,
            total_price: 1000.0,
            create_time: now,
            update_time: None,
        };

        // 创建OrderItemUpdate实例
        let item_update = OrderItemUpdate {
            item_name: Some("Updated Item".to_string()),
            item_description: None,
            quantity: Some(20),
            unit_price: Some(150.0),
        };

        // 手动应用更新
        if let Some(item_name) = item_update.item_name {
            item.item_name = item_name;
        }
        if let Some(quantity) = item_update.quantity {
            item.quantity = quantity;
        }
        if let Some(unit_price) = item_update.unit_price {
            item.unit_price = unit_price;
        }
        // 更新总价
        item.total_price = item.quantity as f64 * item.unit_price;

        // 验证更新结果
        assert_eq!(item.item_name, "Updated Item");
        assert_eq!(item.quantity, 20);
        assert_eq!(item.unit_price, 150.0);
        assert_eq!(item.total_price, 3000.0);
    }
}
