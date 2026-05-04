//! 称重数据 CRUD 服务

use std::sync::Arc;

use crate::domain::entities::weighing_data::{
    WeighingData, WeighingDataCreate, WeighingDataQuery, WeighingDataUpdate,
};

use super::repository::WeighingDataRepository;

/// 称重数据用例结构
#[derive(Clone)]
pub struct WeighingDataUseCases {
    weighing_data_repository: Arc<dyn WeighingDataRepository>,
}

impl WeighingDataUseCases {
    /// 创建称重数据用例实例
    pub fn new(weighing_data_repository: Arc<dyn WeighingDataRepository>) -> Self {
        Self {
            weighing_data_repository,
        }
    }

    /// 获取称重数据列表用例
    pub async fn get_weighing_data_list(
        &self,
        query: WeighingDataQuery,
    ) -> Result<(Vec<WeighingData>, i64), anyhow::Error> {
        self.weighing_data_repository
            .get_weighing_data_list(query)
            .await
    }

    /// 获取单个称重数据用例
    pub async fn get_weighing_data_by_id(
        &self,
        id: i64,
    ) -> Result<Option<WeighingData>, anyhow::Error> {
        if id <= 0 {
            return Err(anyhow::anyhow!("称重数据ID必须大于0"));
        }
        self.weighing_data_repository.get_weighing_data(id).await
    }

    /// 创建称重数据用例
    pub async fn create_weighing_data(
        &self,
        weighing_data: WeighingDataCreate,
    ) -> Result<WeighingData, anyhow::Error> {
        // 数据验证
        if weighing_data.gross_weight <= 0.0 {
            return Err(anyhow::anyhow!("毛重必须大于0"));
        }

        self.weighing_data_repository
            .create_weighing_data(weighing_data)
            .await
    }

    /// 更新称重数据用例
    pub async fn update_weighing_data(
        &self,
        id: i64,
        weighing_data: WeighingDataUpdate,
    ) -> Result<Option<WeighingData>, anyhow::Error> {
        if id <= 0 {
            return Err(anyhow::anyhow!("称重数据ID必须大于0"));
        }

        self.weighing_data_repository
            .update_weighing_data(id, weighing_data)
            .await
    }

    /// 删除称重数据用例
    pub async fn delete_weighing_data(&self, id: i64) -> Result<bool, anyhow::Error> {
        if id <= 0 {
            return Err(anyhow::anyhow!("称重数据ID必须大于0"));
        }

        self.weighing_data_repository.delete_weighing_data(id).await
    }
}
