//! /! 服务初始化模块

use log::{error, info, warn};
use sqlx::postgres::PgPool;
use std::sync::Arc;

/// 启动缓存预热任务
///
/// # 参数
/// - `pool` - 数据库连接池
pub async fn start_cache_preheating(pool: &PgPool) {
    // 仅在开发环境启动缓存预热
    if cfg!(debug_assertions) {
        info!("Starting cache preheating task...");

        let preheat_pool = pool.clone();
        tokio::spawn(async move {
            let cache = crate::cache::VehicleCache::default();
            if let Err(e) = cache.preheat_vehicles(&preheat_pool, &[1, 2, 3]).await {
                warn!("Cache preheating failed: {}", e);
            } else {
                info!("Cache preheating completed successfully");
            }
        });
    }
}

/// 启动数据库连接池监控
///
/// # 参数
/// - `pool` - 数据库连接池
pub async fn start_db_pool_monitoring(pool: &PgPool) {
    info!("Starting database pool monitoring...");

    let monitor_pool = pool.clone();
    tokio::spawn(async move {
        crate::metrics::monitor_db_pool(&monitor_pool).await;
    });
}

/// 启动业务指标监控
///
/// # 参数
/// - `pool` - 数据库连接池
pub async fn start_business_metrics_monitoring(pool: &PgPool) {
    // 仅在生产环境启动业务指标监控
    if !cfg!(debug_assertions) {
        info!("Starting business metrics monitoring...");

        let business_pool = pool.clone();
        tokio::spawn(async move {
            crate::metrics::monitor_business_metrics(&business_pool).await;
        });
    }
}

/// 启动告警服务
///
/// # 参数
/// - `pool` - 数据库连接池
pub async fn start_alert_service(pool: &PgPool) {
    // 仅在生产环境启动告警服务
    if !cfg!(debug_assertions) {
        info!("Starting alert service...");

        let alert_pool = Arc::new(pool.clone());
        tokio::spawn(async move {
            let alert_service = crate::alert::AlertService::new(
                alert_pool,
                crate::central::config::MonitorConfig::default(),
            );
            alert_service.start().await;
            info!("Alert service started successfully");
        });
    }
}

/// 启动数据同步服务
///
/// # 参数
/// - `pool` - 数据库连接池
pub async fn start_data_sync_service(pool: &PgPool) {
    // 仅在需要时启动数据同步服务
    let sync_enabled = std::env::var("LEGACY_SYNC_ENABLED")
        .ok()
        .and_then(|s| s.parse::<bool>().ok())
        .unwrap_or(false);

    if sync_enabled {
        info!("Starting data synchronization service...");

        let sync_pool = Arc::new(pool.clone());
        tokio::spawn(async move {
            // 从环境变量加载同步配置
            let legacy_config = crate::sync::config::LegacyServerConfig {
                enabled: true,
                host: std::env::var("LEGACY_SYNC_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
                port: std::env::var("LEGACY_SYNC_PORT")
                    .ok()
                    .and_then(|s| s.parse::<u16>().ok())
                    .unwrap_or(9809),
                protocol: std::env::var("LEGACY_SYNC_PROTOCOL")
                    .unwrap_or_else(|_| "tcp".to_string()),
                username: std::env::var("LEGACY_SYNC_USERNAME").ok(),
                password: std::env::var("LEGACY_SYNC_PASSWORD").ok(),
                auth_token: None,
            };

            let sync_config = crate::sync::config::SyncConfig {
                enabled: true,
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
                legacy_config.protocol, legacy_config.host, legacy_config.port
            );

            // 创建适配器
            let adapter = crate::sync::adapter::LegacySyncAdapter::new(legacy_config);

            // 创建同步服务
            let sync_service =
                crate::sync::service::DataSyncService::new(adapter, sync_pool, sync_config);

            // 尝试连接旧服务器
            {
                let mut adap: tokio::sync::MutexGuard<'_, crate::sync::adapter::LegacySyncAdapter> =
                    sync_service.adapter.lock().await;
                if let Err(e) = adap.connect().await {
                    warn!("Failed to connect to legacy server: {}", e);
                } else {
                    info!("Successfully connected to legacy server");

                    // 启动全量同步
                    if let Err(e) = sync_service.start_full_sync().await {
                        error!("Full sync failed: {}", e);
                    }
                }
            }

            // 启动增量同步
            if let Err(e) = sync_service.start_incremental_sync().await {
                error!("Failed to start incremental sync: {}", e);
            }

            // 启动实时数据流
            if let Err(e) = sync_service.start_realtime_stream().await {
                error!("Failed to start realtime stream: {}", e);
            }
        });
    }
}

/// 启动视频服务
///
/// # 返回值
/// - `Arc<crate::video::VideoService>` - 视频服务
pub async fn start_video_service() -> Arc<crate::video::VideoService> {
    // 仅在需要时启动视频服务
    let video_enabled = std::env::var("VIDEO_SERVICE_ENABLED")
        .ok()
        .and_then(|s| s.parse::<bool>().ok())
        .unwrap_or(false);

    if video_enabled {
        info!("Initializing video service...");

        let video_service_result = crate::video::VideoService::from_env();

        match video_service_result {
            Ok(service) => {
                let service: Arc<crate::video::VideoService> = Arc::new(service);
                let service_clone = service.clone();

                tokio::spawn(async move {
                    if let Err(e) = service_clone.start().await {
                        error!("Failed to start video service: {}", e);
                    } else {
                        info!("Video service started successfully");
                    }
                });

                info!("Video service initialized successfully");
                service
            }
            Err(e) => {
                warn!(
                    "Failed to initialize video service: {}. Video features will be disabled.",
                    e
                );
                // 创建一个禁用的视频服务
                match crate::video::VideoService::new(crate::video::VideoConfig {
                    enabled: false,
                    ..Default::default()
                }) {
                    Ok(service) => Arc::new(service),
                    Err(create_err) => {
                        error!(
                            "Failed to create disabled video service: {}. Using minimal fallback.",
                            create_err
                        );
                        // 使用expect替代unwrap，提供更清晰的错误信息
                        Arc::new(crate::video::VideoService::new(crate::video::VideoConfig::default()).unwrap_or_else(|fallback_err| {
                            panic!("Video service creation failed completely: {}. Check video config and dependencies.", fallback_err)
                        }))
                    }
                }
            }
        }
    } else {
        // 创建一个禁用的视频服务
        match crate::video::VideoService::new(crate::video::VideoConfig {
            enabled: false,
            ..Default::default()
        }) {
            Ok(service) => Arc::new(service),
            Err(create_err) => {
                error!(
                    "Failed to create disabled video service: {}. Using minimal fallback.",
                    create_err
                );
                Arc::new(crate::video::VideoService::new(crate::video::VideoConfig::default()).unwrap_or_else(|fallback_err| {
                    panic!("Video service creation failed completely: {}. Check video config and dependencies.", fallback_err)
                }))
            }
        }
    }
}

/// 启动性能监控服务
///
/// # 参数
/// - `pool` - 数据库连接池
pub async fn start_performance_monitor(_pool: &PgPool) {
    // 仅在生产环境启动性能监控服务
    if !cfg!(debug_assertions) {
        info!("Starting performance monitoring service...");

        use crate::performance::enhanced_monitor::{
            PerformanceMonitorConfig, PerformanceMonitorService,
        };

        let monitor_config = PerformanceMonitorConfig {
            enabled: true,
            collect_interval: std::time::Duration::from_secs(60),
            retention_period: std::time::Duration::from_secs(3600 * 24 * 7), // 7 days
            alert_thresholds: Default::default(),
            export_prometheus: true,
        };

        let result = PerformanceMonitorService::new(monitor_config);

        match result {
            Ok(service) => {
                let _monitor = service.get_monitor();

                // 启动后台收集任务
                service.start_background_collection();

                info!("Performance monitoring service started successfully");

                // 暴露到全局(如果需要)
                // 这里可以将监控器暴露到应用状态中
            }
            Err(e) => {
                warn!("Failed to start performance monitoring service: {}", e);
            }
        }
    }
}

/// 启动灾难恢复服务
///
/// # 参数
/// - `pool` - 数据库连接池
pub async fn start_disaster_recovery_service(_pool: &PgPool) {
    // 仅在生产环境启动灾难恢复服务
    if !cfg!(debug_assertions) {
        info!("Starting disaster recovery service...");

        use crate::disaster_recovery::{
            DisasterRecoveryConfig, DisasterRecoveryManager, LocalStorageBackend,
        };
        use std::path::PathBuf;
        use std::sync::Arc;

        // 从环境变量获取配置
        let backup_storage_path = std::env::var("DR_BACKUP_STORAGE_PATH")
            .unwrap_or_else(|_| "/var/backups/CarpTMS".to_string());

        let config = DisasterRecoveryConfig {
            backup_storage_path: PathBuf::from(backup_storage_path),
            ..Default::default()
        };

        // 创建存储后端
        let storage_backend =
            Arc::new(LocalStorageBackend::new(config.backup_storage_path.clone()));

        // 确保备份目录存在
        if let Err(e) = tokio::fs::create_dir_all(&config.backup_storage_path).await {
            warn!(
                "Failed to create backup directory: {}, disaster recovery may not work properly",
                e
            );
        }

        // 初始化灾难恢复管理器
        match DisasterRecoveryManager::new(
            config,
            storage_backend,
            "primary-region".to_string(),
            vec![
                "secondary-region-1".to_string(),
                "secondary-region-2".to_string(),
            ],
        )
        .await
        {
            Ok(dr_manager) => {
                // 启动灾难恢复服务
                tokio::spawn(async move {
                    if let Err(e) = dr_manager.initialize().await {
                        error!("Failed to initialize disaster recovery: {}", e);
                    } else {
                        info!("Disaster recovery service initialized successfully");

                        // 启动定期备份任务
                        let dr_manager_clone = Arc::new(dr_manager);
                        tokio::spawn(async move {
                            let mut interval =
                                tokio::time::interval(tokio::time::Duration::from_hours(24));
                            loop {
                                interval.tick().await;
                                if let Err(e) = dr_manager_clone.perform_scheduled_backup().await {
                                    error!("Scheduled backup failed: {}", e);
                                }
                            }
                        });
                    }
                });

                info!("Disaster recovery service started successfully");
            }
            Err(e) => {
                warn!("Failed to start disaster recovery service: {}", e);
            }
        }
    }
}
