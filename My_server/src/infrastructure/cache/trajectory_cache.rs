//! / 轨迹缓存模块
// 用于临时存储轨迹数据,支持轨迹回放和导出

use log::{debug, info};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};

/// 轨迹点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrajectoryPoint {
    pub device_id: String,
    pub latitude: f64,
    pub longitude: f64,
    pub altitude: Option<f32>,
    pub speed: Option<f32>,
    pub direction: Option<i32>,
    pub timestamp: DateTime<Utc>,
    pub address: Option<String>,
    pub is_parking: bool,
    pub parking_duration: Option<i32>, // 停车时长(秒)
}

/// 轨迹缓存
pub struct TrajectoryCache {
    cache: RwLock<HashMap<String, Vec<TrajectoryPoint>>>,
    max_points_per_device: usize,
    cache_ttl_seconds: i64,
}

impl TrajectoryCache {
    pub fn new(max_points: usize, ttl_seconds: i64) -> Self {
        info!("Creating trajectory cache: max_points={}, ttl={}s", max_points, ttl_seconds);
        
        Self {
            cache: RwLock::new(HashMap::new()),
            max_points_per_device: max_points,
            cache_ttl_seconds: ttl_seconds,
        }
    }

    /// 添加轨迹点
    pub async fn add_point(&self, point: TrajectoryPoint) {
        let mut cache = self.cache.write().await;
        let points = cache.entry(point.device_id.clone()).or_insert_with(Vec::new);

        points.push(point);

        // 检查是否超过最大点数
        if points.len() > self.max_points_per_device {
            points.drain(0..points.len() - self.max_points_per_device);
            debug!("Trajectory cache trimmed for device: {} points removed",
                    self.max_points_per_device);
        }

        debug!("Added trajectory point for device {}, total: {} points",
                point.device_id, points.len());
    }

    /// 获取设备的轨迹点
    pub async fn get_points(&self, device_id: &str) -> Vec<TrajectoryPoint> {
        let cache = self.cache.read().await;
        cache.get(device_id).cloned().unwrap_or_default()
    }

    /// 获取时间范围内的轨迹点
    pub async fn get_points_in_range(
        &self,
        device_id: &str,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Vec<TrajectoryPoint> {
        let cache = self.cache.read().await;
        
        if let Some(points) = cache.get(device_id) {
            points.iter()
                .filter(|p| p.timestamp >= start_time && p.timestamp <= end_time)
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }

    /// 清除设备的轨迹缓存
    pub async fn clear(&self, device_id: &str) {
        let mut cache = self.cache.write().await;
        cache.remove(device_id);
        info!("Trajectory cache cleared for device {}", device_id);
    }

    /// 清除所有轨迹缓存
    pub async fn clear_all(&self) {
        let mut cache = self.cache.write().await;
        let count = cache.len();
        cache.clear();
        info!("All trajectory cache cleared: {} devices", count);
    }

    /// 清理过期轨迹点
    pub async fn cleanup_expired(&self) {
        let now = Utc::now();
        let mut cache = self.cache.write().await;

        for (device_id, points) in cache.iter_mut() {
            let original_len = points.len();
            points.retain(|p| {
                now.signed_duration_since(p.timestamp).num_seconds() < self.cache_ttl_seconds
            });
            
            if points.len() != original_len {
                debug!("Cleaned {} expired trajectory points for device {}",
                        original_len - points.len(), device_id);
            }
        }
    }

    /// 获取缓存统计
    pub async fn get_stats(&self) -> TrajectoryCacheStats {
        let cache = self.cache.read().await;
        let total_points: usize = cache.values().map(|v| v.len()).sum();
        
        TrajectoryCacheStats {
            device_count: cache.len(),
            total_points,
            max_points_per_device: self.max_points_per_device,
            average_points_per_device: if cache.is_empty() { 0.0 } else { total_points as f64 / cache.len() as f64 },
        }
    }
}

impl Clone for TrajectoryCache {
    fn clone(&self) -> Self {
        // 注意:这不会克隆内部的 RwLock
        Self {
            cache: RwLock::new(HashMap::new()),
            max_points_per_device: self.max_points_per_device,
            cache_ttl_seconds: self.cache_ttl_seconds,
        }
    }
}

/// 轨迹缓存统计
#[derive(Debug, Clone, Serialize)]
pub struct TrajectoryCacheStats {
    pub device_count: usize,
    pub total_points: usize,
    pub max_points_per_device: usize,
    pub average_points_per_device: f64,
}






