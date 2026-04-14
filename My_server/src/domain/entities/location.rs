use chrono::NaiveDateTime;
use serde_json;


// ==================== 电子围栏实体 ====================
#[derive(Debug, Clone)]
pub struct Fence {
    pub fence_id: i32,
    pub fence_name: String,
    pub fence_type: String, // circle, polygon, rectangle
    pub center_latitude: Option<f64>,
    pub center_longitude: Option<f64>,
    pub radius: Option<f64>,
    pub polygon_points: Option<serde_json::Value>,
    pub rectangle_bounds: Option<serde_json::Value>,
    pub status: String,
    pub description: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(Debug, Clone)]
pub struct FenceCreate {
    pub fence_name: String,
    pub fence_type: String,
    pub center_latitude: Option<f64>,
    pub center_longitude: Option<f64>,
    pub radius: Option<f64>,
    pub polygon_points: Option<serde_json::Value>,
    pub rectangle_bounds: Option<serde_json::Value>,
    pub status: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct FenceQuery {
    pub page: Option<i32>,
    pub page_size: Option<i32>,
    pub status: Option<String>,
    pub fence_type: Option<String>,
}

// ==================== 位置实体 ====================
#[derive(Debug, Clone)]
pub struct Location {
    pub position_id: i32,
    pub place_name: String,
    pub latitude: f64,
    pub longitude: f64,
    pub address: Option<String>,
    pub description: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(Debug, Clone)]
pub struct LocationCreate {
    pub location_name: String,
    pub latitude: f64,
    pub longitude: f64,
    pub address: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone)]
pub struct LocationUpdate {
    pub location_name: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub address: Option<String>,
    pub description: Option<String>,
}

// ==================== 地点实体 ====================
#[derive(Debug, Clone)]
pub struct Place {
    pub place_id: i32,
    pub place_name: String,
    pub address: String,
    pub contact_person: Option<String>,
    pub contact_phone: Option<String>,
    pub contact_email: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub description: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(Debug, Clone)]
pub struct PlaceCreate {
    pub place_name: String,
    pub address: String,
    pub contact_person: Option<String>,
    pub contact_phone: Option<String>,
    pub contact_email: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub description: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PlaceUpdate {
    pub place_name: Option<String>,
    pub address: Option<String>,
    pub contact_person: Option<String>,
    pub contact_phone: Option<String>,
    pub contact_email: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub description: Option<String>,
}

// ==================== 路线实体 ====================
#[derive(Debug, Clone)]
pub struct Route {
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
    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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
