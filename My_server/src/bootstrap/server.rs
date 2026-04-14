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

/// 配置CORS
fn configure_cors() -> actix_cors::Cors {
    actix_cors::Cors::default()
        .allowed_origin("http://localhost:5173")
        .allowed_origin("http://127.0.0.1:5173")
        .allowed_origin("http://localhost:8082")
        .allowed_origin("http://127.0.0.1:8082")
        .allowed_methods(["GET", "POST", "PUT", "DELETE", "OPTIONS"])
        .allowed_headers([
            "Authorization",
            "Content-Type",
            "Accept",
            "Origin",
            "X-Request-ID",
        ])
        .expose_headers(["Content-Length", "X-Requested-With", "X-Request-ID"])
        .max_age(3600)
        .supports_credentials()
}

/// 配置中间件
fn configure_middleware() -> impl actix_web::dev::Middleware {
    use tms_server::middleware;

    (
        // 速率限制中间件
        middleware::rate_limiter::RateLimiterMiddleware::default(),
        // 请求日志中间件
        middleware::request_logger::RequestLogger::new(),
        // 结构化日志
        actix_web::middleware::Logger::default(),
        // CORS
        configure_cors(),
        // 熔断中间件
        middleware::circuit_breaker::CircuitBreakerMiddleware::default(),
        // 请求头验证中间件
        middleware::request_header::default_header_middleware(),
        // 监控中间件
        middleware::metrics::MetricsMiddleware::new(),
    )
}

/// 创建并配置HTTP应用
pub fn create_app(state: ApplicationState, redis_client: Arc<redis::Client>) -> App<
    impl actix_web::dev::ServiceFactory<
        actix_web::dev::ServiceRequest,
        Config = (),
        Error = actix_web::Error,
        InitError = (),
    >,
> {
    App::new()
        .app_data(web::Data::new(state.pool.clone()))
        .wrap(configure_middleware())
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

    println!("Starting server at {}://{}", protocol, server_address);
    info!("Starting server at {}://{}", protocol, server_address);

    println!("Binding server to address: {}", server_address);

    // 创建应用
    let app = create_app(state, config.redis_client);

    // 绑定服务器
    let server = actix_web::HttpServer::new(move || app.clone())
        .workers(num_cpus::get()) // Use number of CPU cores for worker threads
        .bind(&server_address)?

    println!("Successfully bound server to address: {}", server_address);
    info!("Successfully bound server to address: {}", server_address);

    println!("Starting server...");
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
        println!("Received shutdown signal, starting graceful shutdown...");
        info!("Received shutdown signal, starting graceful shutdown...");

        // 执行优雅关闭
        graceful.stop(true).await;
    });

    // 运行服务器
    server.await
}







