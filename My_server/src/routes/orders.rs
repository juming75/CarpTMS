//! 订单路由 - 使用应用服务层
//!
//! 该模块将 HTTP 请求委托给 OrderApplicationService 处理，
//! 遵循 DDD 架构模式。

use actix_web::{web, HttpResponse};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::sync::Arc;
use utoipa::ToSchema;

use crate::application::dto::OrderDto;
use crate::application::services::OrderApplicationService;
use crate::application::PagedResult;
use crate::domain::entities::order::{OrderCreate, OrderItemCreate, OrderQuery, OrderUpdate};
use crate::errors::{success_response_with_message, AppError, AppResult};
use crate::schemas::{
    LogisticsTrackCreate, LogisticsTrackCreateBatch, LogisticsTrackResponse, LogisticsTrackUpdate, OrderCreate as SchemaOrderCreate,
    OrderItemCreate as SchemaOrderItemCreate, OrderItemUpdate, OrderResponse, OrderUpdate as SchemaOrderUpdate,
    PagedResponse, VehicleTracksQuery,
};

// ============== 类型转换函数 ==============

/// 将 OrderDto 转换为 OrderResponse
fn order_dto_to_response(dto: OrderDto, vehicle_name: String, license_plate: String, driver_name: Option<String>) -> OrderResponse {
    OrderResponse {
        order_id: dto.order_id,
        order_no: dto.order_no,
        vehicle_id: dto.vehicle_id,
        vehicle_name,
        license_plate,
        driver_id: dto.driver_id,
        driver_name,
        customer_name: dto.customer_name,
        customer_phone: dto.customer_phone,
        origin: dto.origin,
        destination: dto.destination,
        cargo_type: dto.cargo_type,
        cargo_weight: dto.cargo_weight,
        cargo_volume: Some(dto.cargo_volume),
        cargo_count: Some(dto.cargo_count),
        order_amount: dto.order_amount,
        order_status: dto.order_status,
        order_status_text: match dto.order_status {
            1 => "待分配".to_string(),
            2 => "运输中".to_string(),
            3 => "已完成".to_string(),
            4 => "已取消".to_string(),
            _ => "未知状态".to_string(),
        },
        departure_time: dto.departure_time.map(|t| DateTime::from_naive_utc_and_offset(t, Utc)),
        arrival_time: dto.arrival_time.map(|t| DateTime::from_naive_utc_and_offset(t, Utc)),
        remark: dto.remark,
        create_user_id: dto.create_user_id,
        create_time: DateTime::from_naive_utc_and_offset(dto.create_time, Utc),
        update_time: dto.update_time.map(|t| DateTime::from_naive_utc_and_offset(t, Utc)),
    }
}

/// 将 SchemaOrderCreate 转换为领域层的 OrderCreate
fn schema_create_to_domain(create: SchemaOrderCreate, order_no: String) -> OrderCreate {
    OrderCreate {
        order_no,
        vehicle_id: create.vehicle_id,
        driver_id: create.driver_id,
        customer_name: create.customer_name,
        customer_phone: create.customer_phone,
        origin: create.origin,
        destination: create.destination,
        cargo_type: create.cargo_type,
        cargo_weight: create.cargo_weight,
        cargo_volume: create.cargo_volume.unwrap_or(0.0),
        cargo_count: create.cargo_count.unwrap_or(0),
        order_amount: create.order_amount,
        order_status: 1, // 初始状态:待分配
        departure_time: None,
        arrival_time: None,
        remark: create.remark,
        create_user_id: create.create_user_id,
    }
}

/// 将 SchemaOrderUpdate 转换为领域层的 OrderUpdate
fn schema_update_to_domain(update: SchemaOrderUpdate) -> OrderUpdate {
    OrderUpdate {
        order_no: None,
        vehicle_id: update.vehicle_id,
        driver_id: update.driver_id,
        customer_name: update.customer_name,
        customer_phone: update.customer_phone,
        origin: update.origin,
        destination: update.destination,
        cargo_type: update.cargo_type,
        cargo_weight: update.cargo_weight,
        cargo_volume: update.cargo_volume,
        cargo_count: update.cargo_count,
        order_amount: update.order_amount,
        order_status: update.order_status,
        departure_time: update.departure_time,
        arrival_time: update.arrival_time,
        remark: update.remark,
    }
}

/// 将 PagedResult<OrderDto> 转换为 PagedResponse<OrderResponse>
async fn paged_result_to_response(
    service: &OrderApplicationService,
    result: PagedResult<OrderDto>,
) -> PagedResponse<OrderResponse> {
    let mut responses = Vec::with_capacity(result.items.len());
    
    for dto in result.items {
        let (vehicle_name, license_plate) = service
            .get_vehicle_info(dto.vehicle_id)
            .await
            .ok()
            .flatten()
            .unwrap_or_else(|| ("Unknown Vehicle".to_string(), "Unknown Plate".to_string()));
        
        let driver_name = if let Some(driver_id) = dto.driver_id {
            service.get_driver_name(driver_id).await.ok().flatten()
        } else {
            None
        };
        
        responses.push(order_dto_to_response(dto, vehicle_name, license_plate, driver_name));
    }

    PagedResponse {
        list: responses,
        total: result.total,
        page: result.page,
        page_size: result.page_size,
        pages: ((result.total + result.page_size as i64 - 1) / result.page_size as i64) as i32,
    }
}

/// 从查询参数构建 OrderQuery
fn build_order_query(
    page: Option<i32>,
    page_size: Option<i32>,
    order_no: Option<String>,
    customer_name: Option<String>,
    order_status: Option<i16>,
    vehicle_id: Option<i32>,
) -> OrderQuery {
    OrderQuery {
        page,
        page_size,
        order_no,
        vehicle_id,
        customer_name,
        order_status,
        origin: None,
        destination: None,
    }
}

// ============== 查询参数结构体 ==============

/// 获取订单列表的查询参数
#[derive(Debug, Deserialize, ToSchema)]
pub struct OrderListQuery {
    pub page: Option<i32>,
    pub page_size: Option<i32>,
    pub order_no: Option<String>,
    pub customer_name: Option<String>,
    pub status: Option<i16>,
    pub vehicle_id: Option<i32>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}

// ============== 路由处理器 ==============

/// 获取所有订单(支持分页和筛选)
#[utoipa::path(
    get, path = "/orders",
    responses(
        (status = 200, description = "Orders fetched successfully", body = ApiResponse<PagedResponse<OrderResponse>>)
    )
)]
pub async fn get_orders(
    service: web::Data<Arc<OrderApplicationService>>,
    query: web::Query<OrderListQuery>,
) -> AppResult<HttpResponse> {
    // 构建查询条件
    let order_query = build_order_query(
        query.page,
        query.page_size,
        query.order_no.clone(),
        query.customer_name.clone(),
        query.status,
        query.vehicle_id,
    );

    // 通过应用服务获取订单列表
    let result = service.list_orders(order_query).await?;

    // 转换为响应格式
    let paged_response = paged_result_to_response(&service, result).await;

    Ok(success_response_with_message(
        "Orders fetched successfully",
        paged_response,
    ))
}

/// 获取单个订单
#[utoipa::path(
    get, path = "/orders/{id}",
    responses(
        (status = 200, description = "Order fetched successfully", body = ApiResponse<OrderResponse>),
        (status = 404, description = "Order not found", body = ApiResponse<OrderResponse>)
    )
)]
pub async fn get_order(
    service: web::Data<Arc<OrderApplicationService>>,
    order_id: web::Path<i32>,
) -> AppResult<HttpResponse> {
    let result = service.get_order(*order_id).await?;

    match result {
        Some(dto) => {
            let (vehicle_name, license_plate) = service
                .get_vehicle_info(dto.vehicle_id)
                .await
                .ok()
                .flatten()
                .unwrap_or_else(|| ("Unknown Vehicle".to_string(), "Unknown Plate".to_string()));
            
            let driver_name = if let Some(driver_id) = dto.driver_id {
                service.get_driver_name(driver_id).await.ok().flatten()
            } else {
                None
            };
            
            let response = order_dto_to_response(dto, vehicle_name, license_plate, driver_name);
            Ok(success_response_with_message("Order fetched successfully", Some(response)))
        }
        None => Err(AppError::not_found_error("Order not found".to_string())),
    }
}

/// 创建订单
#[utoipa::path(
    post, path = "/orders",
    request_body = OrderCreate,
    responses(
        (status = 201, description = "Order created successfully", body = ApiResponse<OrderResponse>),
        (status = 400, description = "Invalid input data", body = ApiResponse<OrderResponse>),
        (status = 500, description = "Failed to create order", body = ApiResponse<OrderResponse>)
    )
)]
pub async fn create_order(
    service: web::Data<Arc<OrderApplicationService>>,
    order: web::Json<SchemaOrderCreate>,
) -> AppResult<HttpResponse> {
    // 数据校验
    if order.customer_name.is_empty() {
        return Err(AppError::validation("Customer name is required"));
    }

    if order.origin.is_empty() || order.destination.is_empty() {
        return Err(AppError::validation("Origin and destination are required"));
    }

    // 生成订单号(格式:TMS+年月日+6位随机数)
    let now = chrono::Local::now();
    let date_str = now.format("%Y%m%d").to_string();
    let random_num = rand::random::<u32>() % 1000000;
    let order_no = format!("TMS{}{:06}", date_str, random_num);

    // 转换为领域层类型
    let domain_create = schema_create_to_domain(order.into_inner(), order_no);

    // 通过应用服务创建订单
    let dto = service.create_order(domain_create).await?;
    
    let (vehicle_name, license_plate) = service
        .get_vehicle_info(dto.vehicle_id)
        .await
        .ok()
        .flatten()
        .unwrap_or_else(|| ("Unknown Vehicle".to_string(), "Unknown Plate".to_string()));
    
    let driver_name = if let Some(driver_id) = dto.driver_id {
        service.get_driver_name(driver_id).await.ok().flatten()
    } else {
        None
    };
    
    let response = order_dto_to_response(dto, vehicle_name, license_plate, driver_name);

    Ok(success_response_with_message(
        "Order created successfully",
        Some(response),
    ))
}

/// 更新订单
#[utoipa::path(
    put, path = "/orders/{id}",
    request_body = OrderUpdate,
    responses(
        (status = 200, description = "Order updated successfully", body = ApiResponse<OrderResponse>),
        (status = 400, description = "Invalid input data", body = ApiResponse<OrderResponse>),
        (status = 404, description = "Order not found", body = ApiResponse<OrderResponse>),
        (status = 500, description = "Failed to update order", body = ApiResponse<OrderResponse>)
    )
)]
pub async fn update_order(
    service: web::Data<Arc<OrderApplicationService>>,
    order_id: web::Path<i32>,
    order: web::Json<SchemaOrderUpdate>,
) -> AppResult<HttpResponse> {
    // 转换为领域层类型
    let domain_update = schema_update_to_domain(order.into_inner());

    // 通过应用服务更新订单
    let result = service.update_order(*order_id, domain_update).await?;

    match result {
        Some(dto) => {
            let (vehicle_name, license_plate) = service
                .get_vehicle_info(dto.vehicle_id)
                .await
                .ok()
                .flatten()
                .unwrap_or_else(|| ("Unknown Vehicle".to_string(), "Unknown Plate".to_string()));
            
            let driver_name = if let Some(driver_id) = dto.driver_id {
                service.get_driver_name(driver_id).await.ok().flatten()
            } else {
                None
            };
            
            let response = order_dto_to_response(dto, vehicle_name, license_plate, driver_name);
            Ok(success_response_with_message("Order updated successfully", Some(response)))
        }
        None => Err(AppError::not_found_error("Order not found".to_string())),
    }
}

/// 删除订单
#[utoipa::path(
    delete, path = "/orders/{id}",
    responses(
        (status = 200, description = "Order deleted successfully", body = ApiResponse<()>),
        (status = 404, description = "Order not found", body = ApiResponse<OrderResponse>)
    )
)]
pub async fn delete_order(
    service: web::Data<Arc<OrderApplicationService>>,
    order_id: web::Path<i32>,
) -> AppResult<HttpResponse> {
    let result = service.delete_order(*order_id).await?;

    if result {
        Ok(success_response_with_message("Order deleted successfully", ()))
    } else {
        Err(AppError::not_found_error("Order not found".to_string()))
    }
}

// ============== 订单项相关路由 ==============

/// 创建订单项
#[utoipa::path(
    post, path = "/orders/{order_id}/items",
    request_body = OrderItemCreate,
    responses(
        (status = 201, description = "Order item created successfully", body = ApiResponse<OrderItemDto>),
        (status = 400, description = "Invalid input data", body = ApiResponse<OrderItemDto>),
        (status = 500, description = "Failed to create order item", body = ApiResponse<OrderItemDto>)
    )
)]
pub async fn create_order_item(
    service: web::Data<Arc<OrderApplicationService>>,
    order_id: web::Path<i32>,
    item: web::Json<SchemaOrderItemCreate>,
) -> AppResult<HttpResponse> {
    let domain_item = OrderItemCreate {
        order_id: *order_id,
        item_name: item.item_name.clone(),
        item_description: item.item_description.clone(),
        quantity: item.quantity,
        unit_price: item.unit_price,
    };

    let new_item: crate::application::dto::OrderItemDto = service.create_order_item(domain_item).await?;

    Ok(success_response_with_message(
        "Order item created successfully",
        Some(new_item),
    ))
}

/// 更新订单项
#[utoipa::path(
    put, path = "/orders/items/{item_id}",
    request_body = OrderItemUpdate,
    responses(
        (status = 200, description = "Order item updated successfully", body = ApiResponse<OrderItemDto>),
        (status = 404, description = "Order item not found", body = ApiResponse<OrderItemDto>),
        (status = 500, description = "Failed to update order item", body = ApiResponse<OrderItemDto>)
    )
)]
pub async fn update_order_item(
    service: web::Data<Arc<OrderApplicationService>>,
    item_id: web::Path<i32>,
    item: web::Json<OrderItemUpdate>,
) -> AppResult<HttpResponse> {
    let result = service
        .update_order_item(
            *item_id,
            item.item_name.clone(),
            item.item_description.clone(),
            item.quantity,
            item.unit_price,
        )
        .await?;

    match result {
        Some(updated_item) => Ok(success_response_with_message(
            "Order item updated successfully",
            Some(updated_item),
        )),
        None => Err(AppError::not_found_error("Order item not found".to_string())),
    }
}

/// 删除订单项
#[utoipa::path(
    delete, path = "/orders/items/{item_id}",
    responses(
        (status = 200, description = "Order item deleted successfully", body = ApiResponse<()>),
        (status = 404, description = "Order item not found", body = ApiResponse<OrderItemDto>)
    )
)]
pub async fn delete_order_item(
    service: web::Data<Arc<OrderApplicationService>>,
    item_id: web::Path<i32>,
) -> AppResult<HttpResponse> {
    let result = service.delete_order_item(*item_id).await?;

    if result {
        Ok(success_response_with_message(
            "Order item deleted successfully",
            (),
        ))
    } else {
        Err(AppError::not_found_error("Order item not found".to_string()))
    }
}

// ============== 物流跟踪相关路由 ==============

/// 创建物流跟踪记录
#[utoipa::path(
    post, path = "/orders/{order_id}/tracks",
    request_body = LogisticsTrackCreate,
    responses(
        (status = 201, description = "Logistics track created successfully", body = ApiResponse<LogisticsTrackResponse>),
        (status = 400, description = "Invalid input data", body = ApiResponse<LogisticsTrackResponse>),
        (status = 500, description = "Failed to create logistics track", body = ApiResponse<LogisticsTrackResponse>)
    )
)]
pub async fn create_logistics_track(
    service: web::Data<Arc<OrderApplicationService>>,
    order_id: web::Path<i32>,
    track: web::Json<LogisticsTrackCreate>,
) -> AppResult<HttpResponse> {
    let new_track = service
        .create_logistics_track(
            *order_id,
            track.vehicle_id,
            track.track_time,
            track.latitude,
            track.longitude,
            track.address.clone(),
            track.status,
            track.remark.clone(),
        )
        .await?;

    // 获取车辆信息用于响应
    let vehicle_result: Result<Option<(String, String)>, crate::errors::AppError> = service
        .get_vehicle_info(new_track.vehicle_id)
        .await;
    let vehicle_name = vehicle_result.ok().flatten().map(|(name, _)| name).unwrap_or_default();

    // 获取订单号用于响应
    let order: Option<crate::application::dto::OrderDto> = service.get_order(*order_id).await?;
    let order_no = order.map(|o| o.order_no).unwrap_or_default();

    let response = LogisticsTrackResponse {
        track_id: new_track.track_id,
        order_id: new_track.order_id,
        order_no,
        vehicle_id: new_track.vehicle_id,
        vehicle_name,
        track_time: new_track.track_time.unwrap_or(chrono::Utc::now()),
        latitude: new_track.latitude,
        longitude: new_track.longitude,
        address: new_track.address.unwrap_or_default(),
        status: new_track.status as i16,
        status_text: match new_track.status {
            1 => "待分配".to_string(),
            2 => "运输中".to_string(),
            3 => "已完成".to_string(),
            4 => "已取消".to_string(),
            _ => "未知状态".to_string(),
        },
        remark: new_track.remark,
        create_time: new_track.create_time,
    };

    Ok(success_response_with_message(
        "Logistics track created successfully",
        Some(response),
    ))
}

/// 更新物流跟踪记录
#[utoipa::path(
    put, path = "/orders/tracks/{track_id}",
    request_body = LogisticsTrackCreate,
    responses(
        (status = 200, description = "Logistics track updated successfully", body = ApiResponse<LogisticsTrackResponse>),
        (status = 404, description = "Logistics track not found", body = ApiResponse<LogisticsTrackResponse>),
        (status = 500, description = "Failed to update logistics track", body = ApiResponse<LogisticsTrackResponse>)
    )
)]
pub async fn update_logistics_track(
    service: web::Data<Arc<OrderApplicationService>>,
    track_id: web::Path<i32>,
    track: web::Json<LogisticsTrackUpdate>,
) -> AppResult<HttpResponse> {
    let result = service
        .update_logistics_track(
            *track_id,
            Some(track.vehicle_id),
            Some(track.track_time),
            Some(track.latitude),
            Some(track.longitude),
            Some(track.address.clone()),
            Some(track.status),
            track.remark.clone(),
        )
        .await?;

    match result {
        Some(updated_track) => {
            // 获取车辆信息用于响应
            let vehicle_result: Result<Option<(String, String)>, crate::errors::AppError> = service
                .get_vehicle_info(updated_track.vehicle_id)
                .await;
            let vehicle_name = vehicle_result.ok().flatten().map(|(name, _)| name).unwrap_or_default();

            // 获取订单号用于响应
            let order: Option<crate::application::dto::OrderDto> = service.get_order(updated_track.order_id).await?;
            let order_no = order.map(|o| o.order_no).unwrap_or_default();

            let response = LogisticsTrackResponse {
                track_id: updated_track.track_id,
                order_id: updated_track.order_id,
                order_no,
                vehicle_id: updated_track.vehicle_id,
                vehicle_name,
                track_time: updated_track.track_time.unwrap_or(chrono::Utc::now()),
                latitude: updated_track.latitude,
                longitude: updated_track.longitude,
                address: updated_track.address.unwrap_or_default(),
                status: updated_track.status as i16,
                status_text: match updated_track.status {
                    1 => "待分配".to_string(),
                    2 => "运输中".to_string(),
                    3 => "已完成".to_string(),
                    4 => "已取消".to_string(),
                    _ => "未知状态".to_string(),
                },
                remark: updated_track.remark,
                create_time: updated_track.create_time,
            };

            Ok(success_response_with_message(
                "Logistics track updated successfully",
                Some(response),
            ))
        }
        None => Err(AppError::not_found_error(
            "Logistics track not found".to_string(),
        )),
    }
}

/// 删除物流跟踪记录
#[utoipa::path(
    delete, path = "/orders/tracks/{track_id}",
    responses(
        (status = 200, description = "Logistics track deleted successfully", body = ApiResponse<()>),
        (status = 404, description = "Logistics track not found", body = ApiResponse<LogisticsTrackResponse>)
    )
)]
pub async fn delete_logistics_track(
    service: web::Data<Arc<OrderApplicationService>>,
    track_id: web::Path<i32>,
) -> AppResult<HttpResponse> {
    let result = service.delete_logistics_track(*track_id).await?;

    if result {
        Ok(success_response_with_message(
            "Logistics track deleted successfully",
            (),
        ))
    } else {
        Err(AppError::not_found_error(
            "Logistics track not found".to_string(),
        ))
    }
}

/// 批量创建物流轨迹
#[utoipa::path(
    post, path = "/orders/{order_id}/tracks/batch",
    tag = "orders",
    request_body = LogisticsTrackCreateBatch,
    responses(
        (status = 201, description = "Logistics tracks created successfully", body = ApiResponse<Vec<LogisticsTrackResponse>>),
        (status = 400, description = "Invalid input data", body = ApiResponse<Vec<LogisticsTrackResponse>>),
        (status = 500, description = "Failed to create logistics tracks", body = ApiResponse<Vec<LogisticsTrackResponse>>)
    )
)]
pub async fn create_logistics_tracks_batch(
    service: web::Data<Arc<OrderApplicationService>>,
    order_id: web::Path<i32>,
    batch_tracks: web::Json<LogisticsTrackCreateBatch>,
) -> AppResult<HttpResponse> {
    let tracks = &batch_tracks.tracks;
    if tracks.is_empty() {
        return Err(AppError::validation("No tracks provided"));
    }

    // 批量插入轨迹数据
    let results: Vec<crate::models::LogisticsTrack> = service
        .create_logistics_tracks_batch(*order_id, tracks.to_vec())
        .await?;

    // 获取订单信息
    let order: Option<crate::application::dto::OrderDto> = service.get_order(*order_id).await?;
    let order_no = order.map(|o| o.order_no).unwrap_or_default();

    // 构建响应
    let mut responses = Vec::with_capacity(results.len());
    for track in results {
        let vehicle_result: Result<Option<(String, String)>, crate::errors::AppError> = service
            .get_vehicle_info(track.vehicle_id)
            .await;
        let vehicle_name = vehicle_result.ok().flatten().map(|(name, _)| name).unwrap_or_default();

        responses.push(LogisticsTrackResponse {
            track_id: track.track_id,
            order_id: track.order_id,
            order_no: order_no.clone(),
            vehicle_id: track.vehicle_id,
            vehicle_name,
            track_time: track.track_time.unwrap_or(chrono::Utc::now()),
            latitude: track.latitude,
            longitude: track.longitude,
            address: track.address.unwrap_or_default(),
            status: track.status as i16,
            status_text: match track.status {
                1 => "待分配".to_string(),
                2 => "运输中".to_string(),
                3 => "已完成".to_string(),
                4 => "已取消".to_string(),
                _ => "未知状态".to_string(),
            },
            remark: track.remark,
            create_time: track.create_time,
        });
    }

    let response_count = responses.len();
    Ok(success_response_with_message(
        &format!("Successfully created {} logistics tracks", response_count),
        Some(responses),
    ))
}

// ============== 车辆轨迹查询 ==============

/// 获取车辆轨迹数据(用于轨迹回放)
#[derive(Debug, Deserialize, ToSchema)]
pub struct TrackQuery {
    pub vehicle_id: i32,
    pub start_time: String,
    pub end_time: String,
    pub page: Option<i32>,
    pub page_size: Option<i32>,
}

#[utoipa::path(
    get, path = "/api/tracks",
    responses(
        (status = 200, description = "Track data fetched successfully", body = ApiResponse<PagedResponse<LogisticsTrackResponse>>),
        (status = 400, description = "Invalid date format", body = ApiResponse<PagedResponse<LogisticsTrackResponse>>),
        (status = 500, description = "Failed to fetch track data", body = ApiResponse<PagedResponse<LogisticsTrackResponse>>)
    )
)]
pub async fn get_vehicle_tracks(
    service: web::Data<Arc<OrderApplicationService>>,
    query: web::Query<VehicleTracksQuery>,
) -> AppResult<HttpResponse> {
    // 解析ISO字符串为NaiveDateTime
    let start_time = DateTime::parse_from_rfc3339(&query.start_time)
        .map_err(|e| AppError::validation(&format!("Invalid start_time format: {}", e)))?
        .naive_utc();

    let end_time = DateTime::parse_from_rfc3339(&query.end_time)
        .map_err(|e| AppError::validation(&format!("Invalid end_time format: {}", e)))?
        .naive_utc();

    // 处理分页参数
    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(100);

    // 通过应用服务获取轨迹数据
    let result = service
        .get_vehicle_tracks(query.vehicle_id, start_time, end_time, page, page_size)
        .await?;

    // 构建响应
    let mut tracks = Vec::with_capacity(result.items.len());
    for track in result.items {
        let vehicle_result: Result<Option<(String, String)>, crate::errors::AppError> = service
            .get_vehicle_info(track.vehicle_id)
            .await;
        let vehicle_name = vehicle_result.ok().flatten().map(|(name, _)| name).unwrap_or_default();

        let order: Option<crate::application::dto::OrderDto> = service.get_order(track.order_id).await?;
        let order_no = order.map(|o| o.order_no).unwrap_or_default();

        tracks.push(LogisticsTrackResponse {
            track_id: track.track_id,
            order_id: track.order_id,
            order_no,
            vehicle_id: track.vehicle_id,
            vehicle_name,
            track_time: track.track_time.unwrap_or(chrono::Utc::now()),
            latitude: track.latitude,
            longitude: track.longitude,
            address: track.address.unwrap_or_default(),
            status: track.status as i16,
            status_text: match track.status {
                1 => "待分配".to_string(),
                2 => "运输中".to_string(),
                3 => "已完成".to_string(),
                4 => "已取消".to_string(),
                _ => "未知状态".to_string(),
            },
            remark: track.remark,
            create_time: track.create_time,
        });
    }

    // 构造分页响应
    let paged_response = PagedResponse {
        list: tracks,
        total: result.total,
        page: result.page,
        page_size: result.page_size,
        pages: ((result.total + result.page_size as i64 - 1) / result.page_size as i64) as i32,
    };

    Ok(success_response_with_message(
        "Vehicle tracking data fetched successfully",
        Some(paged_response),
    ))
}
