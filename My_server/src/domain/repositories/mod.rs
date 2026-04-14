//! / 领域层仓库接口定义

pub mod sqlx_sync_repository;
pub mod sqlx_vehicle_repository;
pub mod sync;
pub mod vehicle_repository;

pub use sqlx_sync_repository::SqlxSyncRepository;
pub use sqlx_vehicle_repository::SqlxVehicleRepository;
pub use sync::SyncRepository;
pub use vehicle_repository::VehicleRepository;
