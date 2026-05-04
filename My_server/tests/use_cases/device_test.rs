//! 设备用例集成测试
//!
//! 独立的集成测试，不依赖内嵌测试模块

use std::sync::Arc;
use carptms::domain::use_cases::device::DeviceUseCases;
use carptms::domain::use_cases::device::repository::DeviceRepository;
use carptms::domain::entities::device::{Device, DeviceCreate, DeviceQuery, DeviceUpdate};
use async_trait::async_trait;
use chrono::NaiveDate;

#[allow(dead_code)]
struct MockDeviceRepo {
    devices: Vec<Device>,
}

#[async_trait]
impl DeviceRepository for MockDeviceRepo {
    async fn get_devices(&self, _query: DeviceQuery) -> Result<(Vec<Device>, i64), anyhow::Error> {
        Ok((self.devices.clone(), self.devices.len() as i64))
    }

    async fn get_device(&self, device_id: &str) -> Result<Option<Device>, anyhow::Error> {
        Ok(self
            .devices
            .iter()
            .find(|d| d.device_id == device_id)
            .cloned())
    }

    async fn create_device(&self, device: DeviceCreate) -> Result<Device, anyhow::Error> {
        let now = NaiveDate::from_ymd_opt(2023, 1, 1)
            .unwrap()
            .and_hms_opt(10, 0, 0)
            .unwrap();
        Ok(Device {
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
        })
    }

    async fn update_device(
        &self,
        device_id: &str,
        device: DeviceUpdate,
    ) -> Result<Option<Device>, anyhow::Error> {
        if let Some(existing) = self.devices.iter().find(|d| d.device_id == device_id) {
            let now = NaiveDate::from_ymd_opt(2023, 1, 2)
                .unwrap()
                .and_hms_opt(10, 0, 0)
                .unwrap();
            Ok(Some(Device {
                device_id: existing.device_id.clone(),
                device_name: device.device_name.unwrap_or(existing.device_name.clone()),
                device_type: device.device_type.unwrap_or(existing.device_type.clone()),
                device_model: device.device_model.unwrap_or(existing.device_model.clone()),
                manufacturer: device.manufacturer.unwrap_or(existing.manufacturer.clone()),
                serial_number: device
                    .serial_number
                    .unwrap_or(existing.serial_number.clone()),
                communication_type: device
                    .communication_type
                    .unwrap_or(existing.communication_type.clone()),
                sim_card_no: device.sim_card_no.or(existing.sim_card_no.clone()),
                ip_address: device.ip_address.or(existing.ip_address.clone()),
                port: device.port.or(existing.port),
                mac_address: device.mac_address.or(existing.mac_address.clone()),
                install_date: device.install_date.or(existing.install_date),
                install_address: device.install_address.or(existing.install_address.clone()),
                install_technician: device
                    .install_technician
                    .or(existing.install_technician.clone()),
                status: device.status.unwrap_or(existing.status),
                remark: device.remark.or(existing.remark.clone()),
                create_time: existing.create_time,
                update_time: Some(now),
                create_user_id: existing.create_user_id,
                update_user_id: device.update_user_id.or(existing.update_user_id),
            }))
        } else {
            Ok(None)
        }
    }

    async fn delete_device(&self, device_id: &str) -> Result<bool, anyhow::Error> {
        Ok(self.devices.iter().any(|d| d.device_id == device_id))
    }
}

#[tokio::test]
async fn test_create_device_success() {
    let now = NaiveDate::from_ymd_opt(2023, 1, 1)
        .unwrap()
        .and_hms_opt(10, 0, 0)
        .unwrap();
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

    let mock_repo = Arc::new(MockDeviceRepo { devices: vec![] });
    let use_cases = DeviceUseCases::new(mock_repo);
    let result: Result<Device, anyhow::Error> = use_cases.create_device(device_create).await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap().device_id, "DEV001");
}

#[tokio::test]
async fn test_create_device_empty_id() {
    let device_create = DeviceCreate {
        device_id: "".to_string(),
        device_name: "测试设备1".to_string(),
        device_type: "称重设备".to_string(),
        device_model: "WM-100".to_string(),
        manufacturer: "WeighTech".to_string(),
        serial_number: "SN123456".to_string(),
        communication_type: "4G".to_string(),
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

    let mock_repo = Arc::new(MockDeviceRepo { devices: vec![] });
    let use_cases = DeviceUseCases::new(mock_repo);
    let result: Result<Device, anyhow::Error> = use_cases.create_device(device_create).await;

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().to_string(), "设备ID不能为空");
}

#[tokio::test]
async fn test_create_device_empty_name() {
    let device_create = DeviceCreate {
        device_id: "DEV001".to_string(),
        device_name: "".to_string(),
        device_type: "称重设备".to_string(),
        device_model: "WM-100".to_string(),
        manufacturer: "WeighTech".to_string(),
        serial_number: "SN123456".to_string(),
        communication_type: "4G".to_string(),
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

    let mock_repo = Arc::new(MockDeviceRepo { devices: vec![] });
    let use_cases = DeviceUseCases::new(mock_repo);
    let result: Result<Device, anyhow::Error> = use_cases.create_device(device_create).await;

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().to_string(), "设备名称不能为空");
}
