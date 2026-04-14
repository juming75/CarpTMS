//! Alerts Application Service
//!
//! Encapsulates all SQL for alert management.

use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{PgPool, Row};
use chrono::NaiveDate;

use crate::errors::AppResult;
use crate::redis::{get_cache, set_cache};

#[derive(Debug, Deserialize)]
pub struct AlertQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct PagedAlertResponse {
    pub alerts: Vec<serde_json::Value>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
    pub pages: i64,
}

#[derive(Debug, Serialize)]
pub struct AlertStats {
    pub total: i64,
    pub unprocessed: i64,
    pub processed: i64,
    pub critical: i64,
}

pub struct AlertApplicationService {
    pool: PgPool,
}

impl AlertApplicationService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_alert_stats(&self) -> AppResult<AlertStats> {
        let cache_key = "alerts:stats";
        
        if let Ok(Some(cached)) = get_cache::<serde_json::Value>(cache_key).await {
            return Ok(AlertStats {
                total: cached["total"].as_i64().unwrap_or(0),
                unprocessed: cached["unprocessed"].as_i64().unwrap_or(0),
                processed: cached["processed"].as_i64().unwrap_or(0),
                critical: cached["critical"].as_i64().unwrap_or(0),
            });
        }

        let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM alerts")
            .fetch_one(&self.pool)
            .await
            .unwrap_or(0);

        let unprocessed: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM alerts WHERE status = 0")
            .fetch_one(&self.pool)
            .await
            .unwrap_or(0);

        let processed: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM alerts WHERE status = 1")
            .fetch_one(&self.pool)
            .await
            .unwrap_or(0);

        let critical: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM alerts WHERE priority = 1")
            .fetch_one(&self.pool)
            .await
            .unwrap_or(0);

        let stats = AlertStats {
            total,
            unprocessed,
            processed,
            critical,
        };

        let cache_value = json!({
            "total": stats.total,
            "unprocessed": stats.unprocessed,
            "processed": stats.processed,
            "critical": stats.critical
        });
        let _ = set_cache(cache_key, &cache_value, 300).await;

        Ok(stats)
    }

    pub async fn get_alerts(&self, query: AlertQuery) -> AppResult<PagedAlertResponse> {
        let page = query.page.unwrap_or(1);
        let page_size = query.page_size.unwrap_or(20);
        let offset = (page - 1) * page_size;

        let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM alerts")
            .fetch_one(&self.pool)
            .await
            .unwrap_or(0);

        let rows = sqlx::query("SELECT * FROM alerts ORDER BY created_at DESC LIMIT $1 OFFSET $2")
            .bind(page_size)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
            .unwrap_or_default();

        let alerts: Vec<serde_json::Value> = rows
            .iter()
            .map(|row| {
                json!({
                    "alert_id": row.get::<i64, _>("alert_id"),
                    "vehicle_id": row.get::<i32, _>("vehicle_id"),
                    "alert_type": row.get::<String, _>("alert_type"),
                    "priority": row.get::<i32, _>("priority"),
                    "status": row.get::<i32, _>("status"),
                    "created_at": row.get::<chrono::DateTime<chrono::Utc>, _>("created_at"),
                    "processed_at": row.get::<Option<chrono::DateTime<chrono::Utc>>, _>("processed_at")
                })
            })
            .collect();

        let pages = if total % page_size == 0 {
            total / page_size
        } else {
            total / page_size + 1
        };

        Ok(PagedAlertResponse {
            alerts,
            total,
            page,
            page_size,
            pages,
        })
    }

    pub async fn get_quick_process(&self) -> AppResult<Vec<serde_json::Value>> {
        let rows = sqlx::query("SELECT * FROM alerts WHERE status = 0 ORDER BY priority DESC LIMIT 5")
            .fetch_all(&self.pool)
            .await
            .unwrap_or_default();

        let result: Vec<serde_json::Value> = rows
            .iter()
            .map(|row| {
                json!({
                    "alert_id": row.get::<i64, _>("alert_id"),
                    "vehicle_id": row.get::<i32, _>("vehicle_id"),
                    "alert_type": row.get::<String, _>("alert_type"),
                    "priority": row.get::<i32, _>("priority"),
                    "status": row.get::<i32, _>("status"),
                    "created_at": row.get::<chrono::DateTime<chrono::Utc>, _>("created_at")
                })
            })
            .collect();

        Ok(result)
    }

    pub async fn get_alert_trend(&self) -> AppResult<Vec<serde_json::Value>> {
        let rows = sqlx::query(
            "SELECT DATE(created_at) as date, COUNT(*) as count 
             FROM alerts 
             WHERE created_at > NOW() - INTERVAL '7 days' 
             GROUP BY DATE(created_at) 
             ORDER BY date"
        )
        .fetch_all(&self.pool)
        .await
        .unwrap_or_default();

        let result: Vec<serde_json::Value> = rows
            .iter()
            .map(|row| {
                json!({
                    "date": row.get::<NaiveDate, _>("date"),
                    "count": row.get::<i64, _>("count")
                })
            })
            .collect();

        Ok(result)
    }

    pub async fn get_alert_types(&self) -> AppResult<Vec<serde_json::Value>> {
        let rows = sqlx::query(
            "SELECT alert_type, COUNT(*) as count 
             FROM alerts 
             GROUP BY alert_type 
             ORDER BY count DESC"
        )
        .fetch_all(&self.pool)
        .await
        .unwrap_or_default();

        let result: Vec<serde_json::Value> = rows
            .iter()
            .map(|row| {
                json!({
                    "alert_type": row.get::<String, _>("alert_type"),
                    "count": row.get::<i64, _>("count")
                })
            })
            .collect();

        Ok(result)
    }

    pub async fn process_alert(&self, alert_id: i32) -> AppResult<u64> {
        let result = sqlx::query("UPDATE alerts SET status = 1, processed_at = NOW() WHERE alert_id = $1")
            .bind(alert_id)
            .execute(&self.pool)
            .await
            .unwrap_or_default();

        Ok(result.rows_affected())
    }
}
