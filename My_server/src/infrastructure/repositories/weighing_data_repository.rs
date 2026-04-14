//! / 称重数据仓库PostgreSQL实现

use anyhow::Result;
use sqlx::{PgPool, Row};
use std::sync::Arc;

use crate::domain::entities::weighing_data::{
    WeighingData, WeighingDataCreate, WeighingDataQuery, WeighingDataUpdate,
};
use crate::domain::use_cases::weighing_data::WeighingDataRepository;

/// 称重数据仓库PostgreSQL实现
pub struct PgWeighingDataRepository {
    pool: Arc<PgPool>,
}

impl PgWeighingDataRepository {
    /// 创建称重数据仓库实例
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl WeighingDataRepository for PgWeighingDataRepository {
    /// 获取称重数据列表
    async fn get_weighing_data_list(
        &self,
        query: WeighingDataQuery,
    ) -> Result<(Vec<WeighingData>, i64), anyhow::Error> {
        // 处理分页参数
        let page = query.page.unwrap_or(1);
        let page_size = query.page_size.unwrap_or(20);
        let offset = (page - 1) * page_size;

        // 构建动态查询条件
        let mut where_clauses = Vec::new();
        let mut params = Vec::new();
        let mut param_index = 1;

        if let Some(vehicle_id) = query.vehicle_id {
            where_clauses.push(format!("vehicle_id = ${}", param_index));
            params.push(vehicle_id.to_string());
            param_index += 1;
        }

        if let Some(device_id) = &query.device_id {
            if !device_id.is_empty() {
                where_clauses.push(format!("device_id = ${}", param_index));
                params.push(device_id.clone());
                param_index += 1;
            }
        }

        if let (Some(start_time), Some(end_time)) = (query.start_time, query.end_time) {
            where_clauses.push(format!(
                "weighing_time BETWEEN ${} AND ${}",
                param_index,
                param_index + 1
            ));
            params.push(start_time.to_string());
            params.push(end_time.to_string());
            param_index += 2;
        } else if let Some(start_time) = query.start_time {
            where_clauses.push(format!("weighing_time >= ${}", param_index));
            params.push(start_time.to_string());
            param_index += 1;
        } else if let Some(end_time) = query.end_time {
            where_clauses.push(format!("weighing_time <= ${}", param_index));
            params.push(end_time.to_string());
            param_index += 1;
        }

        if let Some(status) = query.status {
            where_clauses.push(format!("status = ${}", param_index));
            params.push(status.to_string());
            param_index += 1;
        }

        if let Some(min_net_weight) = query.min_net_weight {
            where_clauses.push(format!("net_weight >= ${}", param_index));
            params.push(min_net_weight.to_string());
            param_index += 1;
        }

        if let Some(max_net_weight) = query.max_net_weight {
            where_clauses.push(format!("net_weight <= ${}", param_index));
            params.push(max_net_weight.to_string());
            param_index += 1;
        }

        // 构建完整查询
        let where_sql = if where_clauses.is_empty() {
            "".to_string()
        } else {
            format!("WHERE {}", where_clauses.join(" AND "))
        };

        // 查询总记录数
        let count_query = format!("SELECT COUNT(*) FROM weighing_data {}", where_sql);
        let mut count_sqlx_query = sqlx::query_scalar::<_, i64>(&count_query);

        // 绑定参数
        for param in &params {
            count_sqlx_query = count_sqlx_query.bind(param);
        }

        let total_count = count_sqlx_query.fetch_one(&*self.pool).await?;

        // 查询分页数据（LEFT JOIN vehicles 表获取车牌号）
        let data_query = format!(
            "SELECT w.id, w.vehicle_id, w.device_id, COALESCE(v.license_plate, '') as vehicle_name,
                    w.weighing_time, w.gross_weight, w.tare_weight, w.net_weight,
                    w.axle_count, w.speed, w.lane_no, w.site_id, w.status,
                    w.create_time, w.update_time
             FROM weighing_data w
             LEFT JOIN vehicles v ON w.vehicle_id = v.vehicle_id
             {}
             ORDER BY w.weighing_time DESC
             LIMIT ${} OFFSET ${}",
            where_sql,
            param_index,
            param_index + 1
        );

        // 使用query方法执行动态查询
        let mut sqlx_query = sqlx::query(&data_query);

        // 绑定参数
        for param in params {
            sqlx_query = sqlx_query.bind(param);
        }
        sqlx_query = sqlx_query.bind(page_size).bind(offset);

        let weighing_data = sqlx_query
            .fetch_all(&*self.pool)
            .await?
            .into_iter()
            .map(|row| WeighingData {
                id: row.try_get("id").unwrap_or(0),
                vehicle_id: row.try_get("vehicle_id").unwrap_or(0),
                device_id: row.try_get("device_id").unwrap_or_default(),
                vehicle_name: row.try_get("vehicle_name").unwrap_or_default(),
                weighing_time: row
                    .try_get("weighing_time")
                    .unwrap_or_else(|_| chrono::Utc::now().naive_utc()),
                gross_weight: row.try_get("gross_weight").unwrap_or(0.0),
                tare_weight: row.try_get("tare_weight").ok(),
                net_weight: row.try_get("net_weight").unwrap_or(0.0),
                axle_count: row.try_get("axle_count").ok(),
                speed: row.try_get("speed").ok(),
                lane_no: row.try_get("lane_no").ok(),
                site_id: row.try_get("site_id").ok(),
                status: row.try_get("status").unwrap_or(1),
                create_time: row
                    .try_get("create_time")
                    .unwrap_or_else(|_| chrono::Utc::now().naive_utc()),
                update_time: row.try_get("update_time").ok(),
            })
            .collect::<Vec<WeighingData>>();

        Ok((weighing_data, total_count))
    }

    /// 获取单个称重数据
    async fn get_weighing_data(&self, id: i32) -> Result<Option<WeighingData>, anyhow::Error> {
        let weighing_data = sqlx::query(
            r#"SELECT w.id, w.vehicle_id, w.device_id, COALESCE(v.license_plate, '') as vehicle_name,
                      w.weighing_time, w.gross_weight, w.tare_weight, w.net_weight,
                      w.axle_count, w.speed, w.lane_no, w.site_id, w.status,
                      w.create_time, w.update_time
               FROM weighing_data w
               LEFT JOIN vehicles v ON w.vehicle_id = v.vehicle_id
               WHERE w.id = $1"#,
        )
            .bind(id)
            .fetch_optional(&*self.pool)
            .await?
            .map(|row| WeighingData {
                id: row.try_get("id").unwrap_or(0),
                vehicle_id: row.try_get("vehicle_id").unwrap_or(0),
                device_id: row.try_get("device_id").unwrap_or_default(),
                vehicle_name: row.try_get("vehicle_name").unwrap_or_default(),
                weighing_time: row
                    .try_get("weighing_time")
                    .unwrap_or_else(|_| chrono::Utc::now().naive_utc()),
                gross_weight: row.try_get("gross_weight").unwrap_or(0.0),
                tare_weight: row.try_get("tare_weight").ok(),
                net_weight: row.try_get("net_weight").unwrap_or(0.0),
                axle_count: row.try_get("axle_count").ok(),
                speed: row.try_get("speed").ok(),
                lane_no: row.try_get("lane_no").ok(),
                site_id: row.try_get("site_id").ok(),
                status: row.try_get("status").unwrap_or(1),
                create_time: row
                    .try_get("create_time")
                    .unwrap_or_else(|_| chrono::Utc::now().naive_utc()),
                update_time: row.try_get("update_time").ok(),
            });

        Ok(weighing_data)
    }

    /// 创建称重数据
    async fn create_weighing_data(
        &self,
        weighing_data: WeighingDataCreate,
    ) -> Result<WeighingData, anyhow::Error> {
        let result = sqlx::query(
            r#"INSERT INTO weighing_data (
                vehicle_id, device_id, weighing_time, gross_weight,
                tare_weight, net_weight, axle_count, speed,
                lane_no, site_id, status
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11
            ) RETURNING *"#,
        )
        .bind(weighing_data.vehicle_id)
        .bind(&weighing_data.device_id)
        .bind(weighing_data.weighing_time)
        .bind(weighing_data.gross_weight)
        .bind(weighing_data.tare_weight)
        .bind(weighing_data.net_weight)
        .bind(weighing_data.axle_count)
        .bind(weighing_data.speed)
        .bind(weighing_data.lane_no)
        .bind(weighing_data.site_id)
        .bind(weighing_data.status)
        .fetch_one(&*self.pool)
        .await;

        let wd = match result {
            Ok(row) => {
                // 插入成功后，通过 vehicle_id 查询车牌号
                let vehicle_id: i32 = row.try_get("vehicle_id").unwrap_or(0);
                let vehicle_name: String = sqlx::query_scalar(
                    "SELECT COALESCE(license_plate, '') FROM vehicles WHERE vehicle_id = $1",
                )
                .bind(vehicle_id)
                .fetch_optional(&*self.pool)
                .await
                .unwrap_or(Some(String::new()))
                .unwrap_or(String::new());

                WeighingData {
                    id: row.try_get("id").unwrap_or(0),
                    vehicle_id,
                    device_id: row.try_get("device_id").unwrap_or_default(),
                    vehicle_name,
                    weighing_time: row
                        .try_get("weighing_time")
                        .unwrap_or_else(|_| chrono::Utc::now().naive_utc()),
                    gross_weight: row.try_get("gross_weight").unwrap_or(0.0),
                    tare_weight: row.try_get("tare_weight").ok(),
                    net_weight: row.try_get("net_weight").unwrap_or(0.0),
                    axle_count: row.try_get("axle_count").ok(),
                    speed: row.try_get("speed").ok(),
                    lane_no: row.try_get("lane_no").ok(),
                    site_id: row.try_get("site_id").ok(),
                    status: row.try_get("status").unwrap_or(1),
                    create_time: row
                        .try_get("create_time")
                        .unwrap_or_else(|_| chrono::Utc::now().naive_utc()),
                    update_time: row.try_get("update_time").ok(),
                }
            },
            Err(e) => {
                return Err(anyhow::anyhow!("Failed to create weighing data: {:?}", e));
            }
        };

        Ok(wd)
    }

    /// 更新称重数据
    async fn update_weighing_data(
        &self,
        id: i32,
        weighing_data: WeighingDataUpdate,
    ) -> Result<Option<WeighingData>, anyhow::Error> {
        let result = sqlx::query(
            r#"UPDATE weighing_data 
               SET 
                   vehicle_id = COALESCE($1, vehicle_id),
                   device_id = COALESCE($2, device_id),
                   weighing_time = COALESCE($3, weighing_time),
                   gross_weight = COALESCE($4, gross_weight),
                   tare_weight = COALESCE($5, tare_weight),
                   net_weight = COALESCE($6, net_weight),
                   axle_count = COALESCE($7, axle_count),
                   speed = COALESCE($8, speed),
                   lane_no = COALESCE($9, lane_no),
                   site_id = COALESCE($10, site_id),
                   status = COALESCE($11, status),
                   update_time = CURRENT_TIMESTAMP 
               WHERE id = $12 
               RETURNING *"#,
        )
        .bind(weighing_data.vehicle_id)
        .bind(&weighing_data.device_id)
        .bind(weighing_data.weighing_time)
        .bind(weighing_data.gross_weight)
        .bind(weighing_data.tare_weight)
        .bind(weighing_data.net_weight)
        .bind(weighing_data.axle_count)
        .bind(weighing_data.speed)
        .bind(weighing_data.lane_no)
        .bind(weighing_data.site_id)
        .bind(weighing_data.status)
        .bind(id)
        .fetch_optional(&*self.pool)
        .await;

        match result {
            Ok(Some(row)) => {
                let vehicle_id: i32 = row.try_get("vehicle_id").unwrap_or(0);
                let vehicle_name: String = sqlx::query_scalar(
                    "SELECT COALESCE(license_plate, '') FROM vehicles WHERE vehicle_id = $1",
                )
                .bind(vehicle_id)
                .fetch_optional(&*self.pool)
                .await
                .unwrap_or(Some(String::new()))
                .unwrap_or(String::new());

                let weighing_data = WeighingData {
                    id: row.try_get("id").unwrap_or(0),
                    vehicle_id,
                    device_id: row.try_get("device_id").unwrap_or_default(),
                    vehicle_name,
                    weighing_time: row
                        .try_get("weighing_time")
                        .unwrap_or_else(|_| chrono::Utc::now().naive_utc()),
                    gross_weight: row.try_get("gross_weight").unwrap_or(0.0),
                    tare_weight: row.try_get("tare_weight").ok(),
                    net_weight: row.try_get("net_weight").unwrap_or(0.0),
                    axle_count: row.try_get("axle_count").ok(),
                    speed: row.try_get("speed").ok(),
                    lane_no: row.try_get("lane_no").ok(),
                    site_id: row.try_get("site_id").ok(),
                    status: row.try_get("status").unwrap_or(1),
                    create_time: row
                        .try_get("create_time")
                        .unwrap_or_else(|_| chrono::Utc::now().naive_utc()),
                    update_time: row.try_get("update_time").ok(),
                };

                Ok(Some(weighing_data))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(anyhow::anyhow!("Failed to update weighing data: {:?}", e)),
        }
    }

    /// 删除称重数据
    async fn delete_weighing_data(&self, id: i32) -> Result<bool, anyhow::Error> {
        let result = sqlx::query(r#"DELETE FROM weighing_data WHERE id = $1"#)
            .bind(id)
            .execute(&*self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    /// 按车辆获取称重数据统计
    async fn get_weighing_data_stats_by_vehicle(
        &self,
        vehicle_id: i32,
        start_time: chrono::NaiveDateTime,
        end_time: chrono::NaiveDateTime,
    ) -> Result<Vec<WeighingData>, anyhow::Error> {
        let weighing_data = sqlx::query(r#"SELECT * FROM weighing_data WHERE vehicle_id = $1 AND weighing_time BETWEEN $2 AND $3 ORDER BY weighing_time"#)
            .bind(vehicle_id)
            .bind(start_time)
            .bind(end_time)
            .fetch_all(&*self.pool)
            .await?
            .into_iter()
            .map(|row| {
                WeighingData {
                    id: row.try_get("id").unwrap_or(0),
                    vehicle_id: row.try_get("vehicle_id").unwrap_or(0),
                    device_id: row.try_get("device_id").unwrap_or_default(),
                    vehicle_name: "".to_string(),
                    weighing_time: row.try_get("weighing_time").unwrap_or_else(|_| {
                        chrono::Utc::now().naive_utc()
                    }),
                    gross_weight: row.try_get("gross_weight").unwrap_or(0.0),
                    tare_weight: row.try_get("tare_weight").ok(),
                    net_weight: row.try_get("net_weight").unwrap_or(0.0),
                    axle_count: row.try_get("axle_count").ok(),
                    speed: row.try_get("speed").ok(),
                    lane_no: row.try_get("lane_no").ok(),
                    site_id: row.try_get("site_id").ok(),
                    status: row.try_get("status").unwrap_or(1),
                    create_time: row.try_get("create_time").unwrap_or_else(|_| {
                        chrono::Utc::now().naive_utc()
                    }),
                    update_time: row.try_get("update_time").ok(),
                }
            })
            .collect::<Vec<WeighingData>>();

        Ok(weighing_data)
    }

    /// 按设备获取称重数据统计
    async fn get_weighing_data_stats_by_device(
        &self,
        device_id: &str,
        start_time: chrono::NaiveDateTime,
        end_time: chrono::NaiveDateTime,
    ) -> Result<Vec<WeighingData>, anyhow::Error> {
        let weighing_data = sqlx::query(r#"SELECT * FROM weighing_data WHERE device_id = $1 AND weighing_time BETWEEN $2 AND $3 ORDER BY weighing_time"#)
            .bind(device_id)
            .bind(start_time)
            .bind(end_time)
            .fetch_all(&*self.pool)
            .await?
            .into_iter()
            .map(|row| {
                WeighingData {
                    id: row.try_get("id").unwrap_or(0),
                    vehicle_id: row.try_get("vehicle_id").unwrap_or(0),
                    device_id: row.try_get("device_id").unwrap_or_default(),
                    vehicle_name: "".to_string(),
                    weighing_time: row.try_get("weighing_time").unwrap_or_else(|_| {
                        chrono::Utc::now().naive_utc()
                    }),
                    gross_weight: row.try_get("gross_weight").unwrap_or(0.0),
                    tare_weight: row.try_get("tare_weight").ok(),
                    net_weight: row.try_get("net_weight").unwrap_or(0.0),
                    axle_count: row.try_get("axle_count").ok(),
                    speed: row.try_get("speed").ok(),
                    lane_no: row.try_get("lane_no").ok(),
                    site_id: row.try_get("site_id").ok(),
                    status: row.try_get("status").unwrap_or(1),
                    create_time: row.try_get("create_time").unwrap_or_else(|_| {
                        chrono::Utc::now().naive_utc()
                    }),
                    update_time: row.try_get("update_time").ok(),
                }
            })
            .collect::<Vec<WeighingData>>();

        Ok(weighing_data)
    }
}
