use actix_web::{web, HttpResponse};
use chrono::{Utc};
use log::{info, warn};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;
use validator::Validate;

use crate::domain::entities::organization::{OrganizationCreateRequest, OrganizationUpdateRequest, OrganizationQuery};
use crate::domain::use_cases::organization::OrganizationUseCases;
use crate::errors::{created_response_with_message, empty_success_response, success_response_with_message, AppError, AppResult,};
use crate::schemas::PagedResponse;

// 组织单位请求体
#[derive(Debug, Serialize, Deserialize, ToSchema, Validate)]
pub struct OrganizationUnitCreate {
    #[validate(length(min = 1, max = 50))]
    pub unit_id: String,
    #[validate(length(min = 2, max = 50))]
    pub name: String,
    pub r#type: String,
    pub parent_id: Option<i32>,
    pub description: Option<String>,
    pub contact_person: Option<String>,
    pub contact_phone: Option<String>,
    pub status: Option<String>,
}

// 组织单位更新请求体
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct OrganizationUnitUpdate {
    pub name: Option<String>,
    pub r#type: Option<String>,
    pub parent_id: Option<i32>,
    pub description: Option<String>,
    pub contact_person: Option<String>,
    pub contact_phone: Option<String>,
}

// 组织单位查询参数
#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct OrganizationUnitQuery {
    pub page: Option<i32>,
    pub page_size: Option<i32>,
    pub name: Option<String>,
    pub r#type: Option<String>,
}

// 组织单位响应体
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct OrganizationUnitResponse {
    pub unit_id: String,
    pub name: String,
    pub r#type: String,
    pub parent_id: Option<i32>,
    pub description: Option<String>,
    pub contact_person: Option<String>,
    pub contact_phone: Option<String>,
    pub status: String,
    pub create_time: chrono::DateTime<Utc>,
    pub update_time: Option<chrono::DateTime<Utc>>,
}

// 获取组织单位列表(支持分页和筛选)
#[utoipa::path(
    path = "/api/organizations",
    get,
    params(OrganizationUnitQuery),
    responses(
        (status = 200, description = "Organization units fetched successfully", body = ApiResponse<PagedResponse<OrganizationUnitResponse>>),
        (status = 500, description = "Internal server error", body = ApiResponse<()>)
    )
)]
pub async fn get_organizations(
    organization_use_cases: web::Data<Arc<OrganizationUseCases>>,
    query: web::Query<OrganizationUnitQuery>,
) -> AppResult<HttpResponse> {
    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20);

    // 构建查询参数
    let org_query = OrganizationQuery {
        page: Some(page),
        page_size: Some(page_size),
        name: query.name.clone(),
        r#type: query.r#type.clone(),
    };

    // 调用用例获取组织单位列表
    let (organizations, total) = organization_use_cases.get_all(&org_query).await?;

    // 转换为响应格式
    let org_responses: Vec<OrganizationUnitResponse> = organizations
        .into_iter()
        .map(|org| OrganizationUnitResponse {
            unit_id: org.unit_id,
            name: org.name,
            r#type: org.r#type,
            parent_id: org.parent_id,
            description: org.description,
            contact_person: org.contact_person,
            contact_phone: org.contact_phone,
            status: org.status,
            create_time: org.create_time,
            update_time: org.update_time,
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
        list: org_responses,
        total,
        page,
        page_size,
        pages: pages as i32,
    };

    Ok(success_response_with_message(
        "Organization units fetched successfully",
        Some(paged_response),
    ))
}

// 创建组织单位
#[utoipa::path(
    path = "/api/organizations",
    post,
    request_body = OrganizationUnitCreate,
    responses(
        (status = 201, description = "Organization unit created successfully", body = ApiResponse<OrganizationUnitResponse>),
        (status = 400, description = "Invalid request parameters", body = ApiResponse<()>),
        (status = 500, description = "Internal server error", body = ApiResponse<()>)
    )
)]
pub async fn create_organization(
    organization_use_cases: web::Data<Arc<OrganizationUseCases>>,
    org: web::Json<OrganizationUnitCreate>,
) -> AppResult<HttpResponse> {
    let org_data = org.into_inner();

    // 构建创建请求
    let create_request = OrganizationCreateRequest {
        unit_id: org_data.unit_id,
        name: org_data.name,
        r#type: org_data.r#type,
        parent_id: org_data.parent_id,
        description: org_data.description,
        contact_person: org_data.contact_person,
        contact_phone: org_data.contact_phone,
        status: org_data.status,
    };

    info!("Creating organization: {}", create_request.name);

    // 调用用例创建组织单位
    let created_org = organization_use_cases.create(create_request).await?;

    // 转换为响应格式
    let response = OrganizationUnitResponse {
        unit_id: created_org.unit_id,
        name: created_org.name,
        r#type: created_org.r#type,
        parent_id: created_org.parent_id,
        description: created_org.description,
        contact_person: created_org.contact_person,
        contact_phone: created_org.contact_phone,
        status: created_org.status,
        create_time: created_org.create_time,
        update_time: created_org.update_time,
    };

    info!("Organization created successfully: {}", response.name);
    Ok(created_response_with_message(
        "Organization unit created successfully",
        Some(response),
    ))
}

// 获取组织单位详情
#[utoipa::path(
    path = "/api/organizations/{unit_id}",
    get,
    responses(
        (status = 200, description = "Organization unit fetched successfully", body = ApiResponse<OrganizationUnitResponse>),
        (status = 404, description = "Organization unit not found", body = ApiResponse<()>),
        (status = 500, description = "Internal server error", body = ApiResponse<()>)
    )
)]
pub async fn get_organization(
    organization_use_cases: web::Data<Arc<OrganizationUseCases>>,
    unit_id: web::Path<String>,
) -> AppResult<HttpResponse> {
    let unit_id = unit_id.into_inner();
    let unit_id_clone = unit_id.clone();

    // 调用用例获取组织单位详情
    let org = organization_use_cases.get_by_id(&unit_id).await?;

    match org {
        Some(organization) => {
            let response = OrganizationUnitResponse {
                unit_id: organization.unit_id,
                name: organization.name,
                r#type: organization.r#type,
                parent_id: organization.parent_id,
                description: organization.description,
                contact_person: organization.contact_person,
                contact_phone: organization.contact_phone,
                status: organization.status,
                create_time: organization.create_time,
                update_time: organization.update_time,
            };

            Ok(success_response_with_message(
                "Organization unit fetched successfully",
                Some(response),
            ))
        }
        None => {
            warn!("Organization unit not found: {}", unit_id_clone);
            Err(AppError::not_found_error(
                "Organization unit not found".to_string(),
            ))
        }
    }
}

// 更新组织单位
#[utoipa::path(
    path = "/api/organizations/{unit_id}",
    put,
    request_body = OrganizationUnitUpdate,
    responses(
        (status = 200, description = "Organization unit updated successfully", body = ApiResponse<OrganizationUnitResponse>),
        (status = 400, description = "Invalid request parameters", body = ApiResponse<()>),
        (status = 404, description = "Organization unit not found", body = ApiResponse<()>),
        (status = 500, description = "Internal server error", body = ApiResponse<()>)
    )
)]
pub async fn update_organization(
    organization_use_cases: web::Data<Arc<OrganizationUseCases>>,
    unit_id: web::Path<String>,
    org: web::Json<OrganizationUnitUpdate>,
) -> AppResult<HttpResponse> {
    let unit_id = unit_id.into_inner();
    let unit_id_clone = unit_id.clone();

    // 构建更新请求
    let update_request = OrganizationUpdateRequest {
        name: org.name.clone(),
        r#type: org.r#type.clone(),
        parent_id: org.parent_id,
        description: org.description.clone(),
        contact_person: org.contact_person.clone(),
        contact_phone: org.contact_phone.clone(),
    };

    // 调用用例更新组织单位
    match organization_use_cases.update(&unit_id, update_request).await {
        Ok(updated_org) => {
            // 转换为响应格式
            let response = OrganizationUnitResponse {
                unit_id: updated_org.unit_id,
                name: updated_org.name,
                r#type: updated_org.r#type,
                parent_id: updated_org.parent_id,
                description: updated_org.description,
                contact_person: updated_org.contact_person,
                contact_phone: updated_org.contact_phone,
                status: updated_org.status,
                create_time: updated_org.create_time,
                update_time: updated_org.update_time,
            };

            info!("Organization updated successfully: {}", response.name);
            Ok(success_response_with_message(
                "Organization unit updated successfully",
                Some(response),
            ))
        }
        Err(e) => {
            warn!("Organization unit not found for update: {} - {:?}", unit_id_clone, e);
            Err(AppError::not_found_error(
                "Organization unit not found".to_string(),
            ))
        }
    }
}

// 更新组织单位状态
#[utoipa::path(
    path = "/api/organizations/{unit_id}/status",
    put,
    responses(
        (status = 200, description = "Organization unit status updated successfully", body = ApiResponse<OrganizationUnitResponse>),
        (status = 404, description = "Organization unit not found", body = ApiResponse<()>),
        (status = 500, description = "Internal server error", body = ApiResponse<()>)
    )
)]
pub async fn update_organization_status(
    organization_use_cases: web::Data<Arc<OrganizationUseCases>>,
    unit_id: web::Path<String>,
    status: web::Json<serde_json::Value>,
) -> AppResult<HttpResponse> {
    let unit_id = unit_id.into_inner();
    let unit_id_clone = unit_id.clone();
    let new_status = status
        .get("status")
        .and_then(|s| s.as_str())
        .unwrap_or("active");

    // 调用用例更新组织单位状态
    match organization_use_cases.update_status(&unit_id, new_status).await {
        Ok(updated_org) => {
            // 转换为响应格式
            let response = OrganizationUnitResponse {
                unit_id: updated_org.unit_id,
                name: updated_org.name,
                r#type: updated_org.r#type,
                parent_id: updated_org.parent_id,
                description: updated_org.description,
                contact_person: updated_org.contact_person,
                contact_phone: updated_org.contact_phone,
                status: updated_org.status,
                create_time: updated_org.create_time,
                update_time: updated_org.update_time,
            };

            info!(
                "Organization status updated successfully: {}",
                response.name
            );
            Ok(success_response_with_message(
                "Organization unit status updated successfully",
                Some(response),
            ))
        }
        Err(e) => {
            warn!(
                "Organization unit not found for status update: {} - {:?}",
                unit_id_clone, e
            );
            Err(AppError::not_found_error(
                "Organization unit not found".to_string(),
            ))
        }
    }
}

// 删除组织单位
#[utoipa::path(
    path = "/api/organizations/{unit_id}",
    delete,
    responses(
        (status = 200, description = "Organization unit deleted successfully", body = ApiResponse<()>),
        (status = 404, description = "Organization unit not found", body = ApiResponse<()>),
        (status = 500, description = "Internal server error", body = ApiResponse<()>)
    )
)]
pub async fn delete_organization(
    organization_use_cases: web::Data<Arc<OrganizationUseCases>>,
    unit_id: web::Path<String>,
) -> AppResult<HttpResponse> {
    let unit_id = unit_id.into_inner();
    let unit_id_clone = unit_id.clone();

    // 调用用例删除组织单位
    match organization_use_cases.delete(&unit_id).await {
        Ok(true) => {
            info!("Organization deleted successfully: {}", unit_id_clone);
            Ok(empty_success_response())
        }
        Ok(false) | Err(_) => {
            warn!(
                "Organization unit not found for deletion: {}",
                unit_id_clone
            );
            Err(AppError::not_found_error(
                "Organization unit not found".to_string(),
            ))
        }
    }
}

// 配置组织单位路由
pub fn configure_organizations_routes(cfg: &mut web::ServiceConfig) {
    cfg.route("/organizations", web::get().to(get_organizations))
        .route("/organizations", web::post().to(create_organization))
        .route("/organizations/{unit_id}", web::get().to(get_organization))
        .route(
            "/organizations/{unit_id}",
            web::put().to(update_organization),
        )
        .route(
            "/organizations/{unit_id}",
            web::delete().to(delete_organization),
        )
        .route(
            "/organizations/{unit_id}/status",
            web::put().to(update_organization_status),
        );
}
