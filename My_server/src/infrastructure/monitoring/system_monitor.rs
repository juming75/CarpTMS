//! 系统监控器
//!
//! 实时监控系统各项指标，为自动架构切换提供数据支持

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 系统指标数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    /// 采集时间
    pub timestamp: DateTime<Utc>,
    
    /// 数据库指标
    pub database: DatabaseMetrics,
    
    /// 性能指标
    pub performance: PerformanceMetrics,
    
    /// 资源使用指标
    pub resources: ResourceMetrics,
    
    /// 业务指标
    pub business: BusinessMetrics,
}

/// 数据库指标
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DatabaseMetrics {
    /// 总记录数估算（关键表）
    pub total_records: HashMap<String, i64>,
    /// 数据库连接数
    pub active_connections: u32,
    /// 慢查询数量（最近1分钟）
    pub slow_queries: u32,
    /// 查询平均响应时间（毫秒）
    pub avg_query_time_ms: f64,
    /// 事务吞吐量（每秒）
    pub transaction_tps: f64,
}

/// 性能指标
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// 平均响应时间（毫秒）
    pub avg_response_time_ms: f64,
    /// P95响应时间（毫秒）
    pub p95_response_time_ms: f64,
    /// P99响应时间（毫秒）
    pub p99_response_time_ms: f64,
    /// 每秒请求数（QPS）
    pub requests_per_second: f64,
    /// 错误率（百分比）
    pub error_rate: f64,
    /// 并发连接数
    pub concurrent_connections: u32,
}

/// 资源使用指标
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResourceMetrics {
    /// CPU使用率（百分比）
    pub cpu_usage_percent: f64,
    /// 内存使用率（百分比）
    pub memory_usage_percent: f64,
    /// 内存使用量（MB）
    pub memory_usage_mb: u64,
    /// 磁盘使用率（百分比）
    pub disk_usage_percent: f64,
    /// 网络IO（MB/s）
    pub network_io_mbps: f64,
}

/// 业务指标
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BusinessMetrics {
    /// 日活跃用户（DAU）
    pub daily_active_users: u32,
    /// 日订单量
    pub daily_orders: u32,
    /// 活跃车辆数
    pub active_vehicles: u32,
    /// 活跃司机数
    pub active_drivers: u32,
    /// 数据增长率（每天百分比）
    pub data_growth_rate: f64,
}

/// 系统监控器
pub struct SystemMonitor {
    /// 当前指标
    metrics: Arc<RwLock<SystemMetrics>>,
    /// 历史指标（保留最近1小时）
    history: Arc<RwLock<Vec<SystemMetrics>>>,
    /// 数据库连接池
    db_pool: Option<sqlx::PgPool>,
    /// 采集间隔
    collection_interval: Duration,
    /// 是否运行中
    running: Arc<RwLock<bool>>,
}

impl SystemMonitor {
    /// 创建新的系统监控器
    pub fn new(db_pool: Option<sqlx::PgPool>) -> Self {
        Self {
            metrics: Arc::new(RwLock::new(SystemMetrics {
                timestamp: Utc::now(),
                database: DatabaseMetrics::default(),
                performance: PerformanceMetrics::default(),
                resources: ResourceMetrics::default(),
                business: BusinessMetrics::default(),
            })),
            history: Arc::new(RwLock::new(Vec::with_capacity(60))),
            db_pool,
            collection_interval: Duration::from_secs(60), // 默认每分钟采集一次
            running: Arc::new(RwLock::new(false)),
        }
    }
    
    /// 设置采集间隔
    pub fn with_interval(mut self, interval: Duration) -> Self {
        self.collection_interval = interval;
        self
    }
    
    /// 启动监控
    pub async fn start(&self) {
        let mut running = self.running.write().await;
        if *running {
            tracing::warn!("System monitor is already running");
            return;
        }
        *running = true;
        drop(running);
        
        tracing::info!("Starting system monitor with {:?} interval", self.collection_interval);
        
        let metrics = self.metrics.clone();
        let history = self.history.clone();
        let db_pool = self.db_pool.clone();
        let running = self.running.clone();
        let interval = self.collection_interval;
        
        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(interval);
            
            loop {
                ticker.tick().await;
                
                if !*running.read().await {
                    break;
                }
                
                // 采集指标
                let new_metrics = Self::collect_metrics(db_pool.clone()).await;
                
                // 更新当前指标
                let mut current = metrics.write().await;
                *current = new_metrics.clone();
                drop(current);
                
                // 添加到历史记录
                let mut hist = history.write().await;
                hist.push(new_metrics);
                // 只保留最近60条记录（1小时，如果每分钟采集一次）
                if hist.len() > 60 {
                    hist.remove(0);
                }
                drop(hist);
                
                tracing::debug!("System metrics collected");
            }
            
            tracing::info!("System monitor stopped");
        });
    }
    
    /// 停止监控
    pub async fn stop(&self) {
        let mut running = self.running.write().await;
        *running = false;
        tracing::info!("System monitor stopping...");
    }
    
    /// 获取当前指标
    pub async fn get_current_metrics(&self) -> SystemMetrics {
        self.metrics.read().await.clone()
    }
    
    /// 获取历史指标
    pub async fn get_history(&self) -> Vec<SystemMetrics> {
        self.history.read().await.clone()
    }
    
    /// 采集指标
    async fn collect_metrics(db_pool: Option<sqlx::PgPool>) -> SystemMetrics {
        SystemMetrics {
            timestamp: Utc::now(),
            database: Self::collect_database_metrics(db_pool.clone()).await,
            performance: Self::collect_performance_metrics().await,
            resources: Self::collect_resource_metrics().await,
            business: Self::collect_business_metrics(db_pool.clone()).await,
        }
    }
    
    /// 采集数据库指标
    async fn collect_database_metrics(db_pool: Option<sqlx::PgPool>) -> DatabaseMetrics {
        let mut metrics = DatabaseMetrics::default();
        
        if let Some(pool) = db_pool {
            // 获取活跃连接数
            metrics.active_connections = pool.size();
            
            // 查询关键表记录数
            let tables = vec![
                "vehicles", "drivers", "weighing_records", 
                "users", "orders", "finance_costs", "finance_invoices"
            ];
            
            for table in tables {
                let count_query = format!("SELECT COUNT(*) FROM {}", table);
                if let Ok(count) = sqlx::query_scalar::<_, i64>(&count_query)
                    .fetch_one(&pool)
                    .await {
                    metrics.total_records.insert(table.to_string(), count);
                }
            }
            
            // 计算总记录数
            let total: i64 = metrics.total_records.values().sum();
            metrics.total_records.insert("total".to_string(), total);
        }
        
        metrics
    }
    
    /// 采集性能指标（从应用内部指标获取）
    async fn collect_performance_metrics() -> PerformanceMetrics {
        // 这里可以从全局指标收集器获取
        // 简化实现，返回默认值
        PerformanceMetrics::default()
    }
    
    /// 采集资源指标
    async fn collect_resource_metrics() -> ResourceMetrics {
        let mut metrics = ResourceMetrics::default();
        
        // 获取系统信息
        if let Ok(sys_info) = sysinfo::System::new_all().await {
            // CPU使用率
            metrics.cpu_usage_percent = sys_info.global_cpu_info().cpu_usage() as f64;
            
            // 内存使用
            let total_memory = sys_info.total_memory();
            let used_memory = sys_info.used_memory();
            metrics.memory_usage_mb = used_memory / 1024;
            metrics.memory_usage_percent = (used_memory as f64 / total_memory as f64) * 100.0;
        }
        
        metrics
    }
    
    /// 采集业务指标
    async fn collect_business_metrics(db_pool: Option<sqlx::PgPool>) -> BusinessMetrics {
        let mut metrics = BusinessMetrics::default();
        
        if let Some(pool) = db_pool {
            // 今日活跃用户数
            let today = Utc::now().date_naive();
            if let Ok(count) = sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(DISTINCT user_id) FROM user_activities WHERE DATE(created_at) = $1"
            )
            .bind(today)
            .fetch_one(&pool)
            .await {
                metrics.daily_active_users = count as u32;
            }
            
            // 今日订单数
            if let Ok(count) = sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM orders WHERE DATE(created_at) = $1"
            )
            .bind(today)
            .fetch_one(&pool)
            .await {
                metrics.daily_orders = count as u32;
            }
            
            // 活跃车辆数
            if let Ok(count) = sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM vehicles WHERE status = 'active'"
            )
            .fetch_one(&pool)
            .await {
                metrics.active_vehicles = count as u32;
            }
            
            // 活跃司机数
            if let Ok(count) = sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM drivers WHERE status = 'active'"
            )
            .fetch_one(&pool)
            .await {
                metrics.active_drivers = count as u32;
            }
        }
        
        metrics
    }
    
    /// 计算综合负载分数（0-100）
    pub async fn calculate_load_score(&self) -> f64 {
        let metrics = self.get_current_metrics().await;
        
        let mut score: f64 = 0.0;
        
        // 数据库负载（基于总记录数）
        let total_records = metrics.database.total_records.get("total").copied().unwrap_or(0);
        if total_records > 10_000_000 {
            score += 30.0;
        } else if total_records > 1_000_000 {
            score += 20.0;
        } else if total_records > 100_000 {
            score += 10.0;
        }
        
        // 性能负载
        if metrics.performance.avg_response_time_ms > 500.0 {
            score += 20.0;
        } else if metrics.performance.avg_response_time_ms > 200.0 {
            score += 10.0;
        }
        
        // CPU负载
        if metrics.resources.cpu_usage_percent > 80.0 {
            score += 20.0;
        } else if metrics.resources.cpu_usage_percent > 60.0 {
            score += 10.0;
        }
        
        // 内存负载
        if metrics.resources.memory_usage_percent > 80.0 {
            score += 20.0;
        } else if metrics.resources.memory_usage_percent > 60.0 {
            score += 10.0;
        }
        
        // 业务负载
        if metrics.business.daily_orders > 100_000 {
            score += 10.0;
        }
        
        score.min(100.0)
    }
}

/// 模拟sysinfo模块（简化实现）
mod sysinfo {
    pub struct System;
    
    impl System {
        pub async fn new_all() -> Result<Self, ()> {
            Ok(Self)
        }
        
        pub fn global_cpu_info(&self) -> CpuInfo {
            CpuInfo
        }
        
        pub fn total_memory(&self) -> u64 {
            16 * 1024 * 1024 * 1024 // 16GB
        }
        
        pub fn used_memory(&self) -> u64 {
            8 * 1024 * 1024 * 1024 // 8GB
        }
    }
    
    pub struct CpuInfo;
    
    impl CpuInfo {
        pub fn cpu_usage(&self) -> f32 {
            50.0 // 50%
        }
    }
}