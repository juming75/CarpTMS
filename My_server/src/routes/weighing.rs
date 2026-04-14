use actix_web::{web, HttpResponse};
use chrono::{DateTime, Utc};

use crate::application::services::weighing_data_service::{WeighingDataService, WeighingDataServiceImpl};
use crate::domain::entities::weighing_data::{WeighingDataCreate, WeighingDataQuery, WeighingDataUpdate};
use crate::errors::{created_response_with_message, empty_success_response, success_response_with_message, AppError, AppResult};
use crate::schemas::{PagedResponse, WeighingDataCreate as SchemaWeighingDataCreate, WeighingDataResponse, WeighingHistoryQuery};

// Get weighing data list (with pagination)
#[utoipa::path(
    path = "/api/weighing",
    get,
    responses(
        (status = 200, description = "Weighing data fetched successfully", body = ApiResponse<PagedResponse<WeighingDataResponse>>),
        (status = 500, description = "Internal server error", body = ApiResponse<()>)
    )
)]
pub async fn get_weighing_data(
    weighing_data_service: web::Data<WeighingDataServiceImpl>,
    query: web::Query<WeighingHistoryQuery>,
) -> AppResult<HttpResponse> {
    // Convert query parameters
    let domain_query = WeighingDataQuery {
        page: Some(query.page),
        page_size: Some(query.page_size),
        vehicle_id: query.vehicle_id,
        device_id: None,
        start_time: query.start_time,
        end_time: query.end_time,
        status: None,
        min_net_weight: None,
        max_net_weight: None,
    };

    // Get weighing data list from service
    let (weighing_data, total_count) = weighing_data_service.get_weighing_data_list(domain_query).await?;

    // Convert to response format
    let weighing_responses: Vec<WeighingDataResponse> = weighing_data
        .into_iter()
        .map(|data| WeighingDataResponse {
            id: data.id,
            vehicle_id: data.vehicle_id,
            vehicle_name: data.vehicle_name,
            device_id: data.device_id,
            weighing_time: DateTime::<Utc>::from_naive_utc_and_offset(data.weighing_time, Utc),
            gross_weight: data.gross_weight,
            tare_weight: data.tare_weight,
            net_weight: data.net_weight,
            axle_count: data.axle_count,
            speed: data.speed,
            lane_no: data.lane_no,
            site_id: data.site_id,
            status: data.status,
            create_time: DateTime::<Utc>::from_naive_utc_and_offset(data.create_time, Utc),
            update_time: data.update_time.map(|t| DateTime::<Utc>::from_naive_utc_and_offset(t, Utc)),
        })
        .collect();

    // Create paged response
    let paged_response = PagedResponse {
        list: weighing_responses,
        total: total_count,
        page: query.page,
        page_size: query.page_size,
        pages: ((total_count + query.page_size as i64 - 1) / query.page_size as i64) as i32,
    };

    Ok(success_response_with_message(
        "Weighing data fetched successfully",
        Some(paged_response),
    ))
}

// Create weighing data
#[utoipa::path(
    path = "/api/weighing",
    post,
    request_body = WeighingDataCreate,
    responses(
        (status = 201, description = "Weighing data created successfully", body = ApiResponse<WeighingDataResponse>),
        (status = 400, description = "Invalid request parameters", body = ApiResponse<()>),
        (status = 500, description = "Internal server error", body = ApiResponse<()>)
    )
)]
pub async fn create_weighing_data(
    weighing_data_service: web::Data<WeighingDataServiceImpl>,
    data: web::Json<SchemaWeighingDataCreate>,
) -> AppResult<HttpResponse> {
    // Validate input data
    if data.vehicle_id <= 0 {
        return Err(AppError::validation("Invalid vehicle ID"));
    }

    if data.gross_weight <= 0.0 {
        return Err(AppError::validation("Gross weight must be positive"));
    }

    if data.net_weight <= 0.0 {
        return Err(AppError::validation("Net weight must be positive"));
    }

    // Convert to domain model
    let domain_data = WeighingDataCreate {
        vehicle_id: data.vehicle_id,
        device_id: data.device_id.clone(),
        weighing_time: data.weighing_time,
        gross_weight: data.gross_weight,
        tare_weight: data.tare_weight,
        net_weight: data.net_weight,
        axle_count: data.axle_count,
        speed: data.speed,
        lane_no: data.lane_no,
        site_id: data.site_id,
        status: data.status,
    };

    // Create weighing data via service
    let created_data = weighing_data_service.create_weighing_data(domain_data).await?;

    // Convert to response format
    let response = WeighingDataResponse {
        id: created_data.id,
        vehicle_id: created_data.vehicle_id,
        vehicle_name: created_data.vehicle_name,
        device_id: created_data.device_id,
        weighing_time: DateTime::<Utc>::from_naive_utc_and_offset(created_data.weighing_time, Utc),
        gross_weight: created_data.gross_weight,
        tare_weight: created_data.tare_weight,
        net_weight: created_data.net_weight,
        axle_count: created_data.axle_count,
        speed: created_data.speed,
        lane_no: created_data.lane_no,
        site_id: created_data.site_id,
        status: created_data.status,
        create_time: DateTime::<Utc>::from_naive_utc_and_offset(created_data.create_time, Utc),
        update_time: created_data.update_time.map(|t| DateTime::<Utc>::from_naive_utc_and_offset(t, Utc)),
    };

    Ok(created_response_with_message(
        "Weighing data created successfully",
        Some(response),
    ))
}

// Get weighing history (with pagination)
#[utoipa::path(
    path = "/api/weighing/history",
    get,
    responses(
        (status = 200, description = "Weighing history fetched successfully", body = ApiResponse<PagedResponse<WeighingDataResponse>>),
        (status = 500, description = "Internal server error", body = ApiResponse<()>)
    )
)]
pub async fn get_weighing_history(
    weighing_data_service: web::Data<WeighingDataServiceImpl>,
    query: web::Query<WeighingHistoryQuery>,
) -> AppResult<HttpResponse> {
    // Convert query parameters
    let domain_query = WeighingDataQuery {
        page: Some(query.page),
        page_size: Some(query.page_size),
        vehicle_id: query.vehicle_id,
        device_id: None,
        start_time: query.start_time,
        end_time: query.end_time,
        status: None,
        min_net_weight: None,
        max_net_weight: None,
    };

    // Get weighing history from service
    let (weighing_data, total_count) = weighing_data_service.get_weighing_data_list(domain_query).await?;

    // Convert to response format
    let weighing_responses: Vec<WeighingDataResponse> = weighing_data
        .into_iter()
        .map(|data| WeighingDataResponse {
            id: data.id,
            vehicle_id: data.vehicle_id,
            vehicle_name: data.vehicle_name,
            device_id: data.device_id,
            weighing_time: DateTime::<Utc>::from_naive_utc_and_offset(data.weighing_time, Utc),
            gross_weight: data.gross_weight,
            tare_weight: data.tare_weight,
            net_weight: data.net_weight,
            axle_count: data.axle_count,
            speed: data.speed,
            lane_no: data.lane_no,
            site_id: data.site_id,
            status: data.status,
            create_time: DateTime::<Utc>::from_naive_utc_and_offset(data.create_time, Utc),
            update_time: data.update_time.map(|t| DateTime::<Utc>::from_naive_utc_and_offset(t, Utc)),
        })
        .collect();

    // Create paged response
    let paged_response = PagedResponse {
        list: weighing_responses,
        total: total_count,
        page: query.page,
        page_size: query.page_size,
        pages: ((total_count + query.page_size as i64 - 1) / query.page_size as i64) as i32,
    };

    Ok(success_response_with_message(
        "Weighing history fetched successfully",
        Some(paged_response),
    ))
}

// Get weighing data by ID
#[utoipa::path(
    path = "/api/weighing/{id}",
    get,
    responses(
        (status = 200, description = "Weighing data fetched successfully", body = ApiResponse<WeighingDataResponse>),
        (status = 404, description = "Weighing data not found", body = ApiResponse<()>),
        (status = 500, description = "Internal server error", body = ApiResponse<()>)
    )
)]
pub async fn get_weighing_data_by_id(
    weighing_data_service: web::Data<WeighingDataServiceImpl>,
    id: web::Path<i32>,
) -> AppResult<HttpResponse> {
    // Get weighing data by ID from service
    let weighing_data = weighing_data_service.get_weighing_data(*id).await?;

    match weighing_data {
        Some(data) => {
            let response = WeighingDataResponse {
                id: data.id,
                vehicle_id: data.vehicle_id,
                vehicle_name: data.vehicle_name,
                device_id: data.device_id,
                weighing_time: DateTime::<Utc>::from_naive_utc_and_offset(data.weighing_time, Utc),
                gross_weight: data.gross_weight,
                tare_weight: data.tare_weight,
                net_weight: data.net_weight,
                axle_count: data.axle_count,
                speed: data.speed,
                lane_no: data.lane_no,
                site_id: data.site_id,
                status: data.status,
                create_time: DateTime::<Utc>::from_naive_utc_and_offset(data.create_time, Utc),
                update_time: data.update_time.map(|t| DateTime::<Utc>::from_naive_utc_and_offset(t, Utc)),
            };

            Ok(success_response_with_message(
                "Weighing data fetched successfully",
                Some(response),
            ))
        }
        None => Err(AppError::not_found_error(
            "Weighing data not found".to_string(),
        )),
    }
}

// Update weighing data
#[utoipa::path(
    path = "/api/weighing/{id}",
    put,
    request_body = WeighingDataCreate,
    responses(
        (status = 200, description = "Weighing data updated successfully", body = ApiResponse<WeighingDataResponse>),
        (status = 400, description = "Invalid request parameters", body = ApiResponse<()>),
        (status = 404, description = "Weighing data not found", body = ApiResponse<()>),
        (status = 500, description = "Internal server error", body = ApiResponse<()>)
    )
)]
pub async fn update_weighing_data(
    weighing_data_service: web::Data<WeighingDataServiceImpl>,
    id: web::Path<i32>,
    data: web::Json<SchemaWeighingDataCreate>,
) -> AppResult<HttpResponse> {
    // Validate input data
    if data.gross_weight <= 0.0 {
        return Err(AppError::validation("Gross weight must be positive"));
    }

    if data.net_weight <= 0.0 {
        return Err(AppError::validation("Net weight must be positive"));
    }

    // Convert to update domain model
    let domain_data = WeighingDataUpdate {
        vehicle_id: Some(data.vehicle_id),
        device_id: Some(data.device_id.clone()),
        weighing_time: Some(data.weighing_time),
        gross_weight: Some(data.gross_weight),
        tare_weight: data.tare_weight,
        net_weight: Some(data.net_weight),
        axle_count: data.axle_count,
        speed: data.speed,
        lane_no: data.lane_no,
        site_id: data.site_id,
        status: Some(data.status),
    };

    // Update weighing data via service
    let updated_data = weighing_data_service.update_weighing_data(*id, domain_data).await?;

    match updated_data {
        Some(data) => {
            // Convert to response format
            let response = WeighingDataResponse {
                id: data.id,
                vehicle_id: data.vehicle_id,
                vehicle_name: data.vehicle_name,
                device_id: data.device_id,
                weighing_time: DateTime::<Utc>::from_naive_utc_and_offset(data.weighing_time, Utc),
                gross_weight: data.gross_weight,
                tare_weight: data.tare_weight,
                net_weight: data.net_weight,
                axle_count: data.axle_count,
                speed: data.speed,
                lane_no: data.lane_no,
                site_id: data.site_id,
                status: data.status,
                create_time: DateTime::<Utc>::from_naive_utc_and_offset(data.create_time, Utc),
                update_time: data.update_time.map(|t| DateTime::<Utc>::from_naive_utc_and_offset(t, Utc)),
            };

            Ok(success_response_with_message(
                "Weighing data updated successfully",
                Some(response),
            ))
        }
        None => Err(AppError::not_found_error(
            "Weighing data not found".to_string(),
        )),
    }
}

// Delete weighing data
#[utoipa::path(
    path = "/api/weighing/{id}",
    delete,
    responses(
        (status = 200, description = "Weighing data deleted successfully", body = ApiResponse<()>),
        (status = 404, description = "Weighing data not found", body = ApiResponse<()>),
        (status = 500, description = "Internal server error", body = ApiResponse<()>)
    )
)]
pub async fn delete_weighing_data(
    weighing_data_service: web::Data<WeighingDataServiceImpl>,
    id: web::Path<i32>,
) -> AppResult<HttpResponse> {
    // Delete weighing data via service
    let deleted = weighing_data_service.delete_weighing_data(*id).await?;

    if deleted {
        Ok(empty_success_response())
    } else {
        Err(AppError::not_found_error(
            "Weighing data not found".to_string(),
        ))
    }
}
