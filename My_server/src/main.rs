//! # CarpTMS Server
//! 
//! Transportation Management System (TMS) 主服务器
//! 
//! ## 智能架构模式
//! 系统默认以单体DDD模式启动，通过智能监控自动决定是否需要切换到微服务架构：
//! - 当数据量、负载、QPS等指标超过阈值时，自动建议切换到微服务
//! - 支持手动覆盖自动决策
//! 
//! ## 服务端口
//! - HTTP API: 8082 | Truck Scale: 9808 | Client API: 9809 | WebSocket: 8089

#![recursion_limit = "512"]

use std::sync::Arc;
use tms_server::config::ArchitectureMode;

use tms_server::init;
use tms_server::logging::init_log_manager;

use tms_server::telemetry::{init_telemetry, shutdown_telemetry};
use tms_server::tracing::{init_tracing, shutdown_tracing};
use tms_server::infrastructure::monitoring::{MonitoringManager, default_switching_config};

/// 主函数 - 启动CarpTMS服务器
#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    // 1. 初始化追踪和日志
    let _ = init_tracing();
    let _ = init_telemetry("carptms-server");
    let _ = init_log_manager(10000);

    // 初始化监控指标
    tms_server::metrics::init_metrics();

    // 初始化内存监控和限制
    let memory_monitor_config = tms_server::performance::MemoryMonitorConfig {
        memory_threshold: 80.0,
        check_interval: std::time::Duration::from_secs(60),
        memory_limit: Some(16 * 1024 * 1024 * 1024),
        enable_memory_limit: true,
    };

    let memory_limit_service = tms_server::performance::MemoryLimitService::new(
        memory_monitor_config,
        None,
    );
    memory_limit_service.start();
    tracing::info!("Memory limit service started with 16GB limit");

    // 初始化资源告警服务
    let resource_alert_config = tms_server::performance::ResourceAlertConfig {
        cpu_threshold: 80.0,
        memory_threshold: 85.0,
        disk_threshold: 90.0,
        check_interval: std::time::Duration::from_secs(60),
        cooldown_period: std::time::Duration::from_secs(300),
    };

    let resource_alert_service = tms_server::performance::ResourceAlertService::new(resource_alert_config);
    tokio::spawn(async move {
        resource_alert_service.start().await;
    });
    tracing::info!("Resource alert service started");

    tracing::info!("Starting CarpTMS server...");

    // 2. 加载配置并初始化
    let (app_config, redis_client) = init::init_all().await
        .map_err(|e| std::io::Error::other(format!("Failed to initialize: {}", e)))?;

    // 3. 获取数据库连接池用于监控
    let pool = app_config.pool.clone();

    // 4. 初始化智能监控和自动切换系统
    let initial_mode = ArchitectureMode::MonolithDDD;
    let switching_config = default_switching_config();
    
    // 解包 Arc<Pool> 为 Pool
    let pool_for_monitor = (*pool).clone();
    
    let monitoring_manager = Arc::new(MonitoringManager::new(
        Some(pool_for_monitor),
        initial_mode,
        Some(switching_config),
    ));
    
    monitoring_manager.start().await;
    tracing::info!("Smart monitoring system started (auto-switching enabled)");

    // 5. 检查JWT密钥配置
    if let Err(e) = tms_server::utils::jwt::check_jwt_secret() {
        tracing::error!("JWT密钥配置检查失败: {}", e);
        return Err(std::io::Error::other(format!("JWT configuration error: {}", e)));
    }

    // 6. 执行所有服务初始化
    let server_state = init::finalize::finalize_setup(&app_config, &redis_client)
        .await
        .map_err(|e| std::io::Error::other(format!("Failed to initialize services: {}", e)))?;

    // 7. 获取智能推荐的架构模式
    let recommended_mode = monitoring_manager.get_recommended_mode().await;
    tracing::info!("Recommended architecture mode: {:?}", recommended_mode);

    // 8. 启动服务器（智能模式）
    tracing::info!("Starting in Smart DDD mode...");
    start_api_gateway(server_state, redis_client).await?;

    // 9. 关闭追踪
    shutdown_tracing();
    shutdown_telemetry();
    tracing::info!("Server shutdown complete");

    Ok(())
}

/// API Gateway启动
async fn start_api_gateway(
    server_state: tms_server::init::finalize::ServerState,
    redis_client: Arc<::redis::Client>,
) -> std::io::Result<()> {
    let config = &server_state.config;
    let server_addr = format!("{}:{}", config.server.host, config.server.port);
    let protocol = if config.security.enable_https { "https" } else { "http" };
    tracing::info!("Starting API Gateway at {}://{}", protocol, server_addr);

    // Create circuit breaker
    let circuit_breaker = std::sync::Arc::new(tms_server::middleware::circuit_breaker::CircuitBreaker::new());

    // Create API Gateway config
    let gateway_config = tms_server::gateway::api_gateway::ApiGatewayConfig {
        address: server_addr,
        server_state,
        redis_client: redis_client.clone(),
        circuit_breaker,
    };

    // Create and start API Gateway
    let api_gateway = tms_server::gateway::api_gateway::ApiGateway::new(gateway_config);
    api_gateway.start().await
}


