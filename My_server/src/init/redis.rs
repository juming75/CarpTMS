//! /! Redis初始化模块

use log::{info, warn};
use redis::Client as RedisClient;
use std::sync::Arc;

/// 初始化Redis客户端
///
/// # 返回值
/// - `Some(Arc<RedisClient>)` - Redis客户端成功创建
/// - `None` - 无法创建Redis客户端（服务以降级模式运行）
pub async fn init_redis_client() -> Option<Arc<RedisClient>> {
    info!("Initializing Redis client...");

    let redis_client = std::env::var("REDIS_URL")
        .ok()
        .and_then(|url| RedisClient::open(url).ok())
        .or_else(|| {
            warn!("REDIS_URL not set or invalid, trying localhost:6379");
            RedisClient::open("redis://localhost:6379/0").ok()
        })
        .map(Arc::new);

    if redis_client.is_some() {
        info!("Redis client initialized successfully");
    } else {
        warn!("Failed to create Redis client, service will run without Redis cache");
    }
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
