// VehicleGroup 事件日志
// 轻量级车组事件审计

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 车组事件日志条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VehicleGroupEventLogEntry {
    pub event_id: String,
    pub group_id: i32,
    pub event_type: String,
    pub event_data: String,
    pub source: String,
    pub user_id: Option<i32>,
    pub occurred_at: DateTime<Utc>,
    pub recorded_at: DateTime<Utc>,
}

impl VehicleGroupEventLogEntry {
    pub fn new(
        group_id: i32,
        event_type: &str,
        event_data: &str,
        source: &str,
        user_id: Option<i32>,
    ) -> Self {
        let now = Utc::now();
        Self {
            event_id: format!("G_{}_{}", now.timestamp_millis(), group_id),
            group_id,
            event_type: event_type.to_string(),
            event_data: event_data.to_string(),
            source: source.to_string(),
            user_id,
            occurred_at: now,
            recorded_at: now,
        }
    }
}

/// 车组事件日志记录器
pub struct VehicleGroupEventLogger {
    logs: Vec<VehicleGroupEventLogEntry>,
}

impl Default for VehicleGroupEventLogger {
    fn default() -> Self {
        Self::new()
    }
}

impl VehicleGroupEventLogger {
    pub fn new() -> Self {
        Self { logs: Vec::new() }
    }

    pub fn log(&mut self, entry: VehicleGroupEventLogEntry) {
        self.logs.push(entry);
    }

    pub fn get_by_group_id(&self, group_id: i32) -> Vec<&VehicleGroupEventLogEntry> {
        self.logs
            .iter()
            .filter(|e| e.group_id == group_id)
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
        let entry = VehicleGroupEventLogEntry::new(
            1,
            "Created",
            r#"{"group_name": "测试车组"}"#,
            "API",
            None,
        );

        assert_eq!(entry.group_id, 1);
        assert_eq!(entry.event_type, "Created");
    }

    #[test]
    fn test_event_logger() {
        let mut logger = VehicleGroupEventLogger::new();

        logger.log(VehicleGroupEventLogEntry::new(
            1, "Created", "{}", "API", None,
        ));
        logger.log(VehicleGroupEventLogEntry::new(
            1, "Updated", "{}", "API", None,
        ));

        assert_eq!(logger.count(), 2);
        assert_eq!(logger.get_by_group_id(1).len(), 2);
    }
}
