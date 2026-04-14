use actix_web::{web, HttpResponse};
use log::{info, warn};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::application::services::system_monitor_service::{SystemMonitorService, ProcessInfo};
use crate::errors::{success_response_with_message, AppError, AppResult};
use crate::schemas::PagedResponse;

// 进程查询参数
#[derive(Debug, Clone, Deserialize, ToSchema, utoipa::IntoParams)]
pub struct ProcessQuery {
    pub page: Option<i32>,
    pub page_size: Option<i32>,
    pub name: Option<String>,
}

// 获取系统状态
#[utoipa::path(
    path = "/api/system/status",
    get,
    responses(
        (status = 200, description = "System status fetched successfully", body = ApiResponse<SystemStatus>),
        (status = 500, description = "Internal server error", body = ApiResponse<()>)
    )
)]
pub async fn get_system_status(
    system_monitor_service: web::Data<SystemMonitorService>,
) -> AppResult<HttpResponse> {
    let system_status = system_monitor_service.get_system_status();

    info!("System status fetched successfully");
    Ok(success_response_with_message(
        "System status fetched successfully",
        Some(system_status),
    ))
}

// 获取进程列表
#[utoipa::path(
    path = "/api/system/processes",
    get,
    params(ProcessQuery),
    responses(
        (status = 200, description = "Processes fetched successfully", body = ApiResponse<PagedResponse<ProcessInfo>>),
        (status = 500, description = "Internal server error", body = ApiResponse<()>)
    )
)]
pub async fn get_processes(
    system_monitor_service: web::Data<SystemMonitorService>,
    query: web::Query<ProcessQuery>,
) -> AppResult<HttpResponse> {
    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20);

    // 获取进程列表
    let processes = system_monitor_service.get_processes();

    // 过滤进程
    let filtered_processes: Vec<ProcessInfo> = processes
        .into_iter()
        .filter(|process| {
            if let Some(name) = &query.name {
                if !process.name.contains(name) {
                    return false;
                }
            }
            true
        })
        .collect();

    // 分页处理
    let total = filtered_processes.len() as i64;
    let start = ((page - 1) * page_size) as usize;
    let end = (start + page_size as usize).min(filtered_processes.len());
    let paginated_processes = filtered_processes[start..end].to_vec();

    // 计算总页数
    let pages = if total % page_size as i64 == 0 {
        total / page_size as i64
    } else {
        total / page_size as i64 + 1
    };

    // 构建分页响应
    let paged_response = PagedResponse {
        list: paginated_processes,
        total,
        page,
        page_size,
        pages: pages as i32,
    };

    info!("Processes fetched successfully");
    Ok(success_response_with_message(
        "Processes fetched successfully",
        Some(paged_response),
    ))
}

// 获取指定进程信息
#[utoipa::path(
    path = "/api/system/processes/{pid}",
    get,
    responses(
        (status = 200, description = "Process fetched successfully", body = ApiResponse<ProcessInfo>),
        (status = 404, description = "Process not found", body = ApiResponse<()>),
        (status = 500, description = "Internal server error", body = ApiResponse<()>)
    )
)]
pub async fn get_process_by_pid(
    system_monitor_service: web::Data<SystemMonitorService>,
    pid: web::Path<i32>,
) -> AppResult<HttpResponse> {
    let pid = pid.into_inner();

    // 获取进程信息
    match system_monitor_service.get_process_by_pid(pid) {
        Some(process) => {
            info!("Process fetched successfully: {}", process.name);
            Ok(success_response_with_message(
                "Process fetched successfully",
                Some(process),
            ))
        }
        None => {
            warn!("Process not found: {}", pid);
            Err(AppError::not_found_error(
                "Process not found".to_string(),
            ))
        }
    }
}

// 获取系统负载
#[utoipa::path(
    path = "/api/system/load",
    get,
    responses(
        (status = 200, description = "System load fetched successfully", body = ApiResponse<serde_json::Value>),
        (status = 500, description = "Internal server error", body = ApiResponse<()>)
    )
)]
pub async fn get_system_load(
    system_monitor_service: web::Data<SystemMonitorService>,
) -> AppResult<HttpResponse> {
    let load = system_monitor_service.get_system_load();

    let load_response = serde_json::json!({
        "1min": load.0,
        "5min": load.1,
        "15min": load.2
    });

    info!("System load fetched successfully");
    Ok(success_response_with_message(
        "System load fetched successfully",
        Some(load_response),
    ))
}

// 获取内存使用情况
#[utoipa::path(
    path = "/api/system/memory",
    get,
    responses(
        (status = 200, description = "Memory usage fetched successfully", body = ApiResponse<serde_json::Value>),
        (status = 500, description = "Internal server error", body = ApiResponse<()>)
    )
)]
pub async fn get_memory_usage(
    system_monitor_service: web::Data<SystemMonitorService>,
) -> AppResult<HttpResponse> {
    let (total, available) = system_monitor_service.get_memory_usage();
    let used = total - available;
    let usage_percent = (used as f64 / total as f64) * 100.0;

    let memory_response = serde_json::json!({
        "total": total,
        "available": available,
        "used": used,
        "usage_percent": usage_percent
    });

    info!("Memory usage fetched successfully");
    Ok(success_response_with_message(
        "Memory usage fetched successfully",
        Some(memory_response),
    ))
}

// 配置系统监控路由
pub fn configure_system_monitor_routes(cfg: &mut web::ServiceConfig) {
    cfg.route("/system/status", web::get().to(get_system_status))
        .route("/system/processes", web::get().to(get_processes))
        .route("/system/processes/{pid}", web::get().to(get_process_by_pid))
        .route("/system/load", web::get().to(get_system_load))
        .route("/system/memory", web::get().to(get_memory_usage));
}
