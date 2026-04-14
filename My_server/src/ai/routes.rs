//! /! AI 服务路由模块
//! 定义与 DeepSeek 相关的 API 端点

use actix_web::{web, HttpResponse};
use std::sync::Arc;

use super::{coder::*, v3::*};

/// AI 服务状态
#[derive(Debug, Clone)]
pub struct AiServiceState {
    pub coder_service: DeepSeekCoderService,
    pub v3_service: DeepSeekV3Service,
}

/// 配置 AI 路由
pub fn configure_ai_routes(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(
        web::scope("/api/ai")
            .service(
                web::scope("/coder")
                    .route("/generate-code", web::post().to(generate_code))
                    .route("/optimize-query", web::post().to(optimize_query))
                    .route("/generate-api-doc", web::post().to(generate_api_doc))
                    .route("/generate-tests", web::post().to(generate_tests)),
            )
            .service(
                web::scope("/v3")
                    .route(
                        "/generate-weighing-report",
                        web::post().to(generate_weighing_report),
                    )
                    .route("/analyze-anomalies", web::post().to(analyze_anomalies))
                    .route(
                        "/reply-customer-query",
                        web::post().to(reply_customer_query),
                    )
                    .route("/analyze-logs", web::post().to(analyze_logs)),
            ),
    );
}

/// 生成代码
async fn generate_code(
    state: web::Data<Arc<AiServiceState>>,
    request: web::Json<CodeGenerateRequest>,
) -> HttpResponse {
    match state.coder_service.generate_code(&request).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to generate code: {:?}", e)
        })),
    }
}

/// 优化数据库查询
async fn optimize_query(
    state: web::Data<Arc<AiServiceState>>,
    request: web::Json<QueryOptimizeRequest>,
) -> HttpResponse {
    match state.coder_service.optimize_query(&request).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to optimize query: {:?}", e)
        })),
    }
}

/// 生成 API 文档
async fn generate_api_doc(
    state: web::Data<Arc<AiServiceState>>,
    request: web::Json<ApiDocRequest>,
) -> HttpResponse {
    match state.coder_service.generate_api_doc(&request).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to generate API doc: {:?}", e)
        })),
    }
}

/// 生成单元测试
async fn generate_tests(
    state: web::Data<Arc<AiServiceState>>,
    request: web::Json<TestGenerateRequest>,
) -> HttpResponse {
    match state.coder_service.generate_tests(&request).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to generate tests: {:?}", e)
        })),
    }
}

/// 生成称重报告
async fn generate_weighing_report(
    state: web::Data<Arc<AiServiceState>>,
    request: web::Json<WeighingReportRequest>,
) -> HttpResponse {
    match state.v3_service.generate_weighing_report(&request).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to generate weighing report: {:?}", e)
        })),
    }
}

/// 分析异常数据
async fn analyze_anomalies(
    state: web::Data<Arc<AiServiceState>>,
    request: web::Json<AnomalyAnalysisRequest>,
) -> HttpResponse {
    match state.v3_service.analyze_anomalies(&request).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to analyze anomalies: {:?}", e)
        })),
    }
}

/// 回复客户咨询
async fn reply_customer_query(
    state: web::Data<Arc<AiServiceState>>,
    request: web::Json<CustomerQueryRequest>,
) -> HttpResponse {
    match state.v3_service.reply_customer_query(&request).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to reply customer query: {:?}", e)
        })),
    }
}

/// 分析操作日志
async fn analyze_logs(
    state: web::Data<Arc<AiServiceState>>,
    request: web::Json<LogAnalysisRequest>,
) -> HttpResponse {
    match state.v3_service.analyze_logs(&request).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to analyze logs: {:?}", e)
        })),
    }
}
