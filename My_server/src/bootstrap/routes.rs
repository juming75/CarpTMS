//! 路由配置模块
//!
//! 负责配置所有HTTP路由

use actix_web::web;
use crate::{
    health_check, metrics_endpoint, middleware, ApiDoc,
    routes,
};
use super::services::ApplicationState;

/// 配置应用所有路由
pub fn configure_app_routes(cfg: &mut web::ServiceConfig, state: ApplicationState) -> Result<(), Box<dyn std::error::Error>> {
    cfg.service(
        // 添加Swagger UI
        utoipa_swagger_ui::SwaggerUi::new("/docs/{_:.*}")
            .url("/api-docs/openapi.json", ApiDoc::openapi()),
    )
    // 健康检查路由
    .route("/api/health", web::get().to(health_check))
    // 监控状态路由
    .route("/metrics", web::get().to(metrics_endpoint))
    // 认证路由（无需认证）
    .route(
        "/api/auth/login",
        web::post().to(routes::auth::login),
    )
    .route(
        "/api/auth/refresh",
        web::post().to(routes::auth::refresh_token),
    )
    // WebSocket路由
    .route(
        "/ws",
        web::get().to(|req, stream, data: web::Data<routes::gateway::websocket_server::WsAppState>| {
            routes::gateway::websocket_server::websocket_index_route(req, stream, data)
        }),
    )
    // BFF路由（无需认证，供前端直接调用）
    .app_data(web::Data::new(state.vehicle_aggregator.clone()))
    .app_data(web::Data::new(state.report_service.clone()))
    .app_data(web::Data::new(state.template_engine.clone()))
    .app_data(web::Data::new(state.video_service.clone()))
    .app_data(web::Data::new(state.video_service.recording_manager.clone()))
    .configure(routes::bff::routes::configure_bff_routes)
    .configure(routes::video::configure_video_routes)
    // 需要认证的路由组
    .service(
        web::scope("/api")
            .configure(|cfg| configure_authenticated_routes(cfg, state.clone()))
    )
    // 服务发现路由
    .configure(crate::load_balancing::configure_service_discovery_routes)
    // 动态速率限制配置路由
    .configure(routes::dynamic_rate_config::configure_dynamic_config_routes)
    
    Ok(())
}

/// 配置需要认证的路由
fn configure_authenticated_routes(cfg: &mut web::ServiceConfig, state: ApplicationState) {
    // 称重数据路由
    cfg.route("/weighing", web::get().to(routes::weighing::get_weighing_data))
    //        .wrap(middleware::auth::AuthMiddleware::user()
    //            .resource_str("weighing")
    //            .action_str("read"))
        .route("/weighing", web::post().to(routes::weighing::create_weighing_data))
    //        .wrap(middleware::auth::AuthMiddleware::manager()
    //            .resource_str("weighing")
    //            .action_str("create"))
        .route("/weighing/history", web::get().to(routes::weighing::get_weighing_history))
    //        .wrap(middleware::auth::AuthMiddleware::user()
    //            .resource_str("weighing")
    //            .action_str("read"))
        .route("/weighing/{id}", web::get().to(routes::weighing::get_weighing_data_by_id))
    //        .wrap(middleware::auth::AuthMiddleware::user()
    //            .resource_str("weighing")
    //            .action_str("read"))
        .route("/weighing/{id}", web::put().to(routes::weighing::update_weighing_data))
    //        .wrap(middleware::auth::AuthMiddleware::manager()
    //            .resource_str("weighing")
    //            .action_str("update"))
        .route("/weighing/{id}", web::delete().to(routes::weighing::delete_weighing_data));
    //        .wrap(middleware::auth::AuthMiddleware::manager()
    //            .resource_str("weighing")
    //            .action_str("delete"))

    // 车辆管理路由
    cfg.route("/vehicles", web::get().to(routes::vehicles::get_vehicles))
    //        .wrap(middleware::auth::AuthMiddleware::user()
    //            .resource_str("vehicle")
    //            .action_str("read"))
        .route("/vehicles", web::post().to(routes::vehicles::create_vehicle))
    //        .wrap(middleware::auth::AuthMiddleware::manager()
    //            .resource_str("vehicle")
    //            .action_str("create"))
        .route("/vehicles/{id}", web::get().to(routes::vehicles::get_vehicle))
    //        .wrap(middleware::auth::AuthMiddleware::user()
    //            .resource_str("vehicle")
    //            .action_str("read"))
        .route("/vehicles/{id}", web::put().to(routes::vehicles::update_vehicle))
    //        .wrap(middleware::auth::AuthMiddleware::manager()
    //            .resource_str("vehicle")
    //            .action_str("update"))
        .route("/vehicles/{id}", web::delete().to(routes::vehicles::delete_vehicle))
    //        .wrap(middleware::auth::AuthMiddleware::manager()
    //            .resource_str("vehicle")
    //            .action_str("delete"))

    // 车组管理路由
    cfg.route("/vehicle-groups", web::get().to(routes::vehicle_groups::get_vehicle_groups))
    //        .wrap(middleware::auth::AuthMiddleware::manager()
    //            .resource_str("vehicle_group")
    //            .action_str("read"))
        .route("/vehicle-groups", web::post().to(routes::vehicle_groups::create_vehicle_group))
    //        .wrap(middleware::auth::AuthMiddleware::manager()
    //            .resource_str("vehicle_group")
    //            .action_str("create"))
        .route("/vehicle-groups/{id}", web::get().to(routes::vehicle_groups::get_vehicle_group))
    //        .wrap(middleware::auth::AuthMiddleware::manager()
    //            .resource_str("vehicle_group")
    //            .action_str("read"))
        .route("/vehicle-groups/{id}", web::put().to(routes::vehicle_groups::update_vehicle_group))
    //        .wrap(middleware::auth::AuthMiddleware::manager()
    //            .resource_str("vehicle_group")
    //            .action_str("update"))
        .route("/vehicle-groups/{id}", web::delete().to(routes::vehicle_groups::delete_vehicle_group))
    //        .wrap(middleware::auth::AuthMiddleware::manager()
    //            .resource_str("vehicle_group")
    //            .action_str("delete"))
        .route("/vehicle-groups/tree", web::get().to(routes::vehicle_groups::get_vehicle_group_tree))
    //        .wrap(middleware::auth::AuthMiddleware::manager()
    //            .resource_str("vehicle_group")
    //            .action_str("read"))

    // 用户组管理路由
    cfg.route("/user-groups", web::get().to(routes::user_groups::get_user_groups))
    //        .wrap(middleware::auth::AuthMiddleware::manager()
    //            .resource_str("user_group")
    //            .action_str("read"))
        .route("/user-groups", web::post().to(routes::user_groups::create_user_group))
    //        .wrap(middleware::auth::AuthMiddleware::manager()
    //            .resource_str("user_group")
    //            .action_str("create"))
        .route("/user-groups/{id}", web::get().to(routes::user_groups::get_user_group))
    //        .wrap(middleware::auth::AuthMiddleware::manager()
    //            .resource_str("user_group")
    //            .action_str("read"))
        .route("/user-groups/{id}", web::put().to(routes::user_groups::update_user_group))
    //        .wrap(middleware::auth::AuthMiddleware::manager()
    //            .resource_str("user_group")
    //            .action_str("update"))
        .route("/user-groups/{id}", web::delete().to(routes::user_groups::delete_user_group))
    //        .wrap(middleware::auth::AuthMiddleware::manager()
    //            .resource_str("user_group")
    //            .action_str("delete"))

    // 用户管理路由
    cfg.route("/users", web::get().to(routes::users::get_users))
    //        .wrap(middleware::auth::AuthMiddleware::manager()
    //            .resource_str("user")
    //            .action_str("read"))
        .route("/users", web::post().to(routes::users::create_user))
    //        .wrap(middleware::auth::AuthMiddleware::manager()
    //            .resource_str("user")
    //            .action_str("create"))
        .route("/users/{id}", web::get().to(routes::users::get_user))
    //        .wrap(middleware::auth::AuthMiddleware::manager()
    //            .resource_str("user")
    //            .action_str("read"))
        .route("/users/{id}", web::put().to(routes::users::update_user))
    //        .wrap(middleware::auth::AuthMiddleware::manager()
    //            .resource_str("user")
    //            .action_str("update"))
        .route("/users/{id}", web::delete().to(routes::users::delete_user))
    //        .wrap(middleware::auth::AuthMiddleware::manager()
    //            .resource_str("user")
    //            .action_str("delete"))

    // 报表管理路由
    cfg.route("/reports/templates", web::get().to(routes::reports::get_report_templates))
    //        .wrap(middleware::auth::AuthMiddleware::user()
    //            .resource_str("report")
    //            .action_str("read"))
        .route("/reports/data", web::get().to(routes::reports::get_report_data))
    //        .wrap(middleware::auth::AuthMiddleware::user()
    //            .resource_str("report")
    //            .action_str("read"))
        .route("/reports/generate", web::post().to(routes::reports::generate_report))
    //        .wrap(middleware::auth::AuthMiddleware::user()
    //            .resource_str("report")
    //            .action_str("create"))
}
