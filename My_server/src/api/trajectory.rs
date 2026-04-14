//! / 轨迹查询 API
// 提供历史轨迹查询、停车记录、数据导出等功能

use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use log::{debug, info};
use sqlx::PgPool;

use crate::services::trajectory::{TrajectoryService, TrajectoryQueryParams, TrajectoryType};

/// 查询轨迹
#[utoipa::path(
    get,
    path = "/api/v1/trajectory/{device_id}",
    tag = "轨迹",
    params(
        ("device_id" = String, Path, description = "设备ID"),
        ("start_time" = String, Query, description = "开始时间 (ISO 8601)"),
        ("end_time" = String, Query, description = "结束时间 (ISO 8601)"),
        ("type" = Option<TrajectoryType>, Query, description = "轨迹类型: motion, points, full"),
        ("include_address" = Option<bool>, Query, description = "是否包含地址"),
        ("simplify" = Option<bool>, Query, description = "是否简化轨迹"),
        ("simplification_tolerance" = Option<f64>, Query, description = "简化容差(度)")
    ),
    responses(
        (status = 200, description = "查询成功", body = TrajectoryResult),
        (status = 400, description = "参数错误"),
        (status = 500, description = "服务器错误")
    )
)]
pub async fn get_trajectory(
    device_id: web::Path<String>,
    query: web::Query<TrajectoryQueryRequest>,
    pool: web::Data<PgPool>,
) -> impl Responder {
    let device_id = device_id.into_inner();

    debug!("Trajectory query request for device {}", device_id);

    // 解析时间参数
    let start_time = match chrono::DateTime::parse_from_rfc3339(&query.start_time) {
        Ok(dt) => dt.with_timezone(&chrono::Utc),
        Err(e) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid start_time",
                "message": e.to_string()
            }));
        }
    };

    let end_time = match chrono::DateTime::parse_from_rfc3339(&query.end_time) {
        Ok(dt) => dt.with_timezone(&chrono::Utc),
        Err(e) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid end_time",
                "message": e.to_string()
            }));
        }
    };

    // 构建查询参数
    let params = TrajectoryQueryParams {
        device_id: device_id.clone(),
        start_time,
        end_time,
        trajectory_type: query.r#type.unwrap_or(TrajectoryType::Full),
        include_address: query.include_address.unwrap_or(false),
        simplify: query.simplify.unwrap_or(false),
        simplification_tolerance: query.simplification_tolerance,
    };

    // 创建轨迹服务
    let tencent_api_key = std::env::var("TENCENT_MAP_API_KEY").ok();
    let service = TrajectoryService::new(pool.get_ref().clone(), tencent_api_key);

    // 查询轨迹
    match service.query_trajectory(params).await {
        Ok(result) => {
            info!("Trajectory query successful: {} points", result.points_count);
            HttpResponse::Ok().json(result)
        }
        Err(e) => {
            error!("Trajectory query failed: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Query failed",
                "message": e
            }))
        }
    }
}

/// 查询停车记录
#[utoipa::path(
    get,
    path = "/api/v1/trajectory/{device_id}/parking",
    tag = "轨迹",
    params(
        ("device_id" = String, Path, description = "设备ID"),
        ("start_time" = String, Query, description = "开始时间 (ISO 8601)"),
        ("end_time" = String, Query, description = "结束时间 (ISO 8601)")
    ),
    responses(
        (status = 200, description = "查询成功", body = Vec<ParkingRecord>),
        (status = 400, description = "参数错误"),
        (status = 500, description = "服务器错误")
    )
)]
pub async fn get_parking_records(
    device_id: web::Path<String>,
    query: web::Query<TimeRangeRequest>,
    pool: web::Data<PgPool>,
) -> impl Responder {
    let device_id = device_id.into_inner();

    let start_time = parse_time(&query.start_time)?;
    let end_time = parse_time(&query.end_time)?;

    let service = TrajectoryService::new(pool.get_ref().clone(), None);

    match service.query_parking_records(&device_id, start_time, end_time).await {
        Ok(records) => {
            info!("Found {} parking records for device {}", records.len(), device_id);
            HttpResponse::Ok().json(records)
        }
        Err(e) => {
            error!("Parking records query failed: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Query failed",
                "message": e
            }))
        }
    }
}

/// 查询停车未熄火记录
#[utoipa::path(
    get,
    path = "/api/v1/trajectory/{device_id}/no_shutdown",
    tag = "轨迹",
    params(
        ("device_id" = String, Path, description = "设备ID"),
        ("start_time" = String, Query, description = "开始时间 (ISO 8601)"),
        ("end_time" = String, Query, description = "结束时间 (ISO 8601)")
    ),
    responses(
        (status = 200, description = "查询成功", body = Vec<ParkingRecord>),
        (status = 400, description = "参数错误"),
        (status = 500, description = "服务器错误")
    )
)]
pub async fn get_no_shutdown_records(
    device_id: web::Path<String>,
    query: web::Query<TimeRangeRequest>,
    pool: web::Data<PgPool>,
) -> impl Responder {
    let device_id = device_id.into_inner();

    let start_time = parse_time(&query.start_time)?;
    let end_time = parse_time(&query.end_time)?;

    let service = TrajectoryService::new(pool.get_ref().clone(), None);

    match service.query_no_shutdown_records(&device_id, start_time, end_time).await {
        Ok(records) => {
            info!("Found {} no-shutdown records for device {}", records.len(), device_id);
            HttpResponse::Ok().json(records)
        }
        Err(e) => {
            error!("No-shutdown records query failed: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Query failed",
                "message": e
            }))
        }
    }
}

/// 清除轨迹缓存
#[utoipa::path(
    delete,
    path = "/api/v1/trajectory/{device_id}/cache",
    tag = "轨迹",
    params(
        ("device_id" = String, Path, description = "设备ID")
    ),
    responses(
        (status = 200, description = "清除成功"),
        (status = 500, description = "服务器错误")
    )
)]
pub async fn clear_trajectory_cache(
    device_id: web::Path<String>,
) -> impl Responder {
    let device_id = device_id.into_inner();

    info!("Clearing trajectory cache for device {}", device_id);

    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": "Trajectory cache cleared (to be implemented)"
    }))
}

/// 导出轨迹数据
#[utoipa::path(
    get,
    path = "/api/v1/trajectory/{device_id}/export",
    tag = "轨迹",
    params(
        ("device_id" = String, Path, description = "设备ID"),
        ("start_time" = String, Query, description = "开始时间 (ISO 8601)"),
        ("end_time" = String, Query, description = "结束时间 (ISO 8601)"),
        ("format" = String, Query, description = "导出格式: csv, excel, kml, json")
    ),
    responses(
        (status = 200, description = "导出成功", content_type = "application/octet-stream"),
        (status = 400, description = "参数错误"),
        (status = 500, description = "服务器错误")
    )
)]
pub async fn export_trajectory(
    device_id: web::Path<String>,
    query: web::Query<ExportRequest>,
    pool: web::Data<PgPool>,
) -> impl Responder {
    let device_id = device_id.into_inner();

    let start_time = parse_time(&query.start_time)?;
    let end_time = parse_time(&query.end_time)?;

    let service = TrajectoryService::new(pool.get_ref().clone(), None);

    match service.export_trajectory(&device_id, start_time, end_time, &query.format).await {
        Ok(data) => {
            let content_type = match query.format.to_lowercase().as_str() {
                "csv" => "text/csv",
                "excel" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
                "kml" => "application/vnd.google-earth.kml+xml",
                "json" => "application/json",
                _ => "application/octet-stream",
            };

            let filename = format!("trajectory_{}_{}_{:?}.{}",
                device_id,
                chrono::Utc::now().format("%Y%m%d"),
                query.format,
                query.format);

            HttpResponse::Ok()
                .content_type(content_type)
                .insert_header(("Content-Disposition", format!("attachment; filename=\"{}\"", filename)))
                .body(data)
        }
        Err(e) => {
            error!("Trajectory export failed: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Export failed",
                "message": e
            }))
        }
    }
}

/// 请求类型
#[derive(Debug, Deserialize)]
pub struct TrajectoryQueryRequest {
    pub start_time: String,
    pub end_time: String,
    #[serde(rename = "type")]
    pub r#type: Option<TrajectoryType>,
    pub include_address: Option<bool>,
    pub simplify: Option<bool>,
    pub simplification_tolerance: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct TimeRangeRequest {
    pub start_time: String,
    pub end_time: String,
}

#[derive(Debug, Deserialize)]
pub struct ExportRequest {
    pub start_time: String,
    pub end_time: String,
    pub format: String,
}

/// 解析时间字符串
fn parse_time(time_str: &str) -> Result<chrono::DateTime<chrono::Utc>, actix_web::HttpResponse> {
    chrono::DateTime::parse_from_rfc3339(time_str)
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .map_err(|e| {
            actix_web::HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid time format",
                "message": e.to_string()
            }))
        })
}

/// 响应类型
#[derive(Debug, Serialize)]
pub struct TrajectoryResult {
    pub device_id: String,
    pub trajectory_type: TrajectoryType,
    pub start_time: String,
    pub end_time: String,
    pub total_distance: f64,
    pub total_duration: i64,
    pub points_count: usize,
    pub points: Vec<crate::infrastructure::cache::TrajectoryPoint>,
    pub parking_records: Vec<crate::services::trajectory::ParkingRecord>,
}

#[derive(Debug, Serialize)]
pub struct ParkingRecord {
    pub device_id: String,
    pub start_time: String,
    pub end_time: String,
    pub duration: i64,
    pub latitude: f64,
    pub longitude: f64,
    pub address: Option<String>,
    pub engine_off: bool,
    pub parking_duration: Option<i32>,
}

/// 配置轨迹路由
pub fn configure_trajectory_routes(cfg: &mut web::ServiceConfig) {
    cfg
        .service(get_trajectory)
        .service(get_parking_records)
        .service(get_no_shutdown_records)
        .service(clear_trajectory_cache)
        .service(export_trajectory);
}






