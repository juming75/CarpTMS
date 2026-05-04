//! 车组仓库PostgreSQL实现

use chrono::TimeZone;
use sqlx::{PgPool, Row};
use std::sync::Arc;

use crate::domain::entities::vehicle_group::{
    VehicleGroup, VehicleGroupCreateRequest, VehicleGroupTreeNode, VehicleGroupUpdateRequest,
};
use crate::domain::use_cases::vehicle_group::VehicleGroupRepository;

/// 车组仓库PostgreSQL实现
pub struct PgVehicleGroupRepository {
    pool: Arc<PgPool>,
}

impl PgVehicleGroupRepository {
    /// 创建车组仓库实例
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl VehicleGroupRepository for PgVehicleGroupRepository {
    /// 获取车组列表
    async fn find_all(
        &self,
        page: i32,
        page_size: i32,
    ) -> Result<(Vec<VehicleGroup>, i64), anyhow::Error> {
        let offset = (page - 1) * page_size;

        // 查询总记录数
        let total_count = sqlx::query_scalar::<_, i64>(r#"SELECT COUNT(*) FROM vehicle_groups"#)
            .fetch_one(&*self.pool)
            .await?;

        // 查询分页数据
        let groups = sqlx::query(
            r#"SELECT vg.*, 
               COALESCE(parent.group_name, '') as parent_name, 
               (SELECT COUNT(*) FROM vehicles v WHERE v.group_id = vg.group_id) as vehicle_count 
        FROM vehicle_groups vg 
        LEFT JOIN vehicle_groups parent ON vg.parent_id = parent.group_id 
        ORDER BY vg.group_id 
        LIMIT $1 OFFSET $2"#,
        )
        .bind(page_size)
        .bind(offset)
        .fetch_all(&*self.pool)
        .await?
        .into_iter()
        .map(|row| {
            let group_id: i32 = row.get("group_id");
            let group_name: String = row.get("group_name");
            let parent_id: Option<i32> = row.get("parent_id");
            let parent_name: Option<String> = row.get("parent_name");
            let description: Option<String> = row.get("description");
            let vehicle_count: i64 = row.get("vehicle_count");
            let create_time: chrono::NaiveDateTime = row.get("create_time");
            let update_time: Option<chrono::NaiveDateTime> = row.get("update_time");

            VehicleGroup {
                group_id,
                group_name,
                parent_id,
                parent_name,
                description,
                vehicle_count,
                create_time: chrono::Utc.from_utc_datetime(&create_time),
                update_time: update_time.map(|t| chrono::Utc.from_utc_datetime(&t)),
            }
        })
        .collect();

        Ok((groups, total_count))
    }

    /// 获取单个车组
    async fn find_by_id(&self, group_id: i32) -> Result<Option<VehicleGroup>, anyhow::Error> {
        let row = sqlx::query(
            r#"SELECT vg.*, 
               COALESCE(parent.group_name, '') as parent_name, 
               (SELECT COUNT(*) FROM vehicles v WHERE v.group_id = vg.group_id) as vehicle_count 
        FROM vehicle_groups vg 
        LEFT JOIN vehicle_groups parent ON vg.parent_id = parent.group_id 
        WHERE vg.group_id = $1"#,
        )
        .bind(group_id)
        .fetch_optional(&*self.pool)
        .await?;

        match row {
            Some(row) => {
                let group_id: i32 = row.get("group_id");
                let group_name: String = row.get("group_name");
                let parent_id: Option<i32> = row.get("parent_id");
                let parent_name: Option<String> = row.get("parent_name");
                let description: Option<String> = row.get("description");
                let vehicle_count: i64 = row.get("vehicle_count");
                let create_time: chrono::NaiveDateTime = row.get("create_time");
                let update_time: Option<chrono::NaiveDateTime> = row.get("update_time");

                Ok(Some(VehicleGroup {
                    group_id,
                    group_name,
                    parent_id,
                    parent_name,
                    description,
                    vehicle_count,
                    create_time: chrono::Utc.from_utc_datetime(&create_time),
                    update_time: update_time.map(|t| chrono::Utc.from_utc_datetime(&t)),
                }))
            }
            None => Ok(None),
        }
    }

    /// 创建车组
    async fn create(
        &self,
        group: VehicleGroupCreateRequest,
    ) -> Result<VehicleGroup, anyhow::Error> {
        // 检查车组名称是否已存在
        let existing_count = sqlx::query_scalar::<_, i64>(
            r#"SELECT COUNT(*) FROM vehicle_groups WHERE group_name = $1"#,
        )
        .bind(&group.group_name)
        .fetch_one(&*self.pool)
        .await?;

        if existing_count > 0 {
            return Err(anyhow::anyhow!("车组名称已存在"));
        }

        let result = sqlx::query(
            r#"INSERT INTO vehicle_groups (group_name, parent_id, description, create_time, update_time) 
         VALUES ($1, $2, $3, $4, $4) 
         RETURNING group_id, group_name, parent_id, description, create_time, update_time"#,
        )
        .bind(&group.group_name)
        .bind(group.parent_id)
        .bind(&group.description)
        .bind(chrono::Local::now().naive_local())
        .fetch_one(&*self.pool)
        .await?;

        let group_id: i32 = result.get("group_id");
        let group_name: String = result.get("group_name");
        let parent_id: Option<i32> = result.get("parent_id");
        let description: Option<String> = result.get("description");
        let create_time = result.get("create_time");
        let update_time: Option<chrono::NaiveDateTime> = result.get("update_time");

        // 构建返回的车组对象
        let mut vehicle_group = VehicleGroup {
            group_id,
            group_name,
            parent_id,
            parent_name: None,
            description,
            vehicle_count: 0,
            create_time: chrono::Utc.from_utc_datetime(&create_time),
            update_time: update_time.map(|t| chrono::Utc.from_utc_datetime(&t)),
        };

        // 获取父车组名称
        if let Some(parent_id) = parent_id {
            if let Some(parent_name) = sqlx::query_scalar::<_, Option<String>>(
                r#"SELECT group_name FROM vehicle_groups WHERE group_id = $1"#,
            )
            .bind(parent_id)
            .fetch_optional(&*self.pool)
            .await?
            {
                vehicle_group.parent_name = parent_name;
            }
        }

        Ok(vehicle_group)
    }

    /// 更新车组
    async fn update(
        &self,
        group_id: i32,
        group: VehicleGroupUpdateRequest,
    ) -> Result<VehicleGroup, anyhow::Error> {
        // 检查车组是否存在
        let existing_count = sqlx::query_scalar::<_, i64>(
            r#"SELECT COUNT(*) FROM vehicle_groups WHERE group_id = $1"#,
        )
        .bind(group_id)
        .fetch_one(&*self.pool)
        .await?;

        if existing_count == 0 {
            return Err(anyhow::anyhow!("车组不存在"));
        }

        // 检查车组名称是否已被其他车组使用
        if let Some(group_name) = &group.group_name {
            if !group_name.is_empty() {
                let duplicate_count = sqlx::query_scalar::<_, i64>(
                    r#"SELECT COUNT(*) FROM vehicle_groups WHERE group_name = $1 AND group_id != $2"#
                )
                .bind(group_name)
                .bind(group_id)
                .fetch_one(&*self.pool)
                .await?;

                if duplicate_count > 0 {
                    return Err(anyhow::anyhow!("车组名称已存在"));
                }
            }
        }

        let result = sqlx::query(
            r#"UPDATE vehicle_groups 
         SET group_name = COALESCE($1, group_name), 
             parent_id = COALESCE($2, parent_id), 
             description = COALESCE($3, description), 
             update_time = $4 
         WHERE group_id = $5 
         RETURNING group_id, group_name, parent_id, description, create_time, update_time"#,
        )
        .bind(&group.group_name)
        .bind(group.parent_id)
        .bind(&group.description)
        .bind(chrono::Local::now().naive_local())
        .bind(group_id)
        .fetch_optional(&*self.pool)
        .await?;

        match result {
            Some(row) => {
                let group_id: i32 = row.get("group_id");
                let group_name: String = row.get("group_name");
                let parent_id: Option<i32> = row.get("parent_id");
                let description: Option<String> = row.get("description");
                let create_time = row.get("create_time");
                let update_time: Option<chrono::NaiveDateTime> = row.get("update_time");

                // 构建返回的车组对象
                let mut vehicle_group = VehicleGroup {
                    group_id,
                    group_name,
                    parent_id,
                    parent_name: None,
                    description,
                    vehicle_count: 0,
                    create_time: chrono::Utc.from_utc_datetime(&create_time),
                    update_time: update_time.map(|t| chrono::Utc.from_utc_datetime(&t)),
                };

                // 获取父车组名称
                if let Some(parent_id) = parent_id {
                    if let Some(parent_name) = sqlx::query_scalar::<_, Option<String>>(
                        r#"SELECT group_name FROM vehicle_groups WHERE group_id = $1"#,
                    )
                    .bind(parent_id)
                    .fetch_optional(&*self.pool)
                    .await?
                    {
                        vehicle_group.parent_name = parent_name;
                    }
                }

                // 获取车辆数量
                let vehicle_count = sqlx::query_scalar::<_, i64>(
                    r#"SELECT COUNT(*) FROM vehicles WHERE group_id = $1"#,
                )
                .bind(group_id)
                .fetch_one(&*self.pool)
                .await?;

                vehicle_group.vehicle_count = vehicle_count;

                Ok(vehicle_group)
            }
            None => Err(anyhow::anyhow!("车组不存在")),
        }
    }

    /// 删除车组
    async fn delete(&self, group_id: i32) -> Result<(), anyhow::Error> {
        let result = sqlx::query(r#"DELETE FROM vehicle_groups WHERE group_id = $1"#)
            .bind(group_id)
            .execute(&*self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(anyhow::anyhow!("车组不存在"));
        }

        Ok(())
    }

    /// 检查车组是否有关联数据
    async fn has_related_data(&self, group_id: i32) -> Result<bool, anyhow::Error> {
        // 检查是否有车辆属于该车组
        let vehicle_count =
            sqlx::query_scalar::<_, i64>(r#"SELECT COUNT(*) FROM vehicles WHERE group_id = $1"#)
                .bind(group_id)
                .fetch_one(&*self.pool)
                .await?;

        if vehicle_count > 0 {
            return Ok(true);
        }

        // 检查是否有子车组
        let child_count = sqlx::query_scalar::<_, i64>(
            r#"SELECT COUNT(*) FROM vehicle_groups WHERE parent_id = $1"#,
        )
        .bind(group_id)
        .fetch_one(&*self.pool)
        .await?;

        Ok(child_count > 0)
    }

    /// 获取车组树结构
    async fn get_tree(&self) -> Result<Vec<VehicleGroupTreeNode>, anyhow::Error> {
        // 查询所有车组
        let groups = sqlx::query(
            r#"SELECT group_id, group_name, parent_id, description, create_time, update_time 
                               FROM vehicle_groups 
                               ORDER BY group_name"#,
        )
        .fetch_all(&*self.pool)
        .await?;

        // 查询每个车组的车辆数量
        let vehicle_counts = sqlx::query(
            r#"SELECT group_id, COUNT(*) as count 
                                      FROM vehicles 
                                      GROUP BY group_id"#,
        )
        .fetch_all(&*self.pool)
        .await?;

        // 将车辆数量转换为HashMap,便于查询
        let mut vehicle_count_map = std::collections::HashMap::new();
        for count in vehicle_counts {
            let group_id: i32 = count.get("group_id");
            let count: i64 = count.get("count");
            vehicle_count_map.insert(group_id, count);
        }

        // 构建车组树
        let mut root_groups = Vec::new();
        let mut group_map = std::collections::HashMap::new();

        // 第一遍:创建所有节点并放入map
        for group in groups.iter() {
            let group_id: i32 = group.get("group_id");
            let group_name: String = group.get("group_name");
            let parent_id: Option<i32> = group.get("parent_id");
            let description: Option<String> = group.get("description");
            let vehicle_count = *vehicle_count_map.get(&group_id).unwrap_or(&0);

            let node = VehicleGroupTreeNode {
                group_id,
                group_name,
                parent_id,
                description,
                vehicle_count,
                children: Vec::new(),
            };

            group_map.insert(group_id, node);
        }

        // 收集所有需要处理的节点ID
        let group_ids: Vec<i32> = groups.iter().map(|g| g.get("group_id")).collect();

        // 第二遍:构建树结构
        for group_id in group_ids {
            // 获取父节点ID
            let parent_id = if let Some(group) = group_map.get(&group_id) {
                group.parent_id
            } else {
                continue;
            };

            if let Some(parent_id) = parent_id {
                // 先检查父节点是否存在
                if group_map.contains_key(&parent_id) {
                    // 移除当前节点
                    if let Some(child_node) = group_map.remove(&group_id) {
                        // 再获取父节点的可变引用并添加子节点
                        if let Some(parent_node) = group_map.get_mut(&parent_id) {
                            parent_node.children.push(child_node);
                        }
                    }
                }
            } else {
                // 没有父节点,作为根节点
                if let Some(root_node) = group_map.remove(&group_id) {
                    root_groups.push(root_node);
                }
            }
        }

        Ok(root_groups)
    }

    /// 检查车组是否存在
    async fn exists(&self, group_id: i32) -> Result<bool, anyhow::Error> {
        let count = sqlx::query_scalar::<_, i64>(
            r#"SELECT COUNT(*) FROM vehicle_groups WHERE group_id = $1"#,
        )
        .bind(group_id)
        .fetch_one(&*self.pool)
        .await?;

        Ok(count > 0)
    }

    /// 根据名称统计车组数量
    async fn count_by_name(
        &self,
        name: &str,
        exclude_id: Option<i32>,
    ) -> Result<i64, anyhow::Error> {
        if let Some(exclude_id) = exclude_id {
            sqlx::query_scalar::<_, i64>(
                r#"SELECT COUNT(*) FROM vehicle_groups WHERE group_name = $1 AND group_id != $2"#,
            )
            .bind(name)
            .bind(exclude_id)
            .fetch_one(&*self.pool)
            .await
            .map_err(anyhow::Error::from)
        } else {
            sqlx::query_scalar::<_, i64>(
                r#"SELECT COUNT(*) FROM vehicle_groups WHERE group_name = $1"#,
            )
            .bind(name)
            .fetch_one(&*self.pool)
            .await
            .map_err(anyhow::Error::from)
        }
    }

    /// 统计车组下的车辆数量
    async fn count_vehicles(&self, group_id: i32) -> Result<i64, anyhow::Error> {
        sqlx::query_scalar::<_, i64>(r#"SELECT COUNT(*) FROM vehicles WHERE group_id = $1"#)
            .bind(group_id)
            .fetch_one(&*self.pool)
            .await
            .map_err(anyhow::Error::from)
    }

    /// 统计子车组数量
    async fn count_children(&self, group_id: i32) -> Result<i64, anyhow::Error> {
        sqlx::query_scalar::<_, i64>(r#"SELECT COUNT(*) FROM vehicle_groups WHERE parent_id = $1"#)
            .bind(group_id)
            .fetch_one(&*self.pool)
            .await
            .map_err(anyhow::Error::from)
    }
}
