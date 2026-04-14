//! / 传感器数据聚合器
// 对传感器数据进行聚合统计

use log::{debug, info};
use serde::{Deserialize, Serialize};

/// 聚合方法
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum AggregationMethod {
    /// 平均值
    Average,
    /// 最大值
    Max,
    /// 最小值
    Min,
    /// 求和
    Sum,
    /// 计数
    Count,
}

/// 聚合后的传感器数据
#[derive(Debug, Clone, Serialize)]
pub struct AggregatedSensorData {
    pub device_id: String,
    pub sensor_type: String,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub end_time: chrono::DateTime<chrono::Utc>,
    pub value: f64,
    pub method: AggregationMethod,
    pub count: i64,
}

/// 传感器聚合器
pub struct SensorAggregator;

impl SensorAggregator {
    pub fn new() -> Self {
        info!("Creating sensor aggregator");
        Self
    }

    /// 聚合数据
    pub fn aggregate(
        &self,
        data: &[crate::services::sensor::SensorData],
        method: AggregationMethod,
    ) -> AggregatedSensorData {
        if data.is_empty() {
            panic!("Cannot aggregate empty data");
        }

        let device_id = &data[0].device_id;
        let sensor_type = &data[0].sensor_type;
        let start_time = data[0].timestamp;
        let end_time = data.last().expect("data should have at least one element").timestamp;

        let value = match method {
            AggregationMethod::Average => {
                data.iter().map(|d| d.sensor_value).sum::<f64>() / data.len() as f64
            }
            AggregationMethod::Max => {
                data.iter().map(|d| d.sensor_value).fold(f64::NEG_INFINITY, f64::max)
            }
            AggregationMethod::Min => {
                data.iter().map(|d| d.sensor_value).fold(f64::INFINITY, f64::min)
            }
            AggregationMethod::Sum => {
                data.iter().map(|d| d.sensor_value).sum()
            }
            AggregationMethod::Count => {
                data.len() as f64
            }
        };

        AggregatedSensorData {
            device_id: device_id.clone(),
            sensor_type: sensor_type.clone(),
            start_time,
            end_time,
            value,
            method,
            count: data.len() as i64,
        }
    }

    /// 按时间间隔聚合
    pub fn aggregate_by_interval(
        &self,
        data: &[crate::services::sensor::SensorData],
        interval_seconds: i64,
        method: AggregationMethod,
    ) -> Vec<AggregatedSensorData> {
        if data.is_empty() {
            return Vec::new();
        }

        let mut aggregated = Vec::new();
        let mut current_window_start = data[0].timestamp;
        let mut current_window: Vec<&crate::services::sensor::SensorData> = Vec::new();

        for record in data {
            let duration = record.timestamp.signed_duration_since(current_window_start).num_seconds();

            if duration >= interval_seconds && !current_window.is_empty() {
                // 聚合当前窗口
                let window_data: Vec<crate::services::sensor::SensorData> = current_window.iter().map(|d| (*d).clone()).collect();
                let aggregated_data = self.aggregate(&window_data, method);
                aggregated.push(aggregated_data);

                // 开始新窗口
                current_window_start = record.timestamp;
                current_window.clear();
            }

            current_window.push(record);
        }

        // 处理最后一个窗口
        if !current_window.is_empty() {
            let window_data: Vec<crate::services::sensor::SensorData> = current_window.iter().map(|d| (*d).clone()).collect();
            let aggregated_data = self.aggregate(&window_data, method);
            aggregated.push(aggregated_data);
        }

        debug!("Aggregated {} records into {} intervals", data.len(), aggregated.len());
        aggregated
    }
}

impl Default for SensorAggregator {
    fn default() -> Self {
        Self::new()
    }
}






