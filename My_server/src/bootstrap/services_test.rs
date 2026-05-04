//! / Bootstrap模块服务管理测试

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{timeout, Duration};

    #[tokio::test]
    async fn test_database_pool_creation() {
        let config = Config::default();
        let pool = create_database_pool(&config.database).await;
        assert!(pool.is_ok());
    }

    #[tokio::test]
    async fn test_redis_pool_creation() {
        let config = Config::default();
        let pool = create_redis_pool(&config.redis).await;
        assert!(pool.is_ok());
    }

    #[tokio::test]
    async fn test_state_initialization() {
        let config = Config::default();
        let state = initialize_state(config).await;
        assert!(state.database_pool.is_some());
        assert!(state.redis_pool.is_some());
    }

    #[tokio::test]
    async fn test_service_startup() {
        let config = Config::default();
        let state = initialize_state(config).await;
        let result = start_services(&state).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_service_graceful_shutdown() {
        let config = Config::default();
        let state = initialize_state(config).await;
        start_services(&state).await.unwrap();
        let result = graceful_shutdown(&state).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_service_timeout() {
        let config = Config::default();
        let state = initialize_state(config).await;
        
        // 测试超时
        let result = timeout(Duration::from_secs(5), start_services(&state)).await;
        assert!(result.is_ok());
    }
}






