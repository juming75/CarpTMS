//! / 动态速率限制配置API
use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::middleware::dynamic_config::{DynamicRateLimiterConfig, RateLimitUpdates};
use crate::middleware::rate_limiter::{LimitKeyType, RateLimiterConfig, StorageMode};

// 动态配置管理器(全局单例)
lazy_static::lazy_static! {
    static ref GLOBAL_RATE_LIMITER_CONFIG: Arc<DynamicRateLimiterConfig> =
        Arc::new(DynamicRateLimiterConfig::new(RateLimiterConfig::default()));
}

// 获取全局配置
pub fn get_global_config() -> Arc<DynamicRateLimiterConfig> {
    (*GLOBAL_RATE_LIMITER_CONFIG).clone()
}

// 初始化全局配置
pub async fn init_global_config(config: RateLimiterConfig) {
    (*GLOBAL_RATE_LIMITER_CONFIG)
        .update_config(RateLimitUpdates {
            limit: Some(config.limit),
            window_size: Some(config.window_size),
            burst: Some(config.burst),
            refill_rate: Some(config.refill_rate),
            storage_mode: Some(config.storage_mode),
            key_type: Some(config.key_type),
        })
        .await;
}

// 请求模型 - 更新配置
#[derive(Debug, Deserialize)]
pub struct UpdateConfigRequest {
    pub limit: Option<u32>,
    pub window_size_seconds: Option<u64>,
    pub burst: Option<u32>,
    pub refill_rate: Option<f64>,
    pub storage_mode: Option<String>,
    pub key_type: Option<String>,
    pub exempt_path: Option<String>,
    pub action: Option<String>, // "add" or "remove" for exempt_path
}

// 响应模型 - 配置摘要
#[derive(Debug, Serialize)]
pub struct ConfigSummaryResponse {
    pub success: bool,
    pub message: String,
    pub config: ConfigInfo,
}

#[derive(Debug, Serialize)]
pub struct ConfigInfo {
    pub limit: u32,
    pub window_size_seconds: u64,
    pub burst: u32,
    pub refill_rate: f64,
    pub storage_mode: String,
    pub key_type: String,
    pub exempt_paths_count: usize,
}

impl From<crate::middleware::dynamic_config::ConfigSummary> for ConfigInfo {
    fn from(summary: crate::middleware::dynamic_config::ConfigSummary) -> Self {
        Self {
            limit: summary.limit,
            window_size_seconds: summary.window_size.as_secs(),
            burst: summary.burst,
            refill_rate: summary.refill_rate,
            storage_mode: format!("{:?}", summary.storage_mode),
            key_type: format!("{:?}", summary.key_type),
            exempt_paths_count: summary.exempt_paths_count,
        }
    }
}

// 获取当前配置
pub async fn get_current_config() -> impl Responder {
    let summary = GLOBAL_RATE_LIMITER_CONFIG.get_summary().await;
    let response = ConfigSummaryResponse {
        success: true,
        message: "Current configuration retrieved".to_string(),
        config: summary.into(),
    };
    HttpResponse::Ok().json(response)
}

// 更新配置
pub async fn update_config(req: web::Json<UpdateConfigRequest>) -> impl Responder {
    let updates = req.into_inner();

    // 应用更新
    if let Some(limit) = updates.limit {
        (*GLOBAL_RATE_LIMITER_CONFIG).update_limit(limit).await;
    }

    if let Some(window_secs) = updates.window_size_seconds {
        (*GLOBAL_RATE_LIMITER_CONFIG)
            .update_window(std::time::Duration::from_secs(window_secs))
            .await;
    }

    if let Some(burst) = updates.burst {
        (*GLOBAL_RATE_LIMITER_CONFIG).update_burst(burst).await;
    }

    if let Some(refill_rate) = updates.refill_rate {
        (*GLOBAL_RATE_LIMITER_CONFIG)
            .update_refill_rate(refill_rate)
            .await;
    }

    if let Some(storage_mode_str) = updates.storage_mode {
        let storage_mode = match storage_mode_str.to_lowercase().as_str() {
            "memory" => StorageMode::Memory,
            "redis" => StorageMode::Redis,
            _ => {
                return HttpResponse::BadRequest().json(ConfigSummaryResponse {
                    success: false,
                    message: format!("Invalid storage mode: {}", storage_mode_str),
                    config: ConfigInfo {
                        limit: 0,
                        window_size_seconds: 0,
                        burst: 0,
                        refill_rate: 0.0,
                        storage_mode: "unknown".to_string(),
                        key_type: "unknown".to_string(),
                        exempt_paths_count: 0,
                    },
                })
            }
        };
        (*GLOBAL_RATE_LIMITER_CONFIG)
            .update_storage_mode(storage_mode)
            .await;
    }

    if let Some(key_type_str) = updates.key_type {
        let key_type = match key_type_str.to_lowercase().as_str() {
            "ip" => LimitKeyType::IP,
            "user" => LimitKeyType::User,
            _ => {
                return HttpResponse::BadRequest().json(ConfigSummaryResponse {
                    success: false,
                    message: format!("Invalid key type: {}", key_type_str),
                    config: ConfigInfo {
                        limit: 0,
                        window_size_seconds: 0,
                        burst: 0,
                        refill_rate: 0.0,
                        storage_mode: "unknown".to_string(),
                        key_type: "unknown".to_string(),
                        exempt_paths_count: 0,
                    },
                })
            }
        };
        (*GLOBAL_RATE_LIMITER_CONFIG)
            .update_key_type(key_type)
            .await;
    }

    // 处理豁免路径
    if let Some(path) = updates.exempt_path {
        match updates.action.as_deref() {
            Some("add") => {
                (*GLOBAL_RATE_LIMITER_CONFIG).add_exempt_path(path).await;
            }
            Some("remove") => {
                (*GLOBAL_RATE_LIMITER_CONFIG)
                    .remove_exempt_path(&path)
                    .await;
            }
            _ => {
                // 默认添加
                (*GLOBAL_RATE_LIMITER_CONFIG).add_exempt_path(path).await;
            }
        }
    }

    // 返回更新后的配置
    let summary = (*GLOBAL_RATE_LIMITER_CONFIG).get_summary().await;
    HttpResponse::Ok().json(ConfigSummaryResponse {
        success: true,
        message: "Configuration updated successfully".to_string(),
        config: summary.into(),
    })
}

// 重置配置为默认值
pub async fn reset_config() -> impl Responder {
    let default_config = RateLimiterConfig::default();
    init_global_config(default_config).await;

    let summary = (*GLOBAL_RATE_LIMITER_CONFIG).get_summary().await;
    HttpResponse::Ok().json(ConfigSummaryResponse {
        success: true,
        message: "Configuration reset to defaults".to_string(),
        config: summary.into(),
    })
}

// 配置路由
pub fn configure_dynamic_config_routes(cfg: &mut web::ServiceConfig) {
    cfg.route(
        "/admin/rate-limiter/config",
        web::get().to(get_current_config),
    )
    .route("/admin/rate-limiter/config", web::put().to(update_config))
    .route(
        "/admin/rate-limiter/config/reset",
        web::post().to(reset_config),
    );
}
