//! 称重查询对象 (CQRS - Read Side)
//!
//! 专门用于高性能统计查询，支持分库分表场景

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

/// 称重统计查询
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WeighingStatsQuery {
    /// 车辆ID
    pub vehicle_id: Option<i32>,
    /// 设备ID
    pub device_id: Option<String>,
    /// 站点ID
    pub site_id: Option<i32>,
    /// 开始时间
    pub start_time: Option<NaiveDateTime>,
    /// 结束时间
    pub end_time: Option<NaiveDateTime>,
    /// 状态
    pub status: Option<i32>,
    /// 最小净重
    pub min_net_weight: Option<f64>,
    /// 最大净重
    pub max_net_weight: Option<f64>,
}

/// 称重统计结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeighingStatsResult {
    /// 总记录数
    pub total_count: i64,
    /// 总净重
    pub total_net_weight: f64,
    /// 平均净重
    pub avg_net_weight: f64,
    /// 最大净重
    pub max_net_weight: f64,
    /// 最小净重
    pub min_net_weight: f64,
    /// 超载次数
    pub overload_count: i64,
    /// 超速次数
    pub overspeed_count: i64,
}

/// 按车辆统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VehicleWeighingStats {
    pub vehicle_id: i32,
    pub vehicle_name: String,
    pub weighing_count: i64,
    pub total_weight: f64,
    pub avg_weight: f64,
}

/// 按设备统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceWeighingStats {
    pub device_id: String,
    pub weighing_count: i64,
    pub total_weight: f64,
    pub avg_weight: f64,
    pub device_status: String,
}

/// 按站点统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteWeighingStats {
    pub site_id: i32,
    pub weighing_count: i64,
    pub total_weight: f64,
    pub avg_weight: f64,
}

/// 趋势分析
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeighingTrend {
    pub date: String,
    pub count: i64,
    pub total_weight: f64,
    pub avg_weight: f64,
}

/// 超载告警记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverloadAlert {
    pub weighing_id: i64,
    pub vehicle_id: i32,
    pub vehicle_name: String,
    pub gross_weight: f64,
    pub max_allowed_weight: f64,
    pub overload_percentage: f64,
    pub weighing_time: NaiveDateTime,
}
