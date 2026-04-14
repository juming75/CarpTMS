//! / 称重数据仓库PostgreSQL实现测试

use sqlx::{PgPool, Executor};
use std::sync::Arc;
use tokio::test;
use anyhow::Result;
use chrono::NaiveDate;

use crate::domain::entities::weighing_data::{WeighingData, WeighingDataCreate, WeighingDataQuery, WeighingDataUpdate};
use crate::domain::use_cases::weighing_data::WeighingDataRepository;
use crate::infrastructure::repositories::weighing_data_repository::PgWeighingDataRepository;
use crate::infrastructure::repositories::device_repository::PgDeviceRepository;
use crate::domain::entities::device::{DeviceCreate, DeviceQuery};

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

// 创建测试设备
async fn create_test_device(pool: &Arc<PgPool>) -> Result<String> {
    let device_repo = PgDeviceRepository::new(pool.clone());
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
    
    let device = device_repo.create_device(device_create).await?;
    Ok(device.device_id)
}

#[tokio::test]
async fn test_get_weighing_data_list() -> Result<()> {
    // 创建测试连接池
    let pool = create_test_pool().await?;
    let arc_pool = Arc::new(pool);
    
    // 初始化测试表
    init_test_tables(&arc_pool).await?;
    
    // 创建测试设备
    let device_id = create_test_device(&arc_pool).await?;
    
    // 创建称重数据仓库实例
    let weighing_data_repo = PgWeighingDataRepository::new(arc_pool.clone());
    
    // 创建测试数据
    let now = NaiveDate::from_ymd_opt(2023, 1, 1).unwrap().and_hms_opt(10, 0, 0).unwrap();
    let weighing_data_create = WeighingDataCreate {
        vehicle_id: 1,
        device_id: device_id.clone(),
        weighing_time: now,
        gross_weight: 10.5,
        tare_weight: Some(3.5),
        net_weight: 7.0,
        axle_count: Some(3),
        speed: Some(40.5),
        lane_no: Some(1),
        site_id: Some(1),
        status: 1,
    };
    
    // 创建称重数据
    weighing_data_repo.create_weighing_data(weighing_data_create).await?;
    
    // 测试获取称重数据列表
    let query = WeighingDataQuery::default();
    let (weighing_data, total_count) = weighing_data_repo.get_weighing_data_list(query).await?;
    
    assert_eq!(weighing_data.len(), 1);
    assert_eq!(total_count, 1);
    assert_eq!(weighing_data[0].device_id, device_id);
    assert_eq!(weighing_data[0].gross_weight, 10.5);
    
    // 清理测试数据
    clean_test_data(&arc_pool).await?;
    
    Ok(())
}

#[tokio::test]
async fn test_get_weighing_data() -> Result<()> {
    // 创建测试连接池
    let pool = create_test_pool().await?;
    let arc_pool = Arc::new(pool);
    
    // 初始化测试表
    init_test_tables(&arc_pool).await?;
    
    // 创建测试设备
    let device_id = create_test_device(&arc_pool).await?;
    
    // 创建称重数据仓库实例
    let weighing_data_repo = PgWeighingDataRepository::new(arc_pool.clone());
    
    // 创建测试数据
    let now = NaiveDate::from_ymd_opt(2023, 1, 1).unwrap().and_hms_opt(10, 0, 0).unwrap();
    let weighing_data_create = WeighingDataCreate {
        vehicle_id: 1,
        device_id: device_id.clone(),
        weighing_time: now,
        gross_weight: 10.5,
        tare_weight: Some(3.5),
        net_weight: 7.0,
        axle_count: Some(3),
        speed: Some(40.5),
        lane_no: Some(1),
        site_id: Some(1),
        status: 1,
    };
    
    // 创建称重数据
    let created_data = weighing_data_repo.create_weighing_data(weighing_data_create).await?;
    
    // 测试获取单个称重数据
    let weighing_data = weighing_data_repo.get_weighing_data(created_data.id).await?;
    
    assert!(weighing_data.is_some());
    assert_eq!(weighing_data.unwrap().id, created_data.id);
    
    // 测试获取不存在的称重数据
    let weighing_data = weighing_data_repo.get_weighing_data(999).await?;
    
    assert!(weighing_data.is_none());
    
    // 清理测试数据
    clean_test_data(&arc_pool).await?;
    
    Ok(())
}

#[tokio::test]
async fn test_create_weighing_data() -> Result<()> {
    // 创建测试连接池
    let pool = create_test_pool().await?;
    let arc_pool = Arc::new(pool);
    
    // 初始化测试表
    init_test_tables(&arc_pool).await?;
    
    // 创建测试设备
    let device_id = create_test_device(&arc_pool).await?;
    
    // 创建称重数据仓库实例
    let weighing_data_repo = PgWeighingDataRepository::new(arc_pool.clone());
    
    // 创建测试数据
    let now = NaiveDate::from_ymd_opt(2023, 1, 1).unwrap().and_hms_opt(10, 0, 0).unwrap();
    let weighing_data_create = WeighingDataCreate {
        vehicle_id: 1,
        device_id: device_id.clone(),
        weighing_time: now,
        gross_weight: 10.5,
        tare_weight: Some(3.5),
        net_weight: 7.0,
        axle_count: Some(3),
        speed: Some(40.5),
        lane_no: Some(1),
        site_id: Some(1),
        status: 1,
    };
    
    // 创建称重数据
    let created_data = weighing_data_repo.create_weighing_data(weighing_data_create.clone()).await?;
    
    // 验证创建称重数据结果
    assert_eq!(created_data.device_id, weighing_data_create.device_id);
    assert_eq!(created_data.vehicle_id, weighing_data_create.vehicle_id);
    assert_eq!(created_data.gross_weight, weighing_data_create.gross_weight);
    assert_eq!(created_data.net_weight, weighing_data_create.net_weight);
    
    // 清理测试数据
    clean_test_data(&arc_pool).await?;
    
    Ok(())
}

#[tokio::test]
async fn test_update_weighing_data() -> Result<()> {
    // 创建测试连接池
    let pool = create_test_pool().await?;
    let arc_pool = Arc::new(pool);
    
    // 初始化测试表
    init_test_tables(&arc_pool).await?;
    
    // 创建测试设备
    let device_id = create_test_device(&arc_pool).await?;
    
    // 创建称重数据仓库实例
    let weighing_data_repo = PgWeighingDataRepository::new(arc_pool.clone());
    
    // 创建测试数据
    let now = NaiveDate::from_ymd_opt(2023, 1, 1).unwrap().and_hms_opt(10, 0, 0).unwrap();
    let weighing_data_create = WeighingDataCreate {
        vehicle_id: 1,
        device_id: device_id.clone(),
        weighing_time: now,
        gross_weight: 10.5,
        tare_weight: Some(3.5),
        net_weight: 7.0,
        axle_count: Some(3),
        speed: Some(40.5),
        lane_no: Some(1),
        site_id: Some(1),
        status: 1,
    };
    
    // 创建称重数据
    let created_data = weighing_data_repo.create_weighing_data(weighing_data_create).await?;
    
    // 测试更新称重数据
    let weighing_data_update = WeighingDataUpdate {
        vehicle_id: Some(2),
        device_id: Some(device_id.clone()),
        weighing_time: Some(now),
        gross_weight: Some(15.5),
        tare_weight: Some(4.5),
        net_weight: Some(11.0),
        axle_count: Some(4),
        speed: Some(50.5),
        lane_no: Some(2),
        site_id: Some(2),
        status: Some(2),
    };
    
    let updated_data = weighing_data_repo.update_weighing_data(created_data.id, weighing_data_update).await?;
    
    assert!(updated_data.is_some());
    assert_eq!(updated_data.unwrap().vehicle_id, 2);
    assert_eq!(updated_data.unwrap().gross_weight, 15.5);
    assert_eq!(updated_data.unwrap().net_weight, 11.0);
    assert_eq!(updated_data.unwrap().status, 2);
    
    // 测试更新不存在的称重数据
    let updated_data = weighing_data_repo.update_weighing_data(999, WeighingDataUpdate::default()).await?;
    
    assert!(updated_data.is_none());
    
    // 清理测试数据
    clean_test_data(&arc_pool).await?;
    
    Ok(())
}

#[tokio::test]
async fn test_delete_weighing_data() -> Result<()> {
    // 创建测试连接池
    let pool = create_test_pool().await?;
    let arc_pool = Arc::new(pool);
    
    // 初始化测试表
    init_test_tables(&arc_pool).await?;
    
    // 创建测试设备
    let device_id = create_test_device(&arc_pool).await?;
    
    // 创建称重数据仓库实例
    let weighing_data_repo = PgWeighingDataRepository::new(arc_pool.clone());
    
    // 创建测试数据
    let now = NaiveDate::from_ymd_opt(2023, 1, 1).unwrap().and_hms_opt(10, 0, 0).unwrap();
    let weighing_data_create = WeighingDataCreate {
        vehicle_id: 1,
        device_id: device_id.clone(),
        weighing_time: now,
        gross_weight: 10.5,
        tare_weight: Some(3.5),
        net_weight: 7.0,
        axle_count: Some(3),
        speed: Some(40.5),
        lane_no: Some(1),
        site_id: Some(1),
        status: 1,
    };
    
    // 创建称重数据
    let created_data = weighing_data_repo.create_weighing_data(weighing_data_create).await?;
    
    // 测试删除称重数据
    let result = weighing_data_repo.delete_weighing_data(created_data.id).await?;
    
    assert!(result);
    
    // 验证称重数据已被删除
    let weighing_data = weighing_data_repo.get_weighing_data(created_data.id).await?;
    
    assert!(weighing_data.is_none());
    
    // 测试删除不存在的称重数据
    let result = weighing_data_repo.delete_weighing_data(999).await?;
    
    assert!(!result);
    
    // 清理测试数据
    clean_test_data(&arc_pool).await?;
    
    Ok(())
}

#[tokio::test]
async fn test_get_weighing_data_stats_by_vehicle() -> Result<()> {
    // 创建测试连接池
    let pool = create_test_pool().await?;
    let arc_pool = Arc::new(pool);
    
    // 初始化测试表
    init_test_tables(&arc_pool).await?;
    
    // 创建测试设备
    let device_id = create_test_device(&arc_pool).await?;
    
    // 创建称重数据仓库实例
    let weighing_data_repo = PgWeighingDataRepository::new(arc_pool.clone());
    
    // 创建测试数据
    let now = NaiveDate::from_ymd_opt(2023, 1, 1).unwrap().and_hms_opt(10, 0, 0).unwrap();
    
    // 为同一车辆创建多条称重数据
    for i in 1..=3 {
        let weighing_data_create = WeighingDataCreate {
            vehicle_id: 1,
            device_id: device_id.clone(),
            weighing_time: now,
            gross_weight: 10.5 + i as f64,
            tare_weight: Some(3.5),
            net_weight: 7.0 + i as f64,
            axle_count: Some(3),
            speed: Some(40.5),
            lane_no: Some(1),
            site_id: Some(1),
            status: 1,
        };
        
        weighing_data_repo.create_weighing_data(weighing_data_create).await?;
    }
    
    // 测试按车辆获取称重数据统计
    let stats = weighing_data_repo.get_weighing_data_stats_by_vehicle(1, now, now).await?;
    
    assert_eq!(stats.len(), 3);
    
    // 清理测试数据
    clean_test_data(&arc_pool).await?;
    
    Ok(())
}

#[tokio::test]
async fn test_get_weighing_data_stats_by_device() -> Result<()> {
    // 创建测试连接池
    let pool = create_test_pool().await?;
    let arc_pool = Arc::new(pool);
    
    // 初始化测试表
    init_test_tables(&arc_pool).await?;
    
    // 创建测试设备
    let device_id = create_test_device(&arc_pool).await?;
    
    // 创建称重数据仓库实例
    let weighing_data_repo = PgWeighingDataRepository::new(arc_pool.clone());
    
    // 创建测试数据
    let now = NaiveDate::from_ymd_opt(2023, 1, 1).unwrap().and_hms_opt(10, 0, 0).unwrap();
    
    // 为同一设备创建多条称重数据
    for i in 1..=3 {
        let weighing_data_create = WeighingDataCreate {
            vehicle_id: i,
            device_id: device_id.clone(),
            weighing_time: now,
            gross_weight: 10.5 + i as f64,
            tare_weight: Some(3.5),
            net_weight: 7.0 + i as f64,
            axle_count: Some(3),
            speed: Some(40.5),
            lane_no: Some(1),
            site_id: Some(1),
            status: 1,
        };
        
        weighing_data_repo.create_weighing_data(weighing_data_create).await?;
    }
    
    // 测试按设备获取称重数据统计
    let stats = weighing_data_repo.get_weighing_data_stats_by_device(&device_id, now, now).await?;
    
    assert_eq!(stats.len(), 3);
    
    // 清理测试数据
    clean_test_data(&arc_pool).await?;
    
    Ok(())
}





