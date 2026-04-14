//! / 传感器服务模块
// 提供传感器数据的存储、查询、异常检测和推送功能

pub mod service;
pub mod detector;
pub mod aggregator;

pub use service::{SensorService, SensorQueryParams, SensorType, SensorData, SensorStatistics};
pub use detector::{AnomalyDetector, AnomalyDetectionMethod, AnomalyRecord};
pub use aggregator::{SensorAggregator, AggregationMethod, AggregatedSensorData};






