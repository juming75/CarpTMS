//! Circuit Breaker module
//! Provides circuit breaking capabilities to prevent cascading failures

use std::sync::Arc;
use tokio::sync::RwLock;
use log::{info, warn, debug};
use std::time::{Duration, Instant};

/// Circuit breaker state
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CircuitState {
    Closed,      // Normal operation
    Open,        // Tripped, no requests allowed
    HalfOpen,    // Testing if service is recovered
}

/// Circuit breaker configuration
#[derive(Clone, Debug)]
pub struct CircuitBreakerConfig {
    pub failure_threshold: u32,    // Number of failures before tripping
    pub reset_timeout: Duration,    // Time to wait before entering HalfOpen state
    pub half_open_attempts: u32,    // Number of attempts in HalfOpen state
    pub window_size: Duration,      // Time window for failure counting
}

/// Circuit breaker statistics
#[derive(Clone, Debug)]
pub struct CircuitStats {
    pub state: CircuitState,
    pub failure_count: u32,
    pub success_count: u32,
    pub last_failure: Option<Instant>,
    pub last_success: Option<Instant>,
    pub last_state_change: Instant,
    pub requests: u32,
    pub failures: u32,
    pub successes: u32,
}

/// Circuit breaker
pub struct CircuitBreaker {
    config: CircuitBreakerConfig,
    state: Arc<RwLock<CircuitState>>,
    stats: Arc<RwLock<CircuitStats>>,
    failure_window: Arc<RwLock<Vec<Instant>>>,
}

impl CircuitBreaker {
    /// Create a new circuit breaker
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            config,
            state: Arc::new(RwLock::new(CircuitState::Closed)),
            stats: Arc::new(RwLock::new(CircuitStats {
                state: CircuitState::Closed,
                failure_count: 0,
                success_count: 0,
                last_failure: None,
                last_success: None,
                last_state_change: Instant::now(),
                requests: 0,
                failures: 0,
                successes: 0,
            })),
            failure_window: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Check if the circuit is allowed to make a request
    pub async fn allow_request(&self) -> bool {
        let state = self.state.read().await;
        match *state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                // Check if we should transition to HalfOpen
                let stats = self.stats.read().await;
                if let Some(last_failure) = stats.last_failure {
                    if Instant::now().duration_since(last_failure) > self.config.reset_timeout {
                        drop(stats);
                        drop(state);
                        self.transition_to_half_open().await;
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            CircuitState::HalfOpen => true,
        }
    }

    /// Record a successful request
    pub async fn record_success(&self) {
        let mut state = self.state.write().await;
        let mut stats = self.stats.write().await;
        let mut failure_window = self.failure_window.write().await;

        // Clear failure window
        failure_window.clear();

        stats.success_count += 1;
        stats.last_success = Some(Instant::now());
        stats.requests += 1;
        stats.successes += 1;

        match *state {
            CircuitState::HalfOpen => {
                if stats.success_count >= self.config.half_open_attempts {
                    self.transition_to_closed(&mut state, &mut stats).await;
                }
            }
            _ => {
                stats.failure_count = 0;
            }
        }
    }

    /// Record a failed request
    pub async fn record_failure(&self) {
        let mut state = self.state.write().await;
        let mut stats = self.stats.write().await;
        let mut failure_window = self.failure_window.write().await;

        // Add failure to window
        let now = Instant::now();
        failure_window.push(now);

        // Remove failures outside the window
        failure_window.retain(|&time| now.duration_since(time) < self.config.window_size);

        stats.failure_count = failure_window.len() as u32;
        stats.last_failure = Some(now);
        stats.requests += 1;
        stats.failures += 1;

        match *state {
            CircuitState::Closed => {
                if stats.failure_count >= self.config.failure_threshold {
                    self.transition_to_open(&mut state, &mut stats).await;
                }
            }
            CircuitState::HalfOpen => {
                // Any failure in HalfOpen state trips the circuit back to Open
                self.transition_to_open(&mut state, &mut stats).await;
            }
            _ => {}
        }
    }

    /// Transition to Open state
    async fn transition_to_open(&self, state: &mut CircuitState, stats: &mut CircuitStats) {
        *state = CircuitState::Open;
        stats.state = CircuitState::Open;
        stats.last_state_change = Instant::now();
        stats.success_count = 0;
        warn!("Circuit breaker tripped to Open state");
    }

    /// Transition to Closed state
    async fn transition_to_closed(&self, state: &mut CircuitState, stats: &mut CircuitStats) {
        *state = CircuitState::Closed;
        stats.state = CircuitState::Closed;
        stats.last_state_change = Instant::now();
        stats.failure_count = 0;
        stats.success_count = 0;
        info!("Circuit breaker reset to Closed state");
    }

    /// Transition to HalfOpen state
    async fn transition_to_half_open(&self) {
        let mut state = self.state.write().await;
        let mut stats = self.stats.write().await;

        *state = CircuitState::HalfOpen;
        stats.state = CircuitState::HalfOpen;
        stats.last_state_change = Instant::now();
        stats.success_count = 0;
        info!("Circuit breaker transitioned to HalfOpen state");
    }

    /// Get current state
    pub async fn get_state(&self) -> CircuitState {
        *self.state.read().await
    }

    /// Get statistics
    pub async fn get_stats(&self) -> CircuitStats {
        self.stats.read().await.clone()
    }

    /// Reset the circuit breaker
    pub async fn reset(&self) {
        let mut state = self.state.write().await;
        let mut stats = self.stats.write().await;
        let mut failure_window = self.failure_window.write().await;

        *state = CircuitState::Closed;
        stats.state = CircuitState::Closed;
        stats.failure_count = 0;
        stats.success_count = 0;
        stats.last_failure = None;
        stats.last_success = None;
        stats.last_state_change = Instant::now();
        failure_window.clear();

        info!("Circuit breaker reset");
    }
}

impl Default for CircuitBreaker {
    fn default() -> Self {
        Self::new(CircuitBreakerConfig {
            failure_threshold: 5,
            reset_timeout: Duration::from_secs(30),
            half_open_attempts: 3,
            window_size: Duration::from_secs(60),
        })
    }
}
