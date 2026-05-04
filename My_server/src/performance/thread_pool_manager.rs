//! /! 动态线程池管理器
//!
//! 实现基于系统负载的动态线程池大小调整

use log::info;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use sysinfo::System;

/// 线程池管理器配置
#[derive(Debug, Clone)]
pub struct ThreadPoolManagerConfig {
    /// 最小线程数
    pub min_workers: usize,
    /// 最大线程数
    pub max_workers: usize,
    /// CPU使用率阈值(百分比),超过此值时增加线程
    pub cpu_threshold: f32,
    /// 检查间隔
    pub check_interval: Duration,
    /// 线程数调整步长
    pub adjustment_step: usize,
}

impl Default for ThreadPoolManagerConfig {
    fn default() -> Self {
        Self {
            min_workers: 2,
            max_workers: num_cpus::get() * 4,
            cpu_threshold: 70.0,
            check_interval: Duration::from_secs(30),
            adjustment_step: 2,
        }
    }
}

/// 线程池管理器
pub struct ThreadPoolManager {
    config: ThreadPoolManagerConfig,
    current_workers: Arc<Mutex<usize>>,
    system: Arc<Mutex<System>>,
}

impl ThreadPoolManager {
    /// 创建新的线程池管理器
    pub fn new(config: ThreadPoolManagerConfig) -> Self {
        let system = Arc::new(Mutex::new(System::new_all()));
        // 初始化系统信息
        system.lock().ok().map(|mut s| { s.refresh_all(); s }).unwrap_or_else(System::new_all);

        let min_workers = config.min_workers;

        Self {
            config,
            current_workers: Arc::new(Mutex::new(min_workers)),
            system,
        }
    }

    /// 启动线程池管理
    pub fn start(&self) {
        let current_workers = self.current_workers.clone();
        let config = self.config.clone();
        let system = self.system.clone();

        // 启动监控线程
        thread::spawn(move || {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build();
            let runtime = match runtime { Ok(r) => r, Err(_) => return };

            runtime.block_on(async move {
                loop {
                    tokio::time::sleep(config.check_interval).await;

                    // 刷新系统信息
                    if let Ok(mut sys) = system.lock() {
                        sys.refresh_all();
                        let processors = sys.cpus();
                        let cpu_usage = processors.iter().map(|c| c.cpu_usage()).sum::<f32>() / processors.len() as f32;

                        if let Ok(mut current) = current_workers.lock() {
                            if cpu_usage > config.cpu_threshold && *current < config.max_workers {
                                let new_workers = (*current + config.adjustment_step).min(config.max_workers);
                                info!("CPU usage high ({:.2}%), recommending thread pool size increase from {} to {}", 
                                      cpu_usage, *current, new_workers);
                                *current = new_workers;
                            } else if cpu_usage < config.cpu_threshold * 0.5 && *current > config.min_workers {
                                let new_workers = (*current - config.adjustment_step).max(config.min_workers);
                                info!("CPU usage low ({:.2}%), recommending thread pool size decrease from {} to {}", 
                                      cpu_usage, *current, new_workers);
                                *current = new_workers;
                            }
                        }
                    }
                }
            });
        });

        info!("Thread pool manager started");
    }

    /// 获取当前线程数
    pub fn get_current_workers(&self) -> usize {
        self.current_workers.lock().ok().map(|c| *c).unwrap_or(0)
    }

    /// 获取推荐的线程数
    pub fn get_recommended_workers(&self) -> usize {
        self.current_workers.lock().ok().map(|c| *c).unwrap_or(0)
    }
}

/// 扩展 HttpServer 以支持动态线程池
pub trait HttpServerThreadPoolExt {
    /// 使用动态线程池
    fn with_dynamic_thread_pool(
        self,
        config: ThreadPoolManagerConfig,
    ) -> (Self, Arc<ThreadPoolManager>)
    where
        Self: Sized;
}

impl<T> HttpServerThreadPoolExt for T {
    fn with_dynamic_thread_pool(
        self,
        config: ThreadPoolManagerConfig,
    ) -> (Self, Arc<ThreadPoolManager>)
    where
        Self: Sized,
    {
        let manager = Arc::new(ThreadPoolManager::new(config));
        manager.start();
        (self, manager)
    }
}
