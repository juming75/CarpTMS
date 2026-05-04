//! 车辆路由 - HTTP 适配层
//!
//! 该模块只负责 HTTP 请求的接收和响应，所有业务逻辑委托给 domain/use_cases

use actix_web::{web, HttpResponse};
use log::info;
use serde::Deserialize;
use std::sync::Arc;
use utoipa::ToSchema;

use crate::domain::entities::vehicle::{VehicleCreate, VehicleQuery, VehicleUpdate};
use crate::domain::use_cases::vehicle::VehicleUseCases;
use crate::errors::{
    created_response_with_message, success_response_with_message, AppError, AppResult,
};
use crate::schemas::{
    PagedResponse, VehicleCreate as SchemaVehicleCreate, VehicleResponse,
    VehicleUpdate as SchemaVehicleUpdate,
};

// 车辆查询参数
#[derive(Debug, Deserialize, ToSchema)]
pub struct VehicleQueryParam {
    pub page: Option<i32>,
    pub page_size: Option<i32>,
    pub vehicle_name: Option<String>,
    pub license_plate: Option<String>,
    pub vehicle_type: Option<String>,
    pub status: Option<i32>,
}

// ============== 类型转换函数 (HTTP <-> Domain) ==============

/// 将 SchemaVehicleCreate 转换为领域层的 VehicleCreate
fn schema_create_to_domain(create: SchemaVehicleCreate) -> VehicleCreate {
    VehicleCreate {
        vehicle_name: create.vehicle_name,
        license_plate: create.license_plate,
        vehicle_type: create.vehicle_type,
        vehicle_color: create.vehicle_color,
        vehicle_brand: create.vehicle_brand,
        vehicle_model: create.vehicle_model,
        engine_no: create.engine_no,
        frame_no: create.frame_no,
        register_date: create.register_date.naive_utc(),
        inspection_date: create.inspection_date.naive_utc(),
        insurance_date: create.insurance_date.naive_utc(),
        seating_capacity: create.seating_capacity,
        load_capacity: create.load_capacity,
        vehicle_length: create.vehicle_length,
        vehicle_width: create.vehicle_width,
        vehicle_height: create.vehicle_height,
        device_id: create.device_id,
        terminal_type: create.terminal_type,
        communication_type: create.communication_type,
        sim_card_no: create.sim_card_no,
        install_date: create.install_date.map(|d| d.naive_utc()),
        install_address: create.install_address,
        install_technician: create.install_technician,
        own_no: create.own_no,
        own_name: create.own_name,
        own_phone: create.own_phone,
        own_id_card: create.own_id_card,
        own_address: create.own_address,
        own_email: create.own_email,
        group_id: create.group_id,
        operation_status: create.operation_status,
        operation_route: create.operation_route,
        operation_area: create.operation_area,
        operation_company: create.operation_company,
        driver_name: create.driver_name,
        driver_phone: create.driver_phone,
        driver_license_no: create.driver_license_no,
        purchase_price: create.purchase_price,
        annual_fee: create.annual_fee,
        insurance_fee: create.insurance_fee,
        remark: create.remark,
        status: 1, // 默认状态为启用
        create_user_id: create.create_user_id,
    }
}

/// 将 SchemaVehicleUpdate 转换为领域层的 VehicleUpdate
fn schema_update_to_domain(update: SchemaVehicleUpdate) -> VehicleUpdate {
    VehicleUpdate {
        vehicle_name: update.vehicle_name,
        license_plate: update.license_plate,
        vehicle_type: update.vehicle_type,
        vehicle_color: update.vehicle_color,
        vehicle_brand: update.vehicle_brand,
        vehicle_model: update.vehicle_model,
        engine_no: update.engine_no,
        frame_no: update.frame_no,
        register_date: update.register_date.map(|d| d.naive_utc()),
        inspection_date: update.inspection_date.map(|d| d.naive_utc()),
        insurance_date: update.insurance_date.map(|d| d.naive_utc()),
        seating_capacity: update.seating_capacity,
        load_capacity: update.load_capacity,
        vehicle_length: update.vehicle_length,
        vehicle_width: update.vehicle_width,
        vehicle_height: update.vehicle_height,
        device_id: update.device_id,
        terminal_type: update.terminal_type,
        communication_type: update.communication_type,
        sim_card_no: update.sim_card_no,
        install_date: update.install_date.map(|d| d.naive_utc()),
        install_address: update.install_address,
        install_technician: update.install_technician,
        own_no: update.own_no,
        own_name: update.own_name,
        own_phone: update.own_phone,
        own_id_card: update.own_id_card,
        own_address: update.own_address,
        own_email: update.own_email,
        group_id: update.group_id,
        operation_status: update.operation_status,
        operation_route: update.operation_route,
        operation_area: update.operation_area,
        operation_company: update.operation_company,
        driver_name: update.driver_name,
        driver_phone: update.driver_phone,
        driver_license_no: update.driver_license_no,
        purchase_price: update.purchase_price,
        annual_fee: update.annual_fee,
        insurance_fee: update.insurance_fee,
        remark: update.remark,
        status: update.status,
        update_user_id: update.update_user_id,
    }
}

/// 将领域层 Vehicle 转换为 HTTP 响应 VehicleResponse
fn domain_to_response(vehicle: crate::domain::entities::vehicle::Vehicle) -> VehicleResponse {
    VehicleResponse {
        vehicle_id: vehicle.vehicle_id,
        vehicle_name: vehicle.vehicle_name,
        license_plate: vehicle.license_plate,
        vehicle_type: vehicle.vehicle_type,
        vehicle_color: vehicle.vehicle_color,
        vehicle_brand: vehicle.vehicle_brand,
        vehicle_model: vehicle.vehicle_model,
        engine_no: vehicle.engine_no,
        frame_no: vehicle.frame_no,
        register_date: vehicle.register_date,
        inspection_date: vehicle.inspection_date,
        insurance_date: vehicle.insurance_date,
        seating_capacity: vehicle.seating_capacity,
        load_capacity: vehicle.load_capacity,
        vehicle_length: vehicle.vehicle_length,
        vehicle_width: vehicle.vehicle_width,
        vehicle_height: vehicle.vehicle_height,
        device_id: vehicle.device_id,
        terminal_type: vehicle.terminal_type,
        communication_type: vehicle.communication_type,
        sim_card_no: vehicle.sim_card_no,
        install_date: vehicle.install_date,
        install_address: vehicle.install_address,
        install_technician: vehicle.install_technician,
        own_no: vehicle.own_no,
        own_name: vehicle.own_name,
        own_phone: vehicle.own_phone,
        own_id_card: vehicle.own_id_card,
        own_address: vehicle.own_address,
        own_email: vehicle.own_email,
        group_id: vehicle.group_id,
        operation_status: vehicle.operation_status,
        operation_route: vehicle.operation_route,
        operation_area: vehicle.operation_area,
        operation_company: vehicle.operation_company,
        driver_name: vehicle.driver_name,
        driver_phone: vehicle.driver_phone,
        driver_license_no: vehicle.driver_license_no,
        purchase_price: vehicle.purchase_price,
        annual_fee: vehicle.annual_fee,
        insurance_fee: vehicle.insurance_fee,
        remark: vehicle.remark,
        status: vehicle.status,
        create_time: vehicle.create_time,
        update_time: vehicle.update_time,
        create_user_id: vehicle.create_user_id,
        update_user_id: vehicle.update_user_id,
    }
}

// ============== 路由处理器 (仅 HTTP 适配) ==============

/// 获取所有车辆(支持分页和筛选)
#[utoipa::path(
    get, path = "/vehicles",
    responses(
        (status = 200, description = "Vehicles fetched successfully", body = ApiResponse<PagedResponse<VehicleResponse>>)
    )
)]
pub async fn get_vehicles(
    use_cases: web::Data<Arc<VehicleUseCases>>,
    query: web::Query<VehicleQueryParam>,
) -> AppResult<HttpResponse> {
    info!("HTTP: GET /vehicles");

    // 1. 提取 HTTP 参数
    let query_params = query.into_inner();

    // 2. 构建领域层查询参数
    let vehicle_query = VehicleQuery {
        page: query_params.page,
        page_size: query_params.page_size,
        vehicle_name: query_params.vehicle_name,
        license_plate: query_params.license_plate,
        vehicle_type: query_params.vehicle_type,
        status: query_params.status,
    };

    // 3. 调用 use_cases 执行业务逻辑
    let (vehicles, total_count) = use_cases.get_vehicles(vehicle_query).await?;

    // 4. 转换为 HTTP 响应
    let vehicle_responses: Vec<VehicleResponse> =
        vehicles.into_iter().map(domain_to_response).collect();

    // 5. 构建分页响应
    let page = query_params.page.unwrap_or(1);
    let page_size = query_params.page_size.unwrap_or(20);
    let paged_response = PagedResponse {
        list: vehicle_responses,
        total: total_count,
        page,
        page_size,
        pages: ((total_count + page_size as i64 - 1) / page_size as i64) as i32,
    };

    info!("Returning vehicles response");
    Ok(success_response_with_message(
        "车辆列表获取成功",
        paged_response,
    ))
}

/// 获取单个车辆
#[utoipa::path(
    get, path = "/vehicles/{id}",
    responses(
        (status = 200, description = "Vehicle fetched successfully", body = ApiResponse<VehicleResponse>),
        (status = 404, description = "Vehicle not found", body = ApiResponse<VehicleResponse>)
    )
)]
pub async fn get_vehicle(
    use_cases: web::Data<Arc<VehicleUseCases>>,
    vehicle_id: web::Path<i32>,
) -> AppResult<HttpResponse> {
    info!("HTTP: GET /vehicles/{}", *vehicle_id);

    // 1. 调用 use_cases 执行业务逻辑
    let vehicle = use_cases.get_vehicle(*vehicle_id).await?;

    // 2. 处理结果
    match vehicle {
        Some(v) => {
            // 3. 转换为 HTTP 响应
            let response = domain_to_response(v);
            Ok(success_response_with_message("车辆信息获取成功", response))
        }
        None => Err(AppError::not_found_error("Vehicle not found".to_string())),
    }
}

/// 创建车辆
#[utoipa::path(
    post, path = "/vehicles",
    request_body = VehicleCreate,
    responses(
        (status = 201, description = "Vehicle created successfully", body = ApiResponse<VehicleResponse>),
        (status = 400, description = "Invalid input data", body = ApiResponse<VehicleResponse>),
        (status = 500, description = "Failed to create vehicle", body = ApiResponse<VehicleResponse>)
    )
)]
pub async fn create_vehicle(
    use_cases: web::Data<Arc<VehicleUseCases>>,
    vehicle: web::Json<SchemaVehicleCreate>,
) -> AppResult<HttpResponse> {
    info!("HTTP: POST /vehicles");

    // 1. 反序列化为领域对象
    let create_data = schema_create_to_domain(vehicle.into_inner());

    // 2. 调用 use_cases 执行业务逻辑
    let created_vehicle = use_cases.create_vehicle(create_data).await?;

    // 3. 转换为 HTTP 响应
    let response = domain_to_response(created_vehicle);

    info!(
        "Vehicle created successfully: id={}, name={}",
        response.vehicle_id, response.vehicle_name
    );
    Ok(created_response_with_message("车辆创建成功", response))
}

/// 更新车辆
#[utoipa::path(
    put, path = "/vehicles/{id}",
    request_body = VehicleUpdate,
    responses(
        (status = 200, description = "Vehicle updated successfully", body = ApiResponse<VehicleResponse>),
        (status = 400, description = "Invalid input data", body = ApiResponse<VehicleResponse>),
        (status = 404, description = "Vehicle not found", body = ApiResponse<VehicleResponse>),
        (status = 500, description = "Failed to update vehicle", body = ApiResponse<VehicleResponse>)
    )
)]
pub async fn update_vehicle(
    use_cases: web::Data<Arc<VehicleUseCases>>,
    vehicle_id: web::Path<i32>,
    vehicle: web::Json<SchemaVehicleUpdate>,
) -> AppResult<HttpResponse> {
    info!("HTTP: PUT /vehicles/{}", *vehicle_id);

    // 1. 反序列化为领域对象
    let update_data = schema_update_to_domain(vehicle.into_inner());

    // 2. 调用 use_cases 执行业务逻辑
    let updated_vehicle = use_cases.update_vehicle(*vehicle_id, update_data).await?;

    // 3. 处理结果
    match updated_vehicle {
        Some(v) => {
            // 4. 转换为 HTTP 响应
            let response = domain_to_response(v);
            Ok(success_response_with_message("车辆更新成功", response))
        }
        None => Err(AppError::not_found_error("Vehicle not found".to_string())),
    }
}

/// 删除车辆
#[utoipa::path(
    delete, path = "/vehicles/{id}",
    responses(
        (status = 200, description = "Vehicle deleted successfully", body = ApiResponse<()>),
        (status = 404, description = "Vehicle not found", body = ApiResponse<VehicleResponse>),
        (status = 500, description = "Failed to delete vehicle", body = ApiResponse<VehicleResponse>)
    )
)]
pub async fn delete_vehicle(
    use_cases: web::Data<Arc<VehicleUseCases>>,
    vehicle_id: web::Path<i32>,
) -> AppResult<HttpResponse> {
    info!("HTTP: DELETE /vehicles/{}", *vehicle_id);

    // 1. 调用 use_cases 执行业务逻辑
    let result = use_cases.delete_vehicle(*vehicle_id).await?;

    // 2. 处理结果
    if result {
        info!("Vehicle deleted successfully: id={}", *vehicle_id);
        Ok(success_response_with_message("车辆删除成功", ()))
    } else {
        Err(AppError::not_found_error("Vehicle not found".to_string()))
    }
}
