//! 监控系统API路由
//! 提供系统监控和架构切换相关接口

use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::config::ArchitectureMode;
use crate::infrastructure::monitoring::MonitoringManager;

/// 系统状态响应
#[derive(Debug, Serialize)]
pub struct SystemStatusResponse {
    pub success: bool,
    pub data: SystemStatus,
}

#[derive(Debug, Serialize)]
pub struct SystemStatus {
    pub current_cpu: f64,
    pub current_memory: f64,
    pub current_disk: f64,
    pub avg_response_time_ms: f64,
    pub error_rate: f64,
    pub qps: f64,
    pub active_connections: u64,
    pub load_score: f64,
    pub recommended_mode: String,
}

/// GET /api/monitoring/status - 获取系统状态
pub async fn get_system_status(monitor: web::Data<Arc<MonitoringManager>>) -> impl Responder {
    let metrics = monitor.system_monitor.get_current_metrics().await;
    let recommended_mode = monitor.get_recommended_mode().await;

    HttpResponse::Ok().json(SystemStatusResponse {
        success: true,
        data: SystemStatus {
            current_cpu: metrics.resources.cpu_usage_percent,
            current_memory: metrics.resources.memory_usage_percent,
            current_disk: metrics.resources.disk_usage_percent,
            avg_response_time_ms: metrics.performance.avg_response_time_ms,
            error_rate: metrics.performance.error_rate,
            qps: metrics.performance.requests_per_second,
            active_connections: metrics.performance.concurrent_connections as u64,
            load_score: metrics.timestamp.timestamp() as f64,
            recommended_mode: recommended_mode.to_string(),
        },
    })
}

/// GET /api/monitoring/switching/history - 获取切换历史
pub async fn get_switching_history(monitor: web::Data<Arc<MonitoringManager>>) -> impl Responder {
    if let Some(switcher) = &monitor.architecture_switcher {
        let history = switcher.get_switching_history().await;
        HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "data": history
        }))
    } else {
        HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "data": [],
            "message": "Architecture switching is not enabled"
        }))
    }
}

/// POST /api/monitoring/switching/switch - 手动切换架构
#[derive(Debug, Deserialize)]
pub struct ManualSwitchRequest {
    pub target_mode: String,
    pub reason: Option<String>,
}

pub async fn manual_switch(
    monitor: web::Data<Arc<MonitoringManager>>,
    body: web::Json<ManualSwitchRequest>,
) -> impl Responder {
    if let Some(switcher) = &monitor.architecture_switcher {
        let target_mode = match body.target_mode.to_lowercase().as_str() {
            "microddd" | "microservice" => ArchitectureMode::MicroDDD,
            "monolithddd" | "monolith" => ArchitectureMode::MonolithDDD,
            _ => {
                return HttpResponse::BadRequest().json(serde_json::json!({
                    "success": false,
                    "error": "Invalid mode. Use 'MonolithDDD' or 'MicroDDD'."
                }));
            }
        };
        let reason = body.reason.as_deref().unwrap_or("Manual switch via API");
        switcher.manual_switch(target_mode, reason).await;
        HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "message": format!("Switching to {:?} initiated", target_mode)
        }))
    } else {
        HttpResponse::BadRequest().json(serde_json::json!({
            "success": false,
            "error": "Architecture switching is not enabled"
        }))
    }
}

/// GET /api/monitoring/switching/recommendation - 获取切换建议
pub async fn get_switching_recommendation(
    monitor: web::Data<Arc<MonitoringManager>>,
) -> impl Responder {
    if let Some(switcher) = &monitor.architecture_switcher {
        let (decision, reason) = switcher.get_switching_recommendation().await;
        HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "data": {
                "decision": format!("{:?}", decision),
                "reason": reason,
                "current_mode": format!("{:?}", switcher.get_current_mode().await)
            }
        }))
    } else {
        HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "data": {
                "decision": "KeepCurrent",
                "reason": "Architecture switching is not enabled",
                "current_mode": "MonolithDDD"
            }
        }))
    }
}

/// 配置监控路由
pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/monitoring")
            .route("/status", web::get().to(get_system_status))
            .route("/switching/history", web::get().to(get_switching_history))
            .route(
                "/switching/recommendation",
                web::get().to(get_switching_recommendation),
            )
            .route("/switching/manual", web::post().to(manual_switch)),
    );
}
