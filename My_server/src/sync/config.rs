//! / 数据同步配置
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegacyServerConfig {
    pub enabled: bool,
    pub host: String,
    pub port: u16,
    pub protocol: String, // "tcp" or "http"
    pub username: Option<String>,
    pub password: Option<String>,
    pub auth_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    pub enabled: bool,
    pub sync_interval_seconds: u64,
    pub realtime_sync_enabled: bool,
    pub gps_batch_size: usize,
    pub gps_flush_interval_seconds: u64,
}

impl Default for LegacyServerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            host: "203.170.59.153".to_string(),
            port: 9808,
            protocol: "tcp".to_string(),
            username: Some("ED".to_string()),
            password: Some("888888".to_string()),
            auth_token: None,
        }
    }
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            sync_interval_seconds: 300, // 5分钟
            realtime_sync_enabled: true,
            gps_batch_size: 100,
            gps_flush_interval_seconds: 10,
        }
    }
}
