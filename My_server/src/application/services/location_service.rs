use std::sync::Arc;

use crate::domain::entities::location::{
    Fence, FenceCreate, FenceQuery, FenceUpdate, Location, LocationCreate, LocationUpdate, Place,
    PlaceCreate, PlaceUpdate, Route, RouteCreate, RouteUpdate,
};
use crate::errors::AppResult;
use crate::infrastructure::repositories::location_repository::LocationRepository;

#[async_trait::async_trait]
pub trait LocationService: Send + Sync {
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

pub struct LocationServiceImpl {
    location_repository: Arc<dyn LocationRepository>,
}

impl LocationServiceImpl {
    pub fn new(location_repository: Arc<dyn LocationRepository>) -> Self {
        Self {
            location_repository,
        }
    }
}

#[async_trait::async_trait]
impl LocationService for LocationServiceImpl {
    // ==================== 电子围栏相关 ====================
    async fn get_fences(&self, query: FenceQuery) -> AppResult<(Vec<Fence>, i64)> {
        self.location_repository.get_fences(query).await
    }

    async fn get_fence_by_id(&self, fence_id: i32) -> AppResult<Option<Fence>> {
        self.location_repository.get_fence_by_id(fence_id).await
    }

    async fn create_fence(&self, fence: FenceCreate) -> AppResult<Fence> {
        self.location_repository.create_fence(fence).await
    }

    async fn update_fence(&self, fence_id: i32, fence: FenceUpdate) -> AppResult<Option<Fence>> {
        self.location_repository.update_fence(fence_id, fence).await
    }

    async fn delete_fence(&self, fence_id: i32) -> AppResult<bool> {
        self.location_repository.delete_fence(fence_id).await
    }

    // ==================== 位置相关 ====================
    async fn get_locations(&self, page: i32, page_size: i32) -> AppResult<(Vec<Location>, i64)> {
        self.location_repository
            .get_locations(page, page_size)
            .await
    }

    async fn create_location(&self, location: LocationCreate) -> AppResult<Location> {
        self.location_repository.create_location(location).await
    }

    async fn update_location(
        &self,
        location_id: i32,
        location: LocationUpdate,
    ) -> AppResult<Option<Location>> {
        self.location_repository
            .update_location(location_id, location)
            .await
    }

    async fn delete_location(&self, location_id: i32) -> AppResult<bool> {
        self.location_repository.delete_location(location_id).await
    }

    // ==================== 地点相关 ====================
    async fn get_places(&self, page: i32, page_size: i32) -> AppResult<(Vec<Place>, i64)> {
        self.location_repository.get_places(page, page_size).await
    }

    async fn create_place(&self, place: PlaceCreate) -> AppResult<Place> {
        self.location_repository.create_place(place).await
    }

    async fn update_place(&self, place_id: i32, place: PlaceUpdate) -> AppResult<Option<Place>> {
        self.location_repository.update_place(place_id, place).await
    }

    async fn delete_place(&self, place_id: i32) -> AppResult<bool> {
        self.location_repository.delete_place(place_id).await
    }

    // ==================== 路线相关 ====================
    async fn get_routes(&self, page: i32, page_size: i32) -> AppResult<(Vec<Route>, i64)> {
        self.location_repository.get_routes(page, page_size).await
    }

    async fn create_route(&self, route: RouteCreate) -> AppResult<Route> {
        self.location_repository.create_route(route).await
    }

    async fn update_route(&self, route_id: i32, route: RouteUpdate) -> AppResult<Option<Route>> {
        self.location_repository.update_route(route_id, route).await
    }

    async fn delete_route(&self, route_id: i32) -> AppResult<bool> {
        self.location_repository.delete_route(route_id).await
    }
}
