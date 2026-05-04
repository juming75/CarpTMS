//! /! Secret Manager Benchmark Tests
//!
//! Provides benchmark tests for secret manager and key rotation

#[cfg(test)]
mod benchmarks {
    use super::super::config::secret_manager::{
        KeyRotationConfig, RotationPolicy, SecretManager, SecretType, SecretVersion,
        SecretVersionStore,
    };
    use std::sync::Arc;
    use std::time::{Duration, Instant};

    fn create_test_secret_manager() -> SecretManager {
        SecretManager::new().expect("Failed to create secret manager")
    }

    #[test]
    fn bench_secret_manager_creation() {
        let start = Instant::now();
        for _ in 0..1000 {
            let _manager = SecretManager::new().expect("Failed to create");
        }
        let elapsed = start.elapsed();
        println!("Secret manager creation (1000 iterations): {:?}", elapsed);
        assert!(elapsed < Duration::from_secs(5));
    }

    #[test]
    fn bench_key_generation_jwt() {
        let manager = create_test_secret_manager();
        let start = Instant::now();
        for _ in 0..100 {
            let _key = manager.generate_key(SecretType::JwtSecret, 32);
        }
        let elapsed = start.elapsed();
        println!("JWT key generation (100 iterations): {:?}", elapsed);
        assert!(elapsed < Duration::from_secs(2));
    }

    #[test]
    fn bench_key_generation_api() {
        let manager = create_test_secret_manager();
        let start = Instant::now();
        for _ in 0..100 {
            let _key = manager.generate_key(SecretType::ApiKey, 64);
        }
        let elapsed = start.elapsed();
        println!("API key generation (100 iterations): {:?}", elapsed);
        assert!(elapsed < Duration::from_secs(2));
    }

    #[test]
    fn bench_secret_rotation() {
        let manager = create_test_secret_manager();
        let start = Instant::now();
        for _ in 0..50 {
            let _version = manager
                .rotate_secret(SecretType::JwtSecret)
                .expect("Failed to rotate");
        }
        let elapsed = start.elapsed();
        println!("Secret rotation (50 iterations): {:?}", elapsed);
        assert!(elapsed < Duration::from_secs(5));
    }

    #[test]
    fn bench_get_secret() {
        let manager = create_test_secret_manager();
        manager
            .store_secret(SecretType::JwtSecret, "test_secret_123".to_string())
            .expect("Failed to store");
        let start = Instant::now();
        for _ in 0..1000 {
            let _secret = manager.get_secret(SecretType::JwtSecret);
        }
        let elapsed = start.elapsed();
        println!("Get secret (1000 iterations): {:?}", elapsed);
        assert!(elapsed < Duration::from_secs(3));
    }

    #[test]
    fn bench_get_all_valid_secrets() {
        let manager = create_test_secret_manager();
        for i in 0..5 {
            manager
                .store_secret(SecretType::JwtSecret, format!("secret_v{}", i))
                .expect("Failed to store");
        }
        let start = Instant::now();
        for _ in 0..100 {
            let _secrets = manager.get_all_valid_secrets(SecretType::JwtSecret);
        }
        let elapsed = start.elapsed();
        println!("Get all valid secrets (100 iterations): {:?}", elapsed);
        assert!(elapsed < Duration::from_secs(2));
    }

    #[test]
    fn bench_rotation_config_creation() {
        let start = Instant::now();
        for _ in 0..10000 {
            let _config = KeyRotationConfig::default();
        }
        let elapsed = start.elapsed();
        println!("Rotation config default (10000 iterations): {:?}", elapsed);
        assert!(elapsed < Duration::from_secs(1));
    }

    #[test]
    fn bench_rotation_policy_serialization() {
        let policy = RotationPolicy {
            enabled: true,
            auto_rotate: true,
            rotation_interval_hours: 720,
            grace_period_hours: 24,
        };

        let start = Instant::now();
        for _ in 0..5000 {
            let json = serde_json::to_string(&policy).unwrap();
            let _parsed: RotationPolicy = serde_json::from_str(&json).unwrap();
        }
        let elapsed = start.elapsed();
        println!(
            "Rotation policy serialization (5000 iterations): {:?}",
            elapsed
        );
        assert!(elapsed < Duration::from_secs(3));
    }

    #[test]
    fn bench_secret_type_enum_operations() {
        let start = Instant::now();
        for _ in 0..100000 {
            let _t1 = SecretType::JwtSecret;
            let _t2 = SecretType::ApiKey;
            let _t3 = SecretType::EncryptionKey;
            let _t4 = SecretType::DatabasePassword;
        }
        let elapsed = start.elapsed();
        println!("Secret type enum (100000 iterations): {:?}", elapsed);
        assert!(elapsed < Duration::from_secs(2));
    }

    #[test]
    fn bench_secret_version_creation() {
        let start = Instant::now();
        for i in 0..10000 {
            let _version = SecretVersion::new(SecretType::JwtSecret, format!("secret_key_{}", i));
        }
        let elapsed = start.elapsed();
        println!("Secret version creation (10000 iterations): {:?}", elapsed);
        assert!(elapsed < Duration::from_secs(2));
    }

    #[test]
    fn bench_secret_version_store_operations() {
        let store = SecretVersionStore::new(SecretType::JwtSecret, 10);
        let start = Instant::now();
        for i in 0..1000 {
            let version = SecretVersion::new(SecretType::JwtSecret, format!("secret_key_{}", i));
            store.add_version(version);
        }
        let elapsed = start.elapsed();
        println!("Version store add (1000 iterations): {:?}", elapsed);
        assert!(elapsed < Duration::from_secs(3));
    }

    #[test]
    fn bench_concurrent_rotation_checks() {
        use std::thread;

        let manager = Arc::new(create_test_secret_manager());
        let mut handles = vec![];

        for i in 0..5 {
            let manager_clone = Arc::clone(&manager);
            let handle = thread::spawn(move || {
                for j in 0..20 {
                    let secret_type = match (i + j) % 3 {
                        0 => SecretType::JwtSecret,
                        1 => SecretType::ApiKey,
                        _ => SecretType::EncryptionKey,
                    };
                    let _ = manager_clone.needs_rotation(secret_type);
                }
            });
            handles.push(handle);
        }

        let start = Instant::now();
        for handle in handles {
            handle.join().unwrap();
        }
        let elapsed = start.elapsed();
        println!(
            "Concurrent rotation checks (5 threads x 20 iterations): {:?}",
            elapsed
        );
        assert!(elapsed < Duration::from_secs(10));
    }
}
