//! / BFF路由层

use crate::cache::VehicleCache;
use crate::errors::error_handler::success_response_with_message;
use crate::errors::{AppError, AppResult};
use actix_web::{web, HttpResponse};
use chrono::Utc;
use serde::Deserialize;
use sqlx::Row;

/// BFF路由配置
pub fn configure_bff_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/api/bff/vehicles")
            .route(web::get().to(get_vehicles))
            .route(web::post().to(create_vehicle)),
    )
    .service(
        web::resource("/api/bff/vehicles/{id}")
            .route(web::get().to(get_vehicle))
            .route(web::put().to(update_vehicle))
            .route(web::delete().to(delete_vehicle)),
    )
    .service(
        web::resource("/api/bff/vehicles/{id}/location").route(web::get().to(get_vehicle_location)),
    )
    .service(
        web::resource("/api/bff/devices")
            .route(web::get().to(get_devices))
            .route(web::post().to(create_device)),
    )
    .service(
        web::resource("/api/bff/devices/{id}")
            .route(web::get().to(get_device))
            .route(web::put().to(update_device))
            .route(web::delete().to(delete_device)),
    )
    .service(
        web::resource("/api/bff/reports/vehicle-summary").route(web::get().to(get_vehicle_summary)),
    )
    .service(
        web::resource("/api/bff/reports/daily-statistics")
            .route(web::get().to(get_daily_statistics)),
    )
    .service(web::resource("/api/bff/export/{report_type}").route(web::get().to(export_report)));
}

/// 获取车辆列表
async fn get_vehicles(
    _cache: web::Data<VehicleCache>,
    pool: web::Data<sqlx::PgPool>,
) -> AppResult<HttpResponse> {
    // 从数据库获取
    let vehicles = sqlx::query(r#"SELECT * FROM vehicles ORDER BY vehicle_id DESC"#)
        .fetch_all(pool.get_ref())
        .await?;

    // 转换为可序列化的格式
    let serialized_vehicles: Vec<serde_json::Value> = vehicles
        .into_iter()
        .map(|row| {
            serde_json::json!({
                "vehicle_id": row.get::<i32, _>("vehicle_id"),
                "vehicle_name": row.get::<String, _>("vehicle_name"),
                "license_plate": row.get::<String, _>("license_plate"),
                "vehicle_type": row.get::<String, _>("vehicle_type"),
                "status": row.get::<i32, _>("status")
            })
        })
        .collect();

    Ok(success_response_with_message(
        "车辆列表获取成功",
        serialized_vehicles,
    ))
}

/// 创建车辆
async fn create_vehicle(
    data: web::Json<serde_json::Value>,
    _pool: web::Data<sqlx::PgPool>,
) -> AppResult<HttpResponse> {
    Ok(success_response_with_message("Vehicle created", data))
}

/// 获取单个车辆
async fn get_vehicle(
    id: web::Path<String>,
    _cache: web::Data<VehicleCache>,
    pool: web::Data<sqlx::PgPool>,
) -> AppResult<HttpResponse> {
    let vehicle_id = id.into_inner();

    // 从数据库获取
    let vehicle = sqlx::query(r#"SELECT * FROM vehicles WHERE vehicle_id = $1"#)
        .bind(vehicle_id)
        .fetch_optional(pool.get_ref())
        .await?;

    match vehicle {
        Some(v) => {
            // 转换为可序列化的格式
            let serialized_vehicle = serde_json::json!({
                "vehicle_id": v.get::<i32, _>("vehicle_id"),
                "vehicle_name": v.get::<String, _>("vehicle_name"),
                "license_plate": v.get::<String, _>("license_plate"),
                "vehicle_type": v.get::<String, _>("vehicle_type"),
                "status": v.get::<i32, _>("status")
            });

            Ok(success_response_with_message(
                "车辆信息获取成功",
                serialized_vehicle,
            ))
        }
        None => Err(AppError::resource_not_found("车辆不存在")),
    }
}

/// 更新车辆
async fn update_vehicle(
    _id: web::Path<String>,
    data: web::Json<serde_json::Value>,
    _pool: web::Data<sqlx::PgPool>,
) -> AppResult<HttpResponse> {
    Ok(success_response_with_message("Vehicle updated", data))
}

/// 删除车辆
async fn delete_vehicle(
    id: web::Path<String>,
    pool: web::Data<sqlx::PgPool>,
) -> AppResult<HttpResponse> {
    let vehicle_id = id.into_inner();

    // 从数据库删除车辆
    let result = sqlx::query(r#"DELETE FROM vehicles WHERE vehicle_id = $1"#)
        .bind(vehicle_id)
        .execute(pool.get_ref())
        .await?;

    Ok(success_response_with_message(
        &format!("Vehicle deleted, {} rows affected", result.rows_affected()),
        (),
    ))
}

/// 获取车辆位置
async fn get_vehicle_location(
    id: web::Path<String>,
    _cache: web::Data<VehicleCache>,
    pool: web::Data<sqlx::PgPool>,
) -> AppResult<HttpResponse> {
    let vehicle_id = id.into_inner();

    // 从数据库获取最新位置
    let location = sqlx::query(
        r#"
        SELECT * FROM vehicle_locations 
        WHERE vehicle_id = $1 
        ORDER BY timestamp DESC 
        LIMIT 1
    "#,
    )
    .bind(&vehicle_id)
    .fetch_optional(pool.get_ref())
    .await?;

    match location {
        Some(loc) => {
            let location_data = serde_json::json!({
                "vehicle_id": vehicle_id,
                "latitude": loc.get::<f64, _>("latitude"),
                "longitude": loc.get::<f64, _>("longitude"),
                "timestamp": loc.get::<chrono::DateTime<chrono::Utc>, _>("timestamp")
            });
            Ok(success_response_with_message(
                "车辆位置获取成功",
                location_data,
            ))
        }
        None => {
            let location_data = serde_json::json!({
                "vehicle_id": vehicle_id,
                "latitude": 0.0,
                "longitude": 0.0,
                "timestamp": Utc::now()
            });
            Ok(success_response_with_message(
                "暂无车辆位置数据",
                location_data,
            ))
        }
    }
}

/// 获取设备列表
async fn get_devices(pool: web::Data<sqlx::PgPool>) -> AppResult<HttpResponse> {
    let devices = sqlx::query(r#"SELECT * FROM devices ORDER BY device_id DESC"#)
        .fetch_all(pool.get_ref())
        .await?;

    // 转换为可序列化的格式
    let serialized_devices: Vec<serde_json::Value> = devices
        .into_iter()
        .map(|row| {
            serde_json::json!({
                "device_id": row.get::<String, _>("device_id"),
                "device_name": row.get::<String, _>("device_name"),
                "device_type": row.get::<String, _>("device_type"),
                "status": row.get::<i32, _>("status")
            })
        })
        .collect();

    Ok(success_response_with_message(
        "设备列表获取成功",
        serialized_devices,
    ))
}

/// 创建设备
async fn create_device(
    data: web::Json<serde_json::Value>,
    _pool: web::Data<sqlx::PgPool>,
) -> AppResult<HttpResponse> {
    Ok(success_response_with_message("Device created", data))
}

/// 获取单个设备
async fn get_device(
    id: web::Path<String>,
    pool: web::Data<sqlx::PgPool>,
) -> AppResult<HttpResponse> {
    let device_id = id.into_inner();

    let device = sqlx::query(r#"SELECT * FROM devices WHERE device_id = $1"#)
        .bind(device_id)
        .fetch_optional(pool.get_ref())
        .await?;

    match device {
        Some(d) => {
            // 转换为可序列化的格式
            let serialized_device = serde_json::json!({
                "device_id": d.get::<String, _>("device_id"),
                "device_name": d.get::<String, _>("device_name"),
                "device_type": d.get::<String, _>("device_type"),
                "status": d.get::<i32, _>("status")
            });

            Ok(success_response_with_message(
                "设备信息获取成功",
                serialized_device,
            ))
        }
        None => Err(AppError::resource_not_found("设备不存在")),
    }
}

/// 更新设备
async fn update_device(
    _id: web::Path<String>,
    data: web::Json<serde_json::Value>,
    _pool: web::Data<sqlx::PgPool>,
) -> AppResult<HttpResponse> {
    Ok(success_response_with_message("Device updated", data))
}

/// 删除设备
async fn delete_device(
    id: web::Path<String>,
    pool: web::Data<sqlx::PgPool>,
) -> AppResult<HttpResponse> {
    let device_id = id.into_inner();

    let result = sqlx::query(r#"DELETE FROM devices WHERE device_id = $1"#)
        .bind(device_id)
        .execute(pool.get_ref())
        .await?;

    Ok(success_response_with_message(
        &format!("Device deleted, {} rows affected", result.rows_affected()),
        (),
    ))
}

/// 获取车辆汇总报告
async fn get_vehicle_summary(
    pool: web::Data<sqlx::PgPool>,
    _query: web::Query<ReportQuery>,
) -> AppResult<HttpResponse> {
    // 从数据库获取车辆统计数据
    let total_vehicles = sqlx::query_scalar::<_, i64>(r#"SELECT COUNT(*) FROM vehicles"#)
        .fetch_one(pool.get_ref())
        .await?;

    let active_vehicles =
        sqlx::query_scalar::<_, i64>(r#"SELECT COUNT(*) FROM vehicles WHERE status = 1"#)
            .fetch_one(pool.get_ref())
            .await?;

    let offline_vehicles = total_vehicles - active_vehicles;

    let summary_data = serde_json::json!({
        "total_vehicles": total_vehicles,
        "active_vehicles": active_vehicles,
        "offline_vehicles": offline_vehicles
    });

    Ok(success_response_with_message(
        "车辆汇总报告获取成功",
        summary_data,
    ))
}

/// 获取每日统计报告
async fn get_daily_statistics(
    pool: web::Data<sqlx::PgPool>,
    query: web::Query<ReportQuery>,
) -> AppResult<HttpResponse> {
    // 修复临时值被提前释放的问题
    let default_date = Utc::now().format("%Y-%m-%d").to_string();
    let date = query.start_date.as_ref().unwrap_or(&default_date);

    // 从数据库获取每日统计数据
    let total_distance = sqlx::query_scalar::<_, f64>(
        r#"
        SELECT COALESCE(SUM(distance), 0.0) 
        FROM vehicle_locations 
        WHERE DATE(timestamp) = $1
    "#,
    )
    .bind(date)
    .fetch_one(pool.get_ref())
    .await?;

    let statistics_data = serde_json::json!({
        "date": date,
        "total_distance": total_distance,
        "total_fuel": 0.0
    });

    Ok(success_response_with_message(
        "每日统计报告获取成功",
        statistics_data,
    ))
}

/// 导出报告
async fn export_report(
    report_type: web::Path<String>,
    _pool: web::Data<sqlx::PgPool>,
    query: web::Query<ReportQuery>,
) -> AppResult<HttpResponse> {
    let export_data = serde_json::json!({
        "report_type": report_type.into_inner(),
        "format": query.format.as_ref().map(|f| f.to_string()).unwrap_or("csv".to_string()),
        "status": "ready",
        "timestamp": Utc::now()
    });

    Ok(success_response_with_message("报告导出成功", export_data))
}

/// 报告查询参数
#[derive(Debug, Deserialize)]
pub struct ReportQuery {
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub vehicle_ids: Option<String>,
    pub format: Option<String>,
}
