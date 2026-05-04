//! 权限管理处理器
use crate::truck_scale::db::TruckScaleDb;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::Row;
use std::sync::Arc;

/// 用户权限信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPermissions {
    pub user_id: String,
    pub user_name: String,
    pub real_name: String,
    pub permission: String,
    pub veh_group_list: String,
}

/// 用户组权限信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupPermissions {
    pub group_id: String,
    pub group_name: String,
    pub user_type: i32,
    pub permission: String,
}

/// 权限管理处理器
///
/// 提供用户和用户组的权限查询和更新功能
pub struct PermissionHandler {
    db: Arc<TruckScaleDb>,
}

impl PermissionHandler {
    /// 创建新的权限管理处理器
    pub fn new(db: Arc<TruckScaleDb>) -> Self {
        Self { db }
    }

    /// 查询用户权限
    pub async fn query_user_permissions(&self, user_id: &str) -> Result<Option<UserPermissions>> {
        log::info!("Querying user permissions: {}", user_id);

        let user = sqlx::query(
            "SELECT user_id, user_name, real_name, permission, veh_group_list
             FROM truck_scale_users
             WHERE user_id = $1 AND status = 0",
        )
        .bind(user_id)
        .fetch_optional(self.db.pool())
        .await?;

        Ok(user.map(|u| UserPermissions {
            user_id: u.get::<String, _>("user_id"),
            user_name: u.get::<String, _>("user_name"),
            real_name: u.get::<String, _>("real_name"),
            permission: u.get::<String, _>("permission"),
            veh_group_list: u.get::<String, _>("veh_group_list"),
        }))
    }

    /// 更新用户权限
    pub async fn update_user_permissions(
        &self,
        user_id: &str,
        permission: &str,
        veh_group_list: &str,
        update_by: &str,
    ) -> Result<()> {
        log::info!(
            "Updating user permissions: {} -> permission={}, veh_group_list={}",
            user_id,
            permission,
            veh_group_list
        );

        sqlx::query(
            "UPDATE truck_scale_users 
             SET permission = $2, veh_group_list = $3, update_by = $4, update_time = CURRENT_TIMESTAMP
             WHERE user_id = $1 AND status = 0",
        )
        .bind(user_id)
        .bind(permission)
        .bind(veh_group_list)
        .bind(update_by)
        .execute(self.db.pool())
        .await?;

        Ok(())
    }

    /// 查询所有用户权限
    pub async fn query_all_user_permissions(&self) -> Result<Vec<UserPermissions>> {
        log::info!("Querying all user permissions");

        let users = sqlx::query(
            "SELECT user_id, user_name, real_name, permission, veh_group_list
             FROM truck_scale_users
             WHERE status = 0
             ORDER BY user_id",
        )
        .fetch_all(self.db.pool())
        .await?;

        let result = users
            .into_iter()
            .map(|u| UserPermissions {
                user_id: u.get::<String, _>("user_id"),
                user_name: u.get::<String, _>("user_name"),
                real_name: u.get::<String, _>("real_name"),
                permission: u.get::<String, _>("permission"),
                veh_group_list: u.get::<String, _>("veh_group_list"),
            })
            .collect();

        Ok(result)
    }

    /// 查询用户组权限
    pub async fn query_group_permissions(
        &self,
        group_id: &str,
    ) -> Result<Option<GroupPermissions>> {
        log::info!("Querying group permissions: {}", group_id);

        let group = sqlx::query(
            "SELECT group_id, group_name, user_type, permission
             FROM truck_scale_user_groups
             WHERE group_id = $1 AND status = 0",
        )
        .bind(group_id)
        .fetch_optional(self.db.pool())
        .await?;

        Ok(group.map(|g| GroupPermissions {
            group_id: g.get::<String, _>("group_id"),
            group_name: g.get::<String, _>("group_name"),
            user_type: g.get::<i32, _>("user_type"),
            permission: g.get::<Option<String>, _>("permission").unwrap_or_default(),
        }))
    }

    /// 更新用户组权限
    pub async fn update_group_permissions(
        &self,
        group_id: &str,
        permission: &str,
        update_by: &str,
    ) -> Result<()> {
        log::info!(
            "Updating group permissions: {} -> permission={}",
            group_id,
            permission
        );

        sqlx::query(
            "UPDATE truck_scale_user_groups 
             SET permission = $2, update_by = $3, update_time = CURRENT_TIMESTAMP
             WHERE group_id = $1 AND status = 0",
        )
        .bind(group_id)
        .bind(permission)
        .bind(update_by)
        .execute(self.db.pool())
        .await?;

        Ok(())
    }

    /// 查询所有用户组权限
    pub async fn query_all_group_permissions(&self) -> Result<Vec<GroupPermissions>> {
        log::info!("Querying all group permissions");

        let groups = sqlx::query(
            "SELECT group_id, group_name, user_type, permission
             FROM truck_scale_user_groups
             WHERE status = 0
             ORDER BY group_id",
        )
        .fetch_all(self.db.pool())
        .await?;

        let result = groups
            .into_iter()
            .map(|g| GroupPermissions {
                group_id: g.get::<String, _>("group_id"),
                group_name: g.get::<String, _>("group_name"),
                user_type: g.get::<i32, _>("user_type"),
                permission: g.get::<Option<String>, _>("permission").unwrap_or_default(),
            })
            .collect();

        Ok(result)
    }
}

impl Default for PermissionHandler {
    fn default() -> Self {
        panic!("PermissionHandler requires a database connection")
    }
}
