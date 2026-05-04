//! /! 健康检查处理器
//!
//! 策略:优先复用已有 PgPool 连接池(零额外开销),
//! 仅在直接调用(无 app_data)时创建临时连接。

use super::enhanced::get_enhanced_health_checker;
use actix_web::{web, HttpResponse};
use sqlx::PgPool;

/// 健康检查端点(深度检查)
///
/// # 依赖
/// - `web::Data<PgPool>`: 从 app_data 注入已有连接池(推荐路径)
///
/// # 行为
/// - 数据库连接正常 → 200
/// - 数据库连接失败 → 500
/// - Redis 可用 → status.redis = "ok",不可用 → "warn"(不影响整体状态)
/// - 包含缓存命中率检查
#[utoipa::path(
    get,
    path = "/api/health",
    responses(
        (status = 200, description = "服务健康"),
        (status = 500, description = "服务不健康")
    )
)]
pub async fn health_check(pool: Option<web::Data<PgPool>>) -> HttpResponse {
    // 复用池标记提前取出(避免 move 后再借用)
    let has_pool = pool.is_some();

    // 策略1:从 app_data 复用已有连接池(推荐)
    let db_status = if let Some(pool) = pool {
        let status = match sqlx::query("SELECT 1").fetch_one(pool.get_ref()).await {
            Ok(_) => (String::from("ok"), String::new()),
            Err(e) => {
                let msg = format!("{:?}", e);
                (String::from("error"), msg)
            }
        };
        status
    } else {
        // 策略2:直接调用(fallback,保持向后兼容)
        let config = crate::config::unified::manager::get_config();
        match sqlx::PgPool::connect(&config.database.url).await {
            Ok(temp_pool) => match sqlx::query("SELECT 1").fetch_one(&temp_pool).await {
                Ok(_) => (String::from("ok"), String::new()),
                Err(e) => (String::from("error"), format!("{:?}", e)),
            },
            Err(e) => (String::from("error"), format!("{:?}", e)),
        }
    };

    let redis_status = match crate::redis::is_redis_available().await {
        true => "ok",
        false => "warn",
    };

    // 获取缓存命中率
    let cache_stats = get_cache_stats().await;

    let overall_status = if db_status.0 == "ok" { "ok" } else { "error" };

    let response = serde_json::json!({
        "status": overall_status,
        "service": "tms_server",
        "version": "0.1.0",
        "timestamp": chrono::Local::now().naive_local(),
        "dependencies": {
            "database": db_status.0,
            "redis": redis_status
        },
        "cache": {
            "hit_rate": cache_stats.hit_rate,
            "hits": cache_stats.hits,
            "misses": cache_stats.misses
        },
        "errors": {
            "database": db_status.1
        },
        "hostname": hostname::get()
            .ok()
            .and_then(|h: std::ffi::OsString| h.into_string().ok())
            .unwrap_or_else(|| "unknown".to_string()),
        "pool_source": if has_pool { "shared" } else { "direct" }
    });

    if overall_status == "ok" {
        HttpResponse::Ok().json(response)
    } else {
        HttpResponse::InternalServerError().json(response)
    }
}

/// 就绪检查端点(用于Kubernetes readiness probe)
///
/// # 依赖
/// - `web::Data<PgPool>`: 从 app_data 注入已有连接池
///
/// # 行为
/// - 数据库和Redis都可用 → 200
/// - 任一依赖不可用 → 503
#[utoipa::path(
    get,
    path = "/api/health/ready",
    responses(
        (status = 200, description = "服务就绪"),
        (status = 503, description = "服务未就绪")
    )
)]
pub async fn readiness_check(pool: web::Data<PgPool>) -> HttpResponse {
    // 检查数据库连接
    let db_status = match sqlx::query("SELECT 1").fetch_one(pool.get_ref()).await {
        Ok(_) => "ok",
        Err(e) => {
            log::error!("Database readiness check failed: {:?}", e);
            "error"
        }
    };

    // 检查Redis连接
    let redis_status = match crate::redis::is_redis_available().await {
        true => "ok",
        false => {
            log::error!("Redis readiness check failed");
            "error"
        }
    };

    let overall_status = if db_status == "ok" && redis_status == "ok" {
        "ok"
    } else {
        "error"
    };

    let response = serde_json::json!({
        "status": overall_status,
        "service": "tms_server",
        "timestamp": chrono::Local::now().naive_local(),
        "dependencies": {
            "database": db_status,
            "redis": redis_status
        }
    });

    if overall_status == "ok" {
        HttpResponse::Ok().json(response)
    } else {
        HttpResponse::ServiceUnavailable().json(response)
    }
}

/// 存活检查端点(用于Kubernetes liveness probe)
///
/// # 行为
/// - 服务运行中 → 200
/// - 服务未运行 → 503
#[utoipa::path(
    get,
    path = "/api/health/live",
    responses(
        (status = 200, description = "服务存活"),
        (status = 503, description = "服务未存活")
    )
)]
pub async fn liveness_check() -> HttpResponse {
    let response = serde_json::json!({
        "status": "ok",
        "service": "tms_server",
        "timestamp": chrono::Local::now().naive_local()
    });

    HttpResponse::Ok().json(response)
}

/// 缓存统计信息
struct CacheStats {
    hit_rate: f64,
    hits: u64,
    misses: u64,
}

/// 获取缓存统计信息
async fn get_cache_stats() -> CacheStats {
    // 尝试检查Redis连接
    let redis_available = crate::redis::is_redis_available().await;

    // 由于VehicleCache没有统计方法,我们返回基本的缓存状态
    CacheStats {
        hit_rate: if redis_available { 0.0 } else { -1.0 },
        hits: 0,
        misses: 0,
    }
}

/// Prometheus指标端点
///
/// # 行为
/// - 返回Prometheus格式的指标数据 → 200
#[utoipa::path(
    get,
    path = "/metrics",
    responses(
        (status = 200, description = "Prometheus metrics"),
    )
)]
pub async fn metrics_endpoint() -> HttpResponse {
    use prometheus::Encoder;

    // 创建一个TextEncoder来编码指标
    let encoder = prometheus::TextEncoder::new();

    // 获取所有注册的指标
    let metric_families = prometheus::gather();

    // 编码指标
    let mut buffer = Vec::new();
    if let Err(e) = encoder.encode(&metric_families, &mut buffer) {
        tracing::error!("Failed to encode metrics: {:?}", e);
        return HttpResponse::InternalServerError().body("Failed to encode metrics");
    }

    // 返回编码后的指标
    HttpResponse::Ok()
        .content_type("text/plain; version=0.0.4")
        .body(buffer)
}

/// 增强的健康检查端点
#[utoipa::path(
    get,
    path = "/api/health/enhanced",
    responses(
        (status = 200, description = "详细的健康状态"),
        (status = 503, description = "服务不健康")
    )
)]
pub async fn enhanced_health_check() -> HttpResponse {
    let checker = get_enhanced_health_checker();
    let health_status = checker.check_health().await;

    let mut status_code = match health_status.status.as_str() {
        "ok" => HttpResponse::Ok(),
        "warn" => HttpResponse::Ok(),
        _ => HttpResponse::ServiceUnavailable(),
    };

    status_code.json(health_status)
}

/// 获取健康检查历史
#[utoipa::path(
    get,
    path = "/api/health/history",
    responses(
        (status = 200, description = "健康检查历史记录"),
    )
)]
pub async fn health_history() -> HttpResponse {
    let checker = get_enhanced_health_checker();
    let history = checker.get_check_history(50); // 获取最近50条记录
    HttpResponse::Ok().json(history)
}

/// 更新健康检查动态配置
#[utoipa::path(
    put,
    path = "/api/health/config",
    responses(
        (status = 200, description = "配置更新成功"),
        (status = 400, description = "无效配置")
    )
)]
pub async fn update_health_config(
    config: web::Json<super::enhanced::DynamicHealthConfig>,
) -> HttpResponse {
    let checker = get_enhanced_health_checker();
    checker.set_config(config.into_inner());
    HttpResponse::Ok().json(serde_json::json!({
        "status": "success",
        "message": "Health check configuration updated"
    }))
}

/// 获取当前健康检查配置
#[utoipa::path(
    get,
    path = "/api/health/config",
    responses(
        (status = 200, description = "当前配置"),
    )
)]
pub async fn get_health_config() -> HttpResponse {
    let checker = get_enhanced_health_checker();
    let config = checker.get_config();
    HttpResponse::Ok().json(config)
}
