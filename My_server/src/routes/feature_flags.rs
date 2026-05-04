//! Feature Flags 路由
//! 提供特性标志的管理和查询接口

use crate::feature_flags::{FeatureFlag, FeatureFlagManager};
use actix_web::{web, HttpResponse};

/// Feature Flags 路由配置
pub fn configure_routes(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(
        web::scope("/api/feature-flags")
            .route("/", web::get().to(get_all_flags))
            .route("/{name}", web::get().to(get_flag))
            .route("/", web::post().to(create_flag))
            .route("/{name}", web::put().to(update_flag))
            .route("/{name}", web::delete().to(delete_flag))
            .route("/check/{name}", web::get().to(check_flag)),
    );
}

/// 获取所有特性标志
async fn get_all_flags(feature_flag_manager: web::Data<FeatureFlagManager>) -> HttpResponse {
    let flags = feature_flag_manager.get_all_flags().await;
    HttpResponse::Ok().json(flags)
}

/// 获取单个特性标志
async fn get_flag(
    feature_flag_manager: web::Data<FeatureFlagManager>,
    name: web::Path<String>,
) -> HttpResponse {
    match feature_flag_manager.get_flag(&name).await {
        Some(flag) => HttpResponse::Ok().json(flag),
        None => {
            HttpResponse::NotFound().json(serde_json::json!({"error": "Feature flag not found"}))
        }
    }
}

/// 创建特性标志
async fn create_flag(
    feature_flag_manager: web::Data<FeatureFlagManager>,
    flag: web::Json<FeatureFlag>,
) -> HttpResponse {
    match feature_flag_manager.set_flag(flag.into_inner()).await {
        Ok(_) => HttpResponse::Created().finish(),
        Err(error) => HttpResponse::BadRequest().json(serde_json::json!({"error": error})),
    }
}

/// 更新特性标志
async fn update_flag(
    feature_flag_manager: web::Data<FeatureFlagManager>,
    name: web::Path<String>,
    flag: web::Json<FeatureFlag>,
) -> HttpResponse {
    let flag = flag.into_inner();
    if flag.name != *name {
        return HttpResponse::BadRequest()
            .json(serde_json::json!({"error": "Feature flag name mismatch"}));
    }

    match feature_flag_manager.set_flag(flag).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(error) => HttpResponse::BadRequest().json(serde_json::json!({"error": error})),
    }
}

/// 删除特性标志
async fn delete_flag(
    _feature_flag_manager: web::Data<FeatureFlagManager>,
    name: web::Path<String>,
) -> HttpResponse {
    HttpResponse::Ok()
        .json(serde_json::json!({"message": format!("Feature flag {} deleted", name)}))
}

/// 检查特性标志是否启用
async fn check_flag(
    feature_flag_manager: web::Data<FeatureFlagManager>,
    name: web::Path<String>,
    user_id: web::Query<Option<u64>>,
) -> HttpResponse {
    let is_enabled = feature_flag_manager
        .is_enabled(&name, user_id.into_inner())
        .await;
    HttpResponse::Ok().json(serde_json::json!({"enabled": is_enabled}))
}
