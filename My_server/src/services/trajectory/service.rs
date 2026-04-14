//! / 轨迹查询服务
// 支持多种轨迹查询和回放模式

use actix::prelude::*;
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use chrono::{DateTime, Utc, Timelike};

use super::parking_detector::ParkingDetector;
use super::trajectory_simplifier::TrajectorySimplifier;
use super::exporter::TrajectoryExporter;
use crate::infrastructure::geocoding::TencentGeocoder;
use crate::infrastructure::cache::TrajectoryPoint;

/// 轨迹类型
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TrajectoryType {
    /// 运动轨迹:显示实际行驶路径
    Motion,
    /// 点轨迹:仅显示关键位置点
    Points,
    /// 全部轨迹:显示所有采集点
    Full,
}

/// 轨迹查询参数
#[derive(Debug, Clone, Deserialize)]
pub struct TrajectoryQueryParams {
    pub device_id: String,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    #[serde(default)]
    pub trajectory_type: TrajectoryType,
    #[serde(default)]
    pub include_address: bool,
    #[serde(default)]
    pub simplify: bool,
    #[serde(default)]
    pub simplification_tolerance: Option<f64>, // 简化容差(度)
}

/// 轨迹查询结果
#[derive(Debug, Clone, Serialize)]
pub struct TrajectoryResult {
    pub device_id: String,
    pub trajectory_type: TrajectoryType,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub total_distance: f64, // 总里程(米)
    pub total_duration: i64,  // 总时长(秒)
    pub points_count: usize,
    pub points: Vec<TrajectoryPoint>,
    pub parking_records: Vec<ParkingRecord>,
}

/// 轨迹服务
pub struct TrajectoryService {
    db_pool: PgPool,
    parking_detector: ParkingDetector,
    simplifier: TrajectorySimplifier,
    geocoder: Option<TencentGeocoder>,
    exporter: TrajectoryExporter,
}

impl TrajectoryService {
    /// 创建新的轨迹服务
    pub fn new(
        db_pool: PgPool,
        tencent_api_key: Option<String>,
    ) -> Self {
        info!("Creating trajectory service");

        let geocoder = tencent_api_key.map(TencentGeocoder::new);

        Self {
            db_pool,
            parking_detector: ParkingDetector::new(),
            simplifier: TrajectorySimplifier::new(),
            geocoder,
            exporter: TrajectoryExporter::new(),
        }
    }

    /// 查询轨迹
    pub async fn query_trajectory(
        &self,
        params: TrajectoryQueryParams,
    ) -> Result<TrajectoryResult, String> {
        info!(
            "Querying trajectory for device {} from {} to {} (type: {:?})",
            params.device_id,
            params.start_time,
            params.end_time,
            params.trajectory_type
        );

        // 从数据库查询轨迹点
        let mut points = self.query_trajectory_points(
            &params.device_id,
            params.start_time,
            params.end_time,
        ).await?;

        if points.is_empty() {
            warn!("No trajectory points found for device {}", params.device_id);
            return Ok(TrajectoryResult {
                device_id: params.device_id,
                trajectory_type: params.trajectory_type,
                start_time: params.start_time,
                end_time: params.end_time,
                total_distance: 0.0,
                total_duration: 0,
                points_count: 0,
                points: Vec::new(),
                parking_records: Vec::new(),
            });
        }

        info!("Found {} trajectory points", points.len());

        // 地理编码(如果需要)
        if params.include_address {
            points = self.geocode_points(points).await;
        }

        // 检测停车记录
        let parking_records = self.parking_detector.detect_parking(&points);

        // 简化轨迹(如果需要)
        if params.simplify {
            let tolerance = params.simplification_tolerance.unwrap_or(0.0001); // ~11米
            points = self.simplifier.simplify_douglas_peucker(points, tolerance);
            debug!("Simplified trajectory: {} -> {} points", points.len(), points.len());
        }

        // 根据轨迹类型过滤
        let filtered_points = self.filter_by_type(points, &params.trajectory_type);

        // 计算统计信息
        let (total_distance, total_duration) = self.calculate_stats(&filtered_points);

        Ok(TrajectoryResult {
            device_id: params.device_id,
            trajectory_type: params.trajectory_type,
            start_time: params.start_time,
            end_time: params.end_time,
            total_distance,
            total_duration,
            points_count: filtered_points.len(),
            points: filtered_points,
            parking_records,
        })
    }

    /// 查询停车记录
    pub async fn query_parking_records(
        &self,
        device_id: &str,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Result<Vec<ParkingRecord>, String> {
        info!(
            "Querying parking records for device {} from {} to {}",
            device_id, start_time, end_time
        );

        // 查询轨迹点
        let points = self.query_trajectory_points(device_id, start_time, end_time).await?;

        // 检测停车
        let records = self.parking_detector.detect_parking(&points);

        Ok(records)
    }

    /// 查询停车未熄火记录
    pub async fn query_no_shutdown_records(
        &self,
        device_id: &str,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Result<Vec<ParkingRecord>, String> {
        info!(
            "Querying no-shutdown records for device {} from {} to {}",
            device_id, start_time, end_time
        );

        let records = self.query_parking_records(device_id, start_time, end_time).await?;

        // 过滤出停车未熄火的记录
        let no_shutdown_records: Vec<ParkingRecord> = records
            .into_iter()
            .filter(|r| !r.engine_off)
            .collect();

        Ok(no_shutdown_records)
    }

    /// 导出轨迹数据
    pub async fn export_trajectory(
        &self,
        device_id: &str,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
        format: &str,
    ) -> Result<Vec<u8>, String> {
        info!("Exporting trajectory for device {} in {} format", device_id, format);

        let params = TrajectoryQueryParams {
            device_id: device_id.to_string(),
            start_time,
            end_time,
            trajectory_type: TrajectoryType::Full,
            include_address: true,
            simplify: false,
            simplification_tolerance: None,
        };

        let result = self.query_trajectory(params).await?;

        match format.to_lowercase().as_str() {
            "csv" => self.exporter.export_csv(&result.points),
            "excel" => self.exporter.export_excel(&result.points),
            "kml" => self.exporter.export_kml(&result.points),
            "json" => self.exporter.export_json(&result.points),
            _ => Err(format!("Unsupported export format: {}", format)),
        }
    }

    /// 从数据库查询轨迹点
    async fn query_trajectory_points(
        &self,
        device_id: &str,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Result<Vec<TrajectoryPoint>, String> {
        let query = r#"
            SELECT 
                device_id,
                latitude,
                longitude,
                altitude,
                speed,
                direction,
                timestamp,
                address,
                is_parking,
                parking_duration
            FROM device_trajectory
            WHERE device_id = $1
              AND timestamp >= $2
              AND timestamp <= $3
            ORDER BY timestamp ASC
            LIMIT 10000
        "#;

        sqlx::query_as::<_, (String, f64, f64, Option<f32>, Option<f32>, Option<i32>, DateTime<Utc>, Option<String>, Option<bool>, Option<i32>)>(query)
            .bind(device_id)
            .bind(start_time)
            .bind(end_time)
            .fetch_all(&self.db_pool)
            .await
            .map_err(|e| format!("Database error: {}", e))?
            .into_iter()
            .map(|(device_id, lat, lng, alt, speed, dir, ts, addr, is_parking, park_duration)| {
                TrajectoryPoint {
                    device_id,
                    latitude: lat,
                    longitude: lng,
                    altitude: alt,
                    speed,
                    direction: dir,
                    timestamp: ts,
                    address: addr,
                    is_parking: is_parking.unwrap_or(false),
                    parking_duration: park_duration,
                }
            })
            .collect::<Vec<_>>()
            .into()
    }

    /// 地理编码轨迹点
    async fn geocode_points(&self, mut points: Vec<TrajectoryPoint>) -> Vec<TrajectoryPoint> {
        if let Some(geocoder) = &self.geocoder {
            // 只对没有地址的点进行地理编码
            let points_to_geocode: Vec<_> = points
                .iter()
                .filter(|p| p.address.is_none())
                .map(|p| (p.latitude, p.longitude))
                .collect();

            if !points_to_geocode.is_empty() {
                info!("Geocoding {} points", points_to_geocode.len());

                let results = geocoder.batch_reverse_geocode(points_to_geocode).await;

                // 更新地址信息
                let mut result_idx = 0;
                for point in &mut points {
                    if point.address.is_none() && result_idx < results.len() {
                        if let Ok(geocode) = &results[result_idx] {
                            point.address = Some(geocode.address.clone());
                        }
                        result_idx += 1;
                    }
                }
            }
        }

        points
    }

    /// 根据轨迹类型过滤点
    fn filter_by_type(&self, points: Vec<TrajectoryPoint>, trajectory_type: &TrajectoryType) -> Vec<TrajectoryPoint> {
        match trajectory_type {
            TrajectoryType::Full => points,
            TrajectoryType::Motion => {
                // 运动轨迹:过滤掉停车点
                points.into_iter()
                    .filter(|p| !p.is_parking)
                    .collect()
            }
            TrajectoryType::Points => {
                // 点轨迹:只保留关键点(每小时一个)
                let mut filtered = Vec::new();
                let mut last_hour = None;

                for point in points {
                    let hour = point.timestamp.hour();
                    if last_hour != Some(hour) {
                        filtered.push(point);
                        last_hour = Some(hour);
                    }
                }

                filtered
            }
        }
    }

    /// 计算统计信息
    fn calculate_stats(&self, points: &[TrajectoryPoint]) -> (f64, i64) {
        if points.len() < 2 {
            return (0.0, 0);
        }

        // 计算总距离
        let mut total_distance = 0.0;
        for window in points.windows(2) {
            total_distance += Self::haversine_distance(
                window[0].latitude,
                window[0].longitude,
                window[1].latitude,
                window[1].longitude,
            );
        }

        // 计算总时长
        let total_duration = points.last()
            .expect("points.len() >= 2 checked above")
            .timestamp
            .signed_duration_since(points.first().expect("points.len() >= 2 checked above").timestamp)
            .num_seconds();

        (total_distance, total_duration)
    }

    /// Haversine 公式计算两点间距离(米)
    fn haversine_distance(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
        const EARTH_RADIUS: f64 = 6371000.0; // 地球半径(米)

        let d_lat = (lat2 - lat1).to_radians();
        let d_lon = (lon2 - lon1).to_radians();

        let a = (d_lat / 2.0).sin().powi(2)
            + lat1.to_radians().cos() * lat2.to_radians().cos() * (d_lon / 2.0).sin().powi(2);
        let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

        EARTH_RADIUS * c
    }
}






