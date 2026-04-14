//! Blue-Green Deployment API Routes
//!
//! 蓝绿部署 API 路由

use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::errors::{AppError, AppResult, success_response_with_message};

/// 部署状态响应
#[derive(Debug, Serialize, ToSchema)]
pub struct DeploymentStatusResponse {
    pub current_environment: String,
    pub blue_status: EnvironmentStatus,
    pub green_status: EnvironmentStatus,
    pub canary_enabled: bool,
    pub canary_percentage: f32,
}

/// 环境状态
#[derive(Debug, Serialize, ToSchema)]
pub struct EnvironmentStatus {
    pub endpoint: String,
    pub status: String,
    pub health: String,
    pub traffic_percentage: f32,
    pub version: String,
}

/// 切换流量请求
#[derive(Debug, Deserialize, ToSchema)]
pub struct SwitchTrafficRequest {
    /// 目标环境: blue 或 green
    pub target_environment: String,
    /// 是否启用金丝雀发布
    #[serde(default)]
    pub canary: bool,
    /// 金丝雀流量百分比 (0-100)
    #[serde(default = "default_canary_percentage")]
    pub canary_percentage: f32,
}

fn default_canary_percentage() -> f32 {
    10.0
}

/// 回滚请求
#[derive(Debug, Deserialize, ToSchema)]
pub struct RollbackRequest {
    pub reason: Option<String>,
}

/// 健康检查响应
#[derive(Debug, Serialize, ToSchema)]
pub struct HealthCheckResponse {
    pub environment: String,
    pub healthy: bool,
    pub response_time_ms: u64,
    pub status_code: u16,
    pub checked_at: String,
}

/// 获取部署状态
#[utoipa::path(
    get,
    path = "/api/deployment/status",
    responses(
        (status = 200, description = "部署状态获取成功", body = ApiResponse<DeploymentStatusResponse>)
    ),
    tag = "Deployment"
)]
pub async fn get_deployment_status() -> AppResult<HttpResponse> {
    // 从环境变量读取配置
    let blue_endpoint = std::env::var("DEPLOYMENT_BLUE_ENDPOINT")
        .unwrap_or_else(|_| "http://localhost:8082".to_string());
    let green_endpoint = std::env::var("DEPLOYMENT_GREEN_ENDPOINT")
        .unwrap_or_else(|_| "http://localhost:8083".to_string());
    let current_env = std::env::var("DEPLOYMENT_CURRENT_ENVIRONMENT")
        .unwrap_or_else(|_| "blue".to_string());

    let response = DeploymentStatusResponse {
        current_environment: current_env.clone(),
        blue_status: EnvironmentStatus {
            endpoint: blue_endpoint,
            status: if current_env == "blue" { "live".to_string() } else { "standby".to_string() },
            health: "healthy".to_string(),
            traffic_percentage: if current_env == "blue" { 100.0 } else { 0.0 },
            version: env!("CARGO_PKG_VERSION").to_string(),
        },
        green_status: EnvironmentStatus {
            endpoint: green_endpoint,
            status: if current_env == "green" { "live".to_string() } else { "standby".to_string() },
            health: "healthy".to_string(),
            traffic_percentage: if current_env == "green" { 100.0 } else { 0.0 },
            version: env!("CARGO_PKG_VERSION").to_string(),
        },
        canary_enabled: false,
        canary_percentage: 0.0,
    };

    Ok(success_response_with_message("部署状态获取成功", response))
}

/// 执行健康检查
#[utoipa::path(
    get,
    path = "/api/deployment/health/{environment}",
    responses(
        (status = 200, description = "健康检查完成", body = ApiResponse<HealthCheckResponse>)
    ),
    tag = "Deployment"
)]
pub async fn health_check(
    path: web::Path<String>,
) -> AppResult<HttpResponse> {
    let environment = path.into_inner();
    
    let endpoint = if environment == "blue" {
        std::env::var("DEPLOYMENT_BLUE_ENDPOINT")
            .unwrap_or_else(|_| "http://localhost:8082".to_string())
    } else {
        std::env::var("DEPLOYMENT_GREEN_ENDPOINT")
            .unwrap_or_else(|_| "http://localhost:8083".to_string())
    };

    // 执行健康检查
    let start = std::time::Instant::now();
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| AppError::internal_error(&format!("创建 HTTP 客户端失败: {}", e), None))?;

    let health_url = format!("{}/api/health", endpoint);
    
    let response = client.get(&health_url).send().await;
    
    let response_time_ms = start.elapsed().as_millis() as u64;
    
    let health_response = match response {
        Ok(resp) => HealthCheckResponse {
            environment: environment.clone(),
            healthy: resp.status().is_success(),
            response_time_ms,
            status_code: resp.status().as_u16(),
            checked_at: chrono::Utc::now().to_rfc3339(),
        },
        Err(_e) => HealthCheckResponse {
            environment: environment.clone(),
            healthy: false,
            response_time_ms,
            status_code: 0,
            checked_at: chrono::Utc::now().to_rfc3339(),
        },
    };

    Ok(success_response_with_message("健康检查完成", health_response))
}

/// 切换流量
#[utoipa::path(
    post,
    path = "/api/deployment/switch",
    request_body = SwitchTrafficRequest,
    responses(
        (status = 200, description = "流量切换成功")
    ),
    tag = "Deployment"
)]
pub async fn switch_traffic(
    req: web::Json<SwitchTrafficRequest>,
) -> AppResult<HttpResponse> {
    let target = req.target_environment.to_lowercase();
    
    if target != "blue" && target != "green" {
        return Err(AppError::validation_error(
            "target_environment 必须是 'blue' 或 'green'",
            None,
        ));
    }

    // 获取目标环境端点
    let target_endpoint = if target == "blue" {
        std::env::var("DEPLOYMENT_BLUE_ENDPOINT")
            .unwrap_or_else(|_| "http://localhost:8082".to_string())
    } else {
        std::env::var("DEPLOYMENT_GREEN_ENDPOINT")
            .unwrap_or_else(|_| "http://localhost:8083".to_string())
    };

    // 执行健康检查
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| AppError::internal_error(&format!("创建 HTTP 客户端失败: {}", e), None))?;

    let health_url = format!("{}/api/health", target_endpoint);
    let health_response = client.get(&health_url).send().await
        .map_err(|e| AppError::internal_error(&format!("健康检查失败: {}", e), None))?;

    if !health_response.status().is_success() {
        return Err(AppError::internal_error(
            &format!("目标环境 {} 健康检查失败，无法切换流量", target),
            None,
        ));
    }

    // 更新环境变量（在实际生产中，这应该更新负载均衡器配置）
    std::env::set_var("DEPLOYMENT_CURRENT_ENVIRONMENT", &target);

    Ok(success_response_with_message("流量切换成功", serde_json::json!({
        "previous_environment": if target == "blue" { "green" } else { "blue" },
        "current_environment": target,
        "switched_at": chrono::Utc::now().to_rfc3339(),
        "canary_enabled": req.canary,
        "canary_percentage": if req.canary { req.canary_percentage } else { 0.0 },
    })))
}

/// 执行回滚
#[utoipa::path(
    post,
    path = "/api/deployment/rollback",
    request_body = RollbackRequest,
    responses(
        (status = 200, description = "回滚成功")
    ),
    tag = "Deployment"
)]
pub async fn rollback(
    req: web::Json<RollbackRequest>,
) -> AppResult<HttpResponse> {
    let current = std::env::var("DEPLOYMENT_CURRENT_ENVIRONMENT")
        .unwrap_or_else(|_| "blue".to_string());
    
    let target = if current == "blue" { "green" } else { "blue" };

    // 获取目标环境端点
    let target_endpoint = if target == "blue" {
        std::env::var("DEPLOYMENT_BLUE_ENDPOINT")
            .unwrap_or_else(|_| "http://localhost:8082".to_string())
    } else {
        std::env::var("DEPLOYMENT_GREEN_ENDPOINT")
            .unwrap_or_else(|_| "http://localhost:8083".to_string())
    };

    // 执行健康检查
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| AppError::internal_error(&format!("创建 HTTP 客户端失败: {}", e), None))?;

    let health_url = format!("{}/api/health", target_endpoint);
    let health_response = client.get(&health_url).send().await
        .map_err(|e| AppError::internal_error(&format!("健康检查失败: {}", e), None))?;

    if !health_response.status().is_success() {
        return Err(AppError::internal_error(
            &format!("回滚目标环境 {} 健康检查失败", target),
            None,
        ));
    }

    // 更新环境变量
    std::env::set_var("DEPLOYMENT_CURRENT_ENVIRONMENT", target);

    Ok(success_response_with_message("回滚成功", serde_json::json!({
        "rolled_back_from": current,
        "rolled_back_to": target,
        "rolled_back_at": chrono::Utc::now().to_rfc3339(),
        "reason": req.reason,
    })))
}

/// 配置部署路由
pub fn configure_deployment_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/deployment")
            .route("/status", web::get().to(get_deployment_status))
            .route("/health/{environment}", web::get().to(health_check))
            .route("/switch", web::post().to(switch_traffic))
            .route("/rollback", web::post().to(rollback))
    );
}
