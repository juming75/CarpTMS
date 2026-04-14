use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;

use crate::domain::entities::department::{DepartmentCreateRequest, DepartmentUpdateRequest, DepartmentQuery};
use crate::domain::use_cases::department::DepartmentUseCases;
use crate::errors::{empty_success_response, success_response_with_message, AppError, AppResult};
use crate::schemas::PagedResponse;

// 部门响应体
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct DepartmentResponse {
    pub department_id: i32,
    pub department_name: String,
    pub parent_department_id: Option<i32>,
    pub parent_department_name: Option<String>,
    pub manager_id: Option<i32>,
    pub manager_name: Option<String>,
    pub phone: Option<String>,
    pub description: Option<String>,
    pub create_time: String,
    pub update_time: Option<String>,
}

// 获取部门列表
#[utoipa::path(
    path = "/api/departments",
    get,
    responses(
        (status = 200, description = "Departments fetched successfully"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_departments(
    department_use_cases: web::Data<Arc<DepartmentUseCases>>,
    query: web::Query<DepartmentQuery>,
) -> AppResult<HttpResponse> {
    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20);

    // 转换为领域查询对象
    let domain_query = DepartmentQuery {
        page: Some(page),
        page_size: Some(page_size),
    };

    // 调用用例获取部门列表
    match department_use_cases.get_departments(domain_query).await {
        Ok((departments, total)) => {
            // 转换为响应格式
            let department_responses: Vec<DepartmentResponse> = departments
                .into_iter()
                .map(|department| DepartmentResponse {
                    department_id: department.department_id,
                    department_name: department.department_name,
                    parent_department_id: department.parent_department_id,
                    parent_department_name: department.parent_department_name,
                    manager_id: department.manager_id,
                    manager_name: department.manager_name,
                    phone: department.phone,
                    description: department.description,
                    create_time: department.create_time.to_string(),
                    update_time: department.update_time.map(|t| t.to_string()),
                })
                .collect();

            // 计算总页数
            let pages = if total % page_size as i64 == 0 {
                total / page_size as i64
            } else {
                total / page_size as i64 + 1
            };

            let paged_response = PagedResponse {
                list: department_responses,
                total,
                page,
                page_size,
                pages: pages as i32,
            };

            Ok(success_response_with_message(
                "Departments fetched successfully",
                Some(paged_response),
            ))
        }
        Err(e) => {
            Err(AppError::internal_error(&format!("Failed to fetch departments: {:?}", e), None))
        }
    }
}

// 创建部门
#[utoipa::path(
    path = "/api/departments",
    post,
    request_body = DepartmentCreateRequest,
    responses(
        (status = 201, description = "Department created successfully"),
        (status = 400, description = "Invalid request parameters"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn create_department(
    department_use_cases: web::Data<Arc<DepartmentUseCases>>,
    request: web::Json<DepartmentCreateRequest>,
) -> AppResult<HttpResponse> {
    // 转换为领域创建请求
    let create_request = DepartmentCreateRequest {
        department_name: request.department_name.clone(),
        parent_department_id: request.parent_department_id,
        manager_id: request.manager_id,
        phone: request.phone.clone(),
        description: request.description.clone(),
    };

    // 调用用例创建部门
    match department_use_cases.create_department(create_request).await {
        Ok(created_department) => {
            // 转换为响应格式
            let department_response = DepartmentResponse {
                department_id: created_department.department_id,
                department_name: created_department.department_name,
                parent_department_id: created_department.parent_department_id,
                parent_department_name: created_department.parent_department_name,
                manager_id: created_department.manager_id,
                manager_name: created_department.manager_name,
                phone: created_department.phone,
                description: created_department.description,
                create_time: created_department.create_time.to_string(),
                update_time: created_department.update_time.map(|t| t.to_string()),
            };

            Ok(success_response_with_message(
                "Department created successfully",
                Some(department_response),
            ))
        }
        Err(e) => {
            if e.to_string().contains("Parent department not found") {
                Err(AppError::business_error("Parent department not found", None))
            } else if e.to_string().contains("Department name is required") {
                Err(AppError::validation("Department name is required"))
            } else {
                Err(AppError::internal_error(&format!("Failed to create department: {:?}", e), None))
            }
        }
    }
}

// 获取部门详情
#[utoipa::path(
    path = "/api/departments/{department_id}",
    get,
    responses(
        (status = 200, description = "Department fetched successfully"),
        (status = 404, description = "Department not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_department(
    department_use_cases: web::Data<Arc<DepartmentUseCases>>,
    department_id: web::Path<i32>,
) -> AppResult<HttpResponse> {
    let department_id = department_id.into_inner();

    // 调用用例获取部门详情
    match department_use_cases.get_department(department_id).await {
        Ok(department) => match department {
            Some(department) => {
                // 转换为响应格式
                let department_response = DepartmentResponse {
                    department_id: department.department_id,
                    department_name: department.department_name,
                    parent_department_id: department.parent_department_id,
                    parent_department_name: department.parent_department_name,
                    manager_id: department.manager_id,
                    manager_name: department.manager_name,
                    phone: department.phone,
                    description: department.description,
                    create_time: department.create_time.to_string(),
                    update_time: department.update_time.map(|t| t.to_string()),
                };

                Ok(success_response_with_message(
                    "Department fetched successfully",
                    Some(department_response),
                ))
            }
            None => Err(AppError::not_found_error(
                "Department not found".to_string(),
            )),
        },
        Err(e) => {
            Err(AppError::internal_error(&format!("Failed to fetch department: {:?}", e), None))
        }
    }
}

// 更新部门
#[utoipa::path(
    path = "/api/departments/{department_id}",
    put,
    request_body = DepartmentUpdateRequest,
    responses(
        (status = 200, description = "Department updated successfully"),
        (status = 400, description = "Invalid request parameters"),
        (status = 404, description = "Department not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn update_department(
    department_use_cases: web::Data<Arc<DepartmentUseCases>>,
    department_id: web::Path<i32>,
    request: web::Json<DepartmentUpdateRequest>,
) -> AppResult<HttpResponse> {
    let department_id = department_id.into_inner();

    // 转换为领域更新请求
    let update_request = DepartmentUpdateRequest {
        department_name: request.department_name.clone(),
        parent_department_id: request.parent_department_id,
        manager_id: request.manager_id,
        phone: request.phone.clone(),
        description: request.description.clone(),
    };

    // 调用用例更新部门
    match department_use_cases.update_department(department_id, update_request).await {
        Ok(updated_department) => match updated_department {
            Some(department) => {
                // 转换为响应格式
                let department_response = DepartmentResponse {
                    department_id: department.department_id,
                    department_name: department.department_name,
                    parent_department_id: department.parent_department_id,
                    parent_department_name: department.parent_department_name,
                    manager_id: department.manager_id,
                    manager_name: department.manager_name,
                    phone: department.phone,
                    description: department.description,
                    create_time: department.create_time.to_string(),
                    update_time: department.update_time.map(|t| t.to_string()),
                };

                Ok(success_response_with_message(
                    "Department updated successfully",
                    Some(department_response),
                ))
            }
            None => Err(AppError::not_found_error(
                "Department not found".to_string(),
            )),
        },
        Err(e) => {
            if e.to_string().contains("Department not found") {
                Err(AppError::not_found_error("Department not found".to_string()))
            } else if e.to_string().contains("Parent department not found") {
                Err(AppError::business_error("Parent department not found", None))
            } else if e.to_string().contains("Department cannot be its own parent") {
                Err(AppError::business_error("Department cannot be its own parent", None))
            } else if e.to_string().contains("Department name is required") {
                Err(AppError::validation("Department name is required"))
            } else {
                Err(AppError::internal_error(&format!("Failed to update department: {:?}", e), None))
            }
        }
    }
}

// 删除部门
#[utoipa::path(
    path = "/api/departments/{department_id}",
    delete,
    responses(
        (status = 200, description = "Department deleted successfully"),
        (status = 404, description = "Department not found"),
        (status = 400, description = "Cannot delete department with sub-departments"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn delete_department(
    department_use_cases: web::Data<Arc<DepartmentUseCases>>,
    department_id: web::Path<i32>,
) -> AppResult<HttpResponse> {
    let department_id = department_id.into_inner();

    // 调用用例删除部门
    match department_use_cases.delete_department(department_id).await {
        Ok(deleted) => {
            if deleted {
                Ok(empty_success_response())
            } else {
                Err(AppError::not_found_error("Department not found".to_string()))
            }
        }
        Err(e) => {
            if e.to_string().contains("Cannot delete department with sub-departments") {
                Err(AppError::business_error("Cannot delete department with sub-departments", None))
            } else {
                Err(AppError::internal_error(&format!("Failed to delete department: {:?}", e), None))
            }
        }
    }
}

// 配置部门路由
pub fn configure_departments_routes(cfg: &mut web::ServiceConfig) {
    cfg.route("/departments", web::get().to(get_departments))
        .route("/departments", web::post().to(create_department))
        .route(
            "/departments/{department_id}",
            web::get().to(get_department),
        )
        .route(
            "/departments/{department_id}",
            web::put().to(update_department),
        )
        .route(
            "/departments/{department_id}",
            web::delete().to(delete_department),
        );
}
