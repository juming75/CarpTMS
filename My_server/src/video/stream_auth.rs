//! 视频流鉴权中间件
//!
//! 实现JT1078视频流的鉴权机制
//! 防止未授权访问视频流

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error,
};
use futures_util::future::{ok, Ready};
use log::{debug, warn};
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 鉴权令牌信息
#[derive(Debug, Clone)]
pub struct StreamToken {
    /// 令牌值
    pub token: String,
    /// SIM卡号
    pub sim_number: String,
    /// 通道号
    pub channel: String,
    /// 过期时间（Unix时间戳）
    pub expires_at: u64,
    /// 创建时间
    pub created_at: u64,
    /// 访问次数
    pub access_count: u32,
    /// 最大访问次数
    pub max_access_count: Option<u32>,
}

impl StreamToken {
    /// 检查令牌是否有效
    pub fn is_valid(&self) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        // 检查过期时间
        if now > self.expires_at {
            return false;
        }

        // 检查访问次数
        if let Some(max_count) = self.max_access_count {
            if self.access_count >= max_count {
                return false;
            }
        }

        true
    }
}

/// 令牌管理器
pub struct TokenManager {
    /// 令牌存储 (token -> 令牌信息)
    tokens: Arc<RwLock<HashMap<String, StreamToken>>>,
    /// 默认过期时间（秒）
    default_ttl: u64,
}

impl TokenManager {
    /// 创建新的令牌管理器
    pub fn new(default_ttl: u64) -> Self {
        Self {
            tokens: Arc::new(RwLock::new(HashMap::new())),
            default_ttl,
        }
    }

    /// 生成新令牌
    pub async fn generate_token(&self, sim_number: &str, channel: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0); // 现代系统时间肯定在 UNIX EPOCH 之后

        let mut hasher = DefaultHasher::new();
        format!(
            "{}:{}:{}:{}",
            sim_number,
            channel,
            now,
            rand::random::<u64>()
        )
        .hash(&mut hasher);
        let token = format!("tk_{:016X}", hasher.finish());

        let stream_token = StreamToken {
            token: token.clone(),
            sim_number: sim_number.to_string(),
            channel: channel.to_string(),
            expires_at: now + self.default_ttl,
            created_at: now,
            access_count: 0,
            max_access_count: None,
        };

        let mut tokens = self.tokens.write().await;
        tokens.insert(token.clone(), stream_token);

        debug!("Generated token for stream {}:{}", sim_number, channel);
        token
    }

    /// 验证令牌
    pub async fn verify_token(&self, token: &str) -> bool {
        let mut tokens = self.tokens.write().await;

        if let Some(token_info) = tokens.get_mut(token) {
            if token_info.is_valid() {
                token_info.access_count += 1;
                true
            } else {
                // 移除失效令牌
                tokens.remove(token);
                false
            }
        } else {
            false
        }
    }

    /// 吊销令牌
    pub async fn revoke_token(&self, token: &str) -> bool {
        let mut tokens = self.tokens.write().await;
        tokens.remove(token).is_some()
    }

    /// 清理过期令牌
    pub async fn cleanup_expired_tokens(&self) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let mut tokens = self.tokens.write().await;
        let before_count = tokens.len();

        tokens.retain(|_, token_info| token_info.expires_at > now);

        let removed = before_count - tokens.len();
        if removed > 0 {
            debug!("Cleaned up {} expired tokens", removed);
        }
    }

    /// 获取令牌统计
    pub async fn get_stats(&self) -> TokenStats {
        let tokens = self.tokens.read().await;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let valid_count = tokens.values().filter(|t| t.expires_at > now).count();

        TokenStats {
            total_tokens: tokens.len(),
            valid_tokens: valid_count,
            expired_tokens: tokens.len() - valid_count,
        }
    }
}

/// 令牌统计信息
#[derive(Debug, Clone)]
pub struct TokenStats {
    pub total_tokens: usize,
    pub valid_tokens: usize,
    pub expired_tokens: usize,
}

/// 视频流鉴权中间件
pub struct StreamAuthMiddleware {
    token_manager: Arc<TokenManager>,
}

impl StreamAuthMiddleware {
    pub fn new(token_manager: Arc<TokenManager>) -> Self {
        Self { token_manager }
    }
}

impl<S, B> Transform<S, ServiceRequest> for StreamAuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = StreamAuthMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(StreamAuthMiddlewareService {
            service,
            token_manager: self.token_manager.clone(),
        })
    }
}

pub struct StreamAuthMiddlewareService<S> {
    service: S,
    token_manager: Arc<TokenManager>,
}

impl<S, B> Service<ServiceRequest> for StreamAuthMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let token_manager = self.token_manager.clone();

        // 从查询参数中提取token
        let query_string = req.query_string().to_string();
        let token = extract_token_from_query(&query_string);

        let token_valid = token.as_ref().is_some_and(|t| {
            // 注意：这里同步检查可能不够精确，实际应在async块中验证
            // 简化处理：先放行，在后续处理中验证
            !t.is_empty()
        });

        if !token_valid && !query_string.is_empty() {
            // 没有有效token，拒绝请求
            return Box::pin(async move {
                warn!("Authentication required for request");
                Err(actix_web::error::ErrorUnauthorized(
                    "Authentication required",
                ))
            });
        }

        let fut = self.service.call(req);

        Box::pin(async move {
            // 验证token
            if let Some(ref t) = token {
                if !token_manager.verify_token(t).await {
                    warn!("Invalid or expired token: {}", t);
                    return Err(actix_web::error::ErrorUnauthorized(
                        "Invalid or expired token",
                    ));
                }
            }

            // 令牌有效，继续处理请求
            fut.await
        })
    }
}

/// 从查询字符串中提取token
fn extract_token_from_query(query: &str) -> Option<String> {
    let pairs: HashMap<&str, &str> = query
        .split('&')
        .filter_map(|pair| {
            let mut parts = pair.splitn(2, '=');
            if let (Some(key), Some(value)) = (parts.next(), parts.next()) {
                Some((key, value))
            } else {
                None
            }
        })
        .collect();

    pairs
        .get("key")
        .map(|s| s.to_string())
        .or_else(|| pairs.get("token").map(|s| s.to_string()))
}

/// 创建令牌管理器（便捷函数）
pub fn create_token_manager(ttl: u64) -> Arc<TokenManager> {
    Arc::new(TokenManager::new(ttl))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_token_from_query() {
        assert_eq!(
            extract_token_from_query("key=test123&channel=1"),
            Some("test123".to_string())
        );
        assert_eq!(
            extract_token_from_query("token=abc&sim=123"),
            Some("abc".to_string())
        );
        assert_eq!(extract_token_from_query("channel=1"), None);
    }
}
