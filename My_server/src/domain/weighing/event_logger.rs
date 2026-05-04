// 称重事件日志（轻量级审计）
// 用于记录领域事件，支持日志分析和问题追踪
// 后续可升级为完整的事件溯源仓库

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 事件日志条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventLogEntry {
    /// 事件ID（雪花算法或UUID）
    pub event_id: String,
    /// 聚合根类型
    pub aggregate_type: String,
    /// 聚合根ID
    pub aggregate_id: i64,
    /// 事件类型
    pub event_type: String,
    /// 事件数据（JSON）
    pub event_data: String,
    /// 操作用户ID（可选）
    pub user_id: Option<i32>,
    /// 操作来源（API/gRPC/Batch）
    pub source: String,
    /// 事件时间
    pub occurred_at: DateTime<Utc>,
    /// 记录时间（写入时间）
    pub recorded_at: DateTime<Utc>,
}

impl EventLogEntry {
    /// 创建事件日志条目
    pub fn new(
        aggregate_type: &str,
        aggregate_id: i64,
        event_type: &str,
        event_data: &str,
        source: &str,
        user_id: Option<i32>,
    ) -> Self {
        let now = Utc::now();
        Self {
            event_id: format!("{}_{}", now.timestamp_millis(), aggregate_id),
            aggregate_type: aggregate_type.to_string(),
            aggregate_id,
            event_type: event_type.to_string(),
            event_data: event_data.to_string(),
            user_id,
            source: source.to_string(),
            occurred_at: now,
            recorded_at: now,
        }
    }
}

/// 事件日志记录器（轻量级实现）
pub struct EventLogger {
    /// 日志存储（当前为内存，后续可持久化）
    logs: Vec<EventLogEntry>,
}

impl Default for EventLogger {
    fn default() -> Self {
        Self::new()
    }
}

impl EventLogger {
    pub fn new() -> Self {
        Self { logs: Vec::new() }
    }

    /// 记录事件
    pub fn log(&mut self, entry: EventLogEntry) {
        self.logs.push(entry);
    }

    /// 根据聚合根ID查询事件
    pub fn get_by_aggregate_id(&self, aggregate_id: i64) -> Vec<&EventLogEntry> {
        self.logs
            .iter()
            .filter(|e| e.aggregate_id == aggregate_id)
            .collect()
    }

    /// 根据事件类型查询
    pub fn get_by_event_type(&self, event_type: &str) -> Vec<&EventLogEntry> {
        self.logs
            .iter()
            .filter(|e| e.event_type == event_type)
            .collect()
    }

    /// 获取最近的N条日志
    pub fn get_recent(&self, n: usize) -> Vec<&EventLogEntry> {
        self.logs.iter().rev().take(n).collect()
    }

    /// 获取日志总数
    pub fn count(&self) -> usize {
        self.logs.len()
    }

    /// 导出为可序列化的日志列表
    pub fn export(&self) -> Vec<EventLogEntry> {
        self.logs.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_log_entry_creation() {
        let entry = EventLogEntry::new(
            "Weighing",
            123,
            "Created",
            r#"{"weight": 5000}"#,
            "API",
            Some(1),
        );

        assert_eq!(entry.aggregate_type, "Weighing");
        assert_eq!(entry.aggregate_id, 123);
        assert_eq!(entry.event_type, "Created");
    }

    #[test]
    fn test_event_logger() {
        let mut logger = EventLogger::new();

        // 记录事件
        logger.log(EventLogEntry::new(
            "Weighing", 1, "Created", "{}", "API", None,
        ));
        logger.log(EventLogEntry::new(
            "Weighing", 1, "Updated", "{}", "API", None,
        ));
        logger.log(EventLogEntry::new(
            "Weighing", 2, "Created", "{}", "Batch", None,
        ));

        assert_eq!(logger.count(), 3);
        assert_eq!(logger.get_by_aggregate_id(1).len(), 2);
        assert_eq!(logger.get_by_event_type("Created").len(), 2);
        assert_eq!(logger.get_recent(2).len(), 2);
    }
}
