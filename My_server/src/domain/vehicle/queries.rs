// Vehicle CQRS 查询对象
// 定义车辆读操作查询

use chrono::NaiveDateTime;

/// 车辆统计查询
#[derive(Debug, Clone)]
pub struct VehicleStatsQuery {
    pub start_date: Option<NaiveDateTime>,
    pub end_date: Option<NaiveDateTime>,
    pub group_id: Option<i32>,
}

/// 车辆年检状态查询
#[derive(Debug, Clone)]
pub struct InspectionStatusQuery {
    /// 0: 全部, 1: 正常, 2: 即将过期(30天内), 3: 已过期
    pub status_filter: i32,
}

/// 车辆保险状态查询
#[derive(Debug, Clone)]
pub struct InsuranceStatusQuery {
    /// 0: 全部, 1: 正常, 2: 即将过期(30天内), 3: 已过期
    pub status_filter: i32,
}

/// 车辆逾期统计
#[derive(Debug, Clone)]
pub struct OverdueStatsQuery {
    /// 查询截止日期
    pub as_of_date: NaiveDateTime,
}

/// 车辆批量查询
#[derive(Debug, Clone)]
pub struct BatchVehicleQuery {
    pub vehicle_ids: Vec<i32>,
}
