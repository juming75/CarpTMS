use actix_web::{web, HttpResponse};
use serde::Deserialize;
use serde_json::json;
use sqlx::Row;

use crate::errors::{success_response, AppResult};
use crate::models::ReportTemplate;
use crate::schemas::ReportGenerateRequest;

#[derive(Debug, Deserialize)]
pub struct ReportQuery {
    pub vehicle_id: Option<i32>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub page: Option<i32>,
    pub page_size: Option<i32>,
}

#[utoipa::path(
    path = "/api/reports/templates",
    get,
    responses(
        (status = 200, description = "Report templates fetched successfully", body = ApiResponse<Vec<ReportTemplate>>),
        (status = 500, description = "Internal server error", body = ApiResponse<()>)
    )
)]
pub async fn get_report_templates(pool: web::Data<sqlx::PgPool>) -> AppResult<HttpResponse> {
    let templates =
        sqlx::query_as::<_, ReportTemplate>("SELECT * FROM report_templates ORDER BY id")
            .fetch_all(pool.get_ref())
            .await
            .unwrap_or_default();

    Ok(success_response(Some(templates)))
}

#[utoipa::path(
    path = "/api/reports/data",
    get,
    responses(
        (status = 200, description = "Report data fetched successfully", body = ApiResponse<PagedResponse<serde_json::Value>>),
        (status = 500, description = "Internal server error", body = ApiResponse<()>)
    )
)]
pub async fn get_report_data(
    pool: web::Data<sqlx::PgPool>,
    query: web::Query<ReportQuery>,
) -> AppResult<HttpResponse> {
    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20);
    let offset = (page - 1) * page_size;

    let rows = sqlx::query(
        "SELECT v.vehicle_id, v.plate_number, v.vehicle_type, v.status, v.create_time \
         FROM vehicles v ORDER BY v.create_time DESC LIMIT $1 OFFSET $2",
    )
    .bind(page_size)
    .bind(offset)
    .fetch_all(pool.get_ref())
    .await
    .unwrap_or_default();

    let items: Vec<serde_json::Value> = rows
        .iter()
        .map(|row| {
            json!({
                "vehicle_id": row.try_get::<i32, _>("vehicle_id").unwrap_or(0),
                "plate_number": row.try_get::<String, _>("plate_number").unwrap_or_default(),
                "vehicle_type": row.try_get::<String, _>("vehicle_type").unwrap_or_default(),
                "status": row.try_get::<i32, _>("status").unwrap_or(0),
            })
        })
        .collect();

    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM vehicles")
        .fetch_one(pool.get_ref())
        .await
        .unwrap_or(0);

    let pages = if total % page_size as i64 == 0 {
        total / page_size as i64
    } else {
        total / page_size as i64 + 1
    };

    Ok(success_response(Some(json!({
        "items": items,
        "total": total,
        "page": page,
        "page_size": page_size,
        "pages": pages
    }))))
}

#[utoipa::path(
    path = "/api/reports/generate",
    post,
    request_body = ReportGenerateRequest,
    responses(
        (status = 201, description = "Report generated successfully", body = ApiResponse<serde_json::Value>),
        (status = 400, description = "Invalid request parameters", body = ApiResponse<()>),
        (status = 500, description = "Internal server error", body = ApiResponse<()>)
    )
)]
pub async fn generate_report(
    pool: web::Data<sqlx::PgPool>,
    request: web::Json<ReportGenerateRequest>,
) -> AppResult<HttpResponse> {
    let start_time = request.start_time.naive_utc().to_string();
    let end_time = request.end_time.naive_utc().to_string();

    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM vehicles WHERE create_time >= $1::timestamp AND create_time <= $2::timestamp",
    )
    .bind(&start_time)
    .bind(&end_time)
    .fetch_one(pool.get_ref())
    .await
    .unwrap_or(0);

    Ok(success_response(Some(json!({
        "template_id": request.template_id,
        "start_time": start_time,
        "end_time": end_time,
        "period_type": request.period_type,
        "record_count": count,
        "generated_at": chrono::Utc::now().to_rfc3339(),
    }))))
}

#[utoipa::path(
    path = "/api/reports/export",
    get,
    responses(
        (status = 200, description = "Report exported successfully", body = [u8]),
        (status = 400, description = "Invalid request parameters", body = ApiResponse<()>),
        (status = 500, description = "Internal server error", body = ApiResponse<()>)
    )
)]
pub async fn export_report(
    pool: web::Data<sqlx::PgPool>,
    _query: web::Query<ReportQuery>,
) -> AppResult<HttpResponse> {
    let rows = sqlx::query(
        "SELECT vehicle_id, plate_number, vehicle_type, status FROM vehicles ORDER BY vehicle_id LIMIT 1000",
    )
    .fetch_all(pool.get_ref())
    .await
    .unwrap_or_default();

    let mut csv = String::from("Vehicle ID,Plate Number,Type,Status\n");
    for row in &rows {
        let vid: i32 = row.try_get("vehicle_id").unwrap_or(0);
        let plate: String = row.try_get("plate_number").unwrap_or_default();
        let vtype: String = row.try_get("vehicle_type").unwrap_or_default();
        let status: i32 = row.try_get("status").unwrap_or(0);
        csv.push_str(&format!("{},{},{},{}\n", vid, plate, vtype, status));
    }

    Ok(HttpResponse::Ok()
        .content_type("text/csv; charset=utf-8")
        .append_header((
            "Content-Disposition",
            "attachment; filename=vehicle_report.csv",
        ))
        .body(csv))
}

#[utoipa::path(
    path = "/api/reports/vehicle-history",
    get,
    responses(
        (status = 200, description = "Vehicle history fetched successfully", body = ApiResponse<Vec<serde_json::Value>>),
        (status = 500, description = "Internal server error", body = ApiResponse<()>)
    )
)]
pub async fn get_vehicle_history(
    pool: web::Data<sqlx::PgPool>,
    query: web::Query<ReportQuery>,
) -> AppResult<HttpResponse> {
    let vehicle_id = query.vehicle_id.unwrap_or(0);
    let start_time = query.start_time.as_deref().unwrap_or("1970-01-01");
    let end_time = query.end_time.as_deref().unwrap_or("2100-01-01");

    let rows = sqlx::query(
        "SELECT location_id, vehicle_id, latitude, longitude, speed, heading, create_time \
         FROM vehicle_locations \
         WHERE vehicle_id = $1 AND create_time >= $2::timestamp AND create_time <= $3::timestamp \
         ORDER BY create_time DESC LIMIT 500",
    )
    .bind(vehicle_id)
    .bind(start_time)
    .bind(end_time)
    .fetch_all(pool.get_ref())
    .await
    .unwrap_or_default();

    let items: Vec<serde_json::Value> = rows
        .iter()
        .map(|row| {
            json!({
                "location_id": row.try_get::<i32, _>("location_id").unwrap_or(0),
                "vehicle_id": row.try_get::<i32, _>("vehicle_id").unwrap_or(0),
                "latitude": row.try_get::<f64, _>("latitude").unwrap_or(0.0),
                "longitude": row.try_get::<f64, _>("longitude").unwrap_or(0.0),
                "speed": row.try_get::<f64, _>("speed").unwrap_or(0.0),
                "heading": row.try_get::<f64, _>("heading").unwrap_or(0.0),
                "create_time": row.try_get::<chrono::NaiveDateTime, _>("create_time").map(|t| t.to_string()).unwrap_or_default(),
            })
        })
        .collect();

    Ok(success_response(Some(
        json!({ "items": items, "total": items.len() }),
    )))
}

#[utoipa::path(
    path = "/api/reports/speed-filter",
    get,
    responses(
        (status = 200, description = "Speed filter data fetched successfully", body = ApiResponse<PagedResponse<serde_json::Value>>),
        (status = 500, description = "Internal server error", body = ApiResponse<()>)
    )
)]
pub async fn get_speed_filter(
    pool: web::Data<sqlx::PgPool>,
    query: web::Query<ReportQuery>,
) -> AppResult<HttpResponse> {
    let start_time = query.start_time.as_deref().unwrap_or("1970-01-01");
    let end_time = query.end_time.as_deref().unwrap_or("2100-01-01");
    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20);
    let offset = (page - 1) * page_size;

    let rows = sqlx::query(
        "SELECT vl.vehicle_id, v.plate_number, vl.speed, vl.latitude, vl.longitude, vl.create_time \
         FROM vehicle_locations vl JOIN vehicles v ON vl.vehicle_id = v.vehicle_id \
         WHERE vl.speed > 80 AND vl.create_time >= $1::timestamp AND vl.create_time <= $2::timestamp \
         ORDER BY vl.speed DESC LIMIT $3 OFFSET $4",
    )
    .bind(start_time)
    .bind(end_time)
    .bind(page_size)
    .bind(offset)
    .fetch_all(pool.get_ref())
    .await
    .unwrap_or_default();

    let items: Vec<serde_json::Value> = rows
        .iter()
        .map(|row| {
            json!({
                "vehicle_id": row.try_get::<i32, _>("vehicle_id").unwrap_or(0),
                "plate_number": row.try_get::<String, _>("plate_number").unwrap_or_default(),
                "speed": row.try_get::<f64, _>("speed").unwrap_or(0.0),
                "latitude": row.try_get::<f64, _>("latitude").unwrap_or(0.0),
                "longitude": row.try_get::<f64, _>("longitude").unwrap_or(0.0),
                "create_time": row.try_get::<chrono::NaiveDateTime, _>("create_time").map(|t| t.to_string()).unwrap_or_default(),
            })
        })
        .collect();

    let total: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM vehicle_locations WHERE speed > 80 AND create_time >= $1::timestamp AND create_time <= $2::timestamp",
    )
    .bind(start_time)
    .bind(end_time)
    .fetch_one(pool.get_ref())
    .await
    .unwrap_or(0);

    let pages = if total % page_size as i64 == 0 {
        total / page_size as i64
    } else {
        total / page_size as i64 + 1
    };

    Ok(success_response(Some(
        json!({ "items": items, "total": total, "page": page, "page_size": page_size, "pages": pages }),
    )))
}

#[utoipa::path(
    path = "/api/reports/parking-stats",
    get,
    responses(
        (status = 200, description = "Parking stats fetched successfully", body = ApiResponse<PagedResponse<serde_json::Value>>),
        (status = 500, description = "Internal server error", body = ApiResponse<()>)
    )
)]
pub async fn get_parking_stats(
    pool: web::Data<sqlx::PgPool>,
    query: web::Query<ReportQuery>,
) -> AppResult<HttpResponse> {
    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20);
    let offset = (page - 1) * page_size;

    let rows = sqlx::query(
        "SELECT v.vehicle_id, v.plate_number, v.status, v.update_time \
         FROM vehicles v WHERE v.status = 0 ORDER BY v.update_time DESC LIMIT $1 OFFSET $2",
    )
    .bind(page_size)
    .bind(offset)
    .fetch_all(pool.get_ref())
    .await
    .unwrap_or_default();

    let items: Vec<serde_json::Value> = rows.iter().map(|row| {
        json!({
            "vehicle_id": row.try_get::<i32, _>("vehicle_id").unwrap_or(0),
            "plate_number": row.try_get::<String, _>("plate_number").unwrap_or_default(),
            "status": row.try_get::<i32, _>("status").unwrap_or(0),
            "update_time": row.try_get::<chrono::NaiveDateTime, _>("update_time").map(|t| t.to_string()).unwrap_or_default(),
        })
    }).collect();

    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM vehicles WHERE status = 0")
        .fetch_one(pool.get_ref())
        .await
        .unwrap_or(0);
    let pages = if total % page_size as i64 == 0 {
        total / page_size as i64
    } else {
        total / page_size as i64 + 1
    };

    Ok(success_response(Some(
        json!({ "items": items, "total": total, "page": page, "page_size": page_size, "pages": pages }),
    )))
}

#[utoipa::path(
    path = "/api/reports/online-stats",
    get,
    responses(
        (status = 200, description = "Online stats fetched successfully", body = ApiResponse<PagedResponse<serde_json::Value>>),
        (status = 500, description = "Internal server error", body = ApiResponse<()>)
    )
)]
pub async fn get_online_stats(
    pool: web::Data<sqlx::PgPool>,
    query: web::Query<ReportQuery>,
) -> AppResult<HttpResponse> {
    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20);
    let offset = (page - 1) * page_size;

    let rows = sqlx::query(
        "SELECT v.vehicle_id, v.plate_number, v.status, v.update_time \
         FROM vehicles v WHERE v.status = 1 ORDER BY v.update_time DESC LIMIT $1 OFFSET $2",
    )
    .bind(page_size)
    .bind(offset)
    .fetch_all(pool.get_ref())
    .await
    .unwrap_or_default();

    let items: Vec<serde_json::Value> = rows.iter().map(|row| {
        json!({
            "vehicle_id": row.try_get::<i32, _>("vehicle_id").unwrap_or(0),
            "plate_number": row.try_get::<String, _>("plate_number").unwrap_or_default(),
            "status": row.try_get::<i32, _>("status").unwrap_or(0),
            "update_time": row.try_get::<chrono::NaiveDateTime, _>("update_time").map(|t| t.to_string()).unwrap_or_default(),
        })
    }).collect();

    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM vehicles WHERE status = 1")
        .fetch_one(pool.get_ref())
        .await
        .unwrap_or(0);
    let pages = if total % page_size as i64 == 0 {
        total / page_size as i64
    } else {
        total / page_size as i64 + 1
    };

    Ok(success_response(Some(
        json!({ "items": items, "total": total, "page": page, "page_size": page_size, "pages": pages }),
    )))
}

#[utoipa::path(
    path = "/api/reports/mileage-stats",
    get,
    responses(
        (status = 200, description = "Mileage stats fetched successfully", body = ApiResponse<PagedResponse<serde_json::Value>>),
        (status = 500, description = "Internal server error", body = ApiResponse<()>)
    )
)]
pub async fn get_mileage_stats(
    pool: web::Data<sqlx::PgPool>,
    query: web::Query<ReportQuery>,
) -> AppResult<HttpResponse> {
    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20);
    let offset = (page - 1) * page_size;

    let rows = sqlx::query(
        "SELECT v.vehicle_id, v.plate_number, v.mileage, v.update_time \
         FROM vehicles v WHERE v.mileage IS NOT NULL ORDER BY v.mileage DESC NULLS LAST LIMIT $1 OFFSET $2",
    )
    .bind(page_size)
    .bind(offset)
    .fetch_all(pool.get_ref())
    .await
    .unwrap_or_default();

    let items: Vec<serde_json::Value> = rows.iter().map(|row| {
        json!({
            "vehicle_id": row.try_get::<i32, _>("vehicle_id").unwrap_or(0),
            "plate_number": row.try_get::<String, _>("plate_number").unwrap_or_default(),
            "mileage": row.try_get::<f64, _>("mileage").unwrap_or(0.0),
            "update_time": row.try_get::<chrono::NaiveDateTime, _>("update_time").map(|t| t.to_string()).unwrap_or_default(),
        })
    }).collect();

    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM vehicles WHERE mileage IS NOT NULL")
        .fetch_one(pool.get_ref())
        .await
        .unwrap_or(0);
    let pages = if total % page_size as i64 == 0 {
        total / page_size as i64
    } else {
        total / page_size as i64 + 1
    };

    Ok(success_response(Some(
        json!({ "items": items, "total": total, "page": page, "page_size": page_size, "pages": pages }),
    )))
}

#[utoipa::path(
    path = "/api/reports/status-query",
    get,
    responses(
        (status = 200, description = "Status query data fetched successfully", body = ApiResponse<PagedResponse<serde_json::Value>>),
        (status = 500, description = "Internal server error", body = ApiResponse<()>)
    )
)]
pub async fn get_status_query(
    pool: web::Data<sqlx::PgPool>,
    query: web::Query<ReportQuery>,
) -> AppResult<HttpResponse> {
    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20);
    let offset = (page - 1) * page_size;

    let rows = sqlx::query(
        "SELECT vehicle_id, plate_number, vehicle_type, status, create_time FROM vehicles ORDER BY vehicle_id LIMIT $1 OFFSET $2",
    )
    .bind(page_size)
    .bind(offset)
    .fetch_all(pool.get_ref())
    .await
    .unwrap_or_default();

    let items: Vec<serde_json::Value> = rows.iter().map(|row| {
        json!({
            "vehicle_id": row.try_get::<i32, _>("vehicle_id").unwrap_or(0),
            "plate_number": row.try_get::<String, _>("plate_number").unwrap_or_default(),
            "vehicle_type": row.try_get::<String, _>("vehicle_type").unwrap_or_default(),
            "status": row.try_get::<i32, _>("status").unwrap_or(0),
            "create_time": row.try_get::<chrono::NaiveDateTime, _>("create_time").map(|t| t.to_string()).unwrap_or_default(),
        })
    }).collect();

    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM vehicles")
        .fetch_one(pool.get_ref())
        .await
        .unwrap_or(0);
    let pages = if total % page_size as i64 == 0 {
        total / page_size as i64
    } else {
        total / page_size as i64 + 1
    };

    Ok(success_response(Some(
        json!({ "items": items, "total": total, "page": page, "page_size": page_size, "pages": pages }),
    )))
}

#[utoipa::path(
    path = "/api/reports/alarm-records",
    get,
    responses(
        (status = 200, description = "Alarm records fetched successfully", body = ApiResponse<PagedResponse<serde_json::Value>>),
        (status = 500, description = "Internal server error", body = ApiResponse<()>)
    )
)]
pub async fn get_alarm_records(
    pool: web::Data<sqlx::PgPool>,
    query: web::Query<ReportQuery>,
) -> AppResult<HttpResponse> {
    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20);
    let offset = (page - 1) * page_size;
    let start_time = query.start_time.as_deref().unwrap_or("1970-01-01");
    let end_time = query.end_time.as_deref().unwrap_or("2100-01-01");

    let rows = sqlx::query(
        "SELECT id, vehicle_id, alarm_type, severity, description, status, create_time \
         FROM alarms \
         WHERE create_time >= $1::timestamp AND create_time <= $2::timestamp \
         ORDER BY create_time DESC LIMIT $3 OFFSET $4",
    )
    .bind(start_time)
    .bind(end_time)
    .bind(page_size)
    .bind(offset)
    .fetch_all(pool.get_ref())
    .await
    .unwrap_or_default();

    let items: Vec<serde_json::Value> = rows.iter().map(|row| {
        json!({
            "id": row.try_get::<i32, _>("id").unwrap_or(0),
            "vehicle_id": row.try_get::<i32, _>("vehicle_id").unwrap_or(0),
            "alarm_type": row.try_get::<String, _>("alarm_type").unwrap_or_default(),
            "severity": row.try_get::<String, _>("severity").unwrap_or_default(),
            "description": row.try_get::<String, _>("description").unwrap_or_default(),
            "status": row.try_get::<String, _>("status").unwrap_or_default(),
            "create_time": row.try_get::<chrono::NaiveDateTime, _>("create_time").map(|t| t.to_string()).unwrap_or_default(),
        })
    }).collect();

    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM alarms")
        .fetch_one(pool.get_ref())
        .await
        .unwrap_or(0);
    let pages = if total % page_size as i64 == 0 {
        total / page_size as i64
    } else {
        total / page_size as i64 + 1
    };

    Ok(success_response(Some(
        json!({ "items": items, "total": total, "page": page, "page_size": page_size, "pages": pages }),
    )))
}

#[utoipa::path(
    path = "/api/reports/vehicle-info",
    get,
    responses(
        (status = 200, description = "Vehicle info fetched successfully", body = ApiResponse<PagedResponse<serde_json::Value>>),
        (status = 500, description = "Internal server error", body = ApiResponse<()>)
    )
)]
pub async fn get_vehicle_info(
    pool: web::Data<sqlx::PgPool>,
    query: web::Query<ReportQuery>,
) -> AppResult<HttpResponse> {
    get_status_query(pool, query).await
}

#[utoipa::path(
    path = "/api/reports/daily-event-report",
    get,
    responses(
        (status = 200, description = "Daily event report fetched successfully", body = ApiResponse<PagedResponse<serde_json::Value>>),
        (status = 500, description = "Internal server error", body = ApiResponse<()>)
    )
)]
pub async fn get_daily_event_report(
    pool: web::Data<sqlx::PgPool>,
    query: web::Query<ReportQuery>,
) -> AppResult<HttpResponse> {
    let start_time = query.start_time.as_deref().unwrap_or("1970-01-01");
    let end_time = query.end_time.as_deref().unwrap_or("2100-01-01");

    let vehicle_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM vehicles")
        .fetch_one(pool.get_ref())
        .await
        .unwrap_or(0);
    let online_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM vehicles WHERE status = 1")
        .fetch_one(pool.get_ref())
        .await
        .unwrap_or(0);
    let alarm_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM alarms WHERE create_time >= $1::timestamp AND create_time <= $2::timestamp",
    )
    .bind(start_time).bind(end_time)
    .fetch_one(pool.get_ref()).await.unwrap_or(0);
    let order_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM orders WHERE create_time >= $1::timestamp AND create_time <= $2::timestamp",
    )
    .bind(start_time).bind(end_time)
    .fetch_one(pool.get_ref()).await.unwrap_or(0);

    Ok(success_response(Some(json!({
        "items": [{
            "date": start_time,
            "total_vehicles": vehicle_count,
            "online_vehicles": online_count,
            "alarm_count": alarm_count,
            "order_count": order_count,
        }],
        "total": 1,
        "page": 1,
        "page_size": 20,
        "pages": 1,
    }))))
}

#[utoipa::path(
    path = "/api/reports/track-playback",
    get,
    responses(
        (status = 200, description = "Track playback data fetched successfully", body = ApiResponse<PagedResponse<serde_json::Value>>),
        (status = 500, description = "Internal server error", body = ApiResponse<()>)
    )
)]
pub async fn get_track_playback(
    pool: web::Data<sqlx::PgPool>,
    query: web::Query<ReportQuery>,
) -> AppResult<HttpResponse> {
    get_vehicle_history(pool, query).await
}

pub fn configure_report_routes(cfg: &mut web::ServiceConfig) {
    cfg.route("/reports/templates", web::get().to(get_report_templates))
        .route("/reports/data", web::get().to(get_report_data))
        .route("/reports/generate", web::post().to(generate_report))
        .route("/reports/export", web::get().to(export_report))
        .route(
            "/reports/vehicle-history",
            web::get().to(get_vehicle_history),
        )
        .route("/reports/speed-filter", web::get().to(get_speed_filter))
        .route("/reports/parking-stats", web::get().to(get_parking_stats))
        .route("/reports/online-stats", web::get().to(get_online_stats))
        .route("/reports/mileage-stats", web::get().to(get_mileage_stats))
        .route("/reports/status-query", web::get().to(get_status_query))
        .route("/reports/alarm-records", web::get().to(get_alarm_records))
        .route("/reports/vehicle-info", web::get().to(get_vehicle_info))
        .route(
            "/reports/daily-event-report",
            web::get().to(get_daily_event_report),
        )
        .route("/reports/track-playback", web::get().to(get_track_playback));
}
