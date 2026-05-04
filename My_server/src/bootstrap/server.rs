//! /! HTTP服务器启动模块

use actix_web::{web, App};
use log::info;
use num_cpus;
use std::sync::Arc;

use super::{
    config::AppConfig,
    services::ApplicationState,
    routes::configure_app_routes,
};

/// 配置CORS - 从配置动态加载允许的来源
fn configure_cors(allowed_origins: &[String]) -> actix_cors::Cors {
    let mut cors = actix_cors::Cors::default()
        .allowed_methods(["GET", "POST", "PUT", "DELETE", "OPTIONS"])
        .allowed_headers([
            "Authorization",
            "Content-Type",
            "Accept",
            "Origin",
            "X-Request-ID",
            "X-CSRF-Token",
        ])
        .expose_headers(["Content-Length", "X-Requested-With", "X-Request-ID", "X-CSRF-Token"])
        .max_age(3600)
        .supports_credentials();

    // 开发环境默认 origins
    let has_origins = !allowed_origins.is_empty();
    if has_origins {
        for origin in allowed_origins {
            cors = cors.allowed_origin(origin);
        }
    } else {
        // 仅开发环境的默认值
        cors = cors
            .allowed_origin("http://localhost:5173")
            .allowed_origin("http://127.0.0.1:5173");
    }

    cors
}

/// 配置中间件
fn configure_middleware(allowed_origins: &[String]) -> impl actix_web::dev::Middleware {
    use carptms::middleware;

    (
        middleware::rate_limiter::RateLimiterMiddleware::default(),
        middleware::csrf_protection::csrf_middleware(),
        middleware::audit_logger::AuditLogger::default(),
        middleware::request_logger::RequestLogger::new(),
        actix_web::middleware::Logger::default(),
        configure_cors(allowed_origins),  // 动态CORS
        middleware::circuit_breaker::CircuitBreakerMiddleware::default(),
        middleware::request_header::default_header_middleware(),
        middleware::metrics::MetricsMiddleware::new(),
    )
}

/// 创建并配置HTTP应用
pub fn create_app(
    state: ApplicationState,
    redis_client: Arc<redis::Client>,
    config: &crate::config::unified::UnifiedConfig,
) -> App<
    impl actix_web::dev::ServiceFactory<
        actix_web::dev::ServiceRequest,
        Config = (),
        Error = actix_web::Error,
        InitError = (),
    >,
> {
    App::new()
        .app_data(web::Data::new(state.pool.clone()))
        .wrap(configure_middleware(&config.security.allowed_origins))
        .configure(|cfg| configure_app_routes(cfg, state))
}

/// 启动HTTP服务器
pub async fn start_server(
    config: AppConfig,
    state: ApplicationState,
) -> std::io::Result<()> {
    let host = config.server.host;
    let port = config.server.port;
    let server_address = format!("{}:{}", host, port);

    let protocol = if config.server.security.enable_tls {
        "https"
    } else {
        "http"
    };

    tracing::info!(protocol = %protocol, address = %server_address, "Starting server");
    info!("Starting server at {}://{}", protocol, server_address);

    tracing::debug!(address = %server_address, "Binding server to address");

    // 创建应用
    let app = create_app(state, config.redis_client, &config.config);

    // 绑定服务器
    let server = actix_web::HttpServer::new(move || app.clone())
        .workers(num_cpus::get()) // Use number of CPU cores for worker threads
        .bind(&server_address)?

    tracing::info!(address = %server_address, "Successfully bound server to address");
    info!("Successfully bound server to address: {}", server_address);

    tracing::info!("Starting server...");
    info!("Starting server...");

    // 配置优雅关闭
    let server = server
        .shutdown_timeout(30) // 设置30秒的关闭超时
        .run();

    // 等待信号以优雅关闭服务器
    let graceful = server.handle().clone();
    actix_rt::spawn(async move {
        // 等待SIGINT或SIGTERM信号
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to listen for shutdown signal");
        tracing::info!("Received shutdown signal, starting graceful shutdown...");
        info!("Received shutdown signal, starting graceful shutdown...");

        // 执行优雅关闭
        graceful.stop(true).await;
    });

    // 运行服务器
    server.await
}







