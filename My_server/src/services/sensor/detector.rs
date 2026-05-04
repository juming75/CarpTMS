//! / 传感器异常检测器
// 检测传感器数据的异常情况

use log::{debug, info, warn};
use serde::{Deserialize, Serialize};

use super::service::SensorData;

/// 异常检测方法
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum AnomalyDetectionMethod {
    /// 阈值检测
    Threshold,
    /// 统计检测(3-sigma)
    Statistical,
    /// 趋势检测
    Trend,
}

/// 异常记录
#[derive(Debug, Clone, Serialize)]
pub struct AnomalyRecord {
    pub device_id: String,
    pub sensor_type: String,
    pub sensor_value: f64,
    pub expected_value: f64,
    pub deviation: f64,
    pub detection_method: AnomalyDetectionMethod,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// 传感器异常检测器
pub struct AnomalyDetector {
    /// 各传感器的阈值配置
    thresholds: std::collections::HashMap<String, SensorThreshold>,
    /// 历史数据用于统计检测
    history: std::collections::HashMap<String, Vec<f64>>,
    max_history_size: usize,
}

/// 传感器阈值配置
#[derive(Debug, Clone)]
struct SensorThreshold {
    pub min_value: Option<f64>,
    pub max_value: Option<f64>,
    pub unit: Option<String>,
}

impl AnomalyDetector {
    pub fn new() -> Self {
        info!("Creating anomaly detector");

        let mut thresholds = std::collections::HashMap::new();

        // 温度阈值
        thresholds.insert("temperature".to_string(), SensorThreshold {
            min_value: Some(-40.0),
            max_value: Some(120.0),
            unit: Some("°C".to_string()),
        });

        // 油耗阈值
        thresholds.insert("fuel".to_string(), SensorThreshold {
            min_value: Some(0.0),
            max_value: Some(100.0),
            unit: Some("%".to_string()),
        });

        // 转速阈值
        thresholds.insert("rpm".to_string(), SensorThreshold {
            min_value: Some(0.0),
            max_value: Some(10000.0),
            unit: Some("RPM".to_string()),
        });

        Self {
            thresholds,
            history: std::collections::HashMap::new(),
            max_history_size: 100,
        }
    }

    /// 检测异常
    pub async fn detect(&self, data: &SensorData) -> bool {
        // 阈值检测
        if self.check_threshold(data) {
            return true;
        }

        // 统计检测(如果有足够的历史数据)
        if self.check_statistical(data) {
            return true;
        }

        false
    }

    /// 阈值检测
    fn check_threshold(&self, data: &SensorData) -> bool {
        let sensor_type = data.sensor_type.to_lowercase();

        if let Some(threshold) = self.thresholds.get(&sensor_type) {
            if let Some(min_val) = threshold.min_value {
                if data.sensor_value < min_val {
                    warn!("Threshold anomaly: {} = {} < min {}",
                            data.sensor_type, data.sensor_value, min_val);
                    return true;
                }
            }

            if let Some(max_val) = threshold.max_value {
                if data.sensor_value > max_val {
                    warn!("Threshold anomaly: {} = {} > max {}",
                            data.sensor_type, data.sensor_value, max_val);
                    return true;
                }
            }
        }

        false
    }

    /// 统计检测(3-sigma 规则)
    fn check_statistical(&self, data: &SensorData) -> bool {
        let sensor_type = data.sensor_type.to_lowercase();

        let history = self.history.get(&sensor_type);

        if let Some(historical_values) = history {
            if historical_values.len() >= 10 {
                // 计算均值和标准差
                let mean: f64 = historical_values.iter().sum::<f64>() / historical_values.len() as f64;
                let variance: f64 = historical_values.iter()
                    .map(|v| (v - mean).powi(2))
                    .sum::<f64>() / historical_values.len() as f64;
                let std_dev = variance.sqrt();

                // 检查是否超出 3-sigma 范围
                let deviation = (data.sensor_value - mean).abs();
                if deviation > 3.0 * std_dev {
                    warn!("Statistical anomaly: {} = {}, mean={}, std_dev={}, deviation={}",
                            data.sensor_type, data.sensor_value, mean, std_dev, deviation);
                    return true;
                }
            }
        }

        false
    }

    /// 更新历史数据
    pub fn update_history(&mut self, sensor_type: String, value: f64) {
        let sensor_type = sensor_type.to_lowercase();
        
        self.history
            .entry(sensor_type)
            .or_insert_with(Vec::new)
            .push(value);

        // 限制历史数据大小
        let values = self.history.get_mut(&sensor_type).expect("sensor history should exist");
        if values.len() > self.max_history_size {
            values.drain(0..values.len() - self.max_history_size);
        }
    }
}

impl Default for AnomalyDetector {
    fn default() -> Self {
        Self::new()
    }
}






