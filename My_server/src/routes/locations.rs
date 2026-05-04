use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;

use crate::application::services::location_service::{LocationService, LocationServiceImpl};
use crate::domain::entities::location::{
    FenceCreate as DomainFenceCreate, FenceQuery as DomainFenceQuery,
    FenceUpdate as DomainFenceUpdate, LocationCreate as DomainLocationCreate,
    LocationUpdate as DomainLocationUpdate, PlaceCreate as DomainPlaceCreate,
    PlaceUpdate as DomainPlaceUpdate, RouteCreate as DomainRouteCreate,
    RouteUpdate as DomainRouteUpdate,
};
use crate::errors::{
    created_response_with_message, empty_success_response, success_response_with_message, AppError,
    AppResult,
};
use crate::schemas::PagedResponse;

// ==================== 电子围栏相关 ====================

#[derive(Debug, Deserialize, ToSchema)]
pub struct FenceCreate {
    pub fence_name: String,
    pub fence_type: String, // circle, polygon, rectangle
    pub center_latitude: Option<f64>,
    pub center_longitude: Option<f64>,
    pub radius: Option<f64>,
    pub polygon_points: Option<serde_json::Value>,
    pub rectangle_bounds: Option<serde_json::Value>,
    pub status: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct FenceUpdate {
    pub fence_name: Option<String>,
    pub fence_type: Option<String>,
    pub center_latitude: Option<f64>,
    pub center_longitude: Option<f64>,
    pub radius: Option<f64>,
    pub polygon_points: Option<serde_json::Value>,
    pub rectangle_bounds: Option<serde_json::Value>,
    pub status: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct FenceResponse {
    pub fence_id: i32,
    pub fence_name: String,
    pub fence_type: String,
    pub center_latitude: Option<f64>,
    pub center_longitude: Option<f64>,
    pub radius: Option<f64>,
    pub polygon_points: Option<serde_json::Value>,
    pub rectangle_bounds: Option<serde_json::Value>,
    pub status: String,
    pub description: Option<String>,
    pub create_time: String,
    pub update_time: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct FenceQuery {
    pub page: Option<i32>,
    pub page_size: Option<i32>,
    pub status: Option<String>,
    pub fence_type: Option<String>,
}

pub async fn get_fences(
    location_service: web::Data<Arc<LocationServiceImpl>>,
    query: web::Query<FenceQuery>,
) -> AppResult<HttpResponse> {
    // 转换为领域查询对象
    let domain_query = DomainFenceQuery {
        page: query.page,
        page_size: query.page_size,
        status: query.status.clone(),
        fence_type: query.fence_type.clone(),
    };

    // 调用服务获取围栏列表
    let (fences, total) = location_service.get_fences(domain_query).await?;

    // 转换为响应格式
    let fence_responses: Vec<FenceResponse> = fences
        .into_iter()
        .map(|fence| FenceResponse {
            fence_id: fence.fence_id,
            fence_name: fence.fence_name,
            fence_type: fence.fence_type,
            center_latitude: fence.center_latitude,
            center_longitude: fence.center_longitude,
            radius: fence.radius,
            polygon_points: fence.polygon_points,
            rectangle_bounds: fence.rectangle_bounds,
            status: fence.status,
            description: fence.description,
            create_time: fence.created_at.to_string(),
            update_time: fence.updated_at.map(|t| t.to_string()),
        })
        .collect();

    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20);
    let pages = if total % page_size as i64 == 0 {
        total / page_size as i64
    } else {
        total / page_size as i64 + 1
    };

    let paged_response = PagedResponse {
        list: fence_responses,
        total,
        page,
        page_size,
        pages: pages as i32,
    };

    Ok(success_response_with_message(
        "Fences fetched successfully",
        paged_response,
    ))
}

pub async fn create_fence(
    location_service: web::Data<Arc<LocationServiceImpl>>,
    fence: web::Json<FenceCreate>,
) -> AppResult<HttpResponse> {
    // 转换为领域创建对象
    let domain_fence = DomainFenceCreate {
        fence_name: fence.fence_name.clone(),
        fence_type: fence.fence_type.clone(),
        center_latitude: fence.center_latitude,
        center_longitude: fence.center_longitude,
        radius: fence.radius,
        polygon_points: fence.polygon_points.clone(),
        rectangle_bounds: fence.rectangle_bounds.clone(),
        status: fence.status.clone(),
        description: fence.description.clone(),
    };

    // 调用服务创建设围栏
    let created_fence = location_service.create_fence(domain_fence).await?;

    // 转换为响应格式
    let response = FenceResponse {
        fence_id: created_fence.fence_id,
        fence_name: created_fence.fence_name,
        fence_type: created_fence.fence_type,
        center_latitude: created_fence.center_latitude,
        center_longitude: created_fence.center_longitude,
        radius: created_fence.radius,
        polygon_points: created_fence.polygon_points,
        rectangle_bounds: created_fence.rectangle_bounds,
        status: created_fence.status,
        description: created_fence.description,
        create_time: created_fence.created_at.to_string(),
        update_time: created_fence.updated_at.map(|t| t.to_string()),
    };

    Ok(created_response_with_message(
        "Fence created successfully",
        response,
    ))
}

pub async fn update_fence(
    location_service: web::Data<Arc<LocationServiceImpl>>,
    path: web::Path<(i32,)>,
    fence: web::Json<FenceUpdate>,
) -> AppResult<HttpResponse> {
    let fence_id = path.0;

    // 转换为领域更新对象
    let domain_fence = DomainFenceUpdate {
        fence_name: fence.fence_name.clone(),
        fence_type: fence.fence_type.clone(),
        center_latitude: fence.center_latitude,
        center_longitude: fence.center_longitude,
        radius: fence.radius,
        polygon_points: fence.polygon_points.clone(),
        rectangle_bounds: fence.rectangle_bounds.clone(),
        status: fence.status.clone(),
        description: fence.description.clone(),
    };

    // 调用服务更新围栏
    let updated_fence = location_service
        .update_fence(fence_id, domain_fence)
        .await?;

    match updated_fence {
        Some(fence) => {
            let response = FenceResponse {
                fence_id: fence.fence_id,
                fence_name: fence.fence_name,
                fence_type: fence.fence_type,
                center_latitude: fence.center_latitude,
                center_longitude: fence.center_longitude,
                radius: fence.radius,
                polygon_points: fence.polygon_points,
                rectangle_bounds: fence.rectangle_bounds,
                status: fence.status,
                description: fence.description,
                create_time: fence.created_at.to_string(),
                update_time: fence.updated_at.map(|t| t.to_string()),
            };

            Ok(success_response_with_message(
                "Fence updated successfully",
                response,
            ))
        }
        None => Err(AppError::not_found_error("Fence not found".to_string())),
    }
}

pub async fn delete_fence(
    location_service: web::Data<Arc<LocationServiceImpl>>,
    path: web::Path<(i32,)>,
) -> AppResult<HttpResponse> {
    let fence_id = path.0;

    // 调用服务删除围栏
    let deleted = location_service.delete_fence(fence_id).await?;

    if deleted {
        Ok(empty_success_response())
    } else {
        Err(AppError::not_found_error("Fence not found".to_string()))
    }
}

// ==================== 位置点相关 ====================

#[derive(Debug, Deserialize, ToSchema)]
pub struct LocationCreate {
    pub vehicle_id: i32,
    pub latitude: f64,
    pub longitude: f64,
    pub speed: Option<f64>,
    pub direction: Option<f64>,
    pub status: Option<String>,
    pub altitude: Option<f64>,
    pub accuracy: Option<f64>,
    pub timestamp: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct LocationUpdate {
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub speed: Option<f64>,
    pub direction: Option<f64>,
    pub status: Option<String>,
    pub altitude: Option<f64>,
    pub accuracy: Option<f64>,
    pub timestamp: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct LocationResponse {
    pub position_id: i32,
    pub place_name: String,
    pub latitude: f64,
    pub longitude: f64,
    pub address: Option<String>,
    pub description: Option<String>,
    pub create_time: String,
    pub update_time: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct LocationQuery {
    pub page: Option<i32>,
    pub page_size: Option<i32>,
}

pub async fn get_positions(
    location_service: web::Data<Arc<LocationServiceImpl>>,
    query: web::Query<LocationQuery>,
) -> AppResult<HttpResponse> {
    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20);

    // 调用服务获取位置列表
    let (locations, total) = location_service.get_locations(page, page_size).await?;

    // 转换为响应格式
    let location_responses: Vec<LocationResponse> = locations
        .into_iter()
        .map(|location| LocationResponse {
            position_id: location.position_id,
            place_name: location.place_name,
            latitude: location.latitude,
            longitude: location.longitude,
            address: location.address,
            description: location.description,
            create_time: location.created_at.to_string(),
            update_time: location.updated_at.map(|t| t.to_string()),
        })
        .collect();

    let pages = if total % page_size as i64 == 0 {
        total / page_size as i64
    } else {
        total / page_size as i64 + 1
    };

    let paged_response = PagedResponse {
        list: location_responses,
        total,
        page,
        page_size,
        pages: pages as i32,
    };

    Ok(success_response_with_message(
        "Locations fetched successfully",
        paged_response,
    ))
}

pub async fn create_position(
    location_service: web::Data<Arc<LocationServiceImpl>>,
    location: web::Json<LocationCreate>,
) -> AppResult<HttpResponse> {
    // 转换为领域创建对象
    let domain_location = DomainLocationCreate {
        location_name: "Unknown Location".to_string(), // 暂时使用默认值
        latitude: location.latitude,
        longitude: location.longitude,
        address: location.description.clone(), // 使用description作为address
        description: location.description.clone(),
    };

    // 调用服务创建位置
    let created_location = location_service.create_location(domain_location).await?;

    // 转换为响应格式
    let response = LocationResponse {
        position_id: created_location.position_id,
        place_name: created_location.place_name,
        latitude: created_location.latitude,
        longitude: created_location.longitude,
        address: created_location.address,
        description: created_location.description,
        create_time: created_location.created_at.to_string(),
        update_time: created_location.updated_at.map(|t| t.to_string()),
    };

    Ok(created_response_with_message(
        "Location created successfully",
        response,
    ))
}

pub async fn update_position(
    location_service: web::Data<Arc<LocationServiceImpl>>,
    path: web::Path<(i32,)>,
    location: web::Json<LocationUpdate>,
) -> AppResult<HttpResponse> {
    let location_id = path.0;

    // 转换为领域更新对象
    let domain_location = DomainLocationUpdate {
        location_name: None, // 暂时使用None
        latitude: location.latitude,
        longitude: location.longitude,
        address: location.description.clone(), // 使用description作为address
        description: location.description.clone(),
    };

    // 调用服务更新位置
    let updated_location = location_service
        .update_location(location_id, domain_location)
        .await?;

    match updated_location {
        Some(location) => {
            let response = LocationResponse {
                position_id: location.position_id,
                place_name: location.place_name,
                latitude: location.latitude,
                longitude: location.longitude,
                address: location.address,
                description: location.description,
                create_time: location.created_at.to_string(),
                update_time: location.updated_at.map(|t| t.to_string()),
            };

            Ok(success_response_with_message(
                "Location updated successfully",
                response,
            ))
        }
        None => Err(AppError::not_found_error("Location not found".to_string())),
    }
}

pub async fn delete_position(
    location_service: web::Data<Arc<LocationServiceImpl>>,
    path: web::Path<(i32,)>,
) -> AppResult<HttpResponse> {
    let location_id = path.0;

    // 调用服务删除位置
    let deleted = location_service.delete_location(location_id).await?;

    if deleted {
        Ok(empty_success_response())
    } else {
        Err(AppError::not_found_error("Location not found".to_string()))
    }
}

// ==================== 地点相关 ====================

#[derive(Debug, Deserialize, ToSchema)]
pub struct PlaceCreate {
    pub place_name: String,
    pub address: String,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub description: Option<String>,
    pub contact_person: Option<String>,
    pub contact_phone: Option<String>,
    pub contact_email: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct PlaceUpdate {
    pub place_name: Option<String>,
    pub address: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub description: Option<String>,
    pub contact_person: Option<String>,
    pub contact_phone: Option<String>,
    pub contact_email: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PlaceResponse {
    pub place_id: i32,
    pub place_name: String,
    pub address: String,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub description: Option<String>,
    pub contact_person: Option<String>,
    pub contact_phone: Option<String>,
    pub contact_email: Option<String>,
    pub create_time: String,
    pub update_time: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct PlaceQuery {
    pub page: Option<i32>,
    pub page_size: Option<i32>,
}

pub async fn get_places(
    location_service: web::Data<Arc<LocationServiceImpl>>,
    query: web::Query<PlaceQuery>,
) -> AppResult<HttpResponse> {
    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20);

    // 调用服务获取地点列表
    let (places, total) = location_service.get_places(page, page_size).await?;

    // 转换为响应格式
    let place_responses: Vec<PlaceResponse> = places
        .into_iter()
        .map(|place| PlaceResponse {
            place_id: place.place_id,
            place_name: place.place_name,
            address: place.address,
            latitude: place.latitude,
            longitude: place.longitude,
            description: place.description,
            contact_person: place.contact_person,
            contact_phone: place.contact_phone,
            contact_email: place.contact_email,
            create_time: place.created_at.to_string(),
            update_time: place.updated_at.map(|t| t.to_string()),
        })
        .collect();

    let pages = if total % page_size as i64 == 0 {
        total / page_size as i64
    } else {
        total / page_size as i64 + 1
    };

    let paged_response = PagedResponse {
        list: place_responses,
        total,
        page,
        page_size,
        pages: pages as i32,
    };

    Ok(success_response_with_message(
        "Places fetched successfully",
        paged_response,
    ))
}

pub async fn create_place(
    location_service: web::Data<Arc<LocationServiceImpl>>,
    place: web::Json<PlaceCreate>,
) -> AppResult<HttpResponse> {
    // 转换为领域创建对象
    let domain_place = DomainPlaceCreate {
        place_name: place.place_name.clone(),
        address: place.address.clone(),
        latitude: place.latitude,
        longitude: place.longitude,
        description: place.description.clone(),
        contact_person: place.contact_person.clone(),
        contact_phone: place.contact_phone.clone(),
        contact_email: place.contact_email.clone(),
    };

    // 调用服务创建地点
    let created_place = location_service.create_place(domain_place).await?;

    // 转换为响应格式
    let response = PlaceResponse {
        place_id: created_place.place_id,
        place_name: created_place.place_name,
        address: created_place.address,
        latitude: created_place.latitude,
        longitude: created_place.longitude,
        description: created_place.description,
        contact_person: created_place.contact_person,
        contact_phone: created_place.contact_phone,
        contact_email: created_place.contact_email,
        create_time: created_place.created_at.to_string(),
        update_time: created_place.updated_at.map(|t| t.to_string()),
    };

    Ok(created_response_with_message(
        "Place created successfully",
        response,
    ))
}

pub async fn update_place(
    location_service: web::Data<Arc<LocationServiceImpl>>,
    path: web::Path<(i32,)>,
    place: web::Json<PlaceUpdate>,
) -> AppResult<HttpResponse> {
    let place_id = path.0;

    // 转换为领域更新对象
    let domain_place = DomainPlaceUpdate {
        place_name: place.place_name.clone(),
        address: place.address.clone(),
        latitude: place.latitude,
        longitude: place.longitude,
        description: place.description.clone(),
        contact_person: place.contact_person.clone(),
        contact_phone: place.contact_phone.clone(),
        contact_email: place.contact_email.clone(),
    };

    // 调用服务更新地点
    let updated_place = location_service
        .update_place(place_id, domain_place)
        .await?;

    match updated_place {
        Some(place) => {
            let response = PlaceResponse {
                place_id: place.place_id,
                place_name: place.place_name,
                address: place.address,
                latitude: place.latitude,
                longitude: place.longitude,
                description: place.description,
                contact_person: place.contact_person,
                contact_phone: place.contact_phone,
                contact_email: place.contact_email,
                create_time: place.created_at.to_string(),
                update_time: place.updated_at.map(|t| t.to_string()),
            };

            Ok(success_response_with_message(
                "Place updated successfully",
                response,
            ))
        }
        None => Err(AppError::not_found_error("Place not found".to_string())),
    }
}

pub async fn delete_place(
    location_service: web::Data<Arc<LocationServiceImpl>>,
    path: web::Path<(i32,)>,
) -> AppResult<HttpResponse> {
    let place_id = path.0;

    // 调用服务删除地点
    let deleted = location_service.delete_place(place_id).await?;

    if deleted {
        Ok(empty_success_response())
    } else {
        Err(AppError::not_found_error("Place not found".to_string()))
    }
}

// ==================== 路线相关 ====================

#[derive(Debug, Deserialize, ToSchema)]
pub struct RouteCreate {
    pub route_name: String,
    pub start_point: String,
    pub start_latitude: Option<f64>,
    pub start_longitude: Option<f64>,
    pub end_point: String,
    pub end_latitude: Option<f64>,
    pub end_longitude: Option<f64>,
    pub waypoints: Option<serde_json::Value>,
    pub distance: Option<f64>,
    pub estimated_duration: Option<i32>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct RouteUpdate {
    pub route_name: Option<String>,
    pub start_point: Option<String>,
    pub start_latitude: Option<f64>,
    pub start_longitude: Option<f64>,
    pub end_point: Option<String>,
    pub end_latitude: Option<f64>,
    pub end_longitude: Option<f64>,
    pub waypoints: Option<serde_json::Value>,
    pub distance: Option<f64>,
    pub estimated_duration: Option<i32>,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct RouteResponse {
    pub route_id: i32,
    pub route_name: String,
    pub start_point: String,
    pub start_latitude: Option<f64>,
    pub start_longitude: Option<f64>,
    pub end_point: String,
    pub end_latitude: Option<f64>,
    pub end_longitude: Option<f64>,
    pub waypoints: Option<serde_json::Value>,
    pub distance: Option<f64>,
    pub estimated_duration: Option<i32>,
    pub description: Option<String>,
    pub create_time: String,
    pub update_time: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct RouteQuery {
    pub page: Option<i32>,
    pub page_size: Option<i32>,
    pub route_type: Option<String>,
    pub start_place_id: Option<i32>,
    pub end_place_id: Option<i32>,
}

pub async fn get_routes(
    location_service: web::Data<Arc<LocationServiceImpl>>,
    query: web::Query<RouteQuery>,
) -> AppResult<HttpResponse> {
    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20);

    // 调用服务获取路线列表
    let (routes, total) = location_service.get_routes(page, page_size).await?;

    // 转换为响应格式
    let route_responses: Vec<RouteResponse> = routes
        .into_iter()
        .map(|route| RouteResponse {
            route_id: route.route_id,
            route_name: route.route_name,
            start_point: route.start_point,
            start_latitude: route.start_latitude,
            start_longitude: route.start_longitude,
            end_point: route.end_point,
            end_latitude: route.end_latitude,
            end_longitude: route.end_longitude,
            waypoints: route.waypoints,
            distance: route.distance,
            estimated_duration: route.estimated_duration,
            description: route.description,
            create_time: route.created_at.to_string(),
            update_time: route.updated_at.map(|t| t.to_string()),
        })
        .collect();

    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20);
    let pages = if total % page_size as i64 == 0 {
        total / page_size as i64
    } else {
        total / page_size as i64 + 1
    };

    let paged_response = PagedResponse {
        list: route_responses,
        total,
        page,
        page_size,
        pages: pages as i32,
    };

    Ok(success_response_with_message(
        "Routes fetched successfully",
        paged_response,
    ))
}

pub async fn create_route(
    location_service: web::Data<Arc<LocationServiceImpl>>,
    route: web::Json<RouteCreate>,
) -> AppResult<HttpResponse> {
    // 转换为领域创建对象
    let domain_route = DomainRouteCreate {
        route_name: route.route_name.clone(),
        start_point: route.start_point.clone(),
        start_latitude: route.start_latitude,
        start_longitude: route.start_longitude,
        end_point: route.end_point.clone(),
        end_latitude: route.end_latitude,
        end_longitude: route.end_longitude,
        waypoints: route.waypoints.clone(),
        distance: route.distance,
        estimated_duration: route.estimated_duration,
        description: route.description.clone(),
    };

    // 调用服务创建路线
    let created_route = location_service.create_route(domain_route).await?;

    // 转换为响应格式
    let response = RouteResponse {
        route_id: created_route.route_id,
        route_name: created_route.route_name,
        start_point: created_route.start_point,
        start_latitude: created_route.start_latitude,
        start_longitude: created_route.start_longitude,
        end_point: created_route.end_point,
        end_latitude: created_route.end_latitude,
        end_longitude: created_route.end_longitude,
        waypoints: created_route.waypoints,
        distance: created_route.distance,
        estimated_duration: created_route.estimated_duration,
        description: created_route.description,
        create_time: created_route.created_at.to_string(),
        update_time: created_route.updated_at.map(|t| t.to_string()),
    };

    Ok(created_response_with_message(
        "Route created successfully",
        response,
    ))
}

pub async fn update_route(
    location_service: web::Data<Arc<LocationServiceImpl>>,
    path: web::Path<(i32,)>,
    route: web::Json<RouteUpdate>,
) -> AppResult<HttpResponse> {
    let route_id = path.0;

    // 转换为领域更新对象
    let domain_route = DomainRouteUpdate {
        route_name: route.route_name.clone(),
        start_point: route.start_point.clone(),
        start_latitude: route.start_latitude,
        start_longitude: route.start_longitude,
        end_point: route.end_point.clone(),
        end_latitude: route.end_latitude,
        end_longitude: route.end_longitude,
        waypoints: route.waypoints.clone(),
        distance: route.distance,
        estimated_duration: route.estimated_duration,
        description: route.description.clone(),
    };

    // 调用服务更新路线
    let updated_route = location_service
        .update_route(route_id, domain_route)
        .await?;

    match updated_route {
        Some(route) => {
            let response = RouteResponse {
                route_id: route.route_id,
                route_name: route.route_name,
                start_point: route.start_point,
                start_latitude: route.start_latitude,
                start_longitude: route.start_longitude,
                end_point: route.end_point,
                end_latitude: route.end_latitude,
                end_longitude: route.end_longitude,
                waypoints: route.waypoints,
                distance: route.distance,
                estimated_duration: route.estimated_duration,
                description: route.description,
                create_time: route.created_at.to_string(),
                update_time: route.updated_at.map(|t| t.to_string()),
            };

            Ok(success_response_with_message(
                "Route updated successfully",
                response,
            ))
        }
        None => Err(AppError::not_found_error("Route not found".to_string())),
    }
}

pub async fn delete_route(
    location_service: web::Data<Arc<LocationServiceImpl>>,
    path: web::Path<(i32,)>,
) -> AppResult<HttpResponse> {
    let route_id = path.0;

    // 调用服务删除路线
    let deleted = location_service.delete_route(route_id).await?;

    if deleted {
        Ok(empty_success_response())
    } else {
        Err(AppError::not_found_error("Route not found".to_string()))
    }
}

// 配置位置相关路由
pub fn configure_location_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/location")
            // 电子围栏路由
            .route("/fences", web::get().to(get_fences))
            .route("/fences", web::post().to(create_fence))
            .route("/fences/{id}", web::put().to(update_fence))
            .route("/fences/{id}", web::delete().to(delete_fence))
            // 位置点路由
            .route("/positions", web::get().to(get_positions))
            .route("/positions", web::post().to(create_position))
            .route("/positions/{id}", web::put().to(update_position))
            .route("/positions/{id}", web::delete().to(delete_position))
            // 地点路由
            .route("/places", web::get().to(get_places))
            .route("/places", web::post().to(create_place))
            .route("/places/{id}", web::put().to(update_place))
            .route("/places/{id}", web::delete().to(delete_place))
            // 路线路由
            .route("/routes", web::get().to(get_routes))
            .route("/routes", web::post().to(create_route))
            .route("/routes/{id}", web::put().to(update_route))
            .route("/routes/{id}", web::delete().to(delete_route)),
    );
}
