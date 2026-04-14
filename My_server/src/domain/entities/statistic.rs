//! 统计分析领域实体

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// 统计数据实体
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Statistic {
    pub id: i32,
    pub stat_type: String,
    pub value: f64,
    pub unit: String,
    pub category: Option<String>,
    pub sub_category: Option<String>,
    pub period: String, // 日、周、月、年
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub create_time: DateTime<Utc>,
    pub update_time: Option<DateTime<Utc>>,
}

/// 统计查询参数
#[derive(Debug, Clone, Deserialize)]
pub struct StatisticQuery {
    pub stat_type: Option<String>,
    pub category: Option<String>,
    pub sub_category: Option<String>,
    pub period: Option<String>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub page: Option<i32>,
    pub page_size: Option<i32>,
}

/// 统计数据创建请求
#[derive(Debug, Clone, Deserialize)]
pub struct StatisticCreateRequest {
    pub stat_type: String,
    pub value: f64,
    pub unit: String,
    pub category: Option<String>,
    pub sub_category: Option<String>,
    pub period: String,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
}

/// 统计数据更新请求
#[derive(Debug, Clone, Deserialize)]
pub struct StatisticUpdateRequest {
    pub value: Option<f64>,
    pub unit: Option<String>,
    pub category: Option<String>,
    pub sub_category: Option<String>,
}

/// 统计汇总结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticSummary {
    pub total_count: i64,
    pub total_value: f64,
    pub average_value: f64,
    pub min_value: f64,
    pub max_value: f64,
    pub period: String,
}

/// 统计趋势数据点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticTrendPoint {
    pub period: String,
    pub value: f64,
    pub date: DateTime<Utc>,
}

/// 统计数据创建参数
#[derive(Debug, Clone)]
pub struct StatisticNewParams {
    pub stat_type: String,
    pub value: f64,
    pub unit: String,
    pub category: Option<String>,
    pub sub_category: Option<String>,
    pub period: String,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
}

impl Statistic {
    /// 创建新统计数据
    pub fn new(params: StatisticNewParams) -> Self {
        Self {
            id: 0, // 数据库自动生成
            stat_type: params.stat_type,
            value: params.value,
            unit: params.unit,
            category: params.category,
            sub_category: params.sub_category,
            period: params.period,
            period_start: params.period_start,
            period_end: params.period_end,
            create_time: Utc::now(),
            update_time: None,
        }
    }

    /// 更新统计数据
    pub fn update(&mut self, request: &StatisticUpdateRequest) {
        if let Some(value) = request.value {
            self.value = value;
        }
        if let Some(unit) = &request.unit {
            self.unit = unit.clone();
        }
        if request.category.is_some() {
            self.category = request.category.clone();
        }
        if request.sub_category.is_some() {
            self.sub_category = request.sub_category.clone();
        }
        self.update_time = Some(Utc::now());
    }
}
