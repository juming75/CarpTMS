use actix_web::HttpResponse;
use serde::Serialize;
use serde_json::json;

use crate::bff::ApiResponse;

/// 成功响应函数
pub fn success_response<T: Serialize>(data: T) -> HttpResponse {
    HttpResponse::Ok().json(ApiResponse::success(data))
}

/// 带消息的成功响应函数
pub fn success_response_with_message<T: Serialize>(message: &str, data: T) -> HttpResponse {
    HttpResponse::Ok().json(ApiResponse {
        code: 200,
        message: message.to_string(),
        data,
    })
}

/// 创建成功响应函数
pub fn created_response<T: Serialize>(data: T) -> HttpResponse {
    HttpResponse::Created().json(ApiResponse::success(data))
}

/// 带消息的创建成功响应函数
pub fn created_response_with_message<T: Serialize>(message: &str, data: T) -> HttpResponse {
    HttpResponse::Created().json(ApiResponse {
        code: 201,
        message: message.to_string(),
        data,
    })
}

/// 无内容响应函数
pub fn no_content_response() -> HttpResponse {
    HttpResponse::NoContent().finish()
}

/// 分页响应函数
pub fn paginated_response<T: Serialize>(data: T, total: i64, page: i32, page_size: i32) -> HttpResponse {
    let total_pages = if page_size > 0 {
        (total + page_size as i64 - 1) / page_size as i64
    } else {
        0
    };

    let response = json! {
        {
            "items": data,
            "pagination": {
                "total": total,
                "page": page,
                "page_size": page_size,
                "total_pages": total_pages
            }
        }
    };

    success_response(response)
}
