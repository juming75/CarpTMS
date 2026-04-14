//! / 缓存模块
// 提供多级缓存实现,包括内存缓存和Redis缓存

mod memory_cache;
mod redis_cache;
mod user_cache;
pub mod vehicle_cache;
mod optimization;

pub use memory_cache::MemoryCache;
pub use redis_cache::RedisCache;
pub use user_cache::UserCache;
pub use vehicle_cache::VehicleCache;
pub use optimization::{EnhancedCacheManager, CacheWarmer, EnhancedCacheStats};

use std::time::Duration;

/// 缓存键类型
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CacheKey {
    /// 车辆信息缓存键
    Vehicle(String),
    /// 设备信息缓存键
    Device(String),
    /// 用户信息缓存键
    User(String),
    /// 订单信息缓存键
    Order(String),
    /// 位置信息缓存键
    Location(String),
    /// 系统配置缓存键
    Config(String),
    /// 其他缓存键
    Other(String),
}

impl CacheKey {
    /// 创建车辆信息缓存键
    pub fn vehicle(id: String) -> Self {
        CacheKey::Vehicle(id)
    }

    /// 创建设备信息缓存键
    pub fn device(id: String) -> Self {
        CacheKey::Device(id)
    }

    /// 创建用户信息缓存键
    pub fn user(id: String) -> Self {
        CacheKey::User(id)
    }

    /// 创建订单信息缓存键
    pub fn order(id: String) -> Self {
        CacheKey::Order(id)
    }

    /// 创建位置信息缓存键
    pub fn location(id: String) -> Self {
        CacheKey::Location(id)
    }

    /// 创建系统配置缓存键
    pub fn config(key: String) -> Self {
        CacheKey::Config(key)
    }

    /// 创建其他缓存键
    pub fn other(key: String) -> Self {
        CacheKey::Other(key)
    }
}

use std::fmt;

impl fmt::Display for CacheKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CacheKey::Vehicle(id) => write!(f, "vehicle:{}", id),
            CacheKey::Device(id) => write!(f, "device:{}", id),
            CacheKey::User(id) => write!(f, "user:{}", id),
            CacheKey::Order(id) => write!(f, "order:{}", id),
            CacheKey::Location(id) => write!(f, "location:{}", id),
            CacheKey::Config(key) => write!(f, "config:{}", key),
            CacheKey::Other(key) => write!(f, "{}", key),
        }
    }
}

/// 缓存管理器
/// 实现多级缓存策略:内存缓存 -> Redis缓存
pub struct CacheManager {
    memory_cache: MemoryCache<Vec<u8>>,
    redis_cache: tokio::sync::Mutex<Option<RedisCache>>,
}

impl CacheManager {
    /// 创建新的缓存管理器实例
    pub fn new(memory_cache: MemoryCache<Vec<u8>>, redis_cache: RedisCache) -> Self {
        Self {
            memory_cache,
            redis_cache: tokio::sync::Mutex::new(Some(redis_cache)),
        }
    }

    /// 从环境变量创建缓存管理器 (需要异步初始化)
    pub async fn from_env() -> Result<Self, anyhow::Error> {
        let memory_cache = MemoryCache::default();
        let redis_cache = RedisCache::from_env().await?;
        Ok(Self::new(memory_cache, redis_cache))
    }

    /// 获取缓存项
    /// 首先从内存缓存中获取,若不存在则从Redis缓存中获取,并更新内存缓存
    pub async fn get(&self, key: &str) -> Result<Option<Vec<u8>>, anyhow::Error> {
        // 首先从内存缓存中获取
        if let Some(value) = self.memory_cache.get(key) {
            return Ok(Some(value));
        }

        // 从Redis缓存中获取
        if let Some(redis_cache) = self.redis_cache.lock().await.as_mut() {
            if let Some(value) = redis_cache.get(key).await? {
                // 更新内存缓存
                self.memory_cache
                    .set(key, value.clone(), Some(Duration::from_secs(300)));
                return Ok(Some(value));
            }
        }

        Ok(None)
    }

    /// 批量获取缓存项
    pub async fn mget(&self, keys: &[&str]) -> Result<Vec<Option<Vec<u8>>>, anyhow::Error> {
        // 首先从内存缓存中获取
        let mut keys_to_get_from_redis = Vec::new();
        let mut memory_results: Vec<Option<Vec<u8>>> = vec![None; keys.len()];

        for (i, key) in keys.iter().enumerate() {
            if let Some(value) = self.memory_cache.get(key) {
                memory_results[i] = Some(value);
            } else {
                keys_to_get_from_redis.push((*key, i));
            }
        }

        // 从Redis缓存中获取剩余的键
        if !keys_to_get_from_redis.is_empty() {
            if let Some(redis_cache) = self.redis_cache.lock().await.as_mut() {
                let redis_keys: Vec<&str> =
                    keys_to_get_from_redis.iter().map(|(key, _)| *key).collect();
                let redis_results = redis_cache.mget(&redis_keys).await?;

                for (i, (key, index)) in keys_to_get_from_redis.iter().enumerate() {
                    if let Some(value) = redis_results[i].as_ref() {
                        memory_results[*index] = Some(value.clone());
                        // 更新内存缓存
                        self.memory_cache
                            .set(key, value.clone(), Some(Duration::from_secs(300)));
                    }
                }
            }
        }

        Ok(memory_results)
    }

    /// 设置缓存项
    /// 同时更新内存缓存和Redis缓存
    pub async fn set(
        &self,
        key: &str,
        value: &[u8],
        ttl: Option<Duration>,
    ) -> Result<(), anyhow::Error> {
        // 设置内存缓存
        self.memory_cache.set(key, value.to_vec(), ttl);

        // 设置Redis缓存
        if let Some(redis_cache) = self.redis_cache.lock().await.as_mut() {
            redis_cache.set(key, value, ttl).await?;
        }

        Ok(())
    }

    /// 删除缓存项
    /// 同时删除内存缓存和Redis缓存中的项
    pub async fn del(&self, key: &str) -> Result<(), anyhow::Error> {
        // 删除内存缓存
        self.memory_cache.del(key);

        // 删除Redis缓存
        if let Some(redis_cache) = self.redis_cache.lock().await.as_mut() {
            redis_cache.del(key).await?;
        }

        Ok(())
    }

    /// 删除匹配模式的缓存项
    /// 同时删除内存缓存和Redis缓存中的匹配项
    pub async fn del_pattern(&self, pattern: &str) -> Result<(), anyhow::Error> {
        // 删除内存缓存中的匹配项
        self.memory_cache.del_pattern(pattern);

        // 删除Redis缓存中的匹配项
        if let Some(redis_cache) = self.redis_cache.lock().await.as_mut() {
            redis_cache.del_pattern(pattern).await?;
        }

        Ok(())
    }

    /// 清空所有缓存
    /// 同时清空内存缓存和Redis缓存
    pub async fn flush_all(&self) -> Result<(), anyhow::Error> {
        // 清空内存缓存
        self.memory_cache.flush_all();

        // 清空Redis缓存
        if let Some(redis_cache) = self.redis_cache.lock().await.as_mut() {
            redis_cache.flush_all().await?;
        }

        Ok(())
    }

    /// 获取缓存统计信息
    pub fn get_stats(&self) -> CacheStats {
        CacheStats {
            memory_cache_size: self.memory_cache.size(),
        }
    }
}

/// 缓存统计信息
#[derive(Debug, Clone)]
pub struct CacheStats {
    /// 内存缓存大小
    pub memory_cache_size: usize,
}

/// 缓存管理器的默认实现
impl Default for CacheManager {
    fn default() -> Self {
        Self {
            memory_cache: MemoryCache::default(),
            redis_cache: tokio::sync::Mutex::new(None),
        }
    }
}
