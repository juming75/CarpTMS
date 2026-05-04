//! 错误处理模块单元测试

#[cfg(test)]
mod tests {
    use std::time::Duration;

    // ============= CircuitBreakerState 测试 =============

    #[test]
    fn test_circuit_breaker_state_display() {
        use crate::errors::circuit_breaker::CircuitBreakerState;

        assert_eq!(format!("{}", CircuitBreakerState::Closed), "closed");
        assert_eq!(format!("{}", CircuitBreakerState::Open), "open");
        assert_eq!(format!("{}", CircuitBreakerState::HalfOpen), "half_open");
    }

    // ============= CircuitBreaker 测试 =============

    #[test]
    fn test_circuit_breaker_default_config() {
        use crate::errors::circuit_breaker::CircuitBreakerConfig;

        let config = CircuitBreakerConfig::default();
        assert_eq!(config.failure_threshold, 5);
        assert_eq!(config.success_threshold, 2);
    }

    #[test]
    fn test_circuit_breaker_new() {
        use crate::errors::circuit_breaker::CircuitBreaker;

        let cb = CircuitBreaker::new("test_service");
        assert_eq!(cb.service_name(), "test_service");
    }

    #[test]
    fn test_circuit_breaker_new_with_config() {
        use crate::errors::circuit_breaker::{CircuitBreaker, CircuitBreakerConfig};

        let config = CircuitBreakerConfig {
            failure_threshold: 3,
            success_threshold: 1,
            timeout: Duration::from_secs(30),
        };

        let cb = CircuitBreaker::new_with_config("test_service", config);
        assert_eq!(cb.service_name(), "test_service");
    }

    #[test]
    fn test_circuit_breaker_allow_request_closed() {
        use crate::errors::circuit_breaker::CircuitBreaker;

        let cb = CircuitBreaker::new("test");
        assert!(cb.allow_request());
    }

    // ============= RetryPolicy 测试 =============

    #[test]
    fn test_retry_policy_default() {
        use crate::errors::retry::RetryPolicy;

        let policy = RetryPolicy::default();
        assert_eq!(policy.max_attempts, 3);
    }

    #[test]
    fn test_retry_policy_fixed_delay() {
        use crate::errors::retry::RetryPolicy;

        let policy = RetryPolicy::fixed_delay(5, Duration::from_secs(1));
        assert_eq!(policy.max_attempts, 5);
    }

    #[test]
    fn test_retry_policy_exponential_backoff() {
        use crate::errors::retry::RetryPolicy;

        let policy = RetryPolicy::exponential_backoff(3, Duration::from_millis(100), 2.0);
        assert_eq!(policy.max_attempts, 3);
    }

    #[test]
    fn test_retry_policy_with_jitter() {
        use crate::errors::retry::RetryPolicy;

        let policy = RetryPolicy::with_jitter(3, Duration::from_secs(1));
        assert_eq!(policy.max_attempts, 3);
    }

    #[test]
    fn test_retry_policy_should_retry() {
        use crate::errors::retry::RetryPolicy;

        let policy = RetryPolicy::default();

        assert!(policy.should_retry(0)); // First attempt (0-indexed)
        assert!(policy.should_retry(1)); // Second attempt
        assert!(policy.should_retry(2)); // Third (last) attempt
        assert!(!policy.should_retry(3)); // Fourth attempt - too many
    }

    #[test]
    fn test_retry_policy_delay_calculation_fixed() {
        use crate::errors::retry::RetryPolicy;

        let policy = RetryPolicy::fixed_delay(3, Duration::from_secs(1));

        assert_eq!(policy.calculate_delay(0), Duration::from_secs(1));
        assert_eq!(policy.calculate_delay(1), Duration::from_secs(1));
        assert_eq!(policy.calculate_delay(2), Duration::from_secs(1));
    }

    #[test]
    fn test_retry_policy_delay_calculation_exponential() {
        use crate::errors::retry::RetryPolicy;

        let policy = RetryPolicy::exponential_backoff(3, Duration::from_secs(1), 2.0);

        assert_eq!(policy.calculate_delay(0), Duration::from_secs(1));   // 1 * 2^0
        assert_eq!(policy.calculate_delay(1), Duration::from_secs(2));   // 1 * 2^1
        assert_eq!(policy.calculate_delay(2), Duration::from_secs(4));   // 1 * 2^2
    }

    // ============= ErrorCode 测试 =============

    #[test]
    fn test_error_code_display() {
        use crate::errors::error_codes::ErrorCode;

        assert_eq!(format!("{}", ErrorCode::InternalError), "INTERNAL_ERROR");
        assert_eq!(format!("{}", ErrorCode::NotFound), "NOT_FOUND");
        assert_eq!(format!("{}", ErrorCode::ValidationError), "VALIDATION_ERROR");
    }

    #[test]
    fn test_error_code_is_client_error() {
        use crate::errors::error_codes::ErrorCode;

        assert!(ErrorCode::ValidationError.is_client_error());
        assert!(ErrorCode::NotFound.is_client_error());
        assert!(!ErrorCode::InternalError.is_client_error());
    }

    // ============= AppError 测试 =============

    #[test]
    fn test_app_error_creation() {
        use crate::errors::AppError;

        let error = AppError::new("TEST_ERROR", "Test error message");
        assert_eq!(error.code, "TEST_ERROR");
        assert_eq!(error.message, "Test error message");
    }

    #[test]
    fn test_app_error_internal() {
        use crate::errors::AppError;

        let error = AppError::internal("Internal error");
        assert_eq!(error.code, "INTERNAL_ERROR");
    }

    #[test]
    fn test_app_error_not_found() {
        use crate::errors::AppError;

        let error = AppError::not_found("User", "123");
        assert!(error.message.contains("User"));
        assert!(error.message.contains("123"));
    }

    #[test]
    fn test_app_error_validation() {
        use crate::errors::AppError;

        let error = AppError::validation("Invalid input");
        assert_eq!(error.code, "VALIDATION_ERROR");
    }

    #[test]
    fn test_app_error_to_json() {
        use crate::errors::AppError;

        let error = AppError::validation("Invalid input");
        let json = error.to_json();

        assert!(json.contains("VALIDATION_ERROR"));
        assert!(json.contains("Invalid input"));
    }
}
