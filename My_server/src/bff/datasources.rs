//! / BFF数据源管理器

use crate::bff::models::*;
use anyhow::Result;
use sqlx::PgPool;
use std::sync::Arc;

/// 数据源管理器
pub struct DataSourceManager {
    pub postgres: Arc<PgPool>,
    pub redis: Option<redis::aio::MultiplexedConnection>,
    pub legacy_tcp_enabled: bool,
}

impl DataSourceManager {
    pub fn new(
        postgres: Arc<PgPool>,
        redis: Option<redis::aio::MultiplexedConnection>,
        legacy_tcp_enabled: bool,
    ) -> Self {
        Self {
            postgres,
            redis,
            legacy_tcp_enabled,
        }
    }

    /// 从数据库查询车辆基础信息
    pub async fn get_vehicle(&self, id: i32) -> Result<VehicleBaseInfo> {
        let vehicle = sqlx::query_as::<_, VehicleBaseInfo>(
            r#"
            SELECT 
                v.vehicle_id,
                v.vehicle_name,
                v.license_plate,
                v.vehicle_type,
                v.vehicle_color,
                v.device_id,
                v.terminal_type,
                v.group_id,
                vg.group_name,
                v.status
            FROM vehicles v
            LEFT JOIN vehicle_groups vg ON v.group_id = vg.group_id
            WHERE v.vehicle_id = $1
            "#,
        )
        .bind(id)
        .fetch_one(&*self.postgres)
        .await?;

        Ok(vehicle)
    }

    /// 查询车辆列表(分页)
    pub async fn get_vehicles(
        &self,
        query: &VehicleRealtimeQuery,
    ) -> Result<(Vec<VehicleBaseInfo>, u64)> {
        let page = query.page;
        let size = query.size;
        let offset = (page - 1) * size;

        let mut conditions = vec!["v.status = 1".to_string()];

        if let Some(_group_id) = query.group_id {
            conditions.push(format!("v.group_id = {}", _group_id));
        }

        if let Some(_status) = query.status {
            conditions.push(format!("v.operation_status = {}", _status));
        }

        if let Some(_vehicle_type) = &query.vehicle_type {
            conditions.push(format!("v.vehicle_type = '{}'", _vehicle_type));
        }

        if let Some(_license_plate) = &query.license_plate {
            conditions.push(format!("v.license_plate LIKE '%{}%'", _license_plate));
        }

        let where_clause = conditions.join(" AND ");

        // 查询总数
        let count_sql = format!(
            r#"
            SELECT COUNT(*) as count
            FROM vehicles v
            LEFT JOIN vehicle_groups vg ON v.group_id = vg.group_id
            WHERE {}
            "#,
            where_clause
        );

        let total: i64 = sqlx::query_scalar(&count_sql)
            .fetch_one(&*self.postgres)
            .await?;

        // 查询车辆列表
        let sql = format!(
            r#"
            SELECT 
                v.vehicle_id,
                v.vehicle_name,
                v.license_plate,
                v.vehicle_type,
                v.vehicle_color,
                v.device_id,
                v.terminal_type,
                v.group_id,
                vg.group_name,
                v.status
            FROM vehicles v
            LEFT JOIN vehicle_groups vg ON v.group_id = vg.group_id
            WHERE {}
            ORDER BY v.vehicle_id
            LIMIT {} OFFSET {}
            "#,
            where_clause, size, offset
        );

        let vehicles = sqlx::query_as::<_, VehicleBaseInfo>(&sql)
            .fetch_all(&*self.postgres)
            .await?;

        Ok((vehicles, total as u64))
    }

    /// 批量查询车辆基础信息
    pub async fn batch_get_vehicles(&self, ids: Vec<i32>) -> Result<Vec<VehicleBaseInfo>> {
        if ids.is_empty() {
            return Ok(vec![]);
        }

        let id_list = ids
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<_>>()
            .join(", ");

        let sql = format!(
            r#"
            SELECT 
                v.vehicle_id,
                v.vehicle_name,
                v.license_plate,
                v.vehicle_type,
                v.vehicle_color,
                v.device_id,
                v.terminal_type,
                v.group_id,
                vg.group_name,
                v.status
            FROM vehicles v
            LEFT JOIN vehicle_groups vg ON v.group_id = vg.group_id
            WHERE v.vehicle_id IN ({})
            ORDER BY v.vehicle_id
            "#,
            id_list
        );

        let vehicles = sqlx::query_as::<_, VehicleBaseInfo>(&sql)
            .fetch_all(&*self.postgres)
            .await?;

        Ok(vehicles)
    }

    /// 从Redis获取车辆实时状态
    ///
    /// 优先从 Redis 缓存读取,缓存未命中时返回 None,
    /// 由上层调用方回退到数据库查询。
    pub async fn get_vehicle_realtime_from_cache(
        &self,
        vehicle_id: i32,
    ) -> Result<Option<VehicleRealtimeStatus>> {
        let key = format!("vehicle:realtime:{}", vehicle_id);
        match crate::redis::get_cache::<VehicleRealtimeStatus>(&key).await {
            Ok(result) => Ok(result),
            Err(e) => {
                log::warn!(
                    "Redis get_vehicle_realtime_from_cache failed (key={}): {}",
                    key,
                    e
                );
                Ok(None) // 缓存不可用时降级,不影响主流程
            }
        }
    }

    /// 保存车辆实时状态到Redis缓存
    ///
    /// 写入失败时仅记录警告,不影响主流程(缓存降级策略)。
    pub async fn set_vehicle_realtime_to_cache(
        &self,
        vehicle_id: i32,
        status: &VehicleRealtimeStatus,
        ttl_seconds: u64,
    ) -> Result<()> {
        let key = format!("vehicle:realtime:{}", vehicle_id);
        if let Err(e) = crate::redis::set_cache(&key, status, ttl_seconds).await {
            log::warn!(
                "Redis set_vehicle_realtime_to_cache failed (key={}): {}",
                key,
                e
            );
        }
        Ok(())
    }
}
