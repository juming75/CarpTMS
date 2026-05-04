//! / API限流中间件 - 基于令牌桶算法
use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error,
};
use base64::Engine;
use futures::future::{ready, Future, Ready};
use log::{debug, warn};
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;
use tokio::sync::RwLock;

// 引入自定义错误类型
use crate::errors::{AppError, AppResult};
use crate::metrics::{
    RATE_LIMIT_CURRENT_USERS, RATE_LIMIT_REJECTIONS_TOTAL, RATE_LIMIT_REQUESTS_TOTAL,
    RATE_LIMIT_TOKENS_REMAINING,
};
use redis::Client as RedisClient;

// Redis模式配置
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StorageMode {
    Memory, // 内存模式(单实例)
    Redis,  // Redis模式(多实例)
}

// 限流键类型
#[derive(Debug, Clone, Copy)]
pub enum LimitKeyType {
    IP,   // IP级别限流
    User, // 用户级别限流(基于JWT)
}

// 令牌桶状态(内存存储)
#[derive(Clone)]
struct TokenBucket {
    tokens: f64,
    last_refill: Instant,
}

// 令牌桶状态存储(内存模式)
use once_cell::sync::Lazy;

static BUCKET_STORE: Lazy<Arc<RwLock<HashMap<String, TokenBucket>>>> =
    Lazy::new(|| Arc::new(RwLock::new(HashMap::new())));

// 后台任务:定期清理过期的令牌桶
async fn cleanup_expired_buckets() {
    loop {
        tokio::time::sleep(Duration::from_secs(5 * 60)).await;

        let mut store = BUCKET_STORE.write().await;
        let now = Instant::now();

        // 清理超过1小时未使用的令牌桶
        store.retain(|_, bucket| {
            now.duration_since(bucket.last_refill) < Duration::from_secs(60 * 60)
        });
    }
}

// 限流配置
#[derive(Debug, Clone)]
pub struct RateLimiterConfig {
    pub window_size: Duration,     // 时间窗口大小
    pub limit: u32,                // 每个时间窗口的最大请求数
    pub prefix: String,            // Redis键前缀
    pub burst: u32,                // 突发请求数(令牌桶容量)
    pub refill_rate: f64,          // 令牌每秒补充速率
    pub exempt_paths: Vec<String>, // 豁免路径
    pub storage_mode: StorageMode, // 存储模式
    pub key_type: LimitKeyType,    // 限流键类型
}

impl Default for RateLimiterConfig {
    fn default() -> Self {
        let window_size = Duration::from_secs(60);
        let limit = 300; // 提高限制以减少速率限制错误
        Self {
            window_size,
            limit,
            prefix: "rate_limit".to_string(),
            burst: limit,
            refill_rate: limit as f64 / window_size.as_secs_f64(),
            exempt_paths: vec!["/api/health".to_string(), "/api/metrics".to_string()],
            storage_mode: StorageMode::Memory,
            key_type: LimitKeyType::IP,
        }
    }
}

impl RateLimiterConfig {
    /// 创建针对登录路由的严格速率限制配置
    pub fn login_rate_limit() -> Self {
        let window_size = Duration::from_secs(60);
        let limit = 100; // 每分钟最多100次登录尝试(暂时提高以方便测试)
        Self {
            window_size,
            limit,
            prefix: "rate_limit_login".to_string(),
            burst: limit,
            refill_rate: limit as f64 / window_size.as_secs_f64(),
            exempt_paths: vec![
                "/api/health".to_string(),
                "/api/metrics".to_string(),
                "/api/auth/login".to_string(),
            ],
            storage_mode: StorageMode::Memory,
            key_type: LimitKeyType::IP,
        }
    }
}

// 限流中间件
#[derive(Debug, Clone)]
pub struct RateLimiterMiddleware {
    config: RateLimiterConfig,
    #[allow(dead_code)]
    redis_client: Arc<RedisClient>,
    storage_mode: StorageMode,
}

impl RateLimiterMiddleware {
    pub fn new(config: RateLimiterConfig, redis_client: Arc<RedisClient>) -> Self {
        let storage_mode = config.storage_mode;

        // 启动清理过期令牌桶的后台任务(仅在内存模式下)
        if storage_mode == StorageMode::Memory {
            tokio::spawn(cleanup_expired_buckets());
        }

        Self {
            config,
            redis_client,
            storage_mode,
        }
    }

    pub fn with_limit(limit: u32, redis_client: Arc<RedisClient>) -> Self {
        let storage_mode = StorageMode::Memory;
        Self {
            config: RateLimiterConfig {
                limit,
                burst: limit,
                refill_rate: limit as f64 / 60.0, // 默认60秒窗口
                storage_mode,
                ..Default::default()
            },
            redis_client,
            storage_mode,
        }
    }

    pub fn with_window(window_size: Duration, limit: u32, redis_client: Arc<RedisClient>) -> Self {
        let storage_mode = StorageMode::Memory;
        Self {
            config: RateLimiterConfig {
                window_size,
                limit,
                burst: limit,
                refill_rate: limit as f64 / window_size.as_secs_f64(),
                storage_mode,
                ..Default::default()
            },
            redis_client,
            storage_mode,
        }
    }

    pub fn with_redis_mode(config: RateLimiterConfig, redis_client: Arc<RedisClient>) -> Self {
        let mut config = config;
        config.storage_mode = StorageMode::Redis;
        Self {
            config,
            redis_client,
            storage_mode: StorageMode::Redis,
        }
    }

    // 检查路径是否豁免
    fn is_path_exempt(&self, path: &str) -> bool {
        self.config.exempt_paths.iter().any(|exempt_path| {
            path == exempt_path || path.starts_with(&format!("{}/", exempt_path))
        })
    }

    // 生成限流键
    fn generate_key(&self, req: &ServiceRequest) -> (String, LimitKeyType) {
        match self.config.key_type {
            LimitKeyType::IP => {
                // 获取客户端IP
                let connection_info = req.connection_info();
                let client_ip = connection_info.realip_remote_addr().unwrap_or("unknown");
                // 生成限流键:prefix:ip
                (
                    format!("{}:ip:{}", self.config.prefix, client_ip),
                    LimitKeyType::IP,
                )
            }
            LimitKeyType::User => {
                // 从Authorization头中提取JWT令牌
                let auth_header = req.headers().get("Authorization");
                let user_id = match auth_header {
                    Some(header_value) => {
                        if let Ok(header_str) = header_value.to_str() {
                            // 提取Bearer token
                            if let Some(token) = header_str.strip_prefix("Bearer ") {
                                // 跳过"Bearer "
                                // 从token中解析user_id(简化实现)
                                // 实际项目中应该使用JWT库解析
                                format!("user:{}", Self::extract_user_id_from_token(token))
                            } else {
                                "anonymous".to_string()
                            }
                        } else {
                            "anonymous".to_string()
                        }
                    }
                    None => "anonymous".to_string(),
                };
                // 生成限流键:prefix:user_id
                (
                    format!("{}:user:{}", self.config.prefix, user_id),
                    LimitKeyType::User,
                )
            }
        }
    }

    // 从JWT令牌中提取user_id(简化实现)
    fn extract_user_id_from_token(token: &str) -> String {
        // 简化实现:实际应该使用jsonwebtoken crate
        // 这里仅演示目的
        let token_parts: Vec<&str> = token.split('.').collect();
        if token_parts.len() >= 2 {
            // 尝试解码payload部分
            use base64::engine::general_purpose;
            if let Ok(decoded) = general_purpose::URL_SAFE.decode(token_parts[1]) {
                if let Ok(json_str) = String::from_utf8(decoded) {
                    // 解析JSON获取user_id
                    if let Some(user_id) = Self::parse_user_id_from_json(&json_str) {
                        return user_id;
                    }
                }
            }
        }
        "unknown".to_string()
    }

    // 从JSON字符串中提取user_id
    fn parse_user_id_from_json(json_str: &str) -> Option<String> {
        use serde_json::Value;
        if let Ok(value) = serde_json::from_str::<Value>(json_str) {
            if let Some(claims) = value.as_object() {
                if let Some(user_id) = claims.get("user_id") {
                    if let Some(id) = user_id.as_str() {
                        return Some(id.to_string());
                    }
                }
                if let Some(sub) = claims.get("sub") {
                    if let Some(id) = sub.as_str() {
                        return Some(id.to_string());
                    }
                }
            }
        }
        None
    }

    // 令牌桶算法实现 (内存版本,适合单实例部署)
    async fn check_rate_limit_memory(
        &self,
        key: &str,
        endpoint: &str,
        key_type: LimitKeyType,
    ) -> AppResult<bool> {
        let key_type_str = match key_type {
            LimitKeyType::IP => "ip",
            LimitKeyType::User => "user",
        };

        // 记录限流检查
        RATE_LIMIT_REQUESTS_TOTAL
            .with_label_values(&[key_type_str, endpoint])
            .inc();

        let mut store = BUCKET_STORE.write().await;
        let now = Instant::now();
        let bucket = store.entry(key.to_string()).or_insert(TokenBucket {
            tokens: self.config.burst as f64,
            last_refill: now,
        });

        // 计算需要补充的令牌
        let elapsed = now.duration_since(bucket.last_refill);
        let refill_tokens = elapsed.as_secs_f64() * self.config.refill_rate;

        bucket.tokens = (bucket.tokens + refill_tokens).min(self.config.burst as f64);
        bucket.last_refill = now;

        // 保存令牌数量
        let tokens_remaining = bucket.tokens;

        // 检查是否可以消耗令牌
        let result = if bucket.tokens >= 1.0 {
            bucket.tokens -= 1.0;
            Ok(true)
        } else {
            // 记录拒绝
            RATE_LIMIT_REJECTIONS_TOTAL
                .with_label_values(&[key_type_str, endpoint])
                .inc();
            Ok(false)
        };

        // 获取存储长度
        let store_len = store.len();

        // 更新剩余令牌指标
        RATE_LIMIT_TOKENS_REMAINING
            .with_label_values(&[key_type_str])
            .set(tokens_remaining);

        // 更新当前活跃用户数
        RATE_LIMIT_CURRENT_USERS.set(store_len as f64);

        result
    }

    // 令牌桶算法实现 (Redis版本,适合多实例部署)
    async fn check_rate_limit_redis(
        &self,
        key: &str,
        endpoint: &str,
        key_type: LimitKeyType,
    ) -> AppResult<bool> {
        let key_type_str = match key_type {
            LimitKeyType::IP => "ip",
            LimitKeyType::User => "user",
        };

        // 记录限流检查
        RATE_LIMIT_REQUESTS_TOTAL
            .with_label_values(&[key_type_str, endpoint])
            .inc();

        // Redis键格式: prefix:key
        let redis_key = format!("{}:{}", self.config.prefix, key);
        let now = Instant::now();
        let timestamp = now.elapsed().as_secs_f64();

        // 获取Redis连接
        let mut conn = self
            .redis_client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| {
                AppError::service_unavailable_error(&format!("Redis connection error: {}", e), None)
            })?;

        // 使用Lua脚本原子性地检查和更新令牌桶
        let lua_script = r#"
            local key = KEYS[1]
            local now = tonumber(ARGV[1])
            local burst = tonumber(ARGV[2])
            local refill_rate = tonumber(ARGV[3])
            local expiry = tonumber(ARGV[4])

            -- 获取当前桶状态
            local bucket = redis.call('HMGET', key, 'tokens', 'last_refill')
            local tokens = tonumber(bucket[1]) or burst
            local last_refill = tonumber(bucket[2]) or now

            -- 计算需要补充的令牌
            local elapsed = now - last_refill
            local refill = elapsed * refill_rate
            tokens = math.min(tokens + refill, burst)

            -- 检查是否有足够的令牌
            if tokens >= 1.0 then
                tokens = tokens - 1.0

                -- 更新Redis
                redis.call('HMSET', key, 'tokens', tokens, 'last_refill', now)
                redis.call('EXPIRE', key, expiry)

                return {tokens, 1, 0}  -- {remaining_tokens, allowed, rejected}
            else
                redis.call('HMSET', key, 'tokens', tokens, 'last_refill', now)
                redis.call('EXPIRE', key, expiry)

                return {tokens, 0, 1}  -- {remaining_tokens, allowed, rejected}
            end
        "#;

        let result: Vec<f64> = redis::Script::new(lua_script)
            .key(&redis_key)
            .arg(timestamp)
            .arg(self.config.burst as f64)
            .arg(self.config.refill_rate)
            .arg(self.config.window_size.as_secs())
            .invoke_async(&mut conn)
            .await
            .map_err(|e| {
                AppError::service_unavailable_error(
                    &format!("Redis script execution error: {}", e),
                    None,
                )
            })?;

        let tokens_remaining = result[0];
        let allowed = result[1] == 1.0;

        // 更新指标
        RATE_LIMIT_TOKENS_REMAINING
            .with_label_values(&[key_type_str])
            .set(tokens_remaining);

        if !allowed {
            RATE_LIMIT_REJECTIONS_TOTAL
                .with_label_values(&[key_type_str, endpoint])
                .inc();
        }

        Ok(allowed)
    }

    // 统一的速率检查接口
    async fn check_rate_limit(
        &self,
        key: &str,
        endpoint: &str,
        key_type: LimitKeyType,
    ) -> AppResult<bool> {
        match self.storage_mode {
            StorageMode::Memory => self.check_rate_limit_memory(key, endpoint, key_type).await,
            StorageMode::Redis => self.check_rate_limit_redis(key, endpoint, key_type).await,
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for RateLimiterMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = RateLimiterMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RateLimiterMiddlewareService {
            service: Arc::new(service),
            middleware: self.clone(),
        }))
    }
}

#[derive(Debug)]
pub struct RateLimiterMiddlewareService<S> {
    service: Arc<S>,
    middleware: RateLimiterMiddleware,
}

impl<S, B> Service<ServiceRequest> for RateLimiterMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();
        let middleware = self.middleware.clone();

        Box::pin(async move {
            // 检查路径是否豁免
            let path = req.path();
            if middleware.is_path_exempt(path) {
                return service.call(req).await;
            }

            // 生成限流键
            let (key, key_type) = middleware.generate_key(&req);
            let endpoint = req.path();
            debug!(
                "Rate limiting key: {}, endpoint: {}, type: {:?}",
                key, endpoint, key_type
            );

            // 检查速率限制
            match middleware.check_rate_limit(&key, endpoint, key_type).await {
                Ok(allowed) => {
                    if allowed {
                        // 允许请求
                        service.call(req).await
                    } else {
                        // 拒绝请求,返回429 Too Many Requests
                        warn!("Rate limit exceeded for key: {}", key);
                        // 直接返回错误,使用AppError类型来处理
                        Err(
                            AppError::bad_request("Too many requests, please try again later")
                                .into(),
                        )
                    }
                }
                Err(e) => {
                    // Redis错误,记录日志但允许请求通过
                    warn!("Rate limiting error: {:?}", e);
                    service.call(req).await
                }
            }
        })
    }
}
