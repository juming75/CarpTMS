//! / 查询性能监控模块
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use tracing::{debug, warn};

/// 查询性能指标
#[derive(Debug, Clone)]
pub struct QueryMetrics {
    /// 总查询次数
    pub total_queries: u64,
    /// 慢查询次数(>100ms)
    pub slow_queries: u64,
    /// 失败查询次数
    pub failed_queries: u64,
    /// 平均查询时间(毫秒)
    pub avg_duration_ms: f64,
    /// 最大查询时间(毫秒)
    pub max_duration_ms: u64,
}

/// 查询性能监控器
pub struct QueryMonitor {
    /// 总查询次数
    total_queries: AtomicU64,
    /// 慢查询次数
    slow_queries: AtomicU64,
    /// 失败查询次数
    failed_queries: AtomicU64,
    /// 总查询时间(毫秒)
    total_duration_ms: AtomicU64,
    /// 最大查询时间(毫秒)
    max_duration_ms: AtomicU64,
}

impl QueryMonitor {
    /// 创建新的监控器
    pub fn new() -> Self {
        Self {
            total_queries: AtomicU64::new(0),
            slow_queries: AtomicU64::new(0),
            failed_queries: AtomicU64::new(0),
            total_duration_ms: AtomicU64::new(0),
            max_duration_ms: AtomicU64::new(0),
        }
    }

    /// 记录查询开始
    pub fn start_query(&self) -> QueryTimer<'_> {
        QueryTimer::new(self)
    }

    /// 记录查询结果
    pub fn record_query(&self, duration: Duration, success: bool) {
        let duration_ms = duration.as_millis() as u64;

        // 更新总查询次数
        self.total_queries.fetch_add(1, Ordering::Relaxed);

        // 更新总查询时间
        self.total_duration_ms
            .fetch_add(duration_ms, Ordering::Relaxed);

        // 更新最大查询时间
        let mut current_max = self.max_duration_ms.load(Ordering::Relaxed);
        while duration_ms > current_max {
            match self.max_duration_ms.compare_exchange_weak(
                current_max,
                duration_ms,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(new_max) => current_max = new_max,
            }
        }

        // 判断是否为慢查询
        if duration_ms > 100 {
            let count = self.slow_queries.fetch_add(1, Ordering::Relaxed);
            warn!("Slow query detected #{}: {}ms", count + 1, duration_ms);
        }

        // 记录失败查询
        if !success {
            self.failed_queries.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// 获取性能指标
    pub fn get_metrics(&self) -> QueryMetrics {
        let total = self.total_queries.load(Ordering::Relaxed);
        let slow = self.slow_queries.load(Ordering::Relaxed);
        let failed = self.failed_queries.load(Ordering::Relaxed);
        let total_duration = self.total_duration_ms.load(Ordering::Relaxed);
        let max = self.max_duration_ms.load(Ordering::Relaxed);

        let avg = if total > 0 {
            total_duration as f64 / total as f64
        } else {
            0.0
        };

        QueryMetrics {
            total_queries: total,
            slow_queries: slow,
            failed_queries: failed,
            avg_duration_ms: avg,
            max_duration_ms: max,
        }
    }

    /// 重置指标
    pub fn reset(&self) {
        self.total_queries.store(0, Ordering::Relaxed);
        self.slow_queries.store(0, Ordering::Relaxed);
        self.failed_queries.store(0, Ordering::Relaxed);
        self.total_duration_ms.store(0, Ordering::Relaxed);
        self.max_duration_ms.store(0, Ordering::Relaxed);
    }

    /// 打印性能报告
    pub fn print_report(&self) {
        let metrics = self.get_metrics();

        println!("=== Query Performance Report ===");
        println!("Total queries: {}", metrics.total_queries);
        println!("Slow queries (>100ms): {}", metrics.slow_queries);
        println!("Failed queries: {}", metrics.failed_queries);
        println!("Average duration: {:.2}ms", metrics.avg_duration_ms);
        println!("Max duration: {}ms", metrics.max_duration_ms);

        if metrics.total_queries > 0 {
            let slow_ratio = (metrics.slow_queries as f64 / metrics.total_queries as f64) * 100.0;
            let fail_ratio = (metrics.failed_queries as f64 / metrics.total_queries as f64) * 100.0;
            println!("Slow query ratio: {:.2}%", slow_ratio);
            println!("Failure ratio: {:.2}%", fail_ratio);
        }
        println!("===============================");
    }
}

impl Default for QueryMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// 查询计时器
pub struct QueryTimer<'a> {
    monitor: &'a QueryMonitor,
    start: Instant,
    query_name: Option<String>,
}

impl<'a> QueryTimer<'a> {
    /// 创建新的计时器
    pub fn new(monitor: &'a QueryMonitor) -> Self {
        Self {
            monitor,
            start: Instant::now(),
            query_name: None,
        }
    }

    /// 设置查询名称
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.query_name = Some(name.into());
        self
    }

    /// 结束计时并记录成功查询
    pub fn finish_success(self) {
        let duration = self.start.elapsed();
        if let Some(name) = &self.query_name {
            debug!("Query '{}' succeeded in {:?}", name, duration);
        }
        self.monitor.record_query(duration, true);
    }

    /// 结束计时并记录失败查询
    pub fn finish_failure(self) {
        let duration = self.start.elapsed();
        if let Some(name) = &self.query_name {
            debug!("Query '{}' failed in {:?}", name, duration);
        }
        self.monitor.record_query(duration, false);
    }
}

impl<'a> Drop for QueryTimer<'a> {
    fn drop(&mut self) {
        // 如果未显式调用finish_success或finish_failure,记录为成功
        // 这样可以自动捕获查询完成
        let duration = self.start.elapsed();
        self.monitor.record_query(duration, true);
    }
}

/// 便捷函数:监控查询
pub async fn monitor_query<F, T, E>(
    monitor: &QueryMonitor,
    query_name: &str,
    operation: F,
) -> Result<T, E>
where
    F: std::future::Future<Output = Result<T, E>>,
{
    let timer = monitor.start_query().with_name(query_name);

    match operation.await {
        Ok(result) => {
            timer.finish_success();
            Ok(result)
        }
        Err(e) => {
            timer.finish_failure();
            Err(e)
        }
    }
}

lazy_static::lazy_static! {
    pub static ref GLOBAL_QUERY_MONITOR: QueryMonitor = QueryMonitor::new();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_monitor_creation() {
        let monitor = QueryMonitor::new();
        let metrics = monitor.get_metrics();
        assert_eq!(metrics.total_queries, 0);
        assert_eq!(metrics.slow_queries, 0);
    }

    #[test]
    fn test_record_fast_query() {
        let monitor = QueryMonitor::new();
        monitor.record_query(Duration::from_millis(50), true);

        let metrics = monitor.get_metrics();
        assert_eq!(metrics.total_queries, 1);
        assert_eq!(metrics.slow_queries, 0);
        assert_eq!(metrics.failed_queries, 0);
    }

    #[test]
    fn test_record_slow_query() {
        let monitor = QueryMonitor::new();
        monitor.record_query(Duration::from_millis(150), true);

        let metrics = monitor.get_metrics();
        assert_eq!(metrics.total_queries, 1);
        assert_eq!(metrics.slow_queries, 1);
    }

    #[test]
    fn test_record_failed_query() {
        let monitor = QueryMonitor::new();
        monitor.record_query(Duration::from_millis(50), false);

        let metrics = monitor.get_metrics();
        assert_eq!(metrics.total_queries, 1);
        assert_eq!(metrics.failed_queries, 1);
    }

    #[test]
    fn test_max_duration() {
        let monitor = QueryMonitor::new();
        monitor.record_query(Duration::from_millis(100), true);
        monitor.record_query(Duration::from_millis(200), true);
        monitor.record_query(Duration::from_millis(50), true);

        let metrics = monitor.get_metrics();
        assert_eq!(metrics.max_duration_ms, 200);
    }

    #[test]
    fn test_avg_duration() {
        let monitor = QueryMonitor::new();
        monitor.record_query(Duration::from_millis(100), true);
        monitor.record_query(Duration::from_millis(200), true);
        monitor.record_query(Duration::from_millis(300), true);

        let metrics = monitor.get_metrics();
        assert_eq!(metrics.avg_duration_ms, 200.0);
    }

    #[test]
    fn test_reset() {
        let monitor = QueryMonitor::new();
        monitor.record_query(Duration::from_millis(100), true);

        monitor.reset();

        let metrics = monitor.get_metrics();
        assert_eq!(metrics.total_queries, 0);
        assert_eq!(metrics.max_duration_ms, 0);
    }

    #[tokio::test]
    async fn test_query_timer() {
        let monitor = QueryMonitor::new();

        let timer = monitor.start_query();
        tokio::time::sleep(Duration::from_millis(10)).await;
        timer.finish_success();

        let metrics = monitor.get_metrics();
        assert_eq!(metrics.total_queries, 1);
    }

    #[tokio::test]
    async fn test_monitor_query() {
        let monitor = QueryMonitor::new();

        async fn test_operation() -> Result<i32, &'static str> {
            tokio::time::sleep(Duration::from_millis(10)).await;
            Ok(42)
        }

        let result = monitor_query(&monitor, "test_query", test_operation()).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);

        let metrics = monitor.get_metrics();
        assert_eq!(metrics.total_queries, 1);
    }
}
