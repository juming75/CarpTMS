//! /! 领域事件定义
//!
//! 定义系统中的所有领域事件

use serde::{Deserialize, Serialize};
use super::Event;
use super::EventMetadata;

/// 车辆创建事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VehicleCreatedEvent {
    pub metadata: EventMetadata,
    pub vehicle_id: i32,
    pub plate_number: String,
    pub device_id: Option<String>,
}

impl Event for VehicleCreatedEvent {
    fn event_type() -> &'static str {
        "vehicle.created"
    }

    fn timestamp(&self) -> i64 {
        self.metadata.timestamp
    }

    fn source(&self) -> &str {
        &self.metadata.source
    }
}

/// 车辆位置更新事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VehicleLocationUpdatedEvent {
    pub metadata: EventMetadata,
    pub vehicle_id: i32,
    pub latitude: f64,
    pub longitude: f64,
    pub speed: f64,
    pub direction: f64,
    pub altitude: f64,
}

impl Event for VehicleLocationUpdatedEvent {
    fn event_type() -> &'static str {
        "vehicle.location_updated"
    }

    fn timestamp(&self) -> i64 {
        self.metadata.timestamp
    }

    fn source(&self) -> &str {
        &self.metadata.source
    }
}

/// 订单创建事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderCreatedEvent {
    pub metadata: EventMetadata,
    pub order_id: i32,
    pub vehicle_id: i32,
    pub order_number: String,
    pub status: String,
}

impl Event for OrderCreatedEvent {
    fn event_type() -> &'static str {
        "order.created"
    }

    fn timestamp(&self) -> i64 {
        self.metadata.timestamp
    }

    fn source(&self) -> &str {
        &self.metadata.source
    }
}

/// 订单状态变更事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderStatusChangedEvent {
    pub metadata: EventMetadata,
    pub order_id: i32,
    pub old_status: String,
    pub new_status: String,
}

impl Event for OrderStatusChangedEvent {
    fn event_type() -> &'static str {
        "order.status_changed"
    }

    fn timestamp(&self) -> i64 {
        self.metadata.timestamp
    }

    fn source(&self) -> &str {
        &self.metadata.source
    }
}

/// 设备上线事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceOnlineEvent {
    pub metadata: EventMetadata,
    pub device_id: String,
    pub device_type: String,
    pub ip_address: String,
}

impl Event for DeviceOnlineEvent {
    fn event_type() -> &'static str {
        "device.online"
    }

    fn timestamp(&self) -> i64 {
        self.metadata.timestamp
    }

    fn source(&self) -> &str {
        &self.metadata.source
    }
}

/// 设备离线事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceOfflineEvent {
    pub metadata: EventMetadata,
    pub device_id: String,
    pub device_type: String,
    pub reason: String,
}

impl Event for DeviceOfflineEvent {
    fn event_type() -> &'static str {
        "device.offline"
    }

    fn timestamp(&self) -> i64 {
        self.metadata.timestamp
    }

    fn source(&self) -> &str {
        &self.metadata.source
    }
}

/// 称重数据上报事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeighingDataReceivedEvent {
    pub metadata: EventMetadata,
    pub weighing_id: i64,
    pub vehicle_id: Option<i32>,
    pub weight: f64,
    pub unit: String,
}

impl Event for WeighingDataReceivedEvent {
    fn event_type() -> &'static str {
        "weighing.data_received"
    }

    fn timestamp(&self) -> i64 {
        self.metadata.timestamp
    }

    fn source(&self) -> &str {
        &self.metadata.source
    }
}

/// 警报事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertEvent {
    pub metadata: EventMetadata,
    pub alert_type: String,
    pub severity: String,
    pub message: String,
    pub entity_type: String,
    pub entity_id: String,
}

impl Event for AlertEvent {
    fn event_type() -> &'static str {
        "alert.triggered"
    }

    fn timestamp(&self) -> i64 {
        self.metadata.timestamp
    }

    fn source(&self) -> &str {
        &self.metadata.source
    }
}

/// 数据同步完成事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncCompletedEvent {
    pub metadata: EventMetadata,
    pub sync_type: String,
    pub records_synced: u64,
    pub duration_ms: u64,
}

impl Event for SyncCompletedEvent {
    fn event_type() -> &'static str {
        "sync.completed"
    }

    fn timestamp(&self) -> i64 {
        self.metadata.timestamp
    }

    fn source(&self) -> &str {
        &self.metadata.source
    }
}







