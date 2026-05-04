//! 增强审计日志模块
//! 支持数据库存储、日志分析API、实时告警

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use tokio::sync::mpsc;

/// 审计日志级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "VARCHAR")]
#[sqlx(rename_all = "lowercase")]
pub enum AuditLevel {
    Debug,
    Info,
    Warning,
    Error,
    Critical,
}

impl std::fmt::Display for AuditLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuditLevel::Debug => write!(f, "debug"),
            AuditLevel::Info => write!(f, "info"),
            AuditLevel::Warning => write!(f, "warning"),
            AuditLevel::Error => write!(f, "error"),
            AuditLevel::Critical => write!(f, "critical"),
        }
    }
}

/// 审计事件类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "VARCHAR")]
pub enum AuditEventType {
    Login,
    Logout,
    PasswordChange,
    PasswordExpired,
    PermissionChange,
    PermissionDenied,
    DataAccess,
    DataModification,
    DataDeletion,
    DataExport,
    AdminAction,
    SecurityAlert,
    SystemError,
    FailedAuth,
}

/// 数据库审计日志条目
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AuditLogEntry {
    pub id: i64,
    pub timestamp: DateTime<Utc>,
    pub level: String,
    pub event_type: String,
    pub message: String,
    pub user_id: Option<i32>,
    pub username: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub resource_type: Option<String>,
    pub resource_id: Option<String>,
    pub action: Option<String>,
    pub details: Option<serde_json::Value>,
    pub status: String,
    pub error_message: Option<String>,
    pub duration_ms: Option<i64>,
}

/// 审计日志查询参数
#[derive(Debug, Clone, Deserialize)]
pub struct AuditLogQuery {
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub level: Option<String>,
    pub event_type: Option<String>,
    pub user_id: Option<i32>,
    pub username: Option<String>,
    pub ip_address: Option<String>,
    pub resource_type: Option<String>,
    pub status: Option<String>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

/// 审计日志查询结果
#[derive(Debug, Clone, Serialize)]
pub struct AuditLogResponse {
    pub items: Vec<AuditLogEntry>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
    pub total_pages: i64,
}

/// 审计统计信息
#[derive(Debug, Clone, Serialize, FromRow)]
pub struct AuditStatistics {
    pub total_count: i64,
    pub by_level: serde_json::Value,
    pub by_event_type: serde_json::Value,
    pub by_user: serde_json::Value,
    pub by_hour: serde_json::Value,
    pub failed_count: i64,
    pub critical_count: i64,
}

/// 实时告警配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertConfig {
    pub enabled: bool,
    pub channels: Vec<AlertChannel>,
    pub rules: Vec<AlertRule>,
}

/// 告警渠道
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertChannel {
    Email(Vec<String>),
    Webhook(String),
    Log,
}

/// 告警规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRule {
    pub name: String,
    pub event_type: Option<AuditEventType>,
    pub level: Option<AuditLevel>,
    pub threshold: i32,
    pub window_seconds: i64,
    pub enabled: bool,
}

/// 增强的审计日志服务
#[derive(Clone)]
pub struct EnhancedAuditLogger {
    pool: PgPool,
    alert_sender: Option<mpsc::Sender<AuditAlert>>,
    config: AuditConfig,
}

/// 审计告警
#[derive(Debug, Clone)]
pub struct AuditAlert {
    pub rule_name: String,
    pub event: AuditLogEntry,
    pub message: String,
}

/// 审计配置
#[derive(Debug, Clone)]
pub struct AuditConfig {
    pub retention_days: i32,
    pub enable_db_storage: bool,
    pub enable_realtime_alerts: bool,
    pub alert_config: AlertConfig,
}

impl Default for AuditConfig {
    fn default() -> Self {
        Self {
            retention_days: 180,
            enable_db_storage: true,
            enable_realtime_alerts: true,
            alert_config: AlertConfig {
                enabled: true,
                channels: vec![AlertChannel::Log],
                rules: vec![
                    AlertRule {
                        name: "连续登录失败".to_string(),
                        event_type: Some(AuditEventType::FailedAuth),
                        level: None,
                        threshold: 5,
                        window_seconds: 300,
                        enabled: true,
                    },
                    AlertRule {
                        name: "权限变更".to_string(),
                        event_type: Some(AuditEventType::PermissionChange),
                        level: None,
                        threshold: 1,
                        window_seconds: 60,
                        enabled: true,
                    },
                    AlertRule {
                        name: "敏感数据访问".to_string(),
                        event_type: Some(AuditEventType::DataAccess),
                        level: Some(AuditLevel::Warning),
                        threshold: 50,
                        window_seconds: 60,
                        enabled: true,
                    },
                    AlertRule {
                        name: "安全告警".to_string(),
                        event_type: Some(AuditEventType::SecurityAlert),
                        level: Some(AuditLevel::Critical),
                        threshold: 1,
                        window_seconds: 1,
                        enabled: true,
                    },
                ],
            },
        }
    }
}

impl EnhancedAuditLogger {
    /// 创建新的审计日志记录器
    pub async fn new(pool: PgPool, config: AuditConfig) -> Self {
        let alert_sender = if config.enable_realtime_alerts {
            let (tx, _rx) = mpsc::channel(100);
            Some(tx)
        } else {
            None
        };

        Self {
            pool,
            alert_sender,
            config,
        }
    }

    /// 记录审计日志到数据库
    pub async fn log(&self, entry: AuditLogEntry) -> Result<i64, sqlx::Error> {
        if !self.config.enable_db_storage {
            return Ok(0);
        }

        let result = sqlx::query_scalar::<_, i64>(
            r#"
            INSERT INTO audit_logs (
                timestamp, level, event_type, message,
                user_id, username, ip_address, user_agent,
                resource_type, resource_id, action, details,
                status, error_message, duration_ms
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            RETURNING id
            "#,
        )
        .bind(entry.timestamp)
        .bind(&entry.level)
        .bind(&entry.event_type)
        .bind(&entry.message)
        .bind(entry.user_id)
        .bind(&entry.username)
        .bind(&entry.ip_address)
        .bind(&entry.user_agent)
        .bind(&entry.resource_type)
        .bind(&entry.resource_id)
        .bind(&entry.action)
        .bind(&entry.details)
        .bind(&entry.status)
        .bind(&entry.error_message)
        .bind(entry.duration_ms)
        .fetch_one(&self.pool)
        .await?;

        // 检查是否触发告警
        self.check_and_send_alert(&entry).await;

        Ok(result)
    }

    /// 查询审计日志
    pub async fn query(&self, params: AuditLogQuery) -> Result<AuditLogResponse, sqlx::Error> {
        let page = params.page.unwrap_or(1).max(1);
        let page_size = params.page_size.unwrap_or(20).min(100);
        let offset = (page - 1) * page_size;

        let mut conditions = Vec::new();
        let mut bind_idx = 1;

        if params.start_date.is_some() {
            conditions.push(format!("timestamp >= ${}", bind_idx));
            bind_idx += 1;
        }
        if params.end_date.is_some() {
            conditions.push(format!("timestamp <= ${}", bind_idx));
            bind_idx += 1;
        }
        if params.level.is_some() {
            conditions.push(format!("level = ${}", bind_idx));
            bind_idx += 1;
        }
        if params.event_type.is_some() {
            conditions.push(format!("event_type = ${}", bind_idx));
            bind_idx += 1;
        }
        if params.user_id.is_some() {
            conditions.push(format!("user_id = ${}", bind_idx));
            bind_idx += 1;
        }
        if params.username.is_some() {
            conditions.push(format!("username ILIKE ${}", bind_idx));
            bind_idx += 1;
        }
        if params.ip_address.is_some() {
            conditions.push(format!("ip_address = ${}", bind_idx));
            bind_idx += 1;
        }
        if params.resource_type.is_some() {
            conditions.push(format!("resource_type = ${}", bind_idx));
            bind_idx += 1;
        }
        if params.status.is_some() {
            conditions.push(format!("status = ${}", bind_idx));
            bind_idx += 1;
        }

        let where_clause = if conditions.is_empty() {
            "".to_string()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        };

        // 构建查询
        let items_sql = format!(
            r#"
            SELECT * FROM audit_logs
            {}
            ORDER BY timestamp DESC
            LIMIT ${} OFFSET ${}
            "#,
            where_clause, bind_idx, bind_idx + 1
        );

        let count_sql = format!(
            "SELECT COUNT(*) FROM audit_logs {}",
            where_clause
        );

        let mut query_builder = sqlx::query_as::<_, AuditLogEntry>(&items_sql);
        let mut count_builder = sqlx::query_scalar::<_, i64>(&count_sql);

        if let Some(start_date) = params.start_date {
            query_builder = query_builder.bind(start_date);
            count_builder = count_builder.bind(start_date);
        }
        if let Some(end_date) = params.end_date {
            query_builder = query_builder.bind(end_date);
            count_builder = count_builder.bind(end_date);
        }
        if let Some(ref level) = params.level {
            query_builder = query_builder.bind(level);
            count_builder = count_builder.bind(level);
        }
        if let Some(ref event_type) = params.event_type {
            query_builder = query_builder.bind(event_type);
            count_builder = count_builder.bind(event_type);
        }
        if let Some(user_id) = params.user_id {
            query_builder = query_builder.bind(user_id);
            count_builder = count_builder.bind(user_id);
        }
        if let Some(ref username) = params.username {
            query_builder = query_builder.bind(format!("%{}%", username));
            count_builder = count_builder.bind(format!("%{}%", username));
        }
        if let Some(ref ip_address) = params.ip_address {
            query_builder = query_builder.bind(ip_address);
            count_builder = count_builder.bind(ip_address);
        }
        if let Some(ref resource_type) = params.resource_type {
            query_builder = query_builder.bind(resource_type);
            count_builder = count_builder.bind(resource_type);
        }
        if let Some(ref status) = params.status {
            query_builder = query_builder.bind(status);
            count_builder = count_builder.bind(status);
        }

        query_builder = query_builder.bind(page_size).bind(offset);

        let items = query_builder.fetch_all(&self.pool).await?;
        let total = count_builder.fetch_one(&self.pool).await?;

        Ok(AuditLogResponse {
            total_pages: (total as f64 / page_size as f64).ceil() as i64,
            page,
            page_size,
            total,
            items,
        })
    }

    /// 获取审计统计信息
    pub async fn get_statistics(
        &self,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> Result<AuditStatistics, sqlx::Error> {
        let stats = sqlx::query_as::<_, (i64, serde_json::Value, serde_json::Value, serde_json::Value, serde_json::Value, i64, i64)>(
            r#"
            SELECT
                COUNT(*) as total_count,
                COALESCE(jsonb_object_agg(level, count), '{}') as by_level,
                COALESCE(jsonb_object_agg(event_type, count), '{}') as by_event_type,
                COALESCE(jsonb_object_agg(username, count), '{}') as by_user,
                COALESCE(jsonb_object_agg(hour, count), '{}') as by_hour,
                COUNT(*) FILTER (WHERE status = 'failed') as failed_count,
                COUNT(*) FILTER (WHERE level = 'critical') as critical_count
            FROM (
                SELECT
                    level,
                    event_type,
                    username,
                    DATE_TRUNC('hour', timestamp) as hour,
                    status
                FROM audit_logs
                WHERE timestamp BETWEEN $1 AND $2
            ) sub
            "#,
        )
        .bind(start_date)
        .bind(end_date)
        .fetch_one(&self.pool)
        .await?;

        Ok(AuditStatistics {
            total_count: stats.0,
            by_level: stats.1,
            by_event_type: stats.2,
            by_user: stats.3,
            by_hour: stats.4,
            failed_count: stats.5,
            critical_count: stats.6,
        })
    }

    /// 获取可疑活动报告
    pub async fn get_suspicious_activity_report(
        &self,
        hours: i64,
    ) -> Result<Vec<AuditLogEntry>, sqlx::Error> {
        let entries = sqlx::query_as::<_, AuditLogEntry>(
            r#"
            SELECT * FROM audit_logs
            WHERE timestamp >= NOW() - INTERVAL '1 hour' * $1
            AND (
                status = 'failed'
                OR level IN ('error', 'critical')
                OR event_type IN ('permission_denied', 'security_alert', 'data_deletion')
            )
            ORDER BY timestamp DESC
            LIMIT 100
            "#,
        )
        .bind(hours)
        .fetch_all(&self.pool)
        .await?;

        Ok(entries)
    }

    /// 获取用户活动报告
    pub async fn get_user_activity_report(
        &self,
        user_id: i32,
        days: i64,
    ) -> Result<Vec<AuditLogEntry>, sqlx::Error> {
        let entries = sqlx::query_as::<_, AuditLogEntry>(
            r#"
            SELECT * FROM audit_logs
            WHERE user_id = $1
            AND timestamp >= NOW() - INTERVAL '1 day' * $2
            ORDER BY timestamp DESC
            LIMIT 500
            "#,
        )
        .bind(user_id)
        .bind(days)
        .fetch_all(&self.pool)
        .await?;

        Ok(entries)
    }

    /// 检查并发送告警
    async fn check_and_send_alert(&self, entry: &AuditLogEntry) {
        if !self.config.alert_config.enabled {
            return;
        }

        for rule in &self.config.alert_config.rules {
            if !rule.enabled {
                continue;
            }

            let should_alert = match (&rule.event_type, &rule.level) {
                (Some(event_type), None) => entry.event_type == format!("{:?}", event_type),
                (None, Some(level)) => entry.level == level.to_string(),
                (Some(event_type), Some(level)) => {
                    entry.event_type == format!("{:?}", event_type)
                        && entry.level == level.to_string()
                }
                (None, None) => false,
            };

            if should_alert {
                let alert = AuditAlert {
                    rule_name: rule.name.clone(),
                    event: entry.clone(),
                    message: format!(
                        "告警触发: {} - 用户 {} 从 {} 访问 {}",
                        rule.name,
                        entry.username.as_deref().unwrap_or("未知"),
                        entry.ip_address.as_deref().unwrap_or("未知"),
                        entry.message
                    ),
                };

                if let Some(sender) = &self.alert_sender {
                    let _ = sender.send(alert).await;
                }

                log::warn!("审计告警: {}", alert.message);
            }
        }
    }

    /// 清理过期日志
    pub async fn cleanup_expired_logs(&self) -> Result<u64, sqlx::Error> {
        let result = sqlx::query(
            r#"
            DELETE FROM audit_logs
            WHERE timestamp < NOW() - INTERVAL '1 day' * $1
            "#,
        )
        .bind(self.config.retention_days)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }
}

/// 审计日志数据库表创建SQL
pub const CREATE_AUDIT_LOGS_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS audit_logs (
    id BIGSERIAL PRIMARY KEY,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    level VARCHAR(20) NOT NULL,
    event_type VARCHAR(50) NOT NULL,
    message TEXT NOT NULL,

    -- 用户信息
    user_id INTEGER,
    username VARCHAR(100),
    ip_address VARCHAR(45),
    user_agent TEXT,

    -- 资源信息
    resource_type VARCHAR(50),
    resource_id VARCHAR(100),
    action VARCHAR(50),

    -- 详细信息
    details JSONB,
    status VARCHAR(20) NOT NULL DEFAULT 'success',
    error_message TEXT,

    -- 性能信息
    duration_ms BIGINT,

    -- 索引
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- 索引
CREATE INDEX IF NOT EXISTS idx_audit_logs_timestamp ON audit_logs(timestamp);
CREATE INDEX IF NOT EXISTS idx_audit_logs_user_id ON audit_logs(user_id);
CREATE INDEX IF NOT EXISTS idx_audit_logs_username ON audit_logs(username);
CREATE INDEX IF NOT EXISTS idx_audit_logs_ip_address ON audit_logs(ip_address);
CREATE INDEX IF NOT EXISTS idx_audit_logs_level ON audit_logs(level);
CREATE INDEX IF NOT EXISTS idx_audit_logs_event_type ON audit_logs(event_type);
CREATE INDEX IF NOT EXISTS idx_audit_logs_resource ON audit_logs(resource_type, resource_id);
CREATE INDEX IF NOT EXISTS idx_audit_logs_status ON audit_logs(status);

-- 表注释
COMMENT ON TABLE audit_logs IS '系统审计日志表 - 等保三级要求保留180天';
COMMENT ON COLUMN audit_logs.level IS '日志级别: debug, info, warning, error, critical';
COMMENT ON COLUMN audit_logs.event_type IS '事件类型: login, logout, password_change, permission_change, data_access, data_modification, etc.';
COMMENT ON COLUMN audit_logs.status IS '操作状态: success, failed';
"#;
