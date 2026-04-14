//! / BFF报表服务

use crate::bff::models::*;
use anyhow::Result;
use chrono::Utc;
use sqlx::{PgPool, Row};
use std::sync::Arc;

/// 报表服务
pub struct ReportService {
    pub postgres: Arc<PgPool>,
}

impl ReportService {
    pub fn new(postgres: Arc<PgPool>) -> Self {
        Self { postgres }
    }

    /// 生成车辆运营报表
    pub async fn generate_vehicle_operation_report(
        &self,
        request: &ReportRequest,
    ) -> Result<VehicleOperationReport> {
        log::info!(
            "Generating vehicle operation report: {} - {}",
            request.start_time,
            request.end_time
        );

        // 构建WHERE条件
        let mut conditions = vec!["v.status = 1".to_string()];

        if let Some(ref vehicle_ids) = request.vehicle_ids {
            if !vehicle_ids.is_empty() {
                let id_list = vehicle_ids
                    .iter()
                    .map(|id| id.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                conditions.push(format!("v.vehicle_id IN ({})", id_list));
            }
        }

        if let Some(group_id) = request.group_id {
            conditions.push(format!("v.group_id = {}", group_id));
        }

        let where_clause = conditions.join(" AND ");

        // 查询车辆运营明细
        let sql = format!(
            r#"
            SELECT
                v.vehicle_id,
                v.vehicle_name,
                v.license_plate,
                COALESCE(v.driver_name, '') as driver_name,
                COALESCE(SUM(g.speed * 0.001 * 60), 0) as total_mileage,
                COUNT(*) as track_point_count,
                COALESCE(AVG(g.speed), 0) as average_speed,
                COALESCE(MAX(g.speed), 0) as max_speed,
                COALESCE(SUM(CASE WHEN g.speed > 0 THEN 1 ELSE 0 END), 0) * 60 as total_duration,
                EXTRACT(EPOCH FROM (MAX(g.gps_time) - MIN(g.gps_time))) as online_duration
            FROM vehicles v
            LEFT JOIN gps_track_data g ON v.vehicle_id = g.vehicle_id
                AND g.gps_time >= $1
                AND g.gps_time <= $2
            WHERE {}
            GROUP BY v.vehicle_id, v.vehicle_name, v.license_plate
            ORDER BY total_mileage DESC
            "#,
            where_clause
        );

        let rows = sqlx::query(&sql)
            .bind(request.start_time.naive_utc())
            .bind(request.end_time.naive_utc())
            .fetch_all(&*self.postgres)
            .await?;

        let mut vehicles = Vec::new();
        let mut total_mileage_sum = 0.0;
        let mut total_duration_sum = 0i64;
        let mut total_online_sum = 0i64;
        let mut max_speed_all = 0.0;

        for row in rows {
            let total_mileage: f64 = row.try_get("total_mileage").unwrap_or(0.0);
            let average_speed: f64 = row.try_get("average_speed").unwrap_or(0.0);
            let max_speed: f64 = row.try_get("max_speed").unwrap_or(0.0);
            let total_duration: i64 = row.try_get("total_duration").unwrap_or(0);
            let online_duration: i64 = row.try_get("online_duration").unwrap_or(0);
            let track_point_count: i64 = row.try_get("track_point_count").unwrap_or(0);

            total_mileage_sum += total_mileage;
            total_duration_sum += total_duration;
            total_online_sum += online_duration;
            if max_speed > max_speed_all {
                max_speed_all = max_speed;
            }

            vehicles.push(VehicleOperationSummary {
                vehicle_id: row.try_get("vehicle_id")?,
                vehicle_name: row.try_get("vehicle_name")?,
                license_plate: row.try_get("license_plate")?,
                driver_name: row.try_get("driver_name")?,
                total_mileage,
                total_duration,
                total_fuel_consumption: None, // 需要从传感器表计算
                average_speed,
                max_speed,
                online_duration,
                offline_duration: 0,
                track_point_count,
            });
        }

        // 计算汇总统计
        let vehicle_count = vehicles.len() as i64;
        let summary = OperationSummary {
            total_vehicles: vehicle_count,
            total_mileage: total_mileage_sum,
            total_duration_hours: total_duration_sum as f64 / 3600.0,
            total_fuel_consumption: None,
            average_speed: if vehicle_count > 0 {
                vehicles.iter().map(|v| v.average_speed).sum::<f64>() / vehicle_count as f64
            } else {
                0.0
            },
            max_speed: max_speed_all,
            total_online_hours: total_online_sum as f64 / 3600.0,
        };

        Ok(VehicleOperationReport {
            generated_at: Utc::now(),
            start_time: request.start_time,
            end_time: request.end_time,
            vehicles,
            summary,
        })
    }

    /// 生成称重统计报表
    pub async fn generate_weighing_statistics_report(
        &self,
        request: &ReportRequest,
    ) -> Result<WeighingStatisticsReport> {
        log::info!(
            "Generating weighing statistics report: {} - {}",
            request.start_time,
            request.end_time
        );

        // 构建WHERE条件
        let mut conditions = vec!["1=1".to_string()];

        if let Some(ref vehicle_ids) = request.vehicle_ids {
            if !vehicle_ids.is_empty() {
                let id_list = vehicle_ids
                    .iter()
                    .map(|id| id.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                conditions.push(format!("w.vehicle_id IN ({})", id_list));
            }
        }

        if let Some(group_id) = request.group_id {
            conditions.push(format!("v.group_id = {}", group_id));
        }

        conditions.push(format!(
            "w.weighing_time >= '{}' AND w.weighing_time <= '{}'",
            request.start_time.format("%Y-%m-%d %H:%M:%S"),
            request.end_time.format("%Y-%m-%d %H:%M:%S")
        ));

        let where_clause = conditions.join(" AND ");

        // 查询称重记录
        let sql = format!(
            r#"
            SELECT 
                w.weighing_id,
                w.vehicle_id,
                v.vehicle_name,
                v.license_plate,
                w.gross_weight,
                w.tare_weight,
                w.net_weight,
                w.weighing_time,
                w.location,
                w.material_type
            FROM weighing_data w
            LEFT JOIN vehicles v ON w.vehicle_id = v.vehicle_id
            WHERE {}
            ORDER BY w.weighing_time DESC
            "#,
            where_clause
        );

        let weighings = sqlx::query_as::<_, WeighingRecord>(&sql)
            .fetch_all(&*self.postgres)
            .await?;

        // 计算汇总统计
        let total_weighings = weighings.len() as i64;
        let total_gross_weight: f64 = weighings.iter().map(|w| w.gross_weight).sum();
        let total_tare_weight: f64 = weighings.iter().map(|w| w.tare_weight).sum();
        let total_net_weight: f64 = weighings.iter().map(|w| w.net_weight).sum();
        let max_net_weight = weighings.iter().map(|w| w.net_weight).fold(0.0, f64::max);
        let min_net_weight = weighings
            .iter()
            .map(|w| w.net_weight)
            .fold(f64::MAX, f64::min);
        let min_net_weight = if weighings.is_empty() {
            0.0
        } else {
            min_net_weight
        };

        let summary = WeighingSummary {
            total_weighings,
            total_gross_weight,
            total_tare_weight,
            total_net_weight,
            average_net_weight: if total_weighings > 0 {
                total_net_weight / total_weighings as f64
            } else {
                0.0
            },
            max_net_weight,
            min_net_weight,
        };

        Ok(WeighingStatisticsReport {
            generated_at: Utc::now(),
            start_time: request.start_time,
            end_time: request.end_time,
            weighings,
            summary,
        })
    }

    /// 生成报警分析报表
    pub async fn generate_alarm_analysis_report(
        &self,
        request: &ReportRequest,
    ) -> Result<AlarmAnalysisReport> {
        log::info!(
            "Generating alarm analysis report: {} - {}",
            request.start_time,
            request.end_time
        );

        // 构建WHERE条件
        let mut conditions = vec!["1=1".to_string()];

        if let Some(ref vehicle_ids) = request.vehicle_ids {
            if !vehicle_ids.is_empty() {
                let id_list = vehicle_ids
                    .iter()
                    .map(|id| id.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                conditions.push(format!("a.vehicle_id IN ({})", id_list));
            }
        }

        if let Some(group_id) = request.group_id {
            conditions.push(format!("v.group_id = {}", group_id));
        }

        conditions.push(format!(
            "a.alarm_time >= '{}' AND a.alarm_time <= '{}'",
            request.start_time.format("%Y-%m-%d %H:%M:%S"),
            request.end_time.format("%Y-%m-%d %H:%M:%S")
        ));

        let where_clause = conditions.join(" AND ");

        // 查询报警记录
        let sql = format!(
            r#"
            SELECT 
                a.alarm_id,
                a.vehicle_id,
                v.vehicle_name,
                v.license_plate,
                a.alarm_type,
                a.alarm_level,
                a.alarm_message,
                a.alarm_time,
                a.location,
                COALESCE(a.is_handled, false) as is_handled,
                a.handled_time,
                a.handler
            FROM alarm_records a
            LEFT JOIN vehicles v ON a.vehicle_id = v.vehicle_id
            WHERE {}
            ORDER BY a.alarm_time DESC
            "#,
            where_clause
        );

        let alarms = sqlx::query_as::<_, AlarmRecord>(&sql)
            .fetch_all(&*self.postgres)
            .await?;

        let total_alarms = alarms.len() as i64;
        let handled_alarms = alarms.iter().filter(|a| a.is_handled).count() as i64;
        let unhandled_alarms = total_alarms - handled_alarms;
        let urgent_alarms = alarms.iter().filter(|a| a.alarm_level == 4).count() as i64;
        let high_alarms = alarms.iter().filter(|a| a.alarm_level == 3).count() as i64;

        // 按类型统计
        let mut by_type_map: std::collections::HashMap<String, i64> =
            std::collections::HashMap::new();
        for alarm in &alarms {
            *by_type_map.entry(alarm.alarm_type.clone()).or_insert(0) += 1;
        }

        let by_type: Vec<AlarmTypeStatistics> = by_type_map
            .into_iter()
            .map(|(alarm_type, count)| AlarmTypeStatistics {
                alarm_type,
                count,
                percentage: if total_alarms > 0 {
                    (count as f64 / total_alarms as f64) * 100.0
                } else {
                    0.0
                },
            })
            .collect();

        // 按车辆统计
        let mut by_vehicle_map: std::collections::HashMap<i32, AlarmVehicleStatistics> =
            std::collections::HashMap::new();
        for alarm in &alarms {
            let entry = by_vehicle_map
                .entry(alarm.vehicle_id)
                .or_insert(AlarmVehicleStatistics {
                    vehicle_id: alarm.vehicle_id,
                    vehicle_name: alarm.vehicle_name.clone(),
                    license_plate: alarm.license_plate.clone(),
                    alarm_count: 0,
                    urgent_count: 0,
                    high_count: 0,
                });
            entry.alarm_count += 1;
            if alarm.alarm_level == 4 {
                entry.urgent_count += 1;
            }
            if alarm.alarm_level == 3 {
                entry.high_count += 1;
            }
        }

        let by_vehicle: Vec<AlarmVehicleStatistics> = by_vehicle_map.into_values().collect();

        let summary = AlarmSummary {
            total_alarms,
            handled_alarms,
            unhandled_alarms,
            handling_rate: if total_alarms > 0 {
                (handled_alarms as f64 / total_alarms as f64) * 100.0
            } else {
                100.0
            },
            urgent_alarms,
            high_alarms,
        };

        Ok(AlarmAnalysisReport {
            generated_at: Utc::now(),
            start_time: request.start_time,
            end_time: request.end_time,
            alarms,
            by_type,
            by_vehicle,
            summary,
        })
    }
}
