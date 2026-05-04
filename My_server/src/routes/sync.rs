use actix_web::{web, HttpResponse};
use log::info;
use std::sync::Arc;

use crate::domain::use_cases::sync::SyncUseCases;
use crate::errors::{success_response, AppResult};
use crate::schemas::SyncRequest;

// 数据同步路由

/// 执行数据同步
#[utoipa::path(
    post, path = "/sync/execute",
    request_body = SyncRequest,
    responses(
        (status = 200, description = "Sync executed successfully", body = ApiResponse<SyncResponse>),
        (status = 500, description = "Internal server error", body = ApiResponse<()>)
    )
)]
pub async fn execute_sync(
    sync_use_cases: web::Data<Arc<SyncUseCases>>,
    sync_request: web::Json<SyncRequest>,
) -> AppResult<HttpResponse> {
    info!("Executing data sync: {:?}", sync_request);

    // 执行同步操作
    let result = sync_use_cases
        .execute_sync(sync_request.into_inner())
        .await?;

    Ok(success_response(Some(result)))
}

/// 获取同步状态
#[utoipa::path(
    get, path = "/sync/status/{sync_id}",
    responses(
        (status = 200, description = "Sync status fetched successfully", body = ApiResponse<SyncStatus>),
        (status = 404, description = "Sync not found", body = ApiResponse<()>),
        (status = 500, description = "Internal server error", body = ApiResponse<()>)
    )
)]
pub async fn get_sync_status(
    sync_use_cases: web::Data<Arc<SyncUseCases>>,
    sync_id: web::Path<String>,
) -> AppResult<HttpResponse> {
    info!("Fetching sync status for: {}", sync_id);

    let status = sync_use_cases.get_sync_status(&sync_id).await?;

    Ok(success_response(Some(status)))
}

/// 获取同步历史
#[utoipa::path(
    get, path = "/sync/history",
    responses(
        (status = 200, description = "Sync history fetched successfully", body = ApiResponse<Vec<SyncStatus>>),
        (status = 500, description = "Internal server error", body = ApiResponse<()>)
    )
)]
pub async fn get_sync_history(
    sync_use_cases: web::Data<Arc<SyncUseCases>>,
) -> AppResult<HttpResponse> {
    info!("Fetching sync history");

    let history = sync_use_cases.get_sync_history().await?;

    Ok(success_response(Some(history)))
}

/// 取消同步任务
#[utoipa::path(
    post, path = "/sync/cancel/{sync_id}",
    responses(
        (status = 200, description = "Sync cancelled successfully", body = ApiResponse<SyncStatus>),
        (status = 404, description = "Sync not found", body = ApiResponse<()>),
        (status = 500, description = "Internal server error", body = ApiResponse<()>)
    )
)]
pub async fn cancel_sync(
    sync_use_cases: web::Data<Arc<SyncUseCases>>,
    sync_id: web::Path<String>,
) -> AppResult<HttpResponse> {
    info!("Cancelling sync: {}", sync_id);

    let status = sync_use_cases.cancel_sync(&sync_id).await?;

    Ok(success_response(Some(status)))
}

/// 清理同步历史
#[utoipa::path(
    delete, path = "/sync/history",
    responses(
        (status = 200, description = "Sync history cleaned successfully", body = ApiResponse<() >),
        (status = 500, description = "Internal server error", body = ApiResponse<()>)
    )
)]
pub async fn clean_sync_history(
    sync_use_cases: web::Data<Arc<SyncUseCases>>,
) -> AppResult<HttpResponse> {
    info!("Cleaning sync history");

    sync_use_cases.clean_sync_history().await?;

    Ok(success_response(None::<()>))
}
