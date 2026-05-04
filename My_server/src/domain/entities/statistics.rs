//! 统计领域实体

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// 统计项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticsItem {
    pub label: String,
    pub value: f64,
    pub count: i64,
    pub timestamp: Option<DateTime<Utc>>,
    pub details: Option<Value>,
}

/// 车辆统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VehicleStatistics {
    pub total_vehicles: i64,
    pub active_vehicles: i64,
    pub inactive_vehicles: i64,
    pub vehicles_by_type: Vec<StatisticsItem>,
    pub vehicles_by_group: Vec<StatisticsItem>,
    pub vehicles_with_devices: i64,
}

/// 设备统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceStatistics {
    pub total_devices: i64,
    pub online_devices: i64,
    pub offline_devices: i64,
    pub devices_by_type: Vec<StatisticsItem>,
    pub devices_by_manufacturer: Vec<StatisticsItem>,
    pub devices_with_vehicles: i64,
}

/// 称重统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeighingStatistics {
    pub total_weight: f64,
    pub total_records: i64,
    pub average_weight: f64,
    pub daily_weighing: Vec<StatisticsItem>,
    pub vehicles_by_weight: Vec<StatisticsItem>,
    pub devices_by_weight: Vec<StatisticsItem>,
}

/// 安全排行项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyRankingItem {
    pub id: i32,
    pub name: String,
    pub department: String,
    pub score: i32,
}

/// 自定义统计响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomStatistics {
    pub total: f64,
    pub count: i64,
    pub average: f64,
    pub min: f64,
    pub max: f64,
    pub data: Vec<StatisticsItem>,
}

/// 统计查询参数
#[derive(Debug, Clone, Deserialize)]
pub struct StatisticsQuery {
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
}

/// 统计响应（聚合所有统计）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticsResponse {
    pub vehicle: VehicleStatistics,
    pub device: DeviceStatistics,
    pub weighing: WeighingStatistics,
    pub safety_ranking: Vec<SafetyRankingItem>,
}

impl StatisticsItem {
    pub fn new(label: String, value: f64, count: i64) -> Self {
        Self {
            label,
            value,
            count,
            timestamp: None,
            details: None,
        }
    }

    pub fn with_timestamp(mut self, timestamp: DateTime<Utc>) -> Self {
        self.timestamp = Some(timestamp);
        self
    }

    pub fn with_details(mut self, details: Value) -> Self {
        self.details = Some(details);
        self
    }
}

impl Default for VehicleStatistics {
    fn default() -> Self {
        Self::new()
    }
}

impl VehicleStatistics {
    pub fn new() -> Self {
        Self {
            total_vehicles: 0,
            active_vehicles: 0,
            inactive_vehicles: 0,
            vehicles_by_type: Vec::new(),
            vehicles_by_group: Vec::new(),
            vehicles_with_devices: 0,
        }
    }
}

impl Default for DeviceStatistics {
    fn default() -> Self {
        Self::new()
    }
}

impl DeviceStatistics {
    pub fn new() -> Self {
        Self {
            total_devices: 0,
            online_devices: 0,
            offline_devices: 0,
            devices_by_type: Vec::new(),
            devices_by_manufacturer: Vec::new(),
            devices_with_vehicles: 0,
        }
    }
}

impl Default for WeighingStatistics {
    fn default() -> Self {
        Self::new()
    }
}

impl WeighingStatistics {
    pub fn new() -> Self {
        Self {
            total_weight: 0.0,
            total_records: 0,
            average_weight: 0.0,
            daily_weighing: Vec::new(),
            vehicles_by_weight: Vec::new(),
            devices_by_weight: Vec::new(),
        }
    }
}
