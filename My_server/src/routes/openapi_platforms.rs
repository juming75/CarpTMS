use actix_web::{web, HttpResponse};
use chrono::Utc;
use log::{info, warn};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;

use crate::domain::entities::openapi_platform::{
    OpenapiPlatformCreateRequest, OpenapiPlatformQuery, OpenapiPlatformUpdateRequest,
};
use crate::domain::use_cases::openapi_platform::OpenapiPlatformUseCases;
use crate::errors::{
    created_response_with_message, empty_success_response, success_response,
    success_response_with_message, AppError, AppResult,
};
use crate::schemas::PagedResponse;
use crate::services::platform_integration::{PlatformIntegrationService, PullRequest, PushPayload};

// OpenAPI 平台响应体
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct OpenapiPlatformResponse {
    pub id: i32,
    pub name: String,
    pub url: String,
    pub api_key: String,
    pub status: String,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: Option<chrono::DateTime<Utc>>,
}

// 获取 OpenAPI 平台列表(支持分页和筛选)
#[utoipa::path(
    path = "/api/openapi/platforms",
    get,
    params(OpenapiPlatformQuery),
    responses(
        (status = 200, description = "OpenAPI platforms fetched successfully", body = ApiResponse<PagedResponse<OpenapiPlatformResponse>>),
        (status = 500, description = "Internal server error", body = ApiResponse<()>)
    )
)]
pub async fn get_openapi_platforms(
    openapi_platform_use_cases: web::Data<Arc<OpenapiPlatformUseCases>>,
    query: web::Query<OpenapiPlatformQuery>,
) -> AppResult<HttpResponse> {
    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20);

    // 构建查询参数
    let platform_query = OpenapiPlatformQuery {
        page: Some(page),
        page_size: Some(page_size),
        name: query.name.clone(),
        status: query.status.clone(),
    };

    // 调用用例获取 OpenAPI 平台列表
    let (platforms, total) = openapi_platform_use_cases.get_all(&platform_query).await?;

    // 转换为响应格式
    let platform_responses: Vec<OpenapiPlatformResponse> = platforms
        .into_iter()
        .map(|platform| OpenapiPlatformResponse {
            id: platform.id,
            name: platform.name,
            url: platform.url,
            api_key: platform.api_key,
            status: platform.status,
            created_at: platform.created_at,
            updated_at: platform.updated_at,
        })
        .collect();

    // 计算总页数
    let pages = if total % page_size as i64 == 0 {
        total / page_size as i64
    } else {
        total / page_size as i64 + 1
    };

    // 构建分页响应
    let paged_response = PagedResponse {
        list: platform_responses,
        total,
        page,
        page_size,
        pages: pages as i32,
    };

    Ok(success_response_with_message(
        "OpenAPI platforms fetched successfully",
        Some(paged_response),
    ))
}

// 创建 OpenAPI 平台
#[utoipa::path(
    path = "/api/openapi/platforms",
    post,
    request_body = OpenapiPlatformCreateRequest,
    responses(
        (status = 201, description = "OpenAPI platform created successfully", body = ApiResponse<OpenapiPlatformResponse>),
        (status = 400, description = "Invalid request parameters", body = ApiResponse<()>),
        (status = 500, description = "Internal server error", body = ApiResponse<()>)
    )
)]
pub async fn create_openapi_platform(
    openapi_platform_use_cases: web::Data<Arc<OpenapiPlatformUseCases>>,
    platform: web::Json<OpenapiPlatformCreateRequest>,
) -> AppResult<HttpResponse> {
    let platform_data = platform.into_inner();

    // 构建创建请求
    let create_request = OpenapiPlatformCreateRequest {
        name: platform_data.name,
        url: platform_data.url,
        api_key: platform_data.api_key,
        status: platform_data.status,
    };

    info!("Creating OpenAPI platform: {}", create_request.name);

    // 调用用例创建 OpenAPI 平台
    let created_platform = openapi_platform_use_cases.create(create_request).await?;

    // 转换为响应格式
    let response = OpenapiPlatformResponse {
        id: created_platform.id,
        name: created_platform.name,
        url: created_platform.url,
        api_key: created_platform.api_key,
        status: created_platform.status,
        created_at: created_platform.created_at,
        updated_at: created_platform.updated_at,
    };

    info!("OpenAPI platform created successfully: {}", response.name);
    Ok(created_response_with_message(
        "OpenAPI platform created successfully",
        Some(response),
    ))
}

// 获取 OpenAPI 平台详情
#[utoipa::path(
    path = "/api/openapi/platforms/{id}",
    get,
    responses(
        (status = 200, description = "OpenAPI platform fetched successfully", body = ApiResponse<OpenapiPlatformResponse>),
        (status = 404, description = "OpenAPI platform not found", body = ApiResponse<()>),
        (status = 500, description = "Internal server error", body = ApiResponse<()>)
    )
)]
pub async fn get_openapi_platform(
    openapi_platform_use_cases: web::Data<Arc<OpenapiPlatformUseCases>>,
    id: web::Path<i32>,
) -> AppResult<HttpResponse> {
    let id = id.into_inner();

    // 调用用例获取 OpenAPI 平台详情
    let platform = openapi_platform_use_cases.get_by_id(id).await?;

    match platform {
        Some(platform) => {
            let response = OpenapiPlatformResponse {
                id: platform.id,
                name: platform.name,
                url: platform.url,
                api_key: platform.api_key,
                status: platform.status,
                created_at: platform.created_at,
                updated_at: platform.updated_at,
            };

            Ok(success_response_with_message(
                "OpenAPI platform fetched successfully",
                Some(response),
            ))
        }
        None => {
            warn!("OpenAPI platform not found: {}", id);
            Err(AppError::not_found_error(
                "OpenAPI platform not found".to_string(),
            ))
        }
    }
}

// 更新 OpenAPI 平台
#[utoipa::path(
    path = "/api/openapi/platforms/{id}",
    put,
    request_body = OpenapiPlatformUpdateRequest,
    responses(
        (status = 200, description = "OpenAPI platform updated successfully", body = ApiResponse<OpenapiPlatformResponse>),
        (status = 400, description = "Invalid request parameters", body = ApiResponse<()>),
        (status = 404, description = "OpenAPI platform not found", body = ApiResponse<()>),
        (status = 500, description = "Internal server error", body = ApiResponse<()>)
    )
)]
pub async fn update_openapi_platform(
    openapi_platform_use_cases: web::Data<Arc<OpenapiPlatformUseCases>>,
    id: web::Path<i32>,
    platform: web::Json<OpenapiPlatformUpdateRequest>,
) -> AppResult<HttpResponse> {
    let id = id.into_inner();

    // 构建更新请求
    let update_request = OpenapiPlatformUpdateRequest {
        name: platform.name.clone(),
        url: platform.url.clone(),
        api_key: platform.api_key.clone(),
        status: platform.status.clone(),
    };

    // 调用用例更新 OpenAPI 平台
    match openapi_platform_use_cases.update(id, update_request).await {
        Ok(updated_platform) => {
            // 转换为响应格式
            let response = OpenapiPlatformResponse {
                id: updated_platform.id,
                name: updated_platform.name,
                url: updated_platform.url,
                api_key: updated_platform.api_key,
                status: updated_platform.status,
                created_at: updated_platform.created_at,
                updated_at: updated_platform.updated_at,
            };

            info!("OpenAPI platform updated successfully: {}", response.name);
            Ok(success_response_with_message(
                "OpenAPI platform updated successfully",
                Some(response),
            ))
        }
        Err(e) => {
            warn!("OpenAPI platform not found for update: {} - {:?}", id, e);
            Err(AppError::not_found_error(
                "OpenAPI platform not found".to_string(),
            ))
        }
    }
}

// 更新 OpenAPI 平台状态
#[utoipa::path(
    path = "/api/openapi/platforms/{id}/status",
    put,
    responses(
        (status = 200, description = "OpenAPI platform status updated successfully", body = ApiResponse<OpenapiPlatformResponse>),
        (status = 404, description = "OpenAPI platform not found", body = ApiResponse<()>),
        (status = 500, description = "Internal server error", body = ApiResponse<()>)
    )
)]
pub async fn update_openapi_platform_status(
    openapi_platform_use_cases: web::Data<Arc<OpenapiPlatformUseCases>>,
    id: web::Path<i32>,
    status: web::Json<serde_json::Value>,
) -> AppResult<HttpResponse> {
    let id = id.into_inner();
    let new_status = status
        .get("status")
        .and_then(|s| s.as_str())
        .unwrap_or("active");

    // 调用用例更新 OpenAPI 平台状态
    match openapi_platform_use_cases
        .update_status(id, new_status)
        .await
    {
        Ok(updated_platform) => {
            // 转换为响应格式
            let response = OpenapiPlatformResponse {
                id: updated_platform.id,
                name: updated_platform.name,
                url: updated_platform.url,
                api_key: updated_platform.api_key,
                status: updated_platform.status,
                created_at: updated_platform.created_at,
                updated_at: updated_platform.updated_at,
            };

            info!(
                "OpenAPI platform status updated successfully: {}",
                response.name
            );
            Ok(success_response_with_message(
                "OpenAPI platform status updated successfully",
                Some(response),
            ))
        }
        Err(e) => {
            warn!(
                "OpenAPI platform not found for status update: {} - {:?}",
                id, e
            );
            Err(AppError::not_found_error(
                "OpenAPI platform not found".to_string(),
            ))
        }
    }
}

// 删除 OpenAPI 平台
#[utoipa::path(
    path = "/api/openapi/platforms/{id}",
    delete,
    responses(
        (status = 200, description = "OpenAPI platform deleted successfully", body = ApiResponse<()>),
        (status = 404, description = "OpenAPI platform not found", body = ApiResponse<()>),
        (status = 500, description = "Internal server error", body = ApiResponse<()>)
    )
)]
pub async fn delete_openapi_platform(
    openapi_platform_use_cases: web::Data<Arc<OpenapiPlatformUseCases>>,
    id: web::Path<i32>,
) -> AppResult<HttpResponse> {
    let id = id.into_inner();

    // 调用用例删除 OpenAPI 平台
    match openapi_platform_use_cases.delete(id).await {
        Ok(true) => {
            info!("OpenAPI platform deleted successfully: {}", id);
            Ok(empty_success_response())
        }
        Ok(false) | Err(_) => {
            warn!("OpenAPI platform not found for deletion: {}", id);
            Err(AppError::not_found_error(
                "OpenAPI platform not found".to_string(),
            ))
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct PushRequest {
    pub event_type: String,
    pub data: serde_json::Value,
}

#[utoipa::path(
    path = "/api/openapi/platforms/{id}/push",
    post,
    request_body = PushRequest,
    responses(
        (status = 200, description = "Data pushed successfully"),
        (status = 404, description = "Platform not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn push_data_to_platform(
    pool: web::Data<sqlx::PgPool>,
    openapi_platform_use_cases: web::Data<Arc<OpenapiPlatformUseCases>>,
    id: web::Path<i32>,
    body: web::Json<PushRequest>,
) -> AppResult<HttpResponse> {
    let id = id.into_inner();
    let platform = openapi_platform_use_cases
        .get_by_id(id)
        .await?
        .ok_or_else(|| AppError::not_found_error("Platform not found".to_string()))?;

    if !platform.is_active() {
        return Err(AppError::business_error("Platform is not active", None));
    }

    let service = PlatformIntegrationService::new(pool.into_inner());
    let payload = PushPayload {
        event_type: body.event_type.clone(),
        data: body.data.clone(),
        timestamp: Utc::now(),
        source: "carptms".to_string(),
    };

    let result = service.push_to_platform(&platform, &payload).await;

    Ok(success_response(Some(serde_json::json!({
        "platform_id": result.platform_id,
        "platform_name": result.platform_name,
        "success": result.success,
        "status_code": result.status_code,
        "error": result.error,
    }))))
}

#[utoipa::path(
    path = "/api/openapi/platforms/push-all",
    post,
    request_body = PushRequest,
    responses(
        (status = 200, description = "Data pushed to all platforms"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn push_data_to_all_platforms(
    pool: web::Data<sqlx::PgPool>,
    body: web::Json<PushRequest>,
) -> AppResult<HttpResponse> {
    let service = PlatformIntegrationService::new(pool.into_inner());
    let payload = PushPayload {
        event_type: body.event_type.clone(),
        data: body.data.clone(),
        timestamp: Utc::now(),
        source: "carptms".to_string(),
    };

    let results = service.push_to_all_platforms(&payload).await;
    let success_count = results.iter().filter(|r| r.success).count();

    Ok(success_response(Some(serde_json::json!({
        "total": results.len(),
        "success": success_count,
        "failed": results.len() - success_count,
        "results": results,
    }))))
}

#[utoipa::path(
    path = "/api/openapi/platforms/pull",
    post,
    request_body = PullRequest,
    responses(
        (status = 200, description = "Data pulled successfully"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn pull_data_from_endpoint(
    pool: web::Data<sqlx::PgPool>,
    body: web::Json<PullRequest>,
) -> AppResult<HttpResponse> {
    let service = PlatformIntegrationService::new(pool.into_inner());
    let result = service.pull_from_endpoint(&body).await;

    Ok(success_response(Some(serde_json::json!({
        "success": result.success,
        "status_code": result.status_code,
        "data": result.data,
        "error": result.error,
    }))))
}

#[utoipa::path(
    path = "/api/openapi/platforms/{id}/test",
    post,
    responses(
        (status = 200, description = "Connection test result"),
        (status = 404, description = "Platform not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn test_platform_connection(
    pool: web::Data<sqlx::PgPool>,
    openapi_platform_use_cases: web::Data<Arc<OpenapiPlatformUseCases>>,
    id: web::Path<i32>,
) -> AppResult<HttpResponse> {
    let id = id.into_inner();
    let platform = openapi_platform_use_cases
        .get_by_id(id)
        .await?
        .ok_or_else(|| AppError::not_found_error("Platform not found".to_string()))?;

    let service = PlatformIntegrationService::new(pool.into_inner());
    let payload = PushPayload {
        event_type: "ping".to_string(),
        data: serde_json::json!({"test": true}),
        timestamp: Utc::now(),
        source: "carptms".to_string(),
    };

    let result = service.push_to_platform(&platform, &payload).await;

    Ok(success_response(Some(serde_json::json!({
        "platform_id": result.platform_id,
        "platform_name": result.platform_name,
        "connected": result.success,
        "status_code": result.status_code,
        "error": result.error,
    }))))
}

pub fn configure_openapi_platforms_routes(cfg: &mut web::ServiceConfig) {
    cfg.route("/openapi/platforms", web::get().to(get_openapi_platforms))
        .route(
            "/openapi/platforms",
            web::post().to(create_openapi_platform),
        )
        .route(
            "/openapi/platforms/push-all",
            web::post().to(push_data_to_all_platforms),
        )
        .route(
            "/openapi/platforms/pull",
            web::post().to(pull_data_from_endpoint),
        )
        .route(
            "/openapi/platforms/{id}",
            web::get().to(get_openapi_platform),
        )
        .route(
            "/openapi/platforms/{id}",
            web::put().to(update_openapi_platform),
        )
        .route(
            "/openapi/platforms/{id}",
            web::delete().to(delete_openapi_platform),
        )
        .route(
            "/openapi/platforms/{id}/status",
            web::put().to(update_openapi_platform_status),
        )
        .route(
            "/openapi/platforms/{id}/push",
            web::post().to(push_data_to_platform),
        )
        .route(
            "/openapi/platforms/{id}/test",
            web::post().to(test_platform_connection),
        );
}
