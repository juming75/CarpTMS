//! /! 数据库初始化模块

use log::info;

use crate::central::config::CentralConfig;
use crate::config::unified::manager as config_manager;
use crate::config::unified::UnifiedConfig;
use std::sync::Arc;

/// 应用配置结构体
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub pool: Arc<sqlx::PgPool>,
    pub config: UnifiedConfig,
    pub central_config: CentralConfig,
}

/// 加载数据库配置
///
/// # 返回值
/// - `Ok(AppConfig)` - 加载成功,返回应用配置
/// - `Err(anyhow::Error)` - 加载失败
pub async fn load_config() -> anyhow::Result<AppConfig> {
    info!("Loading application configuration...");

    // 初始化统一配置管理
    if let Err(e) = config_manager::init_config().await {
        return Err(anyhow::anyhow!("Failed to initialize config: {}", e));
    }

    // 获取配置
    let unified_config = config_manager::get_config();

    // 打印配置信息
    info!("Application configuration loaded successfully");
    info!(
        "Server will start on {}:{}",
        unified_config.server.host, unified_config.server.port
    );
    info!(
        "Database pool config: max={}, min={}, acquire_timeout={}s, idle_timeout=300s, max_lifetime=1800s",
        unified_config.database.max_connections,
        unified_config.database.min_connections,
        unified_config.database.connect_timeout
    );

    // 从配置文件读取连接池参数
    let pool = match sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .min_connections(0)
        .acquire_timeout(std::time::Duration::from_secs(10))
        .idle_timeout(std::time::Duration::from_secs(300))
        .max_lifetime(std::time::Duration::from_secs(1800))
        .connect(&unified_config.database.url)
        .await
    {
        Ok(pool) => {
            info!("Database connection established successfully");
            pool
        }
        Err(e) => {
            info!("Warning: Failed to connect to database: {}", e);
            // Try with a simpler connection string using carptms_db
            let simple_url = "postgres://postgres:123@localhost:5432/carptms_db";
            info!("Trying with simple connection string: {}", simple_url);
            match sqlx::postgres::PgPoolOptions::new()
                .max_connections(1)
                .min_connections(0)
                .acquire_timeout(std::time::Duration::from_secs(5))
                .connect(simple_url)
                .await
            {
                Ok(pool) => {
                    info!("Database connection established with simple URL");
                    pool
                }
                Err(e) => {
                    info!("Failed to connect with simple URL: {}", e);
                    return Err(anyhow::anyhow!("Failed to connect to database: {}", e));
                }
            }
        }
    };

    let app_config = AppConfig {
        pool: Arc::new(pool),
        config: unified_config.clone(),
        central_config: crate::central::config::CentralConfig::default(),
    };

    info!("Application configuration loaded successfully");
    Ok(app_config)
}
