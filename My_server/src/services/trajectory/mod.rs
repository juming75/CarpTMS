//! / 轨迹服务模块
// 提供历史轨迹查询和回放功能

pub mod service;
pub mod parking_detector;
pub mod trajectory_simplifier;
pub mod exporter;

pub use service::{TrajectoryService, TrajectoryQueryParams, TrajectoryType};
pub use parking_detector::{ParkingDetector, ParkingRecord};
pub use trajectory_simplifier::{TrajectorySimplifier, SimplificationMethod};
pub use exporter::{TrajectoryExporter, ExportFormat};






