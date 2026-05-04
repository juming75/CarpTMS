//! / 通用宏 - 减少重复代码

/// 简化数据库查询错误处理的宏
#[macro_export]
macro_rules! db_result {
    ($expr:expr) => {
        $expr.map_err(|e| {
            tracing::error!("数据库错误: {}", e);
            crate::errors::AppError::DatabaseError(e.to_string())
        })
    };
}

/// 简化数据库查询错误处理的宏（带自定义消息）
#[macro_export]
macro_rules! db_result_with {
    ($expr:expr, $msg:expr) => {
        $expr.map_err(|e| {
            tracing::error!("{}: {}", $msg, e);
            crate::errors::AppError::db_error(&format!("{}: {}", $msg, e), None)
        })
    };
}

/// 简化内部错误的宏
#[macro_export]
macro_rules! internal_error {
    ($expr:expr) => {
        $expr.map_err(|e| crate::errors::AppError::internal_error(&e.to_string(), None))
    };
    ($expr:expr, $msg:expr) => {
        $expr.map_err(|e| crate::errors::AppError::internal_error(&format!("{}: {}", $msg, e), None))
    };
}

/// 简化验证的宏
#[macro_export]
macro_rules! validate {
    ($condition:expr, $message:expr) => {
        if !$condition {
            return Err(crate::errors::AppError::ValidationError(
                $message.to_string()
            ));
        }
    };
}

/// 简化分页参数解析的宏
#[macro_export]
macro_rules! parse_pagination {
    ($query:expr) => {{
        let page = $query.get("page")
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(1);
        let page_size = $query.get("page_size")
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(20)
            .min(100); // 最大每页100条
        (page, page_size)
    }};
}

/// 简化分页参数解析的宏（返回结构体）
#[macro_export]
macro_rules! pagination {
    ($page:expr, $page_size:expr) => {{
        let page = if $page <= 0 { 1 } else { $page };
        let page_size = if $page_size <= 0 { 20 } else { $page_size.min(100) };
        let offset = (page - 1) * page_size;
        (page, page_size, offset)
    }};
}

/// 简化创建成功响应的宏
#[macro_export]
macro_rules! success {
    ($data:expr) => {
        crate::utils::common::response_helpers::success_response($data)
    };
}

/// 简化创建错误响应的宏
#[macro_export]
macro_rules! error {
    ($err:expr) => {
        crate::utils::common::response_helpers::error_response($err)
    };
}

/// 简化JSON响应的宏
#[macro_export]
macro_rules! json_response {
    ($status:expr, $body:expr) => {
        actix_web::HttpResponse::build($status).json($body)
    };
}

/// 简化日志记录的宏
#[macro_export]
macro_rules! log_request {
    ($method:expr, $path:expr) => {
        tracing::info!(method = %$method, path = %$path, "API请求");
    };
    ($method:expr, $path:expr, $user_id:expr) => {
        tracing::info!(
            method = %$method,
            path = %$path,
            user_id = %$user_id,
            "API请求"
        );
    };
}

/// 简化选项检查和转换的宏
#[macro_export]
macro_rules! option_or_error {
    ($option:expr, $error:expr) => {
        $option.ok_or_else(|| $error)?
    };
}

/// 简化字符串非空检查的宏
#[macro_export]
macro_rules! require_non_empty {
    ($value:expr, $field_name:expr) => {
        if $value.trim().is_empty() {
            return Err(crate::errors::AppError::ValidationError(
                format!("{}不能为空", $field_name)
            ));
        }
    };
}

/// 简化批量操作的宏
#[macro_export]
macro_rules! batch_operation {
    ($items:expr, $operation:expr) => {{
        let mut results = Vec::new();
        for item in $items {
            results.push($operation(item)?);
        }
        Ok(results)
    }};
}

/// 简化重试逻辑的宏
#[macro_export]
macro_rules! retry {
    ($max_attempts:expr, $operation:expr) => {{
        let mut last_error = None;
        for attempt in 1..=$max_attempts {
            match $operation {
                Ok(result) => return Ok(result),
                Err(e) => {
                    last_error = Some(e);
                    if attempt < $max_attempts {
                        tokio::time::sleep(std::time::Duration::from_millis(100 * attempt as u64)).await;
                    }
                }
            }
        }
        Err(last_error.expect("retry: expected last_error to be Some after exhausting all attempts"))
    }};
}

/// 简化计时器的宏
#[macro_export]
macro_rules! timed {
    ($name:expr, $block:expr) => {{
        let start = std::time::Instant::now();
        let result = $block;
        let duration = start.elapsed();
        tracing::debug!(name = %$name, duration_ms = duration.as_millis(), "操作耗时");
        result
    }};
}

/// 简化存在性检查的宏
#[macro_export]
macro_rules! exists_or_not_found {
    ($condition:expr, $entity:expr) => {{
        if !$condition {
            return Err(crate::errors::AppError::not_found_error(format!("{} not found", $entity)));
        }
    }};
}

/// 简化唯一性检查的宏
#[macro_export]
macro_rules! unique_or_conflict {
    ($condition:expr, $message:expr) => {{
        if $condition {
            return Err(crate::errors::AppError::business_error($message, None));
        }
    }};
}

/// 简化关联检查的宏（删除前检查）
#[macro_export]
macro_rules! check_no_associations {
    ($count:expr, $message:expr) => {{
        if $count > 0 {
            return Err(crate::errors::AppError::business_error($message, None));
        }
    }};
}
