use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgPoolOptions, PgPool, Row};
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::RwLock;
use tracing::{error, info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryOptimizationConfig {
    pub max_connections: u32,
    pub min_connections: u32,
    pub connection_timeout: Duration,
    pub idle_timeout: Duration,
    pub max_lifetime: Duration,
    pub slow_query_threshold: Duration,
    pub enable_query_analysis: bool,
    pub enable_cache: bool,
    pub cache_ttl: Duration,
    pub batch_size: usize,
    pub enable_pagination: bool,
    pub max_page_size: usize,
}

impl Default for QueryOptimizationConfig {
    fn default() -> Self {
        Self {
            max_connections: 20,
            min_connections: 5,
            connection_timeout: Duration::from_secs(30),
            idle_timeout: Duration::from_secs(600),
            max_lifetime: Duration::from_secs(1800),
            slow_query_threshold: Duration::from_secs(1),
            enable_query_analysis: true,
            enable_cache: true,
            cache_ttl: Duration::from_secs(300),
            batch_size: 1000,
            enable_pagination: true,
            max_page_size: 100,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryMetrics {
    pub query_id: String,
    pub execution_time: Duration,
    pub rows_affected: i64,
    pub timestamp: u64,
    pub is_slow: bool,
    pub query_plan: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryPlanAnalysis {
    pub query: String,
    pub plan: String,
    pub execution_time: Duration,
    pub total_cost: f64,
    pub startup_cost: f64,
    pub rows_returned: i64,
    pub index_usage: Vec<String>,
    pub optimization_suggestions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationResult<T> {
    pub data: Vec<T>,
    pub total_count: i64,
    pub page: usize,
    pub page_size: usize,
    pub total_pages: usize,
    pub has_next: bool,
    pub has_previous: bool,
}

pub struct QueryOptimizer {
    config: QueryOptimizationConfig,
    pool: PgPool,
    query_cache: Arc<RwLock<HashMap<String, CachedQueryResult>>>,
    metrics_store: Arc<RwLock<Vec<QueryMetrics>>>,
    slow_queries: Arc<RwLock<HashMap<String, SlowQueryInfo>>>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct CachedQueryResult {
    data: Vec<u8>,
    timestamp: Instant,
    hits: u64,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct SlowQueryInfo {
    query: String,
    avg_execution_time: Duration,
    execution_count: u64,
    last_executed: Instant,
}

#[allow(dead_code)]
impl QueryOptimizer {
    pub async fn new(
        database_url: &str,
        config: QueryOptimizationConfig,
    ) -> Result<Self, sqlx::Error> {
        let pool = PgPoolOptions::new()
            .max_connections(config.max_connections)
            .min_connections(config.min_connections)
            .acquire_timeout(config.connection_timeout)
            .idle_timeout(config.idle_timeout)
            .max_lifetime(config.max_lifetime)
            .connect(database_url)
            .await?;

        Ok(Self {
            config,
            pool,
            query_cache: Arc::new(RwLock::new(HashMap::new())),
            metrics_store: Arc::new(RwLock::new(Vec::new())),
            slow_queries: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    pub async fn execute_optimized_query<T>(
        &self,
        query: &str,
        parameters: &[&str],
    ) -> Result<Vec<T>, QueryOptimizationError>
    where
        T: for<'r> FromRow<'r, sqlx::postgres::PgRow> + Send + Unpin + 'static,
    {
        let start_time = Instant::now();

        // Execute query
        let mut query_builder = sqlx::query_as::<_, T>(query);
        for param in parameters {
            query_builder = query_builder.bind(param);
        }
        let result = query_builder
            .fetch_all(&self.pool)
            .await
            .map_err(QueryOptimizationError::DatabaseError)?;

        let execution_time = start_time.elapsed();

        // Update metrics
        self.record_query_metrics(query, execution_time, result.len() as i64)
            .await;

        Ok(result)
    }

    pub async fn execute_paginated_query<T>(
        &self,
        query: &str,
        parameters: &[&str],
        page: usize,
        page_size: usize,
    ) -> Result<PaginationResult<T>, QueryOptimizationError>
    where
        T: for<'r> FromRow<'r, sqlx::postgres::PgRow> + Send + Unpin + 'static,
    {
        if !self.config.enable_pagination {
            return Err(QueryOptimizationError::PaginationDisabled);
        }

        let effective_page_size = page_size.min(self.config.max_page_size).max(1);
        let offset = (page.saturating_sub(1)) * effective_page_size;

        // Get total count
        let count_query = format!("SELECT COUNT(*) FROM ({}) AS count_query", query);
        let total_count: i64 = sqlx::query_scalar(&count_query)
            .fetch_one(&self.pool)
            .await
            .map_err(QueryOptimizationError::DatabaseError)?;

        // Get paginated data
        let paginated_query = format!("{} LIMIT {} OFFSET {}", query, effective_page_size, offset);

        let data = self
            .execute_optimized_query(&paginated_query, parameters)
            .await?;

        let total_pages = ((total_count as f64) / (effective_page_size as f64)).ceil() as usize;

        Ok(PaginationResult {
            data,
            total_count,
            page,
            page_size: effective_page_size,
            total_pages,
            has_next: page < total_pages,
            has_previous: page > 1,
        })
    }

    /// 批量插入（未使用 - 注意：无事务回滚，部分成功时数据不一致风险）
    #[allow(dead_code)]
    pub async fn execute_batch_insert<T>(
        &self,
        table_name: &str,
        columns: &[&str],
        values: &[Vec<&str>],
    ) -> Result<u64, QueryOptimizationError> {
        if values.is_empty() {
            return Ok(0);
        }

        let mut total_inserted = 0u64;

        for chunk in values.chunks(self.config.batch_size) {
            let placeholders: Vec<String> = chunk
                .iter()
                .map(|row| {
                    format!(
                        "({})",
                        row.iter()
                            .enumerate()
                            .map(|(i, _)| format!("${}", i + 1))
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                })
                .collect();

            let query = format!(
                "INSERT INTO {} ({}) VALUES {}",
                table_name,
                columns.join(", "),
                placeholders.join(", ")
            );

            let mut query_builder = sqlx::query(&query);

            for row in chunk {
                for value in row {
                    query_builder = query_builder.bind(value);
                }
            }

            let result = query_builder
                .execute(&self.pool)
                .await
                .map_err(QueryOptimizationError::DatabaseError)?;

            total_inserted += result.rows_affected();
        }

        info!(
            "Batch insert completed: {} rows inserted into {}",
            total_inserted, table_name
        );
        Ok(total_inserted)
    }

    pub async fn analyze_query_performance(
        &self,
        query: &str,
    ) -> Result<QueryPlanAnalysis, QueryOptimizationError> {
        if !self.config.enable_query_analysis {
            return Err(QueryOptimizationError::QueryAnalysisDisabled);
        }

        let explain_query = format!("EXPLAIN (ANALYZE, BUFFERS, FORMAT JSON) {}", query);

        let start_time = Instant::now();

        let plan_result: serde_json::Value = sqlx::query_scalar(&explain_query)
            .fetch_one(&self.pool)
            .await
            .map_err(QueryOptimizationError::DatabaseError)?;

        let execution_time = start_time.elapsed();

        let _plan = serde_json::to_string_pretty(&plan_result)
            .map_err(|e| QueryOptimizationError::SerializationError(e.to_string()))?;

        // Parse PostgreSQL query plan
        let analysis = self.parse_query_plan(&plan_result, execution_time)?;

        Ok(analysis)
    }

    pub async fn get_query_statistics(&self) -> QueryStatistics {
        let metrics = self.metrics_store.read().await;
        let slow_queries = self.slow_queries.read().await;

        let total_queries = metrics.len() as u64;
        let slow_query_count = metrics.iter().filter(|m| m.is_slow).count() as u64;
        let avg_execution_time = if total_queries > 0 {
            metrics
                .iter()
                .map(|m| m.execution_time.as_millis())
                .sum::<u128>() as f64
                / total_queries as f64
        } else {
            0.0
        };

        QueryStatistics {
            total_queries,
            slow_query_count,
            avg_execution_time,
            cache_hit_rate: self.calculate_cache_hit_rate().await,
            slow_queries: slow_queries.len() as u64,
            top_slow_queries: self.get_top_slow_queries(10).await,
        }
    }

    pub async fn optimize_table(
        &self,
        table_name: &str,
    ) -> Result<Vec<String>, QueryOptimizationError> {
        let analysis_query = "SELECT 
                schemaname,
                tablename,
                attname,
                n_distinct,
                correlation,
                avg_width,
                null_frac
            FROM pg_stats 
            WHERE tablename = $1";

        let stats = sqlx::query(analysis_query)
            .bind(table_name)
            .fetch_all(&self.pool)
            .await
            .map_err(QueryOptimizationError::DatabaseError)?;

        let mut suggestions = Vec::new();

        // Analyze table statistics and suggest optimizations
        for row in stats {
            let column_name: String = row.get("attname");
            let null_frac: f64 = row.get("null_frac");
            let n_distinct: f64 = row.get("n_distinct");

            // Suggest index for columns with high selectivity
            if null_frac < 0.1 && n_distinct > 100.0 {
                suggestions.push(format!(
                    "Consider creating index on {}.{} for better query performance",
                    table_name, column_name
                ));
            }
        }

        // Check for missing indexes on foreign keys
        let fk_query = "SELECT 
            tc.constraint_name,
            kcu.column_name,
            ccu.table_name AS foreign_table_name,
            ccu.column_name AS foreign_column_name 
        FROM information_schema.table_constraints AS tc 
        JOIN information_schema.key_column_usage AS kcu
            ON tc.constraint_name = kcu.constraint_name
        JOIN information_schema.constraint_column_usage AS ccu
            ON ccu.constraint_name = tc.constraint_name
        WHERE tc.constraint_type = 'FOREIGN KEY' 
            AND tc.table_name = $1";

        let foreign_keys = sqlx::query(fk_query)
            .bind(table_name)
            .fetch_all(&self.pool)
            .await
            .map_err(QueryOptimizationError::DatabaseError)?;

        for row in foreign_keys {
            let column_name: String = row.get("column_name");
            suggestions.push(format!(
                "Consider creating index on {}.{} for foreign key performance",
                table_name, column_name
            ));
        }

        Ok(suggestions)
    }

    // Private helper methods
    async fn get_cached_result<T>(&self, query_id: &str) -> Option<Vec<T>>
    where
        T: serde::de::DeserializeOwned,
    {
        let cache = self.query_cache.read().await;

        if let Some(cached) = cache.get(query_id) {
            if cached.timestamp.elapsed() < self.config.cache_ttl {
                match serde_json::from_slice(&cached.data) {
                    Ok(data) => {
                        // Update hit count
                        drop(cache);
                        let mut cache = self.query_cache.write().await;
                        if let Some(cached) = cache.get_mut(query_id) {
                            cached.hits += 1;
                        }
                        return Some(data);
                    }
                    Err(e) => {
                        warn!("Failed to deserialize cached query result: {}", e);
                    }
                }
            }
        }

        None
    }

    async fn cache_result<T>(&self, query_id: &str, result: &[T])
    where
        T: serde::Serialize,
    {
        match serde_json::to_vec(result) {
            Ok(serialized_data) => {
                let mut cache = self.query_cache.write().await;
                cache.insert(
                    query_id.to_string(),
                    CachedQueryResult {
                        data: serialized_data,
                        timestamp: Instant::now(),
                        hits: 1,
                    },
                );

                // Cleanup old cache entries
                self.cleanup_cache().await;
            }
            Err(e) => {
                error!("Failed to serialize query result for caching: {}", e);
            }
        }
    }

    async fn cleanup_cache(&self) {
        let mut cache = self.query_cache.write().await;
        let now = Instant::now();

        cache.retain(|_, cached| now.duration_since(cached.timestamp) < self.config.cache_ttl);
    }

    async fn calculate_cache_hit_rate(&self) -> f64 {
        let cache = self.query_cache.read().await;
        let total_hits: u64 = cache.values().map(|cached| cached.hits).sum();
        let total_entries = cache.len() as u64;

        if total_entries > 0 {
            total_hits as f64 / total_entries as f64
        } else {
            0.0
        }
    }

    fn generate_query_id(&self, query: &str, parameters: &[&str]) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        query.hash(&mut hasher);
        parameters.hash(&mut hasher);

        format!("{:x}", hasher.finish())
    }

    async fn record_query_metrics(
        &self,
        query_id: &str,
        execution_time: Duration,
        rows_affected: i64,
    ) {
        let is_slow = execution_time > self.config.slow_query_threshold;

        let metrics = QueryMetrics {
            query_id: query_id.to_string(),
            execution_time,
            rows_affected,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system time always after epoch")
                .as_secs(),
            is_slow,
            query_plan: None,
        };

        let mut metrics_store = self.metrics_store.write().await;
        metrics_store.push(metrics);

        // Keep only last 10000 metrics
        if metrics_store.len() > 10000 {
            let new_start = metrics_store.len() - 10000;
            let new_metrics: Vec<QueryMetrics> = metrics_store.drain(new_start..).collect();
            *metrics_store = new_metrics;
        }
    }

    async fn analyze_slow_query(
        &self,
        query: &str,
        execution_time: Duration,
    ) -> Result<(), QueryOptimizationError> {
        info!(
            "Analyzing slow query ({}ms): {}",
            execution_time.as_millis(),
            query
        );

        if let Ok(analysis) = self.analyze_query_performance(query).await {
            warn!(
                "Slow query analysis: {:?}",
                analysis.optimization_suggestions
            );
        }

        let mut slow_queries = self.slow_queries.write().await;

        if let Some(existing) = slow_queries.get_mut(query) {
            existing.execution_count += 1;
            let total_time_ms = existing.avg_execution_time.as_millis()
                * (existing.execution_count - 1) as u128
                + execution_time.as_millis();
            existing.avg_execution_time =
                Duration::from_millis((total_time_ms / existing.execution_count as u128) as u64);
            existing.last_executed = Instant::now();
        } else {
            slow_queries.insert(
                query.to_string(),
                SlowQueryInfo {
                    query: query.to_string(),
                    avg_execution_time: execution_time,
                    execution_count: 1,
                    last_executed: Instant::now(),
                },
            );
        }

        Ok(())
    }

    async fn get_top_slow_queries(&self, limit: usize) -> Vec<(String, Duration, u64)> {
        let slow_queries = self.slow_queries.read().await;
        let mut queries: Vec<_> = slow_queries
            .values()
            .map(|info| {
                (
                    info.query.clone(),
                    info.avg_execution_time,
                    info.execution_count,
                )
            })
            .collect();

        queries.sort_by(|a, b| b.1.cmp(&a.1));
        queries.into_iter().take(limit).collect()
    }

    fn parse_query_plan(
        &self,
        plan_json: &serde_json::Value,
        execution_time: Duration,
    ) -> Result<QueryPlanAnalysis, QueryOptimizationError> {
        // Extract plan details from PostgreSQL EXPLAIN output
        let plan_array = plan_json
            .as_array()
            .and_then(|arr| arr.first())
            .ok_or_else(|| {
                QueryOptimizationError::PlanParsingError("Invalid plan format".to_string())
            })?;

        let plan_string = serde_json::to_string_pretty(plan_json)
            .map_err(|e| QueryOptimizationError::SerializationError(e.to_string()))?;

        let total_cost = plan_array["Plan"]["Total Cost"].as_f64().unwrap_or(0.0);

        let startup_cost = plan_array["Planning Time"].as_f64().unwrap_or(0.0);

        let rows_returned = plan_array["Plan"]["Actual Rows"].as_i64().unwrap_or(0);

        // Extract index usage information
        let mut index_usage = Vec::new();
        if let Some(plan_node) = plan_array["Plan"].as_object() {
            self.extract_index_usage(plan_node, &mut index_usage);
        }

        let optimization_suggestions =
            self.generate_optimization_suggestions(&plan_string, total_cost, rows_returned);

        Ok(QueryPlanAnalysis {
            query: "Query analysis".to_string(), // This would be passed in a real implementation
            plan: plan_string,
            execution_time,
            total_cost,
            startup_cost,
            rows_returned,
            index_usage,
            optimization_suggestions,
        })
    }

    fn extract_index_usage(
        &self,
        plan_node: &serde_json::Map<String, serde_json::Value>,
        index_usage: &mut Vec<String>,
    ) {
        if let Some(node_type) = plan_node["Node Type"].as_str() {
            if node_type.contains("Index") {
                if let Some(index_name) = plan_node["Index Name"].as_str() {
                    index_usage.push(index_name.to_string());
                }
            }
        }

        // Recursively check child nodes
        if let Some(plans) = plan_node["Plans"].as_array() {
            for child_plan in plans {
                if let Some(child_obj) = child_plan.as_object() {
                    self.extract_index_usage(child_obj, index_usage);
                }
            }
        }
    }

    fn generate_optimization_suggestions(
        &self,
        plan: &str,
        total_cost: f64,
        rows_returned: i64,
    ) -> Vec<String> {
        let mut suggestions = Vec::new();

        if total_cost > 1000.0 {
            suggestions
                .push("High query cost detected. Consider adding appropriate indexes.".to_string());
        }

        if rows_returned > 10000 {
            suggestions.push(
                "Large result set. Consider implementing pagination or filtering.".to_string(),
            );
        }

        if plan.contains("Seq Scan") && !plan.contains("Index") {
            suggestions.push(
                "Sequential scan detected. Consider adding indexes on frequently queried columns."
                    .to_string(),
            );
        }

        if plan.contains("Nested Loop") && total_cost > 500.0 {
            suggestions.push("Expensive nested loop join. Consider optimizing join conditions or adding indexes.".to_string());
        }

        suggestions
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryStatistics {
    pub total_queries: u64,
    pub slow_query_count: u64,
    pub avg_execution_time: f64,
    pub cache_hit_rate: f64,
    pub slow_queries: u64,
    pub top_slow_queries: Vec<(String, Duration, u64)>,
}

#[derive(Debug, thiserror::Error)]
pub enum QueryOptimizationError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Pagination is disabled")]
    PaginationDisabled,

    #[error("Query analysis is disabled")]
    QueryAnalysisDisabled,

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Plan parsing error: {0}")]
    PlanParsingError(String),

    #[error("Cache error: {0}")]
    CacheError(String),
}

// Convenience functions
pub async fn create_query_optimizer(database_url: &str) -> Result<QueryOptimizer, sqlx::Error> {
    QueryOptimizer::new(database_url, QueryOptimizationConfig::default()).await
}

pub async fn create_query_optimizer_with_config(
    database_url: &str,
    config: QueryOptimizationConfig,
) -> Result<QueryOptimizer, sqlx::Error> {
    QueryOptimizer::new(database_url, config).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_query_optimizer_creation() {
        // This would require a test database connection
        // For now, just test the configuration
        let config = QueryOptimizationConfig::default();
        assert_eq!(config.max_connections, 20);
        assert_eq!(config.batch_size, 1000);
    }

    #[tokio::test]
    async fn test_pagination_calculation() {
        let total_count = 1000;
        let page_size = 100;
        let total_pages = ((total_count as f64) / (page_size as f64)).ceil() as usize;

        assert_eq!(total_pages, 10);
    }
}

// Re-export for convenience
pub use sqlx::FromRow;
