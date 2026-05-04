//! / 数据库操作测试

use super::setup_test_db;
use crate::truck_scale::db::TruckScaleDb;
use std::sync::Arc;

#[tokio::test]
async fn test_db_connection() {
    // 设置测试数据库
    let pool = Arc::new(setup_test_db().await.unwrap());
    let db = TruckScaleDb::new(pool);
    
    // 测试获取数据库连接池
    let db_pool = db.pool();
    assert!(db_pool.is_some());
}

#[tokio::test]
async fn test_query_vehicle() {
    // 设置测试数据库
    let pool = Arc::new(setup_test_db().await.unwrap());
    let db = TruckScaleDb::new(pool);
    
    // 测试查询不存在的车辆
    let result = db.query_vehicle("non_existent_vehicle").await;
    assert!(result.is_ok());
    let vehicle = result.unwrap();
    assert!(vehicle.is_none());
}

#[tokio::test]
async fn test_query_user() {
    // 设置测试数据库
    let pool = Arc::new(setup_test_db().await.unwrap());
    let db = TruckScaleDb::new(pool);
    
    // 测试查询不存在的用户
    let result = db.query_user("non_existent_user").await;
    assert!(result.is_ok());
    let user = result.unwrap();
    assert!(user.is_none());
}

#[tokio::test]
async fn test_query_user_by_name() {
    // 设置测试数据库
    let pool = Arc::new(setup_test_db().await.unwrap());
    let db = TruckScaleDb::new(pool);
    
    // 测试查询不存在的用户
    let result = db.query_user_by_name("non_existent_user").await;
    assert!(result.is_ok());
    let user = result.unwrap();
    assert!(user.is_none());
}

#[tokio::test]
async fn test_query_vehicle_list() {
    // 设置测试数据库
    let pool = Arc::new(setup_test_db().await.unwrap());
    let db = TruckScaleDb::new(pool);
    
    // 测试查询车辆列表(分页)
    let result = db.query_vehicle_list(0, 10).await;
    assert!(result.is_ok());
    let vehicles = result.unwrap();
    assert!(vehicles.is_empty() || vehicles.len() <= 10);
}

#[tokio::test]
async fn test_query_user_list() {
    // 设置测试数据库
    let pool = Arc::new(setup_test_db().await.unwrap());
    let db = TruckScaleDb::new(pool);
    
    // 测试查询用户列表(分页)
    let result = db.query_user_list(0, 10).await;
    assert!(result.is_ok());
    let users = result.unwrap();
    assert!(users.is_empty() || users.len() <= 10);
}






