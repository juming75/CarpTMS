//! / 实时数据缓存 API
// 提供设备实时数据查询接口

use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use log::{debug, info};
use actix::Addr;

use crate::infrastructure::cache::{RealtimeCache, GetCache, ClearDeviceCache, CacheStats};

/// 获取设备实时位置
#[utoipa::path(
    get,
    path = "/api/v1/cache/devices/{device_id}/location",
    tag = "缓存",
    responses(
        (status = 200, description = "获取成功", body = LocationResponse),
        (status = 404, description = "数据不存在"),
        (status = 500, description = "服务器错误")
    ),
    params(
        ("device_id" = String, Path, description = "设备ID")
    )
)]
pub async fn get_device_location(
    device_id: web::Path<String>,
    cache: web::Data<Addr<RealtimeCache>>,
) -> impl Responder {
    let device_id = device_id.into_inner();
    
    debug!("Querying location cache for device {}", device_id);

    match cache.send(GetCache {
        key: crate::infrastructure::cache::CacheKey::DeviceLocation(device_id.clone()),
    }).await {
        Ok(Some(crate::infrastructure::cache::CacheValue::Location(location))) => {
            HttpResponse::Ok().json(location)
        }
        Ok(Some(_)) => {
            HttpResponse::NotFound().json(serde_json::json!({
                "error": "Invalid cache type",
                "message": "Expected location data"
            }))
        }
        Ok(None) => {
            HttpResponse::NotFound().json(serde_json::json!({
                "error": "Not found",
                "message": format!("Location data not found for device {}", device_id)
            }))
        }
        Err(e) => {
            error!("Failed to query location cache: {:?}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Cache error",
                "message": e.to_string()
            }))
        }
    }
}

/// 获取设备状态
#[utoipa::path(
    get,
    path = "/api/v1/cache/devices/{device_id}/status",
    tag = "缓存",
    responses(
        (status = 200, description = "获取成功", body = DeviceStatusResponse),
        (status = 404, description = "数据不存在"),
        (status = 500, description = "服务器错误")
    ),
    params(
        ("device_id" = String, Path, description = "设备ID")
    )
)]
pub async fn get_device_status(
    device_id: web::Path<String>,
    cache: web::Data<Addr<RealtimeCache>>,
) -> impl Responder {
    let device_id = device_id.into_inner();
    
    debug!("Querying status cache for device {}", device_id);

    match cache.send(GetCache {
        key: crate::infrastructure::cache::CacheKey::DeviceStatus(device_id.clone()),
    }).await {
        Ok(Some(crate::infrastructure::cache::CacheValue::DeviceStatus(status))) => {
            HttpResponse::Ok().json(status)
        }
        Ok(Some(_)) => {
            HttpResponse::NotFound().json(serde_json::json!({
                "error": "Invalid cache type",
                "message": "Expected status data"
            }))
        }
        Ok(None) => {
            HttpResponse::NotFound().json(serde_json::json!({
                "error": "Not found",
                "message": format!("Status data not found for device {}", device_id)
            }))
        }
        Err(e) => {
            error!("Failed to query status cache: {:?}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Cache error",
                "message": e.to_string()
            }))
        }
    }
}

/// 获取传感器数据
#[utoipa::path(
    get,
    path = "/api/v1/cache/devices/{device_id}/sensors",
    tag = "缓存",
    responses(
        (status = 200, description = "获取成功", body = SensorDataResponse),
        (status = 404, description = "数据不存在"),
        (status = 500, description = "服务器错误")
    ),
    params(
        ("device_id" = String, Path, description = "设备ID")
    )
)]
pub async fn get_sensor_data(
    device_id: web::Path<String>,
    cache: web::Data<Addr<RealtimeCache>>,
) -> impl Responder {
    let device_id = device_id.into_inner();
    
    debug!("Querying sensor cache for device {}", device_id);

    match cache.send(GetCache {
        key: crate::infrastructure::cache::CacheKey::SensorData(device_id.clone()),
    }).await {
        Ok(Some(crate::infrastructure::cache::CacheValue::SensorData(sensors))) => {
            HttpResponse::Ok().json(sensors)
        }
        Ok(Some(_)) => {
            HttpResponse::NotFound().json(serde_json::json!({
                "error": "Invalid cache type",
                "message": "Expected sensor data"
            }))
        }
        Ok(None) => {
            HttpResponse::NotFound().json(serde_json::json!({
                "error": "Not found",
                "message": format!("Sensor data not found for device {}", device_id)
            }))
        }
        Err(e) => {
            error!("Failed to query sensor cache: {:?}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Cache error",
                "message": e.to_string()
            }))
        }
    }
}

/// 清除设备缓存
#[utoipa::path(
    post,
    path = "/api/v1/cache/devices/{device_id}/clear",
    tag = "缓存",
    responses(
        (status = 200, description = "清除成功"),
        (status = 500, description = "服务器错误")
    ),
    params(
        ("device_id" = String, Path, description = "设备ID")
    )
)]
pub async fn clear_device_cache(
    device_id: web::Path<String>,
    cache: web::Data<Addr<RealtimeCache>>,
) -> impl Responder {
    let device_id = device_id.into_inner();
    
    info!("Clearing cache for device {}", device_id);

    match cache.send(ClearDeviceCache {
        device_id: device_id.clone(),
    }).await {
        Ok(Ok(())) => {
            HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "message": format!("Cache cleared for device {}", device_id)
            }))
        }
        Ok(Err(e)) => {
            error!("Failed to clear cache: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Clear failed",
                "message": e
            }))
        }
        Err(e) => {
            error!("Cache actor error: {:?}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Cache error",
                "message": e.to_string()
            }))
        }
    }
}

/// 获取缓存统计信息
#[utoipa::path(
    get,
    path = "/api/v1/cache/stats",
    tag = "缓存",
    responses(
        (status = 200, description = "获取成功", body = CacheStats)
    )
)]
pub async fn get_cache_stats(
    cache: web::Data<Addr<RealtimeCache>>,
) -> impl Responder {
    // TODO: 实现获取统计信息的消息
    #[allow(dead_code)]
    let _cache = cache;
    HttpResponse::Ok().json(serde_json::json!({
        "message": "Cache stats endpoint - to be implemented"
    }))
}

/// 响应类型
#[derive(Debug, Serialize, Deserialize)]
pub struct LocationResponse {
    pub device_id: String,
    pub latitude: f64,
    pub longitude: f64,
    pub altitude: Option<f32>,
    pub speed: Option<f32>,
    pub direction: Option<i32>,
    pub timestamp: String,
    pub address: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeviceStatusResponse {
    pub device_id: String,
    pub online: bool,
    pub auth_status: String,
    pub last_activity: String,
    pub heartbeat_time: Option<String>,
    pub signal_strength: Option<i32>,
    pub battery_level: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SensorDataResponse {
    pub device_id: String,
    pub sensors: std::collections::HashMap<String, f64>,
    pub timestamp: String,
}

/// 配置缓存路由
pub fn configure_cache_routes(cfg: &mut web::ServiceConfig) {
    cfg
        .service(get_device_location)
        .service(get_device_status)
        .service(get_sensor_data)
        .service(clear_device_cache)
        .service(get_cache_stats);
}






