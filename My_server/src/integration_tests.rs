//! CarpTMS 集成测试模块
//!
//! 包含核心业务逻辑的集成测试

#[cfg(test)]
mod tests {
    use crate::domain::entities::vehicle::{Vehicle, VehicleCreate, VehicleQuery};
    use crate::domain::entities::device::{Device, DeviceQuery, DeviceCreate, DeviceUpdate};
    use crate::domain::entities::alarm::{Alarm, AlarmQuery};
    use crate::domain::entities::order::{Order, OrderCreate};
    use chrono::NaiveDateTime;

    // ============= Health Check Integration Tests =============

    #[test]
    fn test_health_check_integration_with_dependencies() {
        use crate::health::enhanced::{
            EnhancedHealthChecker, DynamicHealthConfig, ThresholdConfig, NotificationConfig,
        };

        let checker = EnhancedHealthChecker::new();

        let mut config = DynamicHealthConfig::default();
        config.check_interval_seconds = 60;
        config.custom_thresholds.insert(
            "cpu".to_string(),
            ThresholdConfig { warning: 70.0, critical: 85.0 },
        );
        config.notification_config = NotificationConfig {
            enabled: true,
            channels: vec!["log".to_string()],
            min_severity: "warning".to_string(),
        };
        checker.set_config(config);

        let retrieved_config = checker.get_config();
        assert_eq!(retrieved_config.check_interval_seconds, 60);
        assert!(retrieved_config.notification_config.enabled);
    }

    #[test]
    fn test_health_check_threshold_override() {
        use crate::health::enhanced::EnhancedHealthChecker;

        let checker = EnhancedHealthChecker::new();
        checker.update_threshold("cpu", 50.0, 70.0);
        let threshold = checker.get_threshold("cpu");
        assert_eq!(threshold.warning, 50.0);
        assert_eq!(threshold.critical, 70.0);

        let memory_threshold = checker.get_threshold("memory");
        assert_eq!(memory_threshold.warning, 80.0);
    }

    #[test]
    fn test_health_check_history_tracking() {
        use crate::health::enhanced::EnhancedHealthChecker;

        let checker = EnhancedHealthChecker::new();
        let history = checker.get_check_history(10);
        assert!(history.is_empty() || history.len() <= 10);
    }

    #[test]
    fn test_dynamic_health_config_serialization() {
        use crate::health::enhanced::{DynamicHealthConfig, ThresholdConfig};
        use std::collections::HashMap;

        let mut config = DynamicHealthConfig::default();
        config.custom_thresholds.insert(
            "cpu".to_string(),
            ThresholdConfig { warning: 75.0, critical: 90.0 },
        );

        let json = serde_json::to_string(&config).unwrap();
        let parsed: DynamicHealthConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.check_interval_seconds, config.check_interval_seconds);
        assert_eq!(
            parsed.custom_thresholds.get("cpu").unwrap().warning,
            75.0
        );
    }

    // ============= Config Center Integration Tests =============

    #[test]
    fn test_config_center_storage_integration() {
        use crate::config_center::storage::{MemoryConfigStorage, StorageConfig, create_storage};
        use crate::config_center::models::ConfigKey;

        let config = StorageConfig::memory();
        let storage = create_storage(&config).unwrap();

        let entry = crate::config_center::models::ConfigEntry {
            key: ConfigKey {
                namespace: "test".to_string(),
                key: "test_key".to_string(),
            },
            value: "test_value".to_string(),
            version: 1,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        storage.store(&entry).unwrap();
        let retrieved = storage.get(&entry.key).unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().value, "test_value");
    }

    #[test]
    fn test_config_center_composite_storage() {
        use crate::config_center::storage::{StorageConfig, StorageType, create_storage};

        let config = StorageConfig {
            storage_type: StorageType::Composite,
            config: serde_json::json!({
                "local": StorageConfig::memory(),
            }),
        };

        let storage = create_storage(&config).unwrap();
        assert!(storage.list(&"test".to_string()).is_ok());
    }

    #[test]
    fn test_config_center_file_storage_factory() {
        use crate::config_center::storage::{StorageConfig, StorageType};

        let config = StorageConfig {
            storage_type: StorageType::File,
            config: serde_json::json!({
                "path": "/tmp/config",
            }),
        };

        let result = create_storage(&config);
        assert!(result.is_ok());
    }

    // ============= Secret Manager Integration Tests =============

    #[test]
    fn test_secret_manager_key_rotation_integration() {
        use crate::config::secret_manager::{SecretManager, SecretType};

        let manager = SecretManager::new().expect("Failed to create secret manager");

        manager.store_secret(SecretType::JwtSecret, "initial_secret".to_string())
            .expect("Failed to store secret");

        let new_version = manager.rotate_secret(SecretType::JwtSecret)
            .expect("Failed to rotate secret");
        assert!(new_version > 0);

        let latest = manager.get_secret(SecretType::JwtSecret)
            .expect("Failed to get secret");
        assert!(!latest.is_empty());
    }

    #[test]
    fn test_secret_manager_version_tracking() {
        use crate::config::secret_manager::{SecretManager, SecretType};

        let manager = SecretManager::new().expect("Failed to create secret manager");

        for i in 0..3 {
            manager.store_secret(
                SecretType::ApiKey,
                format!("api_key_v{}", i),
            ).expect("Failed to store");
        }

        let valid_secrets = manager.get_all_valid_secrets(SecretType::ApiKey)
            .expect("Failed to get valid secrets");
        assert!(!valid_secrets.is_empty());
    }

    #[test]
    fn test_secret_manager_rotation_policy() {
        use crate::config::secret_manager::{SecretManager, SecretType, KeyRotationConfig};
        use std::time::Duration;

        let manager = SecretManager::new().expect("Failed to create secret manager");

        let mut config = KeyRotationConfig::default();
        config.auto_rotate = true;
        config.rotation_period = Duration::from_secs(86400);
        manager.set_rotation_config(SecretType::DatabasePassword, config);

        let needs_rotation = manager.needs_rotation(SecretType::DatabasePassword)
            .expect("Failed to check rotation");
        assert!(!needs_rotation);
    }

    // ============= Service Discovery Integration Tests =============

    #[test]
    fn test_service_discovery_integration() {
        use crate::service_discovery::{ServiceRegistry, ServiceInstance, HealthStatus};

        let registry = ServiceRegistry::new();

        let instance = ServiceInstance {
            id: "instance-1".to_string(),
            name: "test-service".to_string(),
            address: "127.0.0.1".to_string(),
            port: 8080,
            health_status: HealthStatus::Healthy,
            metadata: Default::default(),
        };

        registry.register(instance.clone()).expect("Failed to register");
        let instances = registry.get_service("test-service")
            .expect("Failed to get service");
        assert_eq!(instances.len(), 1);
        registry.deregister("instance-1").expect("Failed to deregister");
        let instances = registry.get_service("test-service").unwrap_or_default();
        assert_eq!(instances.len(), 0);
    }

    // ============= Circuit Breaker Integration Tests =============

    #[test]
    fn test_circuit_breaker_integration() {
        use crate::errors::circuit_breaker::{CircuitBreaker, CircuitState};

        let mut breaker = CircuitBreaker::new(3, 1000);
        assert_eq!(breaker.state(), CircuitState::Closed);

        breaker.record_failure();
        breaker.record_failure();
        assert_eq!(breaker.state(), CircuitState::Closed);

        breaker.record_failure();
        assert_eq!(breaker.state(), CircuitState::Open);
    }

    // ============= Retry Policy Integration Tests =============

    #[test]
    fn test_retry_policy_integration() {
        use crate::errors::retry::{RetryPolicy, RetryResult};

        let policy = RetryPolicy::new(3, Duration::from_millis(10));

        let mut attempts = 0;
        let result = policy.execute(|| {
            attempts += 1;
            if attempts < 3 {
                Err("temporary failure".to_string())
            } else {
                Ok("success".to_string())
            }
        });

        assert_eq!(result.unwrap(), "success");
        assert_eq!(attempts, 3);
    }

    // ============= Feature Flag Integration Tests =============

    #[test]
    fn test_feature_flag_integration() {
        use crate::feature_flags::{FeatureFlag, FeatureFlags};

        let mut flags = FeatureFlags::new();

        flags.set_flag(
            "new_feature",
            FeatureFlag {
                enabled: true,
                rollout_percentage: 50,
                description: Some("New feature flag".to_string()),
            },
        ).unwrap();

        let flag = flags.get_flag("new_feature").unwrap();
        assert!(flag.enabled);
        assert_eq!(flag.rollout_percentage, 50);
    }

    // ============= Vehicle Entity Tests =============

    #[test]
    fn test_vehicle_query_default() {
        let query = VehicleQuery::default();
        assert_eq!(query.page, Some(1));
        assert_eq!(query.page_size, Some(20));
        assert_eq!(query.status, None);
    }

    #[test]
    fn test_vehicle_query_with_params() {
        let query = VehicleQuery {
            page: Some(5),
            page_size: Some(50),
            status: Some(1),
            license_plate: Some("京A".to_string()),
            group_id: Some(1),
            vehicle_type: Some("货车".to_string()),
        };

        assert_eq!(query.page, Some(5));
        assert_eq!(query.page_size, Some(50));
        assert_eq!(query.status, Some(1));
    }

    #[test]
    fn test_vehicle_create_validation() {
        let create = VehicleCreate {
            vehicle_name: "测试车辆".to_string(),
            license_plate: "京A12345".to_string(),
            vehicle_type: "货车".to_string(),
            vehicle_color: "白色".to_string(),
            vehicle_brand: "东风".to_string(),
            vehicle_model: "EQ1090".to_string(),
            engine_no: "ENG123".to_string(),
            frame_no: "FRA123".to_string(),
            register_date: NaiveDateTime::parse_from_str(
                "2020-01-01 00:00:00",
                "%Y-%m-%d %H:%M:%S",
            ).unwrap(),
            inspection_date: NaiveDateTime::parse_from_str(
                "2024-01-01 00:00:00",
                "%Y-%m-%d %H:%M:%S",
            ).unwrap(),
            insurance_date: NaiveDateTime::parse_from_str(
                "2024-06-01 00:00:00",
                "%Y-%m-%d %H:%M:%S",
            ).unwrap(),
            seating_capacity: 2,
            load_capacity: 5000.0,
            vehicle_length: 6.0,
            vehicle_width: 2.0,
            vehicle_height: 2.5,
            device_id: None,
            terminal_type: None,
            communication_type: None,
            sim_card_no: None,
            install_date: None,
            install_address: None,
            install_technician: None,
            own_no: None,
            own_name: None,
            own_phone: None,
            own_id_card: None,
            own_address: None,
            own_email: None,
            group_id: 1,
            operation_status: 1,
            operation_route: None,
            operation_area: None,
            operation_company: None,
            driver_name: None,
            driver_phone: None,
            driver_license_no: None,
            purchase_price: None,
            annual_fee: None,
            insurance_fee: None,
            remark: None,
            status: 1,
            create_user_id: 1,
        };

        assert!(!create.vehicle_name.is_empty());
        assert!(!create.license_plate.is_empty());
        assert!(create.seating_capacity > 0);
        assert!(create.load_capacity > 0.0);
    }

    // ============= Rate Limiter Tests =============

    #[test]
    fn test_terminal_rate_limit_config() {
        use crate::middleware::terminal_rate_limiter::TerminalRateLimitConfig;

        let config = TerminalRateLimitConfig::default();
        assert_eq!(config.max_requests_per_window, 50);
        assert_eq!(config.max_terminals_per_ip, 27500);
        assert_eq!(config.max_requests_per_second, 10);
        assert_eq!(config.ban_duration_seconds, 300);
    }

    #[test]
    fn test_ip_rate_limit_config() {
        use crate::middleware::rate_limiter::RateLimitConfig;

        let config = RateLimitConfig::default();
        assert!(config.requests_per_minute > 0);
        assert!(config.ban_duration_seconds > 0);
    }

    // ============= Database Pool Tests =============

    #[test]
    fn test_adaptive_pool_config() {
        use crate::database::pool_manager::AdaptivePoolConfig;

        let config = AdaptivePoolConfig::default();
        assert_eq!(config.min_connections, 10);
        assert_eq!(config.max_connections, 300);
        assert!(config.target_utilization >= 0.5);
        assert!(config.target_utilization <= 0.8);
    }

    // ============= Command Tests =============

    #[test]
    fn test_create_order_command_validation() {
        use crate::application::commands::CreateOrderCommand;

        let cmd = CreateOrderCommand {
            order_no: "ORD202401010001".to_string(),
            order_type: "货运".to_string(),
            vehicle_id: 1,
            driver_name: "张三".to_string(),
            driver_phone: "13800138000".to_string(),
            loading_address: "北京市".to_string(),
            unloading_address: "上海市".to_string(),
            loading_time: NaiveDateTime::parse_from_str(
                "2024-01-15 08:00:00",
                "%Y-%m-%d %H:%M:%S",
            ).unwrap(),
            estimated_arrival: NaiveDateTime::parse_from_str(
                "2024-01-15 18:00:00",
                "%Y-%m-%d %H:%M:%S",
            ).unwrap(),
            cargo_type: "电子产品".to_string(),
            cargo_weight: 5000.0,
            cargo_volume: 100.0,
            freight: 5000.0,
            prepaid_expenses: 1000.0,
            remark: None,
            create_user_id: 1,
        };

        assert_eq!(cmd.command_type(), "create_order");
        assert!(cmd.cargo_weight > 0.0);
    }

    // ============= Pagination Tests =============

    #[test]
    fn test_pagination_validation() {
        use crate::application::Pagination;

        let pagination = Pagination::new(-1, 20);
        assert_eq!(pagination.page, 1);

        let pagination = Pagination::new(1, 0);
        assert_eq!(pagination.page_size, 1);

        let pagination = Pagination::new(1, 9999);
        assert_eq!(pagination.page_size, 100);
    }

    // ============= Error Tests =============

    #[test]
    fn test_app_error_creation() {
        use crate::errors::AppError;

        let error = AppError::NotFound("车辆不存在".to_string());

        match error {
            AppError::NotFound(msg) => {
                assert_eq!(msg, "车辆不存在");
            }
            _ => panic!("Expected NotFound error"),
        }
    }

    #[test]
    fn test_app_error_display() {
        use crate::errors::AppError;

        let error = AppError::DatabaseError("连接失败".to_string());
        let display = format!("{}", error);

        assert!(display.contains("DatabaseError"));
    }
}
