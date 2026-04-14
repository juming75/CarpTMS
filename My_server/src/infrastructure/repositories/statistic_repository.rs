//! 统计分析仓库实现

use std::sync::Arc;

use sqlx::PgPool;

use crate::domain::entities::statistic::{Statistic, StatisticUpdateRequest, StatisticQuery, StatisticSummary, StatisticTrendPoint};
use crate::domain::use_cases::statistic::StatisticRepository;
use crate::errors::{AppError, AppResult};

/// 统计分析仓库实现
#[derive(Clone)]
pub struct StatisticRepositoryImpl {
    db: Arc<PgPool>,
}

impl StatisticRepositoryImpl {
    /// 创建统计分析仓库实例
    pub fn new(db: Arc<PgPool>) -> Self {
        Self {
            db,
        }
    }
}

#[async_trait::async_trait]
impl StatisticRepository for StatisticRepositoryImpl {
    async fn create(&self, statistic: &Statistic) -> AppResult<Statistic> {
        let result = sqlx::query_as::<_, Statistic>(
            r#"INSERT INTO statistics (
                stat_type, 
                value, 
                unit, 
                category, 
                sub_category, 
                period, 
                period_start, 
                period_end, 
                create_time, 
                update_time
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING *"#
        )
        .bind(&statistic.stat_type)
        .bind(statistic.value)
        .bind(&statistic.unit)
        .bind(&statistic.category)
        .bind(&statistic.sub_category)
        .bind(&statistic.period)
        .bind(statistic.period_start)
        .bind(statistic.period_end)
        .bind(statistic.create_time)
        .bind(statistic.update_time)
        .fetch_one(&*self.db)
        .await
        .map_err(|e| AppError::db_error("Failed to create statistic", Some(&e.to_string())))?;

        Ok(result)
    }

    async fn get_by_id(&self, id: i32) -> AppResult<Option<Statistic>> {
        let statistic = sqlx::query_as::<_, Statistic>(
            r#"SELECT 
                id, 
                stat_type, 
                value, 
                unit, 
                category, 
                sub_category, 
                period, 
                period_start, 
                period_end, 
                create_time, 
                update_time
            FROM statistics 
            WHERE id = $1"#
        )
        .bind(id)
        .fetch_optional(&*self.db)
        .await
        .map_err(|e| AppError::db_error("Failed to get statistic by id", Some(&e.to_string())))?;

        Ok(statistic)
    }

    async fn get_by_query(&self, _query: &StatisticQuery) -> AppResult<(Vec<Statistic>, i64)> {
        // 暂时返回空数据，后续会实现完整的查询逻辑
        Ok((vec![], 0))
    }

    async fn update(&self, _id: i32, _statistic: &StatisticUpdateRequest) -> AppResult<Statistic> {
        // 暂时返回错误，后续会实现完整的更新逻辑
        Err(AppError::internal_error("Update method not implemented", None))
    }

    async fn delete(&self, id: i32) -> AppResult<bool> {
        let result = sqlx::query(
            "DELETE FROM statistics WHERE id = $1"
        )
        .bind(id)
        .execute(&*self.db)
        .await
        .map_err(|e| AppError::db_error("Failed to delete statistic", Some(&e.to_string())))?;

        Ok(result.rows_affected() > 0)
    }

    async fn get_summary(&self, query: &StatisticQuery) -> AppResult<StatisticSummary> {
        // 暂时返回默认数据，后续会实现完整的汇总逻辑
        let summary = StatisticSummary {
            total_count: 0,
            total_value: 0.0,
            average_value: 0.0,
            min_value: 0.0,
            max_value: 0.0,
            period: query.period.clone().unwrap_or("all".to_string()),
        };

        Ok(summary)
    }

    async fn get_trend(&self, _query: &StatisticQuery) -> AppResult<Vec<StatisticTrendPoint>> {
        // 暂时返回空数据，后续会实现完整的趋势查询逻辑
        Ok(vec![])
    }

    async fn batch_create(&self, statistics: &[Statistic]) -> AppResult<Vec<Statistic>> {
        if statistics.is_empty() {
            return Ok(vec![]);
        }

        // 构建批量插入查询
        let placeholders = statistics.iter().enumerate().map(|(i, _)| {
            let start = i * 10 + 1;
            format!("(${}, ${}, ${}, ${}, ${}, ${}, ${}, ${}, ${}, ${})", 
                start, start+1, start+2, start+3, start+4, start+5, start+6, start+7, start+8, start+9)
        }).collect::<Vec<_>>().join(", ");

        let query = format!(
            r#"INSERT INTO statistics (
                stat_type, 
                value, 
                unit, 
                category, 
                sub_category, 
                period, 
                period_start, 
                period_end, 
                create_time, 
                update_time
            ) VALUES {} 
            RETURNING *"#,
            placeholders
        );

        // 构建参数
        let mut query_builder = sqlx::query_as::<_, Statistic>(&query);
        for stat in statistics {
            query_builder = query_builder
                .bind(&stat.stat_type)
                .bind(stat.value)
                .bind(&stat.unit)
                .bind(&stat.category)
                .bind(&stat.sub_category)
                .bind(&stat.period)
                .bind(stat.period_start)
                .bind(stat.period_end)
                .bind(stat.create_time)
                .bind(stat.update_time);
        }

        let result = query_builder
            .fetch_all(&*self.db)
            .await
            .map_err(|e| AppError::db_error("Failed to batch create statistics", Some(&e.to_string())))?;

        Ok(result)
    }
}
