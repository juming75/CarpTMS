use actix_web::{web, HttpResponse};
use log::{info, warn};
use validator::Validate;

use crate::application::services::calibration_service::{CalibrationService, CalibrationServiceImpl};
use crate::domain::entities::calibration::SensorCalibration;
use crate::errors::{created_response, success_response, AppError, AppResult};
use crate::redis::{del_cache_pattern, get_cache, set_cache};
use crate::schemas::{CalibrationCreate, CalibrationQuery, CalibrationResponse, CalibrationHistoryResponse, CalibrationUpdate, PagedResponse};

// 获取标定数据列表(支持分页和筛选)
#[utoipa::path(
    path = "/api/weight-calibrations",
    get,
    responses(
        (status = 200, description = "Calibrations fetched successfully", body = ApiResponse<PagedResponse<CalibrationResponse>>),
        (status = 500, description = "Internal server error", body = ApiResponse<()>)
    )
)]
pub async fn get_calibrations(
    calibration_service: web::Data<CalibrationServiceImpl>,
    query: web::Query<CalibrationQuery>,
) -> AppResult<HttpResponse> {
    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20);

    // 生成缓存键
    let cache_key = format!(
        "calibrations:list:sensor_no_{}:vehicle_id_{}:plate_no_{}:page_{}:size_{}",
        query.sensor_no.map(|s| s.to_string()).unwrap_or("".to_string()),
        query.vehicle_id.map(|v| v.to_string()).unwrap_or("".to_string()),
        query.plate_no.as_deref().unwrap_or(""),
        page,
        page_size
    );

    // 尝试从缓存获取
    if let Ok(Some(cached_response)) = get_cache::<PagedResponse<CalibrationResponse>>(&cache_key).await
    {
        return Ok(success_response(Some(cached_response)));
    }

    // 从服务获取标定数据
    let (calibrations, total) = calibration_service
        .get_calibrations(
            page,
            page_size,
            query.sensor_no,
            query.vehicle_id,
            query.plate_no.as_deref(),
        )
        .await?;

    // 转换为响应格式
    let calibration_responses: Vec<CalibrationResponse> = calibrations
        .into_iter()
        .map(|calibration| {
            // 转换 calibration_points 为 serde_json::Value
            let calibration_points_value = calibration.calibration_points.as_ref().and_then(|s| serde_json::from_str(s).ok());
            
            CalibrationResponse {
                id: calibration.id,
                sensor_no: calibration.sensor_no,
                vehicle_id: calibration.vehicle_id,
                plate_no: calibration.plate_no,
                sensor_side: calibration.sensor_side,
                sensor_group: calibration.sensor_group,
                self_weight: calibration.self_weight,
                polynomial_json: calibration.polynomial_json,
                linear_segments_json: calibration.linear_segments_json,
                is_calibrated: calibration.is_calibrated,
                create_time: calibration.create_time,
                update_time: calibration.update_time,
                calibration_points: calibration_points_value,
                pa_raw: calibration.pa_raw.map(|v| v as i32),
                axle_number: calibration.axle_number,
                is_left_wheel: calibration.is_left_wheel,
                turn_point: calibration.turn_point.map(|v| v as i32),
                polynomial_order: calibration.polynomial_order,
                polynomial_coefs_json: None, // SensorCalibration 中没有这个字段
                r2_score: calibration.r2_score,
                rmse: calibration.rmse,
                max_error: calibration.max_error,
                point_count: calibration.point_count,
                rated_total_weight: calibration.rated_total_weight,
                tare_weight: calibration.tare_weight,
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
        list: calibration_responses,
        total,
        page,
        page_size,
        pages: pages as i32,
    };

    // 缓存结果,过期时间30分钟
    let _ = set_cache(&cache_key, &paged_response, 1800).await;

    Ok(success_response(Some(paged_response)))
}

// 创建标定数据
#[utoipa::path(
    path = "/api/weight-calibrations",
    post,
    request_body = CalibrationCreate,
    responses(
        (status = 201, description = "Calibration created successfully", body = ApiResponse<CalibrationResponse>),
        (status = 400, description = "Invalid request parameters", body = ApiResponse<()>),
        (status = 500, description = "Internal server error", body = ApiResponse<()>)
    )
)]
pub async fn create_calibration(
    calibration_service: web::Data<CalibrationServiceImpl>,
    calibration: web::Json<CalibrationCreate>,
) -> AppResult<HttpResponse> {
    info!("Creating calibration for sensor: {}", calibration.sensor_no);

    // 验证请求数据
    calibration.validate()?;

    // 创建传感器标定实体
    let calibration_entity = SensorCalibration::new(
        calibration.sensor_no,
        calibration.vehicle_id,
        calibration.plate_no.clone(),
        calibration.sensor_side.clone(),
        calibration.sensor_group,
        calibration.self_weight,
        Some(calibration.polynomial_json.clone()),
        calibration.linear_segments_json.clone(),
        calibration.is_calibrated,
    );

    // 调用服务创建标定数据
    let created_calibration = calibration_service
        .create_calibration(&calibration_entity)
        .await?;

    // 转换为响应格式
    let calibration_points_value = created_calibration.calibration_points.as_ref().and_then(|s| serde_json::from_str(s).ok());
    let response = CalibrationResponse {
        id: created_calibration.id,
        sensor_no: created_calibration.sensor_no,
        vehicle_id: created_calibration.vehicle_id,
        plate_no: created_calibration.plate_no,
        sensor_side: created_calibration.sensor_side,
        sensor_group: created_calibration.sensor_group,
        self_weight: created_calibration.self_weight,
        polynomial_json: created_calibration.polynomial_json,
        linear_segments_json: created_calibration.linear_segments_json,
        is_calibrated: created_calibration.is_calibrated,
        create_time: created_calibration.create_time,
        update_time: created_calibration.update_time,
        calibration_points: calibration_points_value,
        pa_raw: created_calibration.pa_raw.map(|v| v as i32),
        axle_number: created_calibration.axle_number,
        is_left_wheel: created_calibration.is_left_wheel,
        turn_point: created_calibration.turn_point.map(|v| v as i32),
        polynomial_order: created_calibration.polynomial_order,
        polynomial_coefs_json: None,
        r2_score: created_calibration.r2_score,
        rmse: created_calibration.rmse,
        max_error: created_calibration.max_error,
        point_count: created_calibration.point_count,
        rated_total_weight: created_calibration.rated_total_weight,
        tare_weight: created_calibration.tare_weight,
    };

    // 清理相关缓存
    let _ = del_cache_pattern("calibrations:list:*").await;

    info!("Calibration created successfully for sensor: {}", calibration.sensor_no);
    Ok(created_response(Some(response)))
}

// 获取标定数据详情
#[utoipa::path(
    path = "/api/weight-calibrations/{id}",
    get,
    responses(
        (status = 200, description = "Calibration fetched successfully", body = ApiResponse<CalibrationResponse>),
        (status = 404, description = "Calibration not found", body = ApiResponse<()>),
        (status = 500, description = "Internal server error", body = ApiResponse<()>)
    )
)]
pub async fn get_calibration(
    calibration_service: web::Data<CalibrationServiceImpl>,
    id: web::Path<i32>,
) -> AppResult<HttpResponse> {
    let id = id.into_inner();

    // 尝试从缓存获取
    let cache_key = format!("calibration:{}:details", id);
    if let Ok(Some(cached_response)) = get_cache::<CalibrationResponse>(&cache_key).await {
        return Ok(success_response(Some(cached_response)));
    }

    // 从服务获取标定数据详情
    let calibration = calibration_service.get_calibration(id).await?;

    match calibration {
        Some(calibration_entity) => {
            let calibration_points_value = calibration_entity.calibration_points.as_ref().and_then(|s| serde_json::from_str(s).ok());
            let response = CalibrationResponse {
                id: calibration_entity.id,
                sensor_no: calibration_entity.sensor_no,
                vehicle_id: calibration_entity.vehicle_id,
                plate_no: calibration_entity.plate_no,
                sensor_side: calibration_entity.sensor_side,
                sensor_group: calibration_entity.sensor_group,
                self_weight: calibration_entity.self_weight,
                polynomial_json: calibration_entity.polynomial_json,
                linear_segments_json: calibration_entity.linear_segments_json,
                is_calibrated: calibration_entity.is_calibrated,
                create_time: calibration_entity.create_time,
                update_time: calibration_entity.update_time,
                calibration_points: calibration_points_value,
                pa_raw: calibration_entity.pa_raw.map(|v| v as i32),
                axle_number: calibration_entity.axle_number,
                is_left_wheel: calibration_entity.is_left_wheel,
                turn_point: calibration_entity.turn_point.map(|v| v as i32),
                polynomial_order: calibration_entity.polynomial_order,
                polynomial_coefs_json: None,
                r2_score: calibration_entity.r2_score,
                rmse: calibration_entity.rmse,
                max_error: calibration_entity.max_error,
                point_count: calibration_entity.point_count,
                rated_total_weight: calibration_entity.rated_total_weight,
                tare_weight: calibration_entity.tare_weight,
            };

            // 缓存结果,过期时间30分钟
            let _ = set_cache(&cache_key, &response, 1800).await;

            Ok(success_response(Some(response)))
        }
        None => {
            warn!("Calibration not found: {}", id);
            Err(AppError::not_found_error("Calibration not found".to_string()))
        }
    }
}

// 更新标定数据
#[utoipa::path(
    path = "/api/weight-calibrations/{id}",
    put,
    request_body = CalibrationUpdate,
    responses(
        (status = 200, description = "Calibration updated successfully", body = ApiResponse<CalibrationResponse>),
        (status = 400, description = "Invalid request parameters", body = ApiResponse<()>),
        (status = 404, description = "Calibration not found", body = ApiResponse<()>),
        (status = 500, description = "Internal server error", body = ApiResponse<()>)
    )
)]
pub async fn update_calibration(
    calibration_service: web::Data<CalibrationServiceImpl>,
    id: web::Path<i32>,
    calibration: web::Json<CalibrationUpdate>,
) -> AppResult<HttpResponse> {
    let id = id.into_inner();

    // 获取现有标定数据
    let existing_calibration = calibration_service.get_calibration(id).await?;
    let mut calibration_entity = existing_calibration
        .ok_or_else(|| AppError::not_found_error("Calibration not found".to_string()))?;

    // 更新标定数据
    if let Some(vehicle_id) = calibration.vehicle_id {
        calibration_entity.vehicle_id = vehicle_id;
    }
    if let Some(plate_no) = &calibration.plate_no {
        calibration_entity.plate_no = plate_no.clone();
    }
    if let Some(sensor_side) = &calibration.sensor_side {
        calibration_entity.sensor_side = sensor_side.clone();
    }
    if let Some(sensor_group) = calibration.sensor_group {
        calibration_entity.sensor_group = Some(sensor_group);
    }
    if let Some(self_weight) = calibration.self_weight {
        calibration_entity.self_weight = Some(self_weight);
    }
    if let Some(polynomial_json) = &calibration.polynomial_json {
        calibration_entity.polynomial_json = Some(polynomial_json.clone());
    }
    if let Some(linear_segments_json) = &calibration.linear_segments_json {
        calibration_entity.linear_segments_json = Some(linear_segments_json.clone());
    }
    if let Some(is_calibrated) = calibration.is_calibrated {
        calibration_entity.is_calibrated = is_calibrated;
    }

    // 调用服务更新标定数据
    let updated_calibration = calibration_service
        .update_calibration(id, &calibration_entity)
        .await?;

    // 转换为响应格式
    let calibration_points_value = updated_calibration.calibration_points.as_ref().and_then(|s| serde_json::from_str(s).ok());
    let response = CalibrationResponse {
        id: updated_calibration.id,
        sensor_no: updated_calibration.sensor_no,
        vehicle_id: updated_calibration.vehicle_id,
        plate_no: updated_calibration.plate_no,
        sensor_side: updated_calibration.sensor_side,
        sensor_group: updated_calibration.sensor_group,
        self_weight: updated_calibration.self_weight,
        polynomial_json: updated_calibration.polynomial_json,
        linear_segments_json: updated_calibration.linear_segments_json,
        is_calibrated: updated_calibration.is_calibrated,
        create_time: updated_calibration.create_time,
        update_time: updated_calibration.update_time,
        calibration_points: calibration_points_value,
        pa_raw: updated_calibration.pa_raw.map(|v| v as i32),
        axle_number: updated_calibration.axle_number,
        is_left_wheel: updated_calibration.is_left_wheel,
        turn_point: updated_calibration.turn_point.map(|v| v as i32),
        polynomial_order: updated_calibration.polynomial_order,
        polynomial_coefs_json: None,
        r2_score: updated_calibration.r2_score,
        rmse: updated_calibration.rmse,
        max_error: updated_calibration.max_error,
        point_count: updated_calibration.point_count,
        rated_total_weight: updated_calibration.rated_total_weight,
        tare_weight: updated_calibration.tare_weight,
    };

    // 清理相关缓存
    let _ = del_cache_pattern("calibration:{}:*").await;
    let _ = del_cache_pattern("calibrations:list:*").await;

    info!("Calibration updated successfully: {}", id);
    Ok(success_response(Some(response)))
}

// 删除标定数据
#[utoipa::path(
    path = "/api/weight-calibrations/{id}",
    delete,
    responses(
        (status = 200, description = "Calibration deleted successfully", body = ApiResponse<()>),
        (status = 404, description = "Calibration not found", body = ApiResponse<()>),
        (status = 500, description = "Internal server error", body = ApiResponse<()>)
    )
)]
pub async fn delete_calibration(
    calibration_service: web::Data<CalibrationServiceImpl>,
    id: web::Path<i32>,
) -> AppResult<HttpResponse> {
    let id = id.into_inner();

    // 调用服务删除标定数据
    calibration_service.delete_calibration(id).await?;

    // 清理相关缓存
    let _ = del_cache_pattern("calibration:{}:*").await;
    let _ = del_cache_pattern("calibrations:list:*").await;

    info!("Calibration deleted successfully: {}", id);
    Ok(success_response(()))
}

// 获取标定历史记录
#[utoipa::path(
    path = "/api/weight-calibrations/history",
    get,
    responses(
        (status = 200, description = "Calibration history fetched successfully", body = ApiResponse<PagedResponse<CalibrationHistoryResponse>>),
        (status = 500, description = "Internal server error", body = ApiResponse<()>)
    )
)]
pub async fn get_calibration_history(
    calibration_service: web::Data<CalibrationServiceImpl>,
    query: web::Query<CalibrationQuery>,
) -> AppResult<HttpResponse> {
    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20);

    // 从服务获取标定历史记录
    let (history_records, total) = calibration_service
        .get_calibration_history(
            page,
            page_size,
            query.sensor_no,
            query.vehicle_id,
            query.plate_no.as_deref(),
        )
        .await?;

    // 转换为响应格式
    let history_responses: Vec<CalibrationHistoryResponse> = history_records
        .into_iter()
        .map(|history| CalibrationHistoryResponse {
            id: history.id,
            sensor_no: history.sensor_no,
            vehicle_id: history.vehicle_id,
            plate_no: history.plate_no,
            polynomial_json: history.polynomial_json,
            polynomial_order: history.polynomial_order,
            r2_score: history.r2_score,
            rmse: history.rmse,
            max_error: history.max_error,
            point_count: history.point_count,
            operation_type: history.operation_type,
            operation_type_name: history.operation_type_name,
            operator: history.operator,
            remark: history.remark,
            is_valid: history.is_valid,
            create_time: history.create_time,
            update_time: history.update_time,
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
        list: history_responses,
        total,
        page,
        page_size,
        pages: pages as i32,
    };

    Ok(success_response(Some(paged_response)))
}

// 配置路由
pub fn configure_weight_calibration_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/weight-calibrations")
            .route("", web::get().to(get_calibrations))
            .route("", web::post().to(create_calibration))
            .route("/history", web::get().to(get_calibration_history))
            .route("/{id}", web::get().to(get_calibration))
            .route("/{id}", web::put().to(update_calibration))
            .route("/{id}", web::delete().to(delete_calibration)),
    );
}
