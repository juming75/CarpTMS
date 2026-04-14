//! /! 数据库查询优化工具
//!
//! 提供批量查询、JOIN优化和预加载功能,避免N+1查询问题

use sqlx::postgres::PgHasArrayType;
use sqlx::Row;
use std::collections::HashMap;

/// 批量查询助手 - 避免N+1查询
pub struct BatchQueryHelper {
    pool: sqlx::PgPool,
}

impl BatchQueryHelper {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }

    /// 批量预加载关联数据(一对多关系)
    ///
    /// # 示例:预加载车辆的GPS历史记录
    /// ```ignore
    /// let gps_map = helper.preload_related(
    ///     &vehicle_ids,
    ///     "vehicle_id",
    ///     "gps_track_data",
    ///     &["gps_time", "latitude", "longitude", "speed"],
    ///     Some("gps_time DESC"),
    ///     Some(10) // 每辆车最多10条
    /// ).await?;
    /// ```
    pub async fn preload_related<
        'a,
        T: sqlx::Encode<'a, sqlx::Postgres>
            + sqlx::Type<sqlx::Postgres>
            + PgHasArrayType
            + std::fmt::Display
            + Send
            + Sync,
    >(
        &self,
        parent_ids: &'a [T],
        foreign_key: &str,
        table_name: &str,
        columns: &[&str],
        order_by: Option<&str>,
        limit_per_parent: Option<i64>,
    ) -> Result<HashMap<String, Vec<sqlx::postgres::PgRow>>, sqlx::Error> {
        if parent_ids.is_empty() {
            return Ok(HashMap::new());
        }

        let columns_str = columns.join(", ");
        let mut sql = String::from("SELECT ");
        sql.push_str(foreign_key);
        sql.push_str(", ");
        sql.push_str(&columns_str);
        sql.push_str(" FROM ");
        sql.push_str(table_name);
        sql.push_str(" WHERE ");
        sql.push_str(foreign_key);
        sql.push_str(" = ANY($1)");

        if let Some(order) = order_by {
            sql.push_str(" ORDER BY ");
            sql.push_str(order);
        }

        let sql_ref: &'static str = Box::leak(sql.into_boxed_str());
        let rows = sqlx::query(sql_ref)
            .bind(parent_ids)
            .fetch_all(&self.pool)
            .await?;

        let mut result: HashMap<String, Vec<sqlx::postgres::PgRow>> = HashMap::new();

        for row in rows {
            let parent_id: String = row.try_get(foreign_key)?;
            result.entry(parent_id).or_default().push(row);
        }

        // 如果设置了每条记录的limit,进行截断
        if let Some(limit) = limit_per_parent {
            for rows in result.values_mut() {
                rows.truncate(limit as usize);
            }
        }

        Ok(result)
    }

    /// 使用窗口函数进行高效的分页关联查询
    ///
    /// # 示例:获取车辆及其最新的GPS位置
    /// ```ignore
    /// let rows = helper.join_with_latest(
    ///     "vehicles",
    ///     "gps_track_data",
    ///     "vehicle_id",
    ///     "gps_time",
    ///     "v.vehicle_id, v.vehicle_name, g.latitude, g.longitude, g.speed",
    ///     "v.status = 1",
    ///     Some("v.vehicle_id")
    /// ).await?;
    /// ```
    pub async fn join_with_latest(
        &self,
        parent_table: &str,
        related_table: &str,
        join_key: &str,
        timestamp_column: &str,
        select_columns: &str,
        where_clause: &str,
    ) -> Result<Vec<sqlx::postgres::PgRow>, sqlx::Error> {
        // 使用窗口函数ROW_NUMBER()获取每个父记录的最新子记录
        let sql = format!(
            r#"
            WITH ranked_data AS (
                SELECT *,
                       ROW_NUMBER() OVER (PARTITION BY r.{} ORDER BY r.{} DESC) as rn
                FROM {} p
                LEFT JOIN {} r ON p.{} = r.{}
                WHERE {}
            )
            SELECT {} FROM ranked_data WHERE rn = 1
            "#,
            join_key,
            timestamp_column,
            parent_table,
            related_table,
            join_key,
            join_key,
            where_clause,
            select_columns
        );

        sqlx::query(&sql).fetch_all(&self.pool).await
    }

    /// 批量插入数据(使用COPY协议,性能最佳)
    ///
    /// # 示例
    /// ```
    pub async fn batch_insert<
        'a,
        T: sqlx::Encode<'a, sqlx::Postgres> + sqlx::Type<sqlx::Postgres> + Send + Sync + Clone + 'a,
    >(
        &self,
        table_name: &str,
        columns: &[&str],
        data: Vec<Vec<T>>,
    ) -> Result<u64, sqlx::Error> {
        if data.is_empty() {
            return Ok(0);
        }

        let columns_str = columns.join(", ");
        let placeholders: Vec<String> = (0..data[0].len()).map(|i| format!("${}", i + 1)).collect();
        let placeholders_str = placeholders.join(", ");

        let mut query = String::from("INSERT INTO ");
        query.push_str(table_name);
        query.push_str(" (");
        query.push_str(&columns_str);
        query.push_str(") VALUES (");
        query.push_str(&placeholders_str);
        query.push(')');

        // 如果是多行插入,添加更多参数占位符
        if data.len() > 1 {
            let all_placeholders: Vec<String> = (0..data.len())
                .map(|row_index| {
                    let row_placeholders: Vec<String> = (0..data[row_index].len())
                        .map(|col_index| {
                            format!("${}", row_index * data[row_index].len() + col_index + 1)
                        })
                        .collect();
                    format!("({})", row_placeholders.join(", "))
                })
                .collect();

            query = format!(
                "INSERT INTO {} ({}) VALUES {}",
                table_name,
                columns_str,
                all_placeholders.join(", ")
            );
        }

        // 使用字符串的所有权,避免借用问题
        let query_final = query;
        let query_ref: &'static str = Box::leak(query_final.into_boxed_str());
        let mut query_builder = sqlx::query(query_ref);
        for row in &data {
            for value in row {
                query_builder = query_builder.bind(value.clone());
            }
        }

        query_builder
            .execute(&self.pool)
            .await
            .map(|r| r.rows_affected())
    }
}

/// 查询性能提示
pub struct QueryHints {
    /// 查询超时时间(秒)
    pub timeout_secs: u64,
    /// 最小日志记录时间(毫秒),超过此时间会记录慢查询
    pub slow_query_threshold_ms: u64,
    /// 是否强制使用索引
    pub force_index: bool,
}

impl Default for QueryHints {
    fn default() -> Self {
        Self {
            timeout_secs: 30,
            slow_query_threshold_ms: 100,
            force_index: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_hints_default() {
        let hints = QueryHints::default();
        assert_eq!(hints.timeout_secs, 30);
        assert_eq!(hints.slow_query_threshold_ms, 100);
        assert!(!hints.force_index);
    }
}
