//! / 用户组管理处理器
use crate::truck_scale::db::TruckScaleDb;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

/// 用户组信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserGroupInfo {
    pub group_id: String,
    pub group_name: String,
    pub user_type: i32,
    pub permission: String,
}

/// 用户组管理处理器
pub struct UserGroupHandler {
    db: TruckScaleDb,
}

impl UserGroupHandler {
    /// 创建新的用户组管理处理器
    pub fn new(db: TruckScaleDb) -> Self {
        Self { db }
    }

    /// 从连接池创建
    pub fn new_with_pool(pool: PgPool) -> Self {
        Self {
            db: TruckScaleDb::new(pool.into()),
        }
    }

    /// 查询用户组
    pub async fn query_group(&self, group_id: &str) -> Result<Option<UserGroupInfo>> {
        let group_data = self.db.query_user_group(group_id).await?;
        Ok(group_data.map(|data| UserGroupInfo {
            group_id: data["group_id"].as_str().unwrap_or("").to_string(),
            group_name: data["group_name"].as_str().unwrap_or("").to_string(),
            user_type: data["user_type"].as_i64().unwrap_or(3) as i32,
            permission: data["permission"].as_str().unwrap_or("").to_string(),
        }))
    }

    /// 查询所有用户组
    pub async fn query_all_groups(&self) -> Result<Vec<UserGroupInfo>> {
        let groups_data = self.db.query_all_user_groups().await?;
        Ok(groups_data
            .into_iter()
            .map(|data| UserGroupInfo {
                group_id: data["group_id"].as_str().unwrap_or("").to_string(),
                group_name: data["group_name"].as_str().unwrap_or("").to_string(),
                user_type: data["user_type"].as_i64().unwrap_or(3) as i32,
                permission: data["permission"].as_str().unwrap_or("").to_string(),
            })
            .collect())
    }

    /// 添加用户组
    pub async fn add_user_group(&self, group_data: serde_json::Value) -> Result<String> {
        self.db.add_user_group(group_data).await
    }

    /// 更新用户组
    pub async fn update_user_group(&self, group_data: serde_json::Value) -> Result<()> {
        self.db.update_user_group(group_data).await
    }

    /// 删除用户组
    pub async fn delete_user_group(&self, group_id: &str, delete_by: &str) -> Result<()> {
        self.db.delete_user_group(group_id, delete_by).await
    }

    /// 根据用户类型查询用户组
    pub async fn query_groups_by_type(&self, user_type: i32) -> Result<Vec<serde_json::Value>> {
        let groups = sqlx::query_as::<_, (String, String, i32, Option<String>)>(
            "SELECT group_id, group_name, user_type, permission
             FROM truck_scale_user_groups 
             WHERE user_type = $1 AND status = 0
             ORDER BY group_id",
        )
        .bind(user_type)
        .fetch_all(self.db.pool())
        .await?;

        Ok(groups
            .into_iter()
            .map(|g| {
                serde_json::json!({
                    "group_id": g.0,
                    "group_name": g.1,
                    "user_type": g.2,
                    "permission": g.3,
                })
            })
            .collect())
    }

    /// 获取用户组权限统计
    pub async fn get_group_permission_stats(&self) -> Result<serde_json::Value> {
        let stats = sqlx::query_as::<_, (i64, i64, i64, i64)>(
            "SELECT 
                COUNT(*) as total,
                SUM(CASE WHEN user_type = 0 THEN 1 ELSE 0 END) as super_admin_count,
                SUM(CASE WHEN user_type = 1 THEN 1 ELSE 0 END) as admin_count,
                SUM(CASE WHEN user_type = 2 THEN 1 ELSE 0 END) as operator_count
             FROM truck_scale_user_groups 
             WHERE status = 0",
        )
        .fetch_one(self.db.pool())
        .await?;

        Ok(serde_json::json!({
            "total_count": stats.0,
            "super_admin_count": stats.1,
            "admin_count": stats.2,
            "operator_count": stats.3,
        }))
    }
}
