//! 车辆用例模块
//!
//! 已拆分为目录结构

pub mod repository;
pub mod service;
pub mod stats;

// 重新导出
pub use repository::VehicleRepository;
pub use service::VehicleUseCases;
pub use stats::VehicleStatsUseCases;
