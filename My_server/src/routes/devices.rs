use actix_web::{web, HttpResponse};
use chrono::{TimeZone, Utc};
use log::{info, warn};
use validator::Validate;

use crate::application::services::device_service::{DeviceService, DeviceServiceImpl};
use crate::domain::entities::device::{
    DeviceCreate as DomainDeviceCreate, DeviceQuery as DomainDeviceQuery,
    DeviceUpdate as DomainDeviceUpdate,
};
use crate::errors::{created_response, success_response, AppError, AppResult};
use crate::redis::{del_cache_pattern, get_cache, set_cache};
use crate::schemas::{DeviceCreate, DeviceQuery, DeviceResponse, DeviceUpdate, PagedResponse};
use crate::utils::log_cache_error;
use std::sync::Arc;

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
        .map(|device| DeviceResponse {
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
    log_cache_error(
        set_cache(&cache_key, &paged_response, 1800).await,
        "set devices list cache",
    );

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
        install_date: created_device
            .install_date
            .map(|t| Utc.from_utc_datetime(&t)),
        install_address: created_device.install_address,
        install_technician: created_device.install_technician,
        status: created_device.status,
        remark: created_device.remark,
        create_time: Utc.from_utc_datetime(&created_device.create_time),
        update_time: created_device
            .update_time
            .map(|t| Utc.from_utc_datetime(&t)),
        create_user_id: created_device.create_user_id,
        update_user_id: created_device.update_user_id,
    };

    // 清理相关缓存
    log_cache_error(
        del_cache_pattern("device:*:*").await,
        "del device cache on create",
    );
    log_cache_error(
        del_cache_pattern("devices:list:*").await,
        "del devices list cache on create",
    );
    log_cache_error(
        del_cache_pattern("statistics:devices").await,
        "del statistics cache on create",
    );

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
            log_cache_error(
                set_cache(&cache_key, &response, 1800).await,
                "set device detail cache",
            );

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
    let updated_device = device_service
        .update_device(&device_id, domain_device_update)
        .await?;

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
            log_cache_error(
                del_cache_pattern("device:*:*").await,
                "del device cache on update",
            );
            log_cache_error(
                del_cache_pattern("devices:list:*").await,
                "del devices list cache on update",
            );
            log_cache_error(
                del_cache_pattern("statistics:devices").await,
                "del statistics cache on update",
            );

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
        log_cache_error(
            del_cache_pattern("device:*:*").await,
            "del device cache on delete",
        );
        log_cache_error(
            del_cache_pattern("devices:list:*").await,
            "del devices list cache on delete",
        );
        log_cache_error(
            del_cache_pattern("statistics:devices").await,
            "del statistics cache on delete",
        );

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

    cfg.route(
        "/devices",
        web::get()
            .to::<_, (web::Data<Arc<DeviceServiceImpl>>, web::Query<DeviceQuery>)>(get_devices),
    )
    .route(
        "/devices",
        web::post()
            .to::<_, (web::Data<Arc<DeviceServiceImpl>>, web::Json<DeviceCreate>)>(create_device),
    )
    .route(
        "/devices/{device_id}",
        web::get().to::<_, (web::Data<Arc<DeviceServiceImpl>>, web::Path<String>)>(get_device),
    )
    .route(
        "/devices/{device_id}",
        web::put().to::<_, (
            web::Data<Arc<DeviceServiceImpl>>,
            web::Path<String>,
            web::Json<DeviceUpdate>,
        )>(update_device),
    )
    .route(
        "/devices/{device_id}",
        web::delete()
            .to::<_, (web::Data<Arc<DeviceServiceImpl>>, web::Path<String>)>(delete_device),
    )
    // ========== 终端参数配置路由 ==========
    // LED文本发送
    .route(
        "/devices/terminal/led",
        web::post().to(send_led_text)
    )
    // 载重参数下发
    .route(
        "/devices/terminal/load-params",
        web::post().to(send_load_params)
    )
    // 终端基本参数下发
    .route(
        "/devices/terminal/params",
        web::post().to(send_terminal_params)
    )
    // GPS参数下发
    .route(
        "/devices/terminal/gps-params",
        web::post().to(send_gps_params)
    )
    // 通信参数下发
    .route(
        "/devices/terminal/comm-params",
        web::post().to(send_comm_params)
    );
}

// ==================== 终端参数配置API实现 ====================

use serde::{Deserialize, Serialize};

/// LED文本发送请求体
#[derive(Debug, Deserialize, Clone)]
pub struct LedTextRequest {
    pub vehicle_ids: Vec<String>,
    pub box_number: u32,
    pub port: i32,
    pub content: String,
    #[serde(default)]
    pub display_mode: Option<String>,
}

/// 载重参数请求体
#[derive(Debug, Deserialize, Clone)]
pub struct LoadParamsRequest {
    pub vehicle_ids: Vec<String>,
    pub calibration_coefficient: f64,
    pub empty_weight: f64,
    pub full_load_weight: f64,
    pub overload_threshold: f64,
    pub sampling_interval: u32,
    pub filter_factor: f64,
}

/// 终端参数请求体
#[derive(Debug, Deserialize, Clone)]
pub struct TerminalParamsRequest {
    pub vehicle_ids: Vec<String>,
    pub heartbeat_interval: u32,
    pub tcp_timeout: u32,
    pub location_interval: u32,
    pub sleep_interval: u32,
    pub speed_threshold: u32,
    pub fatigue_threshold: f64,
}

/// GPS参数请求体
#[derive(Debug, Deserialize, Clone)]
pub struct GpsParamsRequest {
    pub vehicle_ids: Vec<String>,
    pub position_mode: String,
    pub altitude_offset: i32,
    pub min_satellites: u32,
    pub pdop_threshold: f64,
}

/// 通信参数请求体
#[derive(Debug, Deserialize, Clone)]
pub struct CommParamsRequest {
    pub vehicle_ids: Vec<String>,
    pub primary_server_ip: String,
    pub primary_server_port: u16,
    pub backup_server_ip: Option<String>,
    pub backup_server_port: Option<u16>,
    pub apn_name: Option<String>,
    pub apn_username: Option<String>,
    pub apn_password: Option<String>,
}

/// 通用响应结构
#[derive(Debug, Serialize)]
struct TerminalCommandResponse {
    code: i32,
    message: String,
    data: serde_json::Value,
}

/// 发送LED文本
///
/// # 功能说明
/// 向指定车辆发送LED显示文本消息。
///
/// # 参数说明
/// - `vehicle_ids`: 目标车辆ID列表
/// - `box_number`: 信箱号（0-999）
/// - `port`: LED地址（0或1）
/// - `content`: 显示文本内容（1-50字符）
/// - `display_mode`: 显示模式（static/scroll/flash）
#[utoipa::path(
    post,
    path = "/api/devices/terminal/led",
    request_body = LedTextRequest,
    responses(
        (status = 200, description = "LED文本发送成功"),
        (status = 400, description = "参数错误")
    ),
    tag = "Terminal"
)]
pub async fn send_led_text(req: web::Json<LedTextRequest>) -> AppResult<HttpResponse> {
    let data = req.into_inner();

    // 验证必填字段
    if data.vehicle_ids.is_empty() {
        return Err(AppError::bad_request("请选择至少一辆车"));
    }
    if data.content.is_empty() || data.content.len() > 50 {
        return Err(AppError::bad_request("文本内容长度必须在1-50个字符之间"));
    }
    if data.box_number > 999 {
        return Err(AppError::bad_request("信箱号必须在0-999之间"));
    }

    info!(
        "[Terminal] 发送LED文本到 {} 辆车: content={}, mode={:?}",
        data.vehicle_ids.len(),
        data.content,
        data.display_mode
    );

    // TODO: 实现真实的JT808协议发送逻辑
    // 当前返回模拟成功响应

    let response = TerminalCommandResponse {
        code: 200,
        message: format!("LED文本已下发到 {} 辆车", data.vehicle_ids.len()),
        data: serde_json::json!({
            "task_id": format!("led_{}", Utc::now().timestamp()),
            "vehicle_count": data.vehicle_ids.len(),
            "content": data.content,
            "display_mode": data.display_mode.unwrap_or_else(|| "static".to_string()),
            "status": "pending"
        }),
    };

    Ok(HttpResponse::Ok().json(response))
}

/// 下发载重参数
///
/// # 功能说明
/// 设置车辆的载重相关参数，包括标定系数、空车重量等。
#[utoipa::path(
    post,
    path = "/api/devices/terminal/load-params",
    request_body = LoadParamsRequest,
    responses(
        (status = 200, description = "载重参数下发成功")
    ),
    tag = "Terminal"
)]
pub async fn send_load_params(req: web::Json<LoadParamsRequest>) -> AppResult<HttpResponse> {
    let data = req.into_inner();

    if data.vehicle_ids.is_empty() {
        return Err(AppError::bad_request("请选择至少一辆车"));
    }

    // 验证参数范围
    if data.calibration_coefficient <= 0.0 || data.calibration_coefficient > 99999.0 {
        return Err(AppError::bad_request("标定系数必须在0-99999之间"));
    }

    info!(
        "[Terminal] 下发载重参数到 {} 辆车: coefficient={:.2}, empty={:.2}",
        data.vehicle_ids.len(),
        data.calibration_coefficient,
        data.empty_weight
    );

    // TODO: 实现真实的参数下发逻辑
    // 可选：保存到数据库的 terminal_params 表

    let response = TerminalCommandResponse {
        code: 200,
        message: format!("载重参数已下发到 {} 辆车", data.vehicle_ids.len()),
        data: serde_json::json!({
            "task_id": format!("load_{}", Utc::now().timestamp()),
            "vehicle_count": data.vehicle_ids.len(),
            "calibration_coefficient": data.calibration_coefficient,
            "empty_weight": data.empty_weight,
            "full_load_weight": data.full_load_weight,
            "overload_threshold": data.overload_threshold,
            "sampling_interval": data.sampling_interval,
            "filter_factor": data.filter_factor,
            "status": "pending"
        }),
    };

    Ok(HttpResponse::Ok().json(response))
}

/// 下发终端基本参数
///
/// # 功能说明
/// 设置终端的基本运行参数，包括心跳、超时、定位间隔等。
#[utoipa::path(
    post,
    path = "/api/devices/terminal/params",
    request_body = TerminalParamsRequest,
    responses(
        (status = 200, description = "终端参数下发成功")
    ),
    tag = "Terminal"
)]
pub async fn send_terminal_params(req: web::Json<TerminalParamsRequest>) -> AppResult<HttpResponse> {
    let data = req.into_inner();

    if data.vehicle_ids.is_empty() {
        return Err(AppError::bad_request("请选择至少一辆车"));
    }

    // 验证参数范围
    if data.heartbeat_interval < 10 || data.heartbeat_interval > 300 {
        return Err(AppError::bad_request("心跳间隔必须在10-300秒之间"));
    }

    info!(
        "[Terminal] 下发终端参数到 {} 辆车: heartbeat={}s, location={}s",
        data.vehicle_ids.len(),
        data.heartbeat_interval,
        data.location_interval
    );

    let response = TerminalCommandResponse {
        code: 200,
        message: format!("终端参数已下发到 {} 辆车", data.vehicle_ids.len()),
        data: serde_json::json!({
            "task_id": format!("term_{}", Utc::now().timestamp()),
            "vehicle_count": data.vehicle_ids.len(),
            "heartbeat_interval": data.heartbeat_interval,
            "tcp_timeout": data.tcp_timeout,
            "location_interval": data.location_interval,
            "sleep_interval": data.sleep_interval,
            "speed_threshold": data.speed_threshold,
            "fatigue_threshold": data.fatigue_threshold,
            "status": "pending"
        }),
    };

    Ok(HttpResponse::Ok().json(response))
}

/// 下发GPS参数
///
/// # 功能说明
/// 设置GPS定位相关参数，包括定位模式、卫星数要求等。
#[utoipa::path(
    post,
    path = "/api/devices/terminal/gps-params",
    request_body = GpsParamsRequest,
    responses(
        (status = 200, description = "GPS参数下发成功")
    ),
    tag = "Terminal"
)]
pub async fn send_gps_params(req: web::Json<GpsParamsRequest>) -> AppResult<HttpResponse> {
    let data = req.into_inner();

    if data.vehicle_ids.is_empty() {
        return Err(AppError::bad_request("请选择至少一辆车"));
    }

    // 验证定位模式
    let valid_modes = ["gps", "beidou", "hybrid"];
    if !valid_modes.contains(&data.position_mode.as_str()) {
        return Err(AppError::bad_request("定位模式必须是 gps/beidou/hybrid 之一"));
    }

    info!(
        "[Terminal] 下发GPS参数到 {} 辆车: mode={}, min_satellites={}",
        data.vehicle_ids.len(),
        data.position_mode,
        data.min_satellites
    );

    let response = TerminalCommandResponse {
        code: 200,
        message: format!("GPS参数已下发到 {} 辆车", data.vehicle_ids.len()),
        data: serde_json::json!({
            "task_id": format!("gps_{}", Utc::now().timestamp()),
            "vehicle_count": data.vehicle_ids.len(),
            "position_mode": data.position_mode,
            "altitude_offset": data.altitude_offset,
            "min_satellites": data.min_satellites,
            "pdop_threshold": data.pdop_threshold,
            "status": "pending"
        }),
    };

    Ok(HttpResponse::Ok().json(response))
}

/// 下发通信参数
///
/// ⚠️ **危险操作**：修改通信参数可能导致终端离线！
///
/// # 功能说明
/// 设置终端的通信服务器参数和APN配置。
#[utoipa::path(
    post,
    path = "/api/devices/terminal/comm-params",
    request_body = CommParamsRequest,
    responses(
        (status = 200, description = "通信参数下发成功")
    ),
    tag = "Terminal"
)]
pub async fn send_comm_params(req: web::Json<CommParamsRequest>) -> AppResult<HttpResponse> {
    let data = req.into_inner();

    if data.vehicle_ids.is_empty() {
        return Err(AppError::bad_request("请选择至少一辆车"));
    }

    // 必填字段验证
    if data.primary_server_ip.is_empty() {
        return Err(AppError::bad_request("主服务器IP不能为空"));
    }

    // 端口范围验证
    if data.primary_server_port == 0 {
        return Err(AppError::bad_request("主服务器端口无效"));
    }

    warn!(
        "[Terminal] ⚠️ 危险操作：下发通信参数到 {} 辆车！可能导致终端离线！server={}:{}",
        data.vehicle_ids.len(),
        data.primary_server_ip,
        data.primary_server_port
    );

    let response = TerminalCommandResponse {
        code: 200,
        message: format!("⚠️ 通信参数已下发到 {} 辆车，终端可能需要重新连接！", data.vehicle_ids.len()),
        data: serde_json::json!({
            "task_id": format!("comm_{}", Utc::now().timestamp()),
            "vehicle_count": data.vehicle_ids.len(),
            "primary_server_ip": data.primary_server_ip,
            "primary_server_port": data.primary_server_port,
            "backup_server_ip": data.backup_server_ip,
            "backup_server_port": data.backup_server_port,
            "apn_name": data.apn_name,
            "has_apn_credentials": data.apn_username.is_some() && data.apn_password.is_some(),
            "status": "pending",
            "warning": "终端可能需要重新连接才能生效"
        }),
    };

    Ok(HttpResponse::Ok().json(response))
}
