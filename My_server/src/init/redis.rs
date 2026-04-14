//! /! Redis初始化模块

use log::{info, warn};
use redis::Client as RedisClient;
use std::sync::Arc;

/// 初始化Redis客户端
///
/// # 返回值
/// - `Arc<RedisClient>` - Redis客户端
pub async fn init_redis_client() -> Arc<RedisClient> {
    info!("Initializing Redis client...");

    let redis_client: Arc<RedisClient> = std::env::var("REDIS_URL")
        .ok()
        .and_then(|url| RedisClient::open(url).ok())
        .or_else(|| RedisClient::open("redis://localhost:6379/0").ok())
        .map(Arc::new)
        .unwrap_or_else(|| {
            // 如果无法创建Redis客户端,仍然创建一个占位符
            warn!("Failed to create Redis client, using placeholder");
            Arc::new(
                RedisClient::open("redis://localhost:6379/0")
                    .expect("Failed to create placeholder Redis client"),
            )
        });

    info!("Redis client initialized successfully");
    redis_client
}

/// 初始化Redis连接池
///
/// # 返回值
/// - `Ok(())` - 初始化成功
/// - `Err(redis::RedisError)` - 初始化失败
pub async fn init_redis_pool() -> Result<(), redis::RedisError> {
    info!("Initializing Redis connection pool...");

    let result = crate::redis::init_redis().await;

    match result {
        Ok(_) => {
            info!("Redis connection pool initialized successfully");
            Ok(())
        }
        Err(e) => {
            warn!("Failed to initialize Redis connection pool: {}", e);
            Err(e)
        }
    }
}
