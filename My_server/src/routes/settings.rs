//! Settings routes - delegates to SettingsApplicationService

use actix_web::{web, HttpResponse, HttpRequest};
use log::info;
use std::sync::Arc;

use crate::application::services::audit_log_service::AuditLogService;
use crate::application::services::settings_service::SettingsApplicationService;
use crate::errors::{success_response_with_message, AppResult};

// Re-export types from service for utoipa compatibility
pub use crate::application::services::settings_service::{SystemSettings, CommunicationSettings};

// 获取系统设置
#[utoipa::path(
    path = "/api/settings",
    get,
    responses(
        (status = 200, description = "System settings fetched successfully"),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn get_settings(
    service: web::Data<Arc<SettingsApplicationService>>,
) -> AppResult<HttpResponse> {
    info!("Fetching system settings");
    let settings = service.get_settings().await?;
    Ok(success_response_with_message(
        "System settings fetched successfully",
        Some(settings),
    ))
}

// 更新系统设置
#[utoipa::path(
    path = "/api/settings",
    put,
    request_body = SystemSettings,
    responses(
        (status = 200, description = "System settings updated successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden")
    )
)]
pub async fn update_settings(
    req: HttpRequest,
    service: web::Data<Arc<SettingsApplicationService>>,
    audit_log_service: web::Data<AuditLogService>,
    settings: web::Json<SystemSettings>,
) -> AppResult<HttpResponse> {
    info!("Updating system settings: {:?}", settings);
    
    // 获取旧设置
    let old_settings = service.get_settings().await?;
    let old_value = serde_json::to_value(old_settings)?;
    
    // 更新设置
    let updated = service.update_settings(settings.into_inner()).await?;
    let new_value = serde_json::to_value(&updated)?;
    
    // 记录审计日志
    let ip_address = req.connection_info().realip_remote_addr().map(|s| s.to_string());
    let user_agent = req.headers().get("user-agent").and_then(|h| h.to_str().ok()).map(|s| s.to_string());
    
    // 暂时使用 None 作为 user_id，实际应该从认证中间件获取
    audit_log_service.log_settings_change(
        None,
        "system",
        Some(old_value),
        Some(new_value),
        ip_address,
        user_agent,
    ).await?;
    
    Ok(success_response_with_message(
        "System settings updated successfully",
        Some(updated),
    ))
}

// 获取通信设置
#[utoipa::path(
    path = "/api/settings/communication",
    get,
    responses(
        (status = 200, description = "Communication settings fetched successfully"),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn get_communication_settings(
    service: web::Data<Arc<SettingsApplicationService>>,
) -> AppResult<HttpResponse> {
    info!("Fetching communication settings");
    let settings = service.get_communication_settings().await?;
    Ok(success_response_with_message(
        "Communication settings fetched successfully",
        Some(settings),
    ))
}

// 更新通信设置
#[utoipa::path(
    path = "/api/settings/communication",
    put,
    request_body = CommunicationSettings,
    responses(
        (status = 200, description = "Communication settings updated successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden")
    )
)]
pub async fn update_communication_settings(
    req: HttpRequest,
    service: web::Data<Arc<SettingsApplicationService>>,
    audit_log_service: web::Data<AuditLogService>,
    settings: web::Json<CommunicationSettings>,
) -> AppResult<HttpResponse> {
    info!("Updating communication settings: {:?}", settings);
    
    // 获取旧设置
    let old_settings = service.get_communication_settings().await?;
    let old_value = serde_json::to_value(old_settings)?;
    
    // 更新设置
    let updated = service.update_communication_settings(settings.into_inner()).await?;
    let new_value = serde_json::to_value(&updated)?;
    
    // 记录审计日志
    let ip_address = req.connection_info().realip_remote_addr().map(|s| s.to_string());
    let user_agent = req.headers().get("user-agent").and_then(|h| h.to_str().ok()).map(|s| s.to_string());
    
    // 暂时使用 None 作为 userId，实际应该从认证中间件获取
    audit_log_service.log_settings_change(
        None,
        "communication",
        Some(old_value),
        Some(new_value),
        ip_address,
        user_agent,
    ).await?;
    
    Ok(success_response_with_message(
        "Communication settings updated successfully",
        Some(updated),
    ))
}

// 配置设置路由
pub fn configure_settings_routes(cfg: &mut web::ServiceConfig) {
    cfg.route("/settings", web::get().to(get_settings))
        .route("/settings", web::put().to(update_settings))
        .route(
            "/settings/communication",
            web::get().to(get_communication_settings),
        )
        .route(
            "/settings/communication",
            web::put().to(update_communication_settings),
        );
}
