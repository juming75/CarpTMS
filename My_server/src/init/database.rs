//! /! 数据库初始化模块

use log::{info, warn};

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

    // 从配置文件读取连接池参数，并添加lc_messages选项来防止GBK编码错误
    let url = if unified_config.database.url.contains('?') {
        format!("{}&options=-c+lc_messages=C", unified_config.database.url)
    } else {
        format!("{}?options=-c+lc_messages=C", unified_config.database.url)
    };
    
    let pool = match sqlx::postgres::PgPoolOptions::new()
        .max_connections(unified_config.database.max_connections)
        .min_connections(unified_config.database.min_connections)
        .acquire_timeout(std::time::Duration::from_secs(
            unified_config.database.connect_timeout,
        ))
        .idle_timeout(std::time::Duration::from_secs(std::cmp::min(
            unified_config.database.connect_timeout.max(300),
            600,
        )))
        .max_lifetime(std::time::Duration::from_secs(1800))
        .connect(&url)
        .await
    {
        Ok(pool) => {
            info!("Database connection established successfully");
            pool
        }
        Err(e) => {
            info!("Warning: Failed to connect to database: {}", e);
            info!("Attempting fallback via DATABASE_URL environment variable...");

            let fallback_url = std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| {
                    info!("DATABASE_URL not set, using dev-only default (no password)");
                    "postgres://postgres@localhost:5432/CarpTMS_db".to_string()
                });

            let fallback_url = if fallback_url.contains('?') {
                format!("{}&options=-c+lc_messages=C", fallback_url)
            } else {
                format!("{}?options=-c+lc_messages=C", fallback_url)
            };

            info!("Trying with fallback connection string: {}:****@{}",
                fallback_url.split('@').next().unwrap_or("unknown"),
                fallback_url.split('@').nth(1).unwrap_or("unknown")
            );

            match sqlx::postgres::PgPoolOptions::new()
                .max_connections(1)
                .min_connections(0)
                .acquire_timeout(std::time::Duration::from_secs(5))
                .connect(&fallback_url)
                .await
            {
                Ok(pool) => {
                    info!("Database connection established with fallback URL");
                    pool
                }
                Err(e) => {
                    info!("Failed to connect with fallback URL: {}", e);
                    return Err(anyhow::anyhow!(
                        "Failed to connect to database. Set DATABASE_URL env var or check config/unified.toml"
                    ));
                }
            }
        }
    };

    let app_config = AppConfig {
        pool: Arc::new(pool),
        config: unified_config.clone(),
        central_config: crate::central::config::CentralConfig::default(),
    };

    // 执行数据库迁移
    run_migrations(&app_config.pool).await?;

    info!("Application configuration loaded successfully");
    Ok(app_config)
}

/// 执行数据库迁移
async fn run_migrations(pool: &Arc<sqlx::PgPool>) -> anyhow::Result<()> {
    info!("Running database migrations...");

    // 检查并添加 password_changed_at 字段
    let check_result = sqlx::query_scalar::<_, Option<bool>>(
        "SELECT EXISTS(
            SELECT 1 FROM information_schema.columns 
            WHERE table_name = 'users' AND column_name = 'password_changed_at'
        )"
    )
    .fetch_one(pool.as_ref())
    .await;

    match check_result {
        Ok(Some(true)) => {
            info!("password_changed_at column already exists");
        }
        Ok(Some(false)) | Ok(None) => {
            info!("Adding password_changed_at column...");
            // 添加列
            sqlx::query("ALTER TABLE users ADD COLUMN IF NOT EXISTS password_changed_at TIMESTAMP")
                .execute(pool.as_ref())
                .await?;
            // 创建索引
            sqlx::query("CREATE INDEX IF NOT EXISTS idx_users_password_changed_at ON users(password_changed_at)")
                .execute(pool.as_ref())
                .await?;
            info!("Successfully added password_changed_at column");
        }
        Err(e) => {
            warn!("Could not check for password_changed_at column: {}", e);
        }
    }

    Ok(())
}
