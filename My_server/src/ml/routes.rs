//! 机器学习路由
//! 提供机器学习服务的API接口

use actix_web::{web, HttpResponse, Scope};
use serde::{Deserialize, Serialize};
use crate::ml::{MachineLearningService, ModelType, MLError};

/// 预测请求
#[derive(Debug, Deserialize, Serialize)]
pub struct PredictRequest {
    /// 模型类型
    pub model_type: String,
    /// 输入数据
    pub input: Vec<f64>,
}

/// 训练请求
#[derive(Debug, Deserialize, Serialize)]
pub struct TrainRequest {
    /// 模型类型
    pub model_type: String,
    /// 训练数据
    pub data: Vec<Vec<f64>>,
    /// 标签
    pub labels: Vec<f64>,
}

/// 配置机器学习路由
pub fn configure_ml_routes() -> Scope {
    web::scope("/api/ml")
        .route("/models", web::get().to(get_all_models))
        .route("/models/{model_type}", web::get().to(get_model_info))
        .route("/predict", web::post().to(predict))
        .route("/train", web::post().to(train))
}

/// 获取所有模型信息
async fn get_all_models(
    ml_service: web::Data<MachineLearningService>,
) -> HttpResponse {
    let models = ml_service.get_all_models().await;
    HttpResponse::Ok().json(models)
}

/// 获取模型信息
async fn get_model_info(
    ml_service: web::Data<MachineLearningService>,
    model_type: web::Path<String>,
) -> HttpResponse {
    let model_type = match model_type.into_inner().as_str() {
        "VehicleTrajectory" => ModelType::VehicleTrajectory,
        "AlarmTrend" => ModelType::AlarmTrend,
        "FuelConsumption" => ModelType::FuelConsumption,
        "VehicleFault" => ModelType::VehicleFault,
        _ => return HttpResponse::BadRequest().json({"error": "Invalid model type"}),
    };
    
    match ml_service.get_model_info(model_type).await {
        Ok(info) => HttpResponse::Ok().json(info),
        Err(error) => HttpResponse::InternalServerError().json({"error": error.to_string()}),
    }
}

/// 进行预测
async fn predict(
    ml_service: web::Data<MachineLearningService>,
    req: web::Json<PredictRequest>,
) -> HttpResponse {
    let model_type = match req.model_type.as_str() {
        "VehicleTrajectory" => ModelType::VehicleTrajectory,
        "AlarmTrend" => ModelType::AlarmTrend,
        "FuelConsumption" => ModelType::FuelConsumption,
        "VehicleFault" => ModelType::VehicleFault,
        _ => return HttpResponse::BadRequest().json({"error": "Invalid model type"}),
    };
    
    match ml_service.predict(model_type, &req.input).await {
        Ok(result) => HttpResponse::Ok().json(result),
        Err(error) => HttpResponse::InternalServerError().json({"error": error.to_string()}),
    }
}

/// 训练模型
async fn train(
    ml_service: web::Data<MachineLearningService>,
    req: web::Json<TrainRequest>,
) -> HttpResponse {
    let model_type = match req.model_type.as_str() {
        "VehicleTrajectory" => ModelType::VehicleTrajectory,
        "AlarmTrend" => ModelType::AlarmTrend,
        "FuelConsumption" => ModelType::FuelConsumption,
        "VehicleFault" => ModelType::VehicleFault,
        _ => return HttpResponse::BadRequest().json({"error": "Invalid model type"}),
    };
    
    match ml_service.train(model_type, &req.data, &req.labels).await {
        Ok(_) => HttpResponse::Ok().json({"message": "Model trained successfully"}),
        Err(error) => HttpResponse::InternalServerError().json({"error": error.to_string()}),
    }
}
