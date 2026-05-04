//! 统计分析用例

use std::sync::Arc;

use crate::domain::entities::statistic::{
    Statistic, StatisticCreateRequest, StatisticNewParams, StatisticQuery, StatisticSummary,
    StatisticTrendPoint, StatisticUpdateRequest,
};
use crate::errors::AppResult;

/// 统计分析仓库接口
#[async_trait::async_trait]
pub trait StatisticRepository: Send + Sync {
    /// 创建统计数据
    async fn create(&self, statistic: &Statistic) -> AppResult<Statistic>;

    /// 根据ID获取统计数据
    async fn get_by_id(&self, id: i32) -> AppResult<Option<Statistic>>;

    /// 根据查询参数获取统计数据列表
    async fn get_by_query(&self, query: &StatisticQuery) -> AppResult<(Vec<Statistic>, i64)>;

    /// 更新统计数据
    async fn update(&self, id: i32, statistic: &StatisticUpdateRequest) -> AppResult<Statistic>;

    /// 删除统计数据
    async fn delete(&self, id: i32) -> AppResult<bool>;

    /// 获取统计汇总数据
    async fn get_summary(&self, query: &StatisticQuery) -> AppResult<StatisticSummary>;

    /// 获取统计趋势数据
    async fn get_trend(&self, query: &StatisticQuery) -> AppResult<Vec<StatisticTrendPoint>>;

    /// 批量创建统计数据
    async fn batch_create(&self, statistics: &[Statistic]) -> AppResult<Vec<Statistic>>;
}

/// 统计分析用例
#[derive(Clone)]
pub struct StatisticUseCases {
    repository: Arc<dyn StatisticRepository + Send + Sync>,
}

impl StatisticUseCases {
    /// 创建统计分析用例
    pub fn new(repository: Arc<dyn StatisticRepository + Send + Sync>) -> Self {
        Self { repository }
    }

    /// 创建统计数据
    pub async fn create(&self, request: StatisticCreateRequest) -> AppResult<Statistic> {
        // 创建统计数据实体
        let params = StatisticNewParams {
            stat_type: request.stat_type,
            value: request.value,
            unit: request.unit,
            category: request.category,
            sub_category: request.sub_category,
            period: request.period,
            period_start: request.period_start,
            period_end: request.period_end,
        };
        let statistic = Statistic::new(params);

        // 保存到数据库
        self.repository.create(&statistic).await
    }

    /// 根据ID获取统计数据
    pub async fn get_by_id(&self, id: i32) -> AppResult<Option<Statistic>> {
        self.repository.get_by_id(id).await
    }

    /// 根据查询参数获取统计数据列表
    pub async fn get_by_query(&self, query: &StatisticQuery) -> AppResult<(Vec<Statistic>, i64)> {
        self.repository.get_by_query(query).await
    }

    /// 更新统计数据
    pub async fn update(&self, id: i32, request: StatisticUpdateRequest) -> AppResult<Statistic> {
        self.repository.update(id, &request).await
    }

    /// 删除统计数据
    pub async fn delete(&self, id: i32) -> AppResult<bool> {
        self.repository.delete(id).await
    }

    /// 获取统计汇总数据
    pub async fn get_summary(&self, query: &StatisticQuery) -> AppResult<StatisticSummary> {
        self.repository.get_summary(query).await
    }

    /// 获取统计趋势数据
    pub async fn get_trend(&self, query: &StatisticQuery) -> AppResult<Vec<StatisticTrendPoint>> {
        self.repository.get_trend(query).await
    }

    /// 批量创建统计数据
    pub async fn batch_create(
        &self,
        requests: &[StatisticCreateRequest],
    ) -> AppResult<Vec<Statistic>> {
        let statistics: Vec<Statistic> = requests
            .iter()
            .map(|request| {
                let params = StatisticNewParams {
                    stat_type: request.stat_type.clone(),
                    value: request.value,
                    unit: request.unit.clone(),
                    category: request.category.clone(),
                    sub_category: request.sub_category.clone(),
                    period: request.period.clone(),
                    period_start: request.period_start,
                    period_end: request.period_end,
                };
                Statistic::new(params)
            })
            .collect();

        self.repository.batch_create(&statistics).await
    }
}

/// 应用服务接口实现
#[async_trait::async_trait]
impl crate::domain::use_cases::application_service::ApplicationService for StatisticUseCases {
    fn name(&self) -> &str {
        "statistic_service"
    }

    fn initialize(&self) -> anyhow::Result<()> {
        // 初始化逻辑（如果需要）
        Ok(())
    }

    async fn execute(&self, _input: serde_json::Value) -> anyhow::Result<serde_json::Value> {
        // 通用执行方法（如果需要）
        Ok(serde_json::Value::Null)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::mock;

    mock! {
        pub StatisticRepositoryImpl {}
        #[async_trait::async_trait]
        impl StatisticRepository for StatisticRepositoryImpl {
            async fn create(&self, statistic: &Statistic) -> AppResult<Statistic>;
            async fn get_by_id(&self, id: i32) -> AppResult<Option<Statistic>>;
            async fn get_by_query(&self, query: &StatisticQuery) -> AppResult<(Vec<Statistic>, i64)>;
            async fn update(&self, id: i32, statistic: &StatisticUpdateRequest) -> AppResult<Statistic>;
            async fn delete(&self, id: i32) -> AppResult<bool>;
            async fn get_summary(&self, query: &StatisticQuery) -> AppResult<StatisticSummary>;
            async fn get_trend(&self, query: &StatisticQuery) -> AppResult<Vec<StatisticTrendPoint>>;
            async fn batch_create(&self, statistics: &[Statistic]) -> AppResult<Vec<Statistic>>;
        }
    }

    #[tokio::test]
    async fn test_create_statistic() -> Result<(), anyhow::Error> {
        let mut mock_repo = MockStatisticRepositoryImpl::new();

        let request = StatisticCreateRequest {
            stat_type: "weight".to_string(),
            value: 100.5,
            unit: "kg".to_string(),
            category: Some("truck".to_string()),
            sub_category: Some("heavy".to_string()),
            period: "day".to_string(),
            period_start: chrono::Utc::now(),
            period_end: chrono::Utc::now(),
        };

        let _expected_statistic = Statistic::new(StatisticNewParams {
            stat_type: "weight".to_string(),
            value: 100.5,
            unit: "kg".to_string(),
            category: Some("truck".to_string()),
            sub_category: Some("heavy".to_string()),
            period: "day".to_string(),
            period_start: request.period_start,
            period_end: request.period_end,
        });

        mock_repo.expect_create().returning(|stat| Ok(stat.clone()));

        let use_cases = StatisticUseCases::new(Arc::new(mock_repo));
        let result = use_cases.create(request).await;

        assert!(result.is_ok());
        let created_statistic = result.unwrap();
        assert_eq!(created_statistic.stat_type, "weight");
        assert_eq!(created_statistic.value, 100.5);

        Ok(())
    }
}
