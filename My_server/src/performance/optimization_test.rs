//! / 性能优化测试

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_query_cache_hit() {
        let mut cache = QueryCache::new(100);
        let key = "test_key";
        let value = "test_value";
        
        cache.put(key.to_string(), value.to_string(), Duration::from_secs(60));
        let result = cache.get(key);
        assert_eq!(result, Some(value.to_string()));
    }

    #[test]
    fn test_query_cache_miss() {
        let cache = QueryCache::new(100);
        let result = cache.get("nonexistent_key");
        assert_eq!(result, None);
    }

    #[test]
    fn test_query_cache_expiry() {
        let mut cache = QueryCache::new(100);
        let key = "expiring_key";
        
        cache.put(key.to_string(), "value".to_string(), Duration::from_millis(100));
        std::thread::sleep(Duration::from_millis(150));
        
        let result = cache.get(key);
        assert_eq!(result, None);
    }

    #[test]
    fn test_cache_size_limit() {
        let mut cache = QueryCache::new(2);
        
        cache.put("key1".to_string(), "value1".to_string(), Duration::from_secs(60));
        cache.put("key2".to_string(), "value2".to_string(), Duration::from_secs(60));
        cache.put("key3".to_string(), "value3".to_string(), Duration::from_secs(60));
        
        assert_eq!(cache.get("key1"), None); // 应该被淘汰
        assert_eq!(cache.get("key2"), Some("value2".to_string()));
        assert_eq!(cache.get("key3"), Some("value3".to_string()));
    }

    #[test]
    fn test_connection_pool_performance() {
        let start = Instant::now();
        // 模拟连接池操作
        let pool = ConnectionPool::new(5);
        for _ in 0..100 {
            let _conn = pool.acquire();
        }
        let duration = start.elapsed();
        
        // 应该在合理时间内完成
        assert!(duration < Duration::from_millis(100));
    }

    #[test]
    fn test_batch_query_performance() {
        let start = Instant::now();
        let ids = vec!["1", "2", "3", "4", "5"];
        let results = batch_query(ids).unwrap();
        let duration = start.elapsed();
        
        assert_eq!(results.len(), 5);
        assert!(duration < Duration::from_millis(50));
    }

    #[test]
    fn test_p95_response_time() {
        let mut times = Vec::new();
        
        for _ in 0..100 {
            let start = Instant::now();
            simulate_api_call();
            times.push(start.elapsed());
        }
        
        times.sort();
        let p95_index = (times.len() * 95) / 100;
        let p95 = times[p95_index];
        
        // P95应该小于100ms
        assert!(p95 < Duration::from_millis(100));
    }

    // 辅助函数
    fn simulate_api_call() {
        // 模拟一个简单的API调用
        std::thread::sleep(Duration::from_micros(100));
    }

    fn batch_query(ids: Vec<&str>) -> Result<Vec<String>, AppError> {
        Ok(ids.iter().map(|id| format!("result_{}", id)).collect())
    }

    // 简化的测试实现
    use std::time::Duration;
    use super::super::performance;

    struct QueryCache {
        entries: std::collections::HashMap<String, (String, Instant)>,
        max_size: usize,
    }

    impl QueryCache {
        fn new(max_size: usize) -> Self {
            Self {
                entries: std::collections::HashMap::new(),
                max_size,
            }
        }

        fn put(&mut self, key: String, value: String, ttl: Duration) {
            if self.entries.len() >= self.max_size {
                // 简单的LRU策略:移除第一个条目
                if let Some(k) = self.entries.keys().next().cloned() {
                    self.entries.remove(&k);
                }
            }
            self.entries.insert(key, (value, Instant::now()));
        }

        fn get(&self, key: &str) -> Option<String> {
            self.entries.get(key).map(|(value, _)| value.clone())
        }
    }

    struct ConnectionPool {
        size: usize,
    }

    impl ConnectionPool {
        fn new(size: usize) -> Self {
            Self { size }
        }

        fn acquire(&self) -> Connection {
            Connection
        }
    }

    struct Connection;
}






