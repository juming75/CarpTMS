use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use thiserror::Error;

// 电路断路器状态
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

// 电路断路器错误
#[derive(Debug, Error)]
pub enum CircuitBreakerError {
    #[error("Circuit is open")]
    CircuitOpen,
    #[error("Circuit is half-open")]
    CircuitHalfOpen,
    #[error("Operation failed")]
    OperationFailed,
}

// 电路断路器配置
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    pub failure_threshold: usize,
    pub success_threshold: usize,
    pub reset_timeout: Duration,
    pub window_size: Duration,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            success_threshold: 3,
            reset_timeout: Duration::from_secs(30),
            window_size: Duration::from_secs(60),
        }
    }
}

// 电路断路器结构体
pub struct CircuitBreaker {
    config: CircuitBreakerConfig,
    state: Arc<Mutex<CircuitState>>,
    failures: Arc<Mutex<Vec<Instant>>>,
    successes: Arc<Mutex<usize>>,
    last_state_change: Arc<Mutex<Instant>>,
    _name: String,
}

impl CircuitBreaker {
    pub fn new(name: &str, config: CircuitBreakerConfig) -> Self {
        let circuit_breaker = Self {
            config,
            state: Arc::new(Mutex::new(CircuitState::Closed)),
            failures: Arc::new(Mutex::new(Vec::new())),
            successes: Arc::new(Mutex::new(0)),
            last_state_change: Arc::new(Mutex::new(Instant::now())),
            _name: name.to_string(),
        };

        // 记录初始状态
        circuit_breaker.record_state();
        circuit_breaker
    }

    pub fn with_defaults(name: &str) -> Self {
        Self::new(name, CircuitBreakerConfig::default())
    }

    pub async fn call<F, T, E>(&self, f: F) -> Result<T, CircuitBreakerError>
    where
        F: Fn() -> Result<T, E>,
        E: std::error::Error + Send + Sync + 'static,
    {
        let state = self.state.lock().ok().map(|s| s.clone()).unwrap_or(CircuitState::Closed);
        match state {
            CircuitState::Closed => self.handle_closed(f).await,
            CircuitState::Open => self.handle_open::<T>().await,
            CircuitState::HalfOpen => self.handle_half_open(f).await,
        }
    }

    async fn handle_closed<F, T, E>(&self, f: F) -> Result<T, CircuitBreakerError>
    where
        F: Fn() -> Result<T, E>,
        E: std::error::Error + Send + Sync + 'static,
    {
        match f() {
            Ok(result) => {
                self.clear_failures();
                Ok(result)
            }
            Err(_) => {
                self.record_failure();
                if self.should_open() {
                    self.set_state(CircuitState::Open);
                }
                Err(CircuitBreakerError::OperationFailed)
            }
        }
    }

    async fn handle_open<T>(&self) -> Result<T, CircuitBreakerError> {
        if self.should_try_half_open() {
            self.set_state(CircuitState::HalfOpen);
            Err(CircuitBreakerError::CircuitHalfOpen)
        } else {
            Err(CircuitBreakerError::CircuitOpen)
        }
    }

    async fn handle_half_open<F, T, E>(&self, f: F) -> Result<T, CircuitBreakerError>
    where
        F: Fn() -> Result<T, E>,
        E: std::error::Error + Send + Sync + 'static,
    {
        match f() {
            Ok(result) => {
                self.record_success();
                if self.should_close() {
                    self.set_state(CircuitState::Closed);
                }
                Ok(result)
            }
            Err(_) => {
                self.set_state(CircuitState::Open);
                Err(CircuitBreakerError::OperationFailed)
            }
        }
    }

    fn record_failure(&self) {
        if let Ok(mut failures) = self.failures.lock() {
            failures.push(Instant::now());
            drop(failures);
            self.cleanup_failures();
        }
    }

    fn record_success(&self) {
        if let Ok(mut successes) = self.successes.lock() {
            *successes += 1;
        }
    }

    fn clear_failures(&self) {
        if let Ok(mut failures) = self.failures.lock() {
            failures.clear();
        }
    }

    fn cleanup_failures(&self) {
        if let Ok(mut failures) = self.failures.lock() {
            let now = Instant::now();
            failures.retain(|&time| now.duration_since(time) < self.config.window_size);
        }
    }

    fn should_open(&self) -> bool {
        self.failures.lock().ok()
            .map(|mut f| {
                let now = Instant::now();
                f.retain(|&time| now.duration_since(time) < self.config.window_size);
                f.len() >= self.config.failure_threshold
            })
            .unwrap_or(false)
    }

    fn should_close(&self) -> bool {
        self.successes.lock().ok()
            .map(|s| *s >= self.config.success_threshold)
            .unwrap_or(false)
    }

    fn should_try_half_open(&self) -> bool {
        self.last_state_change.lock().ok()
            .map(|l| Instant::now().duration_since(*l) >= self.config.reset_timeout)
            .unwrap_or(false)
    }

    fn set_state(&self, new_state: CircuitState) {
        if let Ok(mut state) = self.state.lock() {
            if *state != new_state {
                *state = new_state.clone();
                if let Ok(mut lc) = self.last_state_change.lock() {
                    *lc = Instant::now();
                }
                if new_state == CircuitState::HalfOpen {
                    if let Ok(mut s) = self.successes.lock() { *s = 0; }
                } else if new_state == CircuitState::Closed {
                    self.clear_failures();
                    if let Ok(mut s) = self.successes.lock() { *s = 0; }
                }
                self.record_state();
            }
        }
    }

    // 记录电路断路器状态到监控系统
    fn record_state(&self) {
        if let Ok(state) = self.state.lock() {
            let _state_value = match *state {
                CircuitState::Closed => 0,
                CircuitState::Open => 1,
                CircuitState::HalfOpen => 2,
            };
            // TODO: integrate with metrics crate
        }
    }
}

// 全局电路断路器管理器
pub struct CircuitBreakerManager {
    breakers: Arc<Mutex<std::collections::HashMap<String, Arc<CircuitBreaker>>>>,
}

impl CircuitBreakerManager {
    pub fn new() -> Self {
        Self {
            breakers: Arc::new(Mutex::new(std::collections::HashMap::new())),
        }
    }

    pub fn get_or_create(&self, name: &str) -> Option<Arc<CircuitBreaker>> {
        if let Ok(mut breakers) = self.breakers.lock() {
            return Some(breakers
                .entry(name.to_string())
                .or_insert_with(|| Arc::new(CircuitBreaker::with_defaults(name)))
                .clone());
        }
        None
    }

    pub fn get(&self, name: &str) -> Option<Arc<CircuitBreaker>> {
        self.breakers.lock().ok()?.get(name).cloned()
    }
}

impl Default for CircuitBreakerManager {
    fn default() -> Self {
        Self::new()
    }
}

// 全局电路断路器管理器实例
pub static GLOBAL_CIRCUIT_BREAKER_MANAGER: once_cell::sync::OnceCell<Arc<CircuitBreakerManager>> =
    once_cell::sync::OnceCell::new();

// 初始化全局电路断路器管理器
pub fn init_circuit_breaker_manager() -> Result<(), String> {
    GLOBAL_CIRCUIT_BREAKER_MANAGER
        .set(Arc::new(CircuitBreakerManager::new()))
        .map_err(|_| "Failed to initialize circuit breaker manager".to_string())
}

// 获取全局电路断路器管理器
pub fn get_circuit_breaker_manager() -> Result<Arc<CircuitBreakerManager>, String> {
    GLOBAL_CIRCUIT_BREAKER_MANAGER
        .get()
        .cloned()
        .ok_or_else(|| "Circuit breaker manager not initialized".to_string())
}

// 获取或创建电路断路器
pub fn get_circuit_breaker(name: &str) -> Result<Arc<CircuitBreaker>, String> {
    get_circuit_breaker_manager()
        .and_then(|manager| manager.get_or_create(name).ok_or_else(|| format!("Failed to get circuit breaker: {}", name)))
}

// 电路断路器宏
#[macro_export]
macro_rules! with_circuit_breaker {
    ($name:expr, $code:block) => {{
        let breaker = $crate::errors::get_circuit_breaker($name);
        breaker.call(|| Ok($code)).await.map_err(|e| {
            $crate::errors::AppError::external_service_error(
                &format!("Circuit breaker error: {:?}", e),
                Some(&e.to_string()),
            )
        })?
    }};
}
