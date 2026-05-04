//! 回收站处理器
use crate::truck_scale::db::TruckScaleDb;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::Row;
use std::sync::Arc;

/// 回收车辆信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecycledVehicle {
    pub vehicle_id: String,
    pub plate_no: String,
    pub delete_time: String,
    pub delete_by: String,
}

/// 回收车组信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecycledVehicleGroup {
    pub group_id: String,
    pub group_name: String,
    pub delete_time: String,
    pub delete_by: String,
}

/// 回收用户信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecycledUser {
    pub user_id: String,
    pub user_name: String,
    pub real_name: String,
    pub delete_time: String,
    pub delete_by: String,
}

/// 回收站处理器
///
/// 提供软删除数据的查询和恢复功能
pub struct RecycleHandler {
    db: Arc<TruckScaleDb>,
}

impl RecycleHandler {
    /// 创建新的回收站处理器
    pub fn new(db: Arc<TruckScaleDb>) -> Self {
        Self { db }
    }

    /// 查询回收车辆
    pub async fn query_recycled_vehicles(&self) -> Result<Vec<RecycledVehicle>> {
        log::info!("Querying recycled vehicles");

        let vehicles = sqlx::query(
            "SELECT vehicle_id, plate_no, update_time as delete_time, update_by as delete_by
             FROM truck_scale_vehicles
             WHERE status = 1
             ORDER BY update_time DESC",
        )
        .fetch_all(self.db.pool())
        .await?;

        let result = vehicles
            .into_iter()
            .map(|v| RecycledVehicle {
                vehicle_id: v.get::<String, _>("vehicle_id"),
                plate_no: v.get::<String, _>("plate_no"),
                delete_time: v.get::<String, _>("delete_time"),
                delete_by: v.get::<String, _>("delete_by"),
            })
            .collect();

        Ok(result)
    }

    /// 查询回收车组
    pub async fn query_recycled_groups(&self) -> Result<Vec<RecycledVehicleGroup>> {
        log::info!("Querying recycled vehicle groups");

        let groups = sqlx::query(
            "SELECT group_id, group_name, update_time as delete_time, update_by as delete_by
             FROM truck_scale_vehicle_groups
             WHERE status = 1
             ORDER BY update_time DESC",
        )
        .fetch_all(self.db.pool())
        .await?;

        let result = groups
            .into_iter()
            .map(|g| RecycledVehicleGroup {
                group_id: g.get::<String, _>("group_id"),
                group_name: g.get::<String, _>("group_name"),
                delete_time: g.get::<String, _>("delete_time"),
                delete_by: g.get::<String, _>("delete_by"),
            })
            .collect();

        Ok(result)
    }

    /// 查询回收用户
    pub async fn query_recycled_users(&self) -> Result<Vec<RecycledUser>> {
        log::info!("Querying recycled users");

        let users = sqlx::query(
            "SELECT user_id, user_name, real_name, update_time as delete_time, update_by as delete_by
             FROM truck_scale_users
             WHERE status = 1
             ORDER BY update_time DESC",
        )
        .fetch_all(self.db.pool())
        .await?;

        let result = users
            .into_iter()
            .map(|u| RecycledUser {
                user_id: u.get::<String, _>("user_id"),
                user_name: u.get::<String, _>("user_name"),
                real_name: u.get::<String, _>("real_name"),
                delete_time: u.get::<String, _>("delete_time"),
                delete_by: u.get::<String, _>("delete_by"),
            })
            .collect();

        Ok(result)
    }

    /// 恢复车辆
    pub async fn restore_vehicle(&self, vehicle_id: &str, restore_by: &str) -> Result<()> {
        log::info!("Restoring vehicle: {} by {}", vehicle_id, restore_by);

        sqlx::query(
            "UPDATE truck_scale_vehicles 
             SET status = 0, update_by = $2, update_time = CURRENT_TIMESTAMP
             WHERE vehicle_id = $1 AND status = 1",
        )
        .bind(vehicle_id)
        .bind(restore_by)
        .execute(self.db.pool())
        .await?;

        Ok(())
    }

    /// 恢复车组
    pub async fn restore_vehicle_group(&self, group_id: &str, restore_by: &str) -> Result<()> {
        log::info!("Restoring vehicle group: {} by {}", group_id, restore_by);

        sqlx::query(
            "UPDATE truck_scale_vehicle_groups 
             SET status = 0, update_by = $2, update_time = CURRENT_TIMESTAMP
             WHERE group_id = $1 AND status = 1",
        )
        .bind(group_id)
        .bind(restore_by)
        .execute(self.db.pool())
        .await?;

        Ok(())
    }

    /// 恢复用户
    pub async fn restore_user(&self, user_id: &str, restore_by: &str) -> Result<()> {
        log::info!("Restoring user: {} by {}", user_id, restore_by);

        sqlx::query(
            "UPDATE truck_scale_users 
             SET status = 0, update_by = $2, update_time = CURRENT_TIMESTAMP
             WHERE user_id = $1 AND status = 1",
        )
        .bind(user_id)
        .bind(restore_by)
        .execute(self.db.pool())
        .await?;

        Ok(())
    }

    /// 彻底删除车辆
    pub async fn permanently_delete_vehicle(&self, vehicle_id: &str) -> Result<()> {
        log::info!("Permanently deleting vehicle: {}", vehicle_id);

        sqlx::query("DELETE FROM truck_scale_vehicles WHERE vehicle_id = $1")
            .bind(vehicle_id)
            .execute(self.db.pool())
            .await?;

        Ok(())
    }

    /// 彻底删除车组
    pub async fn permanently_delete_group(&self, group_id: &str) -> Result<()> {
        log::info!("Permanently deleting vehicle group: {}", group_id);

        sqlx::query("DELETE FROM truck_scale_vehicle_groups WHERE group_id = $1")
            .bind(group_id)
            .execute(self.db.pool())
            .await?;

        Ok(())
    }

    /// 彻底删除用户
    pub async fn permanently_delete_user(&self, user_id: &str) -> Result<()> {
        log::info!("Permanently deleting user: {}", user_id);

        sqlx::query("DELETE FROM truck_scale_users WHERE user_id = $1")
            .bind(user_id)
            .execute(self.db.pool())
            .await?;

        Ok(())
    }
}

impl Default for RecycleHandler {
    fn default() -> Self {
        panic!("RecycleHandler requires a database connection")
    }
}
