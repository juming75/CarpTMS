//! / 缓存指标模块
// 用于监控缓存性能和健康状况

use std::sync::atomic::{AtomicU64, Ordering};

/// 缓存指标
pub struct CacheMetrics {
    hits: AtomicU64,
    misses: AtomicU64,
    errors: AtomicU64,
    sets: AtomicU64,
    deletes: AtomicU64,
}

impl CacheMetrics {
    pub fn new() -> Self {
        Self {
            hits: AtomicU64::new(0),
            misses: AtomicU64::new(0),
            errors: AtomicU64::new(0),
            sets: AtomicU64::new(0),
            deletes: AtomicU64::new(0),
        }
    }

    /// 记录缓存命中
    pub fn record_hit(&self) {
        self.hits.fetch_add(1, Ordering::Relaxed);
    }

    /// 记录缓存未命中
    pub fn record_miss(&self) {
        self.misses.fetch_add(1, Ordering::Relaxed);
    }

    /// 记录缓存错误
    pub fn record_error(&self) {
        self.errors.fetch_add(1, Ordering::Relaxed);
    }

    /// 记录缓存设置操作
    pub fn record_set(&self) {
        self.sets.fetch_add(1, Ordering::Relaxed);
    }

    /// 记录缓存删除操作
    pub fn record_delete(&self) {
        self.deletes.fetch_add(1, Ordering::Relaxed);
    }

    /// 获取缓存命中率
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits.load(Ordering::Relaxed) + self.misses.load(Ordering::Relaxed);
        if total == 0 {
            0.0
        } else {
            self.hits.load(Ordering::Relaxed) as f64 / total as f64
        }
    }

    /// 获取总请求数
    pub fn total_requests(&self) -> u64 {
        self.hits.load(Ordering::Relaxed) + self.misses.load(Ordering::Relaxed)
    }

    /// 获取错误数
    pub fn error_count(&self) -> u64 {
        self.errors.load(Ordering::Relaxed)
    }

    /// 重置指标
    pub fn reset(&self) {
        self.hits.store(0, Ordering::Relaxed);
        self.misses.store(0, Ordering::Relaxed);
        self.errors.store(0, Ordering::Relaxed);
        self.sets.store(0, Ordering::Relaxed);
        self.deletes.store(0, Ordering::Relaxed);
    }
}

impl Default for CacheMetrics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_metrics() {
        let metrics = CacheMetrics::new();

        metrics.record_hit();
        metrics.record_hit();
        metrics.record_miss();

        assert_eq!(metrics.total_requests(), 3);
        assert!((metrics.hit_rate() - 0.666).abs() < 0.001);
    }
}




