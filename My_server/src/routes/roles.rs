//! Roles routes - delegates to RoleApplicationService

use actix_web::{web, HttpResponse};
use std::sync::Arc;

use crate::application::services::role_service::RoleApplicationService;
use crate::errors::{success_response_with_message, AppError, AppResult};

// Re-export types for utoipa
pub use crate::application::services::role_service::{
    RoleCreateRequest, RoleResponse, RoleUpdateRequest,
};

#[utoipa::path(
    path = "/api/roles",
    get,
    responses(
        (status = 200, description = "Roles fetched successfully", body = ApiResponse<Vec<RoleResponse>>),
        (status = 500, description = "Internal server error", body = ApiResponse<()>)
    )
)]
pub async fn get_roles(service: web::Data<Arc<RoleApplicationService>>) -> AppResult<HttpResponse> {
    let roles = service.get_roles().await?;
    Ok(success_response_with_message(
        "Roles fetched successfully",
        Some(roles),
    ))
}

#[utoipa::path(
    path = "/api/roles",
    post,
    request_body = RoleCreateRequest,
    responses(
        (status = 201, description = "Role created successfully", body = ApiResponse<RoleResponse>),
        (status = 400, description = "Invalid request parameters", body = ApiResponse<()>),
        (status = 500, description = "Internal server error", body = ApiResponse<()>)
    )
)]
pub async fn create_role(
    service: web::Data<Arc<RoleApplicationService>>,
    request: web::Json<RoleCreateRequest>,
) -> AppResult<HttpResponse> {
    let role = service.create_role(request.into_inner()).await?;
    Ok(success_response_with_message("Role created successfully", Some(role)))
}

#[utoipa::path(
    path = "/api/roles/{role_id}",
    get,
    responses(
        (status = 200, description = "Role fetched successfully", body = ApiResponse<RoleResponse>),
        (status = 404, description = "Role not found", body = ApiResponse<()>),
        (status = 500, description = "Internal server error", body = ApiResponse<()>)
    )
)]
pub async fn get_role(
    service: web::Data<Arc<RoleApplicationService>>,
    role_id: web::Path<i32>,
) -> AppResult<HttpResponse> {
    let role = service.get_role(*role_id).await?;
    match role {
        Some(r) => Ok(success_response_with_message(
            "Role fetched successfully",
            Some(r),
        )),
        None => Err(AppError::not_found_error("Role not found".to_string())),
    }
}

#[utoipa::path(
    path = "/api/roles/{role_id}",
    put,
    request_body = RoleUpdateRequest,
    responses(
        (status = 200, description = "Role updated successfully", body = ApiResponse<RoleResponse>),
        (status = 400, description = "Invalid request parameters", body = ApiResponse<()>),
        (status = 404, description = "Role not found", body = ApiResponse<()>),
        (status = 500, description = "Internal server error", body = ApiResponse<()>)
    )
)]
pub async fn update_role(
    service: web::Data<Arc<RoleApplicationService>>,
    role_id: web::Path<i32>,
    request: web::Json<RoleUpdateRequest>,
) -> AppResult<HttpResponse> {
    let role = service.update_role(*role_id, request.into_inner()).await?;
    Ok(success_response_with_message("Role updated successfully", Some(role)))
}

#[utoipa::path(
    path = "/api/roles/{role_id}",
    delete,
    responses(
        (status = 200, description = "Role deleted successfully", body = ApiResponse<()>),
        (status = 404, description = "Role not found", body = ApiResponse<()>),
        (status = 400, description = "Cannot delete role with active users", body = ApiResponse<()>),
        (status = 500, description = "Internal server error", body = ApiResponse<()>)
    )
)]
pub async fn delete_role(
    service: web::Data<Arc<RoleApplicationService>>,
    role_id: web::Path<i32>,
) -> AppResult<HttpResponse> {
    service.delete_role(*role_id).await?;
    Ok(success_response_with_message("Role deleted successfully", ()))
}

pub fn configure_roles_routes(cfg: &mut web::ServiceConfig) {
    cfg.route("/roles", web::get().to(get_roles))
        .route("/roles", web::post().to(create_role))
        .route("/roles/{role_id}", web::get().to(get_role))
        .route("/roles/{role_id}", web::put().to(update_role))
        .route("/roles/{role_id}", web::delete().to(delete_role));
}
