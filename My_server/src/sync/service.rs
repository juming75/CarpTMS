//! / 数据同步服务
// 已激活 - 2026-01-19

use super::adapter::LegacySyncAdapter;
use super::config::*;
use super::models::*;
use anyhow::Result;
use chrono::Utc;
use log::{debug, error, info, warn};
use sqlx::PgPool;
use std::sync::Arc;
use tokio::time::{sleep, Duration};

/// 数据同步服务
pub struct DataSyncService {
    pub adapter: Arc<tokio::sync::Mutex<LegacySyncAdapter>>,
    pub db: Arc<PgPool>,
    config: SyncConfig,
}

impl DataSyncService {
    /// 创建新的同步服务实例
    pub fn new(adapter: LegacySyncAdapter, db: Arc<PgPool>, config: SyncConfig) -> Self {
        Self {
            adapter: Arc::new(tokio::sync::Mutex::new(adapter)),
            db,
            config,
        }
    }

    /// 启动全量同步
    pub async fn start_full_sync(&self) -> Result<()> {
        if !self.config.enabled {
            info!("Data sync is disabled in config");
            return Ok(());
        }

        info!("Starting full data synchronization...");

        // 同步车辆数据
        if let Err(e) = self.sync_vehicles().await {
            warn!("Failed to sync vehicles: {}", e);
        }

        // 同步用户数据
        if let Err(e) = self.sync_users().await {
            warn!("Failed to sync users: {}", e);
        }

        info!("Full data synchronization completed");
        Ok(())
    }

    /// 启动增量同步
    pub async fn start_incremental_sync(&self) -> Result<()> {
        if !self.config.enabled {
            info!("Data sync is disabled in config");
            return Ok(());
        }

        info!("Starting incremental data synchronization...");

        let interval = Duration::from_secs(self.config.sync_interval_seconds);
        // 使用Arc::clone明确表示共享所有权意图
        let db = Arc::clone(&self.db);
        let adapter = Arc::clone(&self.adapter);

        tokio::spawn(async move {
            loop {
                if let Err(e) = Self::sync_incremental_step(&adapter, &db).await {
                    error!("Incremental sync error: {}", e);
                }

                sleep(interval).await;
            }
        });

        info!("Incremental sync task started");
        Ok(())
    }

    /// 启动实时数据接收
    pub async fn start_realtime_stream(&self) -> Result<()> {
        if !self.config.realtime_sync_enabled {
            info!("Realtime sync is disabled in config");
            return Ok(());
        }

        info!("Starting realtime data stream...");

        let db = Arc::clone(&self.db);
        let adapter = Arc::clone(&self.adapter);

        tokio::spawn(async move {
            loop {
                // 如果连接断开,重新连接
                {
                    let mut adap = adapter.lock().await;
                    if !adap.is_connected() {
                        info!("Reconnecting to legacy server...");
                        if let Err(e) = adap.connect().await {
                            error!("Failed to reconnect: {}", e);
                            tokio::time::sleep(Duration::from_secs(5)).await;
                            continue;
                        }
                    }
                }

                // 启动GPS数据流
                let db_for_callback = Arc::clone(&db);
                let callback = Box::new(move |gps_data: LegacyGpsData| {
                    let db_clone = Arc::clone(&db_for_callback);
                    tokio::spawn(async move {
                        if let Err(e) = DataSyncService::save_gps_data(&db_clone, &gps_data).await {
                            error!("Failed to save GPS data: {}", e);
                        }
                    });
                });

                let mut adap = adapter.lock().await;
                if let Err(e) = adap.start_gps_stream(callback).await {
                    error!("GPS stream error: {}", e);
                    adap.disconnect().await;
                    tokio::time::sleep(Duration::from_secs(10)).await;
                }
            }
        });

        info!("Realtime data stream started");
        Ok(())
    }

    /// 同步车辆数据
    async fn sync_vehicles(&self) -> Result<()> {
        let mut adapter = self.adapter.lock().await;
        let legacy_vehicles = adapter.fetch_vehicles().await?;

        let mut success_count = 0;
        let mut fail_count = 0;

        for legacy_vehicle in legacy_vehicles {
            info!("Syncing vehicle: {}", legacy_vehicle.VehicleID);
            match save_vehicle(&self.db, &legacy_vehicle).await {
                Ok(_) => success_count += 1,
                Err(e) => {
                    error!("Failed to save vehicle {}: {}", legacy_vehicle.VehicleID, e);
                    fail_count += 1;
                }
            }
        }

        info!(
            "Vehicle sync completed: {} success, {} failed",
            success_count, fail_count
        );
        Ok(())
    }

    /// 同步用户数据
    async fn sync_users(&self) -> Result<()> {
        let mut adapter = self.adapter.lock().await;
        let legacy_users = adapter.fetch_users().await?;

        let mut success_count = 0;
        let mut fail_count = 0;

        for legacy_user in legacy_users {
            info!("Syncing user: {}", legacy_user.UserID);
            match save_user(&self.db, &legacy_user).await {
                Ok(_) => success_count += 1,
                Err(e) => {
                    error!("Failed to save user {}: {}", legacy_user.UserID, e);
                    fail_count += 1;
                }
            }
        }

        info!(
            "User sync completed: {} success, {} failed",
            success_count, fail_count
        );
        Ok(())
    }

    /// 增量同步步骤
    async fn sync_incremental_step(
        adapter: &Arc<tokio::sync::Mutex<LegacySyncAdapter>>,
        db: &Arc<PgPool>,
    ) -> Result<()> {
        let mut adap = adapter.lock().await;

        // 获取需要同步的设备ID列表
        let device_ids = Self::get_active_device_ids(db).await.unwrap_or_default();

        if device_ids.is_empty() {
            debug!("No active devices to sync");
            return Ok(());
        }

        info!("Incremental sync for {} devices", device_ids.len());

        for device_id in device_ids {
            let start_time = chrono::Utc::now().timestamp() - 3600; // 1小时前
            let end_time = chrono::Utc::now().timestamp();

            match adap
                .fetch_gps_history(&device_id, start_time, end_time)
                .await
            {
                Ok(gps_data_list) => {
                    let mut saved_count = 0;
                    for gps_data in gps_data_list {
                        if let Err(e) = Self::save_gps_data(db, &gps_data).await {
                            error!("Failed to save GPS data for {}: {}", device_id, e);
                        } else {
                            saved_count += 1;
                        }
                    }

                    if saved_count > 0 {
                        info!("Saved {} GPS points for device {}", saved_count, device_id);
                    }
                }
                Err(e) => {
                    warn!("Failed to fetch GPS history for {}: {}", device_id, e);
                }
            }
        }

        Ok(())
    }

    /// 获取活跃的设备ID列表
    async fn get_active_device_ids(db: &PgPool) -> Result<Vec<String>> {
        let device_ids = sqlx::query_scalar::<_, String>(
            r#"
            SELECT device_id
            FROM vehicles
            WHERE device_id IS NOT NULL
              AND status = 1
              AND operation_status = 1
            ORDER BY vehicle_id
            LIMIT 100
            "#,
        )
        .fetch_all(db)
        .await?;

        Ok(device_ids)
    }

    /// 保存GPS数据到数据库
    async fn save_gps_data(db: &Arc<PgPool>, gps_data: &LegacyGpsData) -> Result<()> {
        // 将LegacyGpsData转换为SyncGpsData
        let timestamp = match parse_datetime(&gps_data.GPSDateTime) {
            Ok(dt) => dt,
            Err(e) => {
                error!(
                    "Failed to parse GPS datetime {}: {}",
                    gps_data.GPSDateTime, e
                );
                return Err(anyhow::anyhow!("Invalid GPS datetime: {}", e));
            }
        };

        // 优化:避免不必要的克隆,使用String::from替换"legacy".to_string()
        let sync_gps = SyncGpsData {
            device_id: gps_data.DeviceID.clone(),
            latitude: gps_data.Latitude,
            longitude: gps_data.Longitude,
            speed: gps_data.Speed,
            direction: gps_data.Direction,
            altitude: gps_data.Altitude,
            timestamp,
            status: gps_data.Status,
            satellite_count: gps_data.SatelliteCount,
            io_status: gps_data.IOStatus.clone(),
            source: String::from("legacy"),
            received_at: Utc::now(),
        };

        let _result = sqlx::query(
            r#"
            INSERT INTO gps_track_data (
                vehicle_id, device_id, longitude, latitude, speed,
                direction, altitude, gps_time, receive_time,
                status, satellite_count, io_status
            )
            VALUES (
                (SELECT vehicle_id FROM vehicles WHERE device_id = $1 LIMIT 1),
                $1, $2, $3, $4, $5, $6, $7, NOW(), $8, $9, $10
            )
            ON CONFLICT (device_id, gps_time) DO NOTHING
            "#,
        )
        .bind(&sync_gps.device_id)
        .bind(sync_gps.longitude)
        .bind(sync_gps.latitude)
        .bind(sync_gps.speed)
        .bind(sync_gps.direction)
        .bind(sync_gps.altitude)
        .bind(sync_gps.timestamp)
        .bind(sync_gps.status)
        .bind(sync_gps.satellite_count)
        .bind(&sync_gps.io_status)
        .execute(&**db)
        .await?;

        debug!("GPS data for {} saved/updated", sync_gps.device_id);
        Ok(())
    }
}

/// 保存车辆数据到数据库
async fn save_vehicle(db: &PgPool, legacy_vehicle: &LegacyVehicle) -> Result<()> {
    // 转换为SyncVehicle
    let sync_vehicle = SyncVehicle {
        vehicle_id: legacy_vehicle.VehicleID.clone(),
        plate_number: legacy_vehicle.PlateNumber.clone(),
        device_id: Some(legacy_vehicle.DeviceID.clone()),
        vehicle_type: legacy_vehicle.VehicleType.clone(),
        status: legacy_vehicle.Status.to_string(),
        phone: legacy_vehicle.Phone.clone(),
        sim_card: legacy_vehicle.SIMCard.clone(),
        install_date: parse_date_utc(&legacy_vehicle.InstallDate),
        expire_date: parse_date_utc(&legacy_vehicle.ExpireDate),
        source: String::from("legacy"),
        legacy_vehicle_id: Some(legacy_vehicle.VehicleID.clone()),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    // 检查车辆是否已存在
    let exists =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM vehicles WHERE vehicle_id::text = $1")
            .bind(&sync_vehicle.vehicle_id)
            .fetch_one(db)
            .await
            .unwrap_or(0)
            > 0;

    if exists {
        // 更新现有车辆
        let result = sqlx::query(
            r#"
            UPDATE vehicles SET
                license_plate = $1,
                device_id = $2,
                vehicle_type = $3,
                status = $4,
                phone = $5,
                sim_card = $6,
                install_date = $7,
                expire_date = $8,
                update_time = NOW()
            WHERE vehicle_id::text = $9
            "#,
        )
        .bind(&sync_vehicle.plate_number)
        .bind(&sync_vehicle.device_id)
        .bind(&sync_vehicle.vehicle_type)
        .bind(&sync_vehicle.status)
        .bind(&sync_vehicle.phone)
        .bind(&sync_vehicle.sim_card)
        .bind(sync_vehicle.install_date)
        .bind(sync_vehicle.expire_date)
        .bind(&sync_vehicle.vehicle_id)
        .execute(db)
        .await?;

        debug!(
            "Vehicle {} updated, rows affected: {}",
            sync_vehicle.vehicle_id,
            result.rows_affected()
        );
    } else {
        // 插入新车辆 (简化版,需要根据实际表结构调整)
        let result = sqlx::query(
            r#"
            INSERT INTO vehicles (
                vehicle_id, license_plate, device_id, vehicle_type,
                status, phone, sim_card, install_date, expire_date,
                create_time, update_time
            )
            VALUES ($1::integer, $2, $3, $4, $5, $6, $7, $8, $9, NOW(), NOW())
            "#,
        )
        .bind(&sync_vehicle.vehicle_id)
        .bind(&sync_vehicle.plate_number)
        .bind(&sync_vehicle.device_id)
        .bind(&sync_vehicle.vehicle_type)
        .bind(&sync_vehicle.status)
        .bind(&sync_vehicle.phone)
        .bind(&sync_vehicle.sim_card)
        .bind(sync_vehicle.install_date)
        .bind(sync_vehicle.expire_date)
        .execute(db)
        .await?;

        debug!(
            "Vehicle {} inserted, rows affected: {}",
            sync_vehicle.vehicle_id,
            result.rows_affected()
        );
    }

    Ok(())
}

/// 保存用户数据到数据库
async fn save_user(db: &PgPool, legacy_user: &LegacyUser) -> Result<()> {
    // 转换为SyncUser
    let sync_user = SyncUser {
        username: legacy_user.Username.clone(),
        email: legacy_user.Email.clone(),
        password_hash: Some(legacy_user.Password.clone()),
        role: legacy_user.Role.clone(),
        phone: legacy_user.Phone.clone(),
        department: legacy_user.Department.clone(),
        source: String::from("legacy"),
        legacy_user_id: Some(legacy_user.UserID.clone()),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    // 检查用户是否已存在
    let exists = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users WHERE user_name = $1")
        .bind(&sync_user.username)
        .fetch_one(db)
        .await
        .unwrap_or(0)
        > 0;

    if exists {
        // 更新现有用户
        let result = sqlx::query(
            r#"
            UPDATE users SET
                email = $1,
                password = $2,
                role = $3,
                phone = $4,
                update_time = NOW()
            WHERE username = $5
            "#,
        )
        .bind(&sync_user.email)
        .bind(&sync_user.password_hash)
        .bind(&sync_user.role)
        .bind(&sync_user.phone)
        .bind(&sync_user.username)
        .execute(db)
        .await?;

        debug!(
            "User {} updated, rows affected: {}",
            sync_user.username,
            result.rows_affected()
        );
    } else {
        // 插入新用户 (简化版,需要根据实际表结构调整)
        let result = sqlx::query(
            r#"
            INSERT INTO users (
                username, email, password, role, phone,
                create_time, update_time
            )
            VALUES ($1, $2, $3, $4, $5, NOW(), NOW())
            "#,
        )
        .bind(&sync_user.username)
        .bind(&sync_user.email)
        .bind(&sync_user.password_hash)
        .bind(&sync_user.role)
        .bind(&sync_user.phone)
        .execute(db)
        .await?;

        debug!(
            "User {} inserted, rows affected: {}",
            sync_user.username,
            result.rows_affected()
        );
    }

    Ok(())
}
