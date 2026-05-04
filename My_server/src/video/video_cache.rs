//! 视频流缓存优化模块
//!
//! 实现FLV/HLS缓存机制，减少重复转码
//! 提升视频流的分发效率

use bytes::Bytes;
use log::{debug, info};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// 缓存项
#[derive(Debug, Clone)]
pub struct CacheItem {
    /// 缓存数据
    pub data: Bytes,
    /// 创建时间
    pub created_at: Instant,
    /// 最后访问时间
    pub last_accessed: Instant,
    /// 访问次数
    pub access_count: u64,
    /// 缓存大小（字节）
    pub size: usize,
}

impl CacheItem {
    /// 检查缓存是否过期
    pub fn is_expired(&self, ttl: Duration) -> bool {
        Instant::now().duration_since(self.created_at) > ttl
    }
}

/// 缓存配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoCacheConfig {
    /// 是否启用缓存
    pub enabled: bool,
    /// 缓存目录
    pub cache_dir: PathBuf,
    /// 最大缓存大小（MB）
    pub max_cache_size_mb: usize,
    /// FLV缓存TTL（秒）
    pub flv_cache_ttl: u64,
    /// HLS分片缓存TTL（秒）
    pub hls_segment_ttl: u64,
    /// HLS播放列表缓存TTL（秒）
    pub hls_playlist_ttl: u64,
    /// 最大缓存项数
    pub max_items: usize,
}

impl Default for VideoCacheConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            cache_dir: PathBuf::from("./video_cache"),
            max_cache_size_mb: 1024,
            flv_cache_ttl: 300,   // 5分钟
            hls_segment_ttl: 600, // 10分钟
            hls_playlist_ttl: 60, // 1分钟
            max_items: 1000,
        }
    }
}

/// 视频流缓存管理器
/// 管理FLV和HLS视频流的缓存
pub struct VideoStreamCache {
    /// FLV流缓存 (stream_id -> 缓存项)
    flv_cache: Arc<RwLock<HashMap<String, CacheItem>>>,
    /// HLS分片缓存 (stream_id/segment -> 缓存项)
    hls_segment_cache: Arc<RwLock<HashMap<String, CacheItem>>>,
    /// HLS播放列表缓存 (stream_id -> 缓存项)
    hls_playlist_cache: Arc<RwLock<HashMap<String, CacheItem>>>,
    /// 缓存配置
    config: VideoCacheConfig,
    /// 缓存统计
    stats: Arc<RwLock<CacheStats>>,
}

/// 缓存统计信息
#[derive(Debug, Clone, Serialize, Default)]
pub struct CacheStats {
    /// 总命中次数
    pub total_hits: u64,
    /// 总未命中次数
    pub total_misses: u64,
    /// 当前缓存项数
    pub current_items: usize,
    /// 当前缓存大小（字节）
    pub current_size_bytes: usize,
    /// 缓存淘汰次数
    pub evictions: u64,
}

impl CacheStats {
    /// 计算缓存命中率
    pub fn hit_rate(&self) -> f64 {
        let total = self.total_hits + self.total_misses;
        if total == 0 {
            return 0.0;
        }
        (self.total_hits as f64 / total as f64) * 100.0
    }
}

impl VideoStreamCache {
    /// 创建新的视频流缓存管理器
    pub fn new(config: VideoCacheConfig) -> Self {
        // P4: 确保缓存目录存在，使用 unwrap_or_else 处理错误
        if !config.cache_dir.exists() {
            if let Err(e) = std::fs::create_dir_all(&config.cache_dir) {
                tracing::error!("Failed to create video cache directory: {}", e);
                // 继续运行，缓存将在内存中进行
            }
        }

        Self {
            flv_cache: Arc::new(RwLock::new(HashMap::new())),
            hls_segment_cache: Arc::new(RwLock::new(HashMap::new())),
            hls_playlist_cache: Arc::new(RwLock::new(HashMap::new())),
            config,
            stats: Arc::new(RwLock::new(CacheStats::default())),
        }
    }

    /// 获取FLV缓存
    pub async fn get_flv(&self, stream_id: &str) -> Option<Bytes> {
        if !self.config.enabled {
            return None;
        }

        let mut flv_cache = self.flv_cache.write().await;
        if let Some(item) = flv_cache.get(stream_id) {
            let ttl = Duration::from_secs(self.config.flv_cache_ttl);
            if !item.is_expired(ttl) {
                let mut stats = self.stats.write().await;
                stats.total_hits += 1;

                // 更新访问统计
                if let Some(existing_item) = flv_cache.get_mut(stream_id) {
                    existing_item.last_accessed = Instant::now();
                    existing_item.access_count += 1;

                    debug!("FLV cache hit for stream {}", stream_id);
                    return Some(existing_item.data.clone());
                }
                return None; // 不应该发生，但安全处理
            }
        }

        let mut stats = self.stats.write().await;
        stats.total_misses += 1;
        None
    }

    /// 设置FLV缓存
    pub async fn set_flv(&self, stream_id: &str, data: Bytes) {
        if !self.config.enabled {
            return;
        }

        let size = data.len();
        let item = CacheItem {
            data,
            created_at: Instant::now(),
            last_accessed: Instant::now(),
            access_count: 0,
            size,
        };

        let mut flv_cache = self.flv_cache.write().await;
        flv_cache.insert(stream_id.to_string(), item);

        let mut stats = self.stats.write().await;
        stats.current_items = flv_cache.len();
        stats.current_size_bytes += size;

        debug!(
            "FLV cache set for stream {}, size: {} bytes",
            stream_id, size
        );

        // 检查是否需要淘汰
        self.evict_if_needed(&mut flv_cache, &mut stats).await;
    }

    /// 获取HLS分片缓存
    pub async fn get_hls_segment(&self, stream_id: &str, segment: &str) -> Option<Bytes> {
        if !self.config.enabled {
            return None;
        }

        let cache_key = format!("{}/{}", stream_id, segment);
        let hls_cache = self.hls_segment_cache.read().await;

        if let Some(item) = hls_cache.get(&cache_key) {
            let ttl = Duration::from_secs(self.config.hls_segment_ttl);
            if !item.is_expired(ttl) {
                let mut stats = self.stats.write().await;
                stats.total_hits += 1;
                debug!("HLS segment cache hit for {}/{}", stream_id, segment);
                return Some(item.data.clone());
            }
        }

        let mut stats = self.stats.write().await;
        stats.total_misses += 1;
        None
    }

    /// 设置HLS分片缓存
    pub async fn set_hls_segment(&self, stream_id: &str, segment: &str, data: Bytes) {
        if !self.config.enabled {
            return;
        }

        let cache_key = format!("{}/{}", stream_id, segment);
        let size = data.len();
        let item = CacheItem {
            data,
            created_at: Instant::now(),
            last_accessed: Instant::now(),
            access_count: 0,
            size,
        };

        let mut hls_cache = self.hls_segment_cache.write().await;
        hls_cache.insert(cache_key, item);

        let mut stats = self.stats.write().await;
        stats.current_size_bytes += size;

        debug!("HLS segment cached for {}/{}", stream_id, segment);
    }

    /// 获取HLS播放列表缓存
    pub async fn get_hls_playlist(&self, stream_id: &str) -> Option<Bytes> {
        if !self.config.enabled {
            return None;
        }

        let hls_cache = self.hls_playlist_cache.read().await;
        if let Some(item) = hls_cache.get(stream_id) {
            let ttl = Duration::from_secs(self.config.hls_playlist_ttl);
            if !item.is_expired(ttl) {
                let mut stats = self.stats.write().await;
                stats.total_hits += 1;
                debug!("HLS playlist cache hit for stream {}", stream_id);
                return Some(item.data.clone());
            }
        }

        let mut stats = self.stats.write().await;
        stats.total_misses += 1;
        None
    }

    /// 设置HLS播放列表缓存
    pub async fn set_hls_playlist(&self, stream_id: &str, data: Bytes) {
        if !self.config.enabled {
            return;
        }

        let size = data.len();
        let item = CacheItem {
            data,
            created_at: Instant::now(),
            last_accessed: Instant::now(),
            access_count: 0,
            size,
        };

        let mut hls_cache = self.hls_playlist_cache.write().await;
        hls_cache.insert(stream_id.to_string(), item);

        debug!("HLS playlist cached for stream {}", stream_id);
    }

    /// 缓存淘汰（LRU策略）
    async fn evict_if_needed(
        &self,
        cache: &mut HashMap<String, CacheItem>,
        stats: &mut CacheStats,
    ) {
        // 检查是否超过最大项数
        if cache.len() > self.config.max_items {
            // 找出最少访问的项
            let mut least_accessed: Option<(String, Instant, u64)> = None;

            for (key, item) in cache.iter() {
                let should_replace = match least_accessed {
                    None => true,
                    Some((_, ref last_time, ref count)) => {
                        item.last_accessed < *last_time
                            || (item.last_accessed == *last_time && item.access_count < *count)
                    }
                };
                if should_replace {
                    least_accessed = Some((key.clone(), item.last_accessed, item.access_count));
                }
            }

            if let Some((key, _, _)) = least_accessed {
                if let Some(item) = cache.remove(&key) {
                    stats.current_size_bytes = stats.current_size_bytes.saturating_sub(item.size);
                    stats.evictions += 1;
                    debug!("Evicted cache item: {}", key);
                }
            }
        }

        // 检查是否超过最大缓存大小
        let max_size_bytes = self.config.max_cache_size_mb * 1024 * 1024;
        while stats.current_size_bytes > max_size_bytes && !cache.is_empty() {
            let mut least_accessed: Option<(String, Instant, u64)> = None;

            for (key, item) in cache.iter() {
                let should_replace = match least_accessed {
                    None => true,
                    Some((_, ref last_time, _)) => item.last_accessed < *last_time,
                };
                if should_replace {
                    least_accessed = Some((key.clone(), item.last_accessed, item.access_count));
                }
            }

            if let Some((key, _, _)) = least_accessed {
                if let Some(item) = cache.remove(&key) {
                    stats.current_size_bytes = stats.current_size_bytes.saturating_sub(item.size);
                    stats.evictions += 1;
                    debug!("Evicted cache item due to size limit: {}", key);
                }
            }
        }
    }

    /// 清理过期缓存
    pub async fn cleanup_expired(&self) {
        let flv_ttl = Duration::from_secs(self.config.flv_cache_ttl);
        let hls_segment_ttl = Duration::from_secs(self.config.hls_segment_ttl);
        let hls_playlist_ttl = Duration::from_secs(self.config.hls_playlist_ttl);

        // 清理FLV缓存
        {
            let mut flv_cache = self.flv_cache.write().await;
            let before_count = flv_cache.len();
            flv_cache.retain(|_, item| !item.is_expired(flv_ttl));
            let removed = before_count - flv_cache.len();
            if removed > 0 {
                debug!("Cleaned up {} expired FLV cache items", removed);
            }
        }

        // 清理HLS分片缓存
        {
            let mut hls_cache = self.hls_segment_cache.write().await;
            let before_count = hls_cache.len();
            hls_cache.retain(|_, item| !item.is_expired(hls_segment_ttl));
            let removed = before_count - hls_cache.len();
            if removed > 0 {
                debug!("Cleaned up {} expired HLS segment cache items", removed);
            }
        }

        // 清理HLS播放列表缓存
        {
            let mut hls_cache = self.hls_playlist_cache.write().await;
            let before_count = hls_cache.len();
            hls_cache.retain(|_, item| !item.is_expired(hls_playlist_ttl));
            let removed = before_count - hls_cache.len();
            if removed > 0 {
                debug!("Cleaned up {} expired HLS playlist cache items", removed);
            }
        }
    }

    /// 获取缓存统计信息
    pub async fn get_stats(&self) -> CacheStats {
        let stats = self.stats.read().await;
        let mut cloned = stats.clone();
        cloned.current_items = self.flv_cache.read().await.len()
            + self.hls_segment_cache.read().await.len()
            + self.hls_playlist_cache.read().await.len();
        cloned
    }

    /// 清空所有缓存
    pub async fn clear_all(&self) {
        self.flv_cache.write().await.clear();
        self.hls_segment_cache.write().await.clear();
        self.hls_playlist_cache.write().await.clear();

        let mut stats = self.stats.write().await;
        stats.current_items = 0;
        stats.current_size_bytes = 0;

        info!("All video cache cleared");
    }
}

/// 创建视频流缓存管理器（便捷函数）
pub fn create_video_cache() -> Arc<VideoStreamCache> {
    Arc::new(VideoStreamCache::new(VideoCacheConfig::default()))
}

/// 创建自定义配置的视频流缓存
pub fn create_video_cache_with_config(config: VideoCacheConfig) -> Arc<VideoStreamCache> {
    Arc::new(VideoStreamCache::new(config))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_video_cache_creation() {
        let cache = VideoStreamCache::new(VideoCacheConfig::default());
        assert!(cache.config.enabled);
    }

    #[tokio::test]
    async fn test_cache_stats() {
        let stats = CacheStats {
            total_hits: 80,
            total_misses: 20,
            ..Default::default()
        };
        assert!((stats.hit_rate() - 80.0).abs() < 0.01);
    }
}
