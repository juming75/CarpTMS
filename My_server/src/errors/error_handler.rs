use actix_web::HttpResponse;
use serde::Serialize;

use crate::bff::ApiResponse;
use crate::errors::{AppError, ErrorCode, ErrorType};
use log;

// 错误日志记录
pub fn log_error(error: &AppError, context: &str) {
    match error.error_type {
        crate::errors::ErrorType::System
        | crate::errors::ErrorType::Database
        | crate::errors::ErrorType::Redis => {
            log::error!("{}: {} - Cause: {:?}", context, error, error.cause);
        }
        crate::errors::ErrorType::Network | crate::errors::ErrorType::ExternalService => {
            log::warn!("{}: {} - Cause: {:?}", context, error, error.cause);
        }
        crate::errors::ErrorType::Validation | crate::errors::ErrorType::Permission => {
            log::info!("{}: {} - Details: {:?}", context, error, error.details);
        }
        _ => {
            log::debug!("{}: {}", context, error);
        }
    }

    // 记录错误到监控系统
    // TODO: 实现错误监控
}

// 错误转换工具
pub trait ErrorExt {
    fn to_app_error(&self, error_type: ErrorType, message: &str) -> AppError;
}

impl<E: std::error::Error> ErrorExt for E {
    fn to_app_error(&self, error_type: ErrorType, message: &str) -> AppError {
        let error_type_clone = error_type.clone();
        AppError::new(
            error_type,
            match error_type_clone {
                ErrorType::System => ErrorCode::InternalError,
                ErrorType::Database => ErrorCode::DbConnectionError,
                ErrorType::Redis => ErrorCode::RedisConnectionError,
                ErrorType::Network => ErrorCode::NetworkConnectionError,
                ErrorType::Business => ErrorCode::BusinessLogicError,
                ErrorType::Permission => ErrorCode::Unauthorized,
                ErrorType::Validation => ErrorCode::InvalidParameters,
                ErrorType::ExternalService => ErrorCode::ExternalServiceError,
                ErrorType::Resource => ErrorCode::ResourceExhausted,
                ErrorType::Unknown => ErrorCode::UnknownError,
            },
            message,
            None,
            Some(&self.to_string()),
        )
    }
}

// 成功响应函数
pub fn success_response<T: Serialize>(data: T) -> HttpResponse {
    HttpResponse::Ok().json(ApiResponse::success(data))
}

// 带消息的成功响应函数
pub fn success_response_with_message<T: Serialize>(message: &str, data: T) -> HttpResponse {
    HttpResponse::Ok().json(ApiResponse {
        code: 200,
        message: message.to_string(),
        data,
    })
}

// 创建成功响应函数
pub fn created_response<T: Serialize>(data: T) -> HttpResponse {
    HttpResponse::Created().json(ApiResponse::success(data))
}

// 带消息的创建成功响应函数
pub fn created_response_with_message<T: Serialize>(message: &str, data: T) -> HttpResponse {
    HttpResponse::Created().json(ApiResponse {
        code: 201,
        message: message.to_string(),
        data,
    })
}

// 空成功响应函数
pub fn empty_success_response() -> HttpResponse {
    HttpResponse::Ok().json(ApiResponse::success(()))
}
