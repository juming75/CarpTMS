use super::{CacheConfig, CacheManager, CachePreheatManager};
use crate::utils::log_cache_error;
use anyhow::Result;
use futures;
use log::{debug, error, info};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tokio::sync::watch;

// 类型别名简化复杂类型
type CacheItems = Arc<RwLock<HashMap<String, (Vec<u8>, CacheItemMetadata)>>>;
type CacheMetadata = Arc<RwLock<HashMap<String, CacheItemMetadata>>>;

// 缓存策略类型
#[derive(Debug, Clone)]
pub enum CacheStrategyType {
    TimeToLive,          // 基于时间的过期
    LeastRecentlyUsed,   // 最近最少使用
    LeastFrequentlyUsed, // 最不经常使用
    WriteThrough,        // 写透策略
    WriteBack,           // 写回策略
}

// 缓存项元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheItemMetadata {
    pub key: String,
    pub created_at: Duration,    // 从系统启动开始的时间
    pub last_accessed: Duration, // 最后访问时间
    pub access_count: u64,       // 访问次数
    pub ttl: Duration,           // 过期时间
    pub size: usize,             // 估算大小
}

// 缓存策略配置
#[derive(Debug, Clone)]
pub struct CacheStrategyConfig {
    pub strategy_type: CacheStrategyType,
    pub ttl: Duration,
    pub max_size: usize,            // 最大缓存项数量
    pub eviction_threshold: f64,    // 驱逐阈值
    pub refresh_interval: Duration, // 刷新间隔
    pub monitor_interval: Duration, // 监控间隔
}

impl Default for CacheStrategyConfig {
    fn default() -> Self {
        Self {
            strategy_type: CacheStrategyType::TimeToLive,
            ttl: Duration::from_secs(300),
            max_size: 10000,
            eviction_threshold: 0.8,
            refresh_interval: Duration::from_secs(60),
            monitor_interval: Duration::from_secs(30),
        }
    }
}

// 布隆过滤器实现,用于防止缓存穿透
#[derive(Debug, Clone)]
pub struct BloomFilter {
    bits: Arc<RwLock<Vec<bool>>>,
    hashes: usize,
    size: usize,
}

impl BloomFilter {
    pub fn new(size: usize, hashes: usize) -> Self {
        Self {
            bits: Arc::new(RwLock::new(vec![false; size])),
            hashes,
            size,
        }
    }

    // 计算多个哈希值
    fn get_hashes(&self, key: &str) -> Vec<usize> {
        let mut hashes = Vec::with_capacity(self.hashes);

        for seed in 0..self.hashes {
            let hash = self.hash_with_seed(key, seed as u64);
            hashes.push(hash % self.size);
        }

        hashes
    }

    // 带种子的哈希函数
    fn hash_with_seed(&self, key: &str, seed: u64) -> usize {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        key.hash(&mut hasher);
        seed.hash(&mut hasher);
        hasher.finish() as usize
    }

    // 添加键到过滤器
    pub fn add(&self, key: &str) {
        if let Ok(mut bits) = self.bits.write() {
            let hashes = self.get_hashes(key);
            for hash in hashes {
                bits[hash] = true;
            }
        }
    }

    // 检查键是否可能存在
    pub fn contains(&self, key: &str) -> bool {
        if let Ok(bits) = self.bits.read() {
            let hashes = self.get_hashes(key);
            return hashes.iter().all(|&hash| bits[hash]);
        }
        false
    }

    // 清除过滤器
    pub fn clear(&self) {
        if let Ok(mut bits) = self.bits.write() {
            bits.iter_mut().for_each(|bit| *bit = false);
        }
    }
}

// 缓存一致性管理器
#[derive(Debug, Clone)]
pub struct CacheConsistencyManager {
    dependencies: Arc<RwLock<HashMap<String, HashSet<String>>>>, // 键依赖关系
    invalidation_queue: Arc<RwLock<Vec<String>>>,                // 失效队列
    consistency_watcher: watch::Sender<()>,                      // 一致性变更通知
    bloom_filter: Arc<BloomFilter>,                              // 用于防止缓存穿透
}

impl CacheConsistencyManager {
    pub fn new() -> Self {
        let (tx, _) = watch::channel(());
        Self {
            dependencies: Arc::new(RwLock::new(HashMap::new())),
            invalidation_queue: Arc::new(RwLock::new(Vec::new())),
            consistency_watcher: tx,
            bloom_filter: Arc::new(BloomFilter::new(1_000_000, 5)), // 1MB 大小,5个哈希函数
        }
    }

    // 注册键依赖关系
    pub fn register_dependency(&self, key: &str, depends_on: &str) {
        if let Ok(mut dependencies) = self.dependencies.write() {
            dependencies
                .entry(depends_on.to_string())
                .or_default()
                .insert(key.to_string());
            debug!("Registered dependency: {} depends on {}", key, depends_on);
        }
    }

    // 触发缓存失效
    pub fn invalidate(&self, key: &str) -> Vec<String> {
        let mut invalidated = Vec::new();
        
        if let Ok(dependencies) = self.dependencies.read() {
            // 收集所有依赖于该键的缓存项
            if let Some(deps) = dependencies.get(key) {
                for dep_key in deps {
                    invalidated.push(dep_key.clone());
                }
            }
        }

        // 添加自身到失效列表
        invalidated.push(key.to_string());

        // 将失效键加入队列
        if let Ok(mut queue) = self.invalidation_queue.write() {
            queue.extend(invalidated.clone());
        }

        // 通知监听器（忽略发送失败）
        let _ = self.consistency_watcher.send(());

        invalidated
    }

    // 获取失效队列
    pub fn get_invalidation_queue(&self) -> Vec<String> {
        match self.invalidation_queue.read() {
            Ok(queue) => queue.clone(),
            Err(e) => {
                log::warn!("获取失效队列锁失败: {}", e);
                Vec::new()
            }
        }
    }

    // 清空失效队列
    pub fn clear_invalidation_queue(&self) {
        if let Ok(mut queue) = self.invalidation_queue.write() {
            queue.clear();
        }
    }

    // 添加键到布隆过滤器
    pub fn add_to_bloom_filter(&self, key: &str) {
        self.bloom_filter.add(key);
    }

    // 检查键是否可能存在于缓存中
    pub fn might_contain(&self, key: &str) -> bool {
        self.bloom_filter.contains(key)
    }

    // 清除布隆过滤器
    pub fn clear_bloom_filter(&self) {
        self.bloom_filter.clear();
    }
}

impl Default for CacheConsistencyManager {
    fn default() -> Self {
        Self::new()
    }
}

// 内存缓存层
#[derive(Debug, Clone)]
pub struct MemoryCacheLayer {
    items: CacheItems,
    metadata: CacheMetadata,
    config: CacheStrategyConfig,
}

impl MemoryCacheLayer {
    pub fn new(config: CacheStrategyConfig) -> Self {
        Self {
            items: Arc::new(RwLock::new(HashMap::new())),
            metadata: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    // 获取当前时间(从系统启动开始的时间)
    fn current_time(&self) -> Duration {
        // 这里简化处理,实际应使用系统启动时间
        Duration::from_secs(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or(Duration::from_secs(0))
                .as_secs(),
        )
    }

    // 设置缓存项
    pub fn set(&self, key: &str, value: Vec<u8>, ttl: Duration) {
        let now = self.current_time();
        let metadata = CacheItemMetadata {
            key: key.to_string(),
            created_at: now,
            last_accessed: now,
            access_count: 1,
            ttl,
            size: value.len(),
        };

        if let (Ok(mut items), Ok(mut metadata_map)) = (self.items.write(), self.metadata.write()) {
            items.insert(key.to_string(), (value, metadata.clone()));
            metadata_map.insert(key.to_string(), metadata);
            drop((items, metadata_map));
            // 检查是否需要驱逐
            self.evict_if_needed();
        }
    }

    // 获取缓存项
    pub fn get(&self, key: &str) -> Option<Vec<u8>> {
        let now = self.current_time();
        let mut items = self.items.write().ok()?;
        let mut metadata_map = self.metadata.write().ok()?;

        if let Some((value, metadata)) = items.get_mut(key) {
            // 更新元数据
            metadata.last_accessed = now;
            metadata.access_count += 1;
            metadata_map.insert(key.to_string(), metadata.clone());

            // 检查是否过期
            if now - metadata.created_at > metadata.ttl {
                items.remove(key);
                metadata_map.remove(key);
                return None;
            }

            Some(value.clone())
        } else {
            None
        }
    }

    // 删除缓存项
    pub fn del(&self, key: &str) {
        if let (Ok(mut items), Ok(mut metadata_map)) = (self.items.write(), self.metadata.write()) {
            items.remove(key);
            metadata_map.remove(key);
        }
    }

    // 执行缓存驱逐
    fn evict_if_needed(&self) {
        let items = match self.items.read() {
            Ok(i) => i,
            Err(e) => {
                log::warn!("获取缓存项锁失败: {}", e);
                return;
            }
        };
        let current_size = items.len();
        drop(items);
        let threshold_size =
            (self.config.max_size as f64 * self.config.eviction_threshold) as usize;

        if current_size > threshold_size {
            self.perform_eviction(current_size - threshold_size);
        }
    }

    // 执行驱逐策略
    fn perform_eviction(&self, count: usize) {
        let mut metadata_map = match self.metadata.write() {
            Ok(m) => m,
            Err(_) => return,
        };
        let mut items = match self.items.write() {
            Ok(i) => i,
            Err(_) => return,
        };

        // 根据策略选择要驱逐的键
        let mut keys_to_evict: Vec<String> = Vec::new();

        match self.config.strategy_type {
            CacheStrategyType::TimeToLive => {
                // 按过期时间排序
                let mut sorted_metadata: Vec<_> = metadata_map.iter().collect();
                sorted_metadata
                    .sort_by(|(_, a), (_, b)| (a.created_at + a.ttl).cmp(&(b.created_at + b.ttl)));

                for (key, _) in sorted_metadata.iter().take(count) {
                    keys_to_evict.push(key.to_string());
                }
            }
            CacheStrategyType::LeastRecentlyUsed => {
                // 按最后访问时间排序
                let mut sorted_metadata: Vec<_> = metadata_map.iter().collect();
                sorted_metadata.sort_by(|(_, a), (_, b)| a.last_accessed.cmp(&b.last_accessed));

                for (key, _) in sorted_metadata.iter().take(count) {
                    keys_to_evict.push(key.to_string());
                }
            }
            CacheStrategyType::LeastFrequentlyUsed => {
                // 按访问次数排序
                let mut sorted_metadata: Vec<_> = metadata_map.iter().collect();
                sorted_metadata.sort_by(|(_, a), (_, b)| a.access_count.cmp(&b.access_count));

                for (key, _) in sorted_metadata.iter().take(count) {
                    keys_to_evict.push(key.to_string());
                }
            }
            _ => {
                // 其他策略默认使用TTL
                let mut sorted_metadata: Vec<_> = metadata_map.iter().collect();
                sorted_metadata
                    .sort_by(|(_, a), (_, b)| (a.created_at + a.ttl).cmp(&(b.created_at + b.ttl)));

                for (key, _) in sorted_metadata.iter().take(count) {
                    keys_to_evict.push(key.to_string());
                }
            }
        }

        // 执行驱逐
        for key in keys_to_evict {
            items.remove(&key);
            metadata_map.remove(&key);
            debug!("Evicted cache key: {}", key);
        }
    }
}

// 统一缓存管理器
#[derive(Debug, Clone)]
pub struct UnifiedCacheManager {
    redis_cache: Arc<CacheManager>,
    memory_cache: Arc<MemoryCacheLayer>,
    consistency_manager: Arc<CacheConsistencyManager>,
    preheat_manager: Arc<CachePreheatManager>,
    config: CacheStrategyConfig,
    cache_stats: Arc<RwLock<CacheStats>>,
    cache_version: Arc<RwLock<String>>, // 缓存版本,用于处理 schema 变更
    distributed_invalidation_channel: Option<Arc<tokio::sync::broadcast::Sender<String>>>, // 分布式失效通知通道
    // 缓存击穿防护:用于热点键的互斥锁
    hot_key_locks: Arc<RwLock<HashMap<String, Arc<tokio::sync::Mutex<()>>>>>,
}

// 缓存统计信息
#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub sets: u64,
    pub deletes: u64,
    pub memory_hits: u64,
    pub redis_hits: u64,
    pub evictions: u64,
    pub invalidations: u64,
}

impl UnifiedCacheManager {
    pub async fn new(
        redis_config: CacheConfig,
        strategy_config: CacheStrategyConfig,
    ) -> Result<Self> {
        let redis_cache = Arc::new(CacheManager::new(redis_config));
        let memory_cache = Arc::new(MemoryCacheLayer::new(strategy_config.clone()));
        let consistency_manager = Arc::new(CacheConsistencyManager::new());
        let preheat_manager = Arc::new(super::create_cache_preheat_manager());
        let cache_stats = Arc::new(RwLock::new(CacheStats::default()));
        let cache_version = Arc::new(RwLock::new(format!("v{}", uuid::Uuid::new_v4())));
        let (tx, _) = tokio::sync::broadcast::channel(100);
        let distributed_invalidation_channel = Some(Arc::new(tx));

        let hot_key_locks = Arc::new(RwLock::new(HashMap::new()));

        let manager = Self {
            redis_cache,
            memory_cache,
            consistency_manager,
            preheat_manager,
            config: strategy_config,
            cache_stats,
            cache_version,
            distributed_invalidation_channel,
            hot_key_locks,
        };

        // 启动后台任务
        manager.start_background_tasks().await;

        Ok(manager)
    }

    // 启动后台任务
    async fn start_background_tasks(&self) {
        let manager = self.clone();

        // 启动缓存监控任务
        tokio::spawn(async move {
            manager.monitoring_task().await;
        });

        // 启动缓存一致性任务
        let manager = self.clone();
        tokio::spawn(async move {
            manager.consistency_task().await;
        });

        // 启动缓存预热任务
        let manager = self.clone();
        tokio::spawn(async move {
            manager.warmup_task().await;
        });

        // 启动分布式失效监听任务
        let manager = self.clone();
        tokio::spawn(async move {
            manager.distributed_invalidation_task().await;
        });

        // 启动预热管理器
        let preheat_manager = self.preheat_manager.clone();
        tokio::spawn(async move {
            if let Err(e) = preheat_manager.start().await {
                error!("Failed to start cache preheat manager: {}", e);
            }
        });
    }

    // 监控任务
    async fn monitoring_task(&self) {
        loop {
            tokio::time::sleep(self.config.monitor_interval).await;

            if let Ok(stats) = self.cache_stats.read() {
                let stats = stats.clone();
                info!("Cache stats: hits={}, misses={}, memory_hits={}, redis_hits={}, evictions={}, invalidations={}",
                      stats.hits, stats.misses, stats.memory_hits, stats.redis_hits, stats.evictions, stats.invalidations);
            }

            // 检查内存缓存大小
            let memory_size = self.memory_cache.items.read().ok().map(|i| i.len()).unwrap_or(0);
            info!("Memory cache size: {} items", memory_size);
        }
    }

    // 一致性任务
    async fn consistency_task(&self) {
        loop {
            // 处理失效队列
            let keys_to_invalidate = self.consistency_manager.get_invalidation_queue();
            if !keys_to_invalidate.is_empty() {
                for key in keys_to_invalidate {
                    // 从内存缓存和Redis中删除
                    self.memory_cache.del(&key);
                    let _ = self.redis_cache.del(&key).await;
                    info!("Invalidated cache key: {}", key);
                }

                // 清空队列
                self.consistency_manager.clear_invalidation_queue();
            }

            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    }

    // 预热任务
    async fn warmup_task(&self) {
        // 实现缓存预热逻辑
        info!("Cache warmup task started");

        // 预热热点数据
        self.warmup_hot_data().await;

        // 预热配置数据
        self.warmup_config_data().await;

        // 预热统计数据
        self.warmup_statistics_data().await;
    }

    // 预热热点数据
    async fn warmup_hot_data(&self) {
        info!("Warming up hot data...");

        // 模拟热点数据键
        let hot_keys = [
            "vehicle:123",
            "vehicle:456",
            "user:admin",
            "user:manager",
            "route:default",
        ];

        // 将热点键添加到预热队列
        let keys_to_preheat: Vec<String> = hot_keys.iter().map(|k| k.to_string()).collect();
        if let Err(e) = self.preheat_manager.preheat_keys(keys_to_preheat).await {
            error!("Failed to preheat hot data: {}", e);
        }
    }

    // 预热配置数据
    async fn warmup_config_data(&self) {
        info!("Warming up config data...");

        // 模拟配置数据键
        let config_keys = [
            "config:system",
            "config:permissions",
            "config:routes",
            "config:defaults",
        ];

        // 将配置键添加到预热队列
        let keys_to_preheat: Vec<String> = config_keys.iter().map(|k| k.to_string()).collect();
        if let Err(e) = self.preheat_manager.preheat_keys(keys_to_preheat).await {
            error!("Failed to preheat config data: {}", e);
        }
    }

    // 预热统计数据
    async fn warmup_statistics_data(&self) {
        info!("Warming up statistics data...");

        // 模拟统计数据键
        let stats_keys = [
            "stats:daily",
            "stats:weekly",
            "stats:monthly",
            "stats:vehicles",
        ];

        // 将统计键添加到预热队列
        let keys_to_preheat: Vec<String> = stats_keys.iter().map(|k| k.to_string()).collect();
        if let Err(e) = self.preheat_manager.preheat_keys(keys_to_preheat).await {
            error!("Failed to preheat statistics data: {}", e);
        }
    }

    // 添加TTL抖动,防止缓存雪崩
    fn add_ttl_jitter(&self, ttl: Duration) -> Duration {
        let mut rng = rand::thread_rng();
        let jitter = rng.gen_range(0..(ttl.as_secs() / 10)); // 10% 的抖动
        ttl + Duration::from_secs(jitter)
    }

    // 设置缓存
    pub async fn set<T: Serialize>(
        &self,
        key: &str,
        value: &T,
        ttl: Option<Duration>,
    ) -> Result<()> {
        let versioned_key = self.get_versioned_key(key);
        let mut ttl = ttl.unwrap_or(self.config.ttl);

        // 添加TTL抖动,防止缓存雪崩
        ttl = self.add_ttl_jitter(ttl);

        // 序列化值
        let serialized = serde_json::to_vec(value)?;

        // 写入内存缓存
        self.memory_cache
            .set(&versioned_key, serialized.clone(), ttl);

        // 写入Redis
        self.redis_cache
            .set(&versioned_key, &serialized, Some(ttl))
            .await?;

        // 添加到布隆过滤器
        self.consistency_manager.add_to_bloom_filter(key);

        // 更新统计
        if let Ok(mut stats) = self.cache_stats.write() {
            stats.sets += 1;
        }

        Ok(())
    }

    // 获取缓存
    pub async fn get<T: for<'de> Deserialize<'de> + Serialize>(
        &self,
        key: &str,
    ) -> Result<Option<T>> {
        let versioned_key = self.get_versioned_key(key);

        // 使用布隆过滤器防止缓存穿透
        if !self.consistency_manager.might_contain(key) {
            // 更新统计
            if let Ok(mut stats) = self.cache_stats.write() {
                stats.misses += 1;
            }
            debug!(
                "Cache penetration prevention: key {} not in bloom filter",
                key
            );
            return Ok(None);
        }

        // 先从内存缓存获取
        if let Some(serialized) = self.memory_cache.get(&versioned_key) {
            // 更新统计
            if let Ok(mut stats) = self.cache_stats.write() {
                stats.hits += 1;
                stats.memory_hits += 1;
            }

            // 反序列化
            let value: T = serde_json::from_slice(&serialized)?;
            return Ok(Some(value));
        }

        // 从Redis获取
        match self.redis_cache.get(&versioned_key).await? {
            Some(data) => {
                // 反序列化
                let value: T = serde_json::from_slice(&data)?;

                // 更新统计
                if let Ok(mut stats) = self.cache_stats.write() {
                    stats.hits += 1;
                    stats.redis_hits += 1;
                }

                // 写入内存缓存,添加TTL抖动
                let ttl = self.add_ttl_jitter(self.config.ttl);
                self.memory_cache.set(&versioned_key, data, ttl);

                Ok(Some(value))
            }
            None => {
                // 更新统计
                if let Ok(mut stats) = self.cache_stats.write() {
                    stats.misses += 1;
                }

                Ok(None)
            }
        }
    }

    // 获取缓存(带缓存击穿防护)
    pub async fn get_with_breakdown_protection<T: for<'de> Deserialize<'de> + Serialize>(
        &self,
        key: &str,
        data_loader: impl futures::Future<Output = Result<Option<T>>>,
    ) -> Result<Option<T>> {
        // 先尝试从缓存获取
        if let Some(value) = self.get(key).await? {
            return Ok(Some(value));
        }

        // 缓存不存在,使用互斥锁防止缓存击穿
        let lock = {
            let mut locks = match self.hot_key_locks.write() {
                Ok(l) => l,
                Err(_) => return self.get(key).await,
            };
            locks
                .entry(key.to_string())
                .or_insert_with(|| Arc::new(tokio::sync::Mutex::new(())))
                .clone()
        };

        // 获得锁后再次检查缓存,防止在获取锁的过程中其他线程已经加载了数据
        let guard = lock.lock().await;
        if let Some(value) = self.get(key).await? {
            drop(guard);
            return Ok(Some(value));
        }

        // 从数据源加载数据
        let value = data_loader.await?;
        if let Some(ref data) = value {
            // 将数据写入缓存
            self.set(key, data, None).await?;
        }

        drop(guard);
        Ok(value)
    }

    // 删除缓存
    pub async fn del(&self, key: &str) -> Result<()> {
        let versioned_key = self.get_versioned_key(key);

        // 删除内存缓存
        self.memory_cache.del(&versioned_key);

        // 删除Redis缓存
        self.redis_cache.del(&versioned_key).await?;

        // 更新统计
        if let Ok(mut stats) = self.cache_stats.write() {
            stats.deletes += 1;
            stats.invalidations += 1;
        }

        Ok(())
    }

    // 触发缓存失效
    pub async fn invalidate(&self, key: &str) -> Result<()> {
        let invalidated_keys = self.consistency_manager.invalidate(key);
        let invalidation_count = invalidated_keys.len();

        for invalidated_key in &invalidated_keys {
            self.del(invalidated_key).await?;
            // 发布分布式失效通知
            self.publish_invalidation(invalidated_key).await;
        }

        // 更新统计
        if let Ok(mut stats) = self.cache_stats.write() {
            stats.invalidations += invalidation_count as u64;
        }

        Ok(())
    }

    // 注册缓存依赖
    pub fn register_dependency(&self, key: &str, depends_on: &str) {
        self.consistency_manager
            .register_dependency(key, depends_on);
    }

    // 获取统计信息
    pub fn get_stats(&self) -> CacheStats {
        match self.cache_stats.read() {
            Ok(stats) => stats.clone(),
            Err(e) => {
                log::warn!("获取缓存统计锁失败: {}", e);
                CacheStats::default()
            }
        }
    }

    // 清空所有缓存
    pub async fn flush_all(&self) -> Result<()> {
        // 清空内存缓存
        // 注意:这里简化处理,实际应实现清空方法

        // 清空Redis缓存
        self.redis_cache.flush_all().await?;

        // 生成新的缓存版本
        if let Ok(mut version) = self.cache_version.write() {
            *version = format!("v{}", uuid::Uuid::new_v4());
            info!(
                "Flushed all cache layers and updated cache version to: {}",
                *version
            );
        }

        Ok(())
    }

    // 获取缓存版本
    pub fn get_cache_version(&self) -> String {
        match self.cache_version.read() {
            Ok(version) => version.clone(),
            Err(e) => {
                log::warn!("获取缓存版本锁失败: {}", e);
                String::new()
            }
        }
    }

    // 生成带版本的缓存键
    pub fn get_versioned_key(&self, key: &str) -> String {
        let version = self.get_cache_version();
        format!("{}:{}", version, key)
    }

    // 发布分布式缓存失效通知
    pub async fn publish_invalidation(&self, key: &str) {
        if let Some(channel) = &self.distributed_invalidation_channel {
            let _ = channel.send(key.to_string());
            debug!("Published invalidation for key: {}", key);
        }
    }

    // 订阅分布式缓存失效通知
    pub fn subscribe_invalidation(&self) -> Option<tokio::sync::broadcast::Receiver<String>> {
        self.distributed_invalidation_channel
            .as_ref()
            .map(|channel| channel.subscribe())
    }

    // 启动分布式失效监听任务
    async fn distributed_invalidation_task(&self) {
        if let Some(receiver) = self.subscribe_invalidation() {
            let mut receiver = receiver;
            info!("Started distributed invalidation listener");

            while let Ok(key) = receiver.recv().await {
                // 处理分布式失效通知
                if let Err(e) = self.del(&key).await {
                    log::error!("分布式缓存失效处理失败: {}", e);
                }
                info!("Handled distributed invalidation for key: {}", key);
            }
        }
    }
}

// 缓存策略工厂
pub struct CacheStrategyFactory;

impl CacheStrategyFactory {
    pub async fn create_unified_cache(redis_url: &str) -> Result<UnifiedCacheManager> {
        let redis_config = CacheConfig {
            url: Some(redis_url.to_string()),
            pool_size: Some(10),
            default_ttl: Duration::from_secs(300),
            max_size: 10000,
            cleanup_interval: Duration::from_secs(60),
        };

        let strategy_config = CacheStrategyConfig {
            strategy_type: CacheStrategyType::TimeToLive,
            ttl: Duration::from_secs(300),
            max_size: 10000,
            eviction_threshold: 0.8,
            refresh_interval: Duration::from_secs(60),
            monitor_interval: Duration::from_secs(30),
        };

        UnifiedCacheManager::new(redis_config, strategy_config).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_unified_cache() {
        // 这里简化测试,实际应使用测试Redis实例
        let config = CacheConfig::default();
        let strategy_config = CacheStrategyConfig::default();

        match UnifiedCacheManager::new(config, strategy_config).await {
            Ok(manager) => {
                // 测试设置缓存
                let key = "test:key";
                let value = json!("test value");

                manager
                    .set(key, &value, Some(Duration::from_secs(60)))
                    .await
                    .unwrap();

                // 测试获取缓存
                let result: Option<String> = manager.get(key).await.unwrap();
                assert_eq!(result, Some("test value".to_string()));

                // 测试删除缓存
                manager.del(key).await.unwrap();

                // 测试缓存不存在
                let result: Option<String> = manager.get(key).await.unwrap();
                assert_eq!(result, None);
            }
            Err(e) => {
                // 如果Redis不可用,测试应该跳过
                info!("Redis not available, skipping cache tests: {}", e);
            }
        }
    }

    #[test]
    fn test_memory_cache() {
        let config = CacheStrategyConfig::default();
        let memory_cache = MemoryCacheLayer::new(config);

        // 测试设置和获取
        let key = "test:key";
        let value = b"test value".to_vec();

        memory_cache.set(key, value.clone(), Duration::from_secs(60));
        let result = memory_cache.get(key);
        assert_eq!(result, Some(value));

        // 测试删除
        memory_cache.del(key);
        let result = memory_cache.get(key);
        assert_eq!(result, None);
    }
}



