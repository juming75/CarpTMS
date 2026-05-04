//! 统一配置管理单元测试

#[cfg(test)]
mod tests {
    use crate::config::unified::{
        CircuitBreakerConfig, DatabaseConfig, GatewayConfig, LoggingConfig, MonitoringConfig,
        RedisConfig, SecurityConfig, ServerConfig, SyncConfig, UnifiedConfig, VideoConfig,
    };

    // ServerConfig 测试
    mod server_config_tests {
        use super::*;

        #[test]
        fn test_server_config_default() {
            let config = ServerConfig::default();
            assert_eq!(config.host, "0.0.0.0");
            assert_eq!(config.port, 8082);
            assert!(!config.enable_tls);
            assert!(config.tls_cert_path.is_none());
            assert!(config.tls_key_path.is_none());
        }

        #[test]
        fn test_server_config_with_tls() {
            let config = ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 8443,
                enable_tls: true,
                tls_cert_path: Some("/path/to/cert.pem".to_string()),
                tls_key_path: Some("/path/to/key.pem".to_string()),
            };
            assert!(config.enable_tls);
            assert_eq!(config.port, 8443);
        }
    }

    // DatabaseConfig 测试
    mod database_config_tests {
        use super::*;

        #[test]
        fn test_database_config_default() {
            let config = DatabaseConfig::default();
            assert!(config.url.contains("postgresql://"));
            assert_eq!(config.max_connections, 20);
            assert_eq!(config.min_connections, 5);
            assert_eq!(config.connect_timeout, 30);
        }

        #[test]
        fn test_database_config_custom() {
            let config = DatabaseConfig {
                url: "postgresql://user:pass@localhost:5432/testdb".to_string(),
                max_connections: 50,
                min_connections: 10,
                connect_timeout: 60,
            };
            assert_eq!(config.max_connections, 50);
            assert_eq!(config.connect_timeout, 60);
        }
    }

    // RedisConfig 测试
    mod redis_config_tests {
        use super::*;

        #[test]
        fn test_redis_config_default() {
            let config = RedisConfig::default();
            assert!(config.url.contains("redis://"));
            assert_eq!(config.max_connections, 10);
            assert_eq!(config.default_ttl, 300);
        }

        #[test]
        fn test_redis_config_custom_url() {
            let config = RedisConfig {
                url: "redis://custom-host:6380".to_string(),
                max_connections: 20,
                default_ttl: 600,
            };
            assert_eq!(config.url, "redis://custom-host:6380");
        }
    }

    // SecurityConfig 测试
    mod security_config_tests {
        use super::*;

        #[test]
        fn test_security_config_default() {
            let config = SecurityConfig::default();
            assert!(config.jwt_secret.len() >= 32);
            assert_eq!(config.jwt_expiration, 86400);
            assert_eq!(config.argon2_memory, 65536);
            assert_eq!(config.argon2_time, 3);
            assert_eq!(config.argon2_parallelism, 4);
            assert!(!config.enable_https);
            assert!(!config.allowed_origins.is_empty());
        }

        #[test]
        fn test_security_config_allowed_origins() {
            let config = SecurityConfig {
                allowed_origins: vec![
                    "http://localhost:5173".to_string(),
                    "http://127.0.0.1:5173".to_string(),
                ],
                ..Default::default()
            };
            assert_eq!(config.allowed_origins.len(), 2);
        }
    }

    // MonitoringConfig 测试
    mod monitoring_config_tests {
        use super::*;

        #[test]
        fn test_monitoring_config_default() {
            let config = MonitoringConfig::default();
            assert!(config.enable_prometheus);
            assert!(config.enable_tracing);
            assert!(config.tracing_service_url.is_none());
            assert_eq!(config.slow_query_threshold, 100);
        }

        #[test]
        fn test_monitoring_config_with_tracing_url() {
            let config = MonitoringConfig {
                tracing_service_url: Some("http://jaeger:14268/api/traces".to_string()),
                ..Default::default()
            };
            assert!(config.tracing_service_url.is_some());
        }
    }

    // LoggingConfig 测试
    mod logging_config_tests {
        use super::*;

        #[test]
        fn test_logging_config_default() {
            let config = LoggingConfig::default();
            assert_eq!(config.level, "info");
            assert_eq!(config.format, "json");
            assert!(config.file_path.is_none());
        }

        #[test]
        fn test_logging_config_with_file() {
            let config = LoggingConfig {
                file_path: Some("/var/log/carptms.log".to_string()),
                level: "debug".to_string(),
                ..Default::default()
            };
            assert!(config.file_path.is_some());
            assert_eq!(config.level, "debug");
        }
    }

    // GatewayConfig 测试
    mod gateway_config_tests {
        use super::*;

        #[test]
        fn test_gateway_config_default() {
            let config = GatewayConfig::default();
            assert_eq!(config.jt808_address, "0.0.0.0:8988");
            assert_eq!(config.websocket_address, "0.0.0.0:8089");
            assert_eq!(config.max_connections, 10000);
        }
    }

    // VideoConfig 测试
    mod video_config_tests {
        use super::*;

        #[test]
        fn test_video_config_default() {
            let config = VideoConfig::default();
            assert!(!config.enabled);
            assert_eq!(config.jt1078_max_connections, 1000);
            assert_eq!(config.gb28181_sip_port, 5060);
            assert_eq!(config.storage_path, "./data/videos");
        }

        #[test]
        fn test_video_config_enabled() {
            let config = VideoConfig {
                enabled: true,
                jt1078_max_connections: 500,
                ..Default::default()
            };
            assert!(config.enabled);
            assert_eq!(config.jt1078_max_connections, 500);
        }
    }

    // SyncConfig 测试
    mod sync_config_tests {
        use super::*;

        #[test]
        fn test_sync_config_default() {
            let config = SyncConfig::default();
            assert!(config.enabled);
            assert_eq!(config.interval_seconds, 300);
            assert_eq!(config.legacy_host, "127.0.0.1");
            assert_eq!(config.legacy_port, 9808);
        }
    }

    // CircuitBreakerConfig 测试
    mod circuit_breaker_config_tests {
        use super::*;

        #[test]
        fn test_circuit_breaker_config_default() {
            let config = CircuitBreakerConfig::default();
            assert_eq!(config.failure_threshold, 0.5);
            assert_eq!(config.request_threshold, 20);
            assert_eq!(config.open_timeout_seconds, 30);
            assert_eq!(config.half_open_requests, 5);
            assert_eq!(config.half_open_success_threshold, 3);
            assert!(!config.exempt_paths.is_empty());
        }

        #[test]
        fn test_circuit_breaker_exempt_paths() {
            let config = CircuitBreakerConfig {
                exempt_paths: vec![
                    "/api/health".to_string(),
                    "/api/metrics".to_string(),
                ],
                ..Default::default()
            };
            assert_eq!(config.exempt_paths.len(), 2);
            assert!(config.exempt_paths.contains(&"/api/health".to_string()));
        }
    }

    // UnifiedConfig 综合测试
    mod unified_config_tests {
        use super::*;

        #[test]
        fn test_unified_config_default() {
            let config = UnifiedConfig::default();
            assert_eq!(config.server.port, 8082);
            assert!(config.database.url.contains("postgresql://"));
            assert!(config.redis.url.contains("redis://"));
            assert!(config.security.jwt_secret.len() >= 32);
        }

        #[test]
        fn test_unified_config_full() {
            let config = UnifiedConfig::default();
            // 验证所有子配置都存在
            assert_eq!(config.architecture.name, "CarpTMS");
            assert_eq!(config.server.port, 8082);
            assert!(!config.database.url.is_empty());
            assert!(!config.redis.url.is_empty());
            assert!(config.security.jwt_expiration > 0);
            assert!(config.monitoring.slow_query_threshold > 0);
            assert!(!config.logging.level.is_empty());
        }

        #[test]
        fn test_unified_config_validate_valid() {
            let config = UnifiedConfig::default();
            assert!(config.validate().is_ok());
        }

        #[test]
        fn test_unified_config_validate_invalid_port() {
            let mut config = UnifiedConfig::default();
            config.server.port = 0;
            assert!(config.validate().is_err());
        }

        #[test]
        fn test_unified_config_validate_empty_db_url() {
            let mut config = UnifiedConfig::default();
            config.database.url = "".to_string();
            assert!(config.validate().is_err());
        }

        #[test]
        fn test_unified_config_validate_invalid_slow_query() {
            let mut config = UnifiedConfig::default();
            config.monitoring.slow_query_threshold = 0;
            assert!(config.validate().is_err());
        }
    }

    // 工厂函数测试
    mod factory_tests {
        use super::*;

        #[test]
        fn test_default_server_host() {
            let host = super::super::default_host();
            assert_eq!(host, "0.0.0.0");
        }

        #[test]
        fn test_default_server_port() {
            let port = super::super::default_port();
            assert_eq!(port, 8082);
        }

        #[test]
        fn test_default_db_max_connections() {
            let max = super::super::default_db_max_connections();
            assert_eq!(max, 20);
        }

        #[test]
        fn test_default_cors_origins() {
            let origins = super::super::default_cors_origins();
            assert!(!origins.is_empty());
            assert!(origins.contains(&"http://localhost:5173".to_string()));
        }

        #[test]
        fn test_default_jwt_expiration() {
            let exp = super::super::default_jwt_expiration();
            assert_eq!(exp, 86400); // 24 hours
        }

        #[test]
        fn test_default_argon2_params() {
            assert_eq!(super::super::default_argon2_memory(), 65536);
            assert_eq!(super::super::default_argon2_time(), 3);
            assert_eq!(super::super::default_argon2_parallelism(), 4);
        }
    }
}
