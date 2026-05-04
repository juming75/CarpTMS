//! /! Health Check Benchmark Tests
//!
//! Provides benchmark tests for health check endpoints and components

#[cfg(test)]
mod benchmarks {
    use super::super::health::enhanced::{
        DependencyStatus, DynamicHealthConfig, EnhancedHealthChecker, HealthStatus,
        NotificationConfig, SystemMetrics, ThresholdConfig,
    };
    use std::collections::HashMap;
    use std::time::{Duration, Instant};

    fn create_test_health_checker() -> EnhancedHealthChecker {
        EnhancedHealthChecker::new()
    }

    fn create_sample_system_metrics() -> SystemMetrics {
        SystemMetrics {
            cpu_usage: 45.5,
            memory_usage: 62.3,
            disk_usage: 55.0,
            load_average: [1.2, 1.0, 0.8],
            uptime: 86400,
            current_time: chrono::Utc::now().to_rfc3339(),
        }
    }

    fn create_sample_dependencies() -> HashMap<String, DependencyStatus> {
        let mut deps = HashMap::new();
        deps.insert(
            "database".to_string(),
            DependencyStatus {
                status: "ok".to_string(),
                error: None,
                response_time_ms: Some(5),
                last_checked: chrono::Utc::now().to_rfc3339(),
            },
        );
        deps.insert(
            "redis".to_string(),
            DependencyStatus {
                status: "ok".to_string(),
                error: None,
                response_time_ms: Some(2),
                last_checked: chrono::Utc::now().to_rfc3339(),
            },
        );
        deps
    }

    #[test]
    fn bench_health_checker_creation() {
        let start = Instant::now();
        for _ in 0..1000 {
            let _checker = EnhancedHealthChecker::new();
        }
        let elapsed = start.elapsed();
        println!("Health checker creation (1000 iterations): {:?}", elapsed);
        assert!(elapsed < Duration::from_secs(5));
    }

    #[test]
    fn bench_dynamic_config_default() {
        let start = Instant::now();
        for _ in 0..10000 {
            let _config = DynamicHealthConfig::default();
        }
        let elapsed = start.elapsed();
        println!("Dynamic config default (10000 iterations): {:?}", elapsed);
        assert!(elapsed < Duration::from_secs(2));
    }

    #[test]
    fn bench_threshold_config_serialization() {
        let config = DynamicHealthConfig {
            enable_all_checks: true,
            enabled_checks: vec!["cpu".to_string(), "memory".to_string(), "disk".to_string()],
            check_interval_seconds: 30,
            custom_thresholds: HashMap::new(),
            notification_config: NotificationConfig {
                enabled: true,
                channels: vec!["log".to_string()],
                min_severity: "warning".to_string(),
            },
        };

        let start = Instant::now();
        for _ in 0..1000 {
            let json = serde_json::to_string(&config).unwrap();
            let _parsed: DynamicHealthConfig = serde_json::from_str(&json).unwrap();
        }
        let elapsed = start.elapsed();
        println!(
            "Threshold config serialization (1000 iterations): {:?}",
            elapsed
        );
        assert!(elapsed < Duration::from_secs(3));
    }

    #[test]
    fn bench_get_threshold_cpu() {
        let checker = create_test_health_checker();
        let start = Instant::now();
        for _ in 0..10000 {
            let _threshold = checker.get_threshold("cpu");
        }
        let elapsed = start.elapsed();
        println!("Get CPU threshold (10000 iterations): {:?}", elapsed);
        assert!(elapsed < Duration::from_secs(1));
    }

    #[test]
    fn bench_get_threshold_custom() {
        let checker = create_test_health_checker();
        checker.update_threshold("custom_check", 30.0, 60.0);

        let start = Instant::now();
        for _ in 0..10000 {
            let _threshold = checker.get_threshold("custom_check");
        }
        let elapsed = start.elapsed();
        println!("Get custom threshold (10000 iterations): {:?}", elapsed);
        assert!(elapsed < Duration::from_secs(1));
    }

    #[test]
    fn bench_update_threshold() {
        let checker = create_test_health_checker();
        let start = Instant::now();
        for i in 0..1000 {
            checker.update_threshold("dynamic_check", 10.0 + i as f64, 20.0 + i as f64);
        }
        let elapsed = start.elapsed();
        println!("Update threshold (1000 iterations): {:?}", elapsed);
        assert!(elapsed < Duration::from_secs(2));
    }

    #[test]
    fn bench_set_and_get_config() {
        let checker = create_test_health_checker();
        let new_config = DynamicHealthConfig {
            enable_all_checks: false,
            enabled_checks: vec!["cpu".to_string()],
            check_interval_seconds: 60,
            custom_thresholds: HashMap::new(),
            notification_config: NotificationConfig::default(),
        };

        let start = Instant::now();
        for _ in 0..1000 {
            checker.set_config(new_config.clone());
            let _retrieved = checker.get_config();
        }
        let elapsed = start.elapsed();
        println!("Set and get config (1000 iterations): {:?}", elapsed);
        assert!(elapsed < Duration::from_secs(3));
    }

    #[test]
    fn bench_check_history_limit() {
        let checker = create_test_health_checker();
        let config = DynamicHealthConfig::default();
        checker.set_config(config);

        let start = Instant::now();
        for i in 0..150 {
            let history = checker.get_check_history(i);
            assert!(history.len() <= i);
        }
        let elapsed = start.elapsed();
        println!("Check history with limits (150 iterations): {:?}", elapsed);
        assert!(elapsed < Duration::from_secs(5));
    }

    #[test]
    fn bench_system_metrics_structure() {
        let start = Instant::now();
        for _ in 0..10000 {
            let _metrics = create_sample_system_metrics();
        }
        let elapsed = start.elapsed();
        println!("Create system metrics (10000 iterations): {:?}", elapsed);
        assert!(elapsed < Duration::from_secs(2));
    }

    #[test]
    fn bench_dependency_status_serialization() {
        let deps = create_sample_dependencies();
        let start = Instant::now();
        for _ in 0..5000 {
            let json = serde_json::to_string(&deps).unwrap();
            let _parsed: HashMap<String, DependencyStatus> = serde_json::from_str(&json).unwrap();
        }
        let elapsed = start.elapsed();
        println!(
            "Dependency status serialization (5000 iterations): {:?}",
            elapsed
        );
        assert!(elapsed < Duration::from_secs(5));
    }

    #[test]
    fn bench_health_status_with_all_fields() {
        let status = HealthStatus {
            status: "ok".to_string(),
            service: "tms_server".to_string(),
            version: "1.1.0".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            hostname: "test-host".to_string(),
            system_metrics: create_sample_system_metrics(),
            dependencies: create_sample_dependencies(),
            alerts: vec![],
            checks: HashMap::new(),
        };

        let start = Instant::now();
        for _ in 0..1000 {
            let json = serde_json::to_string(&status).unwrap();
            let _parsed: HealthStatus = serde_json::from_str(&json).unwrap();
        }
        let elapsed = start.elapsed();
        println!(
            "Full health status serialization (1000 iterations): {:?}",
            elapsed
        );
        assert!(elapsed < Duration::from_secs(3));
    }

    #[test]
    fn bench_concurrent_config_updates() {
        use std::sync::{Arc, RwLock};
        use std::thread;

        let checker = Arc::new(create_test_health_checker());
        let mut handles = vec![];

        for i in 0..10 {
            let checker_clone = Arc::clone(&checker);
            let handle = thread::spawn(move || {
                for j in 0..100 {
                    let config = DynamicHealthConfig {
                        check_interval_seconds: (i * 100 + j) as u64,
                        ..DynamicHealthConfig::default()
                    };
                    checker_clone.set_config(config);
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
            "Concurrent config updates (10 threads x 100 updates): {:?}",
            elapsed
        );
        assert!(elapsed < Duration::from_secs(10));
    }

    #[test]
    fn bench_global_health_checker_singleton() {
        use super::super::health::enhanced::get_enhanced_health_checker;

        let start = Instant::now();
        for _ in 0..100000 {
            let _checker = get_enhanced_health_checker();
        }
        let elapsed = start.elapsed();
        println!(
            "Global health checker access (100000 iterations): {:?}",
            elapsed
        );
        assert!(elapsed < Duration::from_secs(5));
    }

    #[test]
    fn bench_notification_config_serialization() {
        let config = NotificationConfig {
            enabled: true,
            channels: vec![
                "log".to_string(),
                "webhook".to_string(),
                "email".to_string(),
            ],
            min_severity: "warning".to_string(),
        };

        let start = Instant::now();
        for _ in 0..10000 {
            let json = serde_json::to_string(&config).unwrap();
            let _parsed: NotificationConfig = serde_json::from_str(&json).unwrap();
        }
        let elapsed = start.elapsed();
        println!(
            "Notification config serialization (10000 iterations): {:?}",
            elapsed
        );
        assert!(elapsed < Duration::from_secs(3));
    }
}
