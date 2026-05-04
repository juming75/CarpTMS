pub mod adapter;
pub mod migration;
pub mod query_optimization;
pub mod read_write_pool;

pub use adapter::{
    get_db_type, get_read_write_pool, DatabasePool, DatabaseType, ReadWritePool, SqlAdapter,
};

pub use query_optimization::{
    create_query_optimizer, create_query_optimizer_with_config, PaginationResult, QueryMetrics,
    QueryOptimizationConfig, QueryOptimizationError, QueryOptimizer, QueryPlanAnalysis,
    QueryStatistics,
};

pub use read_write_pool::{
    get_read_write_pool as get_pg_read_write_pool, ReadWritePool as PgReadWritePool,
};
