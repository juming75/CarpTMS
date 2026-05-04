//! / BFF性能优化模块
// 包含数据库优化、缓存优化、代码优化策略

use anyhow::Result;
use sqlx::PgPool;
use tracing::info;

/// 性能优化管理器
pub struct PerformanceOptimizer {
    postgres: PgPool,
}

impl PerformanceOptimizer {
    pub fn new(postgres: PgPool) -> Self {
        Self { postgres }
    }

    /// 数据库优化:检查并创建索引
    pub async fn optimize_database(&self) -> Result<()> {
        info!("开始数据库性能优化...");

        // 检查并创建GPS轨迹表索引
        self.create_gps_track_indexes().await?;

        // 检查并创建传感器数据表索引
        self.create_sensor_indexes().await?;

        // 检查并创建报警记录表索引
        self.create_alarm_indexes().await?;

        // 检查并创建称重数据表索引
        self.create_weighing_indexes().await?;

        // 检查并创建车辆表索引
        self.create_vehicle_indexes().await?;

        // 分析表统计信息
        self.analyze_tables().await?;

        info!("数据库性能优化完成");
        Ok(())
    }

    /// 创建GPS轨迹表索引
    async fn create_gps_track_indexes(&self) -> Result<()> {
        let indexes = vec![
            ("idx_gps_track_vehicle_time", "CREATE INDEX IF NOT EXISTS idx_gps_track_vehicle_time ON gps_track_data(vehicle_id, gps_time DESC)"),
            ("idx_gps_track_time", "CREATE INDEX IF NOT EXISTS idx_gps_track_time ON gps_track_data(gps_time DESC)"),
            ("idx_gps_track_spatial", "CREATE INDEX IF NOT EXISTS idx_gps_track_spatial ON gps_track_data USING GIST(location)"),
        ];

        for (index_name, sql) in indexes {
            if !self.index_exists(index_name).await? {
                info!("创建索引: {}", index_name);
                sqlx::query(sql).execute(&self.postgres).await?;
            }
        }

        Ok(())
    }

    /// 创建传感器数据表索引
    async fn create_sensor_indexes(&self) -> Result<()> {
        let indexes = vec![
            ("idx_sensor_vehicle_time", "CREATE INDEX IF NOT EXISTS idx_sensor_vehicle_time ON sensor_data(vehicle_id, collect_time DESC)"),
            ("idx_sensor_time", "CREATE INDEX IF NOT EXISTS idx_sensor_time ON sensor_data(collect_time DESC)"),
            ("idx_sensor_type", "CREATE INDEX IF NOT EXISTS idx_sensor_type ON sensor_data(sensor_type)"),
        ];

        for (index_name, sql) in indexes {
            if !self.index_exists(index_name).await? {
                info!("创建索引: {}", index_name);
                sqlx::query(sql).execute(&self.postgres).await?;
            }
        }

        Ok(())
    }

    /// 创建报警记录表索引
    async fn create_alarm_indexes(&self) -> Result<()> {
        let indexes = vec![
            ("idx_alarm_vehicle_time", "CREATE INDEX IF NOT EXISTS idx_alarm_vehicle_time ON alarm_records(vehicle_id, alarm_time DESC)"),
            ("idx_alarm_time", "CREATE INDEX IF NOT EXISTS idx_alarm_time ON alarm_records(alarm_time DESC)"),
            ("idx_alarm_level", "CREATE INDEX IF NOT EXISTS idx_alarm_level ON alarm_records(alarm_type)"),
            ("idx_alarm_handled", "CREATE INDEX IF NOT EXISTS idx_alarm_handled ON alarm_records(handle_status)"),
        ];

        for (index_name, sql) in indexes {
            if !self.index_exists(index_name).await? {
                info!("创建索引: {}", index_name);
                sqlx::query(sql).execute(&self.postgres).await?;
            }
        }

        Ok(())
    }

    /// 创建称重数据表索引
    async fn create_weighing_indexes(&self) -> Result<()> {
        let indexes = vec![
            ("idx_weighing_vehicle_time", "CREATE INDEX IF NOT EXISTS idx_weighing_vehicle_time ON weighing_data(vehicle_id, weighing_time DESC)"),
            ("idx_weighing_time", "CREATE INDEX IF NOT EXISTS idx_weighing_time ON weighing_data(weighing_time DESC)"),
        ];

        for (index_name, sql) in indexes {
            if !self.index_exists(index_name).await? {
                info!("创建索引: {}", index_name);
                sqlx::query(sql).execute(&self.postgres).await?;
            }
        }

        Ok(())
    }

    /// 创建车辆表索引
    async fn create_vehicle_indexes(&self) -> Result<()> {
        let indexes = vec![
            (
                "idx_vehicle_group",
                "CREATE INDEX IF NOT EXISTS idx_vehicle_group ON vehicles(group_id)",
            ),
            (
                "idx_vehicle_type",
                "CREATE INDEX IF NOT EXISTS idx_vehicle_type ON vehicles(vehicle_type)",
            ),
            (
                "idx_vehicle_status",
                "CREATE INDEX IF NOT EXISTS idx_vehicle_status ON vehicles(status)",
            ),
            (
                "idx_vehicle_license",
                "CREATE INDEX IF NOT EXISTS idx_vehicle_license ON vehicles(license_plate)",
            ),
        ];

        for (index_name, sql) in indexes {
            if !self.index_exists(index_name).await? {
                info!("创建索引: {}", index_name);
                sqlx::query(sql).execute(&self.postgres).await?;
            }
        }

        Ok(())
    }

    /// 检查索引是否存在
    async fn index_exists(&self, index_name: &str) -> Result<bool> {
        let sql = r#"
            SELECT EXISTS (
                SELECT 1 FROM pg_indexes 
                WHERE indexname = $1
            )
        "#;

        let exists: bool = sqlx::query_scalar(sql)
            .bind(index_name)
            .fetch_one(&self.postgres)
            .await?;

        Ok(exists)
    }

    /// 分析表统计信息(更新查询计划器)
    async fn analyze_tables(&self) -> Result<()> {
        let tables = vec![
            "vehicles",
            "gps_track_data",
            "sensor_data",
            "alarm_records",
            "weighing_data",
        ];

        for table in tables {
            let sql = format!("ANALYZE {}", table);
            info!("分析表: {}", table);
            sqlx::query(&sql).execute(&self.postgres).await?;
        }

        Ok(())
    }

    /// 获取数据库连接池状态
    pub async fn get_pool_stats(&self) -> Result<PoolStats> {
        let size = self.postgres.size();
        let idle = self.postgres.num_idle() as u32;
        Ok(PoolStats {
            max_size: size,
            idle_size: idle,
            active_size: size - idle,
        })
    }
}

/// 连接池状态统计
#[derive(Debug, Clone, serde::Serialize)]
pub struct PoolStats {
    pub max_size: u32,
    pub idle_size: u32,
    pub active_size: u32,
}

/// 缓存优化配置
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// 默认TTL(秒)
    pub default_ttl: u64,
    /// 车辆数据TTL
    pub vehicle_ttl: u64,
    /// GPS轨迹数据TTL
    pub gps_track_ttl: u64,
    /// 传感器数据TTL
    pub sensor_ttl: u64,
    /// 报表数据TTL
    pub report_ttl: u64,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            default_ttl: 300,    // 5分钟
            vehicle_ttl: 60,     // 1分钟
            gps_track_ttl: 1800, // 30分钟
            sensor_ttl: 600,     // 10分钟
            report_ttl: 3600,    // 1小时
        }
    }
}

/// 查询优化建议
#[derive(Debug, Clone)]
pub struct QueryOptimization {
    /// 使用EXPLAIN ANALYZE分析查询
    pub analyze_slow_queries: bool,
    /// 记录慢查询(毫秒)
    pub slow_query_threshold: u64,
    /// 启用查询缓存
    pub enable_query_cache: bool,
}

impl Default for QueryOptimization {
    fn default() -> Self {
        Self {
            analyze_slow_queries: true,
            slow_query_threshold: 100, // 100ms
            enable_query_cache: true,
        }
    }
}

/// 代码优化建议
pub struct CodeOptimization;

impl CodeOptimization {
    /// 使用连接池而非新建连接
    pub fn use_connection_pool() -> &'static str {
        "始终使用PgPool连接池,避免为每个请求创建新连接"
    }

    /// 使用批量查询
    pub fn use_batch_queries() -> &'static str {
        "对于需要查询多个记录的场景,使用IN子句或批量查询减少往返次数"
    }

    /// 使用异步并发
    pub fn use_async_concurrency() -> &'static str {
        "使用tokio::spawn或join_all并发执行独立的I/O操作"
    }

    /// 避免N+1查询
    pub fn avoid_n_plus_one_queries() -> &'static str {
        "使用JOIN或Eager Loading避免N+1查询问题"
    }

    /// 使用索引提示
    pub fn use_index_hint() -> &'static str {
        "确保查询使用了适当的索引,定期执行EXPLAIN ANALYZE"
    }

    /// 分页查询优化
    pub fn optimize_pagination() -> &'static str {
        "对于大数据集,使用cursor-based分页而非OFFSET分页"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_config_default() {
        let config = CacheConfig::default();
        assert_eq!(config.default_ttl, 300);
        assert_eq!(config.vehicle_ttl, 60);
        assert_eq!(config.gps_track_ttl, 1800);
    }

    #[test]
    fn test_query_optimization_default() {
        let config = QueryOptimization::default();
        assert!(config.analyze_slow_queries);
        assert_eq!(config.slow_query_threshold, 100);
        assert!(config.enable_query_cache);
    }

    #[test]
    fn test_code_optimization_suggestions() {
        assert!(!CodeOptimization::use_connection_pool().is_empty());
        assert!(!CodeOptimization::use_batch_queries().is_empty());
        assert!(!CodeOptimization::use_async_concurrency().is_empty());
    }
}
