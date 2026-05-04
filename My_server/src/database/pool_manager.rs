use sqlx::postgres::PgPool;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::{Duration, Instant};
use log::{info, debug};

// 连接池性能指标
#[derive(Debug, Clone)]
pub struct PoolMetrics {
    pub active_connections: u32,
    pub idle_connections: u32,
    pub connection_wait_time: Duration,
    pub query_execution_time: Duration,
    pub connection_errors: u32,
    pub last_updated: Instant,
}

impl Default for PoolMetrics {
    fn default() -> Self {
        Self {
            active_connections: 0,
            idle_connections: 0,
            connection_wait_time: Duration::from_secs(0),
            query_execution_time: Duration::from_secs(0),
            connection_errors: 0,
            last_updated: Instant::now(),
        }
    }
}

// 自适应连接池配置
#[derive(Debug, Clone)]
pub struct AdaptivePoolConfig {
    // 基础配置
    pub min_connections: u32,
    pub max_connections: u32,
    pub target_utilization: f64, // 目标利用率 (0.5-0.8)
    
    // 调整参数
    pub scaling_factor: f64, // 缩放因子
    pub min_adjustment_interval: Duration, // 最小调整间隔
    pub max_adjustment_step: u32, // 最大调整步长
    
    // 性能阈值
    pub high_load_threshold: f64, // 高负载阈值
    pub low_load_threshold: f64, // 低负载阈值
    pub max_wait_time_threshold: Duration, // 最大等待时间阈值
}

impl Default for AdaptivePoolConfig {
    fn default() -> Self {
        Self {
            min_connections: 10,
            max_connections: 300,
            target_utilization: 0.7,
            scaling_factor: 1.5,
            min_adjustment_interval: Duration::from_secs(30),
            max_adjustment_step: 20,
            high_load_threshold: 0.8,
            low_load_threshold: 0.3,
            max_wait_time_threshold: Duration::from_secs(5),
        }
    }
}

// 连接池管理器
#[derive(Debug, Clone)]
pub struct PoolManager {
    config: Arc<RwLock<AdaptivePoolConfig>>,
    metrics: Arc<RwLock<PoolMetrics>>,
    last_adjustment: Arc<RwLock<Instant>>,
}

impl PoolManager {
    pub fn new() -> Self {
        Self {
            config: Arc::new(RwLock::new(AdaptivePoolConfig::default())),
            metrics: Arc::new(RwLock::new(PoolMetrics::default())),
            last_adjustment: Arc::new(RwLock::new(Instant::now())),
        }
    }
    
    // 更新性能指标
    pub async fn update_metrics(&self, metrics: PoolMetrics) {
        let mut current_metrics = self.metrics.write().await;
        *current_metrics = metrics;
        debug!("Updated pool metrics: {:?}", current_metrics);
    }
    
    // 获取当前配置
    pub async fn get_config(&self) -> AdaptivePoolConfig {
        self.config.read().await.clone()
    }
    
    // 动态调整连接池配置
    pub async fn adjust_pool_config(&self) -> Option<AdaptivePoolConfig> {
        let now = Instant::now();
        let last_adjustment = *self.last_adjustment.read().await;
        
        // 检查是否达到调整间隔
        if now.duration_since(last_adjustment) < self.config.read().await.min_adjustment_interval {
            return None;
        }
        
        let metrics = self.metrics.read().await.clone();
        let mut config = self.config.write().await;
        
        // 计算当前利用率
        let total_connections = metrics.active_connections + metrics.idle_connections;
        let utilization = if total_connections > 0 {
            metrics.active_connections as f64 / total_connections as f64
        } else {
            0.0
        };
        
        debug!("Current pool utilization: {:.2}, active: {}, idle: {}", 
               utilization, metrics.active_connections, metrics.idle_connections);
        
        // 基于利用率和性能指标调整配置
        let mut adjustment = 0;
        let mut is_decrease = false;
        
        if utilization > config.high_load_threshold || metrics.connection_wait_time > config.max_wait_time_threshold {
            // 高负载,增加连接数
            let needed_increase = ((utilization - config.target_utilization) * total_connections as f64 * config.scaling_factor).ceil() as u32;
            adjustment = needed_increase.min(config.max_adjustment_step);
            info!("High load detected, increasing connections by {}", adjustment);
        } else if utilization < config.low_load_threshold && total_connections > config.min_connections {
            // 低负载,减少连接数
            let possible_decrease = ((config.target_utilization - utilization) * total_connections as f64 * config.scaling_factor).ceil() as u32;
            adjustment = possible_decrease.min(config.max_adjustment_step).min(total_connections - config.min_connections);
            adjustment = adjustment.saturating_sub(1); // 确保至少减少1个
            is_decrease = true;
            if adjustment > 0 {
                info!("Low load detected, decreasing connections by {}", adjustment);
            }
        }
        
        // 应用调整
        if adjustment > 0 {
            if is_decrease {
                // 减少连接数
                config.min_connections = config.min_connections.saturating_sub(1);
                config.min_connections = config.min_connections.max(5);
            } else {
                // 增加连接数
                config.min_connections = (config.min_connections as f64 * 1.1).ceil() as u32;
                config.min_connections = config.min_connections.min(config.max_connections - 10);
            }
        }
        
        // 更新最后调整时间
        *self.last_adjustment.write().await = now;
        
        info!("Adjusted pool config: {:?}", *config);
        Some(config.clone())
    }
    
    // 手动更新配置
    pub async fn update_config(&self, new_config: AdaptivePoolConfig) {
        let mut config = self.config.write().await;
        *config = new_config;
        info!("Manually updated pool config: {:?}", *config);
    }
    
    // 获取当前性能指标
    pub async fn get_metrics(&self) -> PoolMetrics {
        self.metrics.read().await.clone()
    }
}

// 监控任务
pub async fn start_pool_monitoring(
    pool_manager: Arc<PoolManager>,
    pools: Arc<Vec<PgPool>>,
) {
    info!("Starting pool monitoring task");
    
    loop {
        // 每10秒收集一次指标
        tokio::time::sleep(Duration::from_secs(10)).await;
        
        // 收集所有连接池的指标
        let mut total_active = 0;
        let mut total_idle = 0;
        
        for _pool in pools.iter() {
            // 注意:sqlx的PgPool没有直接暴露这些指标
            // 这里我们使用模拟数据,实际项目中可能需要使用监控库或自定义实现
            // 这里仅作为示例
            total_active += 5; // 模拟值
            total_idle += 10;  // 模拟值
        }
        
        // 创建指标
        let metrics = PoolMetrics {
            active_connections: total_active,
            idle_connections: total_idle,
            connection_wait_time: Duration::from_millis(50), // 模拟值
            query_execution_time: Duration::from_millis(100), // 模拟值
            connection_errors: 0,
            last_updated: Instant::now(),
        };
        
        // 更新指标
        pool_manager.update_metrics(metrics).await;
        
        // 尝试调整配置
        pool_manager.adjust_pool_config().await;
    }
}





