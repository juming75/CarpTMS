//! / 动态查询构建器
// 提供类型安全的动态查询构建

use sqlx::PgPool;
use anyhow::Result;
use std::collections::HashMap;

/// 查询条件
#[derive(Debug, Clone)]
pub enum QueryCondition {
    Equals(String, sqlx::types::JsonValue),
    NotEquals(String, sqlx::types::JsonValue),
    Like(String, String),
    In(String, Vec<sqlx::types::JsonValue>),
    GreaterThan(String, sqlx::types::JsonValue),
    LessThan(String, sqlx::types::JsonValue),
    Between(String, (sqlx::types::JsonValue, sqlx::types::JsonValue)),
    IsNull(String),
    IsNotNull(String),
}

/// 排序条件
#[derive(Debug, Clone)]
pub struct OrderBy {
    pub column: String,
    pub ascending: bool,
}

impl OrderBy {
    pub fn asc(column: &str) -> Self {
        Self {
            column: column.to_string(),
            ascending: true,
        }
    }

    pub fn desc(column: &str) -> Self {
        Self {
            column: column.to_string(),
            ascending: false,
        }
    }
}

/// 分页参数
#[derive(Debug, Clone)]
pub struct Pagination {
    pub offset: u32,
    pub limit: u32,
}

impl Pagination {
    pub fn new(page: u32, page_size: u32) -> Self {
        Self {
            offset: (page - 1) * page_size,
            limit: page_size,
        }
    }

    pub fn from_params(offset: u32, limit: u32) -> Self {
        Self { offset, limit }
    }

    pub fn no_pagination() -> Self {
        Self {
            offset: 0,
            limit: u32::MAX,
        }
    }
}

/// 动态查询构建器
pub struct QueryBuilder {
    table_name: String,
    columns: Vec<String>,
    conditions: Vec<QueryCondition>,
    order_by: Option<OrderBy>,
    pagination: Option<Pagination>,
    params: HashMap<String, sqlx::types::JsonValue>,
}

impl QueryBuilder {
    /// 创建新的查询构建器
    pub fn new(table_name: &str) -> Self {
        Self {
            table_name: table_name.to_string(),
            columns: vec!["*".to_string()],
            conditions: Vec::new(),
            order_by: None,
            pagination: None,
            params: HashMap::new(),
        }
    }

    /// 选择指定列
    pub fn select(mut self, columns: &[&str]) -> Self {
        self.columns = columns.iter().map(|c| c.to_string()).collect();
        self
    }

    /// 添加等于条件
    pub fn where_eq(mut self, column: &str, value: serde_json::Value) -> Self {
        self.params.insert(column.to_string(), value.clone());
        self.conditions.push(QueryCondition::Equals(column.to_string(), value));
        self
    }

    /// 添加不等于条件
    pub fn where_ne(mut self, column: &str, value: serde_json::Value) -> Self {
        self.params.insert(column.to_string(), value.clone());
        self.conditions.push(QueryCondition::NotEquals(column.to_string(), value));
        self
    }

    /// 添加 LIKE 条件
    pub fn where_like(mut self, column: &str, pattern: &str) -> Self {
        self.params.insert(column.to_string(), serde_json::json!(pattern));
        self.conditions.push(QueryCondition::Like(column.to_string(), pattern.to_string()));
        self
    }

    /// 添加 IN 条件
    pub fn where_in(mut self, column: &str, values: Vec<serde_json::Value>) -> Self {
        self.params.insert(column.to_string(), serde_json::json!(values));
        self.conditions.push(QueryCondition::In(column.to_string(), values));
        self
    }

    /// 添加大于条件
    pub fn where_gt(mut self, column: &str, value: serde_json::Value) -> Self {
        self.params.insert(column.to_string(), value.clone());
        self.conditions.push(QueryCondition::GreaterThan(column.to_string(), value));
        self
    }

    /// 添加小于条件
    pub fn where_lt(mut self, column: &str, value: serde_json::Value) -> Self {
        self.params.insert(column.to_string(), value.clone());
        self.conditions.push(QueryCondition::LessThan(column.to_string(), value));
        self
    }

    /// 添加 BETWEEN 条件
    pub fn where_between(mut self, column: &str, min: serde_json::Value, max: serde_json::Value) -> Self {
        self.params.insert(format!("{}_min", column), min.clone());
        self.params.insert(format!("{}_max", column), max.clone());
        self.conditions.push(QueryCondition::Between(column.to_string(), (min, max)));
        self
    }

    /// 添加 IS NULL 条件
    pub fn where_is_null(mut self, column: &str) -> Self {
        self.conditions.push(QueryCondition::IsNull(column.to_string()));
        self
    }

    /// 添加 IS NOT NULL 条件
    pub fn where_is_not_null(mut self, column: &str) -> Self {
        self.conditions.push(QueryCondition::IsNotNull(column.to_string()));
        self
    }

    /// 设置排序
    pub fn order_by(mut self, order: OrderBy) -> Self {
        self.order_by = Some(order);
        self
    }

    /// 设置分页
    pub fn paginate(mut self, pagination: Pagination) -> Self {
        self.pagination = Some(pagination);
        self
    }

    /// 构建SQL查询字符串
    pub fn build(&self) -> String {
        let mut sql = String::new();

        // SELECT 子句
        sql.push_str("SELECT ");
        sql.push_str(&self.columns.join(", "));
        sql.push_str(&format!(" FROM {}", self.table_name));

        // WHERE 子句
        if !self.conditions.is_empty() {
            sql.push_str(" WHERE ");
            for (i, condition) in self.conditions.iter().enumerate() {
                if i > 0 {
                    sql.push_str(" AND ");
                }
                sql.push_str(&self.condition_to_sql(condition));
            }
        }

        // ORDER BY 子句
        if let Some(ref order) = self.order_by {
            sql.push_str(&format!(
                " ORDER BY {} {}",
                order.column,
                if order.ascending { "ASC" } else { "DESC" }
            ));
        }

        // LIMIT/OFFSET 子句
        if let Some(ref pagination) = self.pagination {
            sql.push_str(&format!(" LIMIT {} OFFSET {}", pagination.limit, pagination.offset));
        }

        sql
    }

    /// 将条件转换为SQL
    fn condition_to_sql(&self, condition: &QueryCondition) -> String {
        match condition {
            QueryCondition::Equals(col, _) => format!("{} = ${}", col, col),
            QueryCondition::NotEquals(col, _) => format!("{} != ${}", col, col),
            QueryCondition::Like(col, _) => format!("{} LIKE ${}", col, col),
            QueryCondition::In(col, _) => format!("{} = ANY(${})", col, col),
            QueryCondition::GreaterThan(col, _) => format!("{} > ${}", col, col),
            QueryCondition::LessThan(col, _) => format!("{} < ${}", col, col),
            QueryCondition::Between(col, _) => {
                format!("{} BETWEEN ${}_min AND ${}_max", col, col, col)
            }
            QueryCondition::IsNull(col) => format!("{} IS NULL", col),
            QueryCondition::IsNotNull(col) => format!("{} IS NOT NULL", col),
        }
    }

    /// 执行查询(返回 Vec<T>)
    pub async fn fetch_all<T>(
        self,
        pool: &PgPool,
    ) -> Result<Vec<T>>
    where
        T: for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow> + Send + Unpin,
    {
        let sql = self.build();
        tracing::debug!("Executing dynamic query: {}", sql);
        
        let query = sqlx::query_as::<_, T>(&sql);
        let mut query = query;
        
        // 绑定参数
        for value in self.params.values() {
            // 简化处理:将 JsonValue 转换为字符串
            if let serde_json::Value::String(s) = value {
                query = query.bind(s);
            } else if let serde_json::Value::Number(n) = value {
                if let Some(i) = n.as_i64() {
                    query = query.bind(i);
                } else if let Some(f) = n.as_f64() {
                    query = query.bind(f);
                }
            } else if let serde_json::Value::Bool(b) = value {
                query = query.bind(*b);
            }
        }
        
        let results = query.fetch_all(pool).await?;
        Ok(results)
    }

    /// 执行查询(返回 Option<T>)
    pub async fn fetch_one<T>(
        self,
        pool: &PgPool,
    ) -> Result<Option<T>>
    where
        T: for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow> + Send + Unpin,
    {
        let sql = self.build();
        tracing::debug!("Executing dynamic query (one): {}", sql);
        
        let query = sqlx::query_as::<_, T>(&sql);
        let mut query = query;
        
        // 绑定参数(与 fetch_all 相同)
        for value in self.params.values() {
            if let serde_json::Value::String(s) = value {
                query = query.bind(s);
            } else if let serde_json::Value::Number(n) = value {
                if let Some(i) = n.as_i64() {
                    query = query.bind(i);
                } else if let Some(f) = n.as_f64() {
                    query = query.bind(f);
                }
            } else if let serde_json::Value::Bool(b) = value {
                query = query.bind(*b);
            }
        }
        
        let result = query.fetch_optional(pool).await?;
        Ok(result)
    }

    /// 执行查询(返回计数)
    pub async fn count(self, pool: &PgPool) -> Result<i64> {
        let mut sql = self.build();
        // 替换 SELECT ... 为 SELECT COUNT(*)
        sql = sql.replacen("SELECT *", "SELECT COUNT(*)", 1);
        sql = sql.replacen("SELECT", "SELECT COUNT(*)", 1);
        
        tracing::debug!("Executing count query: {}", sql);
        
        let query = sqlx::query_scalar::<_, i64>(&sql);
        let mut query = query;
        
        for value in self.params.values() {
            if let serde_json::Value::String(s) = value {
                query = query.bind(s);
            } else if let serde_json::Value::Number(n) = value {
                if let Some(i) = n.as_i64() {
                    query = query.bind(i);
                } else if let Some(f) = n.as_f64() {
                    query = query.bind(f);
                }
            } else if let serde_json::Value::Bool(b) = value {
                query = query.bind(*b);
            }
        }
        
        let count = query.fetch_one(pool).await?;
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_builder_basic() {
        let builder = QueryBuilder::new("users")
            .select(&["id", "name", "email"])
            .where_eq("status", serde_json::json!("active"))
            .order_by(OrderBy::desc("created_at"))
            .paginate(Pagination::new(1, 10));
        
        let sql = builder.build();
        assert!(sql.contains("SELECT id, name, email FROM users"));
        assert!(sql.contains("status = $status"));
        assert!(sql.contains("ORDER BY created_at DESC"));
        assert!(sql.contains("LIMIT 10 OFFSET 0"));
    }

    #[test]
    fn test_pagination() {
        let p1 = Pagination::new(2, 20); // 第2页,每页20条
        assert_eq!(p1.offset, 20);
        assert_eq!(p1.limit, 20);

        let p2 = Pagination::no_pagination();
        assert_eq!(p2.offset, 0);
        assert_eq!(p2.limit, u32::MAX);
    }

    #[test]
    fn test_order_by() {
        let asc = OrderBy::asc("created_at");
        assert_eq!(asc.column, "created_at");
        assert!(asc.ascending);

        let desc = OrderBy::desc("name");
        assert_eq!(desc.column, "name");
        assert!(!desc.ascending);
    }
}






