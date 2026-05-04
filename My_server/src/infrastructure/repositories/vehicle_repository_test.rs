//! / 车辆仓库测试

use sqlx::{PgPool, Executor}; 
use anyhow::Result;
use std::sync::Arc;

use crate::domain::entities::vehicle::{Vehicle, VehicleCreate, VehicleQuery, VehicleUpdate};
use crate::infrastructure::repositories::vehicle_repository::PgVehicleRepository;
use crate::domain::use_cases::vehicle::VehicleRepository;

// 测试前的设置:创建测试数据库连接
async fn setup_test_db() -> PgPool {
    // 使用测试数据库连接
    let pool = PgPool::connect("postgresql://postgres:password@localhost:5432/test_tms_db")
        .await
        .expect("Failed to connect to test database");
    
    // 创建测试所需的表
    pool.execute(r#"
        CREATE TABLE IF NOT EXISTS vehicles (
            vehicle_id SERIAL PRIMARY KEY,
            vehicle_name VARCHAR(100) NOT NULL,
            license_plate VARCHAR(50) UNIQUE NOT NULL,
            vehicle_type VARCHAR(50) NOT NULL,
            vehicle_brand VARCHAR(50),
            vehicle_model VARCHAR(50),
            vehicle_color VARCHAR(50),
            vehicle_weight NUMERIC(10, 2),
            load_weight NUMERIC(10, 2),
            register_date DATE,
            inspection_date DATE,
            insurance_date DATE,
            status VARCHAR(20) NOT NULL DEFAULT 'active',
            create_time TIMESTAMP NOT NULL DEFAULT NOW(),
            update_time TIMESTAMP
        );
    "#).await.expect("Failed to create test tables");
    
    pool
}

// 测试后的清理:删除测试数据
async fn cleanup_test_db(pool: &PgPool) {
    pool.execute("TRUNCATE TABLE vehicles RESTART IDENTITY CASCADE").await.unwrap();
}

// 测试用例:创建车辆
#[tokio::test]
async fn test_create_vehicle() -> Result<()> {
    // 设置测试环境
    let pool = setup_test_db().await;
    let repo = PgVehicleRepository::new(Arc::new(pool.clone()));
    
    // 准备测试数据
    let vehicle_create = VehicleCreate {
        vehicle_name: "测试车辆".to_string(),
        license_plate: "京A12345".to_string(),
        vehicle_type: "卡车".to_string(),
        vehicle_brand: Some("福田".to_string()),
        vehicle_model: Some("欧曼".to_string()),
        vehicle_color: Some("红色".to_string()),
        vehicle_weight: Some(10.0),
        load_weight: Some(20.0),
        register_date: Some("2020-01-01".to_string()),
        inspection_date: Some("2025-01-01".to_string()),
        insurance_date: Some("2025-01-01".to_string()),
        status: "active".to_string(),
    };
    
    // 执行测试
    let result = repo.create_vehicle(vehicle_create).await?;
    
    // 验证结果
    assert!(result.vehicle_id > 0);
    assert_eq!(result.vehicle_name, "测试车辆");
    assert_eq!(result.license_plate, "京A12345");
    assert_eq!(result.vehicle_type, "卡车");
    
    // 清理测试数据
    cleanup_test_db(&pool).await;
    
    Ok(())
}

// 测试用例:获取车辆列表
#[tokio::test]
async fn test_get_vehicles() -> Result<()> {
    // 设置测试环境
    let pool = setup_test_db().await;
    let repo = PgVehicleRepository::new(Arc::new(pool.clone()));
    
    // 创建测试数据
    let vehicle_create = VehicleCreate {
        vehicle_name: "测试车辆2".to_string(),
        license_plate: "京B67890".to_string(),
        vehicle_type: "卡车".to_string(),
        vehicle_brand: Some("福田".to_string()),
        vehicle_model: Some("欧曼".to_string()),
        vehicle_color: Some("红色".to_string()),
        vehicle_weight: Some(10.0),
        load_weight: Some(20.0),
        register_date: Some("2020-01-01".to_string()),
        inspection_date: Some("2025-01-01".to_string()),
        insurance_date: Some("2025-01-01".to_string()),
        status: "active".to_string(),
    };
    repo.create_vehicle(vehicle_create).await?;
    
    // 执行测试
    let query = VehicleQuery {
        vehicle_name: Some("测试".to_string()),
        ..Default::default()
    };
    let result = repo.get_vehicles(query).await?;
    
    // 验证结果
    assert!(result.0.len() > 0);
    assert!(result.1 > 0);
    
    // 清理测试数据
    cleanup_test_db(&pool).await;
    
    Ok(())
}

// 测试用例:获取单个车辆
#[tokio::test]
async fn test_get_vehicle() -> Result<()> {
    // 设置测试环境
    let pool = setup_test_db().await;
    let repo = PgVehicleRepository::new(Arc::new(pool.clone()));
    
    // 创建测试数据
    let vehicle_create = VehicleCreate {
        vehicle_name: "测试车辆3".to_string(),
        license_plate: "京C34567".to_string(),
        vehicle_type: "卡车".to_string(),
        vehicle_brand: Some("福田".to_string()),
        vehicle_model: Some("欧曼".to_string()),
        vehicle_color: Some("红色".to_string()),
        vehicle_weight: Some(10.0),
        load_weight: Some(20.0),
        register_date: Some("2020-01-01".to_string()),
        inspection_date: Some("2025-01-01".to_string()),
        insurance_date: Some("2025-01-01".to_string()),
        status: "active".to_string(),
    };
    let created_vehicle = repo.create_vehicle(vehicle_create).await?;
    
    // 执行测试
    let result = repo.get_vehicle(created_vehicle.vehicle_id).await?;
    
    // 验证结果
    assert!(result.is_some());
    let vehicle = result.unwrap();
    assert_eq!(vehicle.vehicle_id, created_vehicle.vehicle_id);
    assert_eq!(vehicle.license_plate, "京C34567");
    
    // 清理测试数据
    cleanup_test_db(&pool).await;
    
    Ok(())
}

// 测试用例:更新车辆
#[tokio::test]
async fn test_update_vehicle() -> Result<()> {
    // 设置测试环境
    let pool = setup_test_db().await;
    let repo = PgVehicleRepository::new(Arc::new(pool.clone()));
    
    // 创建测试数据
    let vehicle_create = VehicleCreate {
        vehicle_name: "测试车辆4".to_string(),
        license_plate: "京D45678".to_string(),
        vehicle_type: "卡车".to_string(),
        vehicle_brand: Some("福田".to_string()),
        vehicle_model: Some("欧曼".to_string()),
        vehicle_color: Some("红色".to_string()),
        vehicle_weight: Some(10.0),
        load_weight: Some(20.0),
        register_date: Some("2020-01-01".to_string()),
        inspection_date: Some("2025-01-01".to_string()),
        insurance_date: Some("2025-01-01".to_string()),
        status: "active".to_string(),
    };
    let created_vehicle = repo.create_vehicle(vehicle_create).await?;
    
    // 执行测试
    let vehicle_update = VehicleUpdate {
        vehicle_name: Some("更新后的测试车辆".to_string()),
        status: Some("inactive".to_string()),
        ..Default::default()
    };
    let result = repo.update_vehicle(created_vehicle.vehicle_id, vehicle_update).await?;
    
    // 验证结果
    assert!(result.is_some());
    let updated_vehicle = result.unwrap();
    assert_eq!(updated_vehicle.vehicle_id, created_vehicle.vehicle_id);
    assert_eq!(updated_vehicle.vehicle_name, "更新后的测试车辆");
    assert_eq!(updated_vehicle.status, "inactive");
    
    // 清理测试数据
    cleanup_test_db(&pool).await;
    
    Ok(())
}

// 测试用例:删除车辆
#[tokio::test]
async fn test_delete_vehicle() -> Result<()> {
    // 设置测试环境
    let pool = setup_test_db().await;
    let repo = PgVehicleRepository::new(Arc::new(pool.clone()));
    
    // 创建测试数据
    let vehicle_create = VehicleCreate {
        vehicle_name: "测试车辆5".to_string(),
        license_plate: "京E56789".to_string(),
        vehicle_type: "卡车".to_string(),
        vehicle_brand: Some("福田".to_string()),
        vehicle_model: Some("欧曼".to_string()),
        vehicle_color: Some("红色".to_string()),
        vehicle_weight: Some(10.0),
        load_weight: Some(20.0),
        register_date: Some("2020-01-01".to_string()),
        inspection_date: Some("2025-01-01".to_string()),
        insurance_date: Some("2025-01-01".to_string()),
        status: "active".to_string(),
    };
    let created_vehicle = repo.create_vehicle(vehicle_create).await?;
    
    // 执行测试
    let result = repo.delete_vehicle(created_vehicle.vehicle_id).await?;
    
    // 验证结果
    assert!(result);
    
    // 验证车辆已被删除
    let get_result = repo.get_vehicle(created_vehicle.vehicle_id).await?;
    assert!(get_result.is_none());
    
    // 清理测试数据
    cleanup_test_db(&pool).await;
    
    Ok(())
}






