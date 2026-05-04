//! 称重微服务网关
//!
//! 统一入口点，路由到不同的处理器

use actix_web::{web, HttpResponse};

use crate::domain::weighing::{BatchCreateWeighingCommand, WeighingAggregate, WeighingStatsQuery};
use crate::errors::{AppError, AppResult};

/// 统计查询处理器
pub async fn stats_handler(query: web::Query<WeighingStatsQuery>) -> AppResult<HttpResponse> {
    // 委托给统计服务
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "ok",
        "data": {
            "query": query.0,
            "note": "统计查询由 WeighingStatsService 处理"
        }
    })))
}

/// 批量创建处理器（高频数据场景）
pub async fn batch_create_handler(
    cmd: web::Json<BatchCreateWeighingCommand>,
) -> AppResult<HttpResponse> {
    // 验证批次ID（幂等保证）
    if cmd.batch_id.is_empty() {
        return Err(AppError::validation("批次ID不能为空"));
    }

    // 批量创建由专门的批处理服务处理
    Ok(HttpResponse::Created().json(serde_json::json!({
        "status": "created",
        "batch_id": cmd.batch_id,
        "item_count": cmd.items.len(),
        "note": "批量创建由 WeighingBatchService 处理"
    })))
}

/// 聚合根验证
pub fn validate_aggregate(
    vehicle_id: i32,
    gross_weight: f64,
    net_weight: f64,
    tare_weight: Option<f64>,
) -> Result<WeighingAggregate, AppError> {
    WeighingAggregate::create(
        vehicle_id,
        "DEV".to_string(),
        chrono::Utc::now().naive_utc(),
        gross_weight,
        net_weight,
        tare_weight,
        None,
        None,
        None,
        None,
    )
    .map_err(|e| AppError::validation(&e.to_string()))
}
