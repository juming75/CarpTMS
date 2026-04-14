use actix_web::{web, Error, HttpRequest};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

// 熔断器配置
struct CircuitBreakerConfig {
    failure_threshold: usize,  // 失败阈值
    reset_timeout: Duration,   // 重置超时时间
    #[allow(dead_code)]
    half_open_attempts: usize, // 半开状态尝试次数
}

// 熔断器
pub struct CircuitBreaker {
    state: Arc<AtomicBool>, // true表示Open，false表示Closed/HalfOpen
    failure_count: Arc<AtomicUsize>,
    last_failure_time: Arc<AtomicUsize>,
    config: CircuitBreakerConfig,
}

impl Default for CircuitBreaker {
    fn default() -> Self {
        Self::new()
    }
}

impl CircuitBreaker {
    pub fn new() -> Self {
        Self {
            state: Arc::new(AtomicBool::new(false)),
            failure_count: Arc::new(AtomicUsize::new(0)),
            last_failure_time: Arc::new(AtomicUsize::new(0)),
            config: CircuitBreakerConfig {
                failure_threshold: 5,
                reset_timeout: Duration::from_secs(30),
                half_open_attempts: 3,
            },
        }
    }

    // 检查熔断器状态
    pub fn check(&self) -> bool {
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as usize;
        let last_failure = self.last_failure_time.load(Ordering::SeqCst);

        // 如果是Open状态，检查是否可以切换到HalfOpen
        if self.state.load(Ordering::SeqCst) {
            // 计算时间差（秒）
            if last_failure > 0 && current_time > last_failure {
                let elapsed_secs = current_time - last_failure;
                
                if elapsed_secs > self.config.reset_timeout.as_secs() as usize {
                    // 切换到HalfOpen状态
                    self.state.store(false, Ordering::SeqCst);
                    self.failure_count.store(0, Ordering::SeqCst);
                    return true;
                }
            }
            return false;
        }

        true
    }

    // 记录失败
    pub fn record_failure(&self) {
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as usize;
        let count = self.failure_count.fetch_add(1, Ordering::SeqCst);

        if count >= self.config.failure_threshold - 1 {
            // 切换到Open状态
            self.state.store(true, Ordering::SeqCst);
            self.last_failure_time.store(current_time, Ordering::SeqCst);
        }
    }

    // 记录成功
    pub fn record_success(&self) {
        self.failure_count.store(0, Ordering::SeqCst);
    }
}

// 熔断器中间件
pub async fn circuit_breaker_middleware(
    _req: HttpRequest,
    circuit_breaker: web::Data<Arc<CircuitBreaker>>,
) -> Result<(), Error> {
    if !circuit_breaker.check() {
        return Err(actix_web::error::ErrorServiceUnavailable("服务暂时不可用"));
    }

    Ok(())
}
