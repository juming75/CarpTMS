//! / 传感器数据聚合服务
// 收集、聚合、计算传感器数据

use anyhow::Result;
use chrono::{DateTime, Utc};
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

/// 传感器数据点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorDataPoint {
    pub vehicle_id: i32,
    pub sensor_type: String,
    pub sensor_time: DateTime<Utc>,
    pub value: f64,
    pub unit: Option<String>,
}

/// 聚合数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedSensorData {
    pub vehicle_id: i32,
    pub sensor_type: String,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub count: i32,
    pub min_value: f64,
    pub max_value: f64,
    pub avg_value: f64,
    pub sum_value: f64,
    pub unit: Option<String>,
}

/// 实时聚合缓存
#[derive(Debug)]
struct AggregationCache {
    data_points: Vec<SensorDataPoint>,
    last_aggregation: Option<DateTime<Utc>>,
    aggregation_interval: Duration,
}

impl AggregationCache {
    fn new(interval: Duration) -> Self {
        Self {
            data_points: Vec::new(),
            last_aggregation: None,
            aggregation_interval: interval,
        }
    }

    fn add_data(&mut self, data: SensorDataPoint) {
        self.data_points.push(data);
    }

    fn should_aggregate(&self) -> bool {
        if self.last_aggregation.is_none() {
            return self.data_points.len() >= 100;
        }

        let elapsed = Utc::now().signed_duration_since(self.last_aggregation.expect("last_aggregation is Some due to is_none check above"));
        elapsed.to_std().unwrap_or(Duration::ZERO) >= self.aggregation_interval
            || self.data_points.len() >= 1000
    }
}

/// 传感器数据聚合器
pub struct SensorDataAggregator {
    db: Arc<PgPool>,
    cache: Arc<RwLock<HashMap<i32, AggregationCache>>>, // vehicle_id -> cache
    aggregation_interval: Duration,
}

impl SensorDataAggregator {
    pub fn new(db: Arc<PgPool>, aggregation_interval: Duration) -> Self {
        Self {
            db,
            cache: Arc::new(RwLock::new(HashMap::new())),
            aggregation_interval,
        }
    }

    /// 添加传感器数据点
    pub async fn add_sensor_data(&self, data: SensorDataPoint) -> Result<()> {
        let vehicle_id = data.vehicle_id;

        // 获取或创建缓存
        {
            let mut cache = self.cache.write().await;
            let agg_cache = cache
                .entry(vehicle_id)
                .or_insert_with(|| AggregationCache::new(self.aggregation_interval));

            agg_cache.add_data(data.clone());

            // 检查是否需要聚合
            if agg_cache.should_aggregate() {
                // 执行聚合
                let data_points = std::mem::take(&mut agg_cache.data_points);
                agg_cache.last_aggregation = Some(Utc::now());

                // 异步执行聚合
                let db = self.db.clone();
                tokio::spawn(async move {
                    if let Err(e) = Self::aggregate_and_save(db, &data_points).await {
                        error!("Failed to aggregate sensor data: {:?}", e);
                    }
                });
            }
        }

        // 同时保存原始数据到数据库
        self.save_raw_data(data).await?;

        Ok(())
    }

    /// 批量添加传感器数据
    pub async fn add_sensor_data_batch(&self, data_list: Vec<SensorDataPoint>) -> Result<()> {
        for data in data_list {
            if let Err(e) = self.add_sensor_data(data).await {
                warn!("Failed to add sensor data: {:?}", e);
            }
        }
        Ok(())
    }

    /// 聚合并保存数据
    async fn aggregate_and_save(db: Arc<PgPool>, data_points: &[SensorDataPoint]) -> Result<()> {
        if data_points.is_empty() {
            return Ok(());
        }

        // 按车辆和传感器类型分组
        let mut grouped: HashMap<(i32, String), Vec<&SensorDataPoint>> = HashMap::new();

        for point in data_points {
            let key = (point.vehicle_id, point.sensor_type.clone());
            grouped.entry(key).or_default().push(point);
        }

        // 对每组进行聚合
        for ((vehicle_id, sensor_type), points) in grouped.iter_mut() {
            if let Some(agg) = Self::calculate_aggregation(*vehicle_id, sensor_type, &*points) {
                Self::save_aggregated_data(db.clone(), &agg).await?;
            }
        }

        Ok(())
    }

    /// 计算聚合数据
    fn calculate_aggregation(
        vehicle_id: i32,
        sensor_type: &str,
        points: &[&SensorDataPoint],
    ) -> Option<AggregatedSensorData> {
        if points.is_empty() {
            return None;
        }

        let min_time = points.iter().map(|p| p.sensor_time).min().expect("points is non-empty");
        let max_time = points.iter().map(|p| p.sensor_time).max().expect("points is non-empty");
        let values: Vec<f64> = points.iter().map(|p| p.value).collect();

        let min_value = values.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_value = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let sum_value: f64 = values.iter().sum();
        let avg_value = sum_value / values.len() as f64;

        Some(AggregatedSensorData {
            vehicle_id,
            sensor_type: sensor_type.to_string(),
            start_time: min_time,
            end_time: max_time,
            count: points.len() as i32,
            min_value,
            max_value,
            avg_value,
            sum_value,
            unit: points.first().and_then(|p| p.unit.clone()),
        })
    }

    /// 保存原始数据
    async fn save_raw_data(&self, data: SensorDataPoint) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO sensor_data (vehicle_id, sensor_type, collect_time, sensor_value, unit)
            VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(data.vehicle_id)
        .bind(data.sensor_type)
        .bind(data.sensor_time)
        .bind(data.value)
        .bind(data.unit)
        .execute(&*self.db)
        .await?;

        Ok(())
    }

    /// 保存聚合数据
    async fn save_aggregated_data(db: Arc<PgPool>, agg: &AggregatedSensorData) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO sensor_data_aggregated
            (vehicle_id, sensor_type, start_time, end_time, count, min_value, max_value, avg_value, sum_value, unit)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            ON CONFLICT (vehicle_id, sensor_type, start_time) DO UPDATE SET
                end_time = EXCLUDED.end_time,
                count = EXCLUDED.count,
                min_value = EXCLUDED.min_value,
                max_value = EXCLUDED.max_value,
                avg_value = EXCLUDED.avg_value,
                sum_value = EXCLUDED.sum_value,
                update_time = NOW()
            "#,
        )
        .bind(agg.vehicle_id)
        .bind(agg.sensor_type.clone())
        .bind(agg.start_time)
        .bind(agg.end_time)
        .bind(agg.count)
        .bind(agg.min_value)
        .bind(agg.max_value)
        .bind(agg.avg_value)
        .bind(agg.sum_value)
        .bind(agg.unit.clone())
        .execute(&*db)
        .await?;

        Ok(())
    }

    /// 查询聚合数据
    pub async fn get_aggregated_data(
        &self,
        vehicle_id: i32,
        sensor_type: &str,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Result<Vec<AggregatedSensorData>> {
        type Row = (
            i32,
            String,
            DateTime<Utc>,
            DateTime<Utc>,
            i64,
            f64,
            f64,
            f64,
            f64,
            String,
        );
        let results = sqlx::query_as::<_, Row>(
            r#"
            SELECT
                vehicle_id, sensor_type, start_time, end_time,
                count, min_value, max_value, avg_value, sum_value, unit
            FROM sensor_data_aggregated
            WHERE vehicle_id = $1
                AND sensor_type = $2
                AND start_time >= $3
                AND end_time <= $4
            ORDER BY start_time ASC
            "#,
        )
        .bind(vehicle_id)
        .bind(sensor_type)
        .bind(start_time)
        .bind(end_time)
        .fetch_all(&*self.db)
        .await?;

        let results = results
            .into_iter()
            .map(
                |(
                    vehicle_id,
                    sensor_type,
                    start_time,
                    end_time,
                    count,
                    min_value,
                    max_value,
                    avg_value,
                    sum_value,
                    unit,
                )| {
                    AggregatedSensorData {
                        vehicle_id,
                        sensor_type,
                        start_time,
                        end_time,
                        count: count as i32,
                        min_value,
                        max_value,
                        avg_value,
                        sum_value,
                        unit: if unit.is_empty() { None } else { Some(unit) },
                    }
                },
            )
            .collect();

        Ok(results)
    }

    /// 启动自动聚合任务
    pub async fn start_auto_aggregation(&self) -> Result<()> {
        let cache = self.cache.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));

            loop {
                interval.tick().await;

                // 检查所有缓存的聚合任务
                let mut cache_write = cache.write().await;

                for (vehicle_id, agg_cache) in cache_write.iter_mut() {
                    if agg_cache.should_aggregate() && !agg_cache.data_points.is_empty() {
                        info!("Auto-aggregating data for vehicle {}", vehicle_id);
                        agg_cache.last_aggregation = Some(Utc::now());
                    }
                }
            }
        });

        info!("Auto aggregation task started");
        Ok(())
    }
}

/// Redis 缓存集成(用于快速查询实时数据)
pub struct SensorDataCache {
    redis_client: Arc<redis::aio::MultiplexedConnection>,
    cache_ttl: u64, // 秒
}

impl SensorDataCache {
    pub fn new(redis_client: Arc<redis::aio::MultiplexedConnection>, cache_ttl: u64) -> Self {
        Self {
            redis_client,
            cache_ttl,
        }
    }

    /// 缓存传感器数据
    pub async fn cache_sensor_data(&self, data: &SensorDataPoint) -> Result<()> {
        let key = format!("sensor:{}:{}:latest", data.vehicle_id, data.sensor_type);
        let value = serde_json::to_string(data)?;

        let mut conn = self.redis_client.as_ref().clone();
        redis::cmd("SETEX")
            .arg(&key)
            .arg(self.cache_ttl)
            .arg(&value)
            .query_async::<()>(&mut conn)
            .await?;

        Ok(())
    }

    /// 从缓存获取传感器数据
    pub async fn get_cached_sensor_data(
        &self,
        vehicle_id: i32,
        sensor_type: &str,
    ) -> Result<Option<SensorDataPoint>> {
        let key = format!("sensor:{}:{}:latest", vehicle_id, sensor_type);

        let mut conn = self.redis_client.as_ref().clone();
        let result: Option<String> = redis::cmd("GET").arg(&key).query_async(&mut conn).await?;

        match result {
            Some(value) => {
                let data: SensorDataPoint = serde_json::from_str(&value)?;
                Ok(Some(data))
            }
            None => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aggregation_cache() {
        let mut cache = AggregationCache::new(Duration::from_secs(60));

        for i in 0..10 {
            let data = SensorDataPoint {
                vehicle_id: 1,
                sensor_type: "temperature".to_string(),
                sensor_time: Utc::now(),
                value: 20.0 + i as f64,
                unit: Some("C".to_string()),
            };
            cache.add_data(data);
        }

        assert_eq!(cache.data_points.len(), 10);
    }

    #[test]
    fn test_calculate_aggregation() {
        let points = vec![
            SensorDataPoint {
                vehicle_id: 1,
                sensor_type: "temperature".to_string(),
                sensor_time: Utc::now(),
                value: 20.0,
                unit: Some("C".to_string()),
            },
            SensorDataPoint {
                vehicle_id: 1,
                sensor_type: "temperature".to_string(),
                sensor_time: Utc::now(),
                value: 30.0,
                unit: Some("C".to_string()),
            },
        ];

        let agg = SensorDataAggregator::calculate_aggregation(
            1,
            "temperature",
            &points.iter().collect::<Vec<_>>(),
        )
        .unwrap();

        assert_eq!(agg.min_value, 20.0);
        assert_eq!(agg.max_value, 30.0);
        assert_eq!(agg.avg_value, 25.0);
        assert_eq!(agg.count, 2);
    }
}






