//! 设备仓库接口

use crate::domain::entities::device::{Device, DeviceCreate, DeviceQuery, DeviceUpdate};

/// 设备仓库接口
#[async_trait::async_trait]
pub trait DeviceRepository: Send + Sync {
    async fn get_devices(&self, query: DeviceQuery) -> Result<(Vec<Device>, i64), anyhow::Error>;
    async fn get_device(&self, device_id: &str) -> Result<Option<Device>, anyhow::Error>;
    async fn create_device(&self, device: DeviceCreate) -> Result<Device, anyhow::Error>;
    async fn update_device(
        &self,
        device_id: &str,
        device: DeviceUpdate,
    ) -> Result<Option<Device>, anyhow::Error>;
    async fn delete_device(&self, device_id: &str) -> Result<bool, anyhow::Error>;
}
