use std::time::Duration;

// 重试策略结构体
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    pub max_attempts: usize,
    pub base_delay: Duration,
    pub max_delay: Duration,
    pub backoff_factor: f64,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(10),
            backoff_factor: 2.0,
        }
    }
}

impl RetryPolicy {
    // 创建快速重试策略(适合临时网络问题)
    pub fn quick() -> Self {
        Self {
            max_attempts: 3,
            base_delay: Duration::from_millis(50),
            max_delay: Duration::from_secs(2),
            backoff_factor: 2.0,
        }
    }

    // 创建慢速重试策略(适合服务恢复)
    pub fn slow() -> Self {
        Self {
            max_attempts: 5,
            base_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(10),
            backoff_factor: 1.5,
        }
    }

    // 计算第n次重试的延迟时间
    pub fn get_delay(&self, attempt: usize) -> Duration {
        if attempt == 0 {
            return Duration::from_secs(0);
        }

        let delay = self
            .base_delay
            .mul_f64(self.backoff_factor.powi(attempt as i32 - 1));
        if delay > self.max_delay {
            self.max_delay
        } else {
            delay
        }
    }
}

// 重试函数
pub async fn retry<F, T, E>(policy: &RetryPolicy, mut operation: F) -> Result<T, E>
where
    F: FnMut() -> futures::future::BoxFuture<'static, Result<T, E>>,
    E: std::error::Error + Send + Sync + 'static,
{
    let mut last_error: Option<E> = None;

    for attempt in 0..policy.max_attempts {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                last_error = Some(e);

                if attempt < policy.max_attempts - 1 {
                    // 记录重试
                    // 记录重试到监控系统
                    // TODO: 实现重试监控

                    let delay = policy.get_delay(attempt + 1);
                    tokio::time::sleep(delay).await;
                }
            }
        }
    }

    Err(last_error.expect("至少有一次错误"))
}

// 快速重试便捷函数
pub async fn quick_retry<F, T, E>(operation: F) -> Result<T, E>
where
    F: FnMut() -> futures::future::BoxFuture<'static, Result<T, E>>,
    E: std::error::Error + Send + Sync + 'static,
{
    let policy = RetryPolicy::quick();
    retry(&policy, operation).await
}

// 慢速重试便捷函数
pub async fn slow_retry<F, T, E>(operation: F) -> Result<T, E>
where
    F: FnMut() -> futures::future::BoxFuture<'static, Result<T, E>>,
    E: std::error::Error + Send + Sync + 'static,
{
    let policy = RetryPolicy::slow();
    retry(&policy, operation).await
}
