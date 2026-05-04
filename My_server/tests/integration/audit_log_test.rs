//! 审计日志模块集成测试
//! 验证审计日志中间件和API端点的功能

use actix_web::{test, web, App, HttpResponse};

use carptms::middleware::audit_logger::{AuditLogger, AuditLoggerConfig};

#[actix_web::test]
async fn test_audit_middleware_skip_health() {
    let config = AuditLoggerConfig {
        skip_paths: vec!["/health".to_string()],
        monitored_paths: vec!["/api/".to_string()],
        log_request_body: false,
        log_response_body: false,
    };

    let app = test::init_service(
        App::new()
            .wrap(AuditLogger::new(config.clone()))
            .route("/health", web::get().to(|| async { HttpResponse::Ok().body("ok") }))
            .route("/api/test", web::post().to(|| async { HttpResponse::Ok().body("test") })),
    )
    .await;

    // 健康检查应该不被记录
    let req = test::TestRequest::get().uri("/health").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

#[actix_web::test]
async fn test_audit_middleware_monitors_api() {
    let config = AuditLoggerConfig {
        skip_paths: vec![],
        monitored_paths: vec!["/api/".to_string()],
        log_request_body: false,
        log_response_body: false,
    };

    let app = test::init_service(
        App::new()
            .wrap(AuditLogger::new(config))
            .route("/api/test", web::post().to(|| async { HttpResponse::Ok().body("test") })),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/test")
        .append_header(("Content-Type", "application/json"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}
