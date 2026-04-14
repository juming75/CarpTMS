//! / 设备应用服务

use std::sync::Arc;


use crate::domain::entities::device::{Device, DeviceCreate, DeviceQuery, DeviceUpdate};
use crate::domain::use_cases::device::DeviceUseCases;

/// 设备服务接口
#[async_trait::async_trait]
pub trait DeviceService: Send + Sync {
    /// 获取设备列表
    async fn get_devices(&self, query: DeviceQuery) -> Result<(Vec<Device>, i64), anyhow::Error>;

    /// 获取单个设备
    async fn get_device(&self, device_id: &str) -> Result<Option<Device>, anyhow::Error>;

    /// 创建设备
    async fn create_device(&self, device: DeviceCreate) -> Result<Device, anyhow::Error>;

    /// 更新设备
    async fn update_device(
        &self,
        device_id: &str,
        device: DeviceUpdate,
    ) -> Result<Option<Device>, anyhow::Error>;

    /// 删除设备
    async fn delete_device(&self, device_id: &str) -> Result<bool, anyhow::Error>;
}

/// 设备服务实现
#[derive(Clone)]
pub struct DeviceServiceImpl {
    device_use_cases: Arc<DeviceUseCases>,
}

impl DeviceServiceImpl {
    /// 创建设备服务实例
    pub fn new(device_use_cases: Arc<DeviceUseCases>) -> Self {
        Self { device_use_cases }
    }
}

#[async_trait::async_trait]
impl DeviceService for DeviceServiceImpl {
    /// 获取设备列表
    async fn get_devices(&self, query: DeviceQuery) -> Result<(Vec<Device>, i64), anyhow::Error> {
        self.device_use_cases.get_devices(query).await
    }

    /// 获取单个设备
    async fn get_device(&self, device_id: &str) -> Result<Option<Device>, anyhow::Error> {
        self.device_use_cases.get_device(device_id).await
    }

    /// 创建设备
    async fn create_device(&self, device: DeviceCreate) -> Result<Device, anyhow::Error> {
        self.device_use_cases.create_device(device).await
    }

    /// 更新设备
    async fn update_device(
        &self,
        device_id: &str,
        device: DeviceUpdate,
    ) -> Result<Option<Device>, anyhow::Error> {
        self.device_use_cases.update_device(device_id, device).await
    }

    /// 删除设备
    async fn delete_device(&self, device_id: &str) -> Result<bool, anyhow::Error> {
        self.device_use_cases.delete_device(device_id).await
    }
}
