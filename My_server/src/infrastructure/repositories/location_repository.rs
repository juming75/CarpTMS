use std::sync::Arc;

use chrono;
use sqlx::{PgPool, Row};

use crate::domain::entities::location::{
    Fence, FenceCreate, FenceQuery, FenceUpdate, Location, LocationCreate, LocationUpdate, Place,
    PlaceCreate, PlaceUpdate, Route, RouteCreate, RouteUpdate,
};
use crate::errors::AppResult;

#[async_trait::async_trait]
pub trait LocationRepository: Send + Sync {
    // ==================== 电子围栏相关 ====================
    async fn get_fences(&self, query: FenceQuery) -> AppResult<(Vec<Fence>, i64)>;
    async fn get_fence_by_id(&self, fence_id: i32) -> AppResult<Option<Fence>>;
    async fn create_fence(&self, fence: FenceCreate) -> AppResult<Fence>;
    async fn update_fence(&self, fence_id: i32, fence: FenceUpdate) -> AppResult<Option<Fence>>;
    async fn delete_fence(&self, fence_id: i32) -> AppResult<bool>;

    // ==================== 位置相关 ====================
    async fn get_locations(&self, page: i32, page_size: i32) -> AppResult<(Vec<Location>, i64)>;
    async fn create_location(&self, location: LocationCreate) -> AppResult<Location>;
    async fn update_location(
        &self,
        location_id: i32,
        location: LocationUpdate,
    ) -> AppResult<Option<Location>>;
    async fn delete_location(&self, location_id: i32) -> AppResult<bool>;

    // ==================== 地点相关 ====================
    async fn get_places(&self, page: i32, page_size: i32) -> AppResult<(Vec<Place>, i64)>;
    async fn create_place(&self, place: PlaceCreate) -> AppResult<Place>;
    async fn update_place(&self, place_id: i32, place: PlaceUpdate) -> AppResult<Option<Place>>;
    async fn delete_place(&self, place_id: i32) -> AppResult<bool>;

    // ==================== 路线相关 ====================
    async fn get_routes(&self, page: i32, page_size: i32) -> AppResult<(Vec<Route>, i64)>;
    async fn create_route(&self, route: RouteCreate) -> AppResult<Route>;
    async fn update_route(&self, route_id: i32, route: RouteUpdate) -> AppResult<Option<Route>>;
    async fn delete_route(&self, route_id: i32) -> AppResult<bool>;
}

pub struct PgLocationRepository {
    pool: Arc<PgPool>,
}

impl PgLocationRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl LocationRepository for PgLocationRepository {
    // ==================== 电子围栏相关 ====================
    async fn get_fences(&self, query: FenceQuery) -> AppResult<(Vec<Fence>, i64)> {
        let page = query.page.unwrap_or(1);
        let page_size = query.page_size.unwrap_or(20);
        let offset = (page - 1) * page_size;

        let count_query = "SELECT COUNT(*) FROM location_fences";
        let total: i64 = sqlx::query_scalar(count_query)
            .fetch_one(self.pool.as_ref())
            .await?;

        let list_query = r#"SELECT * FROM location_fences 
            ORDER BY fence_id 
            LIMIT $1 OFFSET $2"#;

        let fences: Vec<Fence> = sqlx::query(list_query)
            .bind(page_size)
            .bind(offset)
            .fetch_all(self.pool.as_ref())
            .await?
            .into_iter()
            .map(|row| Fence {
                fence_id: row.get("fence_id"),
                fence_name: row.get("fence_name"),
                fence_type: row.get("fence_type"),
                center_latitude: row.try_get("center_latitude").unwrap_or(None),
                center_longitude: row.try_get("center_longitude").unwrap_or(None),
                radius: row.try_get("radius").unwrap_or(None),
                polygon_points: row.try_get("polygon_points").unwrap_or(None),
                rectangle_bounds: row.try_get("rectangle_bounds").unwrap_or(None),
                status: row.get("status"),
                description: row.try_get("description").unwrap_or(None),
                created_at: row.get("created_at"),
                updated_at: row.try_get("updated_at").unwrap_or(None),
            })
            .collect();

        Ok((fences, total))
    }

    async fn get_fence_by_id(&self, fence_id: i32) -> AppResult<Option<Fence>> {
        let row = sqlx::query("SELECT * FROM location_fences WHERE fence_id = $1")
            .bind(fence_id)
            .fetch_optional(self.pool.as_ref())
            .await?;

        match row {
            Some(row) => {
                let fence = Fence {
                    fence_id: row.get("fence_id"),
                    fence_name: row.get("fence_name"),
                    fence_type: row.get("fence_type"),
                    center_latitude: row.try_get("center_latitude").unwrap_or(None),
                    center_longitude: row.try_get("center_longitude").unwrap_or(None),
                    radius: row.try_get("radius").unwrap_or(None),
                    polygon_points: row.try_get("polygon_points").unwrap_or(None),
                    rectangle_bounds: row.try_get("rectangle_bounds").unwrap_or(None),
                    status: row.get("status"),
                    description: row.try_get("description").unwrap_or(None),
                    created_at: row.get("created_at"),
                    updated_at: row.try_get("updated_at").unwrap_or(None),
                };
                Ok(Some(fence))
            }
            None => Ok(None),
        }
    }

    async fn create_fence(&self, fence: FenceCreate) -> AppResult<Fence> {
        let row = sqlx::query(
            r#"INSERT INTO location_fences 
            (fence_name, fence_type, center_latitude, center_longitude, radius, 
             polygon_points, rectangle_bounds, status, description, created_at, updated_at) 
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $10)
            RETURNING *"#,
        )
        .bind(&fence.fence_name)
        .bind(&fence.fence_type)
        .bind(fence.center_latitude)
        .bind(fence.center_longitude)
        .bind(fence.radius)
        .bind(&fence.polygon_points)
        .bind(&fence.rectangle_bounds)
        .bind(fence.status.as_deref().unwrap_or("active"))
        .bind(&fence.description)
        .bind(chrono::Utc::now().naive_utc())
        .fetch_one(self.pool.as_ref())
        .await?;

        Ok(Fence {
            fence_id: row.get("fence_id"),
            fence_name: row.get("fence_name"),
            fence_type: row.get("fence_type"),
            center_latitude: row.try_get("center_latitude").unwrap_or(None),
            center_longitude: row.try_get("center_longitude").unwrap_or(None),
            radius: row.try_get("radius").unwrap_or(None),
            polygon_points: row.try_get("polygon_points").unwrap_or(None),
            rectangle_bounds: row.try_get("rectangle_bounds").unwrap_or(None),
            status: row.get("status"),
            description: row.try_get("description").unwrap_or(None),
            created_at: row.get("created_at"),
            updated_at: row.try_get("updated_at").unwrap_or(None),
        })
    }

    async fn update_fence(&self, fence_id: i32, fence: FenceUpdate) -> AppResult<Option<Fence>> {
        let row = sqlx::query(
            r#"UPDATE location_fences 
            SET fence_name = COALESCE($1, fence_name),
                fence_type = COALESCE($2, fence_type),
                center_latitude = COALESCE($3, center_latitude),
                center_longitude = COALESCE($4, center_longitude),
                radius = COALESCE($5, radius),
                polygon_points = COALESCE($6, polygon_points),
                rectangle_bounds = COALESCE($7, rectangle_bounds),
                status = COALESCE($8, status),
                description = COALESCE($9, description),
                updated_at = $10
            WHERE fence_id = $11
            RETURNING *"#,
        )
        .bind(&fence.fence_name)
        .bind(&fence.fence_type)
        .bind(fence.center_latitude)
        .bind(fence.center_longitude)
        .bind(fence.radius)
        .bind(&fence.polygon_points)
        .bind(&fence.rectangle_bounds)
        .bind(&fence.status)
        .bind(&fence.description)
        .bind(chrono::Utc::now().naive_utc())
        .bind(fence_id)
        .fetch_optional(self.pool.as_ref())
        .await?;

        match row {
            Some(row) => {
                let fence = Fence {
                    fence_id: row.get("fence_id"),
                    fence_name: row.get("fence_name"),
                    fence_type: row.get("fence_type"),
                    center_latitude: row.try_get("center_latitude").unwrap_or(None),
                    center_longitude: row.try_get("center_longitude").unwrap_or(None),
                    radius: row.try_get("radius").unwrap_or(None),
                    polygon_points: row.try_get("polygon_points").unwrap_or(None),
                    rectangle_bounds: row.try_get("rectangle_bounds").unwrap_or(None),
                    status: row.get("status"),
                    description: row.try_get("description").unwrap_or(None),
                    created_at: row.get("created_at"),
                    updated_at: row.try_get("updated_at").unwrap_or(None),
                };
                Ok(Some(fence))
            }
            None => Ok(None),
        }
    }

    async fn delete_fence(&self, fence_id: i32) -> AppResult<bool> {
        let result = sqlx::query("DELETE FROM location_fences WHERE fence_id = $1")
            .bind(fence_id)
            .execute(self.pool.as_ref())
            .await?;
        Ok(result.rows_affected() > 0)
    }

    // ==================== 位置相关 ====================
    async fn get_locations(&self, page: i32, page_size: i32) -> AppResult<(Vec<Location>, i64)> {
        let offset = (page - 1) * page_size;

        let count_query = "SELECT COUNT(*) FROM location_positions";
        let total: i64 = sqlx::query_scalar(count_query)
            .fetch_one(self.pool.as_ref())
            .await?;

        let list_query =
            r#"SELECT * FROM location_positions ORDER BY position_id LIMIT $1 OFFSET $2"#;

        let locations: Vec<Location> = sqlx::query(list_query)
            .bind(page_size)
            .bind(offset)
            .fetch_all(self.pool.as_ref())
            .await?
            .into_iter()
            .map(|row| Location {
                position_id: row.get("position_id"),
                place_name: row.get("place_name"),
                latitude: row.get("latitude"),
                longitude: row.get("longitude"),
                address: row.try_get("address").unwrap_or(None),
                description: row.try_get("description").unwrap_or(None),
                created_at: row.get("created_at"),
                updated_at: row.try_get("updated_at").unwrap_or(None),
            })
            .collect();

        Ok((locations, total))
    }

    async fn create_location(&self, location: LocationCreate) -> AppResult<Location> {
        let row = sqlx::query(
            r#"INSERT INTO location_positions 
            (place_name, latitude, longitude, address, description, created_at, updated_at) 
            VALUES ($1, $2, $3, $4, $5, $6, $6)
            RETURNING *"#,
        )
        .bind(&location.location_name)
        .bind(location.latitude)
        .bind(location.longitude)
        .bind(&location.address)
        .bind(&location.description)
        .bind(chrono::Utc::now().naive_utc())
        .fetch_one(self.pool.as_ref())
        .await?;

        Ok(Location {
            position_id: row.get("position_id"),
            place_name: row.get("place_name"),
            latitude: row.get("latitude"),
            longitude: row.get("longitude"),
            address: row.try_get("address").unwrap_or(None),
            description: row.try_get("description").unwrap_or(None),
            created_at: row.get("created_at"),
            updated_at: row.try_get("updated_at").unwrap_or(None),
        })
    }

    async fn update_location(
        &self,
        location_id: i32,
        location: LocationUpdate,
    ) -> AppResult<Option<Location>> {
        let row = sqlx::query(
            r#"UPDATE location_positions 
            SET place_name = COALESCE($1, place_name),
                latitude = COALESCE($2, latitude),
                longitude = COALESCE($3, longitude),
                address = COALESCE($4, address),
                description = COALESCE($5, description),
                updated_at = $6
            WHERE position_id = $7
            RETURNING *"#,
        )
        .bind(&location.location_name)
        .bind(location.latitude)
        .bind(location.longitude)
        .bind(&location.address)
        .bind(&location.description)
        .bind(chrono::Utc::now().naive_utc())
        .bind(location_id)
        .fetch_optional(self.pool.as_ref())
        .await?;

        match row {
            Some(row) => {
                let location = Location {
                    position_id: row.get("position_id"),
                    place_name: row.get("place_name"),
                    latitude: row.get("latitude"),
                    longitude: row.get("longitude"),
                    address: row.try_get("address").unwrap_or(None),
                    description: row.try_get("description").unwrap_or(None),
                    created_at: row.get("created_at"),
                    updated_at: row.try_get("updated_at").unwrap_or(None),
                };
                Ok(Some(location))
            }
            None => Ok(None),
        }
    }

    async fn delete_location(&self, location_id: i32) -> AppResult<bool> {
        let result = sqlx::query("DELETE FROM location_positions WHERE position_id = $1")
            .bind(location_id)
            .execute(self.pool.as_ref())
            .await?;
        Ok(result.rows_affected() > 0)
    }

    // ==================== 地点相关 ====================
    async fn get_places(&self, page: i32, page_size: i32) -> AppResult<(Vec<Place>, i64)> {
        let offset = (page - 1) * page_size;

        let count_query = "SELECT COUNT(*) FROM location_places";
        let total: i64 = sqlx::query_scalar(count_query)
            .fetch_one(self.pool.as_ref())
            .await?;

        let list_query = r#"SELECT * FROM location_places ORDER BY place_id LIMIT $1 OFFSET $2"#;

        let places: Vec<Place> = sqlx::query(list_query)
            .bind(page_size)
            .bind(offset)
            .fetch_all(self.pool.as_ref())
            .await?
            .into_iter()
            .map(|row| Place {
                place_id: row.get("place_id"),
                place_name: row.get("place_name"),
                address: row.get("address"),
                contact_person: row.try_get("contact_person").unwrap_or(None),
                contact_phone: row.try_get("contact_phone").unwrap_or(None),
                contact_email: row.try_get("contact_email").unwrap_or(None),
                latitude: row.try_get("latitude").unwrap_or(None),
                longitude: row.try_get("longitude").unwrap_or(None),
                description: row.try_get("description").unwrap_or(None),
                created_at: row.get("created_at"),
                updated_at: row.try_get("updated_at").unwrap_or(None),
            })
            .collect();

        Ok((places, total))
    }

    async fn create_place(&self, place: PlaceCreate) -> AppResult<Place> {
        let row = sqlx::query(
            r#"INSERT INTO location_places 
            (place_name, address, contact_person, contact_phone, contact_email, 
             latitude, longitude, description, created_at, updated_at) 
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $9)
            RETURNING *"#,
        )
        .bind(&place.place_name)
        .bind(&place.address)
        .bind(&place.contact_person)
        .bind(&place.contact_phone)
        .bind(&place.contact_email)
        .bind(place.latitude)
        .bind(place.longitude)
        .bind(&place.description)
        .bind(chrono::Utc::now().naive_utc())
        .fetch_one(self.pool.as_ref())
        .await?;

        Ok(Place {
            place_id: row.get("place_id"),
            place_name: row.get("place_name"),
            address: row.get("address"),
            contact_person: row.try_get("contact_person").unwrap_or(None),
            contact_phone: row.try_get("contact_phone").unwrap_or(None),
            contact_email: row.try_get("contact_email").unwrap_or(None),
            latitude: row.try_get("latitude").unwrap_or(None),
            longitude: row.try_get("longitude").unwrap_or(None),
            description: row.try_get("description").unwrap_or(None),
            created_at: row.get("created_at"),
            updated_at: row.try_get("updated_at").unwrap_or(None),
        })
    }

    async fn update_place(&self, place_id: i32, place: PlaceUpdate) -> AppResult<Option<Place>> {
        let row = sqlx::query(
            r#"UPDATE location_places 
            SET place_name = COALESCE($1, place_name),
                address = COALESCE($2, address),
                contact_person = COALESCE($3, contact_person),
                contact_phone = COALESCE($4, contact_phone),
                contact_email = COALESCE($5, contact_email),
                latitude = COALESCE($6, latitude),
                longitude = COALESCE($7, longitude),
                description = COALESCE($8, description),
                updated_at = $9
            WHERE place_id = $10
            RETURNING *"#,
        )
        .bind(&place.place_name)
        .bind(&place.address)
        .bind(&place.contact_person)
        .bind(&place.contact_phone)
        .bind(&place.contact_email)
        .bind(place.latitude)
        .bind(place.longitude)
        .bind(&place.description)
        .bind(chrono::Utc::now().naive_utc())
        .bind(place_id)
        .fetch_optional(self.pool.as_ref())
        .await?;

        match row {
            Some(row) => {
                let place = Place {
                    place_id: row.get("place_id"),
                    place_name: row.get("place_name"),
                    address: row.get("address"),
                    contact_person: row.try_get("contact_person").unwrap_or(None),
                    contact_phone: row.try_get("contact_phone").unwrap_or(None),
                    contact_email: row.try_get("contact_email").unwrap_or(None),
                    latitude: row.try_get("latitude").unwrap_or(None),
                    longitude: row.try_get("longitude").unwrap_or(None),
                    description: row.try_get("description").unwrap_or(None),
                    created_at: row.get("created_at"),
                    updated_at: row.try_get("updated_at").unwrap_or(None),
                };
                Ok(Some(place))
            }
            None => Ok(None),
        }
    }

    async fn delete_place(&self, place_id: i32) -> AppResult<bool> {
        let result = sqlx::query("DELETE FROM location_places WHERE place_id = $1")
            .bind(place_id)
            .execute(self.pool.as_ref())
            .await?;
        Ok(result.rows_affected() > 0)
    }

    // ==================== 路线相关 ====================
    async fn get_routes(&self, page: i32, page_size: i32) -> AppResult<(Vec<Route>, i64)> {
        let offset = (page - 1) * page_size;

        let count_query = "SELECT COUNT(*) FROM location_routes";
        let total: i64 = sqlx::query_scalar(count_query)
            .fetch_one(self.pool.as_ref())
            .await?;

        let list_query = r#"SELECT * FROM location_routes ORDER BY route_id LIMIT $1 OFFSET $2"#;

        let routes: Vec<Route> = sqlx::query(list_query)
            .bind(page_size)
            .bind(offset)
            .fetch_all(self.pool.as_ref())
            .await?
            .into_iter()
            .map(|row| Route {
                route_id: row.get("route_id"),
                route_name: row.get("route_name"),
                start_point: row.get("start_point"),
                start_latitude: row.try_get("start_latitude").unwrap_or(None),
                start_longitude: row.try_get("start_longitude").unwrap_or(None),
                end_point: row.get("end_point"),
                end_latitude: row.try_get("end_latitude").unwrap_or(None),
                end_longitude: row.try_get("end_longitude").unwrap_or(None),
                waypoints: row.try_get("waypoints").unwrap_or(None),
                distance: row.try_get("distance").unwrap_or(None),
                estimated_duration: row.try_get("estimated_duration").unwrap_or(None),
                description: row.try_get("description").unwrap_or(None),
                created_at: row.get("created_at"),
                updated_at: row.try_get("updated_at").unwrap_or(None),
            })
            .collect();

        Ok((routes, total))
    }

    async fn create_route(&self, route: RouteCreate) -> AppResult<Route> {
        let row = sqlx::query(
            r#"INSERT INTO location_routes 
            (route_name, start_point, start_latitude, start_longitude, 
             end_point, end_latitude, end_longitude, waypoints, distance, 
             estimated_duration, description, created_at, updated_at) 
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $12)
            RETURNING *"#,
        )
        .bind(&route.route_name)
        .bind(&route.start_point)
        .bind(route.start_latitude)
        .bind(route.start_longitude)
        .bind(&route.end_point)
        .bind(route.end_latitude)
        .bind(route.end_longitude)
        .bind(&route.waypoints)
        .bind(route.distance)
        .bind(route.estimated_duration)
        .bind(&route.description)
        .bind(chrono::Utc::now().naive_utc())
        .fetch_one(self.pool.as_ref())
        .await?;

        Ok(Route {
            route_id: row.get("route_id"),
            route_name: row.get("route_name"),
            start_point: row.get("start_point"),
            start_latitude: row.try_get("start_latitude").unwrap_or(None),
            start_longitude: row.try_get("start_longitude").unwrap_or(None),
            end_point: row.get("end_point"),
            end_latitude: row.try_get("end_latitude").unwrap_or(None),
            end_longitude: row.try_get("end_longitude").unwrap_or(None),
            waypoints: row.try_get("waypoints").unwrap_or(None),
            distance: row.try_get("distance").unwrap_or(None),
            estimated_duration: row.try_get("estimated_duration").unwrap_or(None),
            description: row.try_get("description").unwrap_or(None),
            created_at: row.get("created_at"),
            updated_at: row.try_get("updated_at").unwrap_or(None),
        })
    }

    async fn update_route(&self, route_id: i32, route: RouteUpdate) -> AppResult<Option<Route>> {
        let row = sqlx::query(
            r#"UPDATE location_routes 
            SET route_name = COALESCE($1, route_name),
                start_point = COALESCE($2, start_point),
                start_latitude = COALESCE($3, start_latitude),
                start_longitude = COALESCE($4, start_longitude),
                end_point = COALESCE($5, end_point),
                end_latitude = COALESCE($6, end_latitude),
                end_longitude = COALESCE($7, end_longitude),
                waypoints = COALESCE($8, waypoints),
                distance = COALESCE($9, distance),
                estimated_duration = COALESCE($10, estimated_duration),
                description = COALESCE($11, description),
                updated_at = $12
            WHERE route_id = $13
            RETURNING *"#,
        )
        .bind(&route.route_name)
        .bind(&route.start_point)
        .bind(route.start_latitude)
        .bind(route.start_longitude)
        .bind(&route.end_point)
        .bind(route.end_latitude)
        .bind(route.end_longitude)
        .bind(&route.waypoints)
        .bind(route.distance)
        .bind(route.estimated_duration)
        .bind(&route.description)
        .bind(chrono::Utc::now().naive_utc())
        .bind(route_id)
        .fetch_optional(self.pool.as_ref())
        .await?;

        match row {
            Some(row) => {
                let route = Route {
                    route_id: row.get("route_id"),
                    route_name: row.get("route_name"),
                    start_point: row.get("start_point"),
                    start_latitude: row.try_get("start_latitude").unwrap_or(None),
                    start_longitude: row.try_get("start_longitude").unwrap_or(None),
                    end_point: row.get("end_point"),
                    end_latitude: row.try_get("end_latitude").unwrap_or(None),
                    end_longitude: row.try_get("end_longitude").unwrap_or(None),
                    waypoints: row.try_get("waypoints").unwrap_or(None),
                    distance: row.try_get("distance").unwrap_or(None),
                    estimated_duration: row.try_get("estimated_duration").unwrap_or(None),
                    description: row.try_get("description").unwrap_or(None),
                    created_at: row.get("created_at"),
                    updated_at: row.try_get("updated_at").unwrap_or(None),
                };
                Ok(Some(route))
            }
            None => Ok(None),
        }
    }

    async fn delete_route(&self, route_id: i32) -> AppResult<bool> {
        let result = sqlx::query("DELETE FROM location_routes WHERE route_id = $1")
            .bind(route_id)
            .execute(self.pool.as_ref())
            .await?;
        Ok(result.rows_affected() > 0)
    }
}
