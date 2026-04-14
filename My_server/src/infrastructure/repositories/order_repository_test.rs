//! / 订单仓库测试

use sqlx::{PgPool, Executor}; 
use anyhow::Result;
use std::sync::Arc;

use crate::domain::entities::order::{Order, OrderCreate, OrderItem, OrderItemCreate, OrderItemUpdate, OrderQuery, OrderUpdate};
use crate::infrastructure::repositories::order_repository::PgOrderRepository;
use crate::domain::use_cases::order::OrderRepository;

// 测试前的设置:创建测试数据库连接
async fn setup_test_db() -> PgPool {
    // 使用测试数据库连接
    let pool = PgPool::connect("postgresql://postgres:password@localhost:5432/test_tms_db")
        .await
        .expect("Failed to connect to test database");
    
    // 创建测试所需的表
    pool.execute(r#"
        CREATE TABLE IF NOT EXISTS orders (
            order_id SERIAL PRIMARY KEY,
            order_no VARCHAR(50) UNIQUE NOT NULL,
            customer_name VARCHAR(100) NOT NULL,
            customer_phone VARCHAR(20) NOT NULL,
            origin VARCHAR(100) NOT NULL,
            destination VARCHAR(100) NOT NULL,
            cargo_weight NUMERIC(10, 2) NOT NULL,
            cargo_volume NUMERIC(10, 2) NOT NULL,
            cargo_count INTEGER NOT NULL,
            order_amount NUMERIC(15, 2) NOT NULL,
            order_status VARCHAR(20) NOT NULL,
            driver_name VARCHAR(100),
            driver_phone VARCHAR(20),
            vehicle_id INTEGER,
            create_time TIMESTAMP NOT NULL DEFAULT NOW(),
            update_time TIMESTAMP
        );
        
        CREATE TABLE IF NOT EXISTS order_items (
            item_id SERIAL PRIMARY KEY,
            order_id INTEGER NOT NULL REFERENCES orders(order_id) ON DELETE CASCADE,
            item_name VARCHAR(100) NOT NULL,
            item_description TEXT,
            quantity INTEGER NOT NULL,
            unit_price NUMERIC(10, 2) NOT NULL,
            total_price NUMERIC(15, 2) NOT NULL,
            create_time TIMESTAMP NOT NULL DEFAULT NOW(),
            update_time TIMESTAMP
        );
    "#).await.expect("Failed to create test tables");
    
    pool
}

// 测试后的清理:删除测试数据
async fn cleanup_test_db(pool: &PgPool) {
    pool.execute("TRUNCATE TABLE order_items RESTART IDENTITY CASCADE").await.unwrap();
    pool.execute("TRUNCATE TABLE orders RESTART IDENTITY CASCADE").await.unwrap();
}

// 测试用例:创建订单
#[tokio::test]
async fn test_create_order() -> Result<()> {
    // 设置测试环境
    let pool = setup_test_db().await;
    let repo = PgOrderRepository::new(Arc::new(pool.clone()));
    
    // 准备测试数据
    let order_create = OrderCreate {
        order_no: "TEST-001".to_string(),
        customer_name: "测试客户".to_string(),
        customer_phone: "13800138000".to_string(),
        origin: "北京".to_string(),
        destination: "上海".to_string(),
        cargo_weight: 10.5,
        cargo_volume: 5.2,
        cargo_count: 100,
        order_amount: 10000.0,
        order_status: "pending".to_string(),
        driver_name: Some("张三".to_string()),
        driver_phone: Some("13900139000".to_string()),
        vehicle_id: Some(1),
    };
    
    // 执行测试
    let result = repo.create_order(order_create).await?;
    
    // 验证结果
    assert!(result.order_id > 0);
    assert_eq!(result.order_no, "TEST-001");
    assert_eq!(result.customer_name, "测试客户");
    
    // 清理测试数据
    cleanup_test_db(&pool).await;
    
    Ok(())
}

// 测试用例:获取订单列表
#[tokio::test]
async fn test_get_orders() -> Result<()> {
    // 设置测试环境
    let pool = setup_test_db().await;
    let repo = PgOrderRepository::new(Arc::new(pool.clone()));
    
    // 创建测试数据
    let order_create = OrderCreate {
        order_no: "TEST-002".to_string(),
        customer_name: "测试客户2".to_string(),
        customer_phone: "13800138000".to_string(),
        origin: "北京".to_string(),
        destination: "上海".to_string(),
        cargo_weight: 10.5,
        cargo_volume: 5.2,
        cargo_count: 100,
        order_amount: 10000.0,
        order_status: "pending".to_string(),
        driver_name: Some("张三".to_string()),
        driver_phone: Some("13900139000".to_string()),
        vehicle_id: Some(1),
    };
    repo.create_order(order_create).await?;
    
    // 执行测试
    let query = OrderQuery {
        order_no: Some("TEST".to_string()),
        ..Default::default()
    };
    let result = repo.get_orders(query).await?;
    
    // 验证结果
    assert!(result.0.len() > 0);
    assert!(result.1 > 0);
    
    // 清理测试数据
    cleanup_test_db(&pool).await;
    
    Ok(())
}

// 测试用例:获取单个订单
#[tokio::test]
async fn test_get_order() -> Result<()> {
    // 设置测试环境
    let pool = setup_test_db().await;
    let repo = PgOrderRepository::new(Arc::new(pool.clone()));
    
    // 创建测试数据
    let order_create = OrderCreate {
        order_no: "TEST-003".to_string(),
        customer_name: "测试客户3".to_string(),
        customer_phone: "13800138000".to_string(),
        origin: "北京".to_string(),
        destination: "上海".to_string(),
        cargo_weight: 10.5,
        cargo_volume: 5.2,
        cargo_count: 100,
        order_amount: 10000.0,
        order_status: "pending".to_string(),
        driver_name: Some("张三".to_string()),
        driver_phone: Some("13900139000".to_string()),
        vehicle_id: Some(1),
    };
    let created_order = repo.create_order(order_create).await?;
    
    // 执行测试
    let result = repo.get_order(created_order.order_id).await?;
    
    // 验证结果
    assert!(result.is_some());
    let order = result.unwrap();
    assert_eq!(order.order_id, created_order.order_id);
    assert_eq!(order.order_no, "TEST-003");
    
    // 清理测试数据
    cleanup_test_db(&pool).await;
    
    Ok(())
}

// 测试用例:更新订单
#[tokio::test]
async fn test_update_order() -> Result<()> {
    // 设置测试环境
    let pool = setup_test_db().await;
    let repo = PgOrderRepository::new(Arc::new(pool.clone()));
    
    // 创建测试数据
    let order_create = OrderCreate {
        order_no: "TEST-004".to_string(),
        customer_name: "测试客户4".to_string(),
        customer_phone: "13800138000".to_string(),
        origin: "北京".to_string(),
        destination: "上海".to_string(),
        cargo_weight: 10.5,
        cargo_volume: 5.2,
        cargo_count: 100,
        order_amount: 10000.0,
        order_status: "pending".to_string(),
        driver_name: Some("张三".to_string()),
        driver_phone: Some("13900139000".to_string()),
        vehicle_id: Some(1),
    };
    let created_order = repo.create_order(order_create).await?;
    
    // 执行测试
    let order_update = OrderUpdate {
        order_status: Some("completed".to_string()),
        ..Default::default()
    };
    let result = repo.update_order(created_order.order_id, order_update).await?;
    
    // 验证结果
    assert!(result.is_some());
    let updated_order = result.unwrap();
    assert_eq!(updated_order.order_id, created_order.order_id);
    assert_eq!(updated_order.order_status, "completed");
    
    // 清理测试数据
    cleanup_test_db(&pool).await;
    
    Ok(())
}

// 测试用例:删除订单
#[tokio::test]
async fn test_delete_order() -> Result<()> {
    // 设置测试环境
    let pool = setup_test_db().await;
    let repo = PgOrderRepository::new(Arc::new(pool.clone()));
    
    // 创建测试数据
    let order_create = OrderCreate {
        order_no: "TEST-005".to_string(),
        customer_name: "测试客户5".to_string(),
        customer_phone: "13800138000".to_string(),
        origin: "北京".to_string(),
        destination: "上海".to_string(),
        cargo_weight: 10.5,
        cargo_volume: 5.2,
        cargo_count: 100,
        order_amount: 10000.0,
        order_status: "pending".to_string(),
        driver_name: Some("张三".to_string()),
        driver_phone: Some("13900139000".to_string()),
        vehicle_id: Some(1),
    };
    let created_order = repo.create_order(order_create).await?;
    
    // 执行测试
    let result = repo.delete_order(created_order.order_id).await?;
    
    // 验证结果
    assert!(result);
    
    // 验证订单已被删除
    let get_result = repo.get_order(created_order.order_id).await?;
    assert!(get_result.is_none());
    
    // 清理测试数据
    cleanup_test_db(&pool).await;
    
    Ok(())
}

// 测试用例:创建订单项
#[tokio::test]
async fn test_create_order_item() -> Result<()> {
    // 设置测试环境
    let pool = setup_test_db().await;
    let repo = PgOrderRepository::new(Arc::new(pool.clone()));
    
    // 创建订单
    let order_create = OrderCreate {
        order_no: "TEST-006".to_string(),
        customer_name: "测试客户6".to_string(),
        customer_phone: "13800138000".to_string(),
        origin: "北京".to_string(),
        destination: "上海".to_string(),
        cargo_weight: 10.5,
        cargo_volume: 5.2,
        cargo_count: 100,
        order_amount: 10000.0,
        order_status: "pending".to_string(),
        driver_name: Some("张三".to_string()),
        driver_phone: Some("13900139000".to_string()),
        vehicle_id: Some(1),
    };
    let created_order = repo.create_order(order_create).await?;
    
    // 准备测试数据
    let item_create = OrderItemCreate {
        order_id: created_order.order_id,
        item_name: "测试商品".to_string(),
        item_description: Some("测试商品描述".to_string()),
        quantity: 10,
        unit_price: 100.0,
        total_price: 1000.0,
    };
    
    // 执行测试
    let result = repo.create_order_item(item_create).await?;
    
    // 验证结果
    assert!(result.item_id > 0);
    assert_eq!(result.item_name, "测试商品");
    assert_eq!(result.order_id, created_order.order_id);
    
    // 清理测试数据
    cleanup_test_db(&pool).await;
    
    Ok(())
}

// 测试用例:获取订单项列表
#[tokio::test]
async fn test_get_order_items() -> Result<()> {
    // 设置测试环境
    let pool = setup_test_db().await;
    let repo = PgOrderRepository::new(Arc::new(pool.clone()));
    
    // 创建订单
    let order_create = OrderCreate {
        order_no: "TEST-007".to_string(),
        customer_name: "测试客户7".to_string(),
        customer_phone: "13800138000".to_string(),
        origin: "北京".to_string(),
        destination: "上海".to_string(),
        cargo_weight: 10.5,
        cargo_volume: 5.2,
        cargo_count: 100,
        order_amount: 10000.0,
        order_status: "pending".to_string(),
        driver_name: Some("张三".to_string()),
        driver_phone: Some("13900139000".to_string()),
        vehicle_id: Some(1),
    };
    let created_order = repo.create_order(order_create).await?;
    
    // 创建订单项
    let item_create = OrderItemCreate {
        order_id: created_order.order_id,
        item_name: "测试商品".to_string(),
        item_description: Some("测试商品描述".to_string()),
        quantity: 10,
        unit_price: 100.0,
        total_price: 1000.0,
    };
    repo.create_order_item(item_create).await?;
    
    // 执行测试
    let result = repo.get_order_items(created_order.order_id).await?;
    
    // 验证结果
    assert!(result.len() > 0);
    
    // 清理测试数据
    cleanup_test_db(&pool).await;
    
    Ok(())
}

// 测试用例:更新订单项
#[tokio::test]
async fn test_update_order_item() -> Result<()> {
    // 设置测试环境
    let pool = setup_test_db().await;
    let repo = PgOrderRepository::new(Arc::new(pool.clone()));
    
    // 创建订单
    let order_create = OrderCreate {
        order_no: "TEST-008".to_string(),
        customer_name: "测试客户8".to_string(),
        customer_phone: "13800138000".to_string(),
        origin: "北京".to_string(),
        destination: "上海".to_string(),
        cargo_weight: 10.5,
        cargo_volume: 5.2,
        cargo_count: 100,
        order_amount: 10000.0,
        order_status: "pending".to_string(),
        driver_name: Some("张三".to_string()),
        driver_phone: Some("13900139000".to_string()),
        vehicle_id: Some(1),
    };
    let created_order = repo.create_order(order_create).await?;
    
    // 创建订单项
    let item_create = OrderItemCreate {
        order_id: created_order.order_id,
        item_name: "测试商品".to_string(),
        item_description: Some("测试商品描述".to_string()),
        quantity: 10,
        unit_price: 100.0,
        total_price: 1000.0,
    };
    let created_item = repo.create_order_item(item_create).await?;
    
    // 执行测试
    let item_update = OrderItemUpdate {
        quantity: Some(20),
        unit_price: Some(150.0),
        ..Default::default()
    };
    let result = repo.update_order_item(created_item.item_id, item_update).await?;
    
    // 验证结果
    assert!(result.is_some());
    let updated_item = result.unwrap();
    assert_eq!(updated_item.item_id, created_item.item_id);
    assert_eq!(updated_item.quantity, 20);
    assert_eq!(updated_item.unit_price, 150.0);
    
    // 清理测试数据
    cleanup_test_db(&pool).await;
    
    Ok(())
}

// 测试用例:删除订单项
#[tokio::test]
async fn test_delete_order_item() -> Result<()> {
    // 设置测试环境
    let pool = setup_test_db().await;
    let repo = PgOrderRepository::new(Arc::new(pool.clone()));
    
    // 创建订单
    let order_create = OrderCreate {
        order_no: "TEST-009".to_string(),
        customer_name: "测试客户9".to_string(),
        customer_phone: "13800138000".to_string(),
        origin: "北京".to_string(),
        destination: "上海".to_string(),
        cargo_weight: 10.5,
        cargo_volume: 5.2,
        cargo_count: 100,
        order_amount: 10000.0,
        order_status: "pending".to_string(),
        driver_name: Some("张三".to_string()),
        driver_phone: Some("13900139000".to_string()),
        vehicle_id: Some(1),
    };
    let created_order = repo.create_order(order_create).await?;
    
    // 创建订单项
    let item_create = OrderItemCreate {
        order_id: created_order.order_id,
        item_name: "测试商品".to_string(),
        item_description: Some("测试商品描述".to_string()),
        quantity: 10,
        unit_price: 100.0,
        total_price: 1000.0,
    };
    let created_item = repo.create_order_item(item_create).await?;
    
    // 执行测试
    let result = repo.delete_order_item(created_item.item_id).await?;
    
    // 验证结果
    assert!(result);
    
    // 清理测试数据
    cleanup_test_db(&pool).await;
    
    Ok(())
}






