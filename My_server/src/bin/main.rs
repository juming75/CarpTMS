//! CarpTMS Server 主程序入口
//!
//! 车联网运输管理系统 — 通过 finalize_setup 初始化全部服务

#[macro_use]
extern crate tracing;

use actix_cors::Cors;
use actix_web::{middleware, web, App, HttpServer};
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// 全局panic捕获处理器 - 防止panic导致服务崩溃
fn setup_panic_handler() {
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        // 记录panic信息
        error!("========================================");
        error!("  🔥 全局 Panic 捕获!");
        error!("========================================");
        if let Some(location) = panic_info.location() {
            error!(
                "  位置: {}:{}:{}",
                location.file(),
                location.line(),
                location.column()
            );
        }
        if let Some(message) = panic_info.payload().downcast_ref::<&str>() {
            error!("  消息: {}", message);
        } else if let Some(message) = panic_info.payload().downcast_ref::<String>() {
            error!("  消息: {}", message);
        }
        error!("========================================");

        // 调用默认hook（打印到stderr）
        default_hook(panic_info);
    }));
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // 设置全局panic捕获
    setup_panic_handler();

    // 初始化日志
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "carptms_server=debug,actix_web=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("CarpTMS Server 1.1 启动中...");

    // 加载配置
    let app_config = match carptms::init::database::load_config().await {
        Ok(config) => config,
        Err(e) => {
            error!("配置加载失败: {}. 请检查数据库连接和环境变量配置", e);
            return Err(std::io::Error::other(format!(
                "Configuration load failed: {}",
                e
            )));
        }
    };

    // 初始化 Redis 连接（优雅降级，不 panic）
    let redis_client = match redis::Client::open(app_config.config.redis.url.clone()) {
        Ok(c) => {
            info!("Redis 连接成功");
            Arc::new(c)
        }
        Err(e1) => match redis::Client::open("redis://localhost:6379") {
            Ok(c) => {
                warn!("Redis 配置URL连接失败 ({}), 使用默认 localhost:6379", e1);
                Arc::new(c)
            }
            Err(e2) => {
                error!(
                    "Redis 全部连接失败 (配置URL: {}, fallback: {}), 无缓存运行",
                    e1, e2
                );
                // 使用最终的兜底客户端。Client::open 仅解析 URL 不连接，
                // 硬编码 URL 语法固定，理论上不应失败。
                // 保留 expect 作为 BUG 检测，不影响服务启动
                Arc::new(
                    redis::Client::open("redis://127.0.0.1:6379")
                        .expect("BUG: hardcoded URL 'redis://127.0.0.1:6379' must be valid"),
                )
            }
        },
    };

    // 测试Redis连接（不阻塞启动）
    match redis_client.get_connection() {
        Ok(_) => info!("Redis 连接测试通过"),
        Err(e) => {
            warn!("Redis 连接测试失败: {}, 服务将在无缓存模式下启动", e);
            warn!("提示: 部分依赖缓存的功能可能会降级运行");
        }
    }

    // 通过 finalize_setup 初始化全部服务
    let server_state =
        match carptms::init::finalize::finalize_setup(&app_config, &redis_client).await {
            Ok(state) => state,
            Err(e) => {
                error!("服务初始化失败: {}, 将返回错误码", e);
                return Err(std::io::Error::other(format!(
                    "Service initialization failed: {}",
                    e
                )));
            }
        };

    // 初始化自愈引擎（每300秒自检DB/Redis/Memory/Disk/Threads + 每50秒心跳文件）
    carptms::infrastructure::self_heal::SelfHealEngine::init(None);
    info!("自愈引擎已启动 (自检300s/心跳50s)");

    let bind_addr = format!(
        "{}:{}",
        server_state.config.server.host, server_state.config.server.port
    );

    info!("正在绑定 {}...", bind_addr);

    // 创建 HTTP 服务器
    let server = match HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .app_data(web::Data::new(server_state.pool.clone()))
            .app_data(web::Data::new(server_state.ws_app_state.clone()))
            .app_data(web::Data::new(server_state.vehicle_aggregator.clone()))
            .app_data(web::Data::new(server_state.report_service.clone()))
            .app_data(web::Data::new(server_state.template_engine.clone()))
            .app_data(web::Data::new(server_state.video_service.clone()))
            .app_data(web::Data::new(server_state.ai_service_state.clone()))
            .app_data(web::Data::new(server_state.datasource_manager.clone()))
            .app_data(web::Data::new(server_state.driver_service.clone()))
            .app_data(web::Data::new(server_state.department_service.clone()))
            .app_data(web::Data::new(server_state.auth_service.clone()))
            .app_data(web::Data::new(server_state.organization_service.clone()))
            .app_data(web::Data::new(server_state.statistic_service.clone()))
            .app_data(web::Data::new(server_state.vehicle_service.clone()))
            .app_data(web::Data::new(server_state.location_service.clone()))
            .app_data(web::Data::new(server_state.alert_service.clone()))
            .app_data(web::Data::new(server_state.user_service.clone()))
            .app_data(web::Data::new(server_state.device_service.clone()))
            .app_data(web::Data::new(server_state.sync_service.clone()))
            .app_data(web::Data::new(server_state.order_service.clone()))
            .app_data(web::Data::new(server_state.finance_service.clone()))
            .app_data(web::Data::new(server_state.role_service.clone()))
            .app_data(web::Data::new(server_state.vehicle_group_service.clone()))
            .app_data(web::Data::new(
                server_state.openapi_platform_service.clone(),
            ))
            .app_data(web::Data::new(server_state.system_monitor_service.clone()))
            .app_data(web::Data::new(server_state.audit_log_service.clone()))
            .app_data(web::Data::new(server_state.settings_service.clone()))
            .app_data(web::Data::new(server_state.ml_service.clone()))
            .app_data(web::Data::new(redis_client.clone()))
            .app_data(web::PayloadConfig::new(32 * 1024 * 1024))
            .wrap(cors)
            .wrap(middleware::Logger::default())
            .configure(carptms::routes::configure_routes)
    })
    .bind(&bind_addr)
    {
        Ok(s) => s,
        Err(e) => {
            error!("服务器绑定失败 {}: {}", bind_addr, e);
            return Err(e);
        }
    }
    .run();

    info!("✅ CarpTMS Server 已在 {} 启动", bind_addr);
    info!("提示: 按 Ctrl+C 停止服务");

    server.await
}
