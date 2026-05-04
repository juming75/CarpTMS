//! / 清洁架构车辆路由示例

use actix_web::{web, HttpResponse};
use std::sync::Arc;

use crate::domain::entities::vehicle::{VehicleCreate, VehicleQuery};
use crate::domain::use_cases::vehicle::VehicleUseCases;
use crate::errors::{success_response, AppError, AppResult};
use crate::infrastructure::repositories::vehicle_repository::PgVehicleRepository;

// 创建车辆路由
pub async fn create_clean_vehicle(
    pool: web::Data<Arc<sqlx::PgPool>>,
    vehicle: web::Json<VehicleCreate>,
) -> AppResult<HttpResponse> {
    // 创建仓库实例
    let vehicle_repository = Arc::new(PgVehicleRepository::new(pool.into_inner().as_ref().clone()));

    // 创建用例实例
    let vehicle_use_cases = VehicleUseCases::new(vehicle_repository);

    // 调用用例创建车辆
    let created_vehicle = vehicle_use_cases
        .create_vehicle(vehicle.into_inner())
        .await
        .map_err(|e| AppError::business_error(&e.to_string(), None))?;

    // 返回成功响应
    Ok(success_response(Some(created_vehicle)))
}

// 获取车辆列表路由
pub async fn get_clean_vehicles(
    pool: web::Data<Arc<sqlx::PgPool>>,
    query: web::Query<VehicleQuery>,
) -> AppResult<HttpResponse> {
    // 创建仓库实例
    let vehicle_repository = Arc::new(PgVehicleRepository::new(pool.into_inner().as_ref().clone()));

    // 创建用例实例
    let vehicle_use_cases = VehicleUseCases::new(vehicle_repository);

    // 调用用例获取车辆列表
    let (vehicles, total_count) = vehicle_use_cases
        .get_vehicles(query.into_inner())
        .await
        .map_err(|e| AppError::business_error(&e.to_string(), None))?;

    // 返回成功响应
    Ok(success_response(Some((vehicles, total_count))))
}

// 导出路由配置
pub fn configure_clean_vehicle_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/clean")
            .route("/vehicles", web::post().to(create_clean_vehicle))
            .route("/vehicles", web::get().to(get_clean_vehicles)),
    );
}
