//! 车辆缓存模块单元测试

#[cfg(test)]
mod tests {
    use crate::cache::vehicle_cache::{BatchConfig, VehicleCache};

    #[test]
    fn test_batch_config_default() {
        let config = BatchConfig::default();
        assert_eq!(config.max_batch_size, 1000);
        assert_eq!(config.max_concurrent, 50);
    }

    #[test]
    fn test_batch_config_custom() {
        let config = BatchConfig {
            max_batch_size: 500,
            max_concurrent: 10,
        };
        assert_eq!(config.max_batch_size, 500);
        assert_eq!(config.max_concurrent, 10);
    }

    #[test]
    fn test_vehicle_cache_default_creation() {
        // 测试默认创建（无Redis模式）
        let cache = VehicleCache::default();
        // 默认配置应该可以正常创建
        assert!(std::mem::size_of_val(&cache) > 0);
    }

    #[test]
    fn test_batch_config_bounds() {
        // 测试批量配置边界值
        let small_config = BatchConfig {
            max_batch_size: 1,
            max_concurrent: 1,
        };
        assert_eq!(small_config.max_batch_size, 1);

        let large_config = BatchConfig {
            max_batch_size: usize::MAX,
            max_concurrent: usize::MAX,
        };
        assert_eq!(large_config.max_batch_size, usize::MAX);
    }
}
