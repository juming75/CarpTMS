//! 统一电路断路器（Circuit Breaker）实现
//!
//! 整合自 `errors/circuit_breaker.rs`（三态状态机）和
//! `middleware/circuit_breaker.rs`（Actix-web 中间件）。
//!
//! # 使用
//!
//! ```ignore
//! // 同步调用
//! let result = cb.call(|| some_fallible_operation())?;
//!
//! // 异步调用
//! let result = cb.call_async(|| async { some_async_op().await }).await?;
//!
//! // 中间件模式
//! let guard = cb.guard();
//! if guard.is_allowed() { /* proceed */ } else { /* reject */ }
//! ```

use std::future::Future;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use thiserror::Error;

// ═══════════════════════════════════════════════════════════════
// 核心类型
// ═══════════════════════════════════════════════════════════════

/// 电路断路器状态
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

/// 电路断路器错误
#[derive(Debug, Error)]
pub enum CircuitBreakerError {
    #[error("Circuit is open (rejected)")]
    CircuitOpen,
    #[error("Circuit is half-open (probing)")]
    CircuitHalfOpen,
    #[error("Operation failed")]
    OperationFailed,
}

/// 电路断路器配置
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

// ═══════════════════════════════════════════════════════════════
// 核心电路断路器
// ═══════════════════════════════════════════════════════════════

pub struct CircuitBreaker {
    config: CircuitBreakerConfig,
    state: Arc<Mutex<CircuitState>>,
    failures: Arc<Mutex<Vec<Instant>>>,
    successes: Arc<Mutex<usize>>,
    last_state_change: Arc<Mutex<Instant>>,
    name: String,
}

impl CircuitBreaker {
    pub fn new(name: &str, config: CircuitBreakerConfig) -> Self {
        let cb = Self {
            config,
            state: Arc::new(Mutex::new(CircuitState::Closed)),
            failures: Arc::new(Mutex::new(Vec::new())),
            successes: Arc::new(Mutex::new(0)),
            last_state_change: Arc::new(Mutex::new(Instant::now())),
            name: name.to_string(),
        };
        cb.record_state();
        cb
    }

    pub fn with_defaults(name: &str) -> Self {
        Self::new(name, CircuitBreakerConfig::default())
    }

    /// 获取当前状态（用于监控/指标）
    pub fn state(&self) -> CircuitState {
        self.state
            .lock()
            .ok()
            .map(|s| s.clone())
            .unwrap_or(CircuitState::Closed)
    }

    // ── 同步调用 ──

    pub async fn call<F, T, E>(&self, f: F) -> Result<T, CircuitBreakerError>
    where
        F: Fn() -> Result<T, E>,
        E: std::error::Error + Send + Sync + 'static,
    {
        match self.current_state() {
            CircuitState::Closed => self.handle_closed(f).await,
            CircuitState::Open => self.handle_open::<T>().await,
            CircuitState::HalfOpen => self.handle_half_open(f).await,
        }
    }

    // ── 异步调用 ──

    pub async fn call_async<F, Fut, T, E>(&self, f: F) -> Result<T, CircuitBreakerError>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<T, E>> + Send,
        E: std::error::Error + Send + Sync + 'static,
    {
        match self.current_state() {
            CircuitState::Closed => self.handle_closed_async(f).await,
            CircuitState::Open => self.handle_open::<T>().await,
            CircuitState::HalfOpen => self.handle_half_open_async(f).await,
        }
    }

    // ── 快速检查（用于中间件/同步路径） ──

    /// 快速检查请求是否允许通过
    /// 返回 `false` 表示熔断器已打开，应拒绝请求
    pub fn is_allowed(&self) -> bool {
        match self.current_state() {
            CircuitState::Closed => true,
            CircuitState::Open => {
                if self.should_try_half_open() {
                    self.set_state(CircuitState::HalfOpen);
                    true
                } else {
                    false
                }
            }
            CircuitState::HalfOpen => true, // 半开状态放行以探测
        }
    }

    /// 记录失败（用于中间件路径，记录失败后自动判断是否打开）
    pub fn record_failure(&self) {
        if let Ok(mut failures) = self.failures.lock() {
            failures.push(Instant::now());
        }
        self.cleanup_failures();
        if self.should_open() {
            self.set_state(CircuitState::Open);
        }
    }

    /// 记录成功（用于中间件路径）
    pub fn record_success(&self) {
        self.clear_failures();
        if self.current_state() == CircuitState::HalfOpen {
            self.record_success_count();
            if self.should_close() {
                self.set_state(CircuitState::Closed);
            }
        }
    }

    // ── 内部方法 ──

    fn current_state(&self) -> CircuitState {
        self.state
            .lock()
            .ok()
            .map(|s| s.clone())
            .unwrap_or(CircuitState::Closed)
    }

    async fn handle_closed<F, T, E>(&self, f: F) -> Result<T, CircuitBreakerError>
    where
        F: Fn() -> Result<T, E>,
    {
        match f() {
            Ok(result) => {
                self.clear_failures();
                Ok(result)
            }
            Err(_) => {
                self.record_failure_raw();
                if self.should_open() {
                    self.set_state(CircuitState::Open);
                }
                Err(CircuitBreakerError::OperationFailed)
            }
        }
    }

    async fn handle_closed_async<F, Fut, T, E>(&self, f: F) -> Result<T, CircuitBreakerError>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<T, E>> + Send,
    {
        match f().await {
            Ok(result) => {
                self.clear_failures();
                Ok(result)
            }
            Err(_) => {
                self.record_failure_raw();
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
    {
        match f() {
            Ok(result) => {
                self.record_success_count();
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

    async fn handle_half_open_async<F, Fut, T, E>(&self, f: F) -> Result<T, CircuitBreakerError>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<T, E>> + Send,
    {
        match f().await {
            Ok(result) => {
                self.record_success_count();
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

    fn record_failure_raw(&self) {
        if let Ok(mut failures) = self.failures.lock() {
            failures.push(Instant::now());
        }
        self.cleanup_failures();
    }

    fn record_success_count(&self) {
        if let Ok(mut s) = self.successes.lock() {
            *s += 1;
        }
    }

    fn clear_failures(&self) {
        if let Ok(mut failures) = self.failures.lock() {
            failures.clear();
        }
        if let Ok(mut s) = self.successes.lock() {
            *s = 0;
        }
    }

    fn cleanup_failures(&self) {
        if let Ok(mut f) = self.failures.lock() {
            let now = Instant::now();
            f.retain(|&t| now.duration_since(t) < self.config.window_size);
        }
    }

    fn should_open(&self) -> bool {
        self.failures
            .lock()
            .ok()
            .map(|mut f| {
                let now = Instant::now();
                f.retain(|&t| now.duration_since(t) < self.config.window_size);
                f.len() >= self.config.failure_threshold
            })
            .unwrap_or(false)
    }

    fn should_close(&self) -> bool {
        self.successes
            .lock()
            .ok()
            .map(|s| *s >= self.config.success_threshold)
            .unwrap_or(false)
    }

    fn should_try_half_open(&self) -> bool {
        self.last_state_change
            .lock()
            .ok()
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
                if new_state == CircuitState::HalfOpen || new_state == CircuitState::Closed {
                    self.clear_failures();
                }
                self.record_state();
            }
        }
    }

    fn record_state(&self) {
        // TODO: integrate with metrics/prometheus
        if let Ok(state) = self.state.lock() {
            log::debug!("Circuit breaker '{}' state: {:?}", self.name, *state);
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// 全局熔断器管理器
// ═══════════════════════════════════════════════════════════════

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
        self.breakers.lock().ok().map(|mut b| {
            b.entry(name.to_string())
                .or_insert_with(|| Arc::new(CircuitBreaker::with_defaults(name)))
                .clone()
        })
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

pub static GLOBAL_CIRCUIT_BREAKER_MANAGER: once_cell::sync::OnceCell<Arc<CircuitBreakerManager>> =
    once_cell::sync::OnceCell::new();

pub fn init_circuit_breaker_manager() -> Result<(), String> {
    GLOBAL_CIRCUIT_BREAKER_MANAGER
        .set(Arc::new(CircuitBreakerManager::new()))
        .map_err(|_| "Already initialized".to_string())
}

pub fn get_circuit_breaker_manager() -> Result<Arc<CircuitBreakerManager>, String> {
    GLOBAL_CIRCUIT_BREAKER_MANAGER
        .get()
        .cloned()
        .ok_or_else(|| "Not initialized".to_string())
}

pub fn get_circuit_breaker(name: &str) -> Result<Arc<CircuitBreaker>, String> {
    get_circuit_breaker_manager().and_then(|m| {
        m.get_or_create(name)
            .ok_or_else(|| format!("Failed to get: {}", name))
    })
}

// ═══════════════════════════════════════════════════════════════
// 宏
// ═══════════════════════════════════════════════════════════════

#[macro_export]
macro_rules! with_circuit_breaker {
    ($name:expr, $code:block) => {{
        let breaker = $crate::errors::get_circuit_breaker($name)?;
        breaker.call(|| Ok($code)).await.map_err(|e| {
            $crate::errors::AppError::external_service_error(
                &format!("Circuit breaker: {:?}", e),
                Some(&e.to_string()),
            )
        })?
    }};
}

#[macro_export]
macro_rules! with_circuit_breaker_async {
    ($name:expr, $code:expr) => {{
        let breaker = $crate::errors::get_circuit_breaker($name)?;
        breaker.call_async(|| $code).await.map_err(|e| {
            $crate::errors::AppError::external_service_error(
                &format!("Circuit breaker: {:?}", e),
                Some(&e.to_string()),
            )
        })?
    }};
}
