//! / 传感器服务
// 处理传感器数据的存储、查询和推送

use actix::prelude::*;
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use chrono::{DateTime, Utc};

use super::detector::AnomalyDetector;
use super::aggregator::SensorAggregator;

/// 传感器类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SensorType {
    /// 温度
    Temperature,
    /// 油耗
    Fuel,
    /// 转速
    RPM,
    /// 速度
    Speed,
    /// 电压
    Voltage,
    /// 其他
    Other(String),
}

/// 传感器数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorData {
    pub id: Option<i64>,
    pub device_id: String,
    pub sensor_type: String,
    pub sensor_value: f64,
    pub unit: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub metadata: Option<serde_json::Value>,
    pub is_anomaly: bool,
}

/// 传感器查询参数
#[derive(Debug, Clone, Deserialize)]
pub struct SensorQueryParams {
    pub device_id: Option<String>,
    pub sensor_type: Option<String>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

/// 传感器统计
#[derive(Debug, Clone, Serialize)]
pub struct SensorStatistics {
    pub sensor_type: String,
    pub device_id: String,
    pub count: i64,
    pub min_value: f64,
    pub max_value: f64,
    pub avg_value: f64,
    pub total_value: f64,
    pub anomaly_count: i64,
}

/// 传感器服务
pub struct SensorService {
    db_pool: PgPool,
    anomaly_detector: AnomalyDetector,
    aggregator: SensorAggregator,
}

impl SensorService {
    pub fn new(db_pool: PgPool) -> Self {
        info!("Creating sensor service");

        Self {
            db_pool,
            anomaly_detector: AnomalyDetector::new(),
            aggregator: SensorAggregator::new(),
        }
    }

    /// 保存传感器数据
    pub async fn save_sensor_data(&self, data: SensorData) -> Result<i64, String> {
        debug!("Saving sensor data for device {}: {} = {}",
                data.device_id, data.sensor_type, data.sensor_value);

        // 检测异常
        let is_anomaly = self.anomaly_detector.detect(&data).await;

        let query = r#"
            INSERT INTO device_sensors (
                device_id, sensor_type, sensor_value, unit, timestamp, metadata, is_anomaly
            ) VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id
        "#;

        let id = sqlx::query_scalar::<_, i64>(query)
            .bind(&data.device_id)
            .bind(&data.sensor_type)
            .bind(data.sensor_value)
            .bind(&data.unit)
            .bind(data.timestamp)
            .bind(&data.metadata)
            .bind(is_anomaly)
            .fetch_one(&self.db_pool)
            .await
            .map_err(|e| format!("Database error: {}", e))?;

        // 如果检测到异常,记录并通知
        if is_anomaly {
            warn!("Anomaly detected for device {}: {} = {}",
                   data.device_id, data.sensor_type, data.sensor_value);
            // TODO: 发送异常通知
        }

        info!("Sensor data saved with ID: {}", id);
        Ok(id)
    }

    /// 批量保存传感器数据
    pub async fn save_sensor_batch(&self, data_batch: Vec<SensorData>) -> Result<usize, String> {
        info!("Saving {} sensor data records", data_batch.len());

        let mut saved_count = 0;

        for data in data_batch {
            if let Ok(_) = self.save_sensor_data(data).await {
                saved_count += 1;
            }
        }

        info!("Saved {} sensor data records", saved_count);
        Ok(saved_count)
    }

    /// 查询传感器数据
    pub async fn query_sensor_data(&self, params: SensorQueryParams) -> Result<Vec<SensorData>, String> {
        debug!("Querying sensor data with params: {:?}", params);

        let mut query = "SELECT id, device_id, sensor_type, sensor_value, unit, timestamp, metadata, is_anomaly FROM device_sensors WHERE 1=1".to_string();
        let mut has_conditions = false;

        if let Some(device_id) = &params.device_id {
            query.push_str(&format!(" AND device_id = '{}'", device_id));
            has_conditions = true;
        }

        if let Some(sensor_type) = &params.sensor_type {
            query.push_str(&format!(" AND sensor_type = '{}'", sensor_type));
            has_conditions = true;
        }

        if let Some(start_time) = &params.start_time {
            query.push_str(&format!(" AND timestamp >= '{}'", start_time.format("%Y-%m-%d %H:%M:%S")));
            has_conditions = true;
        }

        if let Some(end_time) = &params.end_time {
            query.push_str(&format!(" AND timestamp <= '{}'", end_time.format("%Y-%m-%d %H:%M:%S")));
            has_conditions = true;
        }

        query.push_str(" ORDER BY timestamp DESC");

        if let Some(limit) = &params.limit {
            query.push_str(&format!(" LIMIT {}", limit));
        }

        if let Some(offset) = &params.offset {
            query.push_str(&format!(" OFFSET {}", offset));
        }

        let result = sqlx::query_as::<_, (i64, String, String, f64, Option<String>, DateTime<Utc>, Option<serde_json::Value>, bool)>(&query)
            .fetch_all(&self.db_pool)
            .await
            .map_err(|e| format!("Database error: {}", e))?;

        let sensors = result.into_iter()
            .map(|(id, device_id, sensor_type, value, unit, ts, meta, is_anomaly)| {
                SensorData {
                    id: Some(id),
                    device_id,
                    sensor_type,
                    sensor_value: value,
                    unit,
                    timestamp: ts,
                    metadata: meta,
                    is_anomaly,
                }
            })
            .collect();

        info!("Found {} sensor records", sensors.len());
        Ok(sensors)
    }

    /// 获取传感器统计
    pub async fn get_statistics(&self, params: SensorQueryParams) -> Result<Vec<SensorStatistics>, String> {
        debug!("Querying sensor statistics");

        let query = r#"
            SELECT 
                sensor_type,
                device_id,
                COUNT(*) as count,
                MIN(sensor_value) as min_value,
                MAX(sensor_value) as max_value,
                AVG(sensor_value) as avg_value,
                SUM(sensor_value) as total_value,
                SUM(CASE WHEN is_anomaly = true THEN 1 ELSE 0 END) as anomaly_count
            FROM device_sensors
            WHERE 1=1
        "#.to_string();

        // TODO: 添加条件参数

        let result = sqlx::query_as::<_, (String, String, i64, Option<f64>, Option<f64>, Option<f64>, Option<f64>, Option<i64>)>(&query)
            .fetch_all(&self.db_pool)
            .await
            .map_err(|e| format!("Database error: {}", e))?;

        let stats = result.into_iter()
            .map(|(sensor_type, device_id, count, min_val, max_val, avg_val, total_val, anomaly_cnt)| {
                SensorStatistics {
                    sensor_type,
                    device_id,
                    count,
                    min_value: min_val.unwrap_or(0.0),
                    max_value: max_val.unwrap_or(0.0),
                    avg_value: avg_val.unwrap_or(0.0),
                    total_value: total_val.unwrap_or(0.0),
                    anomaly_count: anomaly_cnt.unwrap_or(0),
                }
            })
            .collect();

        Ok(stats)
    }

    /// 聚合传感器数据
    pub async fn aggregate_sensor_data(
        &self,
        device_id: &str,
        sensor_type: &str,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
        interval: i64, // 聚合间隔(秒)
    ) -> Result<Vec<SensorData>, String> {
        debug!("Aggregating sensor data for device {}: {}, interval={}s",
                device_id, sensor_type, interval);

        let query = r#"
            SELECT 
                device_id,
                sensor_type,
                AVG(sensor_value) as sensor_value,
                unit,
                timestamp
            FROM device_sensors
            WHERE device_id = $1
              AND sensor_type = $2
              AND timestamp >= $3
              AND timestamp <= $4
            GROUP BY device_id, sensor_type, unit, 
                     FLOOR(EXTRACT(EPOCH FROM timestamp) / $5)
            ORDER BY timestamp ASC
        "#;

        let result = sqlx::query_as::<_, (String, String, f64, Option<String>, DateTime<Utc>)>(query)
            .bind(device_id)
            .bind(sensor_type)
            .bind(start_time)
            .bind(end_time)
            .bind(interval)
            .fetch_all(&self.db_pool)
            .await
            .map_err(|e| format!("Database error: {}", e))?;

        let aggregated = result.into_iter()
            .map(|(device_id, sensor_type, value, unit, ts)| {
                SensorData {
                    id: None,
                    device_id,
                    sensor_type,
                    sensor_value: value,
                    unit,
                    timestamp: ts,
                    metadata: None,
                    is_anomaly: false,
                }
            })
            .collect();

        info!("Aggregated {} sensor records", aggregated.len());
        Ok(aggregated)
    }

    /// 获取异常数据
    pub async fn get_anomalies(&self, params: SensorQueryParams) -> Result<Vec<SensorData>, String> {
        debug!("Querying sensor anomalies");

        let mut query = "SELECT id, device_id, sensor_type, sensor_value, unit, timestamp, metadata, is_anomaly FROM device_sensors WHERE is_anomaly = true".to_string();

        if let Some(device_id) = &params.device_id {
            query.push_str(&format!(" AND device_id = '{}'", device_id));
        }

        if let Some(sensor_type) = &params.sensor_type {
            query.push_str(&format!(" AND sensor_type = '{}'", sensor_type));
        }

        query.push_str(" ORDER BY timestamp DESC");

        if let Some(limit) = &params.limit {
            query.push_str(&format!(" LIMIT {}", limit));
        }

        let result = sqlx::query_as::<_, (i64, String, String, f64, Option<String>, DateTime<Utc>, Option<serde_json::Value>, bool)>(&query)
            .fetch_all(&self.db_pool)
            .await
            .map_err(|e| format!("Database error: {}", e))?;

        let anomalies = result.into_iter()
            .map(|(id, device_id, sensor_type, value, unit, ts, meta, is_anomaly)| {
                SensorData {
                    id: Some(id),
                    device_id,
                    sensor_type,
                    sensor_value: value,
                    unit,
                    timestamp: ts,
                    metadata: meta,
                    is_anomaly,
                }
            })
            .collect();

        info!("Found {} sensor anomalies", anomalies.len());
        Ok(anomalies)
    }
}






