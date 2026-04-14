//! Settings Application Service
//!
//! Encapsulates all SQL for system settings, removing direct DB access from routes.

use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use utoipa::ToSchema;

use crate::errors::AppResult;

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct SystemSettings {
    pub server_url: Option<String>,
    pub sync_interval: Option<i32>,
    pub auto_sync: Option<bool>,
    pub home_page_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct CommunicationSettings {
    pub server_ip: Option<String>,
    pub server_port: Option<i32>,
    pub heartbeat_interval: Option<i32>,
    pub timeout: Option<i32>,
    pub reconnect_count: Option<i32>,
    pub protocol: Option<String>,
    pub compression: Option<bool>,
    pub encryption: Option<bool>,
}

/// Settings application service
pub struct SettingsApplicationService {
    pool: PgPool,
}

impl SettingsApplicationService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_settings(&self) -> AppResult<SystemSettings> {
        // 检查表是否存在
        let table_exists = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS (
                SELECT 1 FROM information_schema.tables 
                WHERE table_name = 'system_settings'
            )"
        )
        .fetch_one(&self.pool)
        .await
        .unwrap_or(false);

        if !table_exists {
            return Ok(SystemSettings::default());
        }

        let row = sqlx::query(
            "SELECT setting_value FROM system_settings WHERE setting_key = 'system_config' LIMIT 1",
        )
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(row) => {
                let value: serde_json::Value = row.get("setting_value");
                Ok(SystemSettings {
                    server_url: value["server_url"].as_str().map(|s: &str| s.to_string()),
                    sync_interval: value["sync_interval"].as_i64().map(|i| i as i32),
                    auto_sync: value["auto_sync"].as_bool(),
                    home_page_name: value["home_page_name"].as_str().map(|s: &str| s.to_string()),
                })
            }
            None => Ok(SystemSettings::default()),
        }
    }

    pub async fn update_settings(&self, settings: SystemSettings) -> AppResult<SystemSettings> {
        let settings_json = serde_json::json!({
            "server_url": settings.server_url,
            "sync_interval": settings.sync_interval,
            "auto_sync": settings.auto_sync,
            "home_page_name": settings.home_page_name
        });

        let _ = sqlx::query(
            "INSERT INTO system_settings (setting_key, setting_value, updated_at)
             VALUES ($1, $2, NOW())
             ON CONFLICT (setting_key)
             DO UPDATE SET setting_value = $2, updated_at = NOW()",
        )
        .bind("system_config")
        .bind(settings_json)
        .execute(&self.pool)
        .await;

        Ok(settings)
    }

    pub async fn get_communication_settings(&self) -> AppResult<CommunicationSettings> {
        // 检查表是否存在
        let table_exists = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS (
                SELECT 1 FROM information_schema.tables 
                WHERE table_name = 'system_settings'
            )"
        )
        .fetch_one(&self.pool)
        .await
        .unwrap_or(false);

        if !table_exists {
            return Ok(CommunicationSettings::default());
        }

        let row = sqlx::query(
            "SELECT setting_value FROM system_settings WHERE setting_key = 'communication_config' LIMIT 1",
        )
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(row) => {
                let value: serde_json::Value = row.get("setting_value");
                Ok(CommunicationSettings {
                    server_ip: value["server_ip"].as_str().map(|s: &str| s.to_string()),
                    server_port: value["server_port"].as_i64().map(|i| i as i32),
                    heartbeat_interval: value["heartbeat_interval"].as_i64().map(|i| i as i32),
                    timeout: value["timeout"].as_i64().map(|i| i as i32),
                    reconnect_count: value["reconnect_count"].as_i64().map(|i| i as i32),
                    protocol: value["protocol"].as_str().map(|s: &str| s.to_string()),
                    compression: value["compression"].as_bool(),
                    encryption: value["encryption"].as_bool(),
                })
            }
            None => Ok(CommunicationSettings::default()),
        }
    }

    pub async fn update_communication_settings(&self, settings: CommunicationSettings) -> AppResult<CommunicationSettings> {
        let settings_json = serde_json::json!({
            "server_ip": settings.server_ip,
            "server_port": settings.server_port,
            "heartbeat_interval": settings.heartbeat_interval,
            "timeout": settings.timeout,
            "reconnect_count": settings.reconnect_count,
            "protocol": settings.protocol,
            "compression": settings.compression,
            "encryption": settings.encryption
        });

        let _ = sqlx::query(
            "INSERT INTO system_settings (setting_key, setting_value, updated_at)
             VALUES ($1, $2, NOW())
             ON CONFLICT (setting_key)
             DO UPDATE SET setting_value = $2, updated_at = NOW()",
        )
        .bind("communication_config")
        .bind(settings_json)
        .execute(&self.pool)
        .await;

        Ok(settings)
    }
}

impl Default for SystemSettings {
    fn default() -> Self {
        Self {
            server_url: Some("http://127.0.0.1:8081".to_string()),
            sync_interval: Some(5),
            auto_sync: Some(true),
            home_page_name: Some("车辆运营监控平台".to_string()),
        }
    }
}

impl Default for CommunicationSettings {
    fn default() -> Self {
        Self {
            server_ip: Some("127.0.0.1".to_string()),
            server_port: Some(8988),
            heartbeat_interval: Some(30),
            timeout: Some(10),
            reconnect_count: Some(3),
            protocol: Some("tcp".to_string()),
            compression: Some(true),
            encryption: Some(true),
        }
    }
}
