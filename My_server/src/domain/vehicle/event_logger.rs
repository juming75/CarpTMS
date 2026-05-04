// Vehicle 事件日志
// 轻量级车辆事件审计

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 车辆事件日志条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VehicleEventLogEntry {
    pub event_id: String,
    pub vehicle_id: i32,
    pub event_type: String,
    pub event_data: String,
    pub source: String,
    pub user_id: Option<i32>,
    pub occurred_at: DateTime<Utc>,
    pub recorded_at: DateTime<Utc>,
}

impl VehicleEventLogEntry {
    pub fn new(
        vehicle_id: i32,
        event_type: &str,
        event_data: &str,
        source: &str,
        user_id: Option<i32>,
    ) -> Self {
        let now = Utc::now();
        Self {
            event_id: format!("V_{}_{}", now.timestamp_millis(), vehicle_id),
            vehicle_id,
            event_type: event_type.to_string(),
            event_data: event_data.to_string(),
            source: source.to_string(),
            user_id,
            occurred_at: now,
            recorded_at: now,
        }
    }
}

/// 车辆事件日志记录器
pub struct VehicleEventLogger {
    logs: Vec<VehicleEventLogEntry>,
}

impl Default for VehicleEventLogger {
    fn default() -> Self {
        Self::new()
    }
}

impl VehicleEventLogger {
    pub fn new() -> Self {
        Self { logs: Vec::new() }
    }

    pub fn log(&mut self, entry: VehicleEventLogEntry) {
        self.logs.push(entry);
    }

    pub fn get_by_vehicle_id(&self, vehicle_id: i32) -> Vec<&VehicleEventLogEntry> {
        self.logs
            .iter()
            .filter(|e| e.vehicle_id == vehicle_id)
            .collect()
    }

    pub fn count(&self) -> usize {
        self.logs.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_log_entry_creation() {
        let entry =
            VehicleEventLogEntry::new(1, "Created", r#"{"vehicle_name": "测试车辆"}"#, "API", None);

        assert_eq!(entry.vehicle_id, 1);
        assert_eq!(entry.event_type, "Created");
    }

    #[test]
    fn test_event_logger() {
        let mut logger = VehicleEventLogger::new();

        logger.log(VehicleEventLogEntry::new(1, "Created", "{}", "API", None));
        logger.log(VehicleEventLogEntry::new(1, "Updated", "{}", "API", None));
        logger.log(VehicleEventLogEntry::new(2, "Created", "{}", "Batch", None));

        assert_eq!(logger.count(), 3);
        assert_eq!(logger.get_by_vehicle_id(1).len(), 2);
    }
}
