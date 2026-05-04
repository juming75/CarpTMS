//! 设备用例实现

use std::sync::Arc;

use crate::domain::entities::device::{Device, DeviceCreate, DeviceQuery, DeviceUpdate};
use crate::domain::use_cases::device::repository::DeviceRepository;

/// 设备用例结构
#[derive(Clone)]
pub struct DeviceUseCases {
    device_repository: Arc<dyn DeviceRepository + Send + Sync>,
}

impl DeviceUseCases {
    pub fn new(device_repository: Arc<dyn DeviceRepository>) -> Self {
        Self { device_repository }
    }

    /// 获取设备列表
    pub async fn get_devices(
        &self,
        query: DeviceQuery,
    ) -> Result<(Vec<Device>, i64), anyhow::Error> {
        self.device_repository.get_devices(query).await
    }

    /// 获取单个设备
    pub async fn get_device(&self, device_id: &str) -> Result<Option<Device>, anyhow::Error> {
        if device_id.is_empty() {
            return Err(anyhow::anyhow!("设备ID不能为空"));
        }
        self.device_repository.get_device(device_id).await
    }

    /// 创建设备
    pub async fn create_device(&self, device: DeviceCreate) -> Result<Device, anyhow::Error> {
        if device.device_id.is_empty() {
            return Err(anyhow::anyhow!("设备ID不能为空"));
        }
        if device.device_name.is_empty() {
            return Err(anyhow::anyhow!("设备名称不能为空"));
        }
        if device.device_type.is_empty() {
            return Err(anyhow::anyhow!("设备类型不能为空"));
        }
        if device.device_model.is_empty() {
            return Err(anyhow::anyhow!("设备型号不能为空"));
        }
        if device.manufacturer.is_empty() {
            return Err(anyhow::anyhow!("制造商不能为空"));
        }
        if device.serial_number.is_empty() {
            return Err(anyhow::anyhow!("序列号不能为空"));
        }
        if device.communication_type.is_empty() {
            return Err(anyhow::anyhow!("通信类型不能为空"));
        }
        self.device_repository.create_device(device).await
    }

    /// 更新设备
    pub async fn update_device(
        &self,
        device_id: &str,
        device: DeviceUpdate,
    ) -> Result<Option<Device>, anyhow::Error> {
        if device_id.is_empty() {
            return Err(anyhow::anyhow!("设备ID不能为空"));
        }
        self.device_repository
            .update_device(device_id, device)
            .await
    }

    /// 删除设备
    pub async fn delete_device(&self, device_id: &str) -> Result<bool, anyhow::Error> {
        if device_id.is_empty() {
            return Err(anyhow::anyhow!("设备ID不能为空"));
        }
        self.device_repository.delete_device(device_id).await
    }
}
