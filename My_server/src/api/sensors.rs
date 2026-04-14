//! / 传感器 API
// 提供传感器数据的查询和统计功能

use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use log::{debug, info};
use sqlx::PgPool;

use crate::services::sensor::{SensorService, SensorQueryParams};

/// 查询传感器数据
#[utoipa::path(
    get,
    path = "/api/v1/sensors",
    tag = "传感器",
    params(
        ("device_id" = Option<String>, Query, description = "设备ID"),
        ("sensor_type" = Option<String>, Query, description = "传感器类型"),
        ("start_time" = Option<String>, Query, description = "开始时间 (ISO 8601)"),
        ("end_time" = Option<String>, Query, description = "结束时间 (ISO 8601)"),
        ("limit" = Option<usize>, Query, description = "返回数量限制"),
        ("offset" = Option<usize>, Query, description = "偏移量")
    ),
    responses(
        (status = 200, description = "查询成功", body = Vec<SensorData>),
        (status = 500, description = "服务器错误")
    )
)]
pub async fn get_sensor_data(
    query: web::Query<SensorQueryRequest>,
    pool: web::Data<PgPool>,
) -> impl Responder {
    debug!("Sensor data query request: {:?}", query);

    let params = SensorQueryParams {
        device_id: query.device_id.clone(),
        sensor_type: query.sensor_type.clone(),
        start_time: query.start_time.as_ref().and_then(|t| parse_time(t).ok()),
        end_time: query.end_time.as_ref().and_then(|t| parse_time(t).ok()),
        limit: query.limit,
        offset: query.offset,
    };

    let service = SensorService::new(pool.get_ref().clone());

    match service.query_sensor_data(params).await {
        Ok(data) => {
            info!("Found {} sensor records", data.len());
            HttpResponse::Ok().json(data)
        }
        Err(e) => {
            error!("Sensor data query failed: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Query failed",
                "message": e
            }))
        }
    }
}

/// 获取传感器统计
#[utoipa::path(
    get,
    path = "/api/v1/sensors/statistics",
    tag = "传感器",
    params(
        ("device_id" = Option<String>, Query, description = "设备ID"),
        ("sensor_type" = Option<String>, Query, description = "传感器类型")
    ),
    responses(
        (status = 200, description = "获取成功", body = Vec<SensorStatistics>),
        (status = 500, description = "服务器错误")
    )
)]
pub async fn get_sensor_statistics(
    query: web::Query<SensorQueryRequest>,
    pool: web::Data<PgPool>,
) -> impl Responder {
    debug!("Sensor statistics request");

    let params = SensorQueryParams {
        device_id: query.device_id.clone(),
        sensor_type: query.sensor_type.clone(),
        start_time: None,
        end_time: None,
        limit: None,
        offset: None,
    };

    let service = SensorService::new(pool.get_ref().clone());

    match service.get_statistics(params).await {
        Ok(stats) => {
            HttpResponse::Ok().json(stats)
        }
        Err(e) => {
            error!("Sensor statistics query failed: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Query failed",
                "message": e
            }))
        }
    }
}

/// 获取传感器异常数据
#[utoipa::path(
    get,
    path = "/api/v1/sensors/anomalies",
    tag = "传感器",
    params(
        ("device_id" = Option<String>, Query, description = "设备ID"),
        ("sensor_type" = Option<String>, Query, description = "传感器类型"),
        ("limit" = Option<usize>, Query, description = "返回数量限制")
    ),
    responses(
        (status = 200, description = "获取成功", body = Vec<SensorData>),
        (status = 500, description = "服务器错误")
    )
)]
pub async fn get_sensor_anomalies(
    query: web::Query<SensorQueryRequest>,
    pool: web::Data<PgPool>,
) -> impl Responder {
    debug!("Sensor anomalies request");

    let params = SensorQueryParams {
        device_id: query.device_id.clone(),
        sensor_type: query.sensor_type.clone(),
        start_time: None,
        end_time: None,
        limit: query.limit,
        offset: None,
    };

    let service = SensorService::new(pool.get_ref().clone());

    match service.get_anomalies(params).await {
        Ok(anomalies) => {
            info!("Found {} sensor anomalies", anomalies.len());
            HttpResponse::Ok().json(anomalies)
        }
        Err(e) => {
            error!("Sensor anomalies query failed: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Query failed",
                "message": e
            }))
        }
    }
}

/// 请求类型
#[derive(Debug, Deserialize)]
pub struct SensorQueryRequest {
    pub device_id: Option<String>,
    pub sensor_type: Option<String>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

/// 响应类型
#[derive(Debug, Serialize)]
pub struct SensorData {
    pub id: Option<i64>,
    pub device_id: String,
    pub sensor_type: String,
    pub sensor_value: f64,
    pub unit: Option<String>,
    pub timestamp: String,
    pub metadata: Option<serde_json::Value>,
    pub is_anomaly: bool,
}

#[derive(Debug, Serialize)]
pub struct SensorStatistics {
    pub sensor_type: String,
    pub device_id: String,
    pub count: i64,
    pub min_value: f64,
    pub max_value: f64,
    pub avg_value: f64,
    pub total_value: f64,
    pub anomaly_count: i64,
}

/// 解析时间字符串
fn parse_time(time_str: &str) -> Result<chrono::DateTime<chrono::Utc>, String> {
    chrono::DateTime::parse_from_rfc3339(time_str)
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .map_err(|e| format!("Invalid time format: {}", e))
}

/// 配置传感器路由
pub fn configure_sensor_routes(cfg: &mut web::ServiceConfig) {
    cfg
        .service(get_sensor_data)
        .service(get_sensor_statistics)
        .service(get_sensor_anomalies);
}






