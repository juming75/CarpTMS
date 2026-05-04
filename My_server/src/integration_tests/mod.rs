//!
//! 集成测试配置
//! 包含数据库、Redis、HTTP客户端等测试资源

#[cfg(test)]
pub mod test_config {
    use std::env;
    
    /// 获取测试数据库URL
    pub fn get_test_database_url() -> String {
        env::var("TEST_DATABASE_URL")
            .unwrap_or_else(|_| "postgres://carptms_test:test123@localhost:5432/carptms_test_db".to_string())
    }
    
    /// 获取测试Redis URL
    pub fn get_test_redis_url() -> String {
        env::var("TEST_REDIS_URL")
            .unwrap_or_else(|_| "redis://:test123@localhost:6379".to_string())
    }
    
    /// 获取测试服务器端口
    pub fn get_test_server_port() -> u16 {
        env::var("TEST_SERVER_PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(0) // 0 表示随机端口
    }
    
    /// 测试超时配置
    pub const TEST_TIMEOUT_SECS: u64 = 30;
    pub const DB_SETUP_TIMEOUT_SECS: u64 = 60;
}

/// 测试数据库池
#[cfg(test)]
pub mod test_db {
    use sqlx::postgres::{PgPool, PgPoolOptions};
    use std::time::Duration;
    use super::test_config::{get_test_database_url, TEST_TIMEOUT_SECS};
    
    /// 创建测试数据库连接池
    pub async fn create_test_pool() -> Result<PgPool, sqlx::Error> {
        let database_url = get_test_database_url();
        
        PgPoolOptions::new()
            .max_connections(5)
            .acquire_timeout(Duration::from_secs(TEST_TIMEOUT_SECS))
            .idle_timeout(Duration::from_secs(60))
            .connect(&database_url)
            .await
    }
    
    /// 运行数据库迁移
    pub async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::Error> {
        // 执行迁移脚本
        sqlx::migrate!("./migrations").run(pool).await?;
        Ok(())
    }
    
    /// 清理测试数据
    pub async fn cleanup_test_data(pool: &PgPool) -> Result<(), sqlx::Error> {
        // 清理所有测试数据
        sqlx::query("TRUNCATE TABLE users, vehicles, drivers, devices RESTART IDENTITY CASCADE")
            .execute(pool)
            .await?;
        Ok(())
    }
}

/// 测试Redis客户端
#[cfg(test)]
pub mod test_redis {
    use redis::Client;
    use std::time::Duration;
    use super::test_config::{get_test_redis_url, TEST_TIMEOUT_SECS};
    
    /// 创建测试Redis客户端
    pub fn create_test_client() -> Result<Client, redis::RedisError> {
        let redis_url = get_test_redis_url();
        Client::open(redis_url)
    }
    
    /// 清理测试Redis数据
    pub async fn cleanup_test_data(client: &Client) -> Result<(), redis::RedisError> {
        let mut conn = client.get_multiplexed_async_connection().await?;
        redis::cmd("FLUSHDB").query_async::<()>(&mut conn).await?;
        Ok(())
    }
}

/// HTTP测试客户端
#[cfg(test)]
pub mod test_http {
    use reqwest::Client;
    use std::time::Duration;
    
    /// 创建HTTP测试客户端
    pub fn create_test_client() -> Client {
        Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("创建测试HTTP客户端失败")
    }
    
    /// 创建认证测试客户端
    pub fn create_authenticated_client(token: &str) -> Client {
        Client::builder()
            .timeout(Duration::from_secs(30))
            .default_headers({
                let mut headers = reqwest::header::HeaderMap::new();
                headers.insert(
                    reqwest::header::AUTHORIZATION,
                    format!("Bearer {}", token).parse().unwrap(),
                );
                headers
            })
            .build()
            .expect("创建认证HTTP客户端失败")
    }
}

/// 压力测试配置
#[cfg(test)]
pub mod load_test_config {
    /// 并发连接数
    pub const CONCURRENT_CONNECTIONS: usize = 100;
    
    /// 每个连接的请求数
    pub const REQUESTS_PER_CONNECTION: usize = 100;
    
    /// 请求超时(ms)
    pub const REQUEST_TIMEOUT_MS: u64 = 5000;
    
    /// 预热请求数
    pub const WARMUP_REQUESTS: usize = 10;
}

/// 测试辅助宏
#[cfg(test)]
#[allow(unused)]
mod macros {
    /// 创建测试用户
    #[macro_export]
    macro_rules! create_test_user {
        ($pool:expr, $username:expr) => {{
            use crate::domain::entities::User;
            use crate::domain::repositories::UserRepository;
            
            let user = User {
                id: None,
                username: $username.to_string(),
                email: format!("{}@test.com", $username),
                password_hash: "hashed_password".to_string(),
                role: crate::domain::entities::UserRole::User,
                created_at: None,
                updated_at: None,
            };
            
            let repo = crate::domain::repositories::SqlxUserRepository::new($pool);
            repo.create(&user).await.unwrap()
        }};
    }
    
    /// 创建测试车辆
    #[macro_export]
    macro_rules! create_test_vehicle {
        ($pool:expr, $plate:expr) => {{
            use crate::domain::entities::Vehicle;
            
            let vehicle = Vehicle {
                id: None,
                plate_number: $plate.to_string(),
                vehicle_type: crate::domain::entities::VehicleType::Heavy,
                status: crate::domain::entities::VehicleStatus::Active,
                created_at: None,
                updated_at: None,
            };
            
            let repo = crate::domain::repositories::SqlxVehicleRepository::new($pool);
            repo.create(&vehicle).await.unwrap()
        }};
    }
}
