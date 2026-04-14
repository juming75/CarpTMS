//! / 路由模块,负责管理所有API路由
use actix::StreamHandler;
use actix_web::{web, HttpResponse, HttpRequest, error};
use actix_web_actors::ws;
use actix_web_httpauth::middleware::HttpAuthentication;
use actix_web_httpauth::extractors::bearer::BearerAuth;

use crate::utils::jwt::Claims;

// 导出子路由模块
pub mod adapter;
pub mod alerts;
pub mod auth;
pub mod clean_vehicles;
pub mod dashboard;
pub mod departments;
pub mod deployment;
pub mod devices;
pub mod disaster_recovery;
pub mod drivers;
pub mod dynamic_rate_config;
pub mod finance;
pub mod jt808_command;
pub mod locations;
pub mod orders;
pub mod organization_settings;
pub mod organizations;
pub mod pagination; // 分页工具模块
pub mod protocol;
pub mod reports;
pub mod roles;
pub mod services;
pub mod settings;
pub mod statistics;
pub mod sync;
pub mod users;
pub mod vehicle_groups;
pub mod vehicles;
pub mod video;
pub mod weight_calibration;
pub mod weighing;
pub mod openapi_platforms;
pub mod openapi_loading_points;
pub mod system_monitor;

use crate::gateway::websocket_server;

use crate::{health_check, liveness_check, metrics_endpoint, readiness_check};

// 路由配置函数 - 使用与 utils::jwt 相同的密钥获取方式
async fn jwt_validator(
    req: actix_web::dev::ServiceRequest,
    credentials: BearerAuth,
) -> Result<actix_web::dev::ServiceRequest, (actix_web::Error, actix_web::dev::ServiceRequest)> {
    let token = credentials.token();
    
    let decoding_key = match crate::utils::jwt::get_hs256_secret().await {
        Ok(secret) => jsonwebtoken::DecodingKey::from_secret(secret.as_bytes()),
        Err(_) => {
            log::error!("JWT_SECRET not configured for token validation");
            return Err((error::ErrorUnauthorized("JWT配置错误"), req));
        }
    };
    
    let algorithm = crate::utils::jwt::get_jwt_algorithm();
    let mut validation = jsonwebtoken::Validation::new(algorithm);
    validation.leeway = 30; // 30秒容错，与 utils::jwt::verify_token 一致
    
    match jsonwebtoken::decode::<Claims>(token, &decoding_key, &validation) {
        Ok(_token_data) => Ok(req),
        Err(e) => {
            log::warn!("JWT验证失败: {}", e);
            Err((error::ErrorUnauthorized(format!("无效的token: {}", e)), req))
        }
    }
}

// 简单的测试路由
async fn test_ws_route() -> HttpResponse {
    log::info!("Test WS route called!");
    HttpResponse::Ok().body("WS route is working!")
}

// 简单的WebSocket测试路由
async fn simple_websocket_test_route(
    req: HttpRequest,
    stream: web::Payload,
) -> Result<HttpResponse, actix_web::Error> {
    log::info!("Simple WebSocket test route called!");
    ws::start(SimpleWsSession, &req, stream)
}

// 简单的WebSocket测试会话
struct SimpleWsSession;

impl actix::Actor for SimpleWsSession {
    type Context = ws::WebsocketContext<Self>;
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for SimpleWsSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Text(text)) => ctx.text(text),
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            _ => (),
        }
    }
}

// 路由配置函数
pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg
        // 健康检查路由
        .route("/api/health", web::get().to(health_check))
        // 就绪检查路由(用于Kubernetes readiness probe)
        .route("/api/health/ready", web::get().to(readiness_check))
        // 存活检查路由(用于Kubernetes liveness probe)
        .route("/api/health/live", web::get().to(liveness_check))
        // 监控状态路由
        .route("/metrics", web::get().to(metrics_endpoint))
        // 认证路由配置(无需认证，包含login、refresh和logout)
        .service(
            web::scope("/api/auth")
                .route("/login", web::post().to(auth::login))
                .route("/refresh", web::post().to(auth::refresh_token))
                .route("/logout", web::post().to(auth::logout))
        )
        // 测试路由 - 先测试HTTP GET
        .route("/ws/test", web::get().to(test_ws_route))
        // 简单的WebSocket测试路由
        .route(
            "/ws/simple",
            web::get().to(simple_websocket_test_route),
        )
        // WebSocket路由
        .route(
            "/ws",
            web::get().to(|req, stream, data| async move {
                websocket_server::websocket_index_route(req, stream, data).await
            }),
        )
        // 视频路由
        .configure(video::configure_video_routes)
        // 服务发现路由
        .configure(crate::load_balancing::configure_service_discovery_routes)
        // 动态速率限制配置路由
        .configure(dynamic_rate_config::configure_dynamic_config_routes)
        // 清理车辆路由
        .configure(clean_vehicles::configure_clean_vehicle_routes)
        // AI路由
        .configure(crate::ai::routes::configure_ai_routes)
        // API路由组
        .service(
            web::scope("/api")
                // 服务状态路由(允许 guest 访问)
                .route(
                    "/services/status",
                    web::get().to(services::get_services_status),
                )
                // 协议管理路由
                .configure(protocol::configure_protocol_routes)
                // JT808 指令路由
                .configure(jt808_command::configure_jt808_command_routes)
                // 需要认证的路由
                .service(
                    web::scope("")
                        // JWT认证中间件 - 验证token后放行
                        .wrap(HttpAuthentication::bearer(jwt_validator))
                        // 报警管理路由(用户级权限)
                        .configure(alerts::configure_alert_routes)
                        // 认证相关路由(用户级权限)
                        .route("/auth/user/{id}", web::get().to(auth::get_current_user))
                        // 称重数据路由(用户级权限)
                        .route("/weighing", web::get().to(weighing::get_weighing_data))
                        .route("/weighing", web::post().to(weighing::create_weighing_data))
                        .route(
                            "/weighing/{id}",
                            web::get().to(weighing::get_weighing_data_by_id),
                        )
                        .route(
                            "/weighing/{id}",
                            web::put().to(weighing::update_weighing_data),
                        )
                        .route(
                            "/weighing/{id}",
                            web::delete().to(weighing::delete_weighing_data),
                        )
                        .route(
                            "/weighing/history",
                            web::get().to(weighing::get_weighing_history),
                        )
                        // 统计路由(用户级权限)
                        .route(
                            "/statistics/vehicles",
                            web::get().to(statistics::get_vehicle_statistics),
                        )
                        .route(
                            "/statistics/devices",
                            web::get().to(statistics::get_device_statistics),
                        )
                        .route(
                            "/statistics/weighing",
                            web::get().to(statistics::get_weighing_statistics),
                        )
                        .route(
                            "/statistics/custom",
                            web::get().to(statistics::get_custom_statistics),
                        )
                        .route(
                            "/statistics/safety-ranking",
                            web::get().to(statistics::get_safety_ranking),
                        )
                        // 车辆管理路由(用户级权限)
                        .route("/vehicles", web::get().to(vehicles::get_vehicles))
                        .route("/vehicles", web::post().to(vehicles::create_vehicle))
                        .route("/vehicles/{id}", web::get().to(vehicles::get_vehicle))
                        .route("/vehicles/{id}", web::put().to(vehicles::update_vehicle))
                        .route("/vehicles/{id}", web::delete().to(vehicles::delete_vehicle))
                        // 订单管理路由(用户级权限)
                        .route("/orders", web::get().to(orders::get_orders))
                        .route("/orders", web::post().to(orders::create_order))
                        .route("/orders/{id}", web::get().to(orders::get_order))
                        .route("/orders/{id}", web::put().to(orders::update_order))
                        .route("/orders/{id}", web::delete().to(orders::delete_order))
                        .route(
                            "/orders/{order_id}/items",
                            web::post().to(orders::create_order_item),
                        )
                        .route(
                            "/orders/items/{item_id}",
                            web::put().to(orders::update_order_item),
                        )
                        .route(
                            "/orders/items/{item_id}",
                            web::delete().to(orders::delete_order_item),
                        )
                        .route(
                            "/orders/{order_id}/tracks",
                            web::post().to(orders::create_logistics_track),
                        )
                        .route(
                            "/orders/tracks/{track_id}",
                            web::put().to(orders::update_logistics_track),
                        )
                        .route(
                            "/orders/tracks/{track_id}",
                            web::delete().to(orders::delete_logistics_track),
                        )
                        .route(
                            "/orders/{order_id}/tracks/batch",
                            web::post().to(orders::create_logistics_tracks_batch),
                        )
                        .route("/tracks", web::get().to(orders::get_vehicle_tracks))
                        // 报表管理路由(用户级权限)
                        .configure(reports::configure_report_routes)
                        // 车组管理路由
                        .configure(vehicle_groups::configure_vehicle_group_routes)
                        // 角色管理路由
                        .configure(roles::configure_roles_routes)
                        // 部门管理路由
                        .configure(departments::configure_departments_routes)
                        // 司机管理路由
                        .configure(drivers::configure_driver_routes)
                        // 财务管理路由
                        .configure(finance::configure_finance_routes)
                        // 用户管理路由
                        .configure(users::configure_users_routes)
                        // 设备管理路由
                        .configure(devices::configure_devices_routes)
                        // 组织单位管理路由
                        .configure(organizations::configure_organizations_routes)
                        // OpenAPI 平台管理路由
                        .configure(openapi_platforms::configure_openapi_platforms_routes)
                        // OpenAPI 加载点路由
                        .configure(openapi_loading_points::configure_loading_points_routes)
                        // 位置管理路由
                        .configure(locations::configure_location_routes)
                        // 组织设置路由(用户级权限)
                        .configure(organization_settings::configure_organization_settings_routes)
                        // 系统设置路由(用户级权限)
                        .configure(settings::configure_settings_routes)
                        // 服务控制路由(用户级权限)
                        .configure(services::configure_services_control_routes)
                        // 仪表盘路由(用户级权限)
                        .configure(dashboard::configure_dashboard_routes)
                        // 灾备恢复路由(用户级权限)
                        .configure(disaster_recovery::configure_dr_routes)
                        // 蓝绿部署路由(用户级权限)
                        .configure(deployment::configure_deployment_routes)
                        // 载重标定路由(用户级权限)
                        .configure(weight_calibration::configure_weight_calibration_routes)
                        // 数据同步路由(用户级权限)
                        .route("/sync/execute", web::post().to(sync::execute_sync))
                        .route("/sync/status/{sync_id}", web::get().to(sync::get_sync_status))
                        .route("/sync/history", web::get().to(sync::get_sync_history))
                        .route("/sync/cancel/{sync_id}", web::post().to(sync::cancel_sync))
                        .route("/sync/history", web::delete().to(sync::clean_sync_history))
                        // 系统监控路由(用户级权限)
                        .configure(system_monitor::configure_system_monitor_routes),
                ),
        );
}
