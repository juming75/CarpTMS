//! / 订单领域用例

use std::sync::Arc;

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

/// 订单用例结构
#[derive(Clone)]
pub struct OrderUseCases {
    order_repository: Arc<dyn OrderRepository + Send + Sync>,
}

impl OrderUseCases {
    /// 创建订单用例实例
    pub fn new(order_repository: Arc<dyn OrderRepository>) -> Self {
        Self { order_repository }
    }

    /// 获取订单列表用例
    pub async fn get_orders(&self, query: OrderQuery) -> Result<(Vec<Order>, i64), anyhow::Error> {
        // 业务逻辑:可以在这里添加额外的业务规则,例如权限检查等
        self.order_repository.get_orders(query).await
    }

    /// 获取单个订单用例
    pub async fn get_order(&self, order_id: i32) -> Result<Option<Order>, anyhow::Error> {
        // 业务逻辑:可以在这里添加额外的业务规则,例如权限检查等
        self.order_repository.get_order(order_id).await
    }

    /// 创建订单用例
    pub async fn create_order(&self, order: OrderCreate) -> Result<Order, anyhow::Error> {
        // 业务逻辑:数据验证
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

        // 调用仓库创建订单
        self.order_repository.create_order(order).await
    }

    /// 更新订单用例
    pub async fn update_order(
        &self,
        order_id: i32,
        order: OrderUpdate,
    ) -> Result<Option<Order>, anyhow::Error> {
        // 业务逻辑:可以在这里添加额外的业务规则,例如状态转换验证等
        // 调用仓库更新订单
        self.order_repository.update_order(order_id, order).await
    }

    /// 删除订单用例
    pub async fn delete_order(&self, order_id: i32) -> Result<bool, anyhow::Error> {
        // 业务逻辑:可以在这里添加额外的业务规则,例如权限检查、关联数据检查等
        // 例如:检查该订单是否有关联的订单项,如果关联则不允许删除

        // 调用仓库删除订单
        self.order_repository.delete_order(order_id).await
    }

    /// 获取订单项列表用例
    pub async fn get_order_items(&self, order_id: i32) -> Result<Vec<OrderItem>, anyhow::Error> {
        // 业务逻辑:可以在这里添加额外的业务规则,例如权限检查等
        self.order_repository.get_order_items(order_id).await
    }

    /// 创建订单项用例
    pub async fn create_order_item(
        &self,
        item: OrderItemCreate,
    ) -> Result<OrderItem, anyhow::Error> {
        // 业务逻辑:数据验证
        if item.item_name.is_empty() {
            return Err(anyhow::anyhow!("商品名称不能为空"));
        }

        if item.quantity <= 0 {
            return Err(anyhow::anyhow!("数量必须大于0"));
        }

        if item.unit_price <= 0.0 {
            return Err(anyhow::anyhow!("单价必须大于0"));
        }

        // 调用仓库创建订单项
        self.order_repository.create_order_item(item).await
    }

    /// 更新订单项用例
    pub async fn update_order_item(
        &self,
        item_id: i32,
        item: OrderItemUpdate,
    ) -> Result<Option<OrderItem>, anyhow::Error> {
        // 业务逻辑:可以在这里添加额外的业务规则,例如数据验证等
        // 调用仓库更新订单项
        self.order_repository.update_order_item(item_id, item).await
    }

    /// 删除订单项用例
    pub async fn delete_order_item(&self, item_id: i32) -> Result<bool, anyhow::Error> {
        // 业务逻辑:可以在这里添加额外的业务规则,例如权限检查等
        // 调用仓库删除订单项
        self.order_repository.delete_order_item(item_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use chrono::NaiveDateTime;
    use std::sync::Arc;

    // 模拟订单仓库实现
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
            let new_order = Order {
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
            };
            Ok(new_order)
        }

        async fn update_order(
            &self,
            order_id: i32,
            order: OrderUpdate,
        ) -> Result<Option<Order>, anyhow::Error> {
            if let Some(mut existing_order) = self.get_order(order_id).await? {
                if let Some(order_status) = order.order_status {
                    existing_order.order_status = order_status;
                }
                Ok(Some(existing_order))
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

        async fn create_order_item(
            &self,
            item: OrderItemCreate,
        ) -> Result<OrderItem, anyhow::Error> {
            let now =
                NaiveDateTime::parse_from_str("2026-01-13 10:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
            let new_item = OrderItem {
                item_id: self.order_items.len() as i32 + 1,
                order_id: item.order_id,
                item_name: item.item_name,
                item_description: item.item_description,
                quantity: item.quantity,
                unit_price: item.unit_price,
                total_price: item.quantity as f64 * item.unit_price,
                create_time: now,
                update_time: None,
            };
            Ok(new_item)
        }

        async fn update_order_item(
            &self,
            item_id: i32,
            item: OrderItemUpdate,
        ) -> Result<Option<OrderItem>, anyhow::Error> {
            if let Some(mut existing_item) = self
                .order_items
                .iter()
                .find(|i| i.item_id == item_id)
                .cloned()
            {
                if let Some(item_name) = item.item_name {
                    existing_item.item_name = item_name;
                }
                if let Some(item_description) = item.item_description {
                    existing_item.item_description = Some(item_description);
                }
                if let Some(quantity) = item.quantity {
                    existing_item.quantity = quantity;
                    existing_item.total_price = quantity as f64 * existing_item.unit_price;
                }
                if let Some(unit_price) = item.unit_price {
                    existing_item.unit_price = unit_price;
                    existing_item.total_price = existing_item.quantity as f64 * unit_price;
                }
                Ok(Some(existing_item))
            } else {
                Ok(None)
            }
        }

        async fn delete_order_item(&self, item_id: i32) -> Result<bool, anyhow::Error> {
            Ok(self.order_items.iter().any(|i| i.item_id == item_id))
        }
    }

    // 测试用例:创建订单成功
    #[tokio::test]
    async fn test_create_order_success() {
        // 准备测试数据
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

        // 创建模拟仓库
        let mock_repo = Arc::new(MockOrderRepo {
            orders: Vec::new(),
            order_items: Vec::new(),
        });

        // 创建用例实例
        let use_cases = OrderUseCases::new(mock_repo);

        // 执行测试
        let result = use_cases.create_order(order_create).await;

        // 验证结果
        assert!(result.is_ok());
        let order = result.unwrap();
        assert_eq!(order.order_no, "ORD20260113001");
        assert_eq!(order.customer_name, "测试客户");
    }

    // 测试用例:创建订单失败 - 订单编号为空
    #[tokio::test]
    async fn test_create_order_empty_order_no() {
        // 准备测试数据
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

        // 创建模拟仓库
        let mock_repo = Arc::new(MockOrderRepo {
            orders: Vec::new(),
            order_items: Vec::new(),
        });

        // 创建用例实例
        let use_cases = OrderUseCases::new(mock_repo);

        // 执行测试
        let result = use_cases.create_order(order_create).await;

        // 验证结果
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "订单编号不能为空");
    }

    // 测试用例:创建订单失败 - 客户名称为空
    #[tokio::test]
    async fn test_create_order_empty_customer_name() {
        // 准备测试数据
        let order_create = OrderCreate {
            order_no: "ORD20260113001".to_string(),
            vehicle_id: 1,
            driver_id: Some(1),
            customer_name: "".to_string(),
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

        // 创建模拟仓库
        let mock_repo = Arc::new(MockOrderRepo {
            orders: Vec::new(),
            order_items: Vec::new(),
        });

        // 创建用例实例
        let use_cases = OrderUseCases::new(mock_repo);

        // 执行测试
        let result = use_cases.create_order(order_create).await;

        // 验证结果
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "客户名称不能为空");
    }

    // 测试用例:创建订单失败 - 货物重量为0
    #[tokio::test]
    async fn test_create_order_zero_weight() {
        // 准备测试数据
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

        // 创建模拟仓库
        let mock_repo = Arc::new(MockOrderRepo {
            orders: Vec::new(),
            order_items: Vec::new(),
        });

        // 创建用例实例
        let use_cases = OrderUseCases::new(mock_repo);

        // 执行测试
        let result = use_cases.create_order(order_create).await;

        // 验证结果
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "货物重量必须大于0");
    }

    // 测试用例:获取订单列表
    #[tokio::test]
    async fn test_get_orders() {
        // 准备测试数据
        let now =
            NaiveDateTime::parse_from_str("2026-01-13 10:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
        let orders = vec![
            Order {
                order_id: 1,
                order_no: "ORD20260113001".to_string(),
                vehicle_id: 1,
                driver_id: Some(1),
                customer_name: "测试客户1".to_string(),
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
                create_time: now,
                update_time: None,
            },
            Order {
                order_id: 2,
                order_no: "ORD20260113002".to_string(),
                vehicle_id: 1,
                driver_id: Some(1),
                customer_name: "测试客户2".to_string(),
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
                create_time: now,
                update_time: None,
            },
        ];

        // 创建模拟仓库
        let mock_repo = Arc::new(MockOrderRepo {
            orders: orders.clone(),
            order_items: Vec::new(),
        });

        // 创建用例实例
        let use_cases = OrderUseCases::new(mock_repo);

        // 执行测试
        let query = OrderQuery::default();
        let result = use_cases.get_orders(query).await;

        // 验证结果
        assert!(result.is_ok());
        let (result_orders, result_total) = result.unwrap();
        assert_eq!(result_orders, orders);
        assert_eq!(result_total, 2);
    }

    // 测试用例:获取单个订单
    #[tokio::test]
    async fn test_get_order() {
        // 准备测试数据
        let now =
            NaiveDateTime::parse_from_str("2026-01-13 10:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
        let order = Order {
            order_id: 1,
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
            create_time: now,
            update_time: None,
        };

        // 创建模拟仓库
        let mock_repo = Arc::new(MockOrderRepo {
            orders: vec![order.clone()],
            order_items: Vec::new(),
        });

        // 创建用例实例
        let use_cases = OrderUseCases::new(mock_repo);

        // 执行测试
        let result = use_cases.get_order(1).await;

        // 验证结果
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some(order));
    }

    // 测试用例:获取单个订单 - 订单不存在
    #[tokio::test]
    async fn test_get_order_not_found() {
        // 创建模拟仓库
        let mock_repo = Arc::new(MockOrderRepo {
            orders: Vec::new(),
            order_items: Vec::new(),
        });

        // 创建用例实例
        let use_cases = OrderUseCases::new(mock_repo);

        // 执行测试
        let result = use_cases.get_order(999).await;

        // 验证结果
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), None);
    }

    // 测试用例:更新订单
    #[tokio::test]
    async fn test_update_order() {
        // 准备测试数据
        let now =
            NaiveDateTime::parse_from_str("2026-01-13 10:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
        let order = Order {
            order_id: 1,
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
            create_time: now,
            update_time: None,
        };

        let order_update = OrderUpdate {
            order_status: Some(2),
            ..Default::default()
        };

        // 创建模拟仓库
        let mock_repo = Arc::new(MockOrderRepo {
            orders: vec![order.clone()],
            order_items: Vec::new(),
        });

        // 创建用例实例
        let use_cases = OrderUseCases::new(mock_repo);

        // 执行测试
        let result = use_cases.update_order(1, order_update).await;

        // 验证结果
        assert!(result.is_ok());
        let updated_order = result.unwrap().unwrap();
        assert_eq!(updated_order.order_status, 2);
    }

    // 测试用例:删除订单
    #[tokio::test]
    async fn test_delete_order() {
        // 准备测试数据
        let now =
            NaiveDateTime::parse_from_str("2026-01-13 10:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
        let order = Order {
            order_id: 1,
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
            create_time: now,
            update_time: None,
        };

        // 创建模拟仓库
        let mock_repo = Arc::new(MockOrderRepo {
            orders: vec![order.clone()],
            order_items: Vec::new(),
        });

        // 创建用例实例
        let use_cases = OrderUseCases::new(mock_repo);

        // 执行测试
        let result = use_cases.delete_order(1).await;

        // 验证结果
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    // 测试用例:创建订单项成功
    #[tokio::test]
    async fn test_create_order_item_success() {
        // 准备测试数据
        let item_create = OrderItemCreate {
            order_id: 1,
            item_name: "商品1".to_string(),
            item_description: None,
            quantity: 10,
            unit_price: 100.0,
        };

        // 创建模拟仓库
        let mock_repo = Arc::new(MockOrderRepo {
            orders: Vec::new(),
            order_items: Vec::new(),
        });

        // 创建用例实例
        let use_cases = OrderUseCases::new(mock_repo);

        // 执行测试
        let result = use_cases.create_order_item(item_create).await;

        // 验证结果
        assert!(result.is_ok());
        let item = result.unwrap();
        assert_eq!(item.item_name, "商品1");
        assert_eq!(item.quantity, 10);
        assert_eq!(item.unit_price, 100.0);
        assert_eq!(item.total_price, 1000.0);
    }

    // 测试用例:创建订单项失败 - 商品名称为空
    #[tokio::test]
    async fn test_create_order_item_empty_name() {
        // 准备测试数据
        let item_create = OrderItemCreate {
            order_id: 1,
            item_name: "".to_string(),
            item_description: None,
            quantity: 10,
            unit_price: 100.0,
        };

        // 创建模拟仓库
        let mock_repo = Arc::new(MockOrderRepo {
            orders: Vec::new(),
            order_items: Vec::new(),
        });

        // 创建用例实例
        let use_cases = OrderUseCases::new(mock_repo);

        // 执行测试
        let result = use_cases.create_order_item(item_create).await;

        // 验证结果
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "商品名称不能为空");
    }

    // 测试用例:创建订单项失败 - 数量为0
    #[tokio::test]
    async fn test_create_order_item_zero_quantity() {
        // 准备测试数据
        let item_create = OrderItemCreate {
            order_id: 1,
            item_name: "商品1".to_string(),
            item_description: None,
            quantity: 0,
            unit_price: 100.0,
        };

        // 创建模拟仓库
        let mock_repo = Arc::new(MockOrderRepo {
            orders: Vec::new(),
            order_items: Vec::new(),
        });

        // 创建用例实例
        let use_cases = OrderUseCases::new(mock_repo);

        // 执行测试
        let result = use_cases.create_order_item(item_create).await;

        // 验证结果
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "数量必须大于0");
    }

    // 测试用例:获取订单项列表
    #[tokio::test]
    async fn test_get_order_items() {
        // 准备测试数据
        let now =
            NaiveDateTime::parse_from_str("2026-01-13 10:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
        let items = vec![
            OrderItem {
                item_id: 1,
                order_id: 1,
                item_name: "商品1".to_string(),
                item_description: None,
                quantity: 10,
                unit_price: 100.0,
                total_price: 1000.0,
                create_time: now,
                update_time: None,
            },
            OrderItem {
                item_id: 2,
                order_id: 1,
                item_name: "商品2".to_string(),
                item_description: None,
                quantity: 5,
                unit_price: 200.0,
                total_price: 1000.0,
                create_time: now,
                update_time: None,
            },
        ];

        // 创建模拟仓库
        let mock_repo = Arc::new(MockOrderRepo {
            orders: Vec::new(),
            order_items: items.clone(),
        });

        // 创建用例实例
        let use_cases = OrderUseCases::new(mock_repo);

        // 执行测试
        let result = use_cases.get_order_items(1).await;

        // 验证结果
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), items);
    }

    // 测试用例:更新订单项
    #[tokio::test]
    async fn test_update_order_item() {
        // 准备测试数据
        let now =
            NaiveDateTime::parse_from_str("2026-01-13 10:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
        let item = OrderItem {
            item_id: 1,
            order_id: 1,
            item_name: "商品1".to_string(),
            item_description: None,
            quantity: 10,
            unit_price: 100.0,
            total_price: 1000.0,
            create_time: now,
            update_time: None,
        };

        let item_update = OrderItemUpdate {
            item_name: None,
            item_description: None,
            quantity: Some(20),
            unit_price: Some(150.0),
        };

        // 创建模拟仓库
        let mock_repo = Arc::new(MockOrderRepo {
            orders: Vec::new(),
            order_items: vec![item.clone()],
        });

        // 创建用例实例
        let use_cases = OrderUseCases::new(mock_repo);

        // 执行测试
        let result = use_cases.update_order_item(1, item_update).await;

        // 验证结果
        assert!(result.is_ok());
        let updated_item = result.unwrap().unwrap();
        assert_eq!(updated_item.quantity, 20);
        assert_eq!(updated_item.unit_price, 150.0);
        assert_eq!(updated_item.total_price, 3000.0);
    }

    // 测试用例:删除订单项
    #[tokio::test]
    async fn test_delete_order_item() {
        // 准备测试数据
        let now =
            NaiveDateTime::parse_from_str("2026-01-13 10:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
        let item = OrderItem {
            item_id: 1,
            order_id: 1,
            item_name: "商品1".to_string(),
            item_description: None,
            quantity: 10,
            unit_price: 100.0,
            total_price: 1000.0,
            create_time: now,
            update_time: None,
        };

        // 创建模拟仓库
        let mock_repo = Arc::new(MockOrderRepo {
            orders: Vec::new(),
            order_items: vec![item.clone()],
        });

        // 创建用例实例
        let use_cases = OrderUseCases::new(mock_repo);

        // 执行测试
        let result = use_cases.delete_order_item(1).await;

        // 验证结果
        assert!(result.is_ok());
        assert!(result.unwrap());
    }
}
