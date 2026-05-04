//! / JT808 指令路由
// 将 JT808 指令 API 集成到路由系统

use actix_web::web;
// TODO: Remove or configure proper jt808 command routes
// use tms_server::api::jt808_command::configure_jt808_routes;

/// 配置 JT808 指令路由
pub fn configure_jt808_command_routes(_cfg: &mut web::ServiceConfig) {
    // cfg.service(
    //     web::scope("/v1")
    //         .configure(configure_jt808_routes)
    // );
}
