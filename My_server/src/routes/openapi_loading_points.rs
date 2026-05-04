use actix_web::{web, HttpResponse};
use log::info;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::errors::{success_response_with_message, AppResult};
use crate::schemas::PagedResponse;

// 加载点响应体
#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct LoadingPointResponse {
    pub id: i32,
    pub name: String,
    pub address: String,
    pub latitude: f64,
    pub longitude: f64,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

// 加载点查询参数
#[derive(Debug, Clone, Deserialize, ToSchema, utoipa::IntoParams)]
pub struct LoadingPointQuery {
    pub page: Option<i32>,
    pub page_size: Option<i32>,
    pub name: Option<String>,
    pub status: Option<String>,
}

// 获取加载点列表
#[utoipa::path(
    path = "/api/openapi/loading-points",
    get,
    params(LoadingPointQuery),
    responses(
        (status = 200, description = "Loading points fetched successfully", body = ApiResponse<PagedResponse<LoadingPointResponse>>),
        (status = 500, description = "Internal server error", body = ApiResponse<()>)
    )
)]
pub async fn get_loading_points(query: web::Query<LoadingPointQuery>) -> AppResult<HttpResponse> {
    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20);

    // 模拟数据 - 实际应该从数据库中获取
    let loading_points = vec![
        LoadingPointResponse {
            id: 1,
            name: "Loading Point 1".to_string(),
            address: "123 Main St, City, Country".to_string(),
            latitude: 39.9042,
            longitude: 116.4074,
            status: "active".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: None,
        },
        LoadingPointResponse {
            id: 2,
            name: "Loading Point 2".to_string(),
            address: "456 Elm St, City, Country".to_string(),
            latitude: 31.2304,
            longitude: 121.4737,
            status: "active".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: None,
        },
        LoadingPointResponse {
            id: 3,
            name: "Loading Point 3".to_string(),
            address: "789 Oak St, City, Country".to_string(),
            latitude: 23.1291,
            longitude: 113.2644,
            status: "inactive".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: None,
        },
    ];

    // 过滤加载点
    let filtered_loading_points: Vec<LoadingPointResponse> = loading_points
        .into_iter()
        .filter(|point| {
            if let Some(name) = &query.name {
                if !point.name.contains(name) {
                    return false;
                }
            }
            if let Some(status) = &query.status {
                if point.status != *status {
                    return false;
                }
            }
            true
        })
        .collect();

    // 分页处理
    let total = filtered_loading_points.len() as i64;
    let start = ((page - 1) * page_size) as usize;
    let end = (start + page_size as usize).min(filtered_loading_points.len());
    let paginated_loading_points = filtered_loading_points[start..end].to_vec();

    // 计算总页数
    let pages = if total % page_size as i64 == 0 {
        total / page_size as i64
    } else {
        total / page_size as i64 + 1
    };

    // 构建分页响应
    let paged_response = PagedResponse {
        list: paginated_loading_points,
        total,
        page,
        page_size,
        pages: pages as i32,
    };

    info!("Loading points fetched successfully");
    Ok(success_response_with_message(
        "Loading points fetched successfully",
        Some(paged_response),
    ))
}

// 配置加载点路由
pub fn configure_loading_points_routes(cfg: &mut web::ServiceConfig) {
    cfg.route("/openapi/loading-points", web::get().to(get_loading_points));
}
