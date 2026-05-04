//! / Redis缓存模块
// 提供基于Redis的缓存实现,作为多级缓存的第二级
// 使用异步 ConnectionManager 避免阻塞 tokio worker 线程

use anyhow::{anyhow, Result};
use redis::aio::ConnectionManager;
use redis::{AsyncCommands, RedisResult};
use std::time::Duration;

/// Redis缓存实现 (异步版本)
/// 使用 ConnectionManager 管理异步连接，线程安全且可 Clone
#[derive(Clone)]
pub struct RedisCache {
    conn_manager: ConnectionManager,
}

impl std::fmt::Debug for RedisCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RedisCache")
            .field("conn_manager", &"<ConnectionManager>")
            .finish()
    }
}

impl RedisCache {
    /// 创建新的Redis缓存实例
    /// 使用异步 ConnectionManager，不会阻塞 tokio runtime
    pub async fn new(redis_url: &str) -> Result<Self> {
        let client = redis::Client::open(redis_url)
            .map_err(|e| anyhow!("Failed to create Redis client: {}", e))?;

        let conn_manager = client
            .get_connection_manager()
            .await
            .map_err(|e| anyhow!("Failed to create Redis connection manager: {}", e))?;

        Ok(Self { conn_manager })
    }

    /// 从环境变量创建Redis缓存实例 (异步版本)
    pub async fn from_env() -> Result<Self> {
        let redis_url =
            std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());
        Self::new(&redis_url).await
    }

    /// 获取缓存项 (真正的异步操作)
    pub async fn get(&mut self, key: &str) -> Result<Option<Vec<u8>>> {
        let result: RedisResult<Option<Vec<u8>>> = self.conn_manager.get(key).await;
        match result {
            Ok(value) => Ok(value),
            Err(e) => Err(anyhow!("Failed to get cache: {}", e)),
        }
    }

    /// 批量获取缓存项 (真正的异步操作)
    pub async fn mget(&mut self, keys: &[&str]) -> Result<Vec<Option<Vec<u8>>>> {
        if keys.is_empty() {
            return Ok(Vec::new());
        }

        let result: RedisResult<Vec<Option<Vec<u8>>>> = self.conn_manager.get(keys).await;
        match result {
            Ok(values) => Ok(values),
            Err(e) => Err(anyhow!("Failed to mget cache: {}", e)),
        }
    }

    /// 设置缓存项 (真正的异步操作)
    /// P1-5优化: 同时更新版本号
    pub async fn set(&mut self, key: &str, value: &[u8], ttl: Option<Duration>) -> Result<()> {
        if let Some(ttl) = ttl {
            let ttl_seconds = ttl.as_secs();
            let result: RedisResult<()> = self.conn_manager.set_ex(key, value, ttl_seconds).await;
            match result {
                Ok(_) => {
                    // P1-5优化: 自动递增版本号
                    let version_key = format!("{}:version", key);
                    let _: RedisResult<()> = self.conn_manager.incr(&version_key, 1u64).await;
                    Ok(())
                }
                Err(e) => Err(anyhow!("Failed to set cache with ttl: {}", e)),
            }
        } else {
            let result: RedisResult<()> = self.conn_manager.set(key, value).await;
            match result {
                Ok(_) => {
                    // P1-5优化: 自动递增版本号
                    let version_key = format!("{}:version", key);
                    let _: RedisResult<()> = self.conn_manager.incr(&version_key, 1u64).await;
                    Ok(())
                }
                Err(e) => Err(anyhow!("Failed to set cache: {}", e)),
            }
        }
    }

    /// 删除缓存项 (真正的异步操作)
    pub async fn del(&mut self, key: &str) -> Result<()> {
        let result: RedisResult<()> = self.conn_manager.del(key).await;
        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(anyhow!("Failed to delete cache: {}", e)),
        }
    }

    /// 删除匹配模式的缓存项 (真正的异步操作)
    pub async fn del_pattern(&mut self, pattern: &str) -> Result<()> {
        let keys: Vec<String> = self.conn_manager.keys(pattern).await?;

        if !keys.is_empty() {
            let result: RedisResult<()> = self.conn_manager.del(&keys[..]).await;
            match result {
                Ok(_) => Ok(()),
                Err(e) => Err(anyhow!("Failed to delete pattern cache: {}", e)),
            }
        } else {
            Ok(())
        }
    }

    /// 清空所有缓存 (真正的异步操作)
    pub async fn flush_all(&mut self) -> Result<()> {
        let result: RedisResult<()> = redis::cmd("FLUSHALL")
            .query_async(&mut self.conn_manager)
            .await;
        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(anyhow!("Failed to flush all cache: {}", e)),
        }
    }

    /// P1-5优化: 获取缓存项版本号
    /// 版本号存储在 key:version 的独立key中
    pub async fn get_version(&mut self, key: &str) -> Result<u64> {
        let version_key = format!("{}:version", key);
        let result: RedisResult<Option<u64>> = self.conn_manager.get(&version_key).await;
        match result {
            Ok(Some(v)) => Ok(v),
            Ok(None) => Ok(0), // 默认版本0表示未设置
            Err(e) => {
                log::debug!("获取缓存版本失败: {}", e);
                Ok(0)
            }
        }
    }

    /// P1-5优化: 设置缓存项版本号
    pub async fn set_version(&mut self, key: &str, version: u64) -> Result<()> {
        let version_key = format!("{}:version", key);
        let result: RedisResult<()> = self.conn_manager.set_ex(&version_key, version, 86400).await;
        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(anyhow!("Failed to set cache version: {}", e)),
        }
    }
}
