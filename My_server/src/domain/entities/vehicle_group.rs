//! 车组领域实体

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 车组实体
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VehicleGroup {
    pub group_id: i32,
    pub group_name: String,
    pub parent_id: Option<i32>,
    pub parent_name: Option<String>,
    pub description: Option<String>,
    pub vehicle_count: i64,
    pub create_time: DateTime<Utc>,
    pub update_time: Option<DateTime<Utc>>,
}

/// 车组创建请求
#[derive(Debug, Clone, Deserialize)]
pub struct VehicleGroupCreateRequest {
    pub group_name: String,
    pub parent_id: Option<i32>,
    pub description: Option<String>,
}

/// 车组更新请求
#[derive(Debug, Clone, Deserialize)]
pub struct VehicleGroupUpdateRequest {
    pub group_name: Option<String>,
    pub parent_id: Option<i32>,
    pub description: Option<String>,
}

/// 车组查询参数
#[derive(Debug, Clone, Deserialize)]
pub struct VehicleGroupQuery {
    pub page: Option<i32>,
    pub page_size: Option<i32>,
    pub group_name: Option<String>,
}

/// 车组树节点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VehicleGroupTreeNode {
    pub group_id: i32,
    pub group_name: String,
    pub parent_id: Option<i32>,
    pub description: Option<String>,
    pub vehicle_count: i64,
    pub children: Vec<VehicleGroupTreeNode>,
}

impl VehicleGroup {
    /// 创建新车组
    pub fn new(
        group_id: i32,
        group_name: String,
        parent_id: Option<i32>,
        description: Option<String>,
    ) -> Self {
        Self {
            group_id,
            group_name,
            parent_id,
            parent_name: None,
            description,
            vehicle_count: 0,
            create_time: Utc::now(),
            update_time: None,
        }
    }

    /// 更新车组信息
    pub fn update(
        &mut self,
        name: Option<String>,
        parent_id: Option<i32>,
        description: Option<String>,
    ) {
        if let Some(name) = name {
            self.group_name = name;
        }
        if parent_id.is_some() {
            self.parent_id = parent_id;
        }
        if description.is_some() {
            self.description = description;
        }
        self.update_time = Some(Utc::now());
    }

    /// 设置父车组名称
    pub fn set_parent_name(&mut self, parent_name: Option<String>) {
        self.parent_name = parent_name;
    }

    /// 设置车辆数量
    pub fn set_vehicle_count(&mut self, count: i64) {
        self.vehicle_count = count;
    }
}
