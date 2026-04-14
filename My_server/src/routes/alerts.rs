//! Alerts routes - delegates to AlertApplicationService

use actix_web::{web, HttpResponse};
use serde_json::json;
use std::sync::Arc;

use crate::application::services::alert_service::{AlertApplicationService, AlertQuery};
use crate::errors::{success_response, AppResult};

// Re-export for compatibility
pub use crate::application::services::alert_service::AlertStats;

pub async fn get_alert_stats(service: web::Data<Arc<AlertApplicationService>>) -> AppResult<HttpResponse> {
    let stats = service.get_alert_stats().await?;
    Ok(success_response(json!({
        "total": stats.total,
        "unprocessed": stats.unprocessed,
        "processed": stats.processed,
        "critical": stats.critical
    })))
}

pub async fn get_alerts(
    service: web::Data<Arc<AlertApplicationService>>,
    query: web::Query<AlertQuery>,
) -> AppResult<HttpResponse> {
    let response = service.get_alerts(query.into_inner()).await?;
    Ok(success_response(response))
}

pub async fn get_quick_process(service: web::Data<Arc<AlertApplicationService>>) -> AppResult<HttpResponse> {
    let items = service.get_quick_process().await?;
    Ok(success_response(items))
}

pub async fn get_alert_trend(service: web::Data<Arc<AlertApplicationService>>) -> AppResult<HttpResponse> {
    let trend = service.get_alert_trend().await?;
    Ok(success_response(trend))
}

pub async fn get_alert_types(service: web::Data<Arc<AlertApplicationService>>) -> AppResult<HttpResponse> {
    let types = service.get_alert_types().await?;
    Ok(success_response(types))
}

pub async fn process_alert(
    service: web::Data<Arc<AlertApplicationService>>,
    id: web::Path<i32>,
    _data: web::Json<serde_json::Value>,
) -> AppResult<HttpResponse> {
    let rows_affected = service.process_alert(*id).await?;
    Ok(success_response(json!({
        "success": true,
        "rows_affected": rows_affected
    })))
}

pub fn configure_alert_routes(cfg: &mut web::ServiceConfig) {
    cfg.route("/alerts/stats", web::get().to(get_alert_stats))
        .route("/alerts", web::get().to(get_alerts))
        .route("/alerts/quick-process", web::get().to(get_quick_process))
        .route("/alerts/trend", web::get().to(get_alert_trend))
        .route("/alerts/types", web::get().to(get_alert_types))
        .route("/alerts/{id}/process", web::put().to(process_alert));
}
