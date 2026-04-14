//! / 设备仓库PostgreSQL实现测试

use sqlx::{PgPool, Executor};
use std::sync::Arc;
use tokio::test;
use anyhow::Result;
use chrono::NaiveDate;

use crate::domain::entities::device::{Device, DeviceCreate, DeviceQuery, DeviceUpdate};
use crate::domain::use_cases::device::DeviceRepository;
use crate::infrastructure::repositories::device_repository::PgDeviceRepository;

// 测试数据库连接字符串
const TEST_DB_URL: &str = "postgres://postgres:password@localhost:5432/tms_test";

// 创建测试数据库连接池
async fn create_test_pool() -> Result<PgPool> {
    let pool = PgPool::connect(TEST_DB_URL).await?;
    Ok(pool)
}

// 初始化测试表
async fn init_test_tables(pool: &PgPool) -> Result<()> {
    // 创建设备表
    pool.execute(r#"
        CREATE TABLE IF NOT EXISTS devices (
            device_id VARCHAR(50) PRIMARY KEY,
            device_name VARCHAR(100) NOT NULL,
            device_type VARCHAR(50) NOT NULL,
            device_model VARCHAR(50) NOT NULL,
            manufacturer VARCHAR(100) NOT NULL,
            serial_number VARCHAR(100) NOT NULL,
            communication_type VARCHAR(50) NOT NULL,
            sim_card_no VARCHAR(20),
            ip_address VARCHAR(50),
            port INTEGER,
            mac_address VARCHAR(50),
            install_date DATE,
            install_address TEXT,
            install_technician VARCHAR(100),
            status INTEGER NOT NULL DEFAULT 1,
            remark TEXT,
            create_time TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            update_time TIMESTAMP,
            create_user_id INTEGER NOT NULL,
            update_user_id INTEGER
        )
    "#).await?;

    // 创建称重数据表
    pool.execute(r#"
        CREATE TABLE IF NOT EXISTS weighing_data (
            id SERIAL PRIMARY KEY,
            vehicle_id INTEGER NOT NULL,
            device_id VARCHAR(50) NOT NULL REFERENCES devices(device_id),
            weighing_time TIMESTAMP NOT NULL,
            gross_weight NUMERIC(10, 2) NOT NULL,
            tare_weight NUMERIC(10, 2),
            net_weight NUMERIC(10, 2) NOT NULL,
            axle_count INTEGER,
            speed NUMERIC(5, 2),
            lane_no INTEGER,
            site_id INTEGER,
            status INTEGER NOT NULL DEFAULT 1,
            create_time TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            update_time TIMESTAMP
        )
    "#).await?;

    Ok(())
}

// 清理测试数据
async fn clean_test_data(pool: &PgPool) -> Result<()> {
    // 删除称重数据
    pool.execute(r#"DELETE FROM weighing_data"#).await?;
    // 删除设备数据
    pool.execute(r#"DELETE FROM devices"#).await?;
    Ok(())
}

#[tokio::test]
async fn test_get_devices() -> Result<()> {
    // 创建测试连接池
    let pool = create_test_pool().await?;
    let arc_pool = Arc::new(pool);
    
    // 初始化测试表
    init_test_tables(&arc_pool).await?;
    
    // 创建设备仓库实例
    let device_repo = PgDeviceRepository::new(arc_pool.clone());
    
    // 创建测试数据
    let now = NaiveDate::from_ymd_opt(2023, 1, 1).unwrap().and_hms_opt(10, 0, 0).unwrap();
    let device_create = DeviceCreate {
        device_id: "DEV001".to_string(),
        device_name: "测试设备1".to_string(),
        device_type: "称重设备".to_string(),
        device_model: "WM-100".to_string(),
        manufacturer: "WeighTech".to_string(),
        serial_number: "SN123456".to_string(),
        communication_type: "4G".to_string(),
        sim_card_no: Some("13800138000".to_string()),
        ip_address: Some("192.168.1.100".to_string()),
        port: Some(8080),
        mac_address: Some("00:11:22:33:44:55".to_string()),
        install_date: Some(now),
        install_address: Some("北京市朝阳区".to_string()),
        install_technician: Some("张三".to_string()),
        status: 1,
        remark: Some("测试设备备注".to_string()),
        create_user_id: 1,
    };
    
    // 创建设备
    device_repo.create_device(device_create).await?;
    
    // 测试获取设备列表
    let query = DeviceQuery::default();
    let (devices, total_count) = device_repo.get_devices(query).await?;
    
    assert_eq!(devices.len(), 1);
    assert_eq!(total_count, 1);
    assert_eq!(devices[0].device_id, "DEV001");
    
    // 清理测试数据
    clean_test_data(&arc_pool).await?;
    
    Ok(())
}

#[tokio::test]
async fn test_get_device() -> Result<()> {
    // 创建测试连接池
    let pool = create_test_pool().await?;
    let arc_pool = Arc::new(pool);
    
    // 初始化测试表
    init_test_tables(&arc_pool).await?;
    
    // 创建设备仓库实例
    let device_repo = PgDeviceRepository::new(arc_pool.clone());
    
    // 创建测试数据
    let now = NaiveDate::from_ymd_opt(2023, 1, 1).unwrap().and_hms_opt(10, 0, 0).unwrap();
    let device_create = DeviceCreate {
        device_id: "DEV001".to_string(),
        device_name: "测试设备1".to_string(),
        device_type: "称重设备".to_string(),
        device_model: "WM-100".to_string(),
        manufacturer: "WeighTech".to_string(),
        serial_number: "SN123456".to_string(),
        communication_type: "4G".to_string(),
        sim_card_no: Some("13800138000".to_string()),
        ip_address: Some("192.168.1.100".to_string()),
        port: Some(8080),
        mac_address: Some("00:11:22:33:44:55".to_string()),
        install_date: Some(now),
        install_address: Some("北京市朝阳区".to_string()),
        install_technician: Some("张三".to_string()),
        status: 1,
        remark: Some("测试设备备注".to_string()),
        create_user_id: 1,
    };
    
    // 创建设备
    device_repo.create_device(device_create).await?;
    
    // 测试获取单个设备
    let device = device_repo.get_device("DEV001").await?;
    
    assert!(device.is_some());
    assert_eq!(device.unwrap().device_id, "DEV001");
    
    // 测试获取不存在的设备
    let device = device_repo.get_device("DEV999").await?;
    
    assert!(device.is_none());
    
    // 清理测试数据
    clean_test_data(&arc_pool).await?;
    
    Ok(())
}

#[tokio::test]
async fn test_create_device() -> Result<()> {
    // 创建测试连接池
    let pool = create_test_pool().await?;
    let arc_pool = Arc::new(pool);
    
    // 初始化测试表
    init_test_tables(&arc_pool).await?;
    
    // 创建设备仓库实例
    let device_repo = PgDeviceRepository::new(arc_pool.clone());
    
    // 创建测试数据
    let now = NaiveDate::from_ymd_opt(2023, 1, 1).unwrap().and_hms_opt(10, 0, 0).unwrap();
    let device_create = DeviceCreate {
        device_id: "DEV001".to_string(),
        device_name: "测试设备1".to_string(),
        device_type: "称重设备".to_string(),
        device_model: "WM-100".to_string(),
        manufacturer: "WeighTech".to_string(),
        serial_number: "SN123456".to_string(),
        communication_type: "4G".to_string(),
        sim_card_no: Some("13800138000".to_string()),
        ip_address: Some("192.168.1.100".to_string()),
        port: Some(8080),
        mac_address: Some("00:11:22:33:44:55".to_string()),
        install_date: Some(now),
        install_address: Some("北京市朝阳区".to_string()),
        install_technician: Some("张三".to_string()),
        status: 1,
        remark: Some("测试设备备注".to_string()),
        create_user_id: 1,
    };
    
    // 创建设备
    let created_device = device_repo.create_device(device_create.clone()).await?;
    
    // 验证创建设备结果
    assert_eq!(created_device.device_id, device_create.device_id);
    assert_eq!(created_device.device_name, device_create.device_name);
    assert_eq!(created_device.device_type, device_create.device_type);
    assert_eq!(created_device.status, device_create.status);
    
    // 清理测试数据
    clean_test_data(&arc_pool).await?;
    
    Ok(())
}

#[tokio::test]
async fn test_update_device() -> Result<()> {
    // 创建测试连接池
    let pool = create_test_pool().await?;
    let arc_pool = Arc::new(pool);
    
    // 初始化测试表
    init_test_tables(&arc_pool).await?;
    
    // 创建设备仓库实例
    let device_repo = PgDeviceRepository::new(arc_pool.clone());
    
    // 创建测试数据
    let now = NaiveDate::from_ymd_opt(2023, 1, 1).unwrap().and_hms_opt(10, 0, 0).unwrap();
    let device_create = DeviceCreate {
        device_id: "DEV001".to_string(),
        device_name: "测试设备1".to_string(),
        device_type: "称重设备".to_string(),
        device_model: "WM-100".to_string(),
        manufacturer: "WeighTech".to_string(),
        serial_number: "SN123456".to_string(),
        communication_type: "4G".to_string(),
        sim_card_no: Some("13800138000".to_string()),
        ip_address: Some("192.168.1.100".to_string()),
        port: Some(8080),
        mac_address: Some("00:11:22:33:44:55".to_string()),
        install_date: Some(now),
        install_address: Some("北京市朝阳区".to_string()),
        install_technician: Some("张三".to_string()),
        status: 1,
        remark: Some("测试设备备注".to_string()),
        create_user_id: 1,
    };
    
    // 创建设备
    device_repo.create_device(device_create).await?;
    
    // 测试更新设备
    let device_update = DeviceUpdate {
        device_name: Some("更新后的设备名称".to_string()),
        device_type: Some("更新后的设备类型".to_string()),
        device_model: Some("WM-200".to_string()),
        manufacturer: Some("UpdatedTech".to_string()),
        serial_number: Some("SN654321".to_string()),
        communication_type: Some("5G".to_string()),
        sim_card_no: Some("13900139000".to_string()),
        ip_address: Some("192.168.1.200".to_string()),
        port: Some(8081),
        mac_address: Some("AA:BB:CC:DD:EE:FF".to_string()),
        install_date: Some(now),
        install_address: Some("上海市浦东新区".to_string()),
        install_technician: Some("李四".to_string()),
        status: Some(2),
        remark: Some("更新后的设备备注".to_string()),
        update_user_id: Some(2),
    };
    
    let updated_device = device_repo.update_device("DEV001", device_update).await?;
    
    assert!(updated_device.is_some());
    assert_eq!(updated_device.unwrap().device_name, "更新后的设备名称");
    assert_eq!(updated_device.unwrap().device_type, "更新后的设备类型");
    assert_eq!(updated_device.unwrap().status, 2);
    
    // 测试更新不存在的设备
    let updated_device = device_repo.update_device("DEV999", DeviceUpdate::default()).await?;
    
    assert!(updated_device.is_none());
    
    // 清理测试数据
    clean_test_data(&arc_pool).await?;
    
    Ok(())
}

#[tokio::test]
async fn test_delete_device() -> Result<()> {
    // 创建测试连接池
    let pool = create_test_pool().await?;
    let arc_pool = Arc::new(pool);
    
    // 初始化测试表
    init_test_tables(&arc_pool).await?;
    
    // 创建设备仓库实例
    let device_repo = PgDeviceRepository::new(arc_pool.clone());
    
    // 创建测试数据
    let now = NaiveDate::from_ymd_opt(2023, 1, 1).unwrap().and_hms_opt(10, 0, 0).unwrap();
    let device_create = DeviceCreate {
        device_id: "DEV001".to_string(),
        device_name: "测试设备1".to_string(),
        device_type: "称重设备".to_string(),
        device_model: "WM-100".to_string(),
        manufacturer: "WeighTech".to_string(),
        serial_number: "SN123456".to_string(),
        communication_type: "4G".to_string(),
        sim_card_no: Some("13800138000".to_string()),
        ip_address: Some("192.168.1.100".to_string()),
        port: Some(8080),
        mac_address: Some("00:11:22:33:44:55".to_string()),
        install_date: Some(now),
        install_address: Some("北京市朝阳区".to_string()),
        install_technician: Some("张三".to_string()),
        status: 1,
        remark: Some("测试设备备注".to_string()),
        create_user_id: 1,
    };
    
    // 创建设备
    device_repo.create_device(device_create).await?;
    
    // 测试删除设备
    let result = device_repo.delete_device("DEV001").await?;
    
    assert!(result);
    
    // 验证设备已被删除
    let device = device_repo.get_device("DEV001").await?;
    
    assert!(device.is_none());
    
    // 测试删除不存在的设备
    let result = device_repo.delete_device("DEV999").await?;
    
    assert!(!result);
    
    // 清理测试数据
    clean_test_data(&arc_pool).await?;
    
    Ok(())
}





