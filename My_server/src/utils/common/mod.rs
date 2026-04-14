//! / 通用工具模块 - 消除代码重复

/// 数据库查询辅助工具
pub mod db_helpers {
    use crate::errors::{AppError, AppResult};

    /// 执行分页查询的通用函数
    pub async fn paginated_query<T, F, Fut>(
        page: u32,
        page_size: u32,
        query_fn: F,
    ) -> AppResult<(Vec<T>, u64)>
    where
        F: Fn(u64, u64) -> Fut,
        Fut: std::future::Future<Output = AppResult<Vec<T>>>,
    {
        let offset = (page - 1) * page_size;
        let limit = page_size;
        
        let items = query_fn(offset as u64, limit as u64).await?;
        let total = items.len() as u64; // 实际应该查询总数
        
        Ok((items, total))
    }

    /// 批量查询辅助函数
    pub async fn batch_query<T, F, Fut>(
        ids: Vec<String>,
        query_fn: F,
    ) -> AppResult<Vec<T>>
    where
        F: Fn(&str) -> Fut,
        Fut: std::future::Future<Output = AppResult<Option<T>>>,
    {
        let mut results = Vec::new();
        for id in ids {
            if let Some(result) = query_fn(&id).await? {
                results.push(result);
            }
        }
        Ok(results)
    }
}

/// HTTP响应辅助工具
pub mod response_helpers {
    use actix_web::{HttpResponse, Responder};
    use serde::Serialize;
    use crate::errors::{AppError, AppResult};

    /// 统一成功响应
    pub fn success_response<T: Serialize>(data: T) -> HttpResponse {
        HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "data": data
        }))
    }

    /// 统一分页响应
    pub fn paginated_response<T: Serialize>(
        data: Vec<T>,
        page: u32,
        page_size: u32,
        total: u64,
    ) -> HttpResponse {
        HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "data": data,
            "pagination": {
                "page": page,
                "page_size": page_size,
                "total": total,
                "total_pages": (total as f64 / page_size as f64).ceil() as u32
            }
        }))
    }

    /// 统一错误响应
    pub fn error_response(err: AppError) -> HttpResponse {
        let status = err.status_code();
        HttpResponse::build(status).json(serde_json::json!({
            "success": false,
            "error": err.to_string(),
            "error_type": std::any::type_name::<AppError>()
        }))
    }

    /// 处理AppResult到HttpResponse
    pub async fn handle_result<T: Serialize + Responder>(
        result: AppResult<T>,
    ) -> HttpResponse {
        match result {
            Ok(data) => success_response(data),
            Err(err) => error_response(err),
        }
    }
}

/// 数据验证辅助工具
pub mod validation_helpers {
    use crate::errors::AppError;

    /// 验证字符串非空
    pub fn validate_non_empty(value: &str, field_name: &str) -> Result<(), AppError> {
        if value.trim().is_empty() {
            return Err(AppError::ValidationError(
                format!("{}不能为空", field_name)
            ));
        }
        Ok(())
    }

    /// 验证字符串长度
    pub fn validate_length(
        value: &str,
        min: usize,
        max: usize,
        field_name: &str,
    ) -> Result<(), AppError> {
        let len = value.len();
        if len < min {
            return Err(AppError::ValidationError(
                format!("{}长度不能少于{}个字符", field_name, min)
            ));
        }
        if len > max {
            return Err(AppError::ValidationError(
                format!("{}长度不能超过{}个字符", field_name, max)
            ));
        }
        Ok(())
    }

    /// 验证数字范围
    pub fn validate_range<T>(
        value: T,
        min: T,
        max: T,
        field_name: &str,
    ) -> Result<(), AppError>
    where
        T: PartialOrd + std::fmt::Display,
    {
        if value < min {
            return Err(AppError::ValidationError(
                format!("{}不能小于{}", field_name, min)
            ));
        }
        if value > max {
            return Err(AppError::ValidationError(
                format!("{}不能大于{}", field_name, max)
            ));
        }
        Ok(())
    }

    /// 批量验证
    pub fn validate_all(validations: Vec<Result<(), AppError>>) -> Result<(), AppError> {
        for result in validations {
            result?;
        }
        Ok(())
    }
}

/// 日志辅助工具
pub mod log_helpers {
    use tracing::{info, warn, error, debug};

    /// 记录API请求日志
    pub fn log_request(method: &str, path: &str, user_id: Option<&str>) {
        if let Some(uid) = user_id {
            info!(method = %method, path = %path, user_id = %uid, "API请求");
        } else {
            info!(method = %method, path = %path, "API请求");
        }
    }

    /// 记录API响应日志
    pub fn log_response(method: &str, path: &str, status: u16, duration_ms: u64) {
        info!(
            method = %method,
            path = %path,
            status = status,
            duration_ms = duration_ms,
            "API响应"
        );
    }

    /// 记录错误日志
    pub fn log_error(error: &str, context: &str) {
        error!(error = %error, context = %context, "错误发生");
    }

    /// 记录警告日志
    pub fn log_warning(warning: &str, context: &str) {
        warn!(warning = %warning, context = %context, "警告");
    }
}

/// 日期时间辅助工具
pub mod datetime_helpers {
    use chrono::{DateTime, Utc, Duration};
    use crate::errors::AppError;

    /// 获取当前时间戳
    pub fn now_timestamp() -> i64 {
        Utc::now().timestamp()
    }

    /// 解析ISO 8601时间字符串
    pub fn parse_datetime(s: &str) -> Result<DateTime<Utc>, AppError> {
        DateTime::parse_from_rfc3339(s)
            .map(|dt| dt.with_timezone(&Utc))
            .map_err(|_| AppError::ValidationError("无效的日期时间格式".to_string()))
    }

    /// 格式化日期时间为ISO 8601
    pub fn format_datetime(dt: &DateTime<Utc>) -> String {
        dt.to_rfc3339()
    }

    /// 检查时间是否过期
    pub fn is_expired(dt: &DateTime<Utc>, expiry_hours: i64) -> bool {
        let now = Utc::now();
        let diff = now.signed_duration_since(*dt);
        diff.num_hours() > expiry_hours
    }

    /// 获取今天的开始时间
    pub fn start_of_today() -> DateTime<Utc> {
        Utc::now()
            .with_hour(0)
            .unwrap()
            .with_minute(0)
            .unwrap()
            .with_second(0)
            .unwrap()
            .with_nanosecond(0)
            .unwrap()
    }

    /// 获取今天的结束时间
    pub fn end_of_today() -> DateTime<Utc> {
        start_of_today() + Duration::days(1) - Duration::nanoseconds(1)
    }
}

pub use db_helpers::*;
pub use response_helpers::*;
pub use validation_helpers::*;
pub use log_helpers::*;
pub use datetime_helpers::*;






