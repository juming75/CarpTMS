//! /! 车辆聚合根实现
//!
//! DDD示例:车辆聚合根及其相关实体和值对象

use super::ddd::{
    AggregateRoot, DomainEvent, Entity, EntityId, EventSourcedAggregate, Specification, ValueObject,
};
use crate::errors::AppResult;
use serde::{Deserialize, Serialize};

/// 车辆ID
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct VehicleId(pub i32);

impl EntityId for VehicleId {
    fn type_name(&self) -> &'static str {
        "VehicleId"
    }
}

impl std::fmt::Display for VehicleId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// 车牌号值对象
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlateNumber(pub String);

impl PlateNumber {
    pub fn new(plate: String) -> AppResult<Self> {
        if plate.is_empty() {
            return Err(crate::errors::AppError::validation(
                "Plate number cannot be empty",
            ));
        }

        // 简单的车牌号验证(根据实际需求调整)
        if plate.len() < 7 || plate.len() > 10 {
            return Err(crate::errors::AppError::validation(
                "Invalid plate number format",
            ));
        }

        Ok(PlateNumber(plate))
    }
}

impl ValueObject for PlateNumber {}

/// GPS位置值对象
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GpsLocation {
    pub latitude: f64,
    pub longitude: f64,
    pub altitude: Option<f64>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl GpsLocation {
    pub fn new(latitude: f64, longitude: f64) -> AppResult<Self> {
        if !(-90.0..=90.0).contains(&latitude) {
            return Err(crate::errors::AppError::validation("Invalid latitude"));
        }
        if !(-180.0..=180.0).contains(&longitude) {
            return Err(crate::errors::AppError::validation("Invalid longitude"));
        }

        Ok(Self {
            latitude,
            longitude,
            altitude: None,
            timestamp: chrono::Utc::now(),
        })
    }
}

// Note: GpsLocation does not implement ValueObject due to f64 fields

/// 设备实体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    #[serde(skip, default)]
    device_id: DeviceId,
    pub id: String,
    pub device_type: String,
    pub phone_number: String,
    pub status: DeviceStatus,
}

impl Device {
    pub fn new(id: String, device_type: String, phone_number: String) -> Self {
        Self {
            device_id: DeviceId(id.clone()),
            id,
            device_type,
            phone_number,
            status: DeviceStatus::Offline,
        }
    }
}

impl Entity for Device {
    fn id(&self) -> &impl EntityId {
        &self.device_id
    }
}

/// 设备ID
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct DeviceId(pub String);

impl EntityId for DeviceId {
    fn type_name(&self) -> &'static str {
        "DeviceId"
    }
}

impl std::fmt::Display for DeviceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// 设备状态
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeviceStatus {
    Online,
    Offline,
    Unknown,
}

/// 车辆聚合根
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vehicle {
    pub id: VehicleId,
    pub plate_number: PlateNumber,
    pub device: Option<Device>,
    pub current_location: Option<GpsLocation>,
    pub status: VehicleStatus,
    pub version: u64,
    events: Vec<DomainEvent>,
}

/// 车辆状态
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VehicleStatus {
    Active,
    Inactive,
    Maintenance,
    Unknown,
}

impl From<i16> for VehicleStatus {
    fn from(value: i16) -> Self {
        match value {
            1 => VehicleStatus::Active,
            2 => VehicleStatus::Inactive,
            3 => VehicleStatus::Maintenance,
            _ => VehicleStatus::Unknown,
        }
    }
}

impl From<i32> for VehicleStatus {
    fn from(value: i32) -> Self {
        match value {
            1 => VehicleStatus::Active,
            2 => VehicleStatus::Inactive,
            3 => VehicleStatus::Maintenance,
            _ => VehicleStatus::Unknown,
        }
    }
}

impl Vehicle {
    /// 创建新车辆
    pub fn new(plate_number: PlateNumber, device: Option<Device>) -> Self {
        let vehicle_id = VehicleId(0); // 由仓储生成

        let mut vehicle = Self {
            id: vehicle_id,
            plate_number,
            device,
            current_location: None,
            status: VehicleStatus::Active,
            version: 0,
            events: Vec::new(),
        };

        // 发布车辆创建事件
        vehicle.events.push(DomainEvent::new(
            "Vehicle",
            &vehicle.id.to_string(),
            "VehicleCreated",
            serde_json::json!({
                "plate_number": vehicle.plate_number.0,
                "device": vehicle.device.as_ref().map(|d| &d.id),
            }),
            vehicle.version as i32,
        ));

        vehicle
    }

    /// 更新GPS位置
    pub fn update_location(&mut self, location: GpsLocation) -> AppResult<()> {
        let lat = location.latitude;
        let lng = location.longitude;
        self.current_location = Some(location);

        self.events.push(DomainEvent::new(
            "Vehicle",
            &self.id.to_string(),
            "VehicleLocationUpdated",
            serde_json::json!({
                "latitude": lat,
                "longitude": lng,
            }),
            self.version as i32,
        ));

        Ok(())
    }

    /// 绑定设备
    pub fn bind_device(&mut self, device: Device) -> AppResult<()> {
        self.device = Some(device);

        self.events.push(DomainEvent::new(
            "Vehicle",
            &self.id.to_string(),
            "DeviceBound",
            serde_json::json!({
                "device_id": self.device.as_ref().map(|d| &d.id),
            }),
            self.version as i32,
        ));

        Ok(())
    }

    /// 解绑设备
    pub fn unbind_device(&mut self) -> AppResult<()> {
        let device_id = self.device.as_ref().map(|d| d.id.clone());
        self.device = None;

        self.events.push(DomainEvent::new(
            "Vehicle",
            &self.id.to_string(),
            "DeviceUnbound",
            serde_json::json!({
                "device_id": device_id,
            }),
            self.version as i32,
        ));

        Ok(())
    }

    /// 更改状态
    pub fn change_status(&mut self, new_status: VehicleStatus) -> AppResult<()> {
        if self.status == new_status {
            return Err(crate::errors::AppError::validation(
                "Vehicle already in this status",
            ));
        }

        let old_status = self.status.clone();
        let status_str = format!("{:?}", new_status);
        self.status = new_status;

        self.events.push(DomainEvent::new(
            "Vehicle",
            &self.id.to_string(),
            "VehicleStatusChanged",
            serde_json::json!({
                "old_status": format!("{:?}", old_status),
                "new_status": status_str,
            }),
            self.version as i32,
        ));

        Ok(())
    }
}

impl Entity for Vehicle {
    fn id(&self) -> &impl EntityId {
        &self.id
    }
}

impl AggregateRoot for Vehicle {
    fn version(&self) -> u64 {
        self.version
    }

    fn events(&self) -> &[DomainEvent] {
        &self.events
    }

    fn clear_events(&mut self) {
        self.events.clear();
        self.version += 1;
    }
}

impl EventSourcedAggregate for Vehicle {
    fn rebuild_from_events(&mut self, events: &[DomainEvent]) -> AppResult<()> {
        for event in events {
            match event.event_type.as_str() {
                "VehicleCreated" => {
                    // 从VehicleCreated事件重建
                    if let Some(plate_number) = event
                        .event_data
                        .get("plate_number")
                        .and_then(|v| v.as_str())
                    {
                        self.plate_number = PlateNumber(plate_number.to_string());
                    }
                    if let Some(device_id) = event.event_data.get("device").and_then(|v| v.as_str())
                    {
                        // 这里简化处理,实际应该从设备仓储获取完整设备信息
                        self.device = Some(Device::new(
                            device_id.to_string(),
                            "unknown".to_string(),
                            "".to_string(),
                        ));
                    }
                    self.status = VehicleStatus::Active;
                }
                "VehicleLocationUpdated" => {
                    // 从VehicleLocationUpdated事件重建
                    if let (Some(lat), Some(lng)) = (
                        event.event_data.get("latitude").and_then(|v| v.as_f64()),
                        event.event_data.get("longitude").and_then(|v| v.as_f64()),
                    ) {
                        self.current_location = Some(GpsLocation::new(lat, lng)?);
                    }
                }
                "DeviceBound" => {
                    // 从DeviceBound事件重建
                    if let Some(device_id) =
                        event.event_data.get("device_id").and_then(|v| v.as_str())
                    {
                        // 这里简化处理,实际应该从设备仓储获取完整设备信息
                        self.device = Some(Device::new(
                            device_id.to_string(),
                            "unknown".to_string(),
                            "".to_string(),
                        ));
                    }
                }
                "DeviceUnbound" => {
                    // 从DeviceUnbound事件重建
                    self.device = None;
                }
                "VehicleStatusChanged" => {
                    // 从VehicleStatusChanged事件重建
                    if let Some(new_status) =
                        event.event_data.get("new_status").and_then(|v| v.as_str())
                    {
                        match new_status {
                            "Active" => self.status = VehicleStatus::Active,
                            "Inactive" => self.status = VehicleStatus::Inactive,
                            "Maintenance" => self.status = VehicleStatus::Maintenance,
                            _ => {}
                        }
                    }
                }
                _ => {
                    // 忽略未知事件类型
                }
            }
        }
        self.version = events.len() as u64;
        Ok(())
    }

    fn get_uncommitted_events(&self) -> &[DomainEvent] {
        &self.events
    }

    fn mark_events_committed(&mut self) {
        self.events.clear();
        self.version += 1;
    }
}

/// 车辆规范:在线车辆
pub struct OnlineVehicleSpecification;

impl Specification<Vehicle> for OnlineVehicleSpecification {
    fn is_satisfied_by(&self, candidate: &Vehicle) -> bool {
        candidate.status == VehicleStatus::Active && candidate.current_location.is_some()
    }
}

/// 车辆规范:有设备的车辆
pub struct VehicleWithDeviceSpecification;

impl Specification<Vehicle> for VehicleWithDeviceSpecification {
    fn is_satisfied_by(&self, candidate: &Vehicle) -> bool {
        candidate.device.is_some()
    }
}

/// 车辆规范:维护中的车辆
pub struct MaintenanceVehicleSpecification;

impl Specification<Vehicle> for MaintenanceVehicleSpecification {
    fn is_satisfied_by(&self, candidate: &Vehicle) -> bool {
        candidate.status == VehicleStatus::Maintenance
    }
}

/// 车辆仓储
#[async_trait::async_trait]
pub trait VehicleRepository: Send + Sync {
    async fn find_by_id(&self, id: &VehicleId) -> AppResult<Option<Vehicle>>;
    async fn save(&self, vehicle: &mut Vehicle) -> AppResult<()>;
    async fn find_by_plate(&self, plate: &PlateNumber) -> AppResult<Option<Vehicle>>;
    async fn find_active_vehicles(&self) -> AppResult<Vec<Vehicle>>;
}

/// 车辆应用服务
pub struct VehicleApplicationService<R>
where
    R: VehicleRepository,
{
    repository: R,
}

impl<R> VehicleApplicationService<R>
where
    R: VehicleRepository,
{
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    /// 创建车辆
    pub async fn create_vehicle(
        &self,
        plate_number: String,
        device: Option<Device>,
    ) -> AppResult<Vehicle> {
        let plate = PlateNumber::new(plate_number)?;
        let mut vehicle = Vehicle::new(plate, device);

        self.repository.save(&mut vehicle).await?;

        Ok(vehicle)
    }

    /// 更新车辆位置
    pub async fn update_location(
        &self,
        vehicle_id: i32,
        latitude: f64,
        longitude: f64,
    ) -> AppResult<()> {
        let vehicle_id = VehicleId(vehicle_id);

        let mut vehicle = self
            .repository
            .find_by_id(&vehicle_id)
            .await?
            .ok_or_else(|| {
                crate::errors::AppError::not_found_error(format!("Vehicle {}", vehicle_id))
            })?;

        let location = GpsLocation::new(latitude, longitude)?;
        vehicle.update_location(location)?;

        self.repository.save(&mut vehicle).await?;

        Ok(())
    }
}
