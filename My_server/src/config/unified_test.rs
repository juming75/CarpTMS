//! / 统一配置管理测试

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_from_yaml() {
        let yaml = r#"
server:
  host: "0.0.0.0"
  port: 8080
database:
  url: "postgresql://localhost/mydb"
"#;
        let config = UnifiedConfig::from_yaml(yaml).unwrap();
        assert_eq!(config.server.host, "0.0.0.0");
        assert_eq!(config.server.port, 8080);
    }

    #[test]
    fn test_config_from_env_override() {
        std::env::set_var("TMS_SERVER_PORT", "9000");
        let config = UnifiedConfig::load().unwrap();
        assert_eq!(config.server.port, 9000);
        std::env::remove_var("TMS_SERVER_PORT");
    }

    #[test]
    fn test_config_validation() {
        let mut config = UnifiedConfig::default();
        config.server.port = 80; // 系统保留端口
        
        let result = config.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_database_connection_string() {
        let config = UnifiedConfig::default();
        let conn_str = config.database.connection_string();
        assert!(conn_str.contains("postgresql://"));
    }

    #[test]
    fn test_redis_connection_string() {
        let config = UnifiedConfig::default();
        let conn_str = config.redis.connection_string();
        assert!(conn_str.contains("redis://"));
    }

    #[test]
    fn test_config_defaults() {
        let config = UnifiedConfig::default();
        assert_eq!(config.server.host, "127.0.0.1");
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.database.max_connections, 10);
    }
}






