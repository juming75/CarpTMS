use std::fmt;

// 错误类型枚举
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorType {
    // 系统错误
    System,
    // 数据库错误
    Database,
    // Redis错误
    Redis,
    // 网络错误
    Network,
    // 业务逻辑错误
    Business,
    // 权限错误
    Permission,
    // 参数错误
    Validation,
    // 外部服务错误
    ExternalService,
    // 资源错误
    Resource,
    // 未知错误
    Unknown,
}

// 错误代码枚举
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorCode {
    // 系统错误 (1000-1999)
    InternalError = 1000,
    ServiceUnavailable = 1001,
    GatewayTimeout = 1002,
    TooManyRequests = 1003,

    // 数据库错误 (2000-2999)
    DbConnectionError = 2000,
    DbQueryError = 2001,
    DbTransactionError = 2002,
    DbConstraintError = 2003,
    DbTimeoutError = 2004,

    // Redis错误 (3000-3999)
    RedisConnectionError = 3000,
    RedisOperationError = 3001,
    RedisTimeoutError = 3002,

    // 网络错误 (4000-4999)
    NetworkConnectionError = 4000,
    NetworkTimeoutError = 4001,
    NetworkInvalidResponse = 4002,

    // 业务逻辑错误 (5000-5999)
    BusinessLogicError = 5000,
    ResourceNotFound = 5001,
    ResourceConflict = 5002,
    OperationFailed = 5003,

    // 权限错误 (6000-6999)
    Unauthorized = 6000,
    Forbidden = 6001,
    TokenExpired = 6002,
    TokenInvalid = 6003,

    // 参数错误 (7000-7999)
    InvalidParameters = 7000,
    MissingParameters = 7001,
    InvalidFormat = 7002,
    OutOfRange = 7003,

    // 外部服务错误 (8000-8999)
    ExternalServiceError = 8000,
    ExternalServiceTimeout = 8001,
    ExternalServiceUnavailable = 8002,

    // 资源错误 (9000-9999)
    ResourceExhausted = 9000,
    DiskFull = 9001,
    MemoryFull = 9002,

    // 未知错误 (9999)
    UnknownError = 9999,
}

// 错误信息结构体
#[derive(Debug, Clone)]
pub struct AppError {
    pub error_type: ErrorType,
    pub error_code: ErrorCode,
    pub message: String,
    pub details: Option<String>,
    pub cause: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl AppError {
    pub fn new(
        error_type: ErrorType,
        error_code: ErrorCode,
        message: &str,
        details: Option<&str>,
        cause: Option<&str>,
    ) -> Self {
        Self {
            error_type,
            error_code,
            message: message.to_string(),
            details: details.map(|s| s.to_string()),
            cause: cause.map(|s| s.to_string()),
            timestamp: chrono::Utc::now(),
        }
    }

    pub fn internal_error(message: &str, cause: Option<&str>) -> Self {
        Self::new(
            ErrorType::System,
            ErrorCode::InternalError,
            message,
            None,
            cause,
        )
    }

    pub fn db_error(message: &str, cause: Option<&str>) -> Self {
        Self::new(
            ErrorType::Database,
            ErrorCode::DbConnectionError,
            message,
            None,
            cause,
        )
    }

    pub fn redis_error(message: &str, cause: Option<&str>) -> Self {
        Self::new(
            ErrorType::Redis,
            ErrorCode::RedisConnectionError,
            message,
            None,
            cause,
        )
    }

    pub fn network_error(message: &str, cause: Option<&str>) -> Self {
        Self::new(
            ErrorType::Network,
            ErrorCode::NetworkConnectionError,
            message,
            None,
            cause,
        )
    }

    pub fn validation_error(message: &str, details: Option<&str>) -> Self {
        Self::new(
            ErrorType::Validation,
            ErrorCode::InvalidParameters,
            message,
            details,
            None,
        )
    }

    pub fn business_error(message: &str, details: Option<&str>) -> Self {
        Self::new(
            ErrorType::Business,
            ErrorCode::BusinessLogicError,
            message,
            details,
            None,
        )
    }

    pub fn permission_error(message: &str) -> Self {
        Self::new(
            ErrorType::Permission,
            ErrorCode::Unauthorized,
            message,
            None,
            None,
        )
    }

    pub fn forbidden_error(message: String) -> Self {
        Self::new(
            ErrorType::Permission,
            ErrorCode::Forbidden,
            &message,
            None,
            None,
        )
    }

    pub fn resource_not_found(message: &str) -> Self {
        Self::new(
            ErrorType::Business,
            ErrorCode::ResourceNotFound,
            message,
            None,
            None,
        )
    }

    pub fn external_service_error(message: &str, cause: Option<&str>) -> Self {
        Self::new(
            ErrorType::ExternalService,
            ErrorCode::ExternalServiceError,
            message,
            None,
            cause,
        )
    }

    pub fn service_unavailable_error(message: &str, cause: Option<&str>) -> Self {
        Self::new(
            ErrorType::System,
            ErrorCode::ServiceUnavailable,
            message,
            None,
            cause,
        )
    }

    // 无参数版本的 validation 方法
    pub fn validation(message: &str) -> Self {
        Self::validation_error(message, None)
    }

    // 无参数版本的 bad_request 方法
    pub fn bad_request(message: &str) -> Self {
        Self::validation_error(message, None)
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}] [{}] {}",
            self.error_type,
            self.error_code.clone() as u32,
            self.message
        )
    }
}

impl std::error::Error for AppError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        // cause 字段存储为 String，丢失了原始错误类型信息
        // TODO: 将 cause 类型从 Option<String> 改为 Option<Box<dyn Error + Send + Sync>>
        //       以支持完整的错误链追溯
        None
    }
}

// 实现 From trait 用于错误转换
impl From<sqlx::Error> for AppError {
    fn from(error: sqlx::Error) -> Self {
        AppError::db_error("Database error", Some(&error.to_string()))
    }
}

// 实现 From trait 用于 redis 错误转换
impl From<redis::RedisError> for AppError {
    fn from(error: redis::RedisError) -> Self {
        AppError::redis_error("Redis error", Some(&error.to_string()))
    }
}

// 实现 From trait 用于 reqwest 错误转换
impl From<reqwest::Error> for AppError {
    fn from(error: reqwest::Error) -> Self {
        AppError::network_error("Network error", Some(&error.to_string()))
    }
}

// 实现 From trait 用于 anyhow::Error 错误转换
impl From<anyhow::Error> for AppError {
    fn from(error: anyhow::Error) -> Self {
        AppError::internal_error("Internal error", Some(&error.to_string()))
    }
}

// 实现 From trait 用于 EncryptionError 错误转换
impl From<crate::utils::encryption::EncryptionError> for AppError {
    fn from(error: crate::utils::encryption::EncryptionError) -> Self {
        AppError::internal_error("Encryption error", Some(&error.to_string()))
    }
}

// 实现 From trait 用于 validator::ValidationErrors 错误转换
impl From<validator::ValidationErrors> for AppError {
    fn from(error: validator::ValidationErrors) -> Self {
        AppError::validation(&error.to_string())
    }
}

// 实现 From trait 用于 jsonwebtoken::errors::Error 错误转换
impl From<jsonwebtoken::errors::Error> for AppError {
    fn from(error: jsonwebtoken::errors::Error) -> Self {
        AppError::permission_error(&error.to_string())
    }
}

// 实现 From trait 用于 serde_json::Error 错误转换
impl From<serde_json::Error> for AppError {
    fn from(error: serde_json::Error) -> Self {
        AppError::internal_error("JSON serialization error", Some(&error.to_string()))
    }
}

// 实现 ResponseError trait 用于 Actix Web
impl actix_web::ResponseError for AppError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self.error_code {
            ErrorCode::Unauthorized => actix_web::http::StatusCode::UNAUTHORIZED,
            ErrorCode::Forbidden => actix_web::http::StatusCode::FORBIDDEN,
            ErrorCode::TokenExpired => actix_web::http::StatusCode::UNAUTHORIZED,
            ErrorCode::TokenInvalid => actix_web::http::StatusCode::UNAUTHORIZED,
            ErrorCode::InvalidParameters => actix_web::http::StatusCode::BAD_REQUEST,
            ErrorCode::MissingParameters => actix_web::http::StatusCode::BAD_REQUEST,
            ErrorCode::InvalidFormat => actix_web::http::StatusCode::BAD_REQUEST,
            ErrorCode::OutOfRange => actix_web::http::StatusCode::BAD_REQUEST,
            ErrorCode::ResourceNotFound => actix_web::http::StatusCode::NOT_FOUND,
            ErrorCode::ResourceConflict => actix_web::http::StatusCode::CONFLICT,
            ErrorCode::ServiceUnavailable => actix_web::http::StatusCode::SERVICE_UNAVAILABLE,
            ErrorCode::GatewayTimeout => actix_web::http::StatusCode::GATEWAY_TIMEOUT,
            ErrorCode::TooManyRequests => actix_web::http::StatusCode::TOO_MANY_REQUESTS,
            _ => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> actix_web::HttpResponse {
        actix_web::HttpResponse::build(self.status_code())
            .content_type("application/json")
            .json(serde_json::json!({
                "code": self.error_code.clone() as u32,
                "message": self.message,
                "data": serde_json::Value::Null,
                "details": self.details,
                "timestamp": self.timestamp,
            }))
    }
}

// 便捷方法:资源未找到错误
impl AppError {
    pub fn not_found_error(message: String) -> Self {
        Self::resource_not_found(&message)
    }
}

impl fmt::Display for ErrorType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorType::System => write!(f, "SYSTEM"),
            ErrorType::Database => write!(f, "DATABASE"),
            ErrorType::Redis => write!(f, "REDIS"),
            ErrorType::Network => write!(f, "NETWORK"),
            ErrorType::Business => write!(f, "BUSINESS"),
            ErrorType::Permission => write!(f, "PERMISSION"),
            ErrorType::Validation => write!(f, "VALIDATION"),
            ErrorType::ExternalService => write!(f, "EXTERNAL_SERVICE"),
            ErrorType::Resource => write!(f, "RESOURCE"),
            ErrorType::Unknown => write!(f, "UNKNOWN"),
        }
    }
}

// 结果类型定义
pub type AppResult<T> = Result<T, AppError>;
