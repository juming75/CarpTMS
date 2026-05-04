//! / Bootstrap模块配置管理测试

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_defaults() {
        let config = Config::default();
        assert_eq!(config.server.host, "127.0.0.1");
        assert_eq!(config.server.port, 8080);
    }

    #[test]
    fn test_config_from_env() {
        std::env::set_var("TMS_SERVER_PORT", "9000");
        let config = Config::from_env().unwrap();
        assert_eq!(config.server.port, 9000);
        std::env::remove_var("TMS_SERVER_PORT");
    }

    #[test]
    fn test_database_config() {
        let config = Config::default();
        assert_eq!(config.database.max_connections, 10);
        assert_eq!(config.database.min_connections, 2);
    }

    #[test]
    fn test_redis_config() {
        let config = Config::default();
        assert_eq!(config.redis.host, "127.0.0.1");
        assert_eq!(config.redis.port, 6379);
    }

    #[test]
    fn test_metrics_config() {
        let config = Config::default();
        assert!(config.metrics.enabled);
        assert_eq!(config.metrics.port, 9090);
    }

    #[test]
    fn test_config_validation() {
        let mut config = Config::default();
        config.server.port = 80; // 系统保留端口
        let result = config.validate();
        assert!(result.is_err());
    }
}






