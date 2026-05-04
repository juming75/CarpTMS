//! 错误处理模块单元测试

#[cfg(test)]
mod tests {
    use crate::errors::error_codes::{ErrorCode, ErrorSeverity};
    use crate::errors::AppError;
    use http::StatusCode;

    // ErrorCode 枚举测试
    mod error_code_tests {
        use super::*;

        #[test]
        fn test_error_code_validation() {
            let code = ErrorCode::ValidationError;
            assert_eq!(code.code(), 4000);
            assert_eq!(code.severity(), ErrorSeverity::Warning);
        }

        #[test]
        fn test_error_code_authentication() {
            let code = ErrorCode::AuthenticationFailed;
            assert_eq!(code.code(), 4010);
            assert_eq!(code.severity(), ErrorSeverity::Warning);
        }

        #[test]
        fn test_error_code_authorization() {
            let code = ErrorCode::AuthorizationFailed;
            assert_eq!(code.code(), 4030);
            assert_eq!(code.severity(), ErrorSeverity::Warning);
        }

        #[test]
        fn test_error_code_not_found() {
            let code = ErrorCode::ResourceNotFound;
            assert_eq!(code.code(), 4040);
            assert_eq!(code.severity(), ErrorSeverity::Info);
        }

        #[test]
        fn test_error_code_database() {
            let code = ErrorCode::DatabaseError;
            assert_eq!(code.code(), 5000);
            assert_eq!(code.severity(), ErrorSeverity::Error);
        }

        #[test]
        fn test_error_code_internal() {
            let code = ErrorCode::InternalError;
            assert_eq!(code.code(), 5001);
            assert_eq!(code.severity(), ErrorSeverity::Critical);
        }

        #[test]
        fn test_error_code_from_code() {
            let code = ErrorCode::from_code(4000);
            assert_eq!(code, ErrorCode::ValidationError);

            let code = ErrorCode::from_code(9999);
            assert_eq!(code, ErrorCode::Unknown);
        }

        #[test]
        fn test_error_severity_order() {
            assert!(ErrorSeverity::Critical > ErrorSeverity::Error);
            assert!(ErrorSeverity::Error > ErrorSeverity::Warning);
            assert!(ErrorSeverity::Warning > ErrorSeverity::Info);
        }
    }

    // AppError 结构体测试
    mod app_error_tests {
        use super::*;

        #[test]
        fn test_app_error_database_error() {
            let err = AppError::DatabaseError("Connection timeout".to_string());
            assert!(err.to_string().contains("Database"));
            assert_eq!(err.status_code(), StatusCode::INTERNAL_SERVER_ERROR);
        }

        #[test]
        fn test_app_error_validation_error() {
            let err = AppError::ValidationError("Invalid input".to_string());
            assert!(err.to_string().contains("Validation"));
            assert_eq!(err.status_code(), StatusCode::BAD_REQUEST);
        }

        #[test]
        fn test_app_error_unauthorized() {
            let err = AppError::Unauthorized("Token expired".to_string());
            assert!(err.to_string().contains("Unauthorized"));
            assert_eq!(err.status_code(), StatusCode::UNAUTHORIZED);
        }

        #[test]
        fn test_app_error_forbidden() {
            let err = AppError::Forbidden("Access denied".to_string());
            assert!(err.to_string().contains("Forbidden"));
            assert_eq!(err.status_code(), StatusCode::FORBIDDEN);
        }

        #[test]
        fn test_app_error_not_found() {
            let err = AppError::NotFound("Vehicle not found".to_string());
            assert!(err.to_string().contains("not found"));
            assert_eq!(err.status_code(), StatusCode::NOT_FOUND);
        }

        #[test]
        fn test_app_error_conflict() {
            let err = AppError::Conflict("Duplicate entry".to_string());
            assert!(err.to_string().contains("Conflict"));
            assert_eq!(err.status_code(), StatusCode::CONFLICT);
        }

        #[test]
        fn test_app_error_internal_error() {
            let err = AppError::InternalError("Something went wrong".to_string());
            assert!(err.to_string().contains("Internal"));
            assert_eq!(err.status_code(), StatusCode::INTERNAL_SERVER_ERROR);
        }

        #[test]
        fn test_app_error_service_unavailable() {
            let err = AppError::ServiceUnavailable("Service down".to_string());
            assert!(err.to_string().contains("unavailable"));
            assert_eq!(err.status_code(), StatusCode::SERVICE_UNAVAILABLE);
        }

        #[test]
        fn test_app_error_rate_limited() {
            let err = AppError::RateLimited;
            assert!(err.to_string().contains("Rate"));
            assert_eq!(err.status_code(), StatusCode::TOO_MANY_REQUESTS);
        }

        #[test]
        fn test_app_error_invalid_input() {
            let err = AppError::InvalidInput("Bad request".to_string());
            assert!(err.to_string().contains("Invalid"));
            assert_eq!(err.status_code(), StatusCode::BAD_REQUEST);
        }
    }

    // AppError 从其他错误类型转换测试
    mod app_error_conversion_tests {
        use super::*;

        #[test]
        fn test_app_error_from_sqlx_not_found() {
            let sqlx_err = sqlx::Error::RowNotFound;
            let app_err: AppError = sqlx_err.into();
            assert_eq!(app_err.status_code(), StatusCode::NOT_FOUND);
        }

        #[test]
        fn test_app_error_from_anyhow() {
            let anyhow_err = anyhow::anyhow!("Custom error");
            let app_err: AppError = anyhow_err.into();
            assert_eq!(app_err.status_code(), StatusCode::INTERNAL_SERVER_ERROR);
        }

        #[test]
        fn test_app_error_from_string() {
            let app_err: AppError = "Direct error string".to_string().into();
            assert_eq!(app_err.status_code(), StatusCode::INTERNAL_SERVER_ERROR);
        }

        #[test]
        fn test_app_error_from_str() {
            let app_err: AppError = "Direct error string".into();
            assert_eq!(app_err.status_code(), StatusCode::INTERNAL_SERVER_ERROR);
        }
    }

    // AppResult 类型测试
    mod app_result_tests {
        use crate::errors::AppResult;
        use super::*;

        #[test]
        fn test_app_result_ok() {
            let result: AppResult<String> = Ok("success".to_string());
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), "success");
        }

        #[test]
        fn test_app_result_err() {
            let result: AppResult<String> = Err(AppError::ValidationError("test".to_string()));
            assert!(result.is_err());
        }

        #[test]
        fn test_app_result_map() {
            let result: AppResult<i32> = Ok(5);
            let doubled = result.map(|x| x * 2);
            assert_eq!(doubled.unwrap(), 10);
        }

        #[test]
        fn test_app_result_and_then() {
            let result: AppResult<i32> = Ok(5);
            let processed = result.and_then(|x| Ok(x + 10));
            assert_eq!(processed.unwrap(), 15);
        }
    }

    // 错误消息格式化测试
    mod error_message_tests {
        use super::*;

        #[test]
        fn test_error_display() {
            let err = AppError::DatabaseError("Connection failed".to_string());
            let display = format!("{}", err);
            assert!(display.contains("Database error"));
            assert!(display.contains("Connection failed"));
        }

        #[test]
        fn test_error_debug() {
            let err = AppError::ValidationError("Invalid field".to_string());
            let debug = format!("{:?}", err);
            assert!(debug.contains("ValidationError"));
        }
    }
}
