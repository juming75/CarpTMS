//! 车组仓库接口

use crate::domain::entities::vehicle_group::{
    VehicleGroup, VehicleGroupCreateRequest, VehicleGroupTreeNode, VehicleGroupUpdateRequest,
};

/// 车组仓库接口
#[async_trait::async_trait]
pub trait VehicleGroupRepository: Send + Sync {
    /// 获取车组列表
    async fn find_all(
        &self,
        page: i32,
        page_size: i32,
    ) -> Result<(Vec<VehicleGroup>, i64), anyhow::Error>;

    /// 获取单个车组
    async fn find_by_id(&self, group_id: i32) -> Result<Option<VehicleGroup>, anyhow::Error>;

    /// 创建车组
    async fn create(&self, group: VehicleGroupCreateRequest)
        -> Result<VehicleGroup, anyhow::Error>;

    /// 更新车组
    async fn update(
        &self,
        group_id: i32,
        group: VehicleGroupUpdateRequest,
    ) -> Result<VehicleGroup, anyhow::Error>;

    /// 删除车组
    async fn delete(&self, group_id: i32) -> Result<(), anyhow::Error>;

    /// 检查车组是否有关联数据
    async fn has_related_data(&self, group_id: i32) -> Result<bool, anyhow::Error>;

    /// 获取车组树结构
    async fn get_tree(&self) -> Result<Vec<VehicleGroupTreeNode>, anyhow::Error>;

    /// 检查车组是否存在
    async fn exists(&self, group_id: i32) -> Result<bool, anyhow::Error>;

    /// 根据名称统计车组数量
    async fn count_by_name(
        &self,
        name: &str,
        exclude_id: Option<i32>,
    ) -> Result<i64, anyhow::Error>;

    /// 统计车组下的车辆数量
    async fn count_vehicles(&self, group_id: i32) -> Result<i64, anyhow::Error>;

    /// 统计子车组数量
    async fn count_children(&self, group_id: i32) -> Result<i64, anyhow::Error>;
}
