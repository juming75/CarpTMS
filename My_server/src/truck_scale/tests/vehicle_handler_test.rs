//! / // 车辆管理处理器测试

use super::setup_test_db;
use crate::truck_scale::handlers::VehicleHandler;
use crate::truck_scale::handlers::VehicleInfo;
use chrono;
use uuid;
use std::sync::Arc;

#[tokio::test]
async fn test_query_vehicle() {
    // 设置测试数据库
    let pool = Arc::new(setup_test_db().await.unwrap());
    let handler = VehicleHandler::new(pool);
    
    // 测试查询不存在的车辆
    let result = handler.query_vehicle("non_existent_vehicle").await;
    assert!(result.is_ok());
    let vehicle = result.unwrap();
    assert!(vehicle.is_none());
}

#[tokio::test]
async fn test_query_vehicle_list() {
    // 设置测试数据库
    let pool = Arc::new(setup_test_db().await.unwrap());
    let handler = VehicleHandler::new(pool);
    
    // 测试查询车辆列表(分页)
    let result = handler.query_vehicle_list(1, 10).await;
    assert!(result.is_ok());
    let vehicles = result.unwrap();
    assert!(vehicles.is_empty() || vehicles.len() <= 10);
}

#[tokio::test]
async fn test_add_vehicle() {
    // 设置测试数据库
    let pool = Arc::new(setup_test_db().await.unwrap());
    let handler = VehicleHandler::new(pool);
    
    // 创建测试车辆信息
    let test_vehicle = VehicleInfo {
        vehicle_id: format!("test_vehicle_{}", uuid::Uuid::new_v4()),
        plate_no: "测试车牌".to_string(),
        terminal_no: "test_terminal".to_string(),
        sim_no: "test_sim".to_string(),
        engine_no: "test_engine".to_string(),
        frame_no: "test_frame".to_string(),
        owner_name: "测试车主".to_string(),
        owner_tel: "13800138000".to_string(),
        owner_address: "测试地址".to_string(),
        vehicle_type: "重型卡车".to_string(),
        vehicle_color: "红色".to_string(),
        vehicle_brand: "测试品牌".to_string(),
        vehicle_model: "测试型号".to_string(),
        group_id: "test_group".to_string(),
        driver_name: "测试司机".to_string(),
        driver_tel: "13900139000".to_string(),
        driver_license: "test_license".to_string(),
        max_weight: 50.0,
        tare_weight: 10.0,
        rated_weight: 40.0,
        length: 12.0,
        width: 2.5,
        height: 3.0,
        fuel_type: "柴油".to_string(),
        manufacturer: "测试制造商".to_string(),
        manufacture_date: "2023-01-01".to_string(),
        registration_date: "2023-01-01".to_string(),
        insurance_expire_date: "2024-01-01".to_string(),
        annual_inspection_date: "2024-01-01".to_string(),
        remark: "测试车辆".to_string(),
        status: 0,
        create_time: chrono::Local::now().to_string(),
        update_time: chrono::Local::now().to_string(),
        create_by: "test_user".to_string(),
        update_by: "test_user".to_string(),
    };
    
    // 测试添加车辆
    let result = handler.add_vehicle(test_vehicle.clone()).await;
    assert!(result.is_ok());
    let vehicle_id = result.unwrap();
    assert_eq!(vehicle_id, test_vehicle.vehicle_id);
    
    // 测试查询刚添加的车辆
    let result = handler.query_vehicle(&vehicle_id).await;
    assert!(result.is_ok());
    let vehicle = result.unwrap();
    assert!(vehicle.is_some());
}

#[tokio::test]
async fn test_update_vehicle() {
    // 设置测试数据库
    let pool = Arc::new(setup_test_db().await.unwrap());
    let handler = VehicleHandler::new(pool);
    
    // 创建测试车辆信息
    let mut test_vehicle = VehicleInfo {
        vehicle_id: format!("test_vehicle_{}", uuid::Uuid::new_v4()),
        plate_no: "测试车牌".to_string(),
        terminal_no: "test_terminal".to_string(),
        sim_no: "test_sim".to_string(),
        engine_no: "test_engine".to_string(),
        frame_no: "test_frame".to_string(),
        owner_name: "测试车主".to_string(),
        owner_tel: "13800138000".to_string(),
        owner_address: "测试地址".to_string(),
        vehicle_type: "重型卡车".to_string(),
        vehicle_color: "红色".to_string(),
        vehicle_brand: "测试品牌".to_string(),
        vehicle_model: "测试型号".to_string(),
        group_id: "test_group".to_string(),
        driver_name: "测试司机".to_string(),
        driver_tel: "13900139000".to_string(),
        driver_license: "test_license".to_string(),
        max_weight: 50.0,
        tare_weight: 10.0,
        rated_weight: 40.0,
        length: 12.0,
        width: 2.5,
        height: 3.0,
        fuel_type: "柴油".to_string(),
        manufacturer: "测试制造商".to_string(),
        manufacture_date: "2023-01-01".to_string(),
        registration_date: "2023-01-01".to_string(),
        insurance_expire_date: "2024-01-01".to_string(),
        annual_inspection_date: "2024-01-01".to_string(),
        remark: "测试车辆".to_string(),
        status: 0,
        create_time: chrono::Local::now().to_string(),
        update_time: chrono::Local::now().to_string(),
        create_by: "test_user".to_string(),
        update_by: "test_user".to_string(),
    };
    
    // 添加车辆
    let add_result = handler.add_vehicle(test_vehicle.clone()).await;
    assert!(add_result.is_ok());
    let vehicle_id = add_result.unwrap();
    
    // 更新车辆信息
    test_vehicle.plate_no = "更新后的车牌".to_string();
    test_vehicle.update_by = "update_user".to_string();
    
    let update_result = handler.update_vehicle(test_vehicle).await;
    assert!(update_result.is_ok());
    
    // 测试查询更新后的车辆
    let query_result = handler.query_vehicle(&vehicle_id).await;
    assert!(query_result.is_ok());
    let vehicle = query_result.unwrap();
    assert!(vehicle.is_some());
}

#[tokio::test]
async fn test_delete_vehicle() {
    // 设置测试数据库
    let pool = Arc::new(setup_test_db().await.unwrap());
    let handler = VehicleHandler::new(pool);
    
    // 创建测试车辆信息
    let test_vehicle = VehicleInfo {
        vehicle_id: format!("test_vehicle_{}", uuid::Uuid::new_v4()),
        plate_no: "测试车牌".to_string(),
        terminal_no: "test_terminal".to_string(),
        sim_no: "test_sim".to_string(),
        engine_no: "test_engine".to_string(),
        frame_no: "test_frame".to_string(),
        owner_name: "测试车主".to_string(),
        owner_tel: "13800138000".to_string(),
        owner_address: "测试地址".to_string(),
        vehicle_type: "重型卡车".to_string(),
        vehicle_color: "红色".to_string(),
        vehicle_brand: "测试品牌".to_string(),
        vehicle_model: "测试型号".to_string(),
        group_id: "test_group".to_string(),
        driver_name: "测试司机".to_string(),
        driver_tel: "13900139000".to_string(),
        driver_license: "test_license".to_string(),
        max_weight: 50.0,
        tare_weight: 10.0,
        rated_weight: 40.0,
        length: 12.0,
        width: 2.5,
        height: 3.0,
        fuel_type: "柴油".to_string(),
        manufacturer: "测试制造商".to_string(),
        manufacture_date: "2023-01-01".to_string(),
        registration_date: "2023-01-01".to_string(),
        insurance_expire_date: "2024-01-01".to_string(),
        annual_inspection_date: "2024-01-01".to_string(),
        remark: "测试车辆".to_string(),
        status: 0,
        create_time: chrono::Local::now().to_string(),
        update_time: chrono::Local::now().to_string(),
        create_by: "test_user".to_string(),
        update_by: "test_user".to_string(),
    };
    
    // 添加车辆
    let add_result = handler.add_vehicle(test_vehicle).await;
    assert!(add_result.is_ok());
    let vehicle_id = add_result.unwrap();
    
    // 删除车辆
    let delete_result = handler.delete_vehicle(&vehicle_id, "delete_user").await;
    assert!(delete_result.is_ok());
    
    // 测试查询删除后的车辆(应该返回 None)
    let query_result = handler.query_vehicle(&vehicle_id).await;
    assert!(query_result.is_ok());
    let vehicle = query_result.unwrap();
    assert!(vehicle.is_none());
}






