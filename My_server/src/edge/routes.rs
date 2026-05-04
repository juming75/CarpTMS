//! 边缘计算路由
//! 提供边缘计算服务的API接口

use actix_web::{web, HttpResponse, Scope};
use serde::{Deserialize, Serialize};
use crate::edge::{EdgeComputingService, EdgeDevice, EdgeTask, EdgeTaskType, EdgeTaskStatus, EdgeDeviceStatus, EdgeDeviceType, EdgeError};

/// 注册设备请求
#[derive(Debug, Deserialize, Serialize)]
pub struct RegisterDeviceRequest {
    pub id: String,
    pub name: String,
    pub device_type: String,
    pub ip_address: String,
    pub port: u16,
    pub compute_capacity: u32,
    pub memory: u32,
    pub storage: u32,
    pub supported_tasks: Vec<String>,
}

/// 更新设备状态请求
#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateDeviceStatusRequest {
    pub status: String,
}

/// 提交任务请求
#[derive(Debug, Deserialize, Serialize)]
pub struct SubmitTaskRequest {
    pub task_type: String,
    pub data: serde_json::Value,
    pub priority: u32,
    pub estimated_duration: u32,
}

/// 配置边缘计算路由
pub fn configure_edge_routes() -> Scope {
    web::scope("/api/edge")
        .route("/devices", web::get().to(get_all_devices))
        .route("/devices", web::post().to(register_device))
        .route("/devices/{device_id}/status", web::put().to(update_device_status))
        .route("/tasks", web::get().to(get_all_tasks))
        .route("/tasks", web::post().to(submit_task))
        .route("/tasks/{task_id}/status", web::get().to(get_task_status))
        .route("/tasks/{task_id}/result", web::get().to(get_task_result))
}

/// 获取所有设备
async fn get_all_devices(
    edge_service: web::Data<EdgeComputingService>,
) -> HttpResponse {
    let devices = edge_service.get_all_devices().await;
    HttpResponse::Ok().json(devices)
}

/// 注册设备
async fn register_device(
    edge_service: web::Data<EdgeComputingService>,
    req: web::Json<RegisterDeviceRequest>,
) -> HttpResponse {
    let device_type = match req.device_type.as_str() {
        "VehicleTerminal" => EdgeDeviceType::VehicleTerminal,
        "RoadsideUnit" => EdgeDeviceType::RoadsideUnit,
        "Gateway" => EdgeDeviceType::Gateway,
        "SensorNode" => EdgeDeviceType::SensorNode,
        _ => return HttpResponse::BadRequest().json({"error": "Invalid device type"}),
    };
    
    let device = EdgeDevice {
        id: req.id.clone(),
        name: req.name.clone(),
        device_type,
        status: EdgeDeviceStatus::Online,
        ip_address: req.ip_address.clone(),
        port: req.port,
        last_heartbeat: chrono::Utc::now().to_rfc3339(),
        compute_capacity: req.compute_capacity,
        memory: req.memory,
        storage: req.storage,
        supported_tasks: req.supported_tasks.clone(),
    };
    
    match edge_service.register_device(device).await {
        Ok(_) => HttpResponse::Ok().json({"message": "Device registered successfully"}),
        Err(error) => HttpResponse::InternalServerError().json({"error": error.to_string()}),
    }
}

/// 更新设备状态
async fn update_device_status(
    edge_service: web::Data<EdgeComputingService>,
    device_id: web::Path<String>,
    req: web::Json<UpdateDeviceStatusRequest>,
) -> HttpResponse {
    let status = match req.status.as_str() {
        "Online" => EdgeDeviceStatus::Online,
        "Offline" => EdgeDeviceStatus::Offline,
        "Busy" => EdgeDeviceStatus::Busy,
        "Error" => EdgeDeviceStatus::Error,
        _ => return HttpResponse::BadRequest().json({"error": "Invalid status"}),
    };
    
    match edge_service.update_device_status(&device_id, status).await {
        Ok(_) => HttpResponse::Ok().json({"message": "Device status updated successfully"}),
        Err(error) => HttpResponse::InternalServerError().json({"error": error.to_string()}),
    }
}

/// 获取所有任务
async fn get_all_tasks(
    edge_service: web::Data<EdgeComputingService>,
) -> HttpResponse {
    let tasks = edge_service.get_all_tasks().await;
    HttpResponse::Ok().json(tasks)
}

/// 提交任务
async fn submit_task(
    edge_service: web::Data<EdgeComputingService>,
    req: web::Json<SubmitTaskRequest>,
) -> HttpResponse {
    let task_type = match req.task_type.as_str() {
        "LocationProcessing" => EdgeTaskType::LocationProcessing,
        "VideoAnalysis" => EdgeTaskType::VideoAnalysis,
        "AlarmProcessing" => EdgeTaskType::AlarmProcessing,
        "DataAggregation" => EdgeTaskType::DataAggregation,
        "PredictiveAnalysis" => EdgeTaskType::PredictiveAnalysis,
        _ => return HttpResponse::BadRequest().json({"error": "Invalid task type"}),
    };
    
    let task = EdgeTask {
        id: format!("task-{}-{}", chrono::Utc::now().format("%Y%m%d-%H%M%S"), uuid::Uuid::new_v4().to_string().split('-').next().unwrap()),
        task_type,
        data: req.data.clone(),
        priority: req.priority,
        created_at: chrono::Utc::now().to_rfc3339(),
        estimated_duration: req.estimated_duration,
        assigned_device_id: None,
        status: EdgeTaskStatus::Pending,
        result: None,
    };
    
    match edge_service.submit_task(task).await {
        Ok(task_id) => HttpResponse::Ok().json({"task_id": task_id, "message": "Task submitted successfully"}),
        Err(error) => HttpResponse::InternalServerError().json({"error": error.to_string()}),
    }
}

/// 获取任务状态
async fn get_task_status(
    edge_service: web::Data<EdgeComputingService>,
    task_id: web::Path<String>,
) -> HttpResponse {
    match edge_service.get_task_status(&task_id).await {
        Ok(status) => HttpResponse::Ok().json({"status": status.to_string()}),
        Err(error) => HttpResponse::InternalServerError().json({"error": error.to_string()}),
    }
}

/// 获取任务结果
async fn get_task_result(
    edge_service: web::Data<EdgeComputingService>,
    task_id: web::Path<String>,
) -> HttpResponse {
    match edge_service.get_task_result(&task_id).await {
        Ok(result) => HttpResponse::Ok().json({"result": result}),
        Err(error) => HttpResponse::InternalServerError().json({"error": error.to_string()}),
    }
}

// 必要的导入
use chrono;
use uuid;
