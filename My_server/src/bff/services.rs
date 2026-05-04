//! / BFF服务层

use crate::bff::datasources::DataSourceManager;
use crate::bff::models::*;

use anyhow::Result;
use chrono::{DateTime, Utc};
use num_traits::cast::ToPrimitive;
use serde::{Deserialize, Serialize};
use sqlx::{types::BigDecimal, PgPool};
use std::sync::Arc;

/// 车辆数据聚合服务
///
/// 负责从多个数据源(缓存、数据库、外部服务)聚合车辆相关数据,
/// 提供统一的实时状态查询接口。
///
/// # 数据源优先级
/// 1. Redis缓存(30秒TTL) - 快速响应
/// 2. PostgreSQL数据库 - 准确数据
/// 3. 外部服务(如需) - 补充数据
///
/// # 性能特性
/// - 自动缓存查询结果,减少数据库压力
/// - 批量查询支持,避免N+1问题
/// - 异步非阻塞设计
///
/// # 示例
/// ```ignore
/// let aggregator = VehicleAggregator::new(datasource_manager, postgres, redis);
/// let status = aggregator.get_vehicle_realtime_status(123).await?;
/// ```
pub struct VehicleAggregator {
    pub datasource_manager: Arc<DataSourceManager>,
    pub postgres: Arc<PgPool>,
    pub redis: Option<redis::aio::MultiplexedConnection>,
}

impl VehicleAggregator {
    /// 创建新的车辆数据聚合器
    ///
    /// # 参数
    /// - `datasource_manager`: 数据源管理器,处理缓存和数据库查询
    /// - `postgres`: PostgreSQL连接池,用于数据库操作
    /// - `redis`: 可选的Redis连接,用于缓存
    ///
    /// # 返回
    /// 返回初始化好的`VehicleAggregator`实例
    pub fn new(
        datasource_manager: Arc<DataSourceManager>,
        postgres: Arc<PgPool>,
        redis: Option<redis::aio::MultiplexedConnection>,
    ) -> Self {
        Self {
            datasource_manager,
            postgres,
            redis,
        }
    }

    /// 获取单个车辆实时状态
    ///
    /// 从缓存或数据库获取车辆的完整实时状态,包括:
    /// - 基础信息(车辆ID、车牌号等)
    /// - GPS实时位置
    /// - 传感器数据
    /// - 运营状态(在线/离线)
    ///
    /// # 参数
    /// - `vehicle_id`: 车辆ID
    ///
    /// # 返回
    /// 成功返回`VehicleRealtimeStatus`结构体,包含完整的车辆实时状态
    ///
    /// # 错误
    /// - 数据库连接失败
    /// - 车辆不存在
    /// - 数据解析失败
    ///
    /// # 示例
    /// ```ignore
    /// let status = aggregator.get_vehicle_realtime_status(123).await?;
    /// println!("Vehicle {} is at {:?}", status.vehicle.vehicle_id, status.gps);
    /// ```
    pub async fn get_vehicle_realtime_status(
        &self,
        vehicle_id: i32,
    ) -> Result<VehicleRealtimeStatus> {
        // 1. 尝试从Redis缓存获取
        if let Some(cached) = self
            .datasource_manager
            .get_vehicle_realtime_from_cache(vehicle_id)
            .await?
        {
            log::debug!("Vehicle {} realtime status found in cache", vehicle_id);
            return Ok(cached);
        }

        // 2. 从数据库查询车辆基础信息
        let vehicle = self.datasource_manager.get_vehicle(vehicle_id).await?;

        // 3. 查询GPS实时位置(从vehicle_realtime_locations表)
        let gps = self.get_gps_realtime(vehicle_id).await.ok();

        // 4. 查询传感器实时数据(从sensor_data表)
        let sensors = self.get_sensor_realtime(vehicle_id).await.ok();

        // 5. 构建运营状态
        let operation = Some(OperationStatus {
            status: if gps.is_some() { 1 } else { 2 }, // 1-在线, 2-离线
            last_online_time: gps.as_ref().map(|g| g.gps_time),
            total_driving_time: None,
            total_mileage: None,
            current_driver: None,
        });

        // 6. 聚合数据
        let status = VehicleRealtimeStatus {
            vehicle,
            gps,
            sensors,
            operation,
            source: DataSource::LocalDB,
            received_at: Utc::now(),
        };

        // 7. 写入Redis缓存(TTL: 30秒)
        if let Err(e) = self
            .datasource_manager
            .set_vehicle_realtime_to_cache(vehicle_id, &status, 30)
            .await
        {
            log::warn!("Failed to cache vehicle realtime status: {}", e);
        }

        Ok(status)
    }

    /// 批量获取车辆实时状态(优化版 - 使用JOIN查询避免N+1问题)
    pub async fn batch_get_vehicle_realtime_status(
        &self,
        vehicle_ids: Vec<i32>,
    ) -> Result<Vec<VehicleRealtimeStatus>> {
        if vehicle_ids.is_empty() {
            return Ok(vec![]);
        }

        // 尝试从数据库查询
        // 使用普通的query而不是query!宏,避免编译时数据库连接
        let vehicle_realtime_list = match sqlx::query_as::<
            _,
            (
                i32,
                String,
                String,
                String,
                i32,
                i32,
                Option<f64>,
                Option<f64>,
                Option<sqlx::types::BigDecimal>,
                Option<sqlx::types::BigDecimal>,
                Option<i32>,
            ),
        >(
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
                r.direction
            FROM vehicles v
            LEFT JOIN vehicle_realtime_locations r ON v.vehicle_id = r.vehicle_id
            WHERE v.vehicle_id = ANY($1)
            ORDER BY v.vehicle_id
            "#,
        )
        .bind(&vehicle_ids)
        .fetch_all(&*self.postgres)
        .await
        {
            Ok(data) => data,
            Err(e) => {
                log::warn!("Database query failed: {:?}, returning empty result", e);
                // 数据库连接失败,返回空结果
                return Ok(vec![]);
            }
        };

        // 构建结果
        let mut results = Vec::with_capacity(vehicle_ids.len());

        for row in vehicle_realtime_list {
            let (
                vehicle_id,
                vehicle_name,
                license_plate,
                vehicle_type,
                group_id,
                status,
                latitude,
                longitude,
                altitude,
                speed,
                direction,
            ) = row;

            // 构建GPS数据
            let gps = if let (Some(latitude), Some(longitude)) = (latitude, longitude) {
                Some(GpsData {
                    latitude,
                    longitude,
                    altitude: Some(altitude.and_then(|v| v.to_f64()).unwrap_or(0.0)),
                    speed: speed.and_then(|v| v.to_f64()).unwrap_or(0.0),
                    direction: direction.unwrap_or(0) as f64,
                    gps_time: Utc::now(),
                    location_accuracy: None,
                    satellite_count: None,
                })
            } else {
                None
            };

            // 构建车辆基础信息
            let vehicle = VehicleBaseInfo {
                vehicle_id,
                vehicle_name,
                license_plate,
                vehicle_type,
                vehicle_color: "".to_string(),
                device_id: None,
                terminal_type: None,
                group_id,
                group_name: None,
                status: status.try_into().expect("valid status code"),
            };

            // 构建运营状态
            let operation = Some(OperationStatus {
                status: if gps.is_some() { 1 } else { 2 }, // 1-在线, 2-离线
                last_online_time: gps.as_ref().map(|g| g.gps_time),
                total_driving_time: None,
                total_mileage: None,
                current_driver: None,
            });

            // 聚合数据
            let status = VehicleRealtimeStatus {
                vehicle,
                gps,
                sensors: None, // 传感器数据需要单独批量查询
                operation,
                source: DataSource::LocalDB,
                received_at: Utc::now(),
            };

            results.push(status);
        }

        Ok(results)
    }

    /// 获取车辆列表(含实时状态,分页)
    pub async fn get_vehicles_with_realtime_status(
        &self,
        query: VehicleRealtimeQuery,
    ) -> Result<PaginatedResponse<VehicleRealtimeStatus>> {
        // 1. 查询车辆基础信息(分页)
        let (vehicles, total) = self.datasource_manager.get_vehicles(&query).await?;

        if vehicles.is_empty() {
            return Ok(PaginatedResponse::new(
                vec![],
                total,
                query.page,
                query.size,
            ));
        }

        // 2. 提取车辆ID列表
        let vehicle_ids: Vec<i32> = vehicles.iter().map(|v| v.vehicle_id).collect();

        // 3. 批量查询实时状态
        let realtime_statuses = self.batch_get_vehicle_realtime_status(vehicle_ids).await?;

        // 4. 聚合数据
        let mut status_map: std::collections::HashMap<i32, VehicleRealtimeStatus> =
            realtime_statuses
                .into_iter()
                .map(|s| (s.vehicle.vehicle_id, s))
                .collect();

        let items: Vec<VehicleRealtimeStatus> = vehicles
            .into_iter()
            .map(|v| {
                if let Some(status) = status_map.remove(&v.vehicle_id) {
                    status
                } else {
                    // 如果没有实时状态,只返回基础信息
                    VehicleRealtimeStatus {
                        vehicle: v.clone(),
                        gps: None,
                        sensors: None,
                        operation: Some(OperationStatus {
                            status: 2, // 离线
                            last_online_time: None,
                            total_driving_time: None,
                            total_mileage: None,
                            current_driver: None,
                        }),
                        source: DataSource::LocalDB,
                        received_at: Utc::now(),
                    }
                }
            })
            .collect();

        Ok(PaginatedResponse::new(items, total, query.page, query.size))
    }

    /// 查询GPS实时位置
    async fn get_gps_realtime(&self, _vehicle_id: i32) -> Result<GpsData> {
        // TODO: 暂时禁用,需要先检查数据库表结构
        // let row = sqlx::query!(
        //     r#"
        //     SELECT
        //         latitude,
        //         longitude,
        //         altitude,
        //         speed,
        //         direction,
        //         gps_time
        //     FROM vehicle_realtime_locations
        //     WHERE vehicle_id = $1
        //     ORDER BY gps_time DESC
        //     LIMIT 1
        //     "#,
        //     vehicle_id
        // )
        // .fetch_optional(&*self.postgres)
        // .await?;

        // match row {
        //     Some(row) => Ok(GpsData {
        //         latitude: row.latitude,
        //         longitude: row.longitude,
        //         altitude: row.altitude,
        //         speed: row.speed.unwrap_or(0.0),
        //         direction: row.direction.unwrap_or(0.0),
        //         gps_time: row.gps_time.and_utc(),
        //         location_accuracy: None,
        //         satellite_count: None,
        //     }),
        //     None => Err(anyhow::anyhow!("GPS data not found")),
        // }

        // 临时返回空数据
        Err(anyhow::anyhow!("GPS realtime query temporarily disabled"))
    }

    /// 查询传感器实时数据
    pub async fn get_sensor_realtime(&self, vehicle_id: i32) -> Result<SensorData> {
        let rows: Vec<(String, BigDecimal, Option<String>, chrono::NaiveDateTime)> =
            sqlx::query_as::<_, (String, BigDecimal, Option<String>, chrono::NaiveDateTime)>(
                r#"
            SELECT
                sensor_type,
                sensor_value,
                unit,
                collect_time
            FROM sensor_data
            WHERE vehicle_id = $1
            ORDER BY collect_time DESC
            LIMIT 100
            "#,
            )
            .bind(vehicle_id)
            .fetch_all(&*self.postgres)
            .await?;

        if rows.is_empty() {
            return Err(anyhow::anyhow!("Sensor data not found"));
        }

        // 获取最新的collect_time
        let collect_time = DateTime::<Utc>::from_naive_utc_and_offset(rows[0].3, Utc);

        // 转换为SensorReading列表
        let sensors: Vec<SensorReading> = rows
            .into_iter()
            .map(
                |(sensor_type, sensor_value, unit, _collect_time): (
                    String,
                    BigDecimal,
                    Option<String>,
                    chrono::NaiveDateTime,
                )| {
                    // sensor_value 已经是 BigDecimal 类型,需要转换为 f64
                    let value_f64 = sensor_value.to_string().parse::<f64>().unwrap_or(0.0);
                    SensorReading {
                        sensor_type,
                        sensor_value: value_f64,
                        unit,
                    }
                },
            )
            .collect();

        Ok(SensorData {
            sensors,
            collect_time,
        })
    }

    /// 查询GPS历史轨迹
    pub async fn get_gps_track(
        &self,
        _vehicle_id: i32,
        _start_time: DateTime<Utc>,
        _end_time: DateTime<Utc>,
        _max_points: u32,
    ) -> Result<Vec<GpsTrackPoint>> {
        // TODO: 暂时禁用,需要先检查数据库表结构
        // // 1. 查询轨迹点总数
        // let total_points: i64 = sqlx::query_scalar!(
        //     r#"
        //     SELECT COUNT(*) as count
        //     FROM gps_track_data
        //     WHERE vehicle_id = $1
        //       AND gps_time >= $2
        //       AND gps_time <= $3
        //     "#,
        //     vehicle_id,
        //     start_time.naive_utc(),
        //     end_time.naive_utc()
        // )
        // .fetch_one(&*self.postgres)
        // .await?;

        // if total_points == 0 {
        //     return Ok(vec![]);
        // }

        // // 2. 计算采样间隔以避免返回过多数据
        // let sample_interval = if total_points as u32 > max_points {
        //     (total_points as f64 / max_points as f64).ceil() as i64
        // } else {
        //     1
        // };

        // // 3. 查询轨迹数据(带降采样)
        // // 使用ROW_NUMBER()实现降采样
        // let track_points = sqlx::query_as!(
        //     GpsTrackPoint,
        //     r#"
        //     SELECT
        //         id,
        //         vehicle_id,
        //         latitude,
        //         longitude,
        //         altitude,
        //         COALESCE(speed, 0.0) as speed,
        //         COALESCE(direction, 0.0) as direction,
        //         gps_time as "gps_time!",
        //         NULL::NUMERIC as "location_accuracy!",
        //         NULL::INTEGER as "satellite_count!",
        //         location_address as address
        //     FROM (
        //         SELECT *,
        //                ROW_NUMBER() OVER (ORDER BY gps_time) as rn
        //         FROM gps_track_data
        //         WHERE vehicle_id = $1
        //           AND gps_time >= $2
        //           AND gps_time <= $3
        //     ) sub
        //     WHERE rn % $4 = 1
        //     ORDER BY gps_time
        //     "#,
        //     vehicle_id,
        //     start_time.naive_utc(),
        //     end_time.naive_utc(),
        //     sample_interval
        // )
        // .fetch_all(&*self.postgres)
        // .await?;

        // log::info!(
        //     "Retrieved {} GPS track points for vehicle {} (original: {}, sampled: {}, interval: {})",
        //     track_points.len(),
        //     vehicle_id,
        //     total_points,
        //     max_points,
        //     sample_interval
        // );

        // Ok(track_points)

        // 临时返回空数据
        Ok(vec![])
    }

    /// 查询传感器历史数据
    pub async fn get_sensor_history(
        &self,
        vehicle_id: i32,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
        sensor_type: Option<String>,
        _sample_interval_minutes: u32,
    ) -> Result<Vec<SensorData>> {
        // 使用原始查询避免类型不兼容
        let rows = if let Some(stype) = sensor_type.as_ref() {
            sqlx::query_as::<_, (String, BigDecimal, Option<String>, chrono::NaiveDateTime)>(
                r#"
                SELECT
                    sensor_type,
                    sensor_value,
                    unit,
                    collect_time
                FROM sensor_data
                WHERE vehicle_id = $1
                  AND collect_time >= $2
                  AND collect_time <= $3
                  AND sensor_type = $4
                ORDER BY collect_time
                "#,
            )
            .bind(vehicle_id)
            .bind(start_time.naive_utc())
            .bind(end_time.naive_utc())
            .bind(stype)
            .fetch_all(&*self.postgres)
            .await?
        } else {
            sqlx::query_as::<_, (String, BigDecimal, Option<String>, chrono::NaiveDateTime)>(
                r#"
                SELECT
                    sensor_type,
                    sensor_value,
                    unit,
                    collect_time
                FROM sensor_data
                WHERE vehicle_id = $1
                  AND collect_time >= $2
                  AND collect_time <= $3
                ORDER BY collect_time
                "#,
            )
            .bind(vehicle_id)
            .bind(start_time.naive_utc())
            .bind(end_time.naive_utc())
            .fetch_all(&*self.postgres)
            .await?
        };

        // 按时间分组传感器数据
        let mut sensor_map: std::collections::HashMap<DateTime<Utc>, Vec<SensorReading>> =
            std::collections::HashMap::new();
        for (sensor_type, sensor_value, unit, collect_time) in rows {
            let key = DateTime::<Utc>::from_naive_utc_and_offset(collect_time, Utc);
            let value_f64: f64 = sensor_value.to_string().parse::<f64>().unwrap_or(0.0);
            sensor_map.entry(key).or_default().push(SensorReading {
                sensor_type,
                sensor_value: value_f64,
                unit,
            });
        }

        // 转换为SensorData列表
        let sensor_data: Vec<SensorData> = sensor_map
            .into_iter()
            .map(|(collect_time, sensors)| SensorData {
                sensors,
                collect_time,
            })
            .collect();

        log::info!(
            "Retrieved {} sensor data points for vehicle {}",
            sensor_data.len(),
            vehicle_id
        );

        Ok(sensor_data)
    }

    /// 轨迹回放 - 返回轨迹点用于前端播放
    pub async fn get_track_for_playback(
        &self,
        vehicle_id: i32,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
        max_points: u32,
    ) -> Result<Vec<GpsTrackPoint>> {
        // 复用get_gps_track方法
        self.get_gps_track(vehicle_id, start_time, end_time, max_points)
            .await
    }

    /// 聚合传感器数据统计信息
    pub async fn get_sensor_statistics(
        &self,
        vehicle_id: i32,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Result<SensorStatistics> {
        type Row = (
            String,
            Option<i64>,
            Option<BigDecimal>,
            Option<BigDecimal>,
            Option<BigDecimal>,
        );
        let rows = sqlx::query_as::<_, Row>(
            r#"
            SELECT
                sensor_type,
                COUNT(*) as count,
                AVG(sensor_value) as avg_value,
                MIN(sensor_value) as min_value,
                MAX(sensor_value) as max_value
            FROM sensor_data
            WHERE vehicle_id = $1
              AND collect_time >= $2
              AND collect_time <= $3
            GROUP BY sensor_type
            "#,
        )
        .bind(vehicle_id)
        .bind(start_time.naive_utc())
        .bind(end_time.naive_utc())
        .fetch_all(&*self.postgres)
        .await?;

        // 解析统计数据
        let mut total_data_points = 0u64;
        let mut avg_fuel: Option<f64> = None;
        let mut min_fuel: Option<f64> = None;
        let mut max_fuel: Option<f64> = None;
        let mut avg_water_temp: Option<f64> = None;
        let mut min_water_temp: Option<f64> = None;
        let mut max_water_temp: Option<f64> = None;
        let mut avg_engine_rpm: Option<f64> = None;
        let mut min_engine_rpm: Option<i32> = None;
        let mut max_engine_rpm: Option<i32> = None;
        let mut avg_load_weight: Option<f64> = None;
        let mut max_load_weight: Option<f64> = None;

        for (sensor_type, count, avg_value, min_value, max_value) in rows {
            let count: i64 = count.unwrap_or(0);
            total_data_points += count as u64;

            match sensor_type.as_str() {
                "fuel" => {
                    avg_fuel = avg_value.and_then(|v: BigDecimal| v.to_f64());
                    min_fuel = min_value.and_then(|v: BigDecimal| v.to_f64());
                    max_fuel = max_value.and_then(|v: BigDecimal| v.to_f64());
                }
                "water_temp" => {
                    avg_water_temp = avg_value.and_then(|v: BigDecimal| v.to_f64());
                    min_water_temp = min_value.and_then(|v: BigDecimal| v.to_f64());
                    max_water_temp = max_value.and_then(|v: BigDecimal| v.to_f64());
                }
                "engine_rpm" => {
                    avg_engine_rpm = avg_value.and_then(|v: BigDecimal| v.to_f64());
                    min_engine_rpm = min_value
                        .and_then(|v: BigDecimal| v.to_f64())
                        .map(|v| v as i32);
                    max_engine_rpm = max_value
                        .and_then(|v: BigDecimal| v.to_f64())
                        .map(|v| v as i32);
                }
                "load_weight" => {
                    avg_load_weight = avg_value.and_then(|v: BigDecimal| v.to_f64());
                    max_load_weight = max_value.and_then(|v: BigDecimal| v.to_f64());
                }
                _ => {}
            }
        }

        Ok(SensorStatistics {
            vehicle_id,
            start_time,
            end_time,
            data_points: total_data_points,
            avg_fuel,
            min_fuel,
            max_fuel,
            avg_water_temp,
            min_water_temp,
            max_water_temp,
            avg_engine_rpm,
            min_engine_rpm,
            max_engine_rpm,
            avg_load_weight,
            max_load_weight,
        })
    }
}

/// 传感器统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorStatistics {
    pub vehicle_id: i32,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub data_points: u64,
    pub avg_fuel: Option<f64>,
    pub min_fuel: Option<f64>,
    pub max_fuel: Option<f64>,
    pub avg_water_temp: Option<f64>,
    pub min_water_temp: Option<f64>,
    pub max_water_temp: Option<f64>,
    pub avg_engine_rpm: Option<f64>,
    pub min_engine_rpm: Option<i32>,
    pub max_engine_rpm: Option<i32>,
    pub avg_load_weight: Option<f64>,
    pub max_load_weight: Option<f64>,
}
