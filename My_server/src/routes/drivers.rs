use actix_web::{web, HttpResponse, Responder};
use log::{error, info, warn};
use std::sync::Arc;


use crate::domain::entities::driver::{DriverCreateRequest, DriverQuery, DriverUpdateRequest};
use crate::domain::use_cases::driver::DriverUseCases;
use crate::errors::{AppError, AppResult, success_response_with_message};
use crate::schemas::{ApiResponse, DriverCreate, DriverResponse, DriverUpdate, PagedResponse};



// 获取司机列表(支持分页和筛选)
#[utoipa::path(
    path = "/api/drivers",
    get,
    responses(
        (status = 200, description = "Drivers fetched successfully", body = ApiResponse<PagedResponse<DriverResponse>>),
        (status = 500, description = "Internal server error", body = ApiResponse<()>)
    )
)]
pub async fn get_drivers(
    driver_use_cases: web::Data<Arc<DriverUseCases>>,
    query: web::Query<DriverQuery>,
) -> impl Responder {
    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20);

    // 转换为领域查询对象
    let domain_query = DriverQuery {
        page: Some(page),
        page_size: Some(page_size),
        driver_name: query.driver_name.clone(),
        license_number: query.license_number.clone(),
        status: query.status,
    };

    // 调用用例获取司机列表
    match driver_use_cases.get_drivers(domain_query).await {
        Ok((drivers, total)) => {
            // 转换为响应格式
            let driver_responses: Vec<DriverResponse> = drivers
                .into_iter()
                .map(|driver| DriverResponse {
                    driver_id: driver.driver_id,
                    driver_name: driver.driver_name,
                    license_number: driver.license_number.unwrap_or_default(),
                    phone_number: driver.phone_number,
                    email: driver.email,
                    status: driver.status,
                    create_time: driver.create_time,
                    update_time: driver.update_time,
                })
                .collect();

            // 计算总页数
            let pages = if total % page_size as i64 == 0 {
                total / page_size as i64
            } else {
                total / page_size as i64 + 1
            };

            HttpResponse::Ok().json(ApiResponse {
                code: 200,
                message: "Drivers fetched successfully".to_string(),
                data: PagedResponse {
                    list: driver_responses,
                    total,
                    page,
                    page_size,
                    pages: pages as i32,
                },
            })
        }
        Err(e) => {
            error!("Failed to fetch drivers: {:?}", e);
            HttpResponse::InternalServerError().json(ApiResponse {
                code: 500,
                message: "Failed to fetch drivers".to_string(),
                data: (),
            })
        }
    }
}

// 创建司机
#[utoipa::path(
    path = "/api/drivers",
    post,
    request_body = DriverCreate,
    responses(
        (status = 201, description = "Driver created successfully", body = ApiResponse<DriverResponse>),
        (status = 400, description = "Invalid request parameters", body = ApiResponse<()>),
        (status = 500, description = "Internal server error", body = ApiResponse<()>)
    )
)]
pub async fn create_driver(
    driver_use_cases: web::Data<Arc<DriverUseCases>>,
    driver: web::Json<DriverCreate>,
) -> impl Responder {
    info!("Creating driver: {}", driver.driver_name);

    // 转换为领域创建请求
    let create_request = DriverCreateRequest {
        driver_name: driver.driver_name.clone(),
        license_number: Some(driver.license_number.clone()),
        phone_number: driver.phone_number.clone(),
        email: driver.email.clone(),
        status: driver.status,
    };

    // 调用用例创建司机
    match driver_use_cases.create_driver(create_request).await {
        Ok(created_driver) => {
            // 转换为响应格式
            let response = DriverResponse {
                driver_id: created_driver.driver_id,
                driver_name: created_driver.driver_name,
                license_number: created_driver.license_number.unwrap_or_default(),
                phone_number: created_driver.phone_number,
                email: created_driver.email,
                status: created_driver.status,
                create_time: created_driver.create_time,
                update_time: created_driver.update_time,
            };

            info!("Driver created successfully: {}", driver.driver_name);
            HttpResponse::Created().json(ApiResponse {
                code: 201,
                message: "Driver created successfully".to_string(),
                data: response,
            })
        }
        Err(e) => {
            error!("Failed to create driver: {:?}", e);
            if e.to_string().contains("司机名称已存在") {
                HttpResponse::BadRequest().json(ApiResponse {
                    code: 400,
                    message: "Driver name already exists".to_string(),
                    data: (),
                })
            } else {
                HttpResponse::InternalServerError().json(ApiResponse {
                    code: 500,
                    message: "Failed to create driver".to_string(),
                    data: (),
                })
            }
        }
    }
}

// 获取司机详情
#[utoipa::path(
    path = "/api/drivers/{driver_id}",
    get,
    responses(
        (status = 200, description = "Driver fetched successfully", body = ApiResponse<DriverResponse>),
        (status = 404, description = "Driver not found", body = ApiResponse<()>),
        (status = 500, description = "Internal server error", body = ApiResponse<()>)
    )
)]
pub async fn get_driver(driver_use_cases: web::Data<Arc<DriverUseCases>>, driver_id: web::Path<i32>) -> impl Responder {
    let driver_id = driver_id.into_inner();

    // 调用用例获取司机详情
    match driver_use_cases.get_driver(driver_id).await {
        Ok(driver) => match driver {
            Some(driver) => {
                // 转换为响应格式
                let response = DriverResponse {
                    driver_id: driver.driver_id,
                    driver_name: driver.driver_name,
                    license_number: driver.license_number.unwrap_or_default(),
                    phone_number: driver.phone_number,
                    email: driver.email,
                    status: driver.status,
                    create_time: driver.create_time,
                    update_time: driver.update_time,
                };

                HttpResponse::Ok().json(ApiResponse {
                    code: 200,
                    message: "Driver fetched successfully".to_string(),
                    data: response,
                })
            }
            None => {
                warn!("Driver not found: {}", driver_id);
                HttpResponse::NotFound().json(ApiResponse {
                    code: 404,
                    message: "Driver not found".to_string(),
                    data: (),
                })
            }
        },
        Err(e) => {
            error!("Failed to fetch driver: {:?}", e);
            HttpResponse::InternalServerError().json(ApiResponse {
                code: 500,
                message: "Failed to fetch driver".to_string(),
                data: (),
            })
        }
    }
}

// 更新司机
#[utoipa::path(
    path = "/api/drivers/{driver_id}",
    put,
    request_body = DriverUpdate,
    responses(
        (status = 200, description = "Driver updated successfully", body = ApiResponse<DriverResponse>),
        (status = 400, description = "Invalid request parameters", body = ApiResponse<()>),
        (status = 404, description = "Driver not found", body = ApiResponse<()>),
        (status = 500, description = "Internal server error", body = ApiResponse<()>)
    )
)]
pub async fn update_driver(
    driver_use_cases: web::Data<Arc<DriverUseCases>>,
    driver_id: web::Path<i32>,
    driver: web::Json<DriverUpdate>,
) -> impl Responder {
    let driver_id = driver_id.into_inner();

    // 转换为领域更新请求
    let update_request = DriverUpdateRequest {
        driver_name: driver.driver_name.clone(),
        license_number: driver.license_number.clone(),
        phone_number: driver.phone_number.clone(),
        email: driver.email.clone(),
        status: driver.status,
    };

    // 调用用例更新司机
    match driver_use_cases.update_driver(driver_id, update_request).await {
        Ok(updated_driver) => match updated_driver {
            Some(driver) => {
                // 转换为响应格式
                let response = DriverResponse {
                    driver_id: driver.driver_id,
                    driver_name: driver.driver_name,
                    license_number: driver.license_number.unwrap_or_default(),
                    phone_number: driver.phone_number,
                    email: driver.email,
                    status: driver.status,
                    create_time: driver.create_time,
                    update_time: driver.update_time,
                };

                info!("Driver updated successfully: {}", response.driver_name);
                HttpResponse::Ok().json(ApiResponse {
                    code: 200,
                    message: "Driver updated successfully".to_string(),
                    data: response,
                })
            }
            None => {
                warn!("Driver not found for update: {}", driver_id);
                HttpResponse::NotFound().json(ApiResponse {
                    code: 404,
                    message: "Driver not found".to_string(),
                    data: (),
                })
            }
        },
        Err(e) => {
            error!("Failed to update driver: {:?}", e);
            if e.to_string().contains("司机名称已存在") {
                HttpResponse::BadRequest().json(ApiResponse {
                    code: 400,
                    message: "Driver name already exists".to_string(),
                    data: (),
                })
            } else {
                HttpResponse::InternalServerError().json(ApiResponse {
                    code: 500,
                    message: "Failed to update driver".to_string(),
                    data: (),
                })
            }
        }
    }
}

// 删除司机
#[utoipa::path(
    path = "/api/drivers/{driver_id}",
    delete,
    responses(
        (status = 200, description = "Driver deleted successfully", body = ApiResponse<()>),
        (status = 404, description = "Driver not found", body = ApiResponse<()>),
        (status = 500, description = "Internal server error", body = ApiResponse<()>)
    )
)]
pub async fn delete_driver(driver_use_cases: web::Data<Arc<DriverUseCases>>, driver_id: web::Path<i32>) -> AppResult<HttpResponse> {
    let driver_id = driver_id.into_inner();

    // 调用用例删除司机
    match driver_use_cases.delete_driver(driver_id).await {
        Ok(deleted) => {
            if deleted {
                info!("Driver deleted successfully: {}", driver_id);
                Ok(success_response_with_message("Driver deleted successfully", ()))
            } else {
                warn!("Driver not found for deletion: {}", driver_id);
                Err(AppError::not_found_error("Driver not found".to_string()))
            }
        }
        Err(e) => {
            error!("Failed to delete driver: {:?}", e);
            if e.to_string().contains("司机有关联数据，无法删除") {
                Err(AppError::business_error("Cannot delete driver with associated data", None))
            } else {
                Err(AppError::db_error("Failed to delete driver", None))
            }
        }
    }
}

// 配置司机路由
pub fn configure_driver_routes(cfg: &mut web::ServiceConfig) {
    cfg.route("/drivers", web::get().to(get_drivers))
        .route("/drivers", web::post().to(create_driver))
        .route("/drivers/{driver_id}", web::get().to(get_driver))
        .route("/drivers/{driver_id}", web::put().to(update_driver))
        .route("/drivers/{driver_id}", web::delete().to(delete_driver));
}
