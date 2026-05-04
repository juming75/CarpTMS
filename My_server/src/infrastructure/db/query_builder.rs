//! / SQL查询构建器 - 消除重复的SQL查询代码

/// 查询条件类型
#[derive(Debug, Clone)]
pub enum QueryCondition {
    /// 等于
    Equals(String, String),
    /// 不等于
    NotEquals(String, String),
    /// 模糊查询
    Like(String, String),
    /// 大于
    GreaterThan(String, String),
    /// 小于
    LessThan(String, String),
    /// 大于等于
    GreaterThanOrEqual(String, String),
    /// 小于等于
    LessThanOrEqual(String, String),
    /// BETWEEN
    Between(String, String, String),
    /// IN
    In(String, Vec<String>),
    /// IS NULL
    IsNull(String),
    /// IS NOT NULL
    IsNotNull(String),
}

/// 查询构建器
pub struct QueryBuilder<'a> {
    table_name: &'a str,
    select_fields: Option<Vec<&'a str>>,
    where_conditions: Vec<QueryCondition>,
    order_by: Option<(String, bool)>, // (field, asc)
    limit: Option<usize>,
    offset: Option<usize>,
    _params: Vec<String>,
}

impl<'a> QueryBuilder<'a> {
    /// 创建新的查询构建器
    pub fn new(table_name: &'a str) -> Self {
        Self {
            table_name,
            select_fields: None,
            where_conditions: Vec::new(),
            order_by: None,
            limit: None,
            offset: None,
            _params: Vec::new(),
        }
    }

    /// 设置查询字段
    pub fn select(mut self, fields: Vec<&'a str>) -> Self {
        self.select_fields = Some(fields);
        self
    }

    /// 添加等于条件
    pub fn where_eq(mut self, field: impl Into<String>, value: impl Into<String>) -> Self {
        self.where_conditions
            .push(QueryCondition::Equals(field.into(), value.into()));
        self
    }

    /// 添加不等于条件
    pub fn where_ne(mut self, field: impl Into<String>, value: impl Into<String>) -> Self {
        self.where_conditions
            .push(QueryCondition::NotEquals(field.into(), value.into()));
        self
    }

    /// 添加模糊查询条件
    pub fn where_like(mut self, field: impl Into<String>, value: impl Into<String>) -> Self {
        self.where_conditions
            .push(QueryCondition::Like(field.into(), value.into()));
        self
    }

    /// 添加大于条件
    pub fn where_gt(mut self, field: impl Into<String>, value: impl Into<String>) -> Self {
        self.where_conditions
            .push(QueryCondition::GreaterThan(field.into(), value.into()));
        self
    }

    /// 添加小于条件
    pub fn where_lt(mut self, field: impl Into<String>, value: impl Into<String>) -> Self {
        self.where_conditions
            .push(QueryCondition::LessThan(field.into(), value.into()));
        self
    }

    /// 添加大于等于条件
    pub fn where_gte(mut self, field: impl Into<String>, value: impl Into<String>) -> Self {
        self.where_conditions
            .push(QueryCondition::GreaterThanOrEqual(
                field.into(),
                value.into(),
            ));
        self
    }

    /// 添加小于等于条件
    pub fn where_lte(mut self, field: impl Into<String>, value: impl Into<String>) -> Self {
        self.where_conditions
            .push(QueryCondition::LessThanOrEqual(field.into(), value.into()));
        self
    }

    /// 添加BETWEEN条件
    pub fn where_between(
        mut self,
        field: impl Into<String>,
        start: impl Into<String>,
        end: impl Into<String>,
    ) -> Self {
        self.where_conditions.push(QueryCondition::Between(
            field.into(),
            start.into(),
            end.into(),
        ));
        self
    }

    /// 添加IN条件
    pub fn where_in(mut self, field: impl Into<String>, values: Vec<impl Into<String>>) -> Self {
        self.where_conditions.push(QueryCondition::In(
            field.into(),
            values.into_iter().map(|v| v.into()).collect(),
        ));
        self
    }

    /// 添加IS NULL条件
    pub fn where_is_null(mut self, field: impl Into<String>) -> Self {
        self.where_conditions
            .push(QueryCondition::IsNull(field.into()));
        self
    }

    /// 添加IS NOT NULL条件
    pub fn where_is_not_null(mut self, field: impl Into<String>) -> Self {
        self.where_conditions
            .push(QueryCondition::IsNotNull(field.into()));
        self
    }

    /// 添加可选等于条件
    pub fn where_eq_opt(self, field: impl Into<String>, value: Option<impl Into<String>>) -> Self {
        if let Some(v) = value {
            self.where_eq(field, v)
        } else {
            self
        }
    }

    /// 添加可选模糊查询条件
    pub fn where_like_opt(
        mut self,
        field: impl Into<String>,
        value: Option<impl Into<String>>,
    ) -> Self {
        if let Some(v) = value {
            let v_str = v.into();
            if !v_str.is_empty() {
                self.where_conditions
                    .push(QueryCondition::Like(field.into(), format!("%{}%", v_str)));
            }
        }
        self
    }

    /// 设置排序
    pub fn order_by(mut self, field: impl Into<String>, asc: bool) -> Self {
        self.order_by = Some((field.into(), asc));
        self
    }

    /// 设置LIMIT
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    /// 设置OFFSET
    pub fn offset(mut self, offset: usize) -> Self {
        self.offset = Some(offset);
        self
    }

    /// 构建查询SQL
    pub fn build_query(&mut self) -> (String, Vec<String>) {
        let fields = self
            .select_fields
            .as_ref()
            .map(|f| f.join(", "))
            .unwrap_or_else(|| "*".to_string());

        let mut sql = format!("SELECT {} FROM {}", fields, self.table_name);
        let mut params = Vec::new();
        let mut param_index = 1;

        // 构建WHERE子句
        if !self.where_conditions.is_empty() {
            let mut clauses = Vec::new();
            for condition in &self.where_conditions {
                let (clause, new_params) = self.build_condition(condition, param_index);
                clauses.push(clause);
                params.extend(new_params.clone());
                param_index += new_params.len();
            }
            sql.push_str(&format!(" WHERE {}", clauses.join(" AND ")));
        }

        // 构建ORDER BY子句
        if let Some((field, asc)) = &self.order_by {
            sql.push_str(&format!(
                " ORDER BY {} {}",
                field,
                if *asc { "ASC" } else { "DESC" }
            ));
        }

        // 构建LIMIT和OFFSET
        if let Some(limit) = self.limit {
            sql.push_str(&format!(" LIMIT ${}", param_index));
            params.push(limit.to_string());
            param_index += 1;
        }

        if let Some(offset) = self.offset {
            sql.push_str(&format!(" OFFSET ${}", param_index));
            params.push(offset.to_string());
        }

        (sql, params)
    }

    /// 构建COUNT查询SQL
    pub fn build_count_query(&mut self) -> (String, Vec<String>) {
        let mut sql = format!("SELECT COUNT(*) FROM {}", self.table_name);
        let mut params = Vec::new();
        let mut param_index = 1;

        // 构建WHERE子句(与查询条件相同)
        if !self.where_conditions.is_empty() {
            let mut clauses = Vec::new();
            for condition in &self.where_conditions {
                let (clause, new_params) = self.build_condition(condition, param_index);
                clauses.push(clause);
                params.extend(new_params.clone());
                param_index += new_params.len();
            }
            sql.push_str(&format!(" WHERE {}", clauses.join(" AND ")));
        }

        (sql, params)
    }

    /// 构建条件子句
    fn build_condition(
        &self,
        condition: &QueryCondition,
        start_index: usize,
    ) -> (String, Vec<String>) {
        match condition {
            QueryCondition::Equals(field, _) => (format!("{} = ${}", field, start_index), vec![]),
            QueryCondition::NotEquals(field, _) => {
                (format!("{} != ${}", field, start_index), vec![])
            }
            QueryCondition::Like(field, _) => (format!("{} LIKE ${}", field, start_index), vec![]),
            QueryCondition::GreaterThan(field, _) => {
                (format!("{} > ${}", field, start_index), vec![])
            }
            QueryCondition::LessThan(field, _) => (format!("{} < ${}", field, start_index), vec![]),
            QueryCondition::GreaterThanOrEqual(field, _) => {
                (format!("{} >= ${}", field, start_index), vec![])
            }
            QueryCondition::LessThanOrEqual(field, _) => {
                (format!("{} <= ${}", field, start_index), vec![])
            }
            QueryCondition::Between(field, _, _) => (
                format!(
                    "{} BETWEEN ${} AND ${}",
                    field,
                    start_index,
                    start_index + 1
                ),
                vec![],
            ),
            QueryCondition::In(field, values) => {
                let placeholders: Vec<String> = (0..values.len())
                    .map(|i| format!("${}", start_index + i))
                    .collect();
                (
                    format!("{} IN ({})", field, placeholders.join(", ")),
                    vec![],
                )
            }
            QueryCondition::IsNull(field) => (format!("{} IS NULL", field), vec![]),
            QueryCondition::IsNotNull(field) => (format!("{} IS NOT NULL", field), vec![]),
        }
    }

    /// 获取条件参数值
    pub fn get_condition_params(&self) -> Vec<String> {
        let mut params = Vec::new();
        for condition in &self.where_conditions {
            match condition {
                QueryCondition::Equals(_, v) => params.push(v.clone()),
                QueryCondition::NotEquals(_, v) => params.push(v.clone()),
                QueryCondition::Like(_, v) => params.push(v.clone()),
                QueryCondition::GreaterThan(_, v) => params.push(v.clone()),
                QueryCondition::LessThan(_, v) => params.push(v.clone()),
                QueryCondition::GreaterThanOrEqual(_, v) => params.push(v.clone()),
                QueryCondition::LessThanOrEqual(_, v) => params.push(v.clone()),
                QueryCondition::Between(_, start, end) => {
                    params.push(start.clone());
                    params.push(end.clone());
                }
                QueryCondition::In(_, values) => params.extend(values.clone()),
                QueryCondition::IsNull(_) | QueryCondition::IsNotNull(_) => {}
            }
        }
        params
    }

    /// 从WHERE条件构建参数
    pub fn extract_params(&self) -> Vec<String> {
        self.get_condition_params()
    }
}

/// 分页查询构建器
pub struct PagedQueryBuilder<'a> {
    builder: QueryBuilder<'a>,
}

impl<'a> PagedQueryBuilder<'a> {
    /// 创建分页查询构建器
    pub fn new(table_name: &'a str) -> Self {
        Self {
            builder: QueryBuilder::new(table_name),
        }
    }

    /// 添加查询条件(委托到内部builder)
    pub fn where_eq(mut self, field: impl Into<String>, value: impl Into<String>) -> Self {
        self.builder = self.builder.where_eq(field, value);
        self
    }

    pub fn where_like(mut self, field: impl Into<String>, value: impl Into<String>) -> Self {
        self.builder = self.builder.where_like(field, value);
        self
    }

    pub fn where_between(
        mut self,
        field: impl Into<String>,
        start: impl Into<String>,
        end: impl Into<String>,
    ) -> Self {
        self.builder = self.builder.where_between(field, start, end);
        self
    }

    pub fn where_eq_opt(
        mut self,
        field: impl Into<String>,
        value: Option<impl Into<String>>,
    ) -> Self {
        self.builder = self.builder.where_eq_opt(field, value);
        self
    }

    pub fn where_like_opt(
        mut self,
        field: impl Into<String>,
        value: Option<impl Into<String>>,
    ) -> Self {
        self.builder = self.builder.where_like_opt(field, value);
        self
    }

    /// 构建分页查询
    pub fn build_paged_query(mut self, page: usize, page_size: usize) -> (String, Vec<String>) {
        let offset = (page - 1) * page_size;
        self.builder = self.builder.limit(page_size).offset(offset);
        self.builder.build_query()
    }

    /// 构建COUNT查询
    pub fn build_count_query(mut self) -> (String, Vec<String>) {
        self.builder.build_count_query()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_select() {
        let mut builder = QueryBuilder::new("vehicles");
        let (sql, params) = builder.build_query();
        assert_eq!(sql, "SELECT * FROM vehicles");
        assert!(params.is_empty());
    }

    #[test]
    fn test_select_with_fields() {
        let mut builder = QueryBuilder::new("vehicles");
        builder = builder.select(vec!["id", "name", "status"]);
        let (sql, _params) = builder.build_query();
        assert_eq!(sql, "SELECT id, name, status FROM vehicles");
    }

    #[test]
    fn test_where_eq() {
        let mut builder = QueryBuilder::new("vehicles");
        builder = builder.where_eq("status", "1");
        let (sql, params) = builder.build_query();
        assert_eq!(sql, "SELECT * FROM vehicles WHERE status = $1");
        assert_eq!(params, vec!["1"]);
    }

    #[test]
    fn test_where_like() {
        let mut builder = QueryBuilder::new("vehicles");
        builder = builder.where_like("name", "test");
        let (sql, params) = builder.build_query();
        assert_eq!(sql, "SELECT * FROM vehicles WHERE name LIKE $1");
        assert_eq!(params, vec!["test"]);
    }

    #[test]
    fn test_where_between() {
        let mut builder = QueryBuilder::new("vehicles");
        builder = builder.where_between("created_at", "2024-01-01", "2024-12-31");
        let (sql, params) = builder.build_query();
        assert_eq!(
            sql,
            "SELECT * FROM vehicles WHERE created_at BETWEEN $1 AND $2"
        );
        assert_eq!(params, vec!["2024-01-01", "2024-12-31"]);
    }

    #[test]
    fn test_order_by_and_limit() {
        let mut builder = QueryBuilder::new("vehicles");
        builder = builder.order_by("id", false).limit(10).offset(20);
        let (sql, params) = builder.build_query();
        assert_eq!(
            sql,
            "SELECT * FROM vehicles ORDER BY id DESC LIMIT $1 OFFSET $2"
        );
        assert_eq!(params, vec!["10", "20"]);
    }

    #[test]
    fn test_complex_query() {
        let mut builder = QueryBuilder::new("vehicles");
        builder = builder
            .select(vec!["id", "name", "status"])
            .where_eq("status", "1")
            .where_like("name", "test")
            .order_by("id", true)
            .limit(10);
        let (sql, params) = builder.build_query();
        assert!(sql.contains("SELECT id, name, status FROM vehicles"));
        assert!(sql.contains("status = $1"));
        assert!(sql.contains("name LIKE $2"));
        assert!(sql.contains("ORDER BY id ASC"));
        assert!(sql.contains("LIMIT $3"));
        assert_eq!(params, vec!["1", "test", "10"]);
    }
}
