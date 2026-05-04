//! / SQL查询执行器 - 封装常见的数据库查询操作
use anyhow::Result;
use sqlx::{Column, PgPool, Row};
use std::sync::Arc;

use super::query_builder::{PagedQueryBuilder, QueryBuilder};

/// 分页查询结果
pub struct PagedResult<T> {
    pub data: Vec<T>,
    pub total: i64,
}

/// 查询执行器 - 提供数据库查询的便捷方法
pub struct QueryExecutor {
    pool: Arc<PgPool>,
}

impl QueryExecutor {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    /// 获取数据库连接池
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// 执行分页查询
    pub async fn query_paged(
        &self,
        table_name: &str,
        page: usize,
        page_size: usize,
        conditions: Vec<(&str, &str)>,
        _order_by: Option<(&str, bool)>,
    ) -> Result<PagedResult<serde_json::Value>> {
        let mut count_builder = PagedQueryBuilder::new(table_name);
        let mut query_builder = PagedQueryBuilder::new(table_name);

        for (field, value) in &conditions {
            count_builder = count_builder.where_eq(field.to_string(), value.to_string());
            query_builder = query_builder.where_eq(field.to_string(), value.to_string());
        }

        let (count_sql, count_params) = count_builder.build_count_query();
        let (query_sql, params) = query_builder.build_paged_query(page, page_size);

        let total = self.execute_count_query(&count_sql, &count_params).await?;
        let data = self.execute_query(&query_sql, &params).await?;

        Ok(PagedResult { data, total })
    }

    /// 执行查询并返回JSON结果
    pub async fn execute_query(
        &self,
        sql: &str,
        params: &[String],
    ) -> Result<Vec<serde_json::Value>> {
        let mut query = sqlx::query(sql);

        for param in params {
            query = query.bind(param.as_str());
        }

        let rows = query.fetch_all(&*self.pool).await?;

        Ok(rows
            .into_iter()
            .map(|row| {
                let mut json = serde_json::Map::new();
                for column in row.columns() {
                    let name = column.name();
                    if let Ok(value) = row.try_get::<String, _>(name) {
                        json.insert(name.to_string(), serde_json::Value::String(value));
                    } else if let Ok(value) = row.try_get::<i32, _>(name) {
                        json.insert(name.to_string(), serde_json::Value::Number(value.into()));
                    } else if let Ok(value) = row.try_get::<i64, _>(name) {
                        json.insert(name.to_string(), serde_json::Value::Number(value.into()));
                    } else if let Ok(value) = row.try_get::<f64, _>(name) {
                        json.insert(name.to_string(), serde_json::json!(value));
                    }
                }
                serde_json::Value::Object(json)
            })
            .collect())
    }

    /// 执行COUNT查询
    pub async fn execute_count_query(&self, sql: &str, params: &[String]) -> Result<i64> {
        let mut query = sqlx::query_scalar::<_, i64>(sql);

        for param in params {
            query = query.bind(param.as_str());
        }

        let count = query.fetch_one(&*self.pool).await?;
        Ok(count)
    }

    /// 执行单行查询
    pub async fn query_one(
        &self,
        sql: &str,
        params: &[String],
    ) -> Result<Option<serde_json::Value>> {
        let results = self.execute_query(sql, params).await?;
        Ok(results.into_iter().next())
    }

    /// 执行INSERT操作
    pub async fn execute_insert(&self, sql: &str, params: &[String]) -> Result<()> {
        let mut query = sqlx::query(sql);

        for param in params {
            query = query.bind(param.as_str());
        }

        query.execute(&*self.pool).await?;
        Ok(())
    }

    /// 执行UPDATE操作
    pub async fn execute_update(&self, sql: &str, params: &[String]) -> Result<()> {
        let mut query = sqlx::query(sql);

        for param in params {
            query = query.bind(param.as_str());
        }

        query.execute(&*self.pool).await?;
        Ok(())
    }

    /// 执行DELETE操作
    pub async fn execute_delete(&self, sql: &str, params: &[String]) -> Result<u64> {
        let mut query = sqlx::query(sql);

        for param in params {
            query = query.bind(param.as_str());
        }

        let result = query.execute(&*self.pool).await?;
        Ok(result.rows_affected())
    }
}

/// 便捷函数:为给定表构建分页查询
pub fn build_paged_query(
    table_name: &str,
    page: usize,
    page_size: usize,
    conditions: Vec<(&str, &str)>,
) -> (String, Vec<String>, String, Vec<String>) {
    let mut count_builder = PagedQueryBuilder::new(table_name);
    let mut query_builder = PagedQueryBuilder::new(table_name);

    for (field, value) in &conditions {
        count_builder = count_builder.where_eq(field.to_string(), value.to_string());
        query_builder = query_builder.where_eq(field.to_string(), value.to_string());
    }

    let (count_sql, count_params) = count_builder.build_count_query();
    let (query_sql, query_params) = query_builder.build_paged_query(page, page_size);

    (query_sql, query_params, count_sql, count_params)
}

/// 便捷函数:构建带条件的查询
pub fn build_query_with_conditions(
    table_name: &str,
    conditions: Vec<(&str, &str)>,
    order_by: Option<(&str, bool)>,
    limit: Option<usize>,
    offset: Option<usize>,
) -> (String, Vec<String>) {
    let mut builder = QueryBuilder::new(table_name);

    for (field, value) in conditions {
        builder = builder.where_eq(field, value);
    }

    if let Some((field, asc)) = order_by {
        builder = builder.order_by(field, asc);
    }

    if let Some(limit) = limit {
        builder = builder.limit(limit);
    }

    if let Some(offset) = offset {
        builder = builder.offset(offset);
    }

    builder.build_query()
}
