//! 缓存模块单元测试

#[cfg(test)]
mod tests {
    use std::time::Duration;

    // ============= MemoryCache 测试 =============

    #[test]
    fn test_memory_cache_new() {
        use crate::cache::memory_cache::MemoryCache;

        let cache = MemoryCache::<String>::new(Duration::from_secs(300), 1000);
        assert!(cache.get("test").is_none());
    }

    #[test]
    fn test_memory_cache_set_and_get() {
        use crate::cache::memory_cache::MemoryCache;

        let cache = MemoryCache::<String>::new(Duration::from_secs(300), 1000);
        cache.set("key1", "value1", None);

        let result = cache.get("key1");
        assert_eq!(result, Some("value1".to_string()));
    }

    #[test]
    fn test_memory_cache_get_nonexistent() {
        use crate::cache::memory_cache::MemoryCache;

        let cache = MemoryCache::<String>::new(Duration::from_secs(300), 1000);
        assert!(cache.get("nonexistent").is_none());
    }

    #[test]
    fn test_memory_cache_delete() {
        use crate::cache::memory_cache::MemoryCache;

        let cache = MemoryCache::<String>::new(Duration::from_secs(300), 1000);
        cache.set("key1", "value1", None);
        assert_eq!(cache.get("key1"), Some("value1".to_string()));

        cache.del("key1");
        assert!(cache.get("key1").is_none());
    }

    #[test]
    fn test_memory_cache_with_custom_ttl() {
        use crate::cache::memory_cache::MemoryCache;

        let cache = MemoryCache::<String>::new(Duration::from_secs(300), 1000);
        cache.set("key1", "value1", Some(Duration::from_millis(10)));

        // 立即获取应该存在
        assert_eq!(cache.get("key1"), Some("value1".to_string()));
    }

    #[test]
    fn test_memory_cache_clone() {
        use crate::cache::memory_cache::MemoryCache;

        let cache1 = MemoryCache::<String>::new(Duration::from_secs(300), 1000);
        cache1.set("key1", "value1", None);

        let cache2 = cache1.clone();
        assert_eq!(cache2.get("key1"), Some("value1".to_string()));
    }

    #[test]
    fn test_memory_cache_default() {
        use crate::cache::memory_cache::MemoryCache;

        let cache = MemoryCache::<String>::default();
        // 默认缓存应该可以正常工作
        cache.set("key1", "value1", None);
        assert_eq!(cache.get("key1"), Some("value1".to_string()));
    }
}
