//! /! 读写分离数据库连接池管理器
//! 
//! 提供主从数据库连接池管理,实现读写分离以减轻主数据库压力

use sqlx::postgres::{PgPool, PgPoolOptions};
use std::sync::Arc;
use std::time::Duration;
use tracing::{info, warn, error};

/// 读写分离连接池配置
#[derive(Debug, Clone)]
pub struct ReadWritePoolConfig {
    /// 主数据库连接字符串(写操作)
    pub master_url: String,
    /// 从数据库连接字符串列表(读操作)
    pub replica_urls: Vec<String>,
    /// 连接池最小连接数
    pub min_connections: u32,
    /// 连接池最大连接数
    pub max_connections: u32,
    /// 连接超时时间
    pub connect_timeout: Duration,
    /// 连接最大空闲时间
    pub idle_timeout: Duration,
    /// 连接最大生命周期
    pub max_lifetime: Duration,
    /// 读操作负载均衡策略
    pub load_balance_strategy: LoadBalanceStrategy,
}

/// 负载均衡策略
#[derive(Debug, Clone)]
pub enum LoadBalanceStrategy {
    /// 轮询
    RoundRobin,
    /// 随机
    Random,
    /// 最少连接数
    LeastConnections,
    /// 加权轮询
    WeightedRoundRobin(Vec<u32>),
}

impl Default for ReadWritePoolConfig {
    fn default() -> Self {
        Self {
            master_url: String::new(),
            replica_urls: vec![],
            min_connections: 5,
            max_connections: 20,
            connect_timeout: Duration::from_secs(30),
            idle_timeout: Duration::from_secs(600),
            max_lifetime: Duration::from_secs(1800),
            load_balance_strategy: LoadBalanceStrategy::RoundRobin,
        }
    }
}

/// 读写分离连接池管理器
pub struct ReadWritePoolManager {
    /// 主数据库连接池(写操作)
    master_pool: PgPool,
    /// 从数据库连接池列表(读操作)
    replica_pools: Vec<PgPool>,
    /// 当前读操作的连接池索引(用于轮询)
    current_replica_index: std::sync::atomic::AtomicUsize,
    /// 配置
    config: ReadWritePoolConfig,
}

impl ReadWritePoolManager {
    /// 创建新的读写分离连接池管理器
    pub async fn new(config: ReadWritePoolConfig) -> Result<Arc<Self>, sqlx::Error> {
        info!("Initializing read-write pool manager...");
        
        // 创建主数据库连接池
        let master_pool = Self::create_pool(&config.master_url, &config, "master").await?;
        info!("Master pool created successfully");
        
        // 创建从数据库连接池列表
        let mut replica_pools = Vec::new();
        for (i, replica_url) in config.replica_urls.iter().enumerate() {
            let pool = Self::create_pool(replica_url, &config, &format!("replica_{}", i)).await?;
            replica_pools.push(pool);
            info!("Replica pool {} created successfully", i);
        }
        
        // 如果没有配置从数据库,使用主数据库作为读操作池
        if replica_pools.is_empty() {
            warn!("No replica pools configured, using master pool for read operations");
            replica_pools.push(master_pool.clone());
        }
        
        let manager = Arc::new(Self {
            master_pool,
            replica_pools,
            current_replica_index: std::sync::atomic::AtomicUsize::new(0),
            config,
        });
        
        // 启动连接池监控
        Self::start_pool_monitoring(manager.clone());
        
        info!("Read-write pool manager initialized successfully");
        Ok(manager)
    }
    
    /// 创建单个连接池
    async fn create_pool(
        url: &str,
        config: &ReadWritePoolConfig,
        name: &str,
    ) -> Result<PgPool, sqlx::Error> {
        info!("Creating pool for {}...", name);
        
        let pool = PgPoolOptions::new()
            .min_connections(config.min_connections)
            .max_connections(config.max_connections)
            .acquire_timeout(config.connect_timeout)
            .idle_timeout(config.idle_timeout)
            .max_lifetime(config.max_lifetime)
            .test_before_acquire(true)
            .after_connect(move |conn, _meta| {
                Box::pin(async move {
                    // 设置连接参数
                    sqlx::query("SET application_name = 'carptms'")
                        .execute(conn)
                        .await?;
                    Ok(())
                })
            })
            .connect(url)
            .await?;
        
        // 测试连接
        sqlx::query("SELECT 1").fetch_one(&pool).await?;
        
        info!("Pool for {} created with {} max connections", name, config.max_connections);
        Ok(pool)
    }
    
    /// 获取主数据库连接池(用于写操作)
    pub fn master(&self) -> &PgPool {
        &self.master_pool
    }
    
    /// 获取从数据库连接池(用于读操作)
    pub fn replica(&self) -> &PgPool {
        match self.config.load_balance_strategy {
            LoadBalanceStrategy::RoundRobin => {
                let index = self.current_replica_index.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                &self.replica_pools[index % self.replica_pools.len()]
            },
            LoadBalanceStrategy::Random => {
                use rand::Rng;
                let index = rand::thread_rng().gen_range(0..self.replica_pools.len());
                &self.replica_pools[index]
            },
            LoadBalanceStrategy::LeastConnections => {
                // 简化实现,实际应该跟踪每个池的连接数
                let index = self.current_replica_index.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                &self.replica_pools[index % self.replica_pools.len()]
            },
            LoadBalanceStrategy::WeightedRoundRobin(ref _weights) => {
                // 简化实现,实际应该根据权重分配
                let index = self.current_replica_index.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                &self.replica_pools[index % self.replica_pools.len()]
            },
        }
    }
    
    /// 启动连接池监控
    fn start_pool_monitoring(manager: Arc<Self>) {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));
            
            loop {
                interval.tick().await;
                
                // 监控主数据库连接池
                if let Err(e) = manager.monitor_pool(&manager.master_pool, "master").await {
                    error!("Failed to monitor master pool: {}", e);
                }
                
                // 监控从数据库连接池
                for (i, pool) in manager.replica_pools.iter().enumerate() {
                    if let Err(e) = manager.monitor_pool(pool, &format!("replica_{}", i)).await {
                        error!("Failed to monitor replica_{} pool: {}", i, e);
                    }
                }
            }
        });
    }
    
    /// 监控单个连接池
    async fn monitor_pool(&self, pool: &PgPool, pool_name: &str) -> Result<(), sqlx::Error> {
        // 获取连接池统计信息
        let size = pool.size() as usize;
        let num_idle = pool.num_idle();
        let active = size.saturating_sub(num_idle);
        
        // 记录到监控表
        sqlx::query(
            r#"
            INSERT INTO db_pool_metrics 
            (pool_name, active_connections, idle_connections, total_connections, query_count)
            VALUES ($1, $2, $3, $4, $5)
            "#
        )
        .bind(pool_name)
        .bind(active as i32)
        .bind(num_idle as i32)
        .bind(size as i32)
        .bind(0i32) // query_count 需要实际统计
        .execute(&self.master_pool)
        .await?;
        
        // 检查连接池健康状态
        if active as f32 / self.config.max_connections as f32 > 0.8 {
            warn!(
                "Pool {} is under high load: {}/{} connections active",
                pool_name, active, self.config.max_connections
            );
        }
        
        Ok(())
    }
    
    /// 获取连接池统计信息
    pub async fn get_pool_stats(&self) -> PoolStats {
        let master_size = self.master_pool.size() as usize;
        let master_idle = self.master_pool.num_idle();
        
        let mut replica_stats = Vec::new();
        for pool in &self.replica_pools {
            let size = pool.size() as usize;
            let idle = pool.num_idle();
            replica_stats.push(PoolStat {
                size,
                idle,
                active: size.saturating_sub(idle),
            });
        }
        
        PoolStats {
            master: PoolStat {
                size: master_size,
                idle: master_idle,
                active: master_size.saturating_sub(master_idle),
            },
            replicas: replica_stats,
        }
    }
    
    /// 优雅关闭连接池
    pub async fn close(&self) {
        info!("Closing read-write pool manager...");
        
        self.master_pool.close().await;
        info!("Master pool closed");
        
        for (i, pool) in self.replica_pools.iter().enumerate() {
            pool.close().await;
            info!("Replica pool {} closed", i);
        }
        
        info!("Read-write pool manager closed successfully");
    }
}

/// 连接池统计信息
#[derive(Debug)]
pub struct PoolStats {
    pub master: PoolStat,
    pub replicas: Vec<PoolStat>,
}

/// 单个连接池统计
#[derive(Debug)]
pub struct PoolStat {
    pub size: usize,
    pub idle: usize,
    pub active: usize,
}

/// 从环境变量创建配置
pub fn create_config_from_env() -> ReadWritePoolConfig {
    use std::env;
    
    let master_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:123@localhost:5432/carptms_db".to_string());
    
    // 从环境变量读取从数据库URL列表
    let replica_urls: Vec<String> = env::var("REPLICA_DATABASE_URLS")
        .unwrap_or_default()
        .split(',')
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect();
    
    let min_connections = env::var("DB_MIN_CONNECTIONS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(5);
    
    let max_connections = env::var("DB_MAX_CONNECTIONS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(20);
    
    ReadWritePoolConfig {
        master_url,
        replica_urls,
        min_connections,
        max_connections,
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_pool_creation() {
        let config = ReadWritePoolConfig {
            master_url: "postgresql://postgres:123@localhost:5432/carptms_db".to_string(),
            replica_urls: vec![],
            min_connections: 1,
            max_connections: 5,
            ..Default::default()
        };
        
        let manager = ReadWritePoolManager::new(config).await;
        assert!(manager.is_ok());
        
        let manager = manager.unwrap();
        let stats = manager.get_pool_stats().await;
        assert!(stats.master.size > 0);
        
        manager.close().await;
    }
}







