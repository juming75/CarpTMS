//! API Gateway module
//! Provides unified authentication, rate limiting, and circuit breaking

use actix_web::{web, App, HttpResponse, HttpServer};
use std::sync::Arc;
use redis::Client as RedisClient;

use crate::middleware::{rate_limiter::RateLimiterMiddleware, circuit_breaker::CircuitBreaker};
use crate::routes::configure_routes;
use crate::init::finalize::ServerState;

/// API Gateway configuration
#[derive(Clone)]
pub struct ApiGatewayConfig {
    pub address: String,
    pub server_state: ServerState,
    pub redis_client: Arc<RedisClient>,
    pub circuit_breaker: Arc<CircuitBreaker>,
}

/// API Gateway service
pub struct ApiGateway {
    config: ApiGatewayConfig,
}

impl ApiGateway {
    /// Create a new API Gateway instance
    pub fn new(config: ApiGatewayConfig) -> Self {
        Self {
            config,
        }
    }

    /// Start the API Gateway server
    pub async fn start(&self) -> std::io::Result<()> {
        let config = self.config.clone();
        
        HttpServer::new(move || {
            let redis_client = config.redis_client.clone();
            let circuit_breaker = config.circuit_breaker.clone();
            let server_state = config.server_state.clone();
            
            App::new()
                // Add data to the app
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
                .app_data(web::Data::new(server_state.order_service.clone()))
                .app_data(web::Data::new(server_state.finance_service.clone()))
                .app_data(web::Data::new(server_state.role_service.clone()))
                .app_data(web::Data::new(server_state.vehicle_group_service.clone()))
                .app_data(web::Data::new(server_state.openapi_platform_service.clone()))
                .app_data(web::Data::new(server_state.system_monitor_service.clone()))
                .app_data(web::Data::new(server_state.audit_log_service.clone()))
                .app_data(web::Data::new(server_state.settings_service.clone()))
                .app_data(web::Data::new(redis_client.clone()))
                .app_data(web::Data::new(circuit_breaker.clone()))
                
                // API Gateway middleware
                .wrap(RateLimiterMiddleware::new(
                    crate::middleware::rate_limiter::RateLimiterConfig::default(),
                    redis_client.clone(),
                ))
                .wrap(crate::middleware::resource_limiter::resource_limiter_middleware())
                .wrap(crate::tracing::TracingMiddleware::new())
                .wrap(crate::logging::LoggingMiddleware::new(
                    crate::logging::get_log_manager().expect("Log manager should be initialized"),
                ))
                .wrap(crate::middleware::request_logger::RequestLogger::new())
                .wrap(actix_web::middleware::Logger::default())
                .wrap(crate::app::build_cors(&server_state.config.security.allowed_origins))
                .wrap(crate::middleware::request_header::default_header_middleware())
                .wrap(crate::middleware::metrics::MetricsMiddleware::new())
                
                // Configure all routes
                .configure(configure_routes)
        })
        .bind(&config.address)?
        .run()
        .await
    }
}

/// Configure API Gateway routes
pub fn configure_api_gateway_routes(cfg: &mut web::ServiceConfig) {
    // API Gateway routes will be handled by the main route configuration
    // This function is a placeholder for any additional gateway-specific routes
    cfg
        .route("/gateway/health", web::get().to(|| async { HttpResponse::Ok().body("API Gateway is healthy") }));
}
