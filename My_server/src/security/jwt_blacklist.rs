//! JWT Token 黑名单模块
//!
//! 用于存储已撤销的 JWT Token，防止 Token 泄露后被滥用
//! 使用 Redis 存储黑名单，设置与 Token 剩余有效期相同的 TTL

use redis::AsyncCommands;

/// JWT 黑名单管理器
#[derive(Clone)]
pub struct JwtBlacklist {
    /// Redis 连接管理器
    redis_url: String,
}

impl JwtBlacklist {
    /// 创建新的 JWT 黑名单管理器
    pub fn new(redis_url: &str) -> Self {
        Self {
            redis_url: redis_url.to_string(),
        }
    }

    /// 从环境变量创建黑名单管理器
    pub fn from_env() -> Self {
        let redis_url =
            std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());
        Self::new(&redis_url)
    }

    /// 将 Token 加入黑名单
    ///
    /// # Arguments
    /// * `token` - JWT Token (jti claim 或完整的 token)
    /// * `ttl` - 过期时间剩余秒数（应与 Token 剩余有效期一致）
    pub async fn add_to_blacklist(&self, token: &str, ttl_seconds: u64) -> Result<(), String> {
        let client = redis::Client::open(self.redis_url.as_str())
            .map_err(|e| format!("Redis 连接失败: {}", e))?;

        let mut conn = client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| format!("获取 Redis 连接失败: {}", e))?;

        // 使用 "jwt_blacklist:" 前缀，值为 "1"，TTL 设置为 Token 剩余有效期
        let key = format!("jwt_blacklist:{}", token);
        let _: () = conn
            .set_ex(&key, "1", ttl_seconds)
            .await
            .map_err(|e| format!("写入黑名单失败: {}", e))?;

        log::info!("Token 已加入黑名单: {}", &token[..token.len().min(20)]);
        Ok(())
    }

    /// 检查 Token 是否在黑名单中
    pub async fn is_blacklisted(&self, token: &str) -> Result<bool, String> {
        let client = redis::Client::open(self.redis_url.as_str())
            .map_err(|e| format!("Redis 连接失败: {}", e))?;

        let mut conn = client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| format!("获取 Redis 连接失败: {}", e))?;

        let key = format!("jwt_blacklist:{}", token);
        let exists: bool = conn
            .exists(&key)
            .await
            .map_err(|e| format!("检查黑名单失败: {}", e))?;

        Ok(exists)
    }

    /// 从黑名单移除 Token（通常不需要，TTL 会自动清理）
    #[allow(dead_code)]
    pub async fn remove_from_blacklist(&self, token: &str) -> Result<(), String> {
        let client = redis::Client::open(self.redis_url.as_str())
            .map_err(|e| format!("Redis 连接失败: {}", e))?;

        let mut conn = client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| format!("获取 Redis 连接失败: {}", e))?;

        let key = format!("jwt_blacklist:{}", token);
        let _: () = conn
            .del(&key)
            .await
            .map_err(|e| format!("从黑名单移除失败: {}", e))?;

        log::info!("Token 已从黑名单移除: {}", &token[..token.len().min(20)]);
        Ok(())
    }

    /// 获取黑名单中的 Token 数量（用于监控）
    #[allow(dead_code)]
    pub async fn count_blacklisted(&self) -> Result<u64, String> {
        let client = redis::Client::open(self.redis_url.as_str())
            .map_err(|e| format!("Redis 连接失败: {}", e))?;

        let mut conn = client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| format!("获取 Redis 连接失败: {}", e))?;

        let keys: Vec<String> = redis::cmd("KEYS")
            .arg("jwt_blacklist:*")
            .query_async(&mut conn)
            .await
            .map_err(|e| format!("查询黑名单失败: {}", e))?;

        Ok(keys.len() as u64)
    }
}

/// 计算 Token 的剩余有效期秒数
///
/// # Arguments
/// * `exp` - Token 的 exp claim (Unix 时间戳)
///
/// # Returns
/// * 剩余秒数，如果已过期则返回 0
pub fn calculate_ttl_seconds(exp: usize) -> u64 {
    let now = chrono::Utc::now().timestamp() as usize;
    if exp > now {
        (exp - now) as u64
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_ttl() {
        let now = chrono::Utc::now().timestamp() as usize;

        // 未来过期
        assert!(calculate_ttl_seconds(now + 3600) > 3500);

        // 已过期
        assert_eq!(calculate_ttl_seconds(now - 100), 0);
    }
}
