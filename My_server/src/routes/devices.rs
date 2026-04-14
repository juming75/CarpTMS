use actix_web::{web, HttpResponse};
use chrono::{TimeZone, Utc};
use log::{info, warn};
use validator::Validate;

use crate::application::services::device_service::{DeviceService, DeviceServiceImpl};
use std::sync::Arc;
use crate::domain::entities::device::{DeviceCreate as DomainDeviceCreate, DeviceQuery as DomainDeviceQuery, DeviceUpdate as DomainDeviceUpdate};
use crate::errors::{created_response, success_response, AppError, AppResult};
use crate::redis::{del_cache_pattern, get_cache, set_cache};
use crate::schemas::{DeviceCreate, DeviceQuery, DeviceResponse, DeviceUpdate, PagedResponse};

// 获取设备列表(支持分页和筛选)
#[utoipa::path(
    path = "/api/devices",
    get,
    responses(
        (status = 200, description = "Devices fetched successfully", body = ApiResponse<PagedResponse<DeviceResponse>>),
        (status = 500, description = "Internal server error", body = ApiResponse<()>)
    )
)]
pub async fn get_devices(
    device_service: web::Data<Arc<DeviceServiceImpl>>,
    query: web::Query<DeviceQuery>,
) -> AppResult<HttpResponse> {
    let page = query.0.page.unwrap_or(1);
    let page_size = query.0.page_size.unwrap_or(20);

    // 生成缓存键
    let cache_key = format!(
        "devices:list:device_id_{}:device_name_{}:device_type_{}:status_{}:manufacturer_{}:page_{}:size_{}",
        query.0.device_id.as_deref().unwrap_or(""),
        query.0.device_name.as_deref().unwrap_or(""),
        query.0.device_type.as_deref().unwrap_or(""),
        query.0.status.map(|s| s.to_string()).unwrap_or_default(),
        query.0.manufacturer.as_deref().unwrap_or(""),
        page,
        page_size
    );

    // 尝试从缓存获取
    if let Ok(Some(cached_response)) = get_cache::<PagedResponse<DeviceResponse>>(&cache_key).await
    {
        return Ok(success_response(Some(cached_response)));
    }

    // 转换为领域查询对象
    let domain_query = DomainDeviceQuery {
        page: Some(page),
        page_size: Some(page_size),
        device_id: query.0.device_id.clone(),
        device_name: query.0.device_name.clone(),
        device_type: query.0.device_type.clone(),
        manufacturer: query.0.manufacturer.clone(),
        status: query.0.status,
    };

    // 调用服务获取设备列表
    let (devices, total) = device_service.get_devices(domain_query).await?;

    // 转换为响应格式
    let device_responses: Vec<DeviceResponse> = devices
        .into_iter()
        .map(|device| {
            DeviceResponse {
                device_id: device.device_id,
                device_name: device.device_name,
                device_type: device.device_type,
                device_model: device.device_model,
                manufacturer: device.manufacturer,
                serial_number: device.serial_number,
                communication_type: device.communication_type,
                sim_card_no: device.sim_card_no,
                ip_address: device.ip_address,
                port: device.port,
                mac_address: device.mac_address,
                install_date: device.install_date.map(|t| Utc.from_utc_datetime(&t)),
                install_address: device.install_address,
                install_technician: device.install_technician,
                status: device.status,
                remark: device.remark,
                create_time: Utc.from_utc_datetime(&device.create_time),
                update_time: device.update_time.map(|t| Utc.from_utc_datetime(&t)),
                create_user_id: device.create_user_id,
                update_user_id: device.update_user_id,
            }
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
        list: device_responses,
        total,
        page,
        page_size,
        pages: pages as i32,
    };

    // 缓存结果,过期时间30分钟
    let _ = set_cache(&cache_key, &paged_response, 1800).await;

    Ok(success_response(Some(paged_response)))
}

// 创建设备
#[utoipa::path(
    path = "/api/devices",
    post,
    request_body = DeviceCreate,
    responses(
        (status = 201, description = "Device created successfully", body = ApiResponse<DeviceResponse>),
        (status = 400, description = "Invalid request parameters", body = ApiResponse<()>),
        (status = 500, description = "Internal server error", body = ApiResponse<()>)
    )
)]
pub async fn create_device(
    device_service: web::Data<Arc<DeviceServiceImpl>>,
    device: web::Json<DeviceCreate>,
) -> AppResult<HttpResponse> {
    let device_id = device.0.device_id.clone();
    info!("Creating device: {}", device_id);

    // 验证请求数据
    device.validate()?;

    // 转换为领域创建对象
    let domain_device_create = DomainDeviceCreate {
        device_id: device.0.device_id,
        device_name: device.0.device_name,
        device_type: device.0.device_type,
        device_model: device.0.device_model,
        manufacturer: device.0.manufacturer,
        serial_number: device.0.serial_number,
        communication_type: device.0.communication_type,
        sim_card_no: device.0.sim_card_no,
        ip_address: device.0.ip_address,
        port: device.0.port,
        mac_address: device.0.mac_address,
        install_date: device.0.install_date,
        install_address: device.0.install_address,
        install_technician: device.0.install_technician,
        status: device.0.status,
        remark: device.0.remark,
        create_user_id: device.0.create_user_id,
    };

    // 调用服务创建设备
    let created_device = device_service.create_device(domain_device_create).await?;

    // 转换为响应格式
    let response = DeviceResponse {
        device_id: created_device.device_id,
        device_name: created_device.device_name,
        device_type: created_device.device_type,
        device_model: created_device.device_model,
        manufacturer: created_device.manufacturer,
        serial_number: created_device.serial_number,
        communication_type: created_device.communication_type,
        sim_card_no: created_device.sim_card_no,
        ip_address: created_device.ip_address,
        port: created_device.port,
        mac_address: created_device.mac_address,
        install_date: created_device.install_date.map(|t| Utc.from_utc_datetime(&t)),
        install_address: created_device.install_address,
        install_technician: created_device.install_technician,
        status: created_device.status,
        remark: created_device.remark,
        create_time: Utc.from_utc_datetime(&created_device.create_time),
        update_time: created_device.update_time.map(|t| Utc.from_utc_datetime(&t)),
        create_user_id: created_device.create_user_id,
        update_user_id: created_device.update_user_id,
    };

    // 清理相关缓存
    let _ = del_cache_pattern("device:*:*").await;
    let _ = del_cache_pattern("devices:list:*").await;
    let _ = del_cache_pattern("statistics:devices").await;

    info!("Device created successfully: {}", device_id);
    Ok(created_response(Some(response)))
}

// 获取设备详情
#[utoipa::path(
    path = "/api/devices/{device_id}",
    get,
    responses(
        (status = 200, description = "Device fetched successfully", body = ApiResponse<DeviceResponse>),
        (status = 404, description = "Device not found", body = ApiResponse<()>),
        (status = 500, description = "Internal server error", body = ApiResponse<()>) 
    )
)]
pub async fn get_device(
    device_service: web::Data<Arc<DeviceServiceImpl>>,
    device_id: web::Path<String>,
) -> AppResult<HttpResponse> {
    let device_id = device_id.into_inner();

    // 尝试从缓存获取
    let cache_key = format!("device:{}:details", device_id);
    if let Ok(Some(cached_response)) = get_cache::<DeviceResponse>(&cache_key).await { 
        return Ok(success_response(Some(cached_response)));
    }

    // 调用服务获取设备
    let device = device_service.get_device(&device_id).await?;

    match device {
        Some(device) => {
            let response = DeviceResponse {
                device_id: device.device_id,
                device_name: device.device_name,
                device_type: device.device_type,
                device_model: device.device_model,
                manufacturer: device.manufacturer,
                serial_number: device.serial_number,
                communication_type: device.communication_type,
                sim_card_no: device.sim_card_no,
                ip_address: device.ip_address,
                port: device.port,
                mac_address: device.mac_address,
                install_date: device.install_date.map(|t| Utc.from_utc_datetime(&t)),
                install_address: device.install_address,
                install_technician: device.install_technician,
                status: device.status,
                remark: device.remark,
                create_time: Utc.from_utc_datetime(&device.create_time),
                update_time: device.update_time.map(|t| Utc.from_utc_datetime(&t)),
                create_user_id: device.create_user_id,
                update_user_id: device.update_user_id,
            };

            // 缓存结果,过期时间30分钟
            let _ = set_cache(&cache_key, &response, 1800).await;

            Ok(success_response(Some(response)))
        }
        None => {
            warn!("Device not found: {}", device_id);
            Err(AppError::not_found_error("Device not found".to_string()))
        }
    }
}

// 更新设备
#[utoipa::path(
    path = "/api/devices/{device_id}",
    put,
    request_body = DeviceUpdate,
    responses(
        (status = 200, description = "Device updated successfully", body = ApiResponse<DeviceResponse>),
        (status = 400, description = "Invalid request parameters", body = ApiResponse<()>),
        (status = 404, description = "Device not found", body = ApiResponse<()>),
        (status = 500, description = "Internal server error", body = ApiResponse<()>)
    )
)]
pub async fn update_device(
    device_service: web::Data<Arc<DeviceServiceImpl>>,
    device_id: web::Path<String>,
    device: web::Json<DeviceUpdate>,
) -> AppResult<HttpResponse> {
    let device_id = device_id.into_inner();

    // 转换为领域更新对象
    let domain_device_update = DomainDeviceUpdate {
        device_name: device.0.device_name,
        device_type: device.0.device_type,
        device_model: device.0.device_model,
        manufacturer: device.0.manufacturer,
        serial_number: device.0.serial_number,
        communication_type: device.0.communication_type,
        sim_card_no: device.0.sim_card_no,
        ip_address: device.0.ip_address,
        port: device.0.port,
        mac_address: device.0.mac_address,
        install_date: device.0.install_date,
        install_address: device.0.install_address,
        install_technician: device.0.install_technician,
        status: device.0.status,
        remark: device.0.remark,
        update_user_id: device.0.update_user_id,
    };

    // 调用服务更新设备
    let updated_device = device_service.update_device(&device_id, domain_device_update).await?;

    match updated_device {
        Some(device) => {
            // 转换为响应格式
            let response = DeviceResponse {
                device_id: device.device_id,
                device_name: device.device_name,
                device_type: device.device_type,
                device_model: device.device_model,
                manufacturer: device.manufacturer,
                serial_number: device.serial_number,
                communication_type: device.communication_type,
                sim_card_no: device.sim_card_no,
                ip_address: device.ip_address,
                port: device.port,
                mac_address: device.mac_address,
                install_date: device.install_date.map(|t| Utc.from_utc_datetime(&t)),
                install_address: device.install_address,
                install_technician: device.install_technician,
                status: device.status,
                remark: device.remark,
                create_time: Utc.from_utc_datetime(&device.create_time),
                update_time: device.update_time.map(|t| Utc.from_utc_datetime(&t)),
                create_user_id: device.create_user_id,
                update_user_id: device.update_user_id,
            };

            // 清理相关缓存
            let _ = del_cache_pattern("device:*:*").await;
            let _ = del_cache_pattern("devices:list:*").await;
            let _ = del_cache_pattern("statistics:devices").await;

            info!("Device updated successfully: {}", device_id);
            Ok(success_response(Some(response)))
        }
        None => {
            warn!("Device not found for update: {}", device_id);
            Err(AppError::not_found_error("Device not found".to_string()))
        }
    }
}

// 删除设备
#[utoipa::path(
    path = "/api/devices/{device_id}",
    delete,
    responses(
        (status = 200, description = "Device deleted successfully", body = ApiResponse<()>),
        (status = 400, description = "Cannot delete device in use by vehicles", body = ApiResponse<()>),
        (status = 404, description = "Device not found", body = ApiResponse<()>),
        (status = 500, description = "Internal server error", body = ApiResponse<()>)
    )
)]
pub async fn delete_device(
    device_service: web::Data<Arc<DeviceServiceImpl>>,
    device_id: web::Path<String>,
) -> AppResult<HttpResponse> {
    let device_id = device_id.into_inner();

    // 调用服务删除设备
    let deleted = device_service.delete_device(&device_id).await?;

    if deleted {
        // 清理相关缓存
        let _ = del_cache_pattern("device:*:*").await;
        let _ = del_cache_pattern("devices:list:*").await;
        let _ = del_cache_pattern("statistics:devices").await;

        info!("Device deleted successfully: {}", device_id);
        Ok(success_response(()))
    } else {
        warn!("Device not found for deletion: {}", device_id);
        Err(AppError::not_found_error("Device not found".to_string()))
    }
}

// 配置设备路由
pub fn configure_devices_routes(cfg: &mut web::ServiceConfig) {
    use crate::application::services::device_service::DeviceServiceImpl;
    
    cfg.route("/devices", web::get().to::<_, (web::Data<Arc<DeviceServiceImpl>>, web::Query<DeviceQuery>)>(get_devices))
        .route("/devices", web::post().to::<_, (web::Data<Arc<DeviceServiceImpl>>, web::Json<DeviceCreate>)>(create_device))
        .route("/devices/{device_id}", web::get().to::<_, (web::Data<Arc<DeviceServiceImpl>>, web::Path<String>)>(get_device))
        .route("/devices/{device_id}", web::put().to::<_, (web::Data<Arc<DeviceServiceImpl>>, web::Path<String>, web::Json<DeviceUpdate>)>(update_device))
        .route("/devices/{device_id}", web::delete().to::<_, (web::Data<Arc<DeviceServiceImpl>>, web::Path<String>)>(delete_device));
}
