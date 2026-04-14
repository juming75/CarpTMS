use log::{debug, error, info, warn};
use redis::aio::MultiplexedConnection;
pub use redis::Client;
use redis::{AsyncCommands, RedisError};
use serde::{Deserialize, Serialize};
use std::env;
use std::time::Duration;
use tokio::sync::OnceCell;
use tokio::time::sleep;

// Redis连接单例
static REDIS_CLIENT: OnceCell<Client> = OnceCell::const_new();
static REDIS_CONNECTION: OnceCell<MultiplexedConnection> = OnceCell::const_new();

// 初始化Redis连接
pub async fn init_redis() -> Result<(), RedisError> {
    // 获取Redis连接字符串
    let redis_url =
        env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379/0".to_string());

    info!("Attempting to connect to Redis at: {}", redis_url);

    // 创建Redis客户端
    let client = Client::open(redis_url)?;

    // 尝试连接,最多重试3次
    let connection = connect_with_retry(&client, 3, Duration::from_secs(2)).await?;

    // 存储到单例中
    REDIS_CLIENT.set(client).ok();
    REDIS_CONNECTION.set(connection).ok();

    // 启动连接健康检查任务
    tokio::spawn(async move {
        health_check_task().await;
    });

    info!("Successfully connected to Redis!");
    Ok(())
}

// 带重试机制的连接方法
async fn connect_with_retry(
    client: &Client,
    max_retries: usize,
    retry_delay: Duration,
) -> Result<MultiplexedConnection, RedisError> {
    for attempt in 1..=max_retries {
        match client.get_multiplexed_tokio_connection().await {
            Ok(connection) => {
                // 测试连接
                if redis::cmd("PING")
                    .query_async::<()>(&mut connection.clone())
                    .await
                    .is_ok()
                {
                    return Ok(connection);
                }
            }
            Err(err) => {
                warn!("Redis connection attempt {} failed: {}", attempt, err);
                if attempt < max_retries {
                    sleep(retry_delay).await;
                } else {
                    return Err(err);
                }
            }
        }
    }
    Err(RedisError::from((
        redis::ErrorKind::IoError,
        "Failed to connect to Redis after multiple attempts",
    )))
}

// 健康检查任务
async fn health_check_task() {
    let check_interval = Duration::from_secs(30);

    loop {
        sleep(check_interval).await;

        if let Some(connection) = REDIS_CONNECTION.get() {
            match redis::cmd("PING")
                .query_async::<()>(&mut connection.clone())
                .await
            {
                Ok(_) => {
                    debug!("Redis connection is healthy");
                }
                Err(err) => {
                    error!("Redis connection health check failed: {}", err);
                    // 尝试重新连接
                    if let Some(client) = REDIS_CLIENT.get() {
                        match connect_with_retry(client, 3, Duration::from_secs(2)).await {
                            Ok(new_connection) => {
                                if REDIS_CONNECTION.set(new_connection).is_ok() {
                                    info!("Successfully reconnected to Redis");
                                }
                            }
                            Err(err) => {
                                error!("Failed to reconnect to Redis: {}", err);
                            }
                        }
                    }
                }
            }
        }
    }
}

// 获取Redis连接
pub async fn get_redis_connection() -> Option<&'static MultiplexedConnection> {
    REDIS_CONNECTION.get()
}

// 检查Redis连接是否可用
pub async fn is_redis_available() -> bool {
    match REDIS_CONNECTION.get() {
        Some(connection) => redis::cmd("PING")
            .query_async::<()>(&mut connection.clone())
            .await
            .is_ok(),
        None => false,
    }
}

// 强制重新连接
pub async fn reconnect_redis() -> Result<(), RedisError> {
    if let Some(client) = REDIS_CLIENT.get() {
        let connection = connect_with_retry(client, 3, Duration::from_secs(2)).await?;
        REDIS_CONNECTION.set(connection).ok();
        info!("Successfully reconnected to Redis");
        Ok(())
    } else {
        Err(RedisError::from((
            redis::ErrorKind::IoError,
            "Redis client not initialized",
        )))
    }
}

// 缓存数据到Redis
pub async fn set_cache<T>(key: &str, value: &T, expire_seconds: u64) -> Result<(), RedisError>
where
    T: Serialize,
{
    if !is_redis_available().await {
        return Ok(());
    }

    if let Some(connection) = get_redis_connection().await {
        let mut conn = connection.clone();

        let json = serde_json::to_string(value)?;
        let _: () = conn.set_ex(key, json, expire_seconds).await?;
    }
    Ok(())
}

// 缓存数据到Redis,带有动态过期策略
pub async fn set_cache_with_dynamic_ttl<T>(
    key: &str,
    value: &T,
    base_ttl: u64,
    access_factor: f64,
) -> Result<(), RedisError>
where
    T: Serialize,
{
    if !is_redis_available().await {
        return Ok(());
    }

    if let Some(connection) = get_redis_connection().await {
        let mut conn = connection.clone();

        // 检查键的访问次数
        let access_count: u64 = conn.get(format!("{}:access_count", key)).await.unwrap_or(0);

        // 根据访问次数动态调整TTL
        // 访问次数越多,TTL越长,最大为base_ttl的5倍
        let dynamic_ttl = std::cmp::min(
            base_ttl + (access_count as f64 * access_factor) as u64,
            base_ttl * 5,
        );

        let json = serde_json::to_string(value)?;
        let _: () = conn.set_ex(key, json, dynamic_ttl).await?;

        // 更新访问次数,设置过期时间为键的TTL的2倍
        let _: () = conn.incr(format!("{}:access_count", key), 1).await?;
        let _: () = conn
            .expire(format!("{}:access_count", key), (dynamic_ttl * 2) as i64)
            .await?;
    }

    Ok(())
}

// 从Redis获取缓存数据,并更新访问次数
pub async fn get_cache_with_access_tracking<T>(key: &str) -> Result<Option<T>, RedisError>
where
    T: for<'de> Deserialize<'de>,
{
    if !is_redis_available().await {
        return Ok(None);
    }

    if let Some(connection) = get_redis_connection().await {
        let mut conn = connection.clone();

        match conn.get::<&str, Option<String>>(key).await {
            Ok(Some(json)) => {
                // 更新访问时间戳,用于LRU策略
                let _: () = conn
                    .set(
                        format!("{}:last_access", key),
                        chrono::Utc::now().timestamp(),
                    )
                    .await?;

                let value: T = serde_json::from_str(&json)?;
                Ok(Some(value))
            }
            Ok(None) => Ok(None),
            Err(err) => Err(err),
        }
    } else {
        Ok(None)
    }
}

// 从Redis获取缓存数据
pub async fn get_cache<T>(key: &str) -> Result<Option<T>, RedisError>
where
    T: for<'de> Deserialize<'de>,
{
    if !is_redis_available().await {
        return Ok(None);
    }

    if let Some(connection) = get_redis_connection().await {
        let mut conn = connection.clone();

        match conn.get::<&str, Option<String>>(key).await {
            Ok(Some(json)) => {
                let value: T = serde_json::from_str(&json)?;
                Ok(Some(value))
            }
            Ok(None) => Ok(None),
            Err(err) => Err(err),
        }
    } else {
        Ok(None)
    }
}

// 删除Redis缓存
pub async fn del_cache(key: &str) -> Result<(), RedisError> {
    if !is_redis_available().await {
        return Ok(());
    }

    if let Some(connection) = get_redis_connection().await {
        let mut conn = connection.clone();

        let _: () = conn.del(key).await?;
    }
    Ok(())
}

// 批量删除Redis缓存
pub async fn del_cache_pattern(pattern: &str) -> Result<(), RedisError> {
    if !is_redis_available().await {
        return Ok(());
    }

    if let Some(connection) = get_redis_connection().await {
        let mut conn = connection.clone();

        let keys: Vec<String> = conn.keys(pattern).await?;
        if !keys.is_empty() {
            let _: () = conn.del(keys).await?;
        }
    }
    Ok(())
}
