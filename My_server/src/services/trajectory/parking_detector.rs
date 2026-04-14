//! / 停车检测器
// 检测车辆停车记录和停车未熄火情况

use log::{debug, info};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Duration};

use crate::infrastructure::cache::TrajectoryPoint;

/// 停车记录
#[derive(Debug, Clone, Serialize)]
pub struct ParkingRecord {
    pub device_id: String,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub duration: i64,          // 停车时长(秒)
    pub latitude: f64,
    pub longitude: f64,
    pub address: Option<String>,
    pub engine_off: bool,        // 是否熄火
    pub parking_duration: Option<i32>, // 停车时长(秒)
}

/// 停车检测器
pub struct ParkingDetector {
    speed_threshold: f32,        // 停车速度阈值(km/h)
    min_parking_duration: i64,   // 最小停车时长(秒)
}

impl ParkingDetector {
    /// 创建新的停车检测器
    pub fn new() -> Self {
        info!("Creating parking detector");
        
        Self {
            speed_threshold: 2.0,  // 2km/h 以下视为停车
            min_parking_duration: 60, // 至少 1 分钟才算停车
        }
    }

    /// 检测停车记录
    pub fn detect_parking(&self, points: &[TrajectoryPoint]) -> Vec<ParkingRecord> {
        debug!("Detecting parking records from {} points", points.len());

        let mut parking_records = Vec::new();
        let mut parking_start: Option<(usize, DateTime<Utc>)> = None;
        let mut engine_off_start: Option<DateTime<Utc>> = None;

        for (i, point) in points.iter().enumerate() {
            // 检查是否停止
            let is_stopped = point.speed.map_or(false, |s| s < self.speed_threshold);

            if is_stopped {
                // 检查熄火状态(通过 speed=0 和 direction=0 判断)
                let is_engine_off = point.speed.map_or(false, |s| s == 0.0)
                    && point.direction.map_or(false, |d| d == 0);

                if parking_start.is_none() {
                    // 开始停车
                    parking_start = Some((i, point.timestamp));
                    
                    if is_engine_off {
                        engine_off_start = Some(point.timestamp);
                    }
                } else {
                    // 更新熄火时间
                    if is_engine_off && engine_off_start.is_none() {
                        engine_off_start = Some(point.timestamp);
                    }
                }
            } else {
                // 车辆移动中
                if let Some((start_idx, start_time)) = parking_start {
                    let duration = point.timestamp.signed_duration_since(start_time).num_seconds();

                    if duration >= self.min_parking_duration {
                        // 检测到有效停车
                        let parking_record = ParkingRecord {
                            device_id: point.device_id.clone(),
                            start_time,
                            end_time: point.timestamp,
                            duration,
                            latitude: points[start_idx].latitude,
                            longitude: points[start_idx].longitude,
                            address: points[start_idx].address.clone(),
                            engine_off: engine_off_start.is_some(),
                            parking_duration: Some(duration as i32),
                        };

                        debug!("Detected parking: {} at ({:.6}, {:.6}), duration: {}s",
                                point.device_id,
                                parking_record.latitude,
                                parking_record.longitude,
                                duration);

                        parking_records.push(parking_record);
                    }

                    parking_start = None;
                    engine_off_start = None;
                }
            }
        }

        // 处理最后一个停车记录
        if let Some((start_idx, start_time)) = parking_start {
            let last_point = points.last().expect("points should not be empty");
            let duration = last_point.timestamp.signed_duration_since(start_time).num_seconds();

            if duration >= self.min_parking_duration {
                let parking_record = ParkingRecord {
                    device_id: last_point.device_id.clone(),
                    start_time,
                    end_time: last_point.timestamp,
                    duration,
                    latitude: points[start_idx].latitude,
                    longitude: points[start_idx].longitude,
                    address: points[start_idx].address.clone(),
                    engine_off: engine_off_start.is_some(),
                    parking_duration: Some(duration as i32),
                };

                parking_records.push(parking_record);
            }
        }

        info!("Detected {} parking records", parking_records.len());
        parking_records
    }

    /// 检测停车未熄火记录
    pub fn detect_no_shutdown(&self, points: &[TrajectoryPoint]) -> Vec<ParkingRecord> {
        let all_parking = self.detect_parking(points);
        
        all_parking
            .into_iter()
            .filter(|r| !r.engine_off)
            .collect()
    }
}

impl Default for ParkingDetector {
    fn default() -> Self {
        Self::new()
    }
}






