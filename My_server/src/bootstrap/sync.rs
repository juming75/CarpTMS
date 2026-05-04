//! /! 数据同步服务初始化模块

use std::sync::Arc;
use log::info;

/// 启动数据同步服务
pub fn start_sync_service(pool: Arc<sqlx::PgPool>) {
    tokio::spawn(async move {
        info!("Starting data synchronization service...");

        // 从环境变量加载同步配置
        let legacy_config = carptms::sync::config::LegacyServerConfig {
            enabled: std::env::var("LEGACY_SYNC_ENABLED")
                .ok()
                .and_then(|s| s.parse::<bool>().ok())
                .unwrap_or(true),
            host: std::env::var("LEGACY_SYNC_HOST")
                .unwrap_or_else(|_| "203.170.59.153".to_string()),
            port: std::env::var("LEGACY_SYNC_PORT")
                .ok()
                .and_then(|s| s.parse::<u16>().ok())
                .unwrap_or(9808),
            protocol: std::env::var("LEGACY_SYNC_PROTOCOL").unwrap_or_else(|_| "tcp".to_string()),
            username: std::env::var("LEGACY_SYNC_USERNAME").ok(),
            password: std::env::var("LEGACY_SYNC_PASSWORD").ok(),
            auth_token: None,
        };

        let sync_config = carptms::sync::config::SyncConfig {
            enabled: std::env::var("LEGACY_SYNC_ENABLED")
                .ok()
                .and_then(|s| s.parse::<bool>().ok())
                .unwrap_or(true),
            sync_interval_seconds: std::env::var("SYNC_INTERVAL_SECONDS")
                .ok()
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(300),
            realtime_sync_enabled: std::env::var("REALTIME_SYNC_ENABLED")
                .ok()
                .and_then(|s| s.parse::<bool>().ok())
                .unwrap_or(true),
            gps_batch_size: std::env::var("GPS_BATCH_SIZE")
                .ok()
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(100),
            gps_flush_interval_seconds: std::env::var("GPS_FLUSH_INTERVAL_SECONDS")
                .ok()
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(10),
        };

        info!(
            "Legacy sync config: {}://{}:{}",
            legacy_config.protocol,
            legacy_config.host,
            legacy_config.port
        );

        // 创建适配器
        let adapter = carptms::sync::adapter::LegacySyncAdapter::new(legacy_config);

        // 创建同步服务
        let sync_service =
            carptms::sync::service::DataSyncService::new(adapter, pool, sync_config);

        // 尝试连接旧服务器
        {
            let mut adap = sync_service.adapter.lock().await;
            if let Err(e) = adap.connect().await {
                log::warn!("Failed to connect to legacy server: {}", e);
            } else {
                log::info!("Successfully connected to legacy server");

                // 启动全量同步
                if let Err(e) = sync_service.start_full_sync().await {
                    log::error!("Full sync failed: {}", e);
                }
            }
        }

        // 启动增量同步
        if let Err(e) = sync_service.start_incremental_sync().await {
            log::error!("Failed to start incremental sync: {}", e);
        }

        // 启动实时数据流
        if let Err(e) = sync_service.start_realtime_stream().await {
            log::error!("Failed to start realtime stream: {}", e);
        }
    });
}







