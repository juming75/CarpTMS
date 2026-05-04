//! 统一调度API路由

use actix_web::{web, HttpResponse, Result};
use serde::Deserialize;
use std::sync::Arc;

use crate::dispatch::{CommandType, DeviceType, UnifiedDispatchService};

/// 发送调度指令请求
#[derive(Debug, Deserialize)]
pub struct SendCommandRequest {
    pub command_type: String,
    pub target_devices: Vec<i64>,
    pub target_type: String,
    #[serde(default)]
    pub parameters: serde_json::Value,
}

/// 获取所有在线设备
pub async fn get_devices(service: web::Data<Arc<UnifiedDispatchService>>) -> Result<HttpResponse> {
    match service.get_online_devices().await {
        Ok(devices) => Ok(HttpResponse::Ok().json(devices)),
        Err(e) => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to get devices: {}", e)
        }))),
    }
}

/// 获取调度组列表
pub async fn get_groups(service: web::Data<Arc<UnifiedDispatchService>>) -> Result<HttpResponse> {
    match service.get_dispatch_groups().await {
        Ok(groups) => Ok(HttpResponse::Ok().json(groups)),
        Err(e) => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to get groups: {}", e)
        }))),
    }
}

/// 发送调度指令
pub async fn send_command(
    service: web::Data<Arc<UnifiedDispatchService>>,
    body: web::Json<SendCommandRequest>,
) -> Result<HttpResponse> {
    let command_type = match body.command_type.as_str() {
        "track" => CommandType::Track,
        "video_stream" => CommandType::VideoStream,
        "voice_call" => CommandType::VoiceCall,
        "group_call" => CommandType::GroupCall,
        "message" => CommandType::Message,
        "return_home" => CommandType::ReturnHome,
        "emergency_stop" => CommandType::EmergencyStop,
        "position_query" => CommandType::PositionQuery,
        _ => {
            return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "error": format!("Unknown command type: {}", body.command_type)
            })))
        }
    };

    let target_type = match body.target_type.as_str() {
        "vehicle" => DeviceType::Vehicle,
        "drone" => DeviceType::Drone,
        "radio" => DeviceType::Radio,
        _ => {
            return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "error": format!("Unknown device type: {}", body.target_type)
            })))
        }
    };

    match service
        .send_command(
            command_type,
            body.target_devices.clone(),
            target_type,
            body.parameters.clone(),
        )
        .await
    {
        Ok(command) => Ok(HttpResponse::Ok().json(command)),
        Err(e) => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to send command: {}", e)
        }))),
    }
}

/// 获取指令状态
pub async fn get_command_status(
    service: web::Data<Arc<UnifiedDispatchService>>,
    path: web::Path<String>,
) -> Result<HttpResponse> {
    let command_id = path.into_inner();
    match service.get_command_status(&command_id).await {
        Ok(Some(command)) => Ok(HttpResponse::Ok().json(command)),
        Ok(None) => Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": "Command not found"
        }))),
        Err(e) => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to get command status: {}", e)
        }))),
    }
}

/// 配置调度路由
pub fn configure_dispatch_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/dispatch")
            .route("/devices", web::get().to(get_devices))
            .route("/groups", web::get().to(get_groups))
            .route("/commands", web::post().to(send_command))
            .route("/commands/{id}", web::get().to(get_command_status)),
    );
}
