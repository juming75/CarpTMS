use actix_web::{web, HttpResponse};
use log::info;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::errors::{success_response_with_message, AppError, AppResult};

// 服务状态结构
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ServiceStatus {
    pub name: String,
    pub status: String, // running, stopped, error
    pub pid: Option<i32>,
    pub uptime: Option<i64>, // 秒
    pub memory_usage: Option<String>,
    pub cpu_usage: Option<f64>,
    pub last_restart: Option<String>,
    pub error_message: Option<String>,
}

// 所有服务状态响应
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct AllServicesStatus {
    pub services: Vec<ServiceStatus>,
    pub total: usize,
    pub running: usize,
    pub stopped: usize,
    pub error: usize,
}

// 启动服务请求
#[derive(Debug, Deserialize, ToSchema)]
pub struct StartServiceRequest {
    pub service_name: String,
}

// 获取所有服务状态
#[utoipa::path(
    path = "/api/services/status",
    get,
    responses(
        (status = 200, description = "All services status fetched successfully", body = ApiResponse<AllServicesStatus>),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn get_services_status() -> AppResult<HttpResponse> {
    info!("Fetching all services status");

    // TODO: 实际检查系统服务状态
    // 这里返回模拟数据,实际应该:
    // 1. 检查数据库服务
    // 2. 检查Redis服务
    // 3. 检查WebSocket服务
    // 4. 检查JT808服务

    let services = vec![
        ServiceStatus {
            name: "database".to_string(),
            status: "running".to_string(),
            pid: Some(12345),
            uptime: Some(3600),
            memory_usage: Some("256MB".to_string()),
            cpu_usage: Some(5.2),
            last_restart: None,
            error_message: None,
        },
        ServiceStatus {
            name: "redis".to_string(),
            status: "running".to_string(),
            pid: Some(12346),
            uptime: Some(3600),
            memory_usage: Some("128MB".to_string()),
            cpu_usage: Some(2.1),
            last_restart: None,
            error_message: None,
        },
        ServiceStatus {
            name: "websocket".to_string(),
            status: "running".to_string(),
            pid: Some(12347),
            uptime: Some(3500),
            memory_usage: Some("64MB".to_string()),
            cpu_usage: Some(1.5),
            last_restart: None,
            error_message: None,
        },
        ServiceStatus {
            name: "jt808".to_string(),
            status: "stopped".to_string(),
            pid: None,
            uptime: None,
            memory_usage: None,
            cpu_usage: None,
            last_restart: Some("2026-03-17 10:00:00".to_string()),
            error_message: Some("Service stopped by user".to_string()),
        },
    ];

    let total = services.len();
    let running = services.iter().filter(|s| s.status == "running").count();
    let stopped = services.iter().filter(|s| s.status == "stopped").count();
    let error_count = services.iter().filter(|s| s.status == "error").count();

    let response = AllServicesStatus {
        services,
        total,
        running,
        stopped: stopped + error_count, // stopped和error都算作未运行
        error: error_count,
    };

    Ok(success_response_with_message(
        "All services status fetched successfully",
        Some(response),
    ))
}

// 启动指定服务
#[utoipa::path(
    path = "/api/services/{service_name}/start",
    post,
    params(
        ("service_name" = String, Path, description = "Service name")
    ),
    responses(
        (status = 200, description = "Service started successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Service not found")
    )
)]
pub async fn start_service(path: web::Path<String>) -> AppResult<HttpResponse> {
    let service_name = path.into_inner();
    info!("Starting service: {}", service_name);

    // TODO: 实际启动系统服务
    // 这里应该:
    // 1. 验证服务名称
    // 2. 检查服务是否已在运行
    // 3. 使用系统命令启动服务(systemctl start, 或直接调用可执行文件)
    // 4. 检查启动是否成功

    let valid_services = ["database", "redis", "websocket", "jt808"];

    if !valid_services.contains(&service_name.as_str()) {
        return Err(AppError::not_found_error(format!(
            "Service '{}' not found",
            service_name
        )));
    }

    // 模拟启动成功
    let _message = format!("Service {} start initiated", service_name);
    Ok(success_response_with_message(
        "Service starting",
        Some(serde_json::json!({
            "service_name": service_name,
            "status": "starting",
            "message": format!("Service {} is starting...", service_name)
        })),
    ))
}

// 停止指定服务
#[utoipa::path(
    path = "/api/services/{service_name}/stop",
    post,
    params(
        ("service_name" = String, Path, description = "Service name")
    ),
    responses(
        (status = 200, description = "Service stopped successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Service not found")
    )
)]
pub async fn stop_service(path: web::Path<String>) -> AppResult<HttpResponse> {
    let service_name = path.into_inner();
    info!("Stopping service: {}", service_name);

    // TODO: 实际停止系统服务
    let valid_services = ["database", "redis", "websocket", "jt808"];

    if !valid_services.contains(&service_name.as_str()) {
        return Err(AppError::not_found_error(format!(
            "Service '{}' not found",
            service_name
        )));
    }

    // 模拟停止成功
    let _message = format!("Service {} stopped successfully", service_name);
    Ok(success_response_with_message(
        "Service stopped successfully",
        Some(serde_json::json!({
            "service_name": service_name,
            "status": "stopped",
            "message": format!("Service {} has been stopped", service_name)
        })),
    ))
}

// 重启指定服务
#[utoipa::path(
    path = "/api/services/{service_name}/restart",
    post,
    params(
        ("service_name" = String, Path, description = "Service name")
    ),
    responses(
        (status = 200, description = "Service restarted successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Service not found")
    )
)]
pub async fn restart_service(path: web::Path<String>) -> AppResult<HttpResponse> {
    let service_name = path.into_inner();
    info!("Restarting service: {}", service_name);

    // TODO: 实际重启系统服务
    let valid_services = ["database", "redis", "websocket", "jt808"];

    if !valid_services.contains(&service_name.as_str()) {
        return Err(AppError::not_found_error(format!(
            "Service '{}' not found",
            service_name
        )));
    }

    // 模拟重启成功
    let _message = format!("Service {} restart initiated", service_name);
    Ok(success_response_with_message(
        "Service restarting",
        Some(serde_json::json!({
            "service_name": service_name,
            "status": "restarting",
            "message": format!("Service {} is restarting...", service_name)
        })),
    ))
}

// 配置服务路由
// 配置服务状态路由(允许 guest 访问)
pub fn configure_services_status_routes(cfg: &mut web::ServiceConfig) {
    cfg.route("/services/status", web::get().to(get_services_status));
}

// 配置服务控制路由(需要 user 权限)
pub fn configure_services_control_routes(cfg: &mut web::ServiceConfig) {
    cfg.route(
        "/services/{service_name}/start",
        web::post().to(start_service),
    )
    .route(
        "/services/{service_name}/stop",
        web::post().to(stop_service),
    )
    .route(
        "/services/{service_name}/restart",
        web::post().to(restart_service),
    );
}
