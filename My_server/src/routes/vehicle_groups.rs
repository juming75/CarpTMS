use actix_web::{web, HttpResponse};
use log::info;
use serde::Deserialize;
use std::sync::Arc;
use utoipa::ToSchema;

use crate::domain::entities::vehicle_group::{
    VehicleGroupCreateRequest, VehicleGroupQuery, VehicleGroupUpdateRequest,
};
use crate::domain::use_cases::vehicle_group::VehicleGroupUseCases;
use crate::errors::{
    created_response_with_message, success_response_with_message, AppError, AppResult,
};
use crate::schemas::{PagedResponse, VehicleGroupCreate, VehicleGroupResponse, VehicleGroupUpdate};

// 车组查询参数
#[derive(Debug, Deserialize, ToSchema)]
pub struct VehicleGroupQueryParam {
    pub page: Option<i32>,
    pub page_size: Option<i32>,
    pub group_name: Option<String>,
}

// ============== 类型转换函数 (HTTP <-> Domain) ==============

/// 将 VehicleGroupCreate 转换为领域层的 VehicleGroupCreateRequest
fn schema_create_to_domain(create: VehicleGroupCreate) -> VehicleGroupCreateRequest {
    VehicleGroupCreateRequest {
        group_name: create.group_name,
        parent_id: create.parent_id,
        description: create.description,
    }
}

/// 将 VehicleGroupUpdate 转换为领域层的 VehicleGroupUpdateRequest
fn schema_update_to_domain(update: VehicleGroupUpdate) -> VehicleGroupUpdateRequest {
    VehicleGroupUpdateRequest {
        group_name: update.group_name,
        parent_id: update.parent_id,
        description: update.description,
    }
}

/// 将领域层 VehicleGroup 转换为 HTTP 响应 VehicleGroupResponse
fn domain_to_response(
    group: crate::domain::entities::vehicle_group::VehicleGroup,
) -> VehicleGroupResponse {
    VehicleGroupResponse {
        group_id: group.group_id,
        group_name: group.group_name,
        parent_id: group.parent_id,
        parent_name: group.parent_name,
        description: group.description,
        vehicle_count: group.vehicle_count as i32,
        create_time: group.create_time,
        update_time: group.update_time,
    }
}

// ============== 路由处理器 (仅 HTTP 适配) ==============

// 获取车组列表(支持分页和筛选)
#[utoipa::path(
    path = "/api/vehicle-groups",
    get,
    responses(
        (status = 200, description = "Vehicle groups fetched successfully", body = ApiResponse<PagedResponse<VehicleGroupResponse>>),
        (status = 500, description = "Internal server error", body = ApiResponse<()>)
    )
)]
pub async fn get_vehicle_groups(
    use_cases: web::Data<Arc<VehicleGroupUseCases>>,
    query: web::Query<VehicleGroupQueryParam>,
) -> AppResult<HttpResponse> {
    info!("HTTP: GET /vehicle-groups");

    // 1. 提取 HTTP 参数
    let query_params = query.into_inner();

    // 2. 构建领域层查询参数
    let group_query = VehicleGroupQuery {
        page: query_params.page,
        page_size: query_params.page_size,
        group_name: query_params.group_name,
    };

    // 3. 调用 use_cases 执行业务逻辑
    let (groups, total_count) = use_cases.get_vehicle_groups(group_query).await?;

    // 4. 转换为 HTTP 响应
    let group_responses: Vec<VehicleGroupResponse> =
        groups.into_iter().map(domain_to_response).collect();

    // 5. 构建分页响应
    let page = query_params.page.unwrap_or(1);
    let page_size = query_params.page_size.unwrap_or(20);
    let pages = ((total_count + page_size as i64 - 1) / page_size as i64) as i32;

    Ok(success_response_with_message(
        "Vehicle groups fetched successfully",
        PagedResponse {
            list: group_responses,
            total: total_count,
            page,
            page_size,
            pages,
        },
    ))
}

// 创建车组
#[utoipa::path(
    path = "/api/vehicle-groups",
    post,
    request_body = VehicleGroupCreate,
    responses(
        (status = 201, description = "Vehicle group created successfully", body = ApiResponse<VehicleGroupResponse>),
        (status = 400, description = "Invalid request parameters", body = ApiResponse<()>),
        (status = 500, description = "Internal server error", body = ApiResponse<()>)
    )
)]
pub async fn create_vehicle_group(
    use_cases: web::Data<Arc<VehicleGroupUseCases>>,
    group: web::Json<VehicleGroupCreate>,
) -> AppResult<HttpResponse> {
    info!("HTTP: POST /vehicle-groups");

    // 1. 反序列化为领域对象
    let create_data = schema_create_to_domain(group.into_inner());

    // 2. 调用 use_cases 执行业务逻辑
    let created_group = use_cases.create_vehicle_group(create_data).await?;

    // 3. 转换为 HTTP 响应
    let response = domain_to_response(created_group);

    info!(
        "Vehicle group created successfully: id={}, name={}",
        response.group_id, response.group_name
    );
    Ok(created_response_with_message(
        "Vehicle group created successfully",
        response,
    ))
}

// 获取车组详情
#[utoipa::path(
    path = "/api/vehicle-groups/{group_id}",
    get,
    responses(
        (status = 200, description = "Vehicle group fetched successfully", body = ApiResponse<VehicleGroupResponse>),
        (status = 404, description = "Vehicle group not found", body = ApiResponse<()>),
        (status = 500, description = "Internal server error", body = ApiResponse<()>)
    )
)]
pub async fn get_vehicle_group(
    use_cases: web::Data<Arc<VehicleGroupUseCases>>,
    group_id: web::Path<i32>,
) -> AppResult<HttpResponse> {
    info!("HTTP: GET /vehicle-groups/{}", *group_id);

    // 1. 调用 use_cases 执行业务逻辑
    let group = use_cases.get_vehicle_group(*group_id).await?;

    // 2. 处理结果
    match group {
        Some(g) => {
            // 3. 转换为 HTTP 响应
            let response = domain_to_response(g);
            Ok(success_response_with_message(
                "Vehicle group fetched successfully",
                response,
            ))
        }
        None => Err(AppError::not_found_error(
            "Vehicle group not found".to_string(),
        )),
    }
}

// 更新车组
#[utoipa::path(
    path = "/api/vehicle-groups/{group_id}",
    put,
    request_body = VehicleGroupUpdate,
    responses(
        (status = 200, description = "Vehicle group updated successfully", body = ApiResponse<VehicleGroupResponse>),
        (status = 400, description = "Invalid request parameters", body = ApiResponse<()>),
        (status = 404, description = "Vehicle group not found", body = ApiResponse<()>),
        (status = 500, description = "Internal server error", body = ApiResponse<()>)
    )
)]
pub async fn update_vehicle_group(
    use_cases: web::Data<Arc<VehicleGroupUseCases>>,
    group_id: web::Path<i32>,
    group: web::Json<VehicleGroupUpdate>,
) -> AppResult<HttpResponse> {
    info!("HTTP: PUT /vehicle-groups/{}", *group_id);

    // 1. 反序列化为领域对象
    let update_data = schema_update_to_domain(group.into_inner());

    // 2. 调用 use_cases 执行业务逻辑
    let updated_group = use_cases
        .update_vehicle_group(*group_id, update_data)
        .await?;

    // 3. 处理结果
    match updated_group {
        Some(g) => {
            // 4. 转换为 HTTP 响应
            let response = domain_to_response(g);
            Ok(success_response_with_message(
                "Vehicle group updated successfully",
                response,
            ))
        }
        None => Err(AppError::not_found_error(
            "Vehicle group not found".to_string(),
        )),
    }
}

// 删除车组
#[utoipa::path(
    path = "/api/vehicle-groups/{group_id}",
    delete,
    responses(
        (status = 200, description = "Vehicle group deleted successfully", body = ApiResponse<()>),
        (status = 400, description = "Cannot delete vehicle group with vehicles or child groups", body = ApiResponse<()>),
        (status = 404, description = "Vehicle group not found", body = ApiResponse<()>),
        (status = 500, description = "Internal server error", body = ApiResponse<()>)
    )
)]
pub async fn delete_vehicle_group(
    use_cases: web::Data<Arc<VehicleGroupUseCases>>,
    group_id: web::Path<i32>,
) -> AppResult<HttpResponse> {
    info!("HTTP: DELETE /vehicle-groups/{}", *group_id);

    // 1. 调用 use_cases 执行业务逻辑
    let result = use_cases.delete_vehicle_group(*group_id).await?;

    // 2. 处理结果
    if result {
        info!("Vehicle group deleted successfully: id={}", *group_id);
        Ok(success_response_with_message(
            "Vehicle group deleted successfully",
            (),
        ))
    } else {
        Err(AppError::not_found_error(
            "Vehicle group not found".to_string(),
        ))
    }
}

// 配置车组路由
pub fn configure_vehicle_group_routes(cfg: &mut web::ServiceConfig) {
    cfg.route("/vehicle-groups", web::get().to(get_vehicle_groups))
        .route("/vehicle-groups", web::post().to(create_vehicle_group))
        .route(
            "/vehicle-groups/{group_id}",
            web::get().to(get_vehicle_group),
        )
        .route(
            "/vehicle-groups/{group_id}",
            web::put().to(update_vehicle_group),
        )
        .route(
            "/vehicle-groups/{group_id}",
            web::delete().to(delete_vehicle_group),
        )
        .route(
            "/vehicle-groups/tree",
            web::get().to(get_vehicle_group_tree),
        );
}

// 获取车组树结构
#[utoipa::path(
    path = "/api/vehicle-groups/tree",
    get,
    responses(
        (status = 200, description = "Vehicle group tree fetched successfully", body = ApiResponse<Vec<serde_json::Value>>),
        (status = 500, description = "Internal server error", body = ApiResponse<()>)
    )
)]
pub async fn get_vehicle_group_tree(
    use_cases: web::Data<Arc<VehicleGroupUseCases>>,
) -> AppResult<HttpResponse> {
    info!("HTTP: GET /vehicle-groups/tree");

    // 1. 调用 use_cases 执行业务逻辑
    let tree = use_cases.get_vehicle_group_tree().await?;

    // 2. 直接返回结果
    Ok(success_response_with_message(
        "Vehicle groups tree fetched successfully",
        Some(tree),
    ))
}
