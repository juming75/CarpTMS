//! / 实时数据缓存
// 设备实时位置、状态、传感器数据缓存

use actix::prelude::*;
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};

use crate::infrastructure::redis::RedisPool;

/// 缓存键类型
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum CacheKey {
    /// 设备位置数据
    DeviceLocation(String),
    /// 设备状态数据
    DeviceStatus(String),
    /// 传感器数据
    SensorData(String),
    /// 车辆状态
    VehicleState(String),
}

impl CacheKey {
    pub fn to_redis_key(&self) -> String {
        match self {
            CacheKey::DeviceLocation(device_id) => format!("device:{}:location", device_id),
            CacheKey::DeviceStatus(device_id) => format!("device:{}:status", device_id),
            CacheKey::SensorData(device_id) => format!("device:{}:sensors", device_id),
            CacheKey::VehicleState(device_id) => format!("device:{}:vehicle", device_id),
        }
    }

    /// 缓存过期时间(秒)
    pub fn ttl(&self) -> u64 {
        match self {
            CacheKey::DeviceLocation(_) => 300,      // 5分钟
            CacheKey::DeviceStatus(_) => 600,         // 10分钟
            CacheKey::SensorData(_) => 60,            // 1分钟
            CacheKey::VehicleState(_) => 600,         // 10分钟
        }
    }
}

/// 缓存值类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CacheValue {
    /// 位置数据
    Location(LocationData),
    /// 设备状态
    DeviceStatus(DeviceStatusData),
    /// 传感器数据
    SensorData(SensorData),
    /// 车辆状态
    VehicleState(VehicleStateData),
}

/// 位置数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationData {
    pub device_id: String,
    pub latitude: f64,
    pub longitude: f64,
    pub altitude: Option<f32>,
    pub speed: Option<f32>,
    pub direction: Option<i32>,
    pub timestamp: DateTime<Utc>,
    pub address: Option<String>,
}

/// 设备状态数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceStatusData {
    pub device_id: String,
    pub online: bool,
    pub auth_status: String,
    pub last_activity: DateTime<Utc>,
    pub heartbeat_time: Option<DateTime<Utc>>,
    pub signal_strength: Option<i32>,
    pub battery_level: Option<i32>,
}

/// 传感器数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorData {
    pub device_id: String,
    pub sensors: HashMap<String, f64>,
    pub timestamp: DateTime<Utc>,
}

/// 车辆状态数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VehicleStateData {
    pub device_id: String,
    pub ignition: bool,
    pub acc: bool,
    pub door_open: bool,
    pub alarm: bool,
    pub mileage: Option<f64>,
    pub fuel: Option<f32>,
    pub temperature: Option<f32>,
    pub timestamp: DateTime<Utc>,
}

/// 实时缓存
pub struct RealtimeCache {
    redis_pool: Arc<RedisPool>,
    local_cache: Arc<RwLock<HashMap<String, (CacheValue, DateTime<Utc>)>>>,
    max_local_entries: usize,
    local_ttl: u64, // 本地缓存过期时间(秒)
}

impl RealtimeCache {
    /// 创建新的实时缓存
    pub fn new(redis_pool: Arc<RedisPool>) -> Self {
        info!("Creating realtime cache with Redis backend");
        
        Self {
            redis_pool,
            local_cache: Arc::new(RwLock::new(HashMap::new())),
            max_local_entries: 10000,
            local_ttl: 60, // 本地缓存 1 分钟
        }
    }

    /// 设置缓存值
    pub async fn set(&self, key: CacheKey, value: CacheValue) -> Result<(), String> {
        let redis_key = key.to_redis_key();
        let ttl = key.ttl();

        // 序列化数据
        let value_json = serde_json::to_string(&value)
            .map_err(|e| format!("Serialization error: {}", e))?;

        // 写入 Redis
        if let Some(mut conn) = self.redis_pool.get().await {
            let _: () = redis::cmd("SETEX")
                .arg(&redis_key)
                .arg(ttl)
                .arg(&value_json)
                .query(&mut conn)
                .map_err(|e| format!("Redis error: {}", e))?;

            debug!("Cache set: {} (TTL: {}s)", redis_key, ttl);
        }

        // 更新本地缓存
        let mut cache = self.local_cache.write().await;
        
        // 清理过期条目
        self.cleanup_local_cache(&mut cache).await;

        // 添加新条目
        cache.insert(
            redis_key.clone(),
            (value.clone(), Utc::now()),
        );

        // 检查缓存大小
        if cache.len() > self.max_local_entries {
            // 简单的 LRU: 删除最旧的 10%
            let mut entries: Vec<_> = cache.iter().collect();
            entries.sort_by_key(|(_, (_, time))| *time);
            
            let remove_count = cache.len() / 10;
            for (key, _) in entries.iter().take(remove_count) {
                cache.remove(*key);
            }
            
            debug!("Local cache trimmed: {} entries removed", remove_count);
        }

        Ok(())
    }

    /// 获取缓存值
    pub async fn get(&self, key: &CacheKey) -> Option<CacheValue> {
        let redis_key = key.to_redis_key();

        // 先查本地缓存
        {
            let cache = self.local_cache.read().await;
            if let Some((value, timestamp)) = cache.get(&redis_key) {
                // 检查本地缓存是否过期
                if Utc::now().signed_duration_since(*timestamp).num_seconds() < self.local_ttl as i64 {
                    debug!("Cache hit (local): {}", redis_key);
                    return Some(value.clone());
                }
            }
        }

        // 查 Redis
        if let Some(mut conn) = self.redis_pool.get().await {
            let result: Option<String> = redis::cmd("GET")
                .arg(&redis_key)
                .query(&mut conn)
                .unwrap_or(None);

            if let Some(value_json) = result {
                debug!("Cache hit (Redis): {}", redis_key);
                
                match serde_json::from_str::<CacheValue>(&value_json) {
                    Ok(value) => {
                        // 更新本地缓存
                        let mut cache = self.local_cache.write().await;
                        cache.insert(redis_key.clone(), (value.clone(), Utc::now()));
                        return Some(value);
                    }
                    Err(e) => {
                        error!("Failed to deserialize cache value for {}: {}", redis_key, e);
                    }
                }
            }
        }

        debug!("Cache miss: {}", redis_key);
        None
    }

    /// 删除缓存
    pub async fn delete(&self, key: &CacheKey) -> Result<(), String> {
        let redis_key = key.to_redis_key();

        // 删除 Redis
        if let Some(mut conn) = self.redis_pool.get().await {
            let _: () = redis::cmd("DEL")
                .arg(&redis_key)
                .query(&mut conn)
                .map_err(|e| format!("Redis error: {}", e))?;
        }

        // 删除本地缓存
        let mut cache = self.local_cache.write().await;
        cache.remove(&redis_key);

        debug!("Cache deleted: {}", redis_key);
        Ok(())
    }

    /// 清空指定设备的所有缓存
    pub async fn clear_device(&self, device_id: &str) -> Result<(), String> {
        let keys = vec![
            CacheKey::DeviceLocation(device_id.to_string()),
            CacheKey::DeviceStatus(device_id.to_string()),
            CacheKey::SensorData(device_id.to_string()),
            CacheKey::VehicleState(device_id.to_string()),
        ];

        for key in keys {
            self.delete(&key).await?;
        }

        info!("All cache cleared for device {}", device_id);
        Ok(())
    }

    /// 获取多个设备的实时状态
    pub async fn get_multi(&self, device_ids: &[String]) -> HashMap<String, CacheValue> {
        let mut results = HashMap::new();

        for device_id in device_ids {
            if let Some(value) = self.get(&CacheKey::DeviceLocation(device_id.clone())).await {
                results.insert(device_id.clone(), value);
            }
        }

        results
    }

    /// 清理本地缓存中的过期条目
    async fn cleanup_local_cache(&self, cache: &mut HashMap<String, (CacheValue, DateTime<Utc>)>) {
        let now = Utc::now();
        let expired_keys: Vec<String> = cache
            .iter()
            .filter(|(_, (_, timestamp))| {
                now.signed_duration_since(**timestamp).num_seconds() > self.local_ttl as i64
            })
            .map(|(key, _)| key.clone())
            .collect();

        for key in expired_keys {
            cache.remove(&key);
        }

        if !expired_keys.is_empty() {
            debug!("Local cache cleanup: {} expired entries removed", expired_keys.len());
        }
    }

    /// 获取缓存统计信息
    pub async fn get_stats(&self) -> CacheStats {
        let cache = self.local_cache.read().await;
        
        let mut by_type = HashMap::new();
        for (key, _) in cache.iter() {
            let cache_type = if key.contains("location") {
                "location"
            } else if key.contains("status") {
                "status"
            } else if key.contains("sensors") {
                "sensors"
            } else if key.contains("vehicle") {
                "vehicle"
            } else {
                "other"
            };
            *by_type.entry(cache_type).or_insert(0) += 1;
        }

        CacheStats {
            local_cache_size: cache.len(),
            max_local_entries: self.max_local_entries,
            local_ttl: self.local_ttl,
            entries_by_type: by_type,
        }
    }
}

impl Actor for RealtimeCache {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        info!("RealtimeCache actor started");
    }
}

/// 消息:设置缓存
pub struct SetCache {
    pub key: CacheKey,
    pub value: CacheValue,
}

impl Message for SetCache {
    type Result = Result<(), String>;
}

impl Handler<SetCache> for RealtimeCache {
    type Result = ResponseActFuture<Self, Result<(), String>>;

    fn handle(&mut self, msg: SetCache, _ctx: &mut Self::Context) -> Self::Result {
        let cache = self.clone();
        
        Box::pin(async move {
            cache.set(msg.key, msg.value).await
        }.into_actor(self))
    }
}

/// 消息:获取缓存
pub struct GetCache {
    pub key: CacheKey,
}

impl Message for GetCache {
    type Result = Option<CacheValue>;
}

impl Handler<GetCache> for RealtimeCache {
    type Result = ResponseActFuture<Self, Option<CacheValue>>;

    fn handle(&mut self, msg: GetCache, _ctx: &mut Self::Context) -> Self::Result {
        let cache = self.clone();
        
        Box::pin(async move {
            cache.get(&msg.key).await
        }.into_actor(self))
    }
}

/// 消息:删除缓存
pub struct DeleteCache {
    pub key: CacheKey,
}

impl Message for DeleteCache {
    type Result = Result<(), String>;
}

impl Handler<DeleteCache> for RealtimeCache {
    type Result = ResponseActFuture<Self, Result<(), String>>;

    fn handle(&mut self, msg: DeleteCache, _ctx: &mut Self::Context) -> Self::Result {
        let cache = self.clone();
        
        Box::pin(async move {
            cache.delete(&msg.key).await
        }.into_actor(self))
    }
}

/// 消息:清除设备缓存
pub struct ClearDeviceCache {
    pub device_id: String,
}

impl Message for ClearDeviceCache {
    type Result = Result<(), String>;
}

impl Handler<ClearDeviceCache> for RealtimeCache {
    type Result = ResponseActFuture<Self, Result<(), String>>;

    fn handle(&mut self, msg: ClearDeviceCache, _ctx: &mut Self::Context) -> Self::Result {
        let cache = self.clone();
        
        Box::pin(async move {
            cache.clear_device(&msg.device_id).await
        }.into_actor(self))
    }
}

impl Clone for RealtimeCache {
    fn clone(&self) -> Self {
        Self {
            redis_pool: Arc::clone(&self.redis_pool),
            local_cache: Arc::clone(&self.local_cache),
            max_local_entries: self.max_local_entries,
            local_ttl: self.local_ttl,
        }
    }
}

/// 缓存统计信息
#[derive(Debug, Clone, Serialize)]
pub struct CacheStats {
    pub local_cache_size: usize,
    pub max_local_entries: usize,
    pub local_ttl: u64,
    pub entries_by_type: HashMap<String, usize>,
}






