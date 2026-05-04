//! 统一领域事件日志服务（轻量级审计）

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::RwLock;

/// 事件严重级别
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum EventLevel {
    #[default]
    Info,
    Warning,
    Error,
    Critical,
}

/// 事件日志条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainEventLog {
    pub event_id: String,
    pub aggregate_type: String,
    pub aggregate_id: i64,
    pub event_type: String,
    pub event_data: String,
    pub level: EventLevel,
    pub user_id: Option<i32>,
    pub source: String,
    pub client_ip: Option<String>,
    pub trace_id: Option<String>,
    pub occurred_at: DateTime<Utc>,
    pub recorded_at: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}

impl DomainEventLog {
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
            level: EventLevel::Info,
            user_id,
            source: source.to_string(),
            client_ip: None,
            trace_id: None,
            occurred_at: now,
            recorded_at: now,
            metadata: HashMap::new(),
        }
    }

    pub fn with_level(
        aggregate_type: &str,
        aggregate_id: i64,
        event_type: &str,
        event_data: &str,
        level: EventLevel,
        source: &str,
        user_id: Option<i32>,
    ) -> Self {
        let mut log = Self::new(
            aggregate_type,
            aggregate_id,
            event_type,
            event_data,
            source,
            user_id,
        );
        log.level = level;
        log
    }

    pub fn with_trace(mut self, trace_id: &str, client_ip: &str) -> Self {
        self.trace_id = Some(trace_id.to_string());
        self.client_ip = Some(client_ip.to_string());
        self
    }

    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }
}

/// 事件查询条件
#[derive(Debug, Clone, Default)]
pub struct EventLogQuery {
    pub aggregate_type: Option<String>,
    pub aggregate_id: Option<i64>,
    pub event_type: Option<String>,
    pub level: Option<EventLevel>,
    pub user_id: Option<i32>,
    pub source: Option<String>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub limit: usize,
    pub offset: usize,
}

/// 统一事件日志记录器
pub struct DomainEventLogger {
    logs: RwLock<Vec<DomainEventLog>>,
    /// 模块统计
    stats: RwLock<HashMap<String, usize>>,
}

impl Default for DomainEventLogger {
    fn default() -> Self {
        Self::new()
    }
}

impl DomainEventLogger {
    pub fn new() -> Self {
        Self {
            logs: RwLock::new(Vec::new()),
            stats: RwLock::new(HashMap::new()),
        }
    }

    /// 记录事件
    pub fn log(&self, event: DomainEventLog) {
        let key = format!("{}:{}", event.aggregate_type, event.event_type);

        // 更新统计
        if let Ok(mut stats) = self.stats.write() {
            *stats.entry(key).or_insert(0) += 1;
        }

        // 记录日志
        if let Ok(mut logs) = self.logs.write() {
            logs.push(event);
        }
    }

    /// 查询事件
    pub fn query(&self, query: EventLogQuery) -> Vec<DomainEventLog> {
        if let Ok(logs) = self.logs.read() {
            logs.iter()
                .filter(|e| {
                    query
                        .aggregate_type
                        .as_ref()
                        .is_none_or(|t| &e.aggregate_type == t)
                        && query.aggregate_id.is_none_or(|id| e.aggregate_id == id)
                        && query.event_type.as_ref().is_none_or(|t| &e.event_type == t)
                        && query.level.is_none_or(|l| e.level == l)
                        && query.user_id.is_none_or(|uid| e.user_id == Some(uid))
                        && query.source.as_ref().is_none_or(|s| &e.source == s)
                        && query.start_time.is_none_or(|t| e.occurred_at >= t)
                        && query.end_time.is_none_or(|t| e.occurred_at <= t)
                })
                .skip(query.offset)
                .take(query.limit)
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }

    /// 根据聚合ID获取事件
    pub fn get_by_aggregate(&self, aggregate_type: &str, aggregate_id: i64) -> Vec<DomainEventLog> {
        self.query(EventLogQuery {
            aggregate_type: Some(aggregate_type.to_string()),
            aggregate_id: Some(aggregate_id),
            limit: 1000,
            ..Default::default()
        })
    }

    /// 获取最近事件
    pub fn get_recent(&self, limit: usize) -> Vec<DomainEventLog> {
        self.query(EventLogQuery {
            limit,
            ..Default::default()
        })
    }

    /// 获取统计数据
    pub fn get_stats(&self) -> HashMap<String, usize> {
        self.stats.read().map(|s| s.clone()).unwrap_or_default()
    }

    /// 获取总数
    pub fn count(&self) -> usize {
        self.logs.read().map(|l| l.len()).unwrap_or(0)
    }

    /// 导出所有日志
    pub fn export(&self) -> Vec<DomainEventLog> {
        self.logs.read().map(|l| l.clone()).unwrap_or_default()
    }

    /// 清理旧日志（保留最近N条）
    pub fn prune(&self, keep_recent: usize) {
        if let Ok(mut logs) = self.logs.write() {
            if logs.len() > keep_recent {
                let drain_count = logs.len() - keep_recent;
                logs.drain(0..drain_count);
            }
        }
    }
}

use std::sync::LazyLock;

/// 全局事件日志实例
static LOGGER: LazyLock<DomainEventLogger> = LazyLock::new(DomainEventLogger::new);

pub fn global_event_logger() -> &'static DomainEventLogger {
    &LOGGER
}

/// 便捷宏：记录领域事件
#[macro_export]
macro_rules! log_domain_event {
    ($aggregate_type:expr, $aggregate_id:expr, $event_type:expr, $event_data:expr, $source:expr) => {
        $crate::domain::event_logger::DomainEventLogger::new().log(
            $crate::domain::event_logger::DomainEventLog::new(
                $aggregate_type,
                $aggregate_id,
                $event_type,
                &$event_data,
                $source,
                None,
            ),
        )
    };
    ($aggregate_type:expr, $aggregate_id:expr, $event_type:expr, $event_data:expr, $source:expr, $user_id:expr) => {
        $crate::domain::event_logger::DomainEventLogger::new().log(
            $crate::domain::event_logger::DomainEventLog::new(
                $aggregate_type,
                $aggregate_id,
                $event_type,
                &$event_data,
                $source,
                Some($user_id),
            ),
        )
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_creation() {
        let event = DomainEventLog::new(
            "Weighing",
            123,
            "Created",
            r#"{"weight": 5000}"#,
            "API",
            Some(1),
        );

        assert_eq!(event.aggregate_type, "Weighing");
        assert_eq!(event.aggregate_id, 123);
        assert_eq!(event.event_type, "Created");
        assert_eq!(event.level, EventLevel::Info);
    }

    #[test]
    fn test_event_with_level() {
        let event = DomainEventLog::with_level(
            "Weighing",
            123,
            "AbnormalDetected",
            r#"{"type": "overload"}"#,
            EventLevel::Warning,
            "API",
            None,
        );

        assert_eq!(event.level, EventLevel::Warning);
    }

    #[test]
    fn test_logger() {
        let logger = DomainEventLogger::new();

        logger.log(DomainEventLog::new(
            "Weighing", 1, "Created", "{}", "API", None,
        ));
        logger.log(DomainEventLog::new(
            "Weighing", 1, "Updated", "{}", "API", None,
        ));

        assert_eq!(logger.count(), 2);
        assert_eq!(logger.get_by_aggregate("Weighing", 1).len(), 2);
        assert_eq!(logger.get_recent(10).len(), 2);
    }

    #[test]
    fn test_query() {
        let logger = DomainEventLogger::new();

        logger.log(DomainEventLog::new(
            "Weighing", 1, "Created", "{}", "API", None,
        ));
        logger.log(DomainEventLog::with_level(
            "Weighing",
            2,
            "AbnormalDetected",
            "{}",
            EventLevel::Warning,
            "API",
            None,
        ));

        let query = EventLogQuery {
            event_type: Some("Created".to_string()),
            limit: 10,
            ..Default::default()
        };

        assert_eq!(logger.query(query).len(), 1);
    }
}
