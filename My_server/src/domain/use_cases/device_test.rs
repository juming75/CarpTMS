//! / 设备用例单元测试

use std::sync::Arc;
use async_trait::async_trait;
use chrono::NaiveDate;

use crate::domain::entities::device::{Device, DeviceCreate, DeviceQuery, DeviceUpdate};
use crate::domain::use_cases::device::{DeviceRepository, DeviceUseCases};

// 模拟DeviceRepository实现
struct MockDeviceRepository {
    devices: Vec<Device>,
}

#[async_trait]
impl DeviceRepository for MockDeviceRepository {
    async fn get_devices(&self, _query: DeviceQuery) -> Result<(Vec<Device>, i64), anyhow::Error> {
        Ok((self.devices.clone(), self.devices.len() as i64))
    }
    
    async fn get_device(&self, device_id: &str) -> Result<Option<Device>, anyhow::Error> {
        Ok(self.devices.iter().find(|d| d.device_id == device_id).cloned())
    }
    
    async fn create_device(&self, device: DeviceCreate) -> Result<Device, anyhow::Error> {
        let now = NaiveDate::from_ymd_opt(2023, 1, 1).unwrap().and_hms_opt(10, 0, 0).unwrap();
        let new_device = Device {
            device_id: device.device_id,
            device_name: device.device_name,
            device_type: device.device_type,
            device_model: device.device_model,
            manufacturer: device.manufacturer,
            serial_number: device.serial_number,
            communication_type: device.communication_type,
            sim_card_no: device.sim_card_no,
            ip_address: device.ip_address,
            port: device.port,
            mac_address: device.mac_address,
            install_date: device.install_date,
            install_address: device.install_address,
            install_technician: device.install_technician,
            status: device.status,
            remark: device.remark,
            create_time: now,
            update_time: None,
            create_user_id: device.create_user_id,
            update_user_id: None,
        };
        
        Ok(new_device)
    }
    
    async fn update_device(&self, device_id: &str, device: DeviceUpdate) -> Result<Option<Device>, anyhow::Error> {
        if let Some(existing_device) = self.devices.iter().find(|d| d.device_id == device_id) {
            let now = NaiveDate::from_ymd_opt(2023, 1, 2).unwrap().and_hms_opt(10, 0, 0).unwrap();
            let updated_device = Device {
                device_id: existing_device.device_id.clone(),
                device_name: device.device_name.unwrap_or(existing_device.device_name.clone()),
                device_type: device.device_type.unwrap_or(existing_device.device_type.clone()),
                device_model: device.device_model.unwrap_or(existing_device.device_model.clone()),
                manufacturer: device.manufacturer.unwrap_or(existing_device.manufacturer.clone()),
                serial_number: device.serial_number.unwrap_or(existing_device.serial_number.clone()),
                communication_type: device.communication_type.unwrap_or(existing_device.communication_type.clone()),
                sim_card_no: device.sim_card_no.or(existing_device.sim_card_no.clone()),
                ip_address: device.ip_address.or(existing_device.ip_address.clone()),
                port: device.port.or(existing_device.port),
                mac_address: device.mac_address.or(existing_device.mac_address.clone()),
                install_date: device.install_date.or(existing_device.install_date),
                install_address: device.install_address.or(existing_device.install_address.clone()),
                install_technician: device.install_technician.or(existing_device.install_technician.clone()),
                status: device.status.unwrap_or(existing_device.status),
                remark: device.remark.or(existing_device.remark.clone()),
                create_time: existing_device.create_time,
                update_time: Some(now),
                create_user_id: existing_device.create_user_id,
                update_user_id: device.update_user_id.or(existing_device.update_user_id),
            };
            
            Ok(Some(updated_device))
        } else {
            Ok(None)
        }
    }
    
    async fn delete_device(&self, device_id: &str) -> Result<bool, anyhow::Error> {
        Ok(self.devices.iter().any(|d| d.device_id == device_id))
    }
}

#[tokio::test]
async fn test_get_devices() {
    // 创建模拟仓库
    let now = NaiveDate::from_ymd_opt(2023, 1, 1).unwrap().and_hms_opt(10, 0, 0).unwrap();
    let mock_repo = Arc::new(MockDeviceRepository {
        devices: vec![
            Device {
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
                create_time: now,
                update_time: None,
                create_user_id: 1,
                update_user_id: None,
            },
        ],
    });
    
    // 创建DeviceUseCases实例
    let device_use_cases = DeviceUseCases::new(mock_repo);
    
    // 测试获取设备列表
    let (devices, total_count) = device_use_cases.get_devices(DeviceQuery::default()).await.unwrap();
    assert_eq!(devices.len(), 1);
    assert_eq!(total_count, 1);
    assert_eq!(devices[0].device_id, "DEV001");
}

#[tokio::test]
async fn test_get_device() {
    // 创建模拟仓库
    let now = NaiveDate::from_ymd_opt(2023, 1, 1).unwrap().and_hms_opt(10, 0, 0).unwrap();
    let mock_repo = Arc::new(MockDeviceRepository {
        devices: vec![
            Device {
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
                create_time: now,
                update_time: None,
                create_user_id: 1,
                update_user_id: None,
            },
        ],
    });
    
    // 创建DeviceUseCases实例
    let device_use_cases = DeviceUseCases::new(mock_repo);
    
    // 测试获取存在的设备
    let device = device_use_cases.get_device("DEV001").await.unwrap();
    assert!(device.is_some());
    assert_eq!(device.unwrap().device_id, "DEV001");
    
    // 测试获取不存在的设备
    let device = device_use_cases.get_device("DEV999").await.unwrap();
    assert!(device.is_none());
    
    // 测试空设备ID
    let result = device_use_cases.get_device("").await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().to_string(), "设备ID不能为空");
}

#[tokio::test]
async fn test_create_device() {
    // 创建模拟仓库
    let mock_repo = Arc::new(MockDeviceRepository {
        devices: vec![],
    });
    
    // 创建DeviceUseCases实例
    let device_use_cases = DeviceUseCases::new(mock_repo);
    
    // 测试创建设备
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
    
    let created_device = device_use_cases.create_device(device_create).await.unwrap();
    assert_eq!(created_device.device_id, "DEV001");
    assert_eq!(created_device.device_name, "测试设备1");
    
    // 测试缺少必填字段
    let invalid_device = DeviceCreate {
        device_id: "".to_string(), // 设备ID为空
        device_name: "测试设备2".to_string(),
        device_type: "称重设备".to_string(),
        device_model: "WM-200".to_string(),
        manufacturer: "WeighTech".to_string(),
        serial_number: "SN789012".to_string(),
        communication_type: "5G".to_string(),
        sim_card_no: None,
        ip_address: None,
        port: None,
        mac_address: None,
        install_date: None,
        install_address: None,
        install_technician: None,
        status: 1,
        remark: None,
        create_user_id: 1,
    };
    
    let result = device_use_cases.create_device(invalid_device).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().to_string(), "设备ID不能为空");
    
    // 测试设备名称为空
    let invalid_device = DeviceCreate {
        device_id: "DEV002".to_string(),
        device_name: "".to_string(), // 设备名称为空
        device_type: "称重设备".to_string(),
        device_model: "WM-200".to_string(),
        manufacturer: "WeighTech".to_string(),
        serial_number: "SN789012".to_string(),
        communication_type: "5G".to_string(),
        sim_card_no: None,
        ip_address: None,
        port: None,
        mac_address: None,
        install_date: None,
        install_address: None,
        install_technician: None,
        status: 1,
        remark: None,
        create_user_id: 1,
    };
    
    let result = device_use_cases.create_device(invalid_device).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().to_string(), "设备名称不能为空");
}

#[tokio::test]
async fn test_update_device() {
    // 创建模拟仓库
    let now = NaiveDate::from_ymd_opt(2023, 1, 1).unwrap().and_hms_opt(10, 0, 0).unwrap();
    let mock_repo = Arc::new(MockDeviceRepository {
        devices: vec![
            Device {
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
                create_time: now,
                update_time: None,
                create_user_id: 1,
                update_user_id: None,
            },
        ],
    });
    
    // 创建DeviceUseCases实例
    let device_use_cases = DeviceUseCases::new(mock_repo);
    
    // 测试更新设备
    let device_update = DeviceUpdate {
        device_name: Some("更新设备1".to_string()),
        device_type: None,
        device_model: Some("WM-200".to_string()),
        manufacturer: None,
        serial_number: None,
        communication_type: Some("5G".to_string()),
        sim_card_no: None,
        ip_address: None,
        port: None,
        mac_address: None,
        install_date: None,
        install_address: None,
        install_technician: None,
        status: Some(2),
        remark: None,
        update_user_id: Some(2),
    };
    
    let updated_device = device_use_cases.update_device("DEV001", device_update).await.unwrap();
    assert!(updated_device.is_some());
    let updated_device = updated_device.unwrap();
    assert_eq!(updated_device.device_id, "DEV001");
    assert_eq!(updated_device.device_name, "更新设备1");
    assert_eq!(updated_device.device_model, "WM-200");
    assert_eq!(updated_device.communication_type, "5G");
    assert_eq!(updated_device.status, 2);
    assert_eq!(updated_device.update_user_id, Some(2));
    assert!(updated_device.update_time.is_some());
    
    // 测试更新不存在的设备
    let updated_device = device_use_cases.update_device("DEV999", device_update).await.unwrap();
    assert!(updated_device.is_none());
    
    // 测试空设备ID
    let result = device_use_cases.update_device("", device_update).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().to_string(), "设备ID不能为空");
}

#[tokio::test]
async fn test_delete_device() {
    // 创建模拟仓库
    let now = NaiveDate::from_ymd_opt(2023, 1, 1).unwrap().and_hms_opt(10, 0, 0).unwrap();
    let mock_repo = Arc::new(MockDeviceRepository {
        devices: vec![
            Device {
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
                create_time: now,
                update_time: None,
                create_user_id: 1,
                update_user_id: None,
            },
        ],
    });
    
    // 创建DeviceUseCases实例
    let device_use_cases = DeviceUseCases::new(mock_repo);
    
    // 测试删除设备
    let result = device_use_cases.delete_device("DEV001").await.unwrap();
    assert!(result);
    
    // 测试删除不存在的设备
    let result = device_use_cases.delete_device("DEV999").await.unwrap();
    assert!(!result);
    
    // 测试空设备ID
    let result = device_use_cases.delete_device("").await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().to_string(), "设备ID不能为空");
}






