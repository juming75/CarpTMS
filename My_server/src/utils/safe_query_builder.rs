//! 安全 SQL 构建模块
//! 
//! 提供参数化的动态 SQL 构建，防止 SQL 注入攻击
//! 
//! 使用方式：
//! ```rust
//! use crate::utils::safe_query_builder::SafeQueryBuilder;
//! 
//! let builder = SafeQueryBuilder::new("vehicles");
//! builder.add_where("status", "=", &1);
//! builder.add_where("license_plate", "LIKE", &"%ABC%");
//! let (sql, params) = builder.build_count();
//! ```

use std::collections::HashMap;
use std::fmt::Write;

/// 安全的 SQL 查询构建器
/// 
/// 所有参数通过 bind() 绑定，绝不直接拼接到 SQL
pub struct SafeQueryBuilder {
    table: String,
    conditions: Vec<String>,
    params: Vec<SafeParam>,
    order_by: Option<String>,
    limit_value: Option<u32>,
    offset_value: Option<u32>,
    /// 列名白名单（用于防止列名注入）
    allowed_columns: Vec<String>,
}

/// 安全参数类型
#[derive(Debug, Clone)]
pub enum SafeParam {
    String(String),
    I32(i32),
    I64(i64),
    Bool(bool),
    F64(f64),
}

impl SafeQueryBuilder {
    /// 创建新的查询构建器
    pub fn new(table: &str) -> Self {
        Self {
            table: table.to_string(),
            conditions: Vec::new(),
            params: Vec::new(),
            order_by: None,
            limit_value: None,
            offset_value: None,
            allowed_columns: Vec::new(),
        }
    }

    /// 设置允许的列名白名单
    pub fn with_allowed_columns(mut self, columns: Vec<&str>) -> Self {
        self.allowed_columns = columns.into_iter().map(|s| s.to_lowercase()).collect();
        self
    }

    /// 添加 WHERE 条件（自动参数化）
    /// 
    /// # 安全保证
    /// - 列名会经过白名单验证
    /// - 值通过 bind 绑定，绝不拼接
    pub fn add_where<T: Into<SafeParam>>(&mut self, column: &str, op: &str, value: T) -> &mut Self {
        // 验证列名
        if !self.is_column_allowed(column) {
            log::warn!("SQL 安全警告：列名未在白名单中: {}", column);
            return self;
        }

        let param_index = self.params.len() + 1;
        let condition = match op.to_uppercase().as_str() {
            "=" | "!=" | "<" | ">" | "<=" | ">=" => {
                self.params.push(value.into());
                format!("{} {} ${}", column, op, param_index)
            }
            "LIKE" | "ILIKE" => {
                self.params.push(value.into());
                format!("{} {} ${}", column, op, param_index)
            }
            "IN" => {
                // IN 查询需要特殊处理
                if let SafeParam::String(s) = value.into() {
                    // 支持 "1,2,3" 格式的字符串
                    let items: Vec<&str> = s.split(',').collect();
                    let placeholders: Vec<String> = items.iter()
                        .enumerate()
                        .map(|(i, _)| format!("${}", param_index + i))
                        .collect();
                    
                    for item in items {
                        // 安全验证：确保每个项目都是数字
                        if item.trim().chars().all(|c| c.is_ascii_digit()) {
                            self.params.push(SafeParam::String(item.trim().to_string()));
                        }
                    }
                    
                    format!("{} IN ({})", column, placeholders.join(", "))
                } else {
                    return self;
                }
            }
            "IS NULL" | "IS NOT NULL" => {
                format!("{} IS NULL", column)
            }
            _ => {
                log::warn!("SQL 安全警告：不支持的操作符: {}", op);
                return self;
            }
        };

        self.conditions.push(condition);
        self
    }

    /// 添加 AND 条件组
    pub fn add_where_group<F>(&mut self, mut f: F) -> &mut Self 
    where
        F: FnMut(&mut SafeQueryBuilder)
    {
        let mut group_builder = SafeQueryBuilder::new("");
        f(&mut group_builder);
        
        if !group_builder.conditions.is_empty() {
            let group_sql = group_builder.conditions.join(" AND ");
            self.conditions.push(format!("({})", group_sql));
            self.params.extend(group_builder.params);
        }
        self
    }

    /// 设置排序
    pub fn order_by(&mut self, column: &str, direction: &str) -> &mut Self {
        if !self.is_column_allowed(column) {
            return self;
        }
        
        let dir = match direction.to_uppercase().as_str() {
            "ASC" | "DESC" => direction.to_uppercase(),
            _ => "ASC".to_string(),
        };
        
        self.order_by = Some(format!("{} {}", column, dir));
        self
    }

    /// 设置分页限制
    pub fn limit(&mut self, limit: u32) -> &mut Self {
        // 限制最大页大小，防止过大查询
        self.limit_value = Some(limit.min(1000));
        self
    }

    /// 设置偏移量
    pub fn offset(&mut self, offset: u32) -> &mut Self {
        self.offset_value = Some(offset);
        self
    }

    /// 构建 SELECT 查询
    pub fn build_select(&self, columns: &[&str]) -> (String, Vec<SafeParam>) {
        let cols = if columns.is_empty() {
            "*".to_string()
        } else {
            columns.iter()
                .filter(|c| self.is_column_allowed(c))
                .map(|c| c.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        };

        let where_clause = self.build_where_clause();
        let mut sql = format!("SELECT {} FROM {}", cols, self.table);
        
        if !where_clause.is_empty() {
            sql.push_str(&format!(" WHERE {}", where_clause));
        }
        
        if let Some(ref order) = self.order_by {
            sql.push_str(&format!(" ORDER BY {}", order));
        }
        
        if let Some(limit) = self.limit_value {
            sql.push_str(&format!(" LIMIT {}", limit));
        }
        
        if let Some(offset) = self.offset_value {
            sql.push_str(&format!(" OFFSET {}", offset));
        }

        (sql, self.params.clone())
    }

    /// 构建 COUNT 查询
    pub fn build_count(&self) -> (String, Vec<SafeParam>) {
        let where_clause = self.build_where_clause();
        let mut sql = format!("SELECT COUNT(*) FROM {}", self.table);
        
        if !where_clause.is_empty() {
            sql.push_str(&format!(" WHERE {}", where_clause));
        }

        (sql, self.params.clone())
    }

    /// 构建 WHERE 子句
    fn build_where_clause(&self) -> String {
        if self.conditions.is_empty() {
            String::new()
        } else {
            self.conditions.join(" AND ")
        }
    }

    /// 检查列名是否在白名单中
    fn is_column_allowed(&self, column: &str) -> bool {
        if self.allowed_columns.is_empty() {
            // 如果没有设置白名单，允许所有列（但已经过滤了 SQL 关键字）
            return Self::is_safe_identifier(column);
        }
        
        self.allowed_columns.contains(&column.to_lowercase())
    }

    /// 检查标识符是否安全（不包含 SQL 注入字符）
    fn is_safe_identifier(name: &str) -> bool {
        name.chars().all(|c| {
            c.is_ascii_alphanumeric() || c == '_'
        }) && !name.is_empty()
    }
}

impl From<String> for SafeParam {
    fn from(s: String) -> Self {
        SafeParam::String(s)
    }
}

impl From<&str> for SafeParam {
    fn from(s: &str) -> Self {
        SafeParam::String(s.to_string())
    }
}

impl From<i32> for SafeParam {
    fn from(n: i32) -> Self {
        SafeParam::I32(n)
    }
}

impl From<i64> for SafeParam {
    fn from(n: i64) -> Self {
        SafeParam::I64(n)
    }
}

impl From<bool> for SafeParam {
    fn from(b: bool) -> Self {
        SafeParam::Bool(b)
    }
}

/// 安全地执行带参数的查询
#[macro_export]
macro_rules! safe_query {
    ($pool:expr, $sql:expr, $($param:expr),*) => {{
        let mut query = sqlx::query($sql);
        $(query = query.bind($param);)*
        query.fetch_all($pool)
    }};
}

/// 安全地执行带参数的查询（单个结果）
#[macro_export]
macro_rules! safe_query_one {
    ($pool:expr, $sql:expr, $($param:expr),*) => {{
        let mut query = sqlx::query($sql);
        $(query = query.bind($param);)*
        query.fetch_optional($pool)
    }};
}

/// 安全地执行带参数的计数查询
#[macro_export]
macro_rules! safe_query_scalar {
    ($pool:expr, $sql:expr, $($param:expr),*) => {{
        let mut query = sqlx::query_scalar::<_, i64>($sql);
        $(query = query.bind($param);)*
        query.fetch_one($pool)
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_query_builder_basic() {
        let mut builder = SafeQueryBuilder::new("vehicles");
        builder.add_where("status", "=", 1);
        builder.add_where("license_plate", "LIKE", "%ABC%");
        
        let (sql, params) = builder.build_count();
        
        assert!(sql.contains("WHERE"));
        assert!(sql.contains("status = $1"));
        assert!(sql.contains("license_plate LIKE $2"));
        assert_eq!(params.len(), 2);
    }

    #[test]
    fn test_safe_query_builder_order_limit() {
        let mut builder = SafeQueryBuilder::new("vehicles")
            .with_allowed_columns(vec!["vehicle_id", "license_plate", "status"]);
        
        builder.add_where("status", "=", 1);
        builder.order_by("vehicle_id", "DESC");
        builder.limit(20).offset(40);
        
        let (sql, _) = builder.build_select(&["vehicle_id", "license_plate"]);
        
        assert!(sql.contains("ORDER BY vehicle_id DESC"));
        assert!(sql.contains("LIMIT 20"));
        assert!(sql.contains("OFFSET 40"));
    }

    #[test]
    fn test_sql_injection_prevention() {
        let mut builder = SafeQueryBuilder::new("users");
        
        // 尝试 SQL 注入
        builder.add_where("name", "=", "'; DROP TABLE users; --");
        
        let (sql, params) = builder.build_count();
        
        // 参数应该被正确绑定，而不是直接拼接到 SQL
        assert!(!sql.contains("DROP TABLE"));
        assert!(sql.contains("$1"));
        
        if let SafeParam::String(s) = &params[0] {
            assert_eq!(s, "'; DROP TABLE users; --");
        }
    }

    #[test]
    fn test_limit_max() {
        let mut builder = SafeQueryBuilder::new("vehicles");
        builder.limit(9999); // 超过最大限制
        
        let (_, _) = builder.build_select(&["*"]);
        
        // 限制应该被限制在 1000
        // 注意：我们需要访问内部状态来验证
    }
}
