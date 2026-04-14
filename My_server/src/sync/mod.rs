//! / 数据同步模块
// 已激活 - 2026-01-19

pub mod adapter;
pub mod adapter_helpers;
pub mod bsj_adapter;
pub mod cache;
pub mod config;
pub mod db_adapter;
pub mod enhanced_service;
pub mod models;
pub mod service;

pub use adapter::LegacySyncAdapter;
pub use bsj_adapter::BsjAdapter;
pub use cache::SyncCache;
pub use config::{LegacyServerConfig, SyncConfig};
pub use db_adapter::DbAdapter;
pub use enhanced_service::{ConflictResolution, EnhancedDataSyncService, SyncStats};
pub use enhanced_service::SyncStatus as EnhancedSyncStatus;  // 别名以避免与schemas::SyncStatus冲突
pub use models::{LegacyGpsData, LegacyUser, LegacyVehicle};
pub use service::DataSyncService;
