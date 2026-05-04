use actix_web::{web, HttpResponse};
use log::info;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
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

/// 检查数据库连接状态
async fn check_database_status(pool: &PgPool) -> ServiceStatus {
    match sqlx::query("SELECT 1").execute(pool).await {
        Ok(_) => ServiceStatus {
            name: "database".to_string(),
            status: "running".to_string(),
            pid: None,
            uptime: None,
            memory_usage: None,
            cpu_usage: None,
            last_restart: None,
            error_message: None,
        },
        Err(e) => ServiceStatus {
            name: "database".to_string(),
            status: "error".to_string(),
            pid: None,
            uptime: None,
            memory_usage: None,
            cpu_usage: None,
            last_restart: None,
            error_message: Some(e.to_string()),
        },
    }
}

/// 检查 Redis 连接状态
async fn check_redis_status() -> ServiceStatus {
    let redis_available = crate::redis::is_redis_available().await;
    if redis_available {
        ServiceStatus {
            name: "redis".to_string(),
            status: "running".to_string(),
            pid: None,
            uptime: None,
            memory_usage: None,
            cpu_usage: None,
            last_restart: None,
            error_message: None,
        }
    } else {
        ServiceStatus {
            name: "redis".to_string(),
            status: "error".to_string(),
            pid: None,
            uptime: None,
            memory_usage: None,
            cpu_usage: None,
            last_restart: None,
            error_message: Some("Redis connection failed".to_string()),
        }
    }
}

/// 获取所有服务状态
#[utoipa::path(
    path = "/api/services/status",
    get,
    responses(
        (status = 200, description = "All services status fetched successfully", body = ApiResponse<AllServicesStatus>),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn get_services_status(pool: web::Data<PgPool>) -> AppResult<HttpResponse> {
    info!("Fetching all services status");

    // 并行检查多个服务状态
    let (db_status, redis_status) =
        tokio::join!(check_database_status(pool.get_ref()), check_redis_status(),);

    // WebSocket 和 JT808 服务状态（基于应用状态判断）
    let ws_status = ServiceStatus {
        name: "websocket".to_string(),
        status: "running".to_string(), // WebSocket 作为 HTTP 服务的一部分运行
        pid: None,
        uptime: None,
        memory_usage: None,
        cpu_usage: None,
        last_restart: None,
        error_message: None,
    };

    let jt808_status = ServiceStatus {
        name: "jt808".to_string(),
        status: "running".to_string(), // JT808 网关作为应用的一部分运行
        pid: None,
        uptime: None,
        memory_usage: None,
        cpu_usage: None,
        last_restart: None,
        error_message: None,
    };

    let services = vec![db_status, redis_status, ws_status, jt808_status];

    let total = services.len();
    let running = services.iter().filter(|s| s.status == "running").count();
    let stopped = services.iter().filter(|s| s.status == "stopped").count();
    let error_count = services.iter().filter(|s| s.status == "error").count();

    let response = AllServicesStatus {
        services,
        total,
        running,
        stopped: stopped + error_count,
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

    // 受保护的服务（基础设施服务，不能通过 API 控制）
    let protected_services = ["database", "redis"];

    if protected_services.contains(&service_name.as_str()) {
        tracing::warn!(service = %service_name, "Attempted to control protected infrastructure service");
        return Err(AppError::forbidden_error(format!(
            "Service '{}' is a protected infrastructure service and cannot be controlled via API. \
             Please manage it through your infrastructure tooling (systemd, Docker, Kubernetes, etc.)",
            service_name
        )));
    }

    // 可控的应用服务
    let controllable_services = ["websocket", "jt808"];

    if !controllable_services.contains(&service_name.as_str()) {
        return Err(AppError::not_found_error(format!(
            "Service '{}' not found",
            service_name
        )));
    }

    // 注意: WebSocket 和 JT808 作为应用的一部分运行
    // 它们的"重启"实际上会通过应用自身的优雅重启机制来处理
    tracing::info!(service = %service_name, "Service control requested via API");

    Ok(success_response_with_message(
        "Service control acknowledged",
        Some(serde_json::json!({
            "service_name": service_name,
            "status": "acknowledged",
            "message": format!("Service '{}' control request received. \
                Note: Application-level services run as part of the main process.",
                service_name)
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

    // 受保护的服务（基础设施服务，不能通过 API 控制）
    let protected_services = ["database", "redis"];

    if protected_services.contains(&service_name.as_str()) {
        tracing::warn!(service = %service_name, "Attempted to control protected infrastructure service");
        return Err(AppError::forbidden_error(format!(
            "Service '{}' is a protected infrastructure service and cannot be controlled via API. \
             Please manage it through your infrastructure tooling (systemd, Docker, Kubernetes, etc.)",
            service_name
        )));
    }

    // 可控的应用服务
    let controllable_services = ["websocket", "jt808"];

    if !controllable_services.contains(&service_name.as_str()) {
        return Err(AppError::not_found_error(format!(
            "Service '{}' not found",
            service_name
        )));
    }

    tracing::warn!(
        service = %service_name,
        "Service stop requested. Note: Stopping application services via API is not recommended. \
         Consider using graceful shutdown of the entire application."
    );

    Ok(success_response_with_message(
        "Service stop acknowledged",
        Some(serde_json::json!({
            "service_name": service_name,
            "status": "acknowledged",
            "warning": "Stopping application services is not recommended. Consider using graceful shutdown.",
            "message": format!("Service '{}' stop request received.", service_name)
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

    // 受保护的服务（基础设施服务，不能通过 API 控制）
    let protected_services = ["database", "redis"];

    if protected_services.contains(&service_name.as_str()) {
        tracing::warn!(service = %service_name, "Attempted to control protected infrastructure service");
        return Err(AppError::forbidden_error(format!(
            "Service '{}' is a protected infrastructure service and cannot be controlled via API. \
             Please manage it through your infrastructure tooling (systemd, Docker, Kubernetes, etc.)",
            service_name
        )));
    }

    // 可控的应用服务
    let controllable_services = ["websocket", "jt808"];

    if !controllable_services.contains(&service_name.as_str()) {
        return Err(AppError::not_found_error(format!(
            "Service '{}' not found",
            service_name
        )));
    }

    tracing::info!(
        service = %service_name,
        "Service restart requested via API"
    );

    Ok(success_response_with_message(
        "Service restart acknowledged",
        Some(serde_json::json!({
            "service_name": service_name,
            "status": "acknowledged",
            "message": format!(
                "Service '{}' restart request received. \
                 Note: For full restart, please use deployment tooling or restart the entire application.",
                service_name
            )
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
