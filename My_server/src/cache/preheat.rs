use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use tokio::sync::{mpsc, RwLock};
use tracing::{error, info};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachePreheatConfig {
    pub max_cache_size: usize,
    pub preheat_interval: Duration,
    pub access_threshold: u64,
    pub time_window: Duration,
    pub enable_smart_preheating: bool,
    pub enable_access_pattern_analysis: bool,
    pub preheat_batch_size: usize,
    pub cache_ttl: Duration,
    pub enable_metrics: bool,
}

impl Default for CachePreheatConfig {
    fn default() -> Self {
        Self {
            max_cache_size: 10000,
            preheat_interval: Duration::from_secs(300), // 5 minutes
            access_threshold: 10,
            time_window: Duration::from_secs(3600), // 1 hour
            enable_smart_preheating: true,
            enable_access_pattern_analysis: true,
            preheat_batch_size: 100,
            cache_ttl: Duration::from_secs(1800), // 30 minutes
            enable_metrics: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CacheEntry {
    pub key: String,
    pub data: Vec<u8>,
    pub access_count: u64,
    pub last_accessed: Instant,
    pub created_at: Instant,
    pub expires_at: Instant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessPattern {
    pub key: String,
    pub access_times: VecDeque<u64>,
    pub access_frequency: f64,
    pub predicted_next_access: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreheatMetrics {
    pub total_preheated_keys: u64,
    pub cache_hit_rate: f64,
    pub preheat_success_rate: f64,
    pub average_preheat_time: Duration,
    pub memory_usage: usize,
    pub active_keys: usize,
    pub expired_keys: u64,
}

#[derive(Debug)]
pub struct CachePreheatManager {
    config: CachePreheatConfig,
    cache: Arc<RwLock<HashMap<String, CacheEntry>>>,
    access_patterns: Arc<RwLock<HashMap<String, AccessPattern>>>,
    preheat_queue: Arc<RwLock<VecDeque<String>>>,
    metrics: Arc<RwLock<PreheatMetrics>>,
    shutdown_tx: Arc<RwLock<Option<mpsc::Sender<()>>>>,
}

impl CachePreheatManager {
    pub fn new(config: CachePreheatConfig) -> Self {
        let initial_metrics = PreheatMetrics {
            total_preheated_keys: 0,
            cache_hit_rate: 0.0,
            preheat_success_rate: 0.0,
            average_preheat_time: Duration::from_secs(0),
            memory_usage: 0,
            active_keys: 0,
            expired_keys: 0,
        };

        Self {
            config,
            cache: Arc::new(RwLock::new(HashMap::new())),
            access_patterns: Arc::new(RwLock::new(HashMap::new())),
            preheat_queue: Arc::new(RwLock::new(VecDeque::new())),
            metrics: Arc::new(RwLock::new(initial_metrics)),
            shutdown_tx: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn start(&self) -> Result<(), CachePreheatError> {
        // 创建一个新的shutdown通道
        let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(1);

        // 存储shutdown_tx
        *self.shutdown_tx.write().await = Some(shutdown_tx);

        let config = self.config.clone();
        let cache = self.cache.clone();
        let access_patterns = self.access_patterns.clone();
        let preheat_queue = self.preheat_queue.clone();
        let metrics = self.metrics.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(config.preheat_interval);

            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        if let Err(e) = Self::perform_preheating(
                            &config,
                            &cache,
                            &access_patterns,
                            &preheat_queue,
                            &metrics,
                        ).await {
                            error!("Cache preheating failed: {}", e);
                        }
                    }
                    _ = shutdown_rx.recv() => {
                        info!("Cache preheating service shutting down");
                        break;
                    }
                }
            }
        });

        // Start cache cleanup task
        self.start_cache_cleanup_task();

        info!("Cache preheat manager started");
        Ok(())
    }

    pub async fn stop(&self) -> Result<(), CachePreheatError> {
        if let Some(tx) = self.shutdown_tx.write().await.take() {
            tx.send(())
                .await
                .map_err(|_| CachePreheatError::ShutdownFailed)?;
        }
        info!("Cache preheat manager stopped");
        Ok(())
    }

    pub async fn get(&self, key: &str) -> Option<Vec<u8>> {
        let mut cache = self.cache.write().await;

        if let Some(entry) = cache.get_mut(key) {
            // Update access statistics
            entry.access_count += 1;
            entry.last_accessed = Instant::now();

            // Check if entry is expired
            if entry.expires_at < Instant::now() {
                cache.remove(key);

                // Update metrics
                let mut metrics = self.metrics.write().await;
                metrics.expired_keys += 1;

                return None;
            }

            // Record access pattern
            self.record_access_pattern(key).await;

            Some(entry.data.clone())
        } else {
            None
        }
    }

    pub async fn set(&self, key: String, data: Vec<u8>) -> Result<(), CachePreheatError> {
        let mut cache = self.cache.write().await;

        // Check cache size limit
        if cache.len() >= self.config.max_cache_size {
            self.evict_lru_entries(&mut cache).await?;
        }

        let now = Instant::now();
        let entry = CacheEntry {
            key: key.clone(),
            data,
            access_count: 1,
            last_accessed: now,
            created_at: now,
            expires_at: now + self.config.cache_ttl,
        };

        cache.insert(key.clone(), entry);

        // Update metrics
        let mut metrics = self.metrics.write().await;
        metrics.active_keys = cache.len();
        metrics.memory_usage = self.estimate_memory_usage(&cache);

        info!("Cache entry set: {} (cache size: {})", key, cache.len());
        Ok(())
    }

    pub async fn preheat_keys(&self, keys: Vec<String>) -> Result<u64, CachePreheatError> {
        let mut preheated_count = 0u64;
        let mut preheat_queue = self.preheat_queue.write().await;

        for key in keys {
            if !self.cache.read().await.contains_key(&key) {
                preheat_queue.push_back(key);
                preheated_count += 1;
            }
        }

        info!("Added {} keys to preheat queue", preheated_count);
        Ok(preheated_count)
    }

    pub async fn analyze_access_patterns(
        &self,
    ) -> Result<AccessPatternAnalysis, CachePreheatError> {
        let access_patterns = self.access_patterns.read().await;
        let cache = self.cache.read().await;

        let mut hot_keys = Vec::new();
        let mut cold_keys = Vec::new();
        let mut total_accesses = 0u64;

        for (key, pattern) in access_patterns.iter() {
            total_accesses += pattern.access_times.len() as u64;

            if pattern.access_frequency > self.config.access_threshold as f64 {
                hot_keys.push(key.clone());
            } else {
                cold_keys.push(key.clone());
            }
        }

        let cache_hit_rate = self.calculate_cache_hit_rate().await;
        let average_access_frequency = if !access_patterns.is_empty() {
            access_patterns
                .values()
                .map(|p| p.access_frequency)
                .sum::<f64>()
                / access_patterns.len() as f64
        } else {
            0.0
        };

        Ok(AccessPatternAnalysis {
            hot_keys,
            cold_keys,
            total_accesses,
            cache_hit_rate,
            average_access_frequency,
            active_patterns: access_patterns.len(),
            cache_size: cache.len(),
        })
    }

    pub async fn get_metrics(&self) -> PreheatMetrics {
        self.metrics.read().await.clone()
    }

    pub async fn clear_cache(&self) -> Result<u64, CachePreheatError> {
        let mut cache = self.cache.write().await;
        let cleared_count = cache.len() as u64;
        cache.clear();

        // Update metrics
        let mut metrics = self.metrics.write().await;
        metrics.active_keys = 0;
        metrics.memory_usage = 0;

        info!("Cache cleared: {} entries removed", cleared_count);
        Ok(cleared_count)
    }

    // Private methods
    async fn perform_preheating(
        config: &CachePreheatConfig,
        cache: &Arc<RwLock<HashMap<String, CacheEntry>>>,
        _access_patterns: &Arc<RwLock<HashMap<String, AccessPattern>>>,
        preheat_queue: &Arc<RwLock<VecDeque<String>>>,
        metrics: &Arc<RwLock<PreheatMetrics>>,
    ) -> Result<(), CachePreheatError> {
        if !config.enable_smart_preheating {
            return Ok(());
        }

        let mut queue = preheat_queue.write().await;
        let batch_size = config.preheat_batch_size.min(queue.len());

        if batch_size == 0 {
            return Ok(());
        }

        let keys_to_preheat: Vec<String> = queue.drain(0..batch_size).collect();
        drop(queue);

        let mut success_count = 0u64;
        let start_time = Instant::now();

        for key in keys_to_preheat {
            // Simulate data loading (in real implementation, this would load actual data)
            if let Ok(data) = Self::load_data_for_key(&key).await {
                let mut cache = cache.write().await;

                if cache.len() < config.max_cache_size {
                    let entry = CacheEntry {
                        key: key.clone(),
                        data,
                        access_count: 0,
                        last_accessed: Instant::now(),
                        created_at: Instant::now(),
                        expires_at: Instant::now() + config.cache_ttl,
                    };

                    cache.insert(key, entry);
                    success_count += 1;
                }
            }
        }

        let preheat_time = start_time.elapsed();

        // Update metrics
        let mut metrics = metrics.write().await;
        metrics.total_preheated_keys += success_count;
        metrics.average_preheat_time = if success_count > 0 {
            preheat_time / success_count as u32
        } else {
            Duration::from_secs(0)
        };
        metrics.preheat_success_rate = if batch_size > 0 {
            success_count as f64 / batch_size as f64
        } else {
            0.0
        };

        info!(
            "Preheated {} keys in {:?} (success rate: {:.2}%)",
            success_count,
            preheat_time,
            metrics.preheat_success_rate * 100.0
        );

        Ok(())
    }

    async fn load_data_for_key(key: &str) -> Result<Vec<u8>, CachePreheatError> {
        // Simulate data loading - in real implementation, this would load from database or external source
        tokio::time::sleep(Duration::from_millis(10)).await;
        Ok(format!("preheated_data_for_{}", key).into_bytes())
    }

    async fn record_access_pattern(&self, key: &str) {
        if !self.config.enable_access_pattern_analysis {
            return;
        }

        let mut access_patterns = self.access_patterns.write().await;
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time always after epoch")
            .as_secs();

        if let Some(pattern) = access_patterns.get_mut(key) {
            pattern.access_times.push_back(current_time);

            // Keep only recent access times within time window
            let cutoff_time = current_time - self.config.time_window.as_secs();
            pattern.access_times.retain(|&time| time >= cutoff_time);

            // Calculate access frequency
            pattern.access_frequency =
                pattern.access_times.len() as f64 / self.config.time_window.as_secs() as f64;

            // Simple prediction (in real implementation, use more sophisticated algorithms)
            if pattern.access_times.len() >= 3 {
                let recent_accesses: Vec<u64> = pattern.access_times.iter().copied().collect();
                let intervals: Vec<u64> = recent_accesses.windows(2).map(|w| w[1] - w[0]).collect();

                if let Some(avg_interval) = intervals
                    .iter()
                    .sum::<u64>()
                    .checked_div(intervals.len() as u64)
                {
                    pattern.predicted_next_access = Some(current_time + avg_interval);
                }
            }
        } else {
            let mut access_times = VecDeque::new();
            access_times.push_back(current_time);

            access_patterns.insert(
                key.to_string(),
                AccessPattern {
                    key: key.to_string(),
                    access_times,
                    access_frequency: 1.0 / self.config.time_window.as_secs() as f64,
                    predicted_next_access: None,
                },
            );
        }
    }

    async fn calculate_cache_hit_rate(&self) -> f64 {
        let access_patterns = self.access_patterns.read().await;
        let cache = self.cache.read().await;

        let total_accesses = access_patterns.len() as f64;
        let cache_hits = access_patterns
            .iter()
            .filter(|(key, _)| cache.contains_key(*key))
            .count() as f64;

        if total_accesses > 0.0 {
            cache_hits / total_accesses
        } else {
            0.0
        }
    }

    async fn evict_lru_entries(
        &self,
        cache: &mut HashMap<String, CacheEntry>,
    ) -> Result<(), CachePreheatError> {
        let entries_to_remove = (cache.len() / 10).max(1); // Remove 10% or at least 1

        let mut entries: Vec<_> = cache.iter().collect();
        entries.sort_by_key(|(_, entry)| entry.last_accessed);

        let keys_to_remove: Vec<String> = entries
            .into_iter()
            .take(entries_to_remove)
            .map(|(key, _)| key.clone())
            .collect();

        for key in keys_to_remove {
            cache.remove(&key);
            info!("Evicted LRU cache entry: {}", key);
        }

        Ok(())
    }

    fn estimate_memory_usage(&self, cache: &HashMap<String, CacheEntry>) -> usize {
        cache
            .values()
            .map(|entry| entry.key.len() + entry.data.len() + std::mem::size_of::<CacheEntry>())
            .sum()
    }

    fn start_cache_cleanup_task(&self) {
        let cache = self.cache.clone();
        let _config = self.config.clone();
        let metrics = self.metrics.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60)); // Cleanup every minute

            loop {
                interval.tick().await;

                let mut cache = cache.write().await;
                let now = Instant::now();
                let mut expired_count = 0u64;

                cache.retain(|_, entry| {
                    let is_valid = entry.expires_at > now;
                    if !is_valid {
                        expired_count += 1;
                    }
                    is_valid
                });

                // Update metrics
                let mut metrics = metrics.write().await;
                metrics.expired_keys += expired_count;
                metrics.active_keys = cache.len();

                if expired_count > 0 {
                    info!("Cleaned up {} expired cache entries", expired_count);
                }
            }
        });
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessPatternAnalysis {
    pub hot_keys: Vec<String>,
    pub cold_keys: Vec<String>,
    pub total_accesses: u64,
    pub cache_hit_rate: f64,
    pub average_access_frequency: f64,
    pub active_patterns: usize,
    pub cache_size: usize,
}

#[derive(Debug, thiserror::Error)]
pub enum CachePreheatError {
    #[error("Cache is full")]
    CacheFull,

    #[error("Key not found")]
    KeyNotFound,

    #[error("Shutdown failed")]
    ShutdownFailed,

    #[error("Data loading failed")]
    DataLoadingFailed,

    #[error("Configuration error: {0}")]
    ConfigurationError(String),
}

// Convenience functions
pub fn create_cache_preheat_manager() -> CachePreheatManager {
    CachePreheatManager::new(CachePreheatConfig::default())
}

pub fn create_cache_preheat_manager_with_config(config: CachePreheatConfig) -> CachePreheatManager {
    CachePreheatManager::new(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cache_basic_operations() {
        let manager = create_cache_preheat_manager();

        // Test set and get
        let key = "test_key".to_string();
        let data = b"test_data".to_vec();

        manager.set(key.clone(), data.clone()).await.unwrap();

        let retrieved = manager.get(&key).await;
        assert_eq!(retrieved, Some(data));
    }

    #[tokio::test]
    async fn test_access_pattern_recording() {
        let manager = create_cache_preheat_manager();
        let key = "pattern_key".to_string();

        // Simulate multiple accesses
        for _ in 0..5 {
            manager.set(key.clone(), b"data".to_vec()).await.unwrap();
            manager.get(&key).await;
            tokio::time::sleep(Duration::from_millis(10)).await;
        }

        let analysis = manager.analyze_access_patterns().await.unwrap();
        assert_eq!(analysis.total_accesses, 5);
        assert!(analysis.hot_keys.contains(&key));
    }

    #[tokio::test]
    async fn test_cache_preheating() {
        let manager = create_cache_preheat_manager();

        let keys = vec!["key1".to_string(), "key2".to_string(), "key3".to_string()];
        let preheated = manager.preheat_keys(keys.clone()).await.unwrap();

        assert_eq!(preheated, 3);

        // Start the manager to process preheat queue
        // Note: In real usage, you would start the manager separately
        // For testing, we'll just verify the preheat queue was populated

        // Wait a bit for preheating to occur
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Check if keys were preheated
        for key in &keys {
            let data = manager.get(key).await;
            assert!(data.is_some());
        }
    }

    #[tokio::test]
    async fn test_cache_cleanup() {
        let mut config = CachePreheatConfig::default();
        config.cache_ttl = Duration::from_millis(50);

        let manager = create_cache_preheat_manager_with_config(config);
        let key = "expire_key".to_string();

        manager.set(key.clone(), b"data".to_vec()).await.unwrap();

        // Wait for expiration
        tokio::time::sleep(Duration::from_millis(100)).await;

        let data = manager.get(&key).await;
        assert_eq!(data, None);
    }
}



