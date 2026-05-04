//! / 性能优化实现

use std::time::Duration;
use crate::errors::AppResult;

/// 查询缓存
pub struct QueryCache {
    entries: lru::LruCache<String, CacheEntry>,
    ttl: Duration,
}

#[derive(Clone)]
struct CacheEntry {
    data: Vec<u8>,
    created_at: std::time::Instant,
}

impl QueryCache {
    pub fn new(capacity: usize, ttl_seconds: u64) -> Self {
        Self {
            entries: lru::LruCache::new(capacity),
            ttl: Duration::from_secs(ttl_seconds),
        }
    }

    pub fn get<T: serde::de::DeserializeOwned>(&mut self, key: &str) -> Option<T> {
        if let Some(entry) = self.entries.get(key) {
            if entry.created_at.elapsed() < self.ttl {
                if let Ok(data) = serde_json::from_slice::<T>(&entry.data) {
                    return Some(data);
                }
            }
        }
        None
    }

    pub fn put<T: serde::Serialize>(&mut self, key: String, value: &T) -> AppResult<()> {
        let data = serde_json::to_vec(value)?;
        self.entries.put(key, CacheEntry {
            data,
            created_at: std::time::Instant::now(),
        });
        Ok(())
    }

    pub fn invalidate(&mut self, key: &str) {
        self.entries.pop(key);
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

/// 连接池配置
pub struct ConnectionPoolConfig {
    pub min_size: u32,
    pub max_size: u32,
    pub acquire_timeout: Duration,
    pub idle_timeout: Duration,
    pub max_lifetime: Duration,
}

impl Default for ConnectionPoolConfig {
    fn default() -> Self {
        Self {
            min_size: 5,
            max_size: 20,
            acquire_timeout: Duration::from_secs(30),
            idle_timeout: Duration::from_secs(600),
            max_lifetime: Duration::from_secs(3600),
        }
    }
}

/// 批量查询优化
pub async fn batch_query_optimized<F, T, Fut>(
    ids: Vec<String>,
    query_fn: F,
    batch_size: usize,
) -> AppResult<Vec<T>>
where
    F: Fn(Vec<String>) -> Fut,
    Fut: std::future::Future<Output = AppResult<Vec<T>>>,
{
    let mut results = Vec::new();
    
    for chunk in ids.chunks(batch_size) {
        let chunk_ids = chunk.to_vec();
        let chunk_results = query_fn(chunk_ids).await?;
        results.extend(chunk_results);
    }
    
    Ok(results)
}

/// N+1查询优化
pub trait EagerLoad {
    type Entity;
    
    async fn with_relations(
        &self,
        relations: &[&str],
    ) -> AppResult<Vec<Self::Entity>>;
}

/// 查询性能监控
pub struct QueryMonitor {
    queries: std::sync::Arc<std::sync::Mutex<Vec<QueryMetric>>>,
}

#[derive(Debug, Clone)]
pub struct QueryMetric {
    pub query_type: String,
    pub duration_ms: u64,
    pub success: bool,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl QueryMonitor {
    pub fn new() -> Self {
        Self {
            queries: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
        }
    }

    pub fn record(&self, metric: QueryMetric) {
        if let Ok(mut queries) = self.queries.lock() {
            queries.push(metric);
            if queries.len() > 1000 {
                queries.remove(0);
            }
        }
    }

    pub fn get_p95_latency(&self) -> u64 {
        if let Ok(queries) = self.queries.lock() {
            if queries.is_empty() {
                return 0;
            }
            let mut latencies: Vec<u64> = queries.iter()
                .filter(|q| q.success)
                .map(|q| q.duration_ms)
                .collect();
            latencies.sort();
            let index = (latencies.len() * 95) / 100;
            return latencies.get(index).copied().unwrap_or(0);
        }
        0
    }

    pub fn get_avg_latency(&self) -> u64 {
        if let Ok(queries) = self.queries.lock() {
            if queries.is_empty() {
                return 0;
            }
            let latencies: Vec<u64> = queries.iter()
                .filter(|q| q.success)
                .map(|q| q.duration_ms)
                .collect();
            let sum: u64 = latencies.iter().sum();
            return sum / latencies.len() as u64;
        }
        0
    }
}

/// 并发查询
pub async fn parallel_query<F, Fut, T>(queries: Vec<F>) -> AppResult<Vec<T>>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = AppResult<T>>,
{
    let handles: Vec<_> = queries.into_iter()
        .map(|query| tokio::spawn(query()))
        .collect();
    
    let mut results = Vec::new();
    for handle in handles {
        let result = handle.await??;
        results.push(result);
    }
    
    Ok(results)
}

/// 流式处理
pub async fn stream_process<F, Fut, T>(
    items: Vec<T>,
    processor: F,
) -> AppResult<Vec<T::Output>>
where
    T: Send + 'static,
    F: Fn(T) -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = AppResult<T::Output>> + Send,
    T::Output: Send,
{
    let (sender, mut receiver) = tokio::sync::mpsc::channel(100);
    let mut results = Vec::new();
    
    // 启动处理任务
    let processor_handle = tokio::spawn(async move {
        for item in items {
            let result = processor(item).await;
            if let Ok(output) = result {
                let _ = sender.send(output).await;
            }
        }
    });
    
    // 收集结果
    while let Some(result) = receiver.recv().await {
        results.push(result);
    }
    
    processor_handle.await?;
    Ok(results)
}






