//! / 报警服务
// 处理报警数据的存储、查询和推送

use actix::prelude::*;
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use chrono::{DateTime, Utc};

use super::parser::AlarmParser;
use super::notifier::AlarmNotifier;

/// 报警级别
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum AlarmLevel {
    /// 提示
    Tip = 0,
    /// 警告
    Warning = 1,
    /// 高危
    High = 2,
    /// 紧急
    Critical = 3,
}

impl From<u32> for AlarmLevel {
    fn from(value: u32) -> Self {
        match value {
            3 => AlarmLevel::Critical,
            2 => AlarmLevel::High,
            1 => AlarmLevel::Warning,
            _ => AlarmLevel::Tip,
        }
    }
}

/// 报警状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlarmStatus {
    /// 待处理
    Pending,
    /// 已处理
    Handled,
    /// 已忽略
    Ignored,
}

/// 报警记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlarmRecord {
    pub id: Option<i64>,
    pub device_id: String,
    pub phone: Option<String>,
    pub alarm_type: String,
    pub alarm_level: AlarmLevel,
    pub alarm_time: DateTime<Utc>,
    pub location: Option<serde_json::Value>,
    pub description: Option<String>,
    pub status: AlarmStatus,
    pub handled_by: Option<String>,
    pub handled_at: Option<DateTime<Utc>>,
}

/// 报警查询参数
#[derive(Debug, Clone, Deserialize)]
pub struct AlarmQueryParams {
    pub device_id: Option<String>,
    pub alarm_type: Option<String>,
    pub alarm_level: Option<AlarmLevel>,
    pub status: Option<AlarmStatus>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

/// 报警统计
#[derive(Debug, Clone, Serialize)]
pub struct AlarmStatistics {
    pub total_count: i64,
    pub pending_count: i64,
    pub handled_count: i64,
    pub ignored_count: i64,
    pub by_level: std::collections::HashMap<String, i64>,
    pub by_type: std::collections::HashMap<String, i64>,
}

/// 报警服务
pub struct AlarmService {
    db_pool: PgPool,
    parser: AlarmParser,
    notifier: AlarmNotifier,
}

impl AlarmService {
    pub fn new(db_pool: PgPool) -> Self {
        info!("Creating alarm service");

        Self {
            db_pool,
            parser: AlarmParser::new(),
            notifier: AlarmNotifier::new(),
        }
    }

    /// 创建报警记录
    pub async fn create_alarm(&self, alarm: AlarmRecord) -> Result<i64, String> {
        debug!("Creating alarm for device {}: {}", alarm.device_id, alarm.alarm_type);

        let query = r#"
            INSERT INTO device_alarms (
                device_id, phone, alarm_type, alarm_level, alarm_time,
                location, description, status
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id
        "#;

        let id = sqlx::query_scalar::<_, i64>(query)
            .bind(&alarm.device_id)
            .bind(&alarm.phone)
            .bind(&alarm.alarm_type)
            .bind(alarm.alarm_level as i32)
            .bind(alarm.alarm_time)
            .bind(&alarm.location)
            .bind(&alarm.description)
            .bind(serde_json::to_string(&AlarmStatus::Pending).unwrap_or_else(|_| "pending".to_string()))
            .fetch_one(&self.db_pool)
            .await
            .map_err(|e| format!("Database error: {}", e))?;

        info!("Alarm created with ID: {}", id);

        // 通知报警
        let alarm_with_id = AlarmRecord {
            id: Some(id),
            ..alarm
        };
        if let Err(e) = self.notifier.notify(&alarm_with_id).await {
            error!("Failed to send alarm notification: {}", e);
        }

        Ok(id)
    }

    /// 查询报警列表
    pub async fn query_alarms(&self, params: AlarmQueryParams) -> Result<Vec<AlarmRecord>, String> {
        debug!("Querying alarms with params: {:?}", params);

        let mut query = "SELECT id, device_id, phone, alarm_type, alarm_level, alarm_time, location, description, status, handled_by, handled_at FROM device_alarms WHERE 1=1".to_string();
        let mut index = 1;
        let mut binds: Vec<Box<dyn sqlx::Encode<sqlx::Postgres> + Send>> = Vec::new();

        if let Some(device_id) = &params.device_id {
            query.push_str(&format!(" AND device_id = ${}", index));
            index += 1;
            binds.push(Box::new(device_id.clone()));
        }

        if let Some(alarm_type) = &params.alarm_type {
            query.push_str(&format!(" AND alarm_type = ${}", index));
            index += 1;
            binds.push(Box::new(alarm_type.clone()));
        }

        if let Some(alarm_level) = &params.alarm_level {
            query.push_str(&format!(" AND alarm_level = ${}", index));
            index += 1;
            binds.push(Box::new(*alarm_level as i32));
        }

        if let Some(status) = &params.status {
            query.push_str(&format!(" AND status = ${}", index));
            index += 1;
            binds.push(Box::new(serde_json::to_string(status).unwrap_or_else(|_| "pending".to_string())));
        }

        if let Some(start_time) = &params.start_time {
            query.push_str(&format!(" AND alarm_time >= ${}", index));
            index += 1;
            binds.push(Box::new(*start_time));
        }

        if let Some(end_time) = &params.end_time {
            query.push_str(&format!(" AND alarm_time <= ${}", index));
            index += 1;
            binds.push(Box::new(*end_time));
        }

        query.push_str(&format!(" ORDER BY alarm_time DESC LIMIT ${} OFFSET ${}", index, index + 1));

        let limit = params.limit.unwrap_or(100) as i64;
        let offset = params.offset.unwrap_or(0) as i64;
        binds.push(Box::new(limit));
        binds.push(Box::new(offset));

        // 注意:这里需要更复杂的查询构建,简化实现
        let result = sqlx::query_as::<_, (i64, String, Option<String>, String, i32, DateTime<Utc>, Option<serde_json::Value>, Option<String>, String, Option<String>, Option<DateTime<Utc>)>(&query)
            .bind(&params.device_id)
            .bind(&params.alarm_type)
            .fetch_all(&self.db_pool)
            .await
            .map_err(|e| format!("Database error: {}", e))?;

        let alarms = result.into_iter()
            .map(|(id, device_id, phone, alarm_type, level, time, loc, desc, status, handled_by, handled_at)| {
                AlarmRecord {
                    id: Some(id),
                    device_id,
                    phone,
                    alarm_type,
                    alarm_level: level.into(),
                    alarm_time: time,
                    location: loc,
                    description: desc,
                    status: parse_status(&status),
                    handled_by,
                    handled_at,
                }
            })
            .collect();

        info!("Found {} alarms", alarms.len());
        Ok(alarms)
    }

    /// 获取报警详情
    pub async fn get_alarm(&self, alarm_id: i64) -> Result<Option<AlarmRecord>, String> {
        debug!("Querying alarm with ID: {}", alarm_id);

        let result = sqlx::query_as::<_, (i64, String, Option<String>, String, i32, DateTime<Utc>, Option<serde_json::Value>, Option<String>, String, Option<String>, Option<DateTime<Utc>)>(
            "SELECT id, device_id, phone, alarm_type, alarm_level, alarm_time, location, description, status, handled_by, handled_at FROM device_alarms WHERE id = $1"
        )
        .bind(alarm_id)
        .fetch_optional(&self.db_pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(result.map(|(id, device_id, phone, alarm_type, level, time, loc, desc, status, handled_by, handled_at)| {
            AlarmRecord {
                id: Some(id),
                device_id,
                phone,
                alarm_type,
                alarm_level: level.into(),
                alarm_time: time,
                location: loc,
                description: desc,
                status: parse_status(&status),
                handled_by,
                handled_at,
            }
        }))
    }

    /// 获取报警统计
    pub async fn get_statistics(&self, params: AlarmQueryParams) -> Result<AlarmStatistics, String> {
        debug!("Querying alarm statistics");

        let total_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM device_alarms WHERE status = 'pending'")
            .fetch_one(&self.db_pool)
            .await
            .unwrap_or(0);

        let pending_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM device_alarms WHERE status = 'pending'")
            .fetch_one(&self.db_pool)
            .await
            .unwrap_or(0);

        let handled_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM device_alarms WHERE status = 'handled'")
            .fetch_one(&self.db_pool)
            .await
            .unwrap_or(0);

        let ignored_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM device_alarms WHERE status = 'ignored'")
            .fetch_one(&self.db_pool)
            .await
            .unwrap_or(0);

        Ok(AlarmStatistics {
            total_count,
            pending_count,
            handled_count,
            ignored_count,
            by_level: std::collections::HashMap::new(),
            by_type: std::collections::HashMap::new(),
        })
    }

    /// 处理报警
    pub async fn handle_alarm(&self, alarm_id: i64, handled_by: String) -> Result<(), String> {
        info!("Handling alarm {}: handled_by={}", alarm_id, handled_by);

        sqlx::query("UPDATE device_alarms SET status = 'handled', handled_by = $1, handled_at = NOW() WHERE id = $2")
            .bind(&handled_by)
            .bind(alarm_id)
            .execute(&self.db_pool)
            .await
            .map_err(|e| format!("Database error: {}", e))?;

        Ok(())
    }

    /// 忽略报警
    pub async fn ignore_alarm(&self, alarm_id: i64) -> Result<(), String> {
        info!("Ignoring alarm {}", alarm_id);

        sqlx::query("UPDATE device_alarms SET status = 'ignored' WHERE id = $1")
            .bind(alarm_id)
            .execute(&self.db_pool)
            .await
            .map_err(|e| format!("Database error: {}", e))?;

        Ok(())
    }
}

/// 解析状态字符串
fn parse_status(status: &str) -> AlarmStatus {
    match status {
        "handled" => AlarmStatus::Handled,
        "ignored" => AlarmStatus::Ignored,
        _ => AlarmStatus::Pending,
    }
}






