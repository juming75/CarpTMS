//! Device Aggregate Root
use crate::domain::ddd::{AggregateRoot, DomainEvent, Entity, EntityId, EventSourcedAggregate};
use crate::errors::AppResult;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeviceStatus {
    Online,
    Offline,
    Maintenance,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceAggregate {
    pub id: DeviceId,
    pub device_name: String,
    pub device_type: String,
    pub status: DeviceStatus,
    pub sim_card_no: Option<String>,
    pub ip_address: Option<String>,
    pub version: u64,
    events: Vec<DomainEvent>,
}

impl DeviceAggregate {
    pub fn new(device_id: String, device_name: String, device_type: String) -> Self {
        let mut d = Self {
            id: DeviceId(device_id),
            device_name,
            device_type,
            status: DeviceStatus::Unknown,
            sim_card_no: None,
            ip_address: None,
            version: 0,
            events: Vec::new(),
        };
        d.raise_event("DeviceRegistered", serde_json::json!({"device_id": d.id.0}));
        d
    }
    pub fn go_online(&mut self, ip: Option<String>) -> AppResult<()> {
        self.status = DeviceStatus::Online;
        self.ip_address = ip;
        self.raise_event("DeviceOnline", serde_json::json!({"device_id": self.id.0}));
        Ok(())
    }
    pub fn go_offline(&mut self) -> AppResult<()> {
        self.status = DeviceStatus::Offline;
        self.raise_event("DeviceOffline", serde_json::json!({"device_id": self.id.0}));
        Ok(())
    }
    pub fn enter_maintenance(&mut self) -> AppResult<()> {
        self.status = DeviceStatus::Maintenance;
        self.raise_event(
            "DeviceMaintenance",
            serde_json::json!({"device_id": self.id.0}),
        );
        Ok(())
    }
    fn raise_event(&mut self, et: &str, data: serde_json::Value) {
        self.events.push(DomainEvent::new(
            "Device",
            &self.id.to_string(),
            et,
            data,
            self.version as i32 + 1,
        ));
        self.version += 1;
    }
    fn apply_event(&mut self, e: &DomainEvent) {
        match e.event_type.as_str() {
            "DeviceRegistered" | "DeviceOnline" => self.status = DeviceStatus::Online,
            "DeviceOffline" => self.status = DeviceStatus::Offline,
            "DeviceMaintenance" => self.status = DeviceStatus::Maintenance,
            _ => {}
        }
        self.version = e.version as u64;
        if let Some(v) = e.event_data.get("ip_address").and_then(|v| v.as_str()) {
            self.ip_address = Some(v.to_string());
        }
    }
}
impl Entity for DeviceAggregate {
    fn id(&self) -> &impl EntityId {
        &self.id
    }
}
impl AggregateRoot for DeviceAggregate {
    fn version(&self) -> u64 {
        self.version
    }
    fn events(&self) -> &[DomainEvent] {
        &self.events
    }
    fn clear_events(&mut self) {
        self.events.clear();
    }
}
impl EventSourcedAggregate for DeviceAggregate {
    fn rebuild_from_events(&mut self, events: &[DomainEvent]) -> AppResult<()> {
        for e in events {
            self.apply_event(e);
        }
        Ok(())
    }
    fn get_uncommitted_events(&self) -> &[DomainEvent] {
        &self.events
    }
    fn mark_events_committed(&mut self) {
        self.events.clear();
    }
}
