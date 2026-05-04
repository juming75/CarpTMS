//! AI 服务路由模块（纯本地推理，Qwen3.5 Candle 后端）

use actix_web::{web, HttpResponse};
use serde::Deserialize;
use serde_json::json;
use std::sync::{Arc, Mutex};

use super::qwen3_5::pipeline::Qwen3_5Pipeline;
use super::qwen3_5::tasks::*;
use super::resource::{AiModuleConfig, ResourceDetector};
use crate::ml::{MachineLearningService, ModelType};

#[derive(Clone)]
pub struct AiServiceState {
    pub pipeline: Arc<Mutex<Qwen3_5Pipeline>>,
}

mod ml_routes {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Deserialize, Serialize)]
    pub struct PredictRequest {
        pub model_type: String,
        pub input: Vec<f64>,
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub struct TrainRequest {
        pub model_type: String,
        pub data: Vec<Vec<f64>>,
        pub labels: Vec<f64>,
    }

    pub async fn get_all_models(m: web::Data<MachineLearningService>) -> HttpResponse {
        HttpResponse::Ok().json(m.get_all_models().await)
    }

    pub async fn predict(
        m: web::Data<MachineLearningService>,
        r: web::Json<PredictRequest>,
    ) -> HttpResponse {
        let t = match r.model_type.as_str() {
            "VehicleTrajectory" => ModelType::VehicleTrajectory,
            "AlarmTrend" => ModelType::AlarmTrend,
            "FuelConsumption" => ModelType::FuelConsumption,
            "VehicleFault" => ModelType::VehicleFault,
            _ => return HttpResponse::BadRequest().finish(),
        };
        match m.predict(t, &r.input).await {
            Ok(v) => HttpResponse::Ok().json(v),
            Err(e) => HttpResponse::InternalServerError().json(json!({"e": e.to_string()})),
        }
    }

    pub async fn train(
        m: web::Data<MachineLearningService>,
        r: web::Json<TrainRequest>,
    ) -> HttpResponse {
        let t = match r.model_type.as_str() {
            "VehicleTrajectory" => ModelType::VehicleTrajectory,
            "AlarmTrend" => ModelType::AlarmTrend,
            "FuelConsumption" => ModelType::FuelConsumption,
            "VehicleFault" => ModelType::VehicleFault,
            _ => return HttpResponse::BadRequest().finish(),
        };
        match m.train(t, &r.data, &r.labels).await {
            Ok(()) => HttpResponse::Ok().json(json!({"ok": true})),
            Err(e) => HttpResponse::InternalServerError().json(json!({"e": e.to_string()})),
        }
    }
}

pub fn configure_ai_routes(cfg: &mut actix_web::web::ServiceConfig) {
    let api = web::scope("/api/ai")
        .service(
            web::scope("/ml")
                .route("/models", web::get().to(ml_routes::get_all_models))
                .route("/models/{model_type}", web::get().to(model_info))
                .route("/predict", web::post().to(ml_routes::predict))
                .route("/train", web::post().to(ml_routes::train)),
        )
        .service(
            web::scope("/adaptive")
                .route("/status", web::get().to(status))
                .route("/resources", web::get().to(resources))
                .route("/inference", web::post().to(inference))
                .route("/capability", web::get().to(capability)),
        )
        .service(
            web::scope("/qwen")
                .route("/status", web::get().to(qs))
                .route("/inference", web::post().to(qi))
                .route("/report/generate", web::post().to(qr))
                .route("/calibration/analyze", web::post().to(qca))
                .route("/calibration/calculate", web::post().to(qcc))
                .route("/location/monitor", web::post().to(qlm))
                .route("/video/anomaly", web::post().to(qva))
                .route("/field-consistency/check", web::post().to(qfc))
                .route("/code-quality/analyze", web::post().to(qcq)),
        );
    cfg.service(api);
}

async fn model_info(m: web::Data<MachineLearningService>, p: web::Path<String>) -> HttpResponse {
    let t = match p.into_inner().as_str() {
        "VehicleTrajectory" => ModelType::VehicleTrajectory,
        "AlarmTrend" => ModelType::AlarmTrend,
        "FuelConsumption" => ModelType::FuelConsumption,
        "VehicleFault" => ModelType::VehicleFault,
        _ => return HttpResponse::BadRequest().finish(),
    };
    match m.get_model_info(t).await {
        Ok(v) => HttpResponse::Ok().json(v),
        Err(e) => HttpResponse::InternalServerError().json(json!({"e": e.to_string()})),
    }
}

async fn status() -> HttpResponse {
    match crate::di::container::get_adaptive_ai_manager().await {
        Ok(m) => HttpResponse::Ok().json(m.get_status().await),
        Err(e) => HttpResponse::ServiceUnavailable().json(json!({"e": e})),
    }
}

async fn resources() -> HttpResponse {
    let r = ResourceDetector::detect();
    HttpResponse::Ok().json(json!({"r": r}))
}

async fn capability() -> HttpResponse {
    let r = ResourceDetector::detect();
    let c = AiModuleConfig::from(r);
    HttpResponse::Ok().json(json!({
        "level": format!("{:?}", c.capability_level),
        "enabled": c.enabled,
        "model": c.recommended_model
    }))
}

async fn inference(s: web::Data<AiServiceState>, r: web::Json<AdaptiveReq>) -> HttpResponse {
    let pipeline = s.pipeline.clone();
    let prompt = r.prompt.clone();
    let result = web::block(move || {
        let mut p = pipeline.lock().map_err(|_| "AI管道锁被污染".to_string())?;
        p.infer(&prompt).map_err(|e| e.to_string())
    })
    .await;
    match result {
        Ok(Ok(content)) => HttpResponse::Ok().json(json!({"content": content})),
        Ok(Err(e)) => HttpResponse::InternalServerError().json(json!({"e": e})),
        Err(e) => HttpResponse::InternalServerError().json(json!({"e": e.to_string()})),
    }
}

#[derive(Deserialize)]
pub struct AdaptiveReq {
    pub prompt: String,
}

async fn qs(s: web::Data<AiServiceState>) -> HttpResponse {
    let model_loaded = match s.pipeline.lock() {
        Ok(p) => p.model.as_ref().is_some(),
        Err(_) => false,
    };
    HttpResponse::Ok().json(json!({
        "status": "qwen3_5",
        "model_loaded": model_loaded
    }))
}

async fn qi(s: web::Data<AiServiceState>, r: web::Json<serde_json::Value>) -> HttpResponse {
    let prompt = r
        .get("prompt")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let pipeline = s.pipeline.clone();
    let result = web::block(move || {
        let mut p = pipeline.lock().map_err(|_| "AI管道锁被污染".to_string())?;
        p.infer(&prompt).map_err(|e| e.to_string())
    })
    .await;
    match result {
        Ok(Ok(content)) => HttpResponse::Ok().json(json!({"content": content})),
        Ok(Err(e)) => HttpResponse::InternalServerError().json(json!({"e": e})),
        Err(e) => HttpResponse::InternalServerError().json(json!({"e": e.to_string()})),
    }
}

async fn qr(s: web::Data<AiServiceState>, r: web::Json<ReportGenerationRequest>) -> HttpResponse {
    let pipeline = s.pipeline.clone();
    let request = r.into_inner();
    let result = web::block(move || {
        let mut p = pipeline.lock().map_err(|_| "AI管道锁被污染".to_string())?;
        generate_report(&mut p, &request)
    })
    .await;
    match result {
        Ok(Ok(v)) => HttpResponse::Ok().json(v),
        Ok(Err(e)) => HttpResponse::InternalServerError().json(json!({"e": e})),
        Err(e) => HttpResponse::InternalServerError().json(json!({"e": e.to_string()})),
    }
}

async fn qca(
    s: web::Data<AiServiceState>,
    r: web::Json<CalibrationAnalysisRequest>,
) -> HttpResponse {
    let pipeline = s.pipeline.clone();
    let request = r.into_inner();
    let result = web::block(move || {
        let mut p = pipeline.lock().map_err(|_| "AI管道锁被污染".to_string())?;
        analyze_calibration(&mut p, &request)
    })
    .await;
    match result {
        Ok(Ok(v)) => HttpResponse::Ok().json(v),
        Ok(Err(e)) => HttpResponse::InternalServerError().json(json!({"e": e})),
        Err(e) => HttpResponse::InternalServerError().json(json!({"e": e.to_string()})),
    }
}

async fn qcc(
    s: web::Data<AiServiceState>,
    r: web::Json<CalibrationCalculationRequest>,
) -> HttpResponse {
    let pipeline = s.pipeline.clone();
    let request = r.into_inner();
    let result = web::block(move || {
        let mut p = pipeline.lock().map_err(|_| "AI管道锁被污染".to_string())?;
        calculate_calibration(&mut p, &request)
    })
    .await;
    match result {
        Ok(Ok(v)) => HttpResponse::Ok().json(v),
        Ok(Err(e)) => HttpResponse::InternalServerError().json(json!({"e": e})),
        Err(e) => HttpResponse::InternalServerError().json(json!({"e": e.to_string()})),
    }
}

async fn qlm(
    s: web::Data<AiServiceState>,
    r: web::Json<LocationMonitoringRequest>,
) -> HttpResponse {
    let pipeline = s.pipeline.clone();
    let request = r.into_inner();
    let result = web::block(move || {
        let mut p = pipeline.lock().map_err(|_| "AI管道锁被污染".to_string())?;
        monitor_location(&mut p, &request)
    })
    .await;
    match result {
        Ok(Ok(v)) => HttpResponse::Ok().json(v),
        Ok(Err(e)) => HttpResponse::InternalServerError().json(json!({"e": e})),
        Err(e) => HttpResponse::InternalServerError().json(json!({"e": e.to_string()})),
    }
}

async fn qva(s: web::Data<AiServiceState>, r: web::Json<VideoAnomalyRequest>) -> HttpResponse {
    let pipeline = s.pipeline.clone();
    let request = r.into_inner();
    let result = web::block(move || {
        let mut p = pipeline.lock().map_err(|_| "AI管道锁被污染".to_string())?;
        detect_video_anomaly(&mut p, &request)
    })
    .await;
    match result {
        Ok(Ok(v)) => HttpResponse::Ok().json(v),
        Ok(Err(e)) => HttpResponse::InternalServerError().json(json!({"e": e})),
        Err(e) => HttpResponse::InternalServerError().json(json!({"e": e.to_string()})),
    }
}

async fn qfc(s: web::Data<AiServiceState>, r: web::Json<FieldConsistencyRequest>) -> HttpResponse {
    let pipeline = s.pipeline.clone();
    let request = r.into_inner();
    let result = web::block(move || {
        let mut p = pipeline.lock().map_err(|_| "AI管道锁被污染".to_string())?;
        check_field_consistency(&mut p, &request)
    })
    .await;
    match result {
        Ok(Ok(v)) => HttpResponse::Ok().json(v),
        Ok(Err(e)) => HttpResponse::InternalServerError().json(json!({"e": e})),
        Err(e) => HttpResponse::InternalServerError().json(json!({"e": e.to_string()})),
    }
}

async fn qcq(s: web::Data<AiServiceState>, r: web::Json<CodeQualityRequest>) -> HttpResponse {
    let pipeline = s.pipeline.clone();
    let request = r.into_inner();
    let result = web::block(move || {
        let mut p = pipeline.lock().map_err(|_| "AI管道锁被污染".to_string())?;
        analyze_code_quality(&mut p, &request)
    })
    .await;
    match result {
        Ok(Ok(v)) => HttpResponse::Ok().json(v),
        Ok(Err(e)) => HttpResponse::InternalServerError().json(json!({"e": e})),
        Err(e) => HttpResponse::InternalServerError().json(json!({"e": e.to_string()})),
    }
}
