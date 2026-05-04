//! 数据库定时任务模块
//! 用于定期执行历史数据归档等操作

use log::{info, error};
use sqlx::PgPool;
use tokio::time::{interval, Duration};
use crate::infrastructure::database::cold_storage::{ColdStorageService, ColdStorageConfig};

/// 数据库定时任务调度器
pub struct DatabaseScheduler {
    pool: PgPool,
    cold_storage_service: ColdStorageService,
}

impl DatabaseScheduler {
    /// 创建新的调度器
    pub fn new(pool: PgPool) -> Self {
        let config = ColdStorageConfig::default();
        let cold_storage_service = ColdStorageService::new(pool.clone(), config);
        
        Self {
            pool,
            cold_storage_service,
        }
    }

    /// 启动所有定时任务
    pub async fn start(&self) {
        info!("启动数据库定时任务调度器");
        
        // 启动历史数据归档任务
        tokio::spawn(self.run_archive_task());
        
        // 启动其他定时任务（如果需要）
        // tokio::spawn(self.run_other_task());
    }

    /// 运行历史数据归档任务
    async fn run_archive_task(&self) {
        let interval = Duration::from_hours(self.cold_storage_service.config.archive_interval_hours);
        let mut timer = interval(interval);
        
        // 立即执行一次
        if let Err(e) = self.cold_storage_service.archive_all_tables().await {
            error!("初始归档执行失败: {}", e);
        }
        
        loop {
            timer.tick().await;
            info!("执行定期历史数据归档");
            
            if let Err(e) = self.cold_storage_service.archive_all_tables().await {
                error!("归档任务执行失败: {}", e);
            }
        }
    }

    /// 运行其他定时任务（示例）
    async fn run_other_task(&self) {
        let interval = Duration::from_hours(1);
        let mut timer = interval(interval);
        
        loop {
            timer.tick().await;
            // 这里可以添加其他定时任务
        }
    }
}
