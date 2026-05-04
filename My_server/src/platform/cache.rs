//! Unified Cache Abstraction Layer
//!
//! Provides a unified interface for different cache providers including:
//! - In-memory cache
//! - Redis cache
//! - Distributed cache

use async_trait::async_trait;
use redis::{aio::ConnectionManager, AsyncCommands, Client};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use tokio::sync::RwLock;
use std::collections::HashMap;

/// Cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    pub provider: CacheProviderType,
    pub redis_url: Option<String>,
    pub max_memory_size: usize,
    pub default_ttl: Duration,
    pub cleanup_interval: Duration,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            provider: CacheProviderType::Memory,
            redis_url: None,
            max_memory_size: 100 * 1024 * 1024, // 100MB
            default_ttl: Duration::from_secs(3600), // 1 hour
            cleanup_interval: Duration::from_secs(300), // 5 minutes
        }
    }
}

/// Cache provider types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CacheProviderType {
    Memory,
    Redis,
    Distributed,
}

/// Cache error types
#[derive(Error, Debug)]
pub enum CacheError {
    #[error("Cache provider error: {0}")]
    Provider(String),
    
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    #[error("Deserialization error: {0}")]
    Deserialization(String),
    
    #[error("Connection error: {0}")]
    Connection(String),
    
    #[error("Key not found")]
    NotFound,
}

/// Cache provider trait
#[async_trait]
pub trait CacheProvider: Send + Sync {
    /// Get a value from cache
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>, CacheError>;
    
    /// Set a value in cache with optional TTL
    async fn set(&self, key: &str, value: &[u8], ttl: Option<Duration>) -> Result<(), CacheError>;
    
    /// Delete a value from cache
    async fn delete(&self, key: &str) -> Result<(), CacheError>;
    
    /// Clear all values from cache
    async fn clear(&self) -> Result<(), CacheError>;
    
    /// Check if a key exists
    async fn exists(&self, key: &str) -> Result<bool, CacheError>;
}

/// In-memory cache provider
pub struct MemoryCacheProvider {
    cache: Arc<RwLock<HashMap<String, (Vec<u8>, Option<std::time::Instant>)>>>,
    max_size: usize,
}

impl MemoryCacheProvider {
    pub fn new(max_size: usize) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            max_size,
        }
    }
}

#[async_trait]
impl CacheProvider for MemoryCacheProvider {
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>, CacheError> {
        let mut cache = self.cache.write().await;
        
        if let Some((value, expiry)) = cache.get_mut(key) {
            if let Some(expiry_time) = expiry {
                if expiry_time.elapsed() > std::time::Duration::from_secs(0) {
                    cache.remove(key);
                    return Ok(None);
                }
            }
            return Ok(Some(value.clone()));
        }
        
        Ok(None)
    }
    
    async fn set(&self, key: &str, value: &[u8], ttl: Option<Duration>) -> Result<(), CacheError> {
        let mut cache = self.cache.write().await;
        
        // Simple size management - remove oldest entries if needed
        if cache.len() >= self.max_size {
            // Remove a random key (in production, use LRU or similar)
            if let Some(old_key) = cache.keys().next().cloned() {
                cache.remove(&old_key);
            }
        }
        
        let expiry = ttl.map(|duration| std::time::Instant::now() + duration);
        cache.insert(key.to_string(), (value.to_vec(), expiry));
        Ok(())
    }
    
    async fn delete(&self, key: &str) -> Result<(), CacheError> {
        let mut cache = self.cache.write().await;
        cache.remove(key);
        Ok(())
    }
    
    async fn clear(&self) -> Result<(), CacheError> {
        let mut cache = self.cache.write().await;
        cache.clear();
        Ok(())
    }
    
    async fn exists(&self, key: &str) -> Result<bool, CacheError> {
        let cache = self.cache.read().await;
        Ok(cache.contains_key(key))
    }
}

/// Redis cache provider
pub struct RedisCacheProvider {
    connection: ConnectionManager,
}

impl RedisCacheProvider {
    pub async fn new(redis_url: &str) -> Result<Self, CacheError> {
        let client = Client::open(redis_url)
            .map_err(|e| CacheError::Connection(e.to_string()))?;
        let connection = client
            .get_connection_manager()
            .await
            .map_err(|e| CacheError::Connection(e.to_string()))?;
        
        Ok(Self { connection })
    }
}

#[async_trait]
impl CacheProvider for RedisCacheProvider {
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>, CacheError> {
        let mut conn = self.connection.clone();
        let result: Option<Vec<u8>> = conn
            .get(key)
            .await
            .map_err(|e| CacheError::Provider(e.to_string()))?;
        
        Ok(result)
    }
    
    async fn set(&self, key: &str, value: &[u8], ttl: Option<Duration>) -> Result<(), CacheError> {
        let mut conn = self.connection.clone();
        
        if let Some(ttl_duration) = ttl {
            conn
                .set_ex(key, value, ttl_duration.as_secs() as usize)
                .await
                .map_err(|e| CacheError::Provider(e.to_string()))?;
        } else {
            conn
                .set(key, value)
                .await
                .map_err(|e| CacheError::Provider(e.to_string()))?;
        }
        
        Ok(())
    }
    
    async fn delete(&self, key: &str) -> Result<(), CacheError> {
        let mut conn = self.connection.clone();
        conn
            .del(key)
            .await
            .map_err(|e| CacheError::Provider(e.to_string()))?;
        Ok(())
    }
    
    async fn clear(&self) -> Result<(), CacheError> {
        let mut conn = self.connection.clone();
        conn
            .flushdb()
            .await
            .map_err(|e| CacheError::Provider(e.to_string()))?;
        Ok(())
    }
    
    async fn exists(&self, key: &str) -> Result<bool, CacheError> {
        let mut conn = self.connection.clone();
        let exists: bool = conn
            .exists(key)
            .await
            .map_err(|e| CacheError::Provider(e.to_string()))?;
        
        Ok(exists)
    }
}

/// Cache manager that provides a unified interface
pub struct CacheManager {
    config: CacheConfig,
    provider: Arc<dyn CacheProvider>,
}

impl CacheManager {
    /// Create a new cache manager with the given configuration
    pub async fn new(config: Arc<crate::platform::config::ConfigManager>) -> Result<Self, CacheError> {
        let cache_config = config
            .get::<CacheConfig>("cache")
            .await
            .unwrap_or_default();
        
        let provider: Arc<dyn CacheProvider> = match cache_config.provider {
            CacheProviderType::Memory => {
                Arc::new(MemoryCacheProvider::new(cache_config.max_memory_size))
            }
            CacheProviderType::Redis => {
                if let Some(redis_url) = &cache_config.redis_url {
                    Arc::new(RedisCacheProvider::new(redis_url).await?)
                } else {
                    return Err(CacheError::Configuration("Redis URL not provided".to_string()));
                }
            }
            CacheProviderType::Distributed => {
                // For now, use Redis as distributed cache
                if let Some(redis_url) = &cache_config.redis_url {
                    Arc::new(RedisCacheProvider::new(redis_url).await?)
                } else {
                    return Err(CacheError::Configuration("Redis URL not provided for distributed cache".to_string()));
                }
            }
        };
        
        Ok(Self {
            config: cache_config,
            provider,
        })
    }
    
    /// Get a typed value from cache
    pub async fn get<T: serde::de::DeserializeOwned>(&self, key: &str) -> Result<Option<T>, CacheError> {
        if let Some(data) = self.provider.get(key).await? {
            let value = bincode::deserialize(&data)
                .map_err(|e| CacheError::Deserialization(e.to_string()))?;
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }
    
    /// Set a typed value in cache with optional TTL
    pub async fn set<T: serde::Serialize>(
        &self,
        key: &str,
        value: &T,
        ttl: Option<Duration>,
    ) -> Result<(), CacheError> {
        let data = bincode::serialize(value)
            .map_err(|e| CacheError::Serialization(e.to_string()))?;
        
        let final_ttl = ttl.or(Some(self.config.default_ttl));
        self.provider.set(key, &data, final_ttl).await
    }
    
    /// Delete a value from cache
    pub async fn delete(&self, key: &str) -> Result<(), CacheError> {
        self.provider.delete(key).await
    }
    
    /// Clear all values from cache
    pub async fn clear(&self) -> Result<(), CacheError> {
        self.provider.clear().await
    }
    
    /// Check if a key exists
    pub async fn exists(&self, key: &str) -> Result<bool, CacheError> {
        self.provider.exists(key).await
    }
    
    /// Get or insert a value with a factory function
    pub async fn get_or_insert_with<T, F>(
        &self,
        key: &str,
        f: F,
        ttl: Option<Duration>,
    ) -> Result<T, CacheError>
    where
        T: serde::Serialize + serde::de::DeserializeOwned,
        F: FnOnce() -> Result<T, CacheError>,
    {
        if let Some(value) = self.get::<T>(key).await? {
            return Ok(value);
        }
        
        let value = f()?;
        self.set(key, &value, ttl).await?;
        Ok(value)
    }
}