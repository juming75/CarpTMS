//! / 内存缓存模块
// 提供基于内存的缓存实现,作为多级缓存的第一级

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

/// 缓存项结构
#[derive(Debug)]
struct CacheItem<T> {
    value: T,
    expires_at: Option<Instant>,
}

/// 内存缓存实现
#[derive(Debug, Clone)]
pub struct MemoryCache<T: Clone + Send + Sync> {
    cache: Arc<RwLock<HashMap<String, CacheItem<T>>>>,
    default_ttl: Duration,
    max_size: usize,
}

impl<T: Clone + Send + Sync> MemoryCache<T> {
    /// 创建新的内存缓存实例
    pub fn new(default_ttl: Duration, max_size: usize) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            default_ttl,
            max_size,
        }
    }

}

impl<T: Clone + Send + Sync> Default for MemoryCache<T> {
    fn default() -> Self {
        Self::new(Duration::from_secs(300), 10000)
    }
}

impl<T: Clone + Send + Sync> MemoryCache<T> {

    /// 获取缓存项
    pub fn get(&self, key: &str) -> Option<T> {
        if let Ok(mut cache) = self.cache.write() {
            // 清理过期项
            self.clean_expired(&mut cache);

            // 检查缓存项是否存在且未过期
            if let Some(item) = cache.get(key) {
                if item.expires_at.map(|exp| exp > Instant::now()).unwrap_or(true) {
                    return Some(item.value.clone());
                }
            }
        }
        None
    }

    /// 设置缓存项
    pub fn set(&self, key: &str, value: T, ttl: Option<Duration>) {
        if let Ok(mut cache) = self.cache.write() {
            // 清理过期项
            self.clean_expired(&mut cache);

            // 如果缓存已满,删除最旧的项
            if cache.len() >= self.max_size {
                self.evict_oldest(&mut cache);
            }

            // 设置缓存项
            let expires_at = ttl
                .map(|d| Instant::now() + d)
                .or_else(|| Some(Instant::now() + self.default_ttl));

            cache.insert(key.to_string(), CacheItem { value, expires_at });
        }
    }

    /// 删除缓存项
    pub fn del(&self, key: &str) {
        if let Ok(mut cache) = self.cache.write() {
            cache.remove(key);
        }
    }

    /// 删除匹配模式的缓存项
    pub fn del_pattern(&self, pattern: &str) {
        if let Ok(mut cache) = self.cache.write() {
            // 简单的模式匹配,仅支持*通配符
            let keys_to_remove: Vec<String> = cache
                .keys()
                .filter(|key| self.matches_pattern(key, pattern))
                .cloned()
                .collect();

            for key in keys_to_remove {
                cache.remove(&key);
            }
        }
    }

    /// 清空所有缓存
    pub fn flush_all(&self) {
        if let Ok(mut cache) = self.cache.write() {
            cache.clear();
        }
    }

    /// 获取缓存大小
    pub fn size(&self) -> usize {
        self.cache.read().ok().map(|c| c.len()).unwrap_or(0)
    }

    /// 清理过期项
    fn clean_expired(&self, cache: &mut HashMap<String, CacheItem<T>>) {
        let now = Instant::now();
        cache.retain(|_, item| item.expires_at.map(|exp| exp > now).unwrap_or(true));
    }

    /// 删除最旧的项
    fn evict_oldest(&self, cache: &mut HashMap<String, CacheItem<T>>) {
        // 找到最早过期的项
        let oldest_key = cache
            .iter()
            .min_by(|(_, a), (_, b)| {
                let a_exp = a
                    .expires_at
                    .unwrap_or(Instant::now() + Duration::from_secs(3600));
                let b_exp = b
                    .expires_at
                    .unwrap_or(Instant::now() + Duration::from_secs(3600));
                a_exp.cmp(&b_exp)
            })
            .map(|(key, _)| key.clone());

        if let Some(key) = oldest_key {
            cache.remove(&key);
        }
    }

    /// 检查键是否匹配模式
    fn matches_pattern(&self, key: &str, pattern: &str) -> bool {
        // 简单的模式匹配,仅支持*通配符
        let parts: Vec<&str> = pattern.split('*').collect();

        if parts.is_empty() {
            return key.is_empty();
        }

        // 检查前缀
        if !key.starts_with(parts[0]) {
            return false;
        }

        // 检查后缀
        if let Some(last_part) = parts.last() {
            if !last_part.is_empty() && !key.ends_with(last_part) {
                return false;
            }
        }

        // 检查中间部分
        let mut current_pos = parts[0].len();
        for part in parts.iter().skip(1).take(parts.len() - 2) {
            if part.is_empty() {
                continue;
            }

            if let Some(pos) = key[current_pos..].find(part) {
                current_pos += pos + part.len();
            } else {
                return false;
            }
        }

        true
    }
}


