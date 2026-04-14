//! / 统一错误处理测试

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_error_creation() {
        let err = AppError::DatabaseError("Connection failed".to_string());
        assert!(matches!(err, AppError::DatabaseError(_)));
    }

    #[test]
    fn test_validation_error_creation() {
        let err = AppError::ValidationError("Invalid input".to_string());
        assert!(matches!(err, AppError::ValidationError(_)));
    }

    #[test]
    fn test_error_to_status_code() {
        assert_eq!(
            AppError::DatabaseError("test".to_string()).status_code(),
            StatusCode::INTERNAL_SERVER_ERROR
        );
        assert_eq!(
            AppError::ValidationError("test".to_string()).status_code(),
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            AppError::Unauthorized("test".to_string()).status_code(),
            StatusCode::UNAUTHORIZED
        );
    }

    #[test]
    fn test_error_display() {
        let err = AppError::DatabaseError("Connection failed".to_string());
        let msg = format!("{}", err);
        assert!(msg.contains("Database error"));
    }

    #[test]
    fn test_error_from_sqlx() {
        let sqlx_err = sqlx::Error::RowNotFound;
        let app_err = AppError::from(sqlx_err);
        assert!(matches!(app_err, AppError::DatabaseError(_)));
    }

    #[test]
    fn test_appresult_type() {
        let result: AppResult<String> = Ok("success".to_string());
        assert!(result.is_ok());
        
        let result: AppResult<String> = Err(AppError::ValidationError("test".to_string()));
        assert!(result.is_err());
    }
}






