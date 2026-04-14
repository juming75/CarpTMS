//! /! 带性能监控的BFF服务
//!
//! 展示如何在实际业务代码中集成查询性能监控

use crate::bff::models::*;
use crate::performance::query_monitor::{monitored_query, QueryMonitor};
use anyhow::Result;
use chrono::Utc;
use sqlx::{PgPool, Row};
use std::sync::Arc;

/// 带监控的车辆数据聚合服务
pub struct MonitoredVehicleAggregator {
    pub postgres: Arc<PgPool>,
    /// 查询监控器
    pub monitor: QueryMonitor,
}

impl MonitoredVehicleAggregator {
    pub fn new(postgres: Arc<PgPool>) -> Self {
        Self {
            postgres,
            monitor: QueryMonitor::new(),
        }
    }

    /// 获取车辆实时状态(带性能监控)
    ///
    /// 此方法使用`monitored_query`宏自动记录查询性能:
    /// - 执行时间
    /// - 查询次数
    /// - 错误率
    /// - 超时检测
    #[monitored_query("get_vehicle_realtime_status")]
    pub async fn get_vehicle_realtime_status(
        &self,
        vehicle_id: i32,
    ) -> Result<VehicleRealtimeStatus> {
        // 查询逻辑
        let vehicle = sqlx::query!(
            r#"
            SELECT
                v.vehicle_id,
                v.vehicle_name,
                v.license_plate,
                v.vehicle_type,
                v.group_id,
                v.status,
                r.latitude,
                r.longitude,
                r.altitude,
                r.speed,
                r.direction,
                r.gps_time
            FROM vehicles v
            LEFT JOIN vehicle_realtime_locations r ON v.vehicle_id = r.vehicle_id
            WHERE v.vehicle_id = $1
            "#,
            vehicle_id
        )
        .fetch_one(&*self.postgres)
        .await?;

        // 构建GPS数据
        let gps = if let (Some(lat), Some(lng)) = (vehicle.latitude, vehicle.longitude) {
            Some(GpsData {
                latitude: lat,
                longitude: lng,
                altitude: Some(vehicle.altitude.and_then(|v| v.to_f64()).unwrap_or(0.0)),
                speed: vehicle.speed.and_then(|v| v.to_f64()).unwrap_or(0.0),
                direction: vehicle.direction.unwrap_or(0) as f64,
                gps_time: vehicle.gps_time.unwrap_or_else(|| Utc::now()),
                location_accuracy: None,
                satellite_count: None,
            })
        } else {
            None
        };

        // 构建车辆基础信息
        let vehicle_info = VehicleBaseInfo {
            vehicle_id: vehicle.vehicle_id,
            vehicle_name: vehicle.vehicle_name,
            license_plate: vehicle.license_plate,
            vehicle_type: vehicle.vehicle_type,
            vehicle_color: "".to_string(),
            device_id: None,
            terminal_type: None,
            group_id: vehicle.group_id,
            group_name: None,
            status: vehicle.status,
        };

        // 构建运营状态
        let operation = Some(OperationStatus {
            status: if gps.is_some() { 1 } else { 2 },
            last_online_time: gps.as_ref().map(|g| g.gps_time),
            total_driving_time: None,
            total_mileage: None,
            current_driver: None,
        });

        Ok(VehicleRealtimeStatus {
            vehicle: vehicle_info,
            gps,
            sensors: None,
            operation,
            source: DataSource::LocalDB,
            received_at: Utc::now(),
        })
    }

    /// 批量获取车辆实时状态(带性能监控)
    ///
    /// 使用批量查询避免N+1问题,并自动记录性能指标
    #[monitored_query("batch_get_vehicle_realtime_status")]
    pub async fn batch_get_vehicle_realtime_status(
        &self,
        vehicle_ids: Vec<i32>,
    ) -> Result<Vec<VehicleRealtimeStatus>> {
        if vehicle_ids.is_empty() {
            return Ok(Vec::new());
        }

        // 单次批量查询所有车辆数据
        let vehicle_realtime_list = sqlx::query!(
            r#"
            SELECT
                v.vehicle_id,
                v.vehicle_name,
                v.license_plate,
                v.vehicle_type,
                v.group_id,
                v.status,
                r.latitude,
                r.longitude,
                r.altitude,
                r.speed,
                r.direction,
                r.gps_time
            FROM vehicles v
            LEFT JOIN vehicle_realtime_locations r ON v.vehicle_id = r.vehicle_id
            WHERE v.vehicle_id = ANY($1)
            ORDER BY v.vehicle_id
            "#,
            &vehicle_ids
        )
        .fetch_all(&*self.postgres)
        .await?;

        // 构建结果
        let mut results = Vec::with_capacity(vehicle_ids.len());

        for row in vehicle_realtime_list {
            let gps = if let (Some(lat), Some(lng)) = (row.latitude, row.longitude) {
                Some(GpsData {
                    latitude: lat,
                    longitude: lng,
                    altitude: Some(row.altitude.and_then(|v| v.to_f64()).unwrap_or(0.0)),
                    speed: row.speed.and_then(|v| v.to_f64()).unwrap_or(0.0),
                    direction: row.direction.unwrap_or(0) as f64,
                    gps_time: row.gps_time.unwrap_or_else(|| Utc::now()),
                    location_accuracy: None,
                    satellite_count: None,
                })
            } else {
                None
            };

            let vehicle = VehicleBaseInfo {
                vehicle_id: row.vehicle_id,
                vehicle_name: row.vehicle_name,
                license_plate: row.license_plate,
                vehicle_type: row.vehicle_type,
                vehicle_color: "".to_string(),
                device_id: None,
                terminal_type: None,
                group_id: row.group_id,
                group_name: None,
                status: row.status,
            };

            let operation = Some(OperationStatus {
                status: if gps.is_some() { 1 } else { 2 },
                last_online_time: gps.as_ref().map(|g| g.gps_time),
                total_driving_time: None,
                total_mileage: None,
                current_driver: None,
            });

            let status = VehicleRealtimeStatus {
                vehicle,
                gps,
                sensors: None,
                operation,
                source: DataSource::LocalDB,
                received_at: Utc::now(),
            };

            results.push(status);
        }

        Ok(results)
    }

    /// 获取性能报告
    ///
    /// 返回所有监控的查询的性能统计
    pub fn get_performance_report(&self) -> PerformanceReport {
        let metrics = self.monitor.get_metrics();

        // 计算汇总统计
        let total_queries: u64 = metrics.values().map(|m| m.count).sum();
        let avg_duration_ms: f64 = if total_queries > 0 {
            let total_duration: f64 = metrics
                .values()
                .map(|m| m.avg_duration_ms * m.count as f64)
                .sum();
            total_duration / total_queries as f64
        } else {
            0.0
        };

        let slow_queries: Vec<SlowQueryInfo> = metrics
            .iter()
            .filter(|(_, m)| m.avg_duration_ms > 100.0) // 超过100ms认为是慢查询
            .map(|(name, m)| SlowQueryInfo {
                query_name: name.clone(),
                avg_duration_ms: m.avg_duration_ms,
                max_duration_ms: m.max_duration_ms,
                count: m.count,
                error_count: m.error_count,
                error_rate: if m.count > 0 {
                    m.error_count as f64 / m.count as f64
                } else {
                    0.0
                },
            })
            .collect();

        PerformanceReport {
            total_queries,
            avg_duration_ms,
            slow_queries,
            report_time: Utc::now(),
        }
    }
}

/// 性能报告
#[derive(Debug, Clone, serde::Serialize)]
pub struct PerformanceReport {
    /// 总查询次数
    pub total_queries: u64,
    /// 平均查询时间(毫秒)
    pub avg_duration_ms: f64,
    /// 慢查询列表(>100ms)
    pub slow_queries: Vec<SlowQueryInfo>,
    /// 报告生成时间
    pub report_time: chrono::DateTime<chrono::Utc>,
}

/// 慢查询信息
#[derive(Debug, Clone, serde::Serialize)]
pub struct SlowQueryInfo {
    /// 查询名称
    pub query_name: String,
    /// 平均执行时间(毫秒)
    pub avg_duration_ms: f64,
    /// 最大执行时间(毫秒)
    pub max_duration_ms: f64,
    /// 执行次数
    pub count: u64,
    /// 错误次数
    pub error_count: u64,
    /// 错误率(0-1)
    pub error_rate: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_monitored_query() {
        // 此测试需要数据库连接,实际测试时需要mock
        let pool = match Arc::new(PgPool::connect("postgres://...").await) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Failed to connect: {}", e);
                return;
            }
        };
        let aggregator = MonitoredVehicleAggregator::new(pool);

        // 执行查询
        let _ = aggregator
            .batch_get_vehicle_realtime_status(vec![1, 2, 3])
            .await;

        // 获取性能报告
        let report = aggregator.get_performance_report();
        assert!(report.total_queries > 0);
    }
}







