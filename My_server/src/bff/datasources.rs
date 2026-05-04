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

        // 安全构建查询条件 - 使用参数化查询
        let mut conditions = vec!["v.status = 1".to_string()];
        let mut params: Vec<String> = Vec::new();

        if let Some(group_id) = query.group_id {
            // 安全验证：确保是有效数字
            if group_id > 0 {
                conditions.push(format!("v.group_id = ${}", params.len() + 1));
                params.push(group_id.to_string());
            }
        }

        if let Some(status) = query.status {
            // 安全验证：确保是有效数字
            if status >= 0 {
                conditions.push(format!("v.operation_status = ${}", params.len() + 1));
                params.push(status.to_string());
            }
        }

        if let Some(vehicle_type) = &query.vehicle_type {
            if !vehicle_type.is_empty() {
                conditions.push(format!("v.vehicle_type = ${}", params.len() + 1));
                params.push(vehicle_type.clone());
            }
        }

        if let Some(license_plate) = &query.license_plate {
            if !license_plate.is_empty() {
                // 安全验证：限制 LIKE 模式的长度
                let safe_pattern = license_plate.chars().take(50).collect::<String>();
                conditions.push(format!("v.license_plate LIKE ${}", params.len() + 1));
                params.push(format!("%{}%", safe_pattern));
            }
        }

        let where_clause = conditions.join(" AND ");

        // 构建参数化查询
        let count_sql = format!(
            r#"
            SELECT COUNT(*) as count
            FROM vehicles v
            LEFT JOIN vehicle_groups vg ON v.group_id = vg.group_id
            WHERE {}
            "#,
            where_clause
        );
        let mut count_query = sqlx::query_scalar::<_, i64>(&count_sql);

        // 绑定参数
        for param in &params {
            if let Ok(i) = param.parse::<i32>() {
                count_query = count_query.bind(i);
            } else if let Ok(s) = param.parse::<i16>() {
                count_query = count_query.bind(s);
            } else {
                count_query = count_query.bind(param);
            }
        }

        let total = count_query.fetch_one(&*self.postgres).await?;

        // 构建数据查询
        let data_sql = format!(
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
            LIMIT ${} OFFSET ${}
            "#,
            where_clause,
            params.len() + 1,
            params.len() + 2
        );
        let mut data_query = sqlx::query_as::<_, VehicleBaseInfo>(&data_sql);

        // 绑定参数
        for param in &params {
            if let Ok(i) = param.parse::<i32>() {
                data_query = data_query.bind(i);
            } else if let Ok(s) = param.parse::<i16>() {
                data_query = data_query.bind(s);
            } else {
                data_query = data_query.bind(param);
            }
        }
        data_query = data_query.bind(size as i64).bind(offset as i64);

        let vehicles = data_query.fetch_all(&*self.postgres).await?;

        Ok((vehicles, total as u64))
    }

    /// 批量查询车辆基础信息
    pub async fn batch_get_vehicles(&self, ids: Vec<i32>) -> Result<Vec<VehicleBaseInfo>> {
        if ids.is_empty() {
            return Ok(vec![]);
        }

        // 安全验证：过滤掉无效的 ID
        let valid_ids: Vec<i32> = ids.into_iter().filter(|id| *id > 0).collect();
        if valid_ids.is_empty() {
            return Ok(vec![]);
        }

        // 安全构建 IN 子句
        let placeholders: Vec<String> = valid_ids
            .iter()
            .enumerate()
            .map(|(i, _)| format!("${}", i + 1))
            .collect();
        let in_clause = placeholders.join(", ");

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
            in_clause
        );

        // 使用参数化查询
        let mut query = sqlx::query_as::<_, VehicleBaseInfo>(&sql);
        for id in &valid_ids {
            query = query.bind(id);
        }

        let vehicles = query.fetch_all(&*self.postgres).await?;

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
