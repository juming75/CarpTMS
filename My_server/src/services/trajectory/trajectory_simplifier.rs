//! / 轨迹简化算法
// 使用 Douglas-Peucker 算法简化轨迹,减少数据量

use log::{debug, info};
use serde::{Deserialize, Serialize};

use crate::infrastructure::cache::TrajectoryPoint;

/// 简化方法
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SimplificationMethod {
    /// Douglas-Peucker 算法
    DouglasPeucker,
    /// 基于距离的简化
    DistanceBased,
    /// 基于时间的简化
    TimeBased,
}

/// 轨迹简化器
pub struct TrajectorySimplifier;

impl TrajectorySimplifier {
    pub fn new() -> Self {
        info!("Creating trajectory simplifier");
        Self
    }

    /// Douglas-Peucker 算法简化轨迹
    pub fn simplify_douglas_peucker(&self, points: Vec<TrajectoryPoint>, tolerance: f64) -> Vec<TrajectoryPoint> {
        if points.len() <= 2 {
            return points;
        }

        debug!("Simplifying trajectory using Douglas-Peucker, tolerance={}", tolerance);

        let mut result_indices = vec![0, points.len() - 1];
        self.douglas_peucker_recursive(&points, tolerance, 0, points.len() - 1, &mut result_indices);

        result_indices.sort();
        result_indices.uniq();

        let simplified: Vec<TrajectoryPoint> = result_indices
            .iter()
            .map(|&i| points[i].clone())
            .collect();

        debug!("Simplified trajectory: {} -> {} points", points.len(), simplified.len());
        simplified
    }

    /// 递归 Douglas-Peucker 算法
    fn douglas_peucker_recursive(
        &self,
        points: &[TrajectoryPoint],
        tolerance: f64,
        first: usize,
        last: usize,
        indices: &mut Vec<usize>,
    ) {
        let mut max_distance = 0.0;
        let mut max_index = first;

        // 找到距离首尾连线最远的点
        for i in (first + 1)..last {
            let distance = self.perpendicular_distance(
                &points[i],
                &points[first],
                &points[last],
            );

            if distance > max_distance {
                max_distance = distance;
                max_index = i;
            }
        }

        // 如果最大距离超过容差,递归处理
        if max_distance > tolerance {
            indices.push(max_index);
            self.douglas_peucker_recursive(points, tolerance, first, max_index, indices);
            self.douglas_peucker_recursive(points, tolerance, max_index, last, indices);
        }
    }

    /// 计算点到直线的垂直距离
    fn perpendicular_distance(&self, point: &TrajectoryPoint, line_start: &TrajectoryPoint, line_end: &TrajectoryPoint) -> f64 {
        let dx = line_end.longitude - line_start.longitude;
        let dy = line_end.latitude - line_start.latitude;

        // 点到直线的垂直距离公式
        let distance = (dy * point.longitude - dx * point.latitude + line_end.longitude * line_start.latitude - line_end.latitude * line_start.longitude).abs()
            / (dy * dy + dx * dx).sqrt();

        distance
    }

    /// 基于距离的简化
    pub fn simplify_by_distance(&self, points: Vec<TrajectoryPoint>, min_distance: f64) -> Vec<TrajectoryPoint> {
        if points.is_empty() {
            return points;
        }

        debug!("Simplifying trajectory by distance, min_distance={}m", min_distance * 111000.0);

        let mut simplified = Vec::new();
        simplified.push(points[0].clone());

        for point in points.iter().skip(1) {
            let last_point = simplified.last().expect("simplified should have at least one point");
            let distance = self.haversine_distance(
                last_point.latitude,
                last_point.longitude,
                point.latitude,
                point.longitude,
            );

            if distance >= min_distance * 111000.0 { // 转换为米
                simplified.push(point.clone());
            }
        }

        debug!("Simplified trajectory: {} -> {} points", points.len(), simplified.len());
        simplified
    }

    /// 基于时间的简化
    pub fn simplify_by_time(&self, points: Vec<TrajectoryPoint>, min_interval: i64) -> Vec<TrajectoryPoint> {
        if points.is_empty() {
            return points;
        }

        debug!("Simplifying trajectory by time, min_interval={}s", min_interval);

        let mut simplified = Vec::new();
        simplified.push(points[0].clone());

        for point in points.iter().skip(1) {
            let last_point = simplified.last().expect("simplified should have at least one point");
            let interval = point.timestamp.signed_duration_since(last_point.timestamp).num_seconds();

            if interval >= min_interval {
                simplified.push(point.clone());
            }
        }

        debug!("Simplified trajectory: {} -> {} points", points.len(), simplified.len());
        simplified
    }

    /// Haversine 公式计算两点间距离(米)
    fn haversine_distance(&self, lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
        const EARTH_RADIUS: f64 = 6371000.0;

        let d_lat = (lat2 - lat1).to_radians();
        let d_lon = (lon2 - lon1).to_radians();

        let a = (d_lat / 2.0).sin().powi(2)
            + lat1.to_radians().cos() * lat2.to_radians().cos() * (d_lon / 2.0).sin().powi(2);
        let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

        EARTH_RADIUS * c
    }
}

impl Default for TrajectorySimplifier {
    fn default() -> Self {
        Self::new()
    }
}






