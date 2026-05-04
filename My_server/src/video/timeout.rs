//! 超时和重试策略统一配置
//!
//! 为所有外部调用提供标准化的超时和重试配置
//!
//! # 设计原则
//! - 统一的超时配置
//! - 指数退避重试策略
//! - 降级处理机制
//! - 可观测性（metrics + logging）
//!
//! # 使用示例
//! ```ignore
//! use crate::video::timeout::{with_timeout, RetryConfig, retry_with_backoff};
//!
//! // 带超时的操作
//! let result = with_timeout(
//!     async move { some_operation().await },
//!     Duration::from_secs(5),
//!     "operation_name"
//! ).await;
//!
//! // 带重试的操作
//! let result = retry_with_backoff(
//!     async move { some_fallible_operation().await },
//!     RetryConfig::default()
//! ).await;
//! ```

use std::future::Future;
use std::pin::Pin;
use std::time::Duration;
use tokio::time::{sleep, timeout};
use tracing::{error, info, warn};

/// 超时配置常量
#[derive(Debug, Clone, Copy)]
pub struct TimeoutConfig {
    /// 连接超时
    pub connect_timeout: Duration,
    /// 读取超时
    pub read_timeout: Duration,
    /// 写入超时
    pub write_timeout: Duration,
    /// 整体操作超时
    pub operation_timeout: Duration,
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        Self {
            connect_timeout: Duration::from_secs(5),
            read_timeout: Duration::from_secs(30),
            write_timeout: Duration::from_secs(30),
            operation_timeout: Duration::from_secs(60),
        }
    }
}

impl TimeoutConfig {
    /// 创建快速配置（适用于实时操作）
    pub fn fast() -> Self {
        Self {
            connect_timeout: Duration::from_secs(2),
            read_timeout: Duration::from_secs(5),
            write_timeout: Duration::from_secs(5),
            operation_timeout: Duration::from_secs(10),
        }
    }

    /// 创建标准配置（适用于普通操作）
    pub fn standard() -> Self {
        Self::default()
    }

    /// 创建慢速配置（适用于批处理等）
    pub fn slow() -> Self {
        Self {
            connect_timeout: Duration::from_secs(10),
            read_timeout: Duration::from_secs(120),
            write_timeout: Duration::from_secs(120),
            operation_timeout: Duration::from_secs(300),
        }
    }

    /// 创建数据库配置
    pub fn database() -> Self {
        Self {
            connect_timeout: Duration::from_secs(10),
            read_timeout: Duration::from_secs(30),
            write_timeout: Duration::from_secs(30),
            operation_timeout: Duration::from_secs(60),
        }
    }

    /// 创建缓存配置
    pub fn cache() -> Self {
        Self {
            connect_timeout: Duration::from_secs(1),
            read_timeout: Duration::from_secs(2),
            write_timeout: Duration::from_secs(2),
            operation_timeout: Duration::from_secs(5),
        }
    }

    /// 创建外部 API 配置
    pub fn external_api() -> Self {
        Self {
            connect_timeout: Duration::from_secs(10),
            read_timeout: Duration::from_secs(60),
            write_timeout: Duration::from_secs(30),
            operation_timeout: Duration::from_secs(120),
        }
    }
}

/// 重试配置
#[derive(Debug, Clone, Copy)]
pub struct RetryConfig {
    /// 最大重试次数
    pub max_retries: u32,
    /// 初始重试延迟
    pub initial_delay: Duration,
    /// 最大重试延迟
    pub max_delay: Duration,
    /// 指数退避基数
    pub backoff_base: u32,
    /// 是否使用抖动
    pub use_jitter: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(10),
            backoff_base: 2,
            use_jitter: true,
        }
    }
}

impl RetryConfig {
    /// 创建最小重试配置（仅重试1次）
    pub fn minimal() -> Self {
        Self {
            max_retries: 1,
            initial_delay: Duration::from_millis(50),
            max_delay: Duration::from_secs(1),
            backoff_base: 2,
            use_jitter: true,
        }
    }

    /// 创建保守重试配置（重试多次，长延迟）
    pub fn conservative() -> Self {
        Self {
            max_retries: 5,
            initial_delay: Duration::from_millis(500),
            max_delay: Duration::from_secs(30),
            backoff_base: 2,
            use_jitter: true,
        }
    }

    /// 创建无重试配置
    pub fn no_retry() -> Self {
        Self {
            max_retries: 0,
            initial_delay: Duration::ZERO,
            max_delay: Duration::ZERO,
            backoff_base: 2,
            use_jitter: false,
        }
    }

    /// 计算下一次重试的延迟
    pub fn calculate_delay(&self, attempt: u32) -> Duration {
        let base_delay = self.initial_delay * self.backoff_base.pow(attempt);
        let delay = base_delay.min(self.max_delay);

        if self.use_jitter {
            // 添加随机抖动 (0.5x - 1.5x)
            use std::time::Instant;
            let jitter_range = delay / 2;
            let start = delay - jitter_range;
            let random_offset =
                (Instant::now().elapsed().as_nanos() % jitter_range.as_nanos()) as u64;
            Duration::from_nanos(random_offset) + start
        } else {
            delay
        }
    }
}

/// 超时错误类型
#[derive(Debug)]
pub enum TimeoutError {
    /// 操作超时
    OperationTimedOut {
        operation: String,
        duration: Duration,
    },
    /// 操作被取消
    OperationCancelled { operation: String },
}

impl std::fmt::Display for TimeoutError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OperationTimedOut {
                operation,
                duration,
            } => {
                write!(
                    f,
                    "Operation '{}' timed out after {:?}",
                    operation, duration
                )
            }
            Self::OperationCancelled { operation } => {
                write!(f, "Operation '{}' was cancelled", operation)
            }
        }
    }
}

impl std::error::Error for TimeoutError {}

/// 带超时的操作执行
pub async fn with_timeout<F, T, E>(
    future: F,
    duration: Duration,
    operation_name: &str,
) -> Result<T, TimeoutError>
where
    F: Future<Output = Result<T, E>>,
{
    match timeout(duration, future).await {
        Ok(Ok(result)) => Ok(result),
        Ok(Err(_e)) => Err(TimeoutError::OperationCancelled {
            operation: operation_name.to_string(),
        }),
        Err(_) => {
            warn!(
                operation = operation_name,
                timeout_ms = duration.as_millis(),
                "Operation timed out"
            );
            Err(TimeoutError::OperationTimedOut {
                operation: operation_name.to_string(),
                duration,
            })
        }
    }
}

/// 带超时的操作执行（忽略错误类型）
pub async fn with_timeout_ignore_error<F, T>(
    future: F,
    duration: Duration,
    operation_name: &str,
) -> Option<T>
where
    F: Future<Output = T>,
{
    match timeout(duration, future).await {
        Ok(result) => Some(result),
        Err(_) => {
            warn!(
                operation = operation_name,
                timeout_ms = duration.as_millis(),
                "Operation timed out"
            );
            None
        }
    }
}

/// 使用指数退避策略重试操作
pub async fn retry_with_backoff<F, T, E>(mut operation: F, config: RetryConfig) -> Result<T, E>
where
    F: FnMut() -> Pin<Box<dyn Future<Output = Result<T, E>>>>,
{
    let mut last_error = None;

    for attempt in 0..=config.max_retries {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                last_error = Some(e);

                if attempt < config.max_retries {
                    let delay = config.calculate_delay(attempt);
                    info!(
                        attempt = attempt + 1,
                        max_retries = config.max_retries,
                        delay_ms = delay.as_millis(),
                        "Retrying operation after error"
                    );
                    sleep(delay).await;
                }
            }
        }
    }

    Err(last_error.unwrap())
}

/// 带超时和重试的操作
pub async fn retry_with_timeout<F, T, E>(
    mut operation: F,
    timeout_duration: Duration,
    retry_config: RetryConfig,
    operation_name: &str,
) -> Result<T, TimeoutError>
where
    F: FnMut() -> Pin<Box<dyn Future<Output = Result<T, E>>>>,
{
    let start = std::time::Instant::now();

    for attempt in 0..=retry_config.max_retries {
        // 检查是否超时
        if start.elapsed() >= timeout_duration {
            warn!(
                operation = operation_name,
                attempt = attempt,
                elapsed_ms = start.elapsed().as_millis(),
                "Operation timed out during retry"
            );
            return Err(TimeoutError::OperationTimedOut {
                operation: operation_name.to_string(),
                duration: start.elapsed(),
            });
        }

        match timeout(timeout_duration - start.elapsed(), operation()).await {
            Ok(Ok(result)) => return Ok(result),
            Ok(Err(_e)) => {
                if attempt < retry_config.max_retries {
                    let delay = retry_config.calculate_delay(attempt);
                    info!(
                        operation = operation_name,
                        attempt = attempt + 1,
                        max_retries = retry_config.max_retries,
                        delay_ms = delay.as_millis(),
                        "Retrying after error"
                    );
                    sleep(delay).await;
                }
            }
            Err(_) => {
                return Err(TimeoutError::OperationTimedOut {
                    operation: operation_name.to_string(),
                    duration: start.elapsed(),
                });
            }
        }
    }

    Err(TimeoutError::OperationTimedOut {
        operation: operation_name.to_string(),
        duration: start.elapsed(),
    })
}

/// 降级处理函数类型
pub type FallbackFn<T> = Box<dyn Fn() -> T + Send + Sync>;

/// 带降级方案的操作执行
pub async fn with_fallback<T, F, Fut>(
    operation: impl Future<Output = T>,
    fallback: FallbackFn<T>,
    timeout_duration: Duration,
    operation_name: &str,
) -> T
where
    T: Clone + Send + Sync + 'static,
    F: Future<Output = T>,
{
    match with_timeout_ignore_error(operation, timeout_duration, operation_name).await {
        Some(result) => result,
        None => {
            error!(
                operation = operation_name,
                "Operation failed, using fallback"
            );
            fallback()
        }
    }
}

/// 连接超时配置
#[derive(Debug, Clone, Copy)]
pub struct ConnectTimeout {
    /// TCP 连接超时
    pub tcp_connect: Duration,
    /// DNS 解析超时
    pub dns_resolve: Duration,
    /// TLS 握手超时
    pub tls_handshake: Duration,
    /// 总连接超时
    pub total: Duration,
}

impl Default for ConnectTimeout {
    fn default() -> Self {
        Self {
            tcp_connect: Duration::from_secs(5),
            dns_resolve: Duration::from_secs(2),
            tls_handshake: Duration::from_secs(10),
            total: Duration::from_secs(15),
        }
    }
}

/// 请求超时装饰器
pub struct RequestTimeout {
    pub read: Duration,
    pub write: Duration,
}

impl Default for RequestTimeout {
    fn default() -> Self {
        Self {
            read: Duration::from_secs(30),
            write: Duration::from_secs(30),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_timeout_success() {
        let result =
            with_timeout(async { Ok::<_, ()>(42) }, Duration::from_secs(1), "test_op").await;

        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_timeout_failure() {
        let result = with_timeout(
            async {
                sleep(Duration::from_secs(2)).await;
                Ok::<_, ()>(42)
            },
            Duration::from_millis(100),
            "test_op",
        )
        .await;

        assert!(result.is_err());
    }

    #[test]
    fn test_retry_config_delay() {
        let config = RetryConfig::default();

        assert_eq!(config.calculate_delay(0), Duration::from_millis(200)); // 100 * 2^0
        assert_eq!(config.calculate_delay(1), Duration::from_millis(400)); // 100 * 2^1
        assert_eq!(config.calculate_delay(2), Duration::from_millis(800)); // 100 * 2^2
    }

    #[test]
    fn test_timeout_config_presets() {
        let fast = TimeoutConfig::fast();
        assert_eq!(fast.connect_timeout, Duration::from_secs(2));

        let db = TimeoutConfig::database();
        assert_eq!(db.connect_timeout, Duration::from_secs(10));
    }
}
