//! 统计应用服务

use chrono::{DateTime, Utc};
use sqlx::Row;

use crate::domain::entities::statistics::{CustomStatistics, DeviceStatistics, SafetyRankingItem, StatisticsItem, VehicleStatistics, WeighingStatistics};
use crate::errors::AppResult;

/// 统计应用服务
pub struct StatisticsService {
    pool: sqlx::PgPool,
}

impl StatisticsService {
    /// 创建新统计服务
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }

    /// 获取车辆统计信息
    pub async fn get_vehicle_statistics(&self) -> AppResult<VehicleStatistics> {
        // 总车辆数
        let total_vehicles: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM vehicles WHERE vehicle_type IS NOT NULL")
            .fetch_one(&self.pool)
            .await
            .unwrap_or(0);

        // 活跃车辆数 (有设备绑定的)
        let active_vehicles: i64 = sqlx::query_scalar(
            "SELECT COUNT(DISTINCT vehicle_id) FROM devices WHERE vehicle_id IS NOT NULL"
        )
        .fetch_one(&self.pool)
        .await
        .unwrap_or(0);

        let inactive_vehicles = total_vehicles - active_vehicles;

        // 按类型统计
        let type_rows = sqlx::query(
            r#"SELECT vehicle_type as label, COUNT(*) as count 
               FROM vehicles 
               WHERE vehicle_type IS NOT NULL 
               GROUP BY vehicle_type"#
        )
        .fetch_all(&self.pool)
        .await
        .unwrap_or_default();

        let vehicles_by_type: Vec<StatisticsItem> = type_rows
            .iter()
            .map(|r| StatisticsItem {
                label: r.try_get("label").unwrap_or_default(),
                value: r.try_get::<i64, _>("count").unwrap_or(0) as f64,
                count: r.try_get::<i64, _>("count").unwrap_or(0),
                timestamp: None,
                details: None,
            })
            .collect();

        // 按车组统计
        let group_rows = sqlx::query(
            r#"SELECT vg.group_name as label, COUNT(v.vehicle_id) as count 
               FROM vehicles v 
               LEFT JOIN vehicle_groups vg ON v.group_id = vg.group_id 
               GROUP BY vg.group_name"#
        )
        .fetch_all(&self.pool)
        .await
        .unwrap_or_default();

        let vehicles_by_group: Vec<StatisticsItem> = group_rows
            .iter()
            .map(|r| StatisticsItem {
                label: r.try_get("label").unwrap_or_else(|_| "未分组".to_string()),
                value: r.try_get::<i64, _>("count").unwrap_or(0) as f64,
                count: r.try_get::<i64, _>("count").unwrap_or(0),
                timestamp: None,
                details: None,
            })
            .collect();

        Ok(VehicleStatistics {
            total_vehicles,
            active_vehicles,
            inactive_vehicles,
            vehicles_by_type,
            vehicles_by_group,
            vehicles_with_devices: active_vehicles,
        })
    }

    /// 获取设备统计信息
    pub async fn get_device_statistics(&self) -> AppResult<DeviceStatistics> {
        // 总设备数
        let total_devices: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM devices")
            .fetch_one(&self.pool)
            .await
            .unwrap_or(0);

        // 在线设备数 (最近1小时有数据的)
        let online_devices: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM devices WHERE last_update > NOW() - INTERVAL '1 hour'"
        )
        .fetch_one(&self.pool)
        .await
        .unwrap_or(0);

        let offline_devices = total_devices - online_devices;

        // 按类型统计
        let type_rows = sqlx::query(
            "SELECT device_type as label, COUNT(*) as count FROM devices GROUP BY device_type"
        )
        .fetch_all(&self.pool)
        .await
        .unwrap_or_default();

        let devices_by_type: Vec<StatisticsItem> = type_rows
            .iter()
            .map(|r| StatisticsItem {
                label: r.try_get("label").unwrap_or_default(),
                value: r.try_get::<i64, _>("count").unwrap_or(0) as f64,
                count: r.try_get::<i64, _>("count").unwrap_or(0),
                timestamp: None,
                details: None,
            })
            .collect();

        // 绑定车辆的设备数
        let devices_with_vehicles: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM devices WHERE vehicle_id IS NOT NULL"
        )
        .fetch_one(&self.pool)
        .await
        .unwrap_or(0);

        Ok(DeviceStatistics {
            total_devices,
            online_devices,
            offline_devices,
            devices_by_type,
            devices_by_manufacturer: vec![], // 简化实现
            devices_with_vehicles,
        })
    }

    /// 获取称重数据统计信息
    pub async fn get_weighing_statistics(
        &self,
        _start_time: Option<DateTime<Utc>>,
        _end_time: Option<DateTime<Utc>>,
    ) -> AppResult<WeighingStatistics> {
        // 总重量
        let total_weight: f64 = sqlx::query_scalar("SELECT COALESCE(SUM(gross_weight - tare_weight), 0) FROM weighing_records")
            .fetch_one(&self.pool)
            .await
            .unwrap_or(0.0);

        // 总记录数
        let total_records: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM weighing_records")
            .fetch_one(&self.pool)
            .await
            .unwrap_or(0);

        let average_weight = if total_records > 0 {
            total_weight / total_records as f64
        } else {
            0.0
        };

        Ok(WeighingStatistics {
            total_weight,
            total_records,
            average_weight,
            daily_weighing: vec![],
            vehicles_by_weight: vec![],
            devices_by_weight: vec![],
        })
    }

    /// 获取安全指数排行
    pub async fn get_safety_ranking(&self, limit: i32) -> AppResult<Vec<SafetyRankingItem>> {
        let rows = sqlx::query(
            r#"SELECT v.vehicle_id as id, v.plate_number as name, 
                      COALESCE(d.department_name, '未分配') as department,
                      100 - COALESCE(a.alert_count, 0) as score
               FROM vehicles v
               LEFT JOIN drivers d ON v.driver_id = d.driver_id
               LEFT JOIN (
                   SELECT vehicle_id, COUNT(*) as alert_count 
                   FROM alerts 
                   WHERE created_at > NOW() - INTERVAL '30 days'
                   GROUP BY vehicle_id
               ) a ON v.vehicle_id = a.vehicle_id
               ORDER BY score DESC
               LIMIT $1"#
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .unwrap_or_default();

        Ok(rows
            .iter()
            .map(|r| SafetyRankingItem {
                id: r.try_get("id").unwrap_or(0),
                name: r.try_get("name").unwrap_or_default(),
                department: r.try_get("department").unwrap_or_default(),
                score: r.try_get("score").unwrap_or(0),
            })
            .collect())
    }

    /// 获取自定义统计信息
    pub async fn get_custom_statistics(
        &self,
        metric: &str,
        _start_time: Option<DateTime<Utc>>,
        _end_time: Option<DateTime<Utc>>,
    ) -> AppResult<CustomStatistics> {
        // 简化实现，根据 metric 返回不同统计
        let (total, count) = match metric {
            "vehicles" => {
                let c: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM vehicles")
                    .fetch_one(&self.pool)
                    .await
                    .unwrap_or(0);
                (c as f64, c)
            }
            "devices" => {
                let c: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM devices")
                    .fetch_one(&self.pool)
                    .await
                    .unwrap_or(0);
                (c as f64, c)
            }
            "orders" => {
                let c: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM orders")
                    .fetch_one(&self.pool)
                    .await
                    .unwrap_or(0);
                (c as f64, c)
            }
            _ => (0.0, 0),
        };

        Ok(CustomStatistics {
            total,
            count,
            average: if count > 0 { total / count as f64 } else { 0.0 },
            min: 0.0,
            max: total,
            data: vec![],
        })
    }
}

use crate::application::ApplicationService;

#[async_trait::async_trait]
impl ApplicationService for StatisticsService {
    fn name(&self) -> &str {
        "StatisticsService"
    }

    async fn initialize(&self) -> AppResult<()> {
        Ok(())
    }
}
