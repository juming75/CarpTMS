//! / 同步缓存模块
// 已激活 - 2026-01-19

use log::{debug, info, warn};
use std::collections::HashMap;

use std::time::Duration;
use tokio::time::Instant;

/// 同步缓存
pub struct SyncCache {
    cache: HashMap<String, CachedItem>,
    ttl: Duration,
}

#[derive(Debug, Clone)]
struct CachedItem {
    data: Vec<u8>,
    timestamp: Instant,
}

impl SyncCache {
    /// 创建新的缓存实例
    pub fn new(ttl_seconds: u64) -> Self {
        Self {
            cache: HashMap::new(),
            ttl: Duration::from_secs(ttl_seconds),
        }
    }

    /// 缓存车辆数据
    pub fn cache_vehicles(&mut self, data: &[u8]) {
        let key = "vehicles".to_string();
        self.cache.insert(
            key,
            CachedItem {
                data: data.to_vec(),
                timestamp: Instant::now(),
            },
        );
        debug!("Cached vehicles data, size: {} bytes", data.len());
    }

    /// 缓存GPS数据
    pub fn cache_gps(&mut self, device_id: &str, data: &[u8]) {
        let key = format!("gps:{}", device_id);
        self.cache.insert(
            key,
            CachedItem {
                data: data.to_vec(),
                timestamp: Instant::now(),
            },
        );
        debug!(
            "Cached GPS data for device {}, size: {} bytes",
            device_id,
            data.len()
        );
    }

    /// 获取车辆数据
    pub fn get_vehicles(&self) -> Option<&[u8]> {
        self.get("vehicles")
    }

    /// 获取GPS数据
    pub fn get_gps(&self, device_id: &str) -> Option<&[u8]> {
        self.get(&format!("gps:{}", device_id))
    }

    /// 获取缓存项
    fn get(&self, key: &str) -> Option<&[u8]> {
        if let Some(item) = self.cache.get(key) {
            if item.timestamp.elapsed() < self.ttl {
                return Some(&item.data);
            } else {
                warn!("Cache expired for key: {}", key);
                return None;
            }
        }
        None
    }

    /// 清理过期缓存
    pub fn cleanup_expired(&mut self) {
        let before_count = self.cache.len();
        self.cache
            .retain(|_, item| item.timestamp.elapsed() < self.ttl);
        let after_count = self.cache.len();
        if before_count > after_count {
            info!(
                "Cleaned up {} expired cache entries",
                before_count - after_count
            );
        }
    }

    /// 清空缓存
    pub fn clear(&mut self) {
        let count = self.cache.len();
        self.cache.clear();
        info!("Cleared all cache entries: {}", count);
    }
}
