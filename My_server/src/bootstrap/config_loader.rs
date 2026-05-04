//! /! 配置加载模块
//!
//! 统一加载应用配置

use log::info;
use std::sync::Arc;

/// 应用配置结构体 - 简化版
pub struct AppConfig {
    /// 数据库连接池
    pub pool: Arc<sqlx::PgPool>,
    /// 完整的应用配置
    pub config: crate::config::unified::UnifiedConfig,
    /// 中心服务配置
    pub central_config: crate::central::config::CentralConfig,
}

/// 加载应用配置
///
/// # 返回
/// 返回应用配置或错误信息
pub async fn load_config() -> Result<AppConfig, Box<dyn std::error::Error>> {
    info!("Loading application configuration...");

    // 初始化统一配置管理
    if let Err(e) = crate::config::unified::manager::init_config().await {
        return Err(format!("Failed to initialize config: {}", e).into());
    }

    // 获取配置
    let unified_config = crate::config::unified::manager::get_config();

    // 打印数据库连接配置,用于调试
    info!("Database connection URL: {}", unified_config.database.url);
    info!(
        "Database max connections: {}",
        unified_config.database.max_connections
    );
    info!(
        "Database min connections: {}",
        unified_config.database.min_connections
    );
    info!(
        "Database connect timeout: {}",
        unified_config.database.connect_timeout
    );

    // 构建AppConfig,使用配置的连接池参数
    let pool = unified_config
        .get_pool_config()
        .connect(&unified_config.database.url)
        .await
        .map_err(|e| format!("Failed to initialize database: {}", e))?;

    info!("Database pool created successfully");

    let app_config = AppConfig {
        pool: Arc::new(pool),
        config: unified_config.clone(),
        central_config: crate::central::config::CentralConfig::from_env().unwrap_or_default(),
    };

    info!("Configuration loaded successfully");
    Ok(app_config)
}
