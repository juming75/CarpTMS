//! / 订单用例测试

use std::sync::Arc;
use mockall::mock;
use anyhow::Result;

use crate::domain::entities::order::{Order, OrderCreate, OrderItem, OrderItemCreate, OrderItemUpdate, OrderQuery, OrderUpdate};
use crate::domain::use_cases::order::{OrderRepository, OrderUseCases};

// 模拟订单仓库
mock! {
    pub OrderRepo {
    }
    
    #[async_trait::async_trait]
    impl OrderRepository for OrderRepo {
        async fn get_orders(&self, query: OrderQuery) -> Result<(Vec<Order>, i64), anyhow::Error>;
        async fn get_order(&self, order_id: i32) -> Result<Option<Order>, anyhow::Error>;
        async fn create_order(&self, order: OrderCreate) -> Result<Order, anyhow::Error>;
        async fn update_order(&self, order_id: i32, order: OrderUpdate) -> Result<Option<Order>, anyhow::Error>;
        async fn delete_order(&self, order_id: i32) -> Result<bool, anyhow::Error>;
        async fn get_order_items(&self, order_id: i32) -> Result<Vec<OrderItem>, anyhow::Error>;
        async fn create_order_item(&self, item: OrderItemCreate) -> Result<OrderItem, anyhow::Error>;
        async fn update_order_item(&self, item_id: i32, item: OrderItemUpdate) -> Result<Option<OrderItem>, anyhow::Error>;
        async fn delete_order_item(&self, item_id: i32) -> Result<bool, anyhow::Error>;
    }
}

// 测试用例:创建订单成功
#[tokio::test]
async fn test_create_order_success() {
    // 准备测试数据
    let order_create = OrderCreate {
        order_no: "ORD20260113001".to_string(),
        customer_name: "测试客户".to_string(),
        customer_phone: "13800138000".to_string(),
        origin: "北京".to_string(),
        destination: "上海".to_string(),
        cargo_weight: 10.5,
        cargo_volume: 5.2,
        cargo_count: 100,
        order_amount: 10000.0,
        status: "pending".to_string(),
        ..Default::default()
    };
    
    let expected_order = Order {
        id: 1,
        order_no: "ORD20260113001".to_string(),
        customer_name: "测试客户".to_string(),
        customer_phone: "13800138000".to_string(),
        origin: "北京".to_string(),
        destination: "上海".to_string(),
        cargo_weight: 10.5,
        cargo_volume: 5.2,
        cargo_count: 100,
        order_amount: 10000.0,
        status: "pending".to_string(),
        ..Default::default()
    };
    
    // 设置模拟仓库
    let mut mock_repo = MockOrderRepo::new();
    mock_repo
        .expect_create_order()
        .return_once(|_| Ok(expected_order.clone()));
    
    // 创建用例实例
    let use_cases = OrderUseCases::new(Arc::new(mock_repo));
    
    // 执行测试
    let result = use_cases.create_order(order_create).await;
    
    // 验证结果
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), expected_order);
}

// 测试用例:创建订单失败 - 订单编号为空
#[tokio::test]
async fn test_create_order_empty_order_no() {
    // 准备测试数据
    let order_create = OrderCreate {
        order_no: "".to_string(),
        customer_name: "测试客户".to_string(),
        customer_phone: "13800138000".to_string(),
        origin: "北京".to_string(),
        destination: "上海".to_string(),
        cargo_weight: 10.5,
        cargo_volume: 5.2,
        cargo_count: 100,
        order_amount: 10000.0,
        status: "pending".to_string(),
        ..Default::default()
    };
    
    // 创建用例实例
    let mock_repo = MockOrderRepo::new();
    let use_cases = OrderUseCases::new(Arc::new(mock_repo));
    
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
        customer_name: "".to_string(),
        customer_phone: "13800138000".to_string(),
        origin: "北京".to_string(),
        destination: "上海".to_string(),
        cargo_weight: 10.5,
        cargo_volume: 5.2,
        cargo_count: 100,
        order_amount: 10000.0,
        status: "pending".to_string(),
        ..Default::default()
    };
    
    // 创建用例实例
    let mock_repo = MockOrderRepo::new();
    let use_cases = OrderUseCases::new(Arc::new(mock_repo));
    
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
        customer_name: "测试客户".to_string(),
        customer_phone: "13800138000".to_string(),
        origin: "北京".to_string(),
        destination: "上海".to_string(),
        cargo_weight: 0.0,
        cargo_volume: 5.2,
        cargo_count: 100,
        order_amount: 10000.0,
        status: "pending".to_string(),
        ..Default::default()
    };
    
    // 创建用例实例
    let mock_repo = MockOrderRepo::new();
    let use_cases = OrderUseCases::new(Arc::new(mock_repo));
    
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
    let orders = vec![
        Order {
            id: 1,
            order_no: "ORD20260113001".to_string(),
            customer_name: "测试客户1".to_string(),
            ..Default::default()
        },
        Order {
            id: 2,
            order_no: "ORD20260113002".to_string(),
            customer_name: "测试客户2".to_string(),
            ..Default::default()
        }
    ];
    let total = 2;
    
    // 设置模拟仓库
    let mut mock_repo = MockOrderRepo::new();
    mock_repo
        .expect_get_orders()
        .return_once(|_| Ok((orders.clone(), total)));
    
    // 创建用例实例
    let use_cases = OrderUseCases::new(Arc::new(mock_repo));
    
    // 执行测试
    let query = OrderQuery::default();
    let result = use_cases.get_orders(query).await;
    
    // 验证结果
    assert!(result.is_ok());
    let (result_orders, result_total) = result.unwrap();
    assert_eq!(result_orders, orders);
    assert_eq!(result_total, total);
}

// 测试用例:获取单个订单
#[tokio::test]
async fn test_get_order() {
    // 准备测试数据
    let order = Order {
        id: 1,
        order_no: "ORD20260113001".to_string(),
        customer_name: "测试客户".to_string(),
        ..Default::default()
    };
    
    // 设置模拟仓库
    let mut mock_repo = MockOrderRepo::new();
    mock_repo
        .expect_get_order()
        .return_once(|_| Ok(Some(order.clone())));
    
    // 创建用例实例
    let use_cases = OrderUseCases::new(Arc::new(mock_repo));
    
    // 执行测试
    let result = use_cases.get_order(1).await;
    
    // 验证结果
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Some(order));
}

// 测试用例:获取单个订单 - 订单不存在
#[tokio::test]
async fn test_get_order_not_found() {
    // 设置模拟仓库
    let mut mock_repo = MockOrderRepo::new();
    mock_repo
        .expect_get_order()
        .return_once(|_| Ok(None));
    
    // 创建用例实例
    let use_cases = OrderUseCases::new(Arc::new(mock_repo));
    
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
    let order_update = OrderUpdate {
        status: Some("completed".to_string()),
        ..Default::default()
    };
    
    let updated_order = Order {
        id: 1,
        order_no: "ORD20260113001".to_string(),
        customer_name: "测试客户".to_string(),
        status: "completed".to_string(),
        ..Default::default()
    };
    
    // 设置模拟仓库
    let mut mock_repo = MockOrderRepo::new();
    mock_repo
        .expect_update_order()
        .return_once(|_, _| Ok(Some(updated_order.clone())));
    
    // 创建用例实例
    let use_cases = OrderUseCases::new(Arc::new(mock_repo));
    
    // 执行测试
    let result = use_cases.update_order(1, order_update).await;
    
    // 验证结果
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Some(updated_order));
}

// 测试用例:删除订单
#[tokio::test]
async fn test_delete_order() {
    // 设置模拟仓库
    let mut mock_repo = MockOrderRepo::new();
    mock_repo
        .expect_delete_order()
        .return_once(|_| Ok(true));
    
    // 创建用例实例
    let use_cases = OrderUseCases::new(Arc::new(mock_repo));
    
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
        quantity: 10,
        unit_price: 100.0,
        ..Default::default()
    };
    
    let expected_item = OrderItem {
        id: 1,
        order_id: 1,
        item_name: "商品1".to_string(),
        quantity: 10,
        unit_price: 100.0,
        total_price: 1000.0,
        ..Default::default()
    };
    
    // 设置模拟仓库
    let mut mock_repo = MockOrderRepo::new();
    mock_repo
        .expect_create_order_item()
        .return_once(|_| Ok(expected_item.clone()));
    
    // 创建用例实例
    let use_cases = OrderUseCases::new(Arc::new(mock_repo));
    
    // 执行测试
    let result = use_cases.create_order_item(item_create).await;
    
    // 验证结果
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), expected_item);
}

// 测试用例:创建订单项失败 - 商品名称为空
#[tokio::test]
async fn test_create_order_item_empty_name() {
    // 准备测试数据
    let item_create = OrderItemCreate {
        order_id: 1,
        item_name: "".to_string(),
        quantity: 10,
        unit_price: 100.0,
        ..Default::default()
    };
    
    // 创建用例实例
    let mock_repo = MockOrderRepo::new();
    let use_cases = OrderUseCases::new(Arc::new(mock_repo));
    
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
        quantity: 0,
        unit_price: 100.0,
        ..Default::default()
    };
    
    // 创建用例实例
    let mock_repo = MockOrderRepo::new();
    let use_cases = OrderUseCases::new(Arc::new(mock_repo));
    
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
    let items = vec![
        OrderItem {
            id: 1,
            order_id: 1,
            item_name: "商品1".to_string(),
            quantity: 10,
            unit_price: 100.0,
            total_price: 1000.0,
            ..Default::default()
        },
        OrderItem {
            id: 2,
            order_id: 1,
            item_name: "商品2".to_string(),
            quantity: 5,
            unit_price: 200.0,
            total_price: 1000.0,
            ..Default::default()
        }
    ];
    
    // 设置模拟仓库
    let mut mock_repo = MockOrderRepo::new();
    mock_repo
        .expect_get_order_items()
        .return_once(|_| Ok(items.clone()));
    
    // 创建用例实例
    let use_cases = OrderUseCases::new(Arc::new(mock_repo));
    
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
    let item_update = OrderItemUpdate {
        quantity: Some(20),
        unit_price: Some(150.0),
        ..Default::default()
    };
    
    let updated_item = OrderItem {
        id: 1,
        order_id: 1,
        item_name: "商品1".to_string(),
        quantity: 20,
        unit_price: 150.0,
        total_price: 3000.0,
        ..Default::default()
    };
    
    // 设置模拟仓库
    let mut mock_repo = MockOrderRepo::new();
    mock_repo
        .expect_update_order_item()
        .return_once(|_, _| Ok(Some(updated_item.clone())));
    
    // 创建用例实例
    let use_cases = OrderUseCases::new(Arc::new(mock_repo));
    
    // 执行测试
    let result = use_cases.update_order_item(1, item_update).await;
    
    // 验证结果
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Some(updated_item));
}

// 测试用例:删除订单项
#[tokio::test]
async fn test_delete_order_item() {
    // 设置模拟仓库
    let mut mock_repo = MockOrderRepo::new();
    mock_repo
        .expect_delete_order_item()
        .return_once(|_| Ok(true));
    
    // 创建用例实例
    let use_cases = OrderUseCases::new(Arc::new(mock_repo));
    
    // 执行测试
    let result = use_cases.delete_order_item(1).await;
    
    // 验证结果
    assert!(result.is_ok());
    assert!(result.unwrap());
}






