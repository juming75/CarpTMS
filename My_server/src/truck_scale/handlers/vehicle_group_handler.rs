//! / 车组管理处理器
use crate::truck_scale::db::TruckScaleDb;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

/// 车组信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VehicleGroupInfo {
    pub group_id: String,
    pub parent_id: String,
    pub group_name: String,
    pub contact_people: String,
    pub contact_tel: String,
}

/// 车组管理处理器
pub struct VehicleGroupHandler {
    db: TruckScaleDb,
}

impl VehicleGroupHandler {
    /// 创建新的车组管理处理器
    pub fn new(db: TruckScaleDb) -> Self {
        Self { db }
    }

    /// 从连接池创建
    pub fn new_with_pool(pool: PgPool) -> Self {
        Self {
            db: TruckScaleDb::new(pool.into()),
        }
    }

    /// 查询车组
    pub async fn query_group(&self, group_id: &str) -> Result<Option<VehicleGroupInfo>> {
        let group_data = self.db.query_vehicle_group(group_id).await?;
        Ok(group_data.map(|data| VehicleGroupInfo {
            group_id: data["group_id"].as_str().unwrap_or("").to_string(),
            parent_id: data["parent_id"].as_str().unwrap_or("").to_string(),
            group_name: data["group_name"].as_str().unwrap_or("").to_string(),
            contact_people: data["contact_people"].as_str().unwrap_or("").to_string(),
            contact_tel: data["contact_tel"].as_str().unwrap_or("").to_string(),
        }))
    }

    /// 查询所有车组
    pub async fn query_all_groups(&self) -> Result<Vec<VehicleGroupInfo>> {
        let groups_data = self.db.query_all_vehicle_groups().await?;
        Ok(groups_data
            .into_iter()
            .map(|data| VehicleGroupInfo {
                group_id: data["group_id"].as_str().unwrap_or("").to_string(),
                parent_id: data["parent_id"].as_str().unwrap_or("").to_string(),
                group_name: data["group_name"].as_str().unwrap_or("").to_string(),
                contact_people: data["contact_people"].as_str().unwrap_or("").to_string(),
                contact_tel: data["contact_tel"].as_str().unwrap_or("").to_string(),
            })
            .collect())
    }

    /// 添加车组
    pub async fn add_vehicle_group(&self, group_data: serde_json::Value) -> Result<String> {
        self.db.add_vehicle_group(group_data).await
    }

    /// 更新车组
    pub async fn update_vehicle_group(&self, group_data: serde_json::Value) -> Result<()> {
        self.db.update_vehicle_group(group_data).await
    }

    /// 删除车组
    pub async fn delete_vehicle_group(&self, group_id: &str, delete_by: &str) -> Result<()> {
        self.db.delete_vehicle_group(group_id, delete_by).await
    }

    /// 根据父车组ID查询子车组
    pub async fn query_child_groups(&self, parent_id: &str) -> Result<Vec<VehicleGroupInfo>> {
        let groups = sqlx::query_as::<
            _,
            (
                String,
                Option<String>,
                String,
                Option<String>,
                Option<String>,
            ),
        >(
            "SELECT group_id, parent_id, group_name, contact_people, contact_tel
             FROM truck_scale_vehicle_groups
             WHERE parent_id = $1 AND status = 0
             ORDER BY group_id",
        )
        .bind(parent_id)
        .fetch_all(self.db.pool())
        .await?;

        Ok(groups
            .into_iter()
            .map(
                |(group_id, parent_id, group_name, contact_people, contact_tel)| VehicleGroupInfo {
                    group_id,
                    parent_id: parent_id.unwrap_or_default(),
                    group_name,
                    contact_people: contact_people.unwrap_or_default(),
                    contact_tel: contact_tel.unwrap_or_default(),
                },
            )
            .collect())
    }

    /// 获取车组树形结构
    pub async fn get_vehicle_group_tree(&self) -> Result<serde_json::Value> {
        let groups = self.query_all_groups().await?;

        // 构建树形结构
        let mut root_groups = Vec::new();
        let mut group_map: std::collections::HashMap<String, serde_json::Value> =
            std::collections::HashMap::new();

        // 先创建所有节点
        for group in &groups {
            let group_id = group.group_id.clone();
            let node = serde_json::json!({
                "group_id": group.group_id,
                "parent_id": group.parent_id,
                "group_name": group.group_name,
                "contact_people": group.contact_people,
                "contact_tel": group.contact_tel,
                "children": Vec::<serde_json::Value>::new()
            });
            group_map.insert(group_id, node);
        }

        // 构建树
        for group in &groups {
            let group_id = group.group_id.clone();
            let node = group_map.get(&group_id).expect("group should be in map after insert").clone();

            if !group.parent_id.is_empty() {
                if let Some(parent) = group_map.get_mut(&group.parent_id) {
                    parent["children"].as_array_mut().expect("children should always be an array").push(node);
                }
            } else {
                root_groups.push(node);
            }
        }

        Ok(serde_json::json!(root_groups))
    }
}
