//! /! Config Center Benchmark Tests
//!
//! Provides benchmark tests for configuration center storage and operations

#[cfg(test)]
mod benchmarks {
    use super::super::config_center::models::{ConfigEntry, ConfigKey};
    use super::super::config_center::storage::{
        create_storage, CompositeConfigStorage, FileConfigStorage, MemoryConfigStorage,
        StorageConfig, StorageType,
    };
    use std::sync::Arc;
    use std::time::{Duration, Instant};

    fn create_test_config_entry(key: &str, value: &str) -> ConfigEntry {
        ConfigEntry {
            key: ConfigKey {
                namespace: "test".to_string(),
                key: key.to_string(),
            },
            value: value.to_string(),
            version: 1,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    #[test]
    fn bench_memory_storage_create() {
        let start = Instant::now();
        for _ in 0..10000 {
            let _storage = MemoryConfigStorage::new(None);
        }
        let elapsed = start.elapsed();
        println!("Memory storage creation (10000 iterations): {:?}", elapsed);
        assert!(elapsed < Duration::from_secs(2));
    }

    #[test]
    fn bench_memory_storage_operations() {
        let storage = MemoryConfigStorage::new(None);
        let entry = create_test_config_entry("test_key", "test_value");

        let start = Instant::now();
        for _ in 0..1000 {
            storage.store(&entry).unwrap();
            let _result = storage.get(&entry.key).unwrap();
        }
        let elapsed = start.elapsed();
        println!("Memory storage store+get (1000 iterations): {:?}", elapsed);
        assert!(elapsed < Duration::from_secs(2));
    }

    #[test]
    fn bench_memory_storage_batch_operations() {
        let storage = MemoryConfigStorage::new(None);
        let entries: Vec<ConfigEntry> = (0..100)
            .map(|i| create_test_config_entry(&format!("key_{}", i), &format!("value_{}", i)))
            .collect();

        let start = Instant::now();
        for entry in entries {
            storage.store(&entry).unwrap();
        }
        let elapsed = start.elapsed();
        println!("Memory storage batch store (100 entries): {:?}", elapsed);
        assert!(elapsed < Duration::from_secs(1));
    }

    #[test]
    fn bench_storage_factory_memory() {
        let config = StorageConfig::memory();
        let start = Instant::now();
        for _ in 0..1000 {
            let _storage = create_storage(&config).unwrap();
        }
        let elapsed = start.elapsed();
        println!("Storage factory memory (1000 iterations): {:?}", elapsed);
        assert!(elapsed < Duration::from_secs(3));
    }

    #[test]
    fn bench_storage_config_serialization() {
        let config = StorageConfig {
            storage_type: StorageType::Memory,
            config: serde_json::json!({
                "path": "/tmp/config",
                "backup_enabled": true
            }),
        };

        let start = Instant::now();
        for _ in 0..10000 {
            let json = serde_json::to_string(&config).unwrap();
            let _parsed: StorageConfig = serde_json::from_str(&json).unwrap();
        }
        let elapsed = start.elapsed();
        println!(
            "Storage config serialization (10000 iterations): {:?}",
            elapsed
        );
        assert!(elapsed < Duration::from_secs(5));
    }

    #[test]
    fn bench_composite_storage_create() {
        let config = StorageConfig {
            storage_type: StorageType::Composite,
            config: serde_json::json!({
                "local": StorageConfig::memory(),
            }),
        };

        let start = Instant::now();
        for _ in 0..1000 {
            let _storage = create_storage(&config).unwrap();
        }
        let elapsed = start.elapsed();
        println!(
            "Composite storage creation (1000 iterations): {:?}",
            elapsed
        );
        assert!(elapsed < Duration::from_secs(3));
    }

    #[test]
    fn bench_config_key_creation() {
        let start = Instant::now();
        for i in 0..10000 {
            let _key = ConfigKey {
                namespace: format!("namespace_{}", i % 10),
                key: format!("key_{}", i),
            };
        }
        let elapsed = start.elapsed();
        println!("Config key creation (10000 iterations): {:?}", elapsed);
        assert!(elapsed < Duration::from_secs(1));
    }

    #[test]
    fn bench_config_entry_serialization() {
        let entry = create_test_config_entry("test_key", "test_value");

        let start = Instant::now();
        for _ in 0..10000 {
            let json = serde_json::to_string(&entry).unwrap();
            let _parsed: ConfigEntry = serde_json::from_str(&json).unwrap();
        }
        let elapsed = start.elapsed();
        println!(
            "Config entry serialization (10000 iterations): {:?}",
            elapsed
        );
        assert!(elapsed < Duration::from_secs(5));
    }

    #[test]
    fn bench_memory_storage_list() {
        let storage = MemoryConfigStorage::new(None);
        for i in 0..100 {
            let entry = create_test_config_entry(&format!("key_{}", i), &format!("value_{}", i));
            storage.store(&entry).unwrap();
        }

        let start = Instant::now();
        for _ in 0..1000 {
            let _entries = storage.list(&"test".to_string()).unwrap();
        }
        let elapsed = start.elapsed();
        println!("Memory storage list (1000 iterations): {:?}", elapsed);
        assert!(elapsed < Duration::from_secs(2));
    }

    #[test]
    fn bench_storage_type_enum() {
        let start = Instant::now();
        for _ in 0..100000 {
            let _t1 = StorageType::Memory;
            let _t2 = StorageType::File;
            let _t3 = StorageType::Redis;
            let _t4 = StorageType::Composite;
        }
        let elapsed = start.elapsed();
        println!(
            "Storage type enum operations (100000 iterations): {:?}",
            elapsed
        );
        assert!(elapsed < Duration::from_secs(2));
    }
}
