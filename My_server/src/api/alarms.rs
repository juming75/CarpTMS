//! / 报警 API
// 提供报警数据的查询、处理和统计功能

use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use log::{debug, info};
use sqlx::PgPool;

use crate::services::alarm::{AlarmService, AlarmQueryParams, AlarmStatus};

/// 查询报警列表
#[utoipa::path(
    get,
    path = "/api/v1/alarms",
    tag = "报警",
    params(
        ("device_id" = Option<String>, Query, description = "设备ID"),
        ("alarm_type" = Option<String>, Query, description = "报警类型"),
        ("alarm_level" = Option<i32>, Query, description = "报警级别: 0-提示, 1-警告, 2-高危, 3-紧急"),
        ("status" = Option<String>, Query, description = "状态: pending, handled, ignored"),
        ("start_time" = Option<String>, Query, description = "开始时间 (ISO 8601)"),
        ("end_time" = Option<String>, Query, description = "结束时间 (ISO 8601)"),
        ("limit" = Option<usize>, Query, description = "返回数量限制"),
        ("offset" = Option<usize>, Query, description = "偏移量")
    ),
    responses(
        (status = 200, description = "查询成功", body = Vec<AlarmRecord>),
        (status = 500, description = "服务器错误")
    )
)]
pub async fn get_alarms(
    query: web::Query<AlarmQueryRequest>,
    pool: web::Data<PgPool>,
) -> impl Responder {
    debug!("Alarm query request: {:?}", query);

    let params = AlarmQueryParams {
        device_id: query.device_id.clone(),
        alarm_type: query.alarm_type.clone(),
        alarm_level: query.alarm_level.map(|l| l.into()),
        status: query.status.as_ref().and_then(|s| parse_alarm_status(s)),
        start_time: query.start_time.as_ref().and_then(|t| parse_time(t).ok()),
        end_time: query.end_time.as_ref().and_then(|t| parse_time(t).ok()),
        limit: query.limit,
        offset: query.offset,
    };

    let service = AlarmService::new(pool.get_ref().clone());

    match service.query_alarms(params).await {
        Ok(alarms) => {
            info!("Found {} alarms", alarms.len());
            HttpResponse::Ok().json(alarms)
        }
        Err(e) => {
            error!("Alarm query failed: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Query failed",
                "message": e
            }))
        }
    }
}

/// 获取报警详情
#[utoipa::path(
    get,
    path = "/api/v1/alarms/{alarm_id}",
    tag = "报警",
    params(
        ("alarm_id" = i64, Path, description = "报警ID")
    ),
    responses(
        (status = 200, description = "获取成功", body = AlarmRecord),
        (status = 404, description = "报警不存在"),
        (status = 500, description = "服务器错误")
    )
)]
pub async fn get_alarm_detail(
    alarm_id: web::Path<i64>,
    pool: web::Data<PgPool>,
) -> impl Responder {
    let alarm_id = alarm_id.into_inner();

    debug!("Querying alarm detail for ID: {}", alarm_id);

    let service = AlarmService::new(pool.get_ref().clone());

    match service.get_alarm(alarm_id).await {
        Ok(Some(alarm)) => {
            HttpResponse::Ok().json(alarm)
        }
        Ok(None) => {
            HttpResponse::NotFound().json(serde_json::json!({
                "error": "Not found",
                "message": format!("Alarm {} not found", alarm_id)
            }))
        }
        Err(e) => {
            error!("Alarm detail query failed: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Query failed",
                "message": e
            }))
        }
    }
}

/// 获取报警统计
#[utoipa::path(
    get,
    path = "/api/v1/alarms/statistics",
    tag = "报警",
    responses(
        (status = 200, description = "获取成功", body = AlarmStatistics),
        (status = 500, description = "服务器错误")
    )
)]
pub async fn get_alarm_statistics(
    query: web::Query<AlarmQueryRequest>,
    pool: web::Data<PgPool>,
) -> impl Responder {
    debug!("Alarm statistics request");

    let params = AlarmQueryParams {
        device_id: query.device_id.clone(),
        alarm_type: query.alarm_type.clone(),
        alarm_level: query.alarm_level.map(|l| l.into()),
        status: query.status.as_ref().and_then(|s| parse_alarm_status(s)),
        start_time: query.start_time.as_ref().and_then(|t| parse_time(t).ok()),
        end_time: query.end_time.as_ref().and_then(|t| parse_time(t).ok()),
        limit: None,
        offset: None,
    };

    let service = AlarmService::new(pool.get_ref().clone());

    match service.get_statistics(params).await {
        Ok(stats) => {
            HttpResponse::Ok().json(stats)
        }
        Err(e) => {
            error!("Alarm statistics query failed: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Query failed",
                "message": e
            }))
        }
    }
}

/// 处理报警
#[utoipa::path(
    post,
    path = "/api/v1/alarms/{alarm_id}/handle",
    tag = "报警",
    params(
        ("alarm_id" = i64, Path, description = "报警ID")
    ),
    request_body = HandleAlarmRequest,
    responses(
        (status = 200, description = "处理成功"),
        (status = 404, description = "报警不存在"),
        (status = 500, description = "服务器错误")
    )
)]
pub async fn handle_alarm(
    alarm_id: web::Path<i64>,
    request: web::Json<HandleAlarmRequest>,
    pool: web::Data<PgPool>,
) -> impl Responder {
    let alarm_id = alarm_id.into_inner();
    let handled_by = request.handled_by.clone();

    info!("Handling alarm {}: handled_by={}", alarm_id, handled_by);

    let service = AlarmService::new(pool.get_ref().clone());

    match service.handle_alarm(alarm_id, handled_by).await {
        Ok(()) => {
            HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "message": "Alarm handled successfully"
            }))
        }
        Err(e) => {
            error!("Alarm handle failed: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Handle failed",
                "message": e
            }))
        }
    }
}

/// 忽略报警
#[utoipa::path(
    post,
    path = "/api/v1/alarms/{alarm_id}/ignore",
    tag = "报警",
    params(
        ("alarm_id" = i64, Path, description = "报警ID")
    ),
    responses(
        (status = 200, description = "忽略成功"),
        (status = 404, description = "报警不存在"),
        (status = 500, description = "服务器错误")
    )
)]
pub async fn ignore_alarm(
    alarm_id: web::Path<i64>,
    pool: web::Data<PgPool>,
) -> impl Responder {
    let alarm_id = alarm_id.into_inner();

    info!("Ignoring alarm {}", alarm_id);

    let service = AlarmService::new(pool.get_ref().clone());

    match service.ignore_alarm(alarm_id).await {
        Ok(()) => {
            HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "message": "Alarm ignored successfully"
            }))
        }
        Err(e) => {
            error!("Alarm ignore failed: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Ignore failed",
                "message": e
            }))
        }
    }
}

/// 请求类型
#[derive(Debug, Deserialize)]
pub struct AlarmQueryRequest {
    pub device_id: Option<String>,
    pub alarm_type: Option<String>,
    pub alarm_level: Option<i32>,
    pub status: Option<String>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct HandleAlarmRequest {
    pub handled_by: String,
    pub note: Option<String>,
}

/// 响应类型
#[derive(Debug, Serialize)]
pub struct AlarmRecord {
    pub id: Option<i64>,
    pub device_id: String,
    pub phone: Option<String>,
    pub alarm_type: String,
    pub alarm_level: i32,
    pub alarm_time: String,
    pub location: Option<serde_json::Value>,
    pub description: Option<String>,
    pub status: String,
    pub handled_by: Option<String>,
    pub handled_at: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AlarmStatistics {
    pub total_count: i64,
    pub pending_count: i64,
    pub handled_count: i64,
    pub ignored_count: i64,
    pub by_level: std::collections::HashMap<String, i64>,
    pub by_type: std::collections::HashMap<String, i64>,
}

/// 解析报警状态
fn parse_alarm_status(status: &str) -> Option<AlarmStatus> {
    match status.to_lowercase().as_str() {
        "pending" => Some(AlarmStatus::Pending),
        "handled" => Some(AlarmStatus::Handled),
        "ignored" => Some(AlarmStatus::Ignored),
        _ => None,
    }
}

/// 解析时间字符串
fn parse_time(time_str: &str) -> Result<chrono::DateTime<chrono::Utc>, String> {
    chrono::DateTime::parse_from_rfc3339(time_str)
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .map_err(|e| format!("Invalid time format: {}", e))
}

/// 配置报警路由
pub fn configure_alarm_routes(cfg: &mut web::ServiceConfig) {
    cfg
        .service(get_alarms)
        .service(get_alarm_detail)
        .service(get_alarm_statistics)
        .service(handle_alarm)
        .service(ignore_alarm);
}






