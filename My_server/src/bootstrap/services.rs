//! /! 应用服务初始化模块
//!
//! 负责创建和配置所有应用服务

use log::info;
use std::sync::Arc;

use crate::domain::use_cases::driver::{DriverRepository, DriverUseCases};
use crate::infrastructure::repositories::driver_repository::PgDriverRepository;
use crate::{alert, bff, central, gateway, video};

/// 应用状态,包含所有需要共享的服务
#[derive(Clone)]
pub struct ApplicationState {
    /// 数据库连接池
    pub pool: Arc<sqlx::PgPool>,
    /// BFF数据源管理器
    pub datasource_manager: Arc<bff::datasources::DataSourceManager>,
    /// BFF车辆聚合服务
    pub vehicle_aggregator: Arc<bff::services::VehicleAggregator>,
    /// BFF报表服务
    pub report_service: Arc<bff::reports::ReportService>,
    /// BFF报表模板引擎
    pub template_engine: Arc<bff::templates::ReportTemplateEngine>,
    /// 视频服务
    pub video_service: Arc<video::VideoService>,
    /// WebSocket应用状态
    pub ws_app_state: Arc<gateway::websocket_server::WsAppState>,
    /// 车辆缓存服务
    pub vehicle_cache: Arc<crate::cache::VehicleCache>,
    /// 司机服务
    pub driver_service: Arc<DriverUseCases>,
}

impl ApplicationState {
    /// 创建新的应用状态
    pub async fn new(
        pool: Arc<sqlx::PgPool>,
        _central_config: &central::config::CentralConfig,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // 初始化BFF层数据源管理器
        let redis_connection = crate::redis::get_redis_connection().await.cloned();
        let datasource_manager = Arc::new(bff::datasources::DataSourceManager::new(
            pool.clone(),
            redis_connection, // 使用Redis连接
            true,             // 启用旧服务器TCP连接
        ));

        // 初始化BFF车辆聚合服务
        let vehicle_aggregator = Arc::new(bff::services::VehicleAggregator::new(
            datasource_manager.clone(),
            pool.clone(),
            None,
        ));

        // 初始化BFF报表服务
        let report_service = Arc::new(bff::reports::ReportService::new(pool.clone()));

        // 初始化BFF报表模板引擎
        let template_engine = Arc::new(
            bff::templates::ReportTemplateEngine::new("")
                .map_err(|e| format!("Failed to initialize template engine: {}", e))?,
        );

        // 初始化视频服务
        let video_service = Self::init_video_service()?;

        // 初始化车辆缓存服务
        let vehicle_cache = Arc::new(crate::cache::VehicleCache::default());

        // 初始化WebSocket应用状态
        let ws_app_state = Arc::new(gateway::websocket_server::WsAppState::new());

        // 初始化司机服务
        let driver_repository: Arc<dyn DriverRepository + Send + Sync> =
            Arc::new(PgDriverRepository::new(pool.clone()));
        let driver_service = Arc::new(DriverUseCases::new(driver_repository));

        // 启动缓存预热任务
        let preheat_pool = pool.clone();
        let preheat_cache = vehicle_cache.clone();
        tokio::spawn(async move {
            let _ = preheat_cache
                .preheat_vehicles(&preheat_pool, &[1, 2, 3])
                .await;
        });

        // 启动数据库连接池健康检查任务
        let health_pool = pool.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
            loop {
                interval.tick().await;
                match sqlx::query("SELECT 1").execute(health_pool.as_ref()).await {
                    Ok(_) => {
                        log::debug!("[DB Health] Check passed");
                    }
                    Err(e) => {
                        log::error!("[DB Health] Check failed: {}", e);
                    }
                }
            }
        });

        Ok(ApplicationState {
            pool,
            datasource_manager,
            vehicle_aggregator,
            report_service,
            template_engine,
            video_service,
            ws_app_state,
            vehicle_cache,
            driver_service,
        })
    }

    /// 初始化视频服务
    fn init_video_service() -> Result<Arc<video::VideoService>, Box<dyn std::error::Error>> {
        let video_service_result = video::VideoService::from_env();
        match video_service_result {
            Ok(service) => {
                let service: Arc<video::VideoService> = Arc::new(service);
                let service_clone = service.clone();

                tokio::spawn(async move {
                    if let Err(e) = service_clone.start().await {
                        log::error!("Failed to start video service: {}", e);
                    }
                });

                info!("Video service initialized successfully");
                Ok(service)
            }
            Err(e) => {
                log::warn!(
                    "Failed to initialize video service: {}. Video features will be disabled.",
                    e
                );
                // 创建一个禁用的视频服务
                match video::VideoService::new(video::VideoConfig {
                    enabled: false,
                    ..Default::default()
                }) {
                    Ok(service) => Ok(Arc::new(service)),
                    Err(create_err) => {
                        log::error!(
                            "Failed to create disabled video service: {}. Using minimal fallback.",
                            create_err
                        );
                        // 使用unwrap_or_else替代unwrap，提供更清晰的错误信息
                        Ok(Arc::new(video::VideoService::new(video::VideoConfig::default()).unwrap_or_else(|fallback_err| {
                            panic!("Video service creation failed completely: {}. Check video config and dependencies.", fallback_err)
                        })))
                    }
                }
            }
        }
    }

    /// 启动后台服务
    pub fn start_background_services(&self, _central_config: &central::config::CentralConfig) {
        // 启动数据库连接池监控
        let monitor_pool = self.pool.clone();
        tokio::spawn(async move {
            crate::metrics::monitor_db_pool(&monitor_pool).await;
        });

        // 启动业务指标监控
        let business_pool = self.pool.clone();
        tokio::spawn(async move {
            crate::metrics::monitor_business_metrics(&business_pool).await;
        });

        // 启动告警服务
        let alert_pool = self.pool.clone();
        tokio::spawn(async move {
            let alert_service =
                alert::AlertService::new(alert_pool, central::config::MonitorConfig::default());
            alert_service.start().await;
        });
    }
}
