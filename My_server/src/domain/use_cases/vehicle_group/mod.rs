//! 车组用例模块
//!
//! 已拆分为目录结构

pub mod repository;
pub mod service;
pub mod stats;

// 重新导出
pub use repository::VehicleGroupRepository;
pub use service::VehicleGroupUseCases;
pub use stats::VehicleGroupStatsUseCases;
