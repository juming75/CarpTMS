//! 路由配置模块

use actix_web::web;
use crate::{
    health_check, metrics_endpoint, middleware, ApiDoc,
    routes,
};
use super::services::ApplicationState;

/// 配置应用所有路由
pub fn configure_app_routes(cfg: &mut web::ServiceConfig, state: ApplicationState) -> Result<(), Box<dyn std::error::Error>> {
    cfg.service(
        utoipa_swagger_ui::SwaggerUi::new("/docs/{_:.*}")
            .url("/api-docs/openapi.json", ApiDoc::openapi()),
    )
    .route("/api/health", web::get().to(health_check))
    .route("/metrics", web::get().to(metrics_endpoint))
    .route(
        "/ws",
        web::get().to(|req, stream, data: web::Data<routes::gateway::websocket_server::WsAppState>| {
            routes::gateway::websocket_server::websocket_index_route(req, stream, data)
        }),
    )
    .app_data(web::Data::new(state.vehicle_aggregator.clone()))
    .app_data(web::Data::new(state.report_service.clone()))
    .app_data(web::Data::new(state.template_engine.clone()))
    .app_data(web::Data::new(state.video_service.clone()))
    .app_data(web::Data::new(state.video_service.recording_manager.clone()))
    .configure(routes::bff::routes::configure_bff_routes)
    .configure(routes::video::configure_video_routes)
    // 认证路由 - 直接注册，不使用 scope
    .route("/api/auth/login", web::post().to(routes::auth::login))
    .route("/api/auth/refresh", web::post().to(routes::auth::refresh_token))
    .route("/api/auth/logout", web::post().to(routes::auth::logout))
    .route("/api/auth/change-password", web::post().to(routes::auth::change_password))
    .route("/api/auth/user", web::get().to(routes::auth::get_current_user_by_token))
    .route("/api/auth/user/{id}", web::get().to(routes::auth::get_current_user))
    // 业务路由 - 直接注册
    .route("/api/weighing", web::get().to(routes::weighing::get_weighing_data))
    .route("/api/weighing", web::post().to(routes::weighing::create_weighing_data))
    .route("/api/weighing/history", web::get().to(routes::weighing::get_weighing_history))
    .route("/api/weighing/{id}", web::get().to(routes::weighing::get_weighing_data_by_id))
    .route("/api/weighing/{id}", web::put().to(routes::weighing::update_weighing_data))
    .route("/api/weighing/{id}", web::delete().to(routes::weighing::delete_weighing_data))
    .route("/api/vehicles", web::get().to(routes::vehicles::get_vehicles))
    .route("/api/vehicles", web::post().to(routes::vehicles::create_vehicle))
    .route("/api/vehicles/{id}", web::get().to(routes::vehicles::get_vehicle))
    .route("/api/vehicles/{id}", web::put().to(routes::vehicles::update_vehicle))
    .route("/api/vehicles/{id}", web::delete().to(routes::vehicles::delete_vehicle))
    .route("/api/vehicle-groups", web::get().to(routes::vehicle_groups::get_vehicle_groups))
    .route("/api/vehicle-groups", web::post().to(routes::vehicle_groups::create_vehicle_group))
    .route("/api/vehicle-groups/{id}", web::get().to(routes::vehicle_groups::get_vehicle_group))
    .route("/api/vehicle-groups/{id}", web::put().to(routes::vehicle_groups::update_vehicle_group))
    .route("/api/vehicle-groups/{id}", web::delete().to(routes::vehicle_groups::delete_vehicle_group))
    .route("/api/vehicle-groups/tree", web::get().to(routes::vehicle_groups::get_vehicle_group_tree))
    .route("/api/user-groups", web::get().to(routes::user_groups::get_user_groups))
    .route("/api/user-groups", web::post().to(routes::user_groups::create_user_group))
    .route("/api/user-groups/{id}", web::get().to(routes::user_groups::get_user_group))
    .route("/api/user-groups/{id}", web::put().to(routes::user_groups::update_user_group))
    .route("/api/user-groups/{id}", web::delete().to(routes::user_groups::delete_user_group))
    .route("/api/users", web::get().to(routes::users::get_users))
    .route("/api/users", web::post().to(routes::users::create_user))
    .route("/api/users/{id}", web::get().to(routes::users::get_user))
    .route("/api/users/{id}", web::put().to(routes::users::update_user))
    .route("/api/users/{id}", web::delete().to(routes::users::delete_user))
    .route("/api/reports/templates", web::get().to(routes::reports::get_report_templates))
    .route("/api/reports/data", web::get().to(routes::reports::get_report_data))
    .route("/api/reports/generate", web::post().to(routes::reports::generate_report))
    .configure(crate::load_balancing::configure_service_discovery_routes)
    .configure(routes::dynamic_rate_config::configure_dynamic_config_routes)
    .configure(routes::monitoring::configure_routes)
    .configure(routes::audit::configure_routes);

    Ok(())
}
