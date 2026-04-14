//! /! 基于事件溯源的车辆仓储实现
//!
//! 使用事件溯源模式持久化和重建车辆聚合根

use crate::domain::ddd::EventSourcedAggregate;
use crate::domain::vehicle_aggregate::{PlateNumber, Vehicle, VehicleId, VehicleRepository};
use crate::errors::{AppError, AppResult};
use crate::events::event_store::{get_events_by_aggregate_id, save_events};
use log::{debug, info};
use sqlx::PgPool;

/// 基于事件溯源的车辆仓储
pub struct EventSourcedVehicleRepository {
    db: PgPool,
}

impl EventSourcedVehicleRepository {
    /// 创建新的事件溯源车辆仓储
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    /// 从事件重建车辆
    async fn rebuild_vehicle(&self, vehicle_id: &VehicleId) -> AppResult<Option<Vehicle>> {
        let aggregate_id = vehicle_id.to_string();

        // 从事件存储获取该车辆的所有事件
        let events = get_events_by_aggregate_id(&aggregate_id).await?;

        if events.is_empty() {
            return Ok(None);
        }

        // 创建一个空的车辆实例
        let mut vehicle = Vehicle::new(PlateNumber("temp".to_string()), None);

        // 从事件重建车辆状态
        vehicle.id = vehicle_id.clone();
        vehicle.rebuild_from_events(&events)?;

        debug!(
            "Rebuilt vehicle {} from {} events",
            vehicle_id,
            events.len()
        );
        Ok(Some(vehicle))
    }

    /// 保存车辆事件到事件存储
    async fn save_vehicle_events(&self, vehicle: &Vehicle) -> AppResult<()> {
        let events = vehicle.get_uncommitted_events();

        if !events.is_empty() {
            save_events(events).await?;
            info!("Saved {} events for vehicle {}", events.len(), vehicle.id);
        }

        Ok(())
    }

    /// 保存车辆到传统数据库(用于快速查询)
    async fn save_to_database(&self, vehicle: &Vehicle) -> AppResult<()> {
        let sql = r#"
        INSERT INTO vehicles (
            id, plate_number, status, device_id, version
        ) VALUES (
            $1, $2, $3, $4, $5
        ) ON CONFLICT (id) DO UPDATE SET
            plate_number = $2,
            status = $3,
            device_id = $4,
            version = $5
        "#;

        let device_id = vehicle.device.as_ref().map(|d| d.id.clone());
        let status = format!("{:?}", vehicle.status);

        sqlx::query::<sqlx::Postgres>(sql)
            .bind(vehicle.id.0)
            .bind(vehicle.plate_number.0.clone())
            .bind(status)
            .bind(device_id)
            .bind(vehicle.version as i64)
            .execute(&self.db)
            .await
            .map_err(|e| AppError::db_error("Failed to save vehicle", Some(&e.to_string())))?;

        Ok(())
    }
}

// 实现VehicleRepository trait
#[async_trait::async_trait]
impl VehicleRepository for EventSourcedVehicleRepository {
    async fn find_by_id(&self, id: &VehicleId) -> AppResult<Option<Vehicle>> {
        // 先尝试从传统数据库查询(快速路径)
        let sql = "SELECT id FROM vehicles WHERE id = $1";
        let exists = sqlx::query_scalar::<_, i32>(sql)
            .bind(id.0)
            .fetch_optional(&self.db)
            .await
            .map_err(|e| {
                AppError::db_error("Failed to check vehicle existence", Some(&e.to_string()))
            })?;

        if exists.is_none() {
            return Ok(None);
        }

        // 从事件重建车辆(确保状态一致性)
        self.rebuild_vehicle(id).await
    }

    async fn save(&self, vehicle: &mut Vehicle) -> AppResult<()> {
        let transaction =
            self.db.begin().await.map_err(|e| {
                AppError::db_error("Failed to begin transaction", Some(&e.to_string()))
            })?;

        // 保存事件到事件存储
        self.save_vehicle_events(vehicle).await?;

        // 保存到传统数据库(用于快速查询)
        self.save_to_database(vehicle).await?;

        // 标记事件为已提交
        vehicle.mark_events_committed();

        transaction.commit().await.map_err(|e| {
            AppError::db_error("Failed to commit transaction", Some(&e.to_string()))
        })?;

        info!("Saved vehicle {}", vehicle.id);
        Ok(())
    }

    async fn find_by_plate(&self, plate: &PlateNumber) -> AppResult<Option<Vehicle>> {
        // 从传统数据库查询车辆ID
        let sql = "SELECT id FROM vehicles WHERE plate_number = $1";
        let vehicle_id = sqlx::query_scalar::<sqlx::Postgres, i32>(sql)
            .bind(plate.0.clone())
            .fetch_optional(&self.db)
            .await
            .map_err(|e| {
                AppError::db_error("Failed to find vehicle by plate", Some(&e.to_string()))
            })?;

        if let Some(id) = vehicle_id {
            let vehicle_id = VehicleId(id);
            self.rebuild_vehicle(&vehicle_id).await
        } else {
            Ok(None)
        }
    }

    async fn find_active_vehicles(&self) -> AppResult<Vec<Vehicle>> {
        // 从传统数据库查询活跃车辆ID
        let sql = "SELECT id FROM vehicles WHERE status = 'Active'";
        let vehicle_ids = sqlx::query_scalar::<_, i32>(sql)
            .fetch_all(&self.db)
            .await
            .map_err(|e| {
                AppError::db_error("Failed to find active vehicles", Some(&e.to_string()))
            })?;

        let mut vehicles = Vec::new();
        for id in vehicle_ids {
            let vehicle_id = VehicleId(id);
            if let Some(vehicle) = self.rebuild_vehicle(&vehicle_id).await? {
                vehicles.push(vehicle);
            }
        }

        Ok(vehicles)
    }
}
