//! 服务器初始化模块
//!
//! 负责服务器组件的初始化和启动

use actix::{Actor, Addr};
use log::info;
use std::sync::Arc;

use crate::{
    alert, cache::vehicle_cache, central, gateway, infrastructure,
    infrastructure::message_router::router::MessageRouter, metrics, truck_scale,
};

use crate::bff::datasources::DataSourceManager;
use crate::bff::reports::ReportService;
use crate::bff::services::VehicleAggregator;
use crate::bff::templates::ReportTemplateEngine;
use redis::Client as RedisClient;

/// 服务器初始化上下文
pub struct ServerInitContext {
    /// 数据库连接池
    pub pool: Arc<sqlx::PgPool>,
    /// 服务器配置
    pub config: crate::config::unified::UnifiedConfig,
    /// Redis客户端
    pub redis_client: Option<std::sync::Arc<RedisClient>>,
    /// 消息路由器地址
    pub message_router: Option<Addr<MessageRouter>>,
}

/// 初始化监控和追踪
pub async fn init_monitoring() {
    // 初始化日志系统
    let log_config = crate::utils::log::load_log_config_from_env();
    if let Err(e) = crate::utils::log::init_logging(log_config).await {
        eprintln!("Failed to initialize logging: {:?}", e);
    }

    // 初始化分布式追踪
    if let Err(e) = crate::telemetry::init_telemetry("tms_server") {
        log::warn!("Failed to initialize telemetry: {:?}", e);
    } else {
        log::info!("Telemetry initialized successfully");
    }

    // 初始化监控指标
    metrics::init_metrics();
}

/// 初始化数据库
pub async fn init_database() -> Result<Arc<sqlx::PgPool>, std::io::Error> {
    info!("Initializing database connection...");
    let config = crate::init::database::load_config()
        .await
        .map_err(|e| std::io::Error::other(format!("Failed to load database config: {}", e)))?;
    info!("Database initialized successfully");
    Ok(config.pool)
}

/// 初始化Redis
pub async fn init_redis(pool: Arc<sqlx::PgPool>) -> Option<std::sync::Arc<RedisClient>> {
    info!("Initializing Redis connection...");
    let redis_result = crate::redis::init_redis().await;

    let redis_client = match redis_result {
        Ok(_) => {
            info!("Successfully initialized Redis connection pool.");

            // 缓存预热任务已在ApplicationState初始化时启动

            // 创建Redis客户端用于速率限制
            let client = std::env::var("REDIS_URL")
                .ok()
                .and_then(|url| RedisClient::open(url).ok())
                .or_else(|| RedisClient::open("redis://localhost:6379/0").ok())
                .map(std::sync::Arc::new)
                .unwrap_or_else(|| {
                    // 如果无法创建Redis客户端,仍然创建一个占位符
                    std::sync::Arc::new(
                        RedisClient::open("redis://localhost:6379/0")
                            .expect("Failed to create placeholder Redis client"),
                    )
                });

            Some(client)
        }
        Err(e) => {
            info!(
                "Failed to initialize Redis: {}. Redis features will be disabled.",
                e
            );
            None
        }
    };

    redis_client
}

/// 初始化后台监控任务
pub fn init_background_monitoring(pool: Arc<sqlx::PgPool>) {
    // 启动数据库连接池监控
    let monitor_pool = pool.clone();
    tokio::spawn(async move {
        metrics::monitor_db_pool(&monitor_pool).await;
    });

    // 启动业务指标监控
    let business_pool = pool.clone();
    tokio::spawn(async move {
        metrics::monitor_business_metrics(&business_pool).await;
    });

    // 启动告警服务
    let alert_pool = pool.clone();
    tokio::spawn(async move {
        let alert_service =
            alert::AlertService::new(alert_pool, central::config::MonitorConfig::default());
        alert_service.start().await;
    });
}

/// 初始化BFF服务
pub async fn init_bff_services(pool: Arc<sqlx::PgPool>) -> (Arc<DataSourceManager>, Arc<ReportService>) {
    // 初始化BFF层数据源管理器
    let redis_connection = crate::redis::get_redis_connection().await.cloned();
    let datasource_manager = Arc::new(DataSourceManager::new(
        pool.clone(),
        redis_connection, // 使用Redis连接
        true, // 启用旧服务器TCP连接
    ));

    // 初始化BFF车辆聚合服务
    let _vehicle_aggregator = Arc::new(VehicleAggregator::new(
        datasource_manager.clone(),
        pool.clone(),
        None, // Redis connection is managed by datasource_manager
    ));

    // 初始化BFF报表服务
    let report_service = Arc::new(ReportService::new(pool.clone()));

    (datasource_manager, report_service)
}

/// 初始化报表模板引擎
pub fn init_template_engine() -> Result<Arc<ReportTemplateEngine>, std::io::Error> {
    info!("Initializing report template engine...");
    let template_engine = Arc::new(ReportTemplateEngine::new("").map_err(|e| {
        std::io::Error::other(format!("Failed to initialize template engine: {}", e))
    })?);
    Ok(template_engine)
}

/// 初始化网关服务
pub fn init_gateway_services(
    _pool: Arc<sqlx::PgPool>,
    _config: &crate::config::unified::UnifiedConfig,
    message_router: &mut Option<Addr<MessageRouter>>,
) -> AppResult<(
    actix::Addr<gateway::gprs_server::ProtocolAnalyzer>,
    Option<actix::Addr<central::service::CentralService>>,
    actix::Addr<crate::protocols::jt808::session::Jt808SessionManager>,
)> {
    let central_config = central::config::CentralConfig::from_env()
        .with_context(|| "Failed to load central configuration")?;

    // 初始化中心服务
    let central_service = central::service::CentralService::new();
    let central_addr = central_service.start();

    // 启动中心服务
    central_addr.do_send(central::StartCentralService {
        config: central_config.clone(),
    });

    // 初始化协议分析器(ProtocolAnalyzer)作为Actor
    let protocol_analyzer = gateway::gprs_server::ProtocolAnalyzer::new();
    let protocol_analyzer_addr = protocol_analyzer.start();

    // 初始化 TCP 会话管理器
    let tcp_session_manager =
        infrastructure::message_router::tcp_session::TcpSessionManager::default();
    let tcp_session_manager_addr = tcp_session_manager.start();

    // 初始化消息路由器
    info!("Initializing message router...");
    let message_router_instance = MessageRouter::new(tcp_session_manager_addr);
    let message_router_addr = message_router_instance.start();

    // 初始化 JT808 会话管理器
    info!("Initializing JT808 session manager...");
    let jt808_session_manager = crate::protocols::jt808::session::Jt808SessionManager::new();
    let jt808_session_manager_addr = jt808_session_manager.start();

    *message_router = Some(message_router_addr);

    Ok((
        protocol_analyzer_addr,
        Some(central_addr),
        jt808_session_manager_addr,
    ))
}

/// 初始化Truck Scale服务器
pub fn init_truck_scale_server(
    pool: Arc<sqlx::PgPool>,
    _central_config: &central::config::CentralConfig,
) {
    info!("Initializing Truck Scale protocol adapter...");
    let truck_scale_config = truck_scale::TruckScaleConfig::from_env().unwrap_or_default();
    let truck_scale_pool = pool.clone();
    tokio::spawn(async move {
        let mut truck_scale_server = 
            truck_scale::TruckScaleServer::new(truck_scale_config, truck_scale_pool);
        info!("Starting Truck Scale server on port 9809...");
        if let Err(e) = truck_scale_server.start().await {
            log::error!("Truck Scale server error: {}", e);
        }
    });
}






