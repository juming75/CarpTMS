//! / OpenAPI/Swagger文档配置
use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};
use utoipa::{Modify, OpenApi};
use utoipa_swagger_ui::SwaggerUi;

/// OpenAPI安全配置
#[derive(OpenApi)]
#[openapi(
    info(
        title = "CarpTMS API",
        description = r#"
# 交通运输管理系统API文档

## 概述
CarpTMS (Transportation Management System) 是一个功能完备的交通运输管理系统,
支持车辆监控、订单管理、称重数据采集、视频监控等多种功能。

## 认证
大部分API需要JWT Bearer Token认证。请在请求头中添加:
```
Authorization: Bearer <your_jwt_token>
```

## CSRF保护
所有非安全方法(POST/PUT/DELETE)需要提供CSRF Token:
- GET请求会设置 `csrf_token` Cookie
- 后续POST/PUT/DELETE请求需要在Header中添加 `X-CSRF-Token`

## 错误响应格式
所有错误响应统一格式:
```json
{
  "success": false,
  "error": "错误描述信息",
  "error_type": "错误类型"
}
```

## 成功响应格式
```json
{
  "success": true,
  "data": { ... }
}
```

## 分页
分页响应包含额外字段:
```json
{
  "success": true,
  "data": [...],
  "pagination": {
    "page": 1,
    "page_size": 20,
    "total": 100,
    "total_pages": 5
  }
}
```
"#,
        version = "1.0.0",
        contact(
            name = "CarpTMS Team",
            email = "support@carptms.com"
        ),
        license(
            name = "MIT",
            url = "https://opensource.org/licenses/MIT"
        )
    ),
    paths(
        // 健康检查
        crate::health_check,
        crate::metrics_endpoint,

        // 车辆管理API
        crate::routes::vehicles::get_vehicles,
        crate::routes::vehicles::get_vehicle,
        crate::routes::vehicles::create_vehicle,
        crate::routes::vehicles::update_vehicle,
        crate::routes::vehicles::delete_vehicle,

        // 订单管理API
        crate::routes::orders::list_orders,
        crate::routes::orders::get_order,
        crate::routes::orders::create_order,
        crate::routes::orders::update_order,
        crate::routes::orders::delete_order,
        crate::routes::orders::update_order_status,

        // 用户管理API
        crate::routes::users::list_users,
        crate::routes::users::get_user,
        crate::routes::users::create_user,
        crate::routes::users::update_user,
        crate::routes::users::delete_user,

        // 设备管理API
        crate::routes::devices::list_devices,
        crate::routes::devices::get_device,
        crate::routes::devices::create_device,
        crate::routes::devices::update_device,
        crate::routes::devices::delete_device,

        // 称重数据API
        crate::routes::weighing::list_weighing_data,
        crate::routes::weighing::get_weighing_record,
        crate::routes::weighing::create_weighing_record,

        // 视频监控API
        crate::video::video_manager::list_streams,
        crate::video::video_manager::get_stream_info,
        crate::video::video_manager::start_stream,
        crate::video::video_manager::stop_stream,

        // BFF API
        crate::bff::mod::dashboard_stats,
        crate::bff::mod::vehicle_map,
        crate::bff::mod::recent_alerts,
        crate::bff::mod::performance_metrics,

        // 实时数据API
        crate::websocket::optimized::websocket_handler,

        // 认证API
        crate::routes::auth::login,
        crate::routes::auth::refresh_token,
        crate::routes::auth::logout,
        crate::routes::auth::get_current_user_by_token,

        // 审计日志API
        crate::routes::audit::get_audit_logs,

        // 监控API
        crate::routes::monitoring::get_system_status,
        crate::routes::monitoring::get_switching_history,
        crate::routes::monitoring::get_switching_recommendation,
        crate::routes::monitoring::manual_switch,

        // 动态限流配置API
        crate::routes::dynamic_rate_config::get_rate_config,
        crate::routes::dynamic_rate_config::update_rate_config,

        // 报警管理API
        crate::routes::alerts::list_alerts,
        crate::routes::alerts::get_alert,
        crate::routes::alerts::acknowledge_alert,

        // 系统监控API
        crate::routes::system_monitor::configure_system_monitor_routes,

        // ML预测API
        crate::ml::routes::handle_predict,
        crate::ml::routes::list_models,
        crate::ml::routes::handle_train,
    ),
    components(
        schemas(
            // 通用响应
            crate::SuccessResponse,
            crate::ErrorResponse,
            crate::PaginationResponse,

            // 车辆相关
            crate::models::Vehicle,
            crate::models::VehicleStatus,
            crate::models::VehicleLocation,
            crate::models::VehicleQuery,

            // 订单相关
            crate::models::Order,
            crate::models::OrderStatus,
            crate::models::OrderQuery,

            // 用户相关
            crate::models::User,
            crate::models::UserRole,

            // 设备相关
            crate::models::Device,
            crate::models::DeviceType,

            // 称重数据
            crate::models::WeighingData,

            // 视频相关
            crate::video::StreamInfo,
            crate::video::StreamConfig,

            // 报表相关
            crate::bff::mod::DashboardStats,
            crate::bff::mod::PerformanceMetrics,

            // 监控相关
            crate::routes::monitoring::SystemStatusResponse,
            crate::routes::monitoring::SystemStatus,
        )
    ),
    tags(
        (name = "健康检查", description = "系统健康检查端点"),
        (name = "车辆管理", description = "车辆的增删改查和状态监控"),
        (name = "订单管理", description = "订单的创建和状态管理"),
        (name = "用户管理", description = "用户账户管理"),
        (name = "设备管理", description = "车载设备管理"),
        (name = "称重数据", description = "称重数据采集和查询"),
        (name = "视频监控", description = "视频流管理和播放"),
        (name = "BFF API", description = "前端后端接口"),
        (name = "实时数据", description = "WebSocket实时数据推送"),
        (name = "认证", description = "用户认证和会话管理"),
        (name = "审计日志", description = "操作审计日志查询"),
        (name = "系统监控", description = "系统状态监控和架构切换"),
        (name = "限流配置", description = "动态速率限制配置"),
        (name = "报警管理", description = "系统报警和告警管理"),
        (name = "AI预测", description = "机器学习模型推理"),
    )
)]
pub struct ApiDoc;

/// 修改OpenAPI文档以添加JWT认证
pub struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build()
                ),
            )
        }
    }
}

/// 配置Swagger UI
pub fn configure_swagger(cfg: &mut actix_web::web::ServiceConfig) {
    SwaggerUi::new("/swagger-ui/{_:.*}")
        .url("/api-docs/openapi.json", ApiDoc::openapi())
        .map(
            utoipa_swagger_ui::Config::from(
                "/api-docs/openapi.json"
            )
        )
        .persist_authorization(true)
        .configure(cfg);
}
