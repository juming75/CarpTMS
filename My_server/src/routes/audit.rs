//! 审计日志查询API路由

use actix_web::{web, HttpResponse, Responder};
use serde::Deserialize;
use sqlx::PgPool;

use crate::utils::audit::{search_audit_logs, AuditLogSearchParams, PaginationParams};

/// 审计日志查询参数
#[derive(Debug, Deserialize)]
pub struct AuditQueryParams {
    pub page: Option<i32>,
    pub page_size: Option<i32>,
    pub user_id: Option<i32>,
    pub action: Option<String>,
    pub resource: Option<String>,
}

/// GET /api/audit/logs - 查询审计日志
pub async fn get_audit_logs(
    pool: web::Data<PgPool>,
    query: web::Query<AuditQueryParams>,
) -> impl Responder {
    let search_params = AuditLogSearchParams {
        user_id: query.user_id,
        action: query.action.clone(),
        resource: query.resource.clone(),
        start_time: None,
        end_time: None,
    };

    let pagination = PaginationParams {
        page: query.page.unwrap_or(1),
        page_size: query.page_size.unwrap_or(20).min(100),
    };

    match search_audit_logs(&pool, search_params, pagination).await {
        Ok(logs) => HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "data": logs
        })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "success": false,
            "error": format!("Failed to query audit logs: {}", e)
        })),
    }
}

/// 配置审计日志路由
pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/api/audit").route("/logs", web::get().to(get_audit_logs)));
}
