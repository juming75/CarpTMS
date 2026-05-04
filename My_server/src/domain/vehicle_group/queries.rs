// VehicleGroup CQRS 查询对象
// 定义车组读操作查询

/// 车组树查询
#[derive(Debug, Clone)]
pub struct VehicleGroupTreeQuery {
    pub include_vehicle_count: bool,
}

/// 车组统计查询
#[derive(Debug, Clone)]
pub struct VehicleGroupStatsQuery {
    pub group_id: Option<i32>,
    pub include_children: bool,
}

/// 子车组查询
#[derive(Debug, Clone)]
pub struct ChildGroupsQuery {
    pub parent_id: Option<i32>,
    pub recursive: bool,
}
