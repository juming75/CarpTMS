//! /! 应用初始化模块
//!
//! 负责初始化应用所需的各种组件和服务

pub mod database;
pub mod finalize;
pub mod monitoring;
pub mod redis;
pub mod servers;
pub mod services;

use ::redis::Client;
use log::{error, info, warn};
use std::sync::Arc;

use crate::init::database::AppConfig;

/// 初始化所有组件
///
/// # 返回值
/// - `Ok((AppConfig, Arc<Client>))` - 初始化成功,返回应用配置和Redis客户端
/// - `Err(std::io::Error)` - 初始化失败
pub async fn init_all() -> std::io::Result<(AppConfig, Arc<Client>)> {
    // 加载环境变量
    dotenv::dotenv().ok();

    // 初始化监控和追踪
    monitoring::init_monitoring();

    // 加载配置,即使数据库连接失败也会继续
    let app_config: AppConfig = match database::load_config().await {
        Ok(config) => config,
        Err(e) => {
            // 数据库连接失败,返回一个最小的 AppConfig
            warn!(
                "Failed to load configuration: {}, using minimal configuration",
                e
            );

            // 初始化统一配置
            if let Err(e) = crate::config::unified::manager::init_config().await {
                error!("Failed to initialize config: {}", e);
                return Err(std::io::Error::other(format!(
                    "Failed to initialize config: {}",
                    e
                )));
            }

            let unified_config = crate::config::unified::manager::get_config();

            // 创建一个空的连接池
            let pool = match sqlx::postgres::PgPool::connect(&unified_config.database.url).await {
                Ok(pool) => pool,
                Err(_) => {
                    // 数据库连接失败，返回错误
                    return Err(std::io::Error::other("Failed to connect to database"));
                }
            };

            AppConfig {
                pool: Arc::new(pool),
                config: unified_config,
                central_config: crate::central::config::CentralConfig::default(),
            }
        }
    };

    // 初始化 Redis 客户端
    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
    let redis_client = match Client::open(redis_url.as_str()) {
        Ok(client) => Arc::new(client),
        Err(e) => {
            warn!("Failed to create Redis client: {}, creating fallback", e);
            // 使用硬编码本地地址创建 fallback 客户端
            // Client::open 仅解析 URL，不发起连接
            let fallback = Client::open("redis://127.0.0.1:6379").map_err(|fe| {
                std::io::Error::other(format!(
                    "Redis unavailable: primary URL '{}' failed: {}, fallback also failed: {}",
                    redis_url, e, fe
                ))
            })?;
            warn!("Using fallback Redis client (127.0.0.1:6379)");
            Arc::new(fallback)
        }
    };

    info!("Application initialized successfully");
    Ok((app_config, redis_client))
}

/// 启动所有后台服务
///
/// # 参数
/// - `app_config` - 应用配置
/// - `redis_client` - Redis 客户端
pub async fn start_all_services(app_config: &AppConfig, _redis_client: &Arc<Client>) {
    info!("Starting all background services...");

    // 启动缓存预热
    services::start_cache_preheating(&app_config.pool).await;

    // 启动数据库连接池监控
    services::start_db_pool_monitoring(&app_config.pool).await;

    // 启动业务指标监控
    services::start_business_metrics_monitoring(&app_config.pool).await;

    // 启动告警服务
    services::start_alert_service(&app_config.pool).await;

    // 启动数据同步服务
    services::start_data_sync_service(&app_config.pool).await;

    // 启动性能监控服务
    services::start_performance_monitor(&app_config.pool).await;

    // 启动灾难恢复服务
    services::start_disaster_recovery_service(&app_config.pool).await;

    info!("All background services started");
}
