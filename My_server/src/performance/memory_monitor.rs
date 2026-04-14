//! Memory Monitoring and Limiting Module
//!
//! Provides memory usage monitoring and limit enforcement capabilities

use log::{error, info, warn};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use sysinfo::{System, SystemExt};

/// 内存使用回调类型
pub type MemoryUsageCallback = Arc<Mutex<Option<Box<dyn Fn(f32) + Send + Sync>>>>;

/// 内存限制回调类型
pub type MemoryLimitCallback = Arc<Mutex<Option<Box<dyn Fn(u64, u64) + Send + Sync>>>>;

/// 内存监控配置
#[derive(Debug, Clone)]
pub struct MemoryMonitorConfig {
    /// 内存使用阈值(百分比)
    pub memory_threshold: f32,
    /// 检查间隔
    pub check_interval: Duration,
    /// 内存限制(字节)
    pub memory_limit: Option<u64>,
    /// 是否启用内存限制
    pub enable_memory_limit: bool,
}

impl Default for MemoryMonitorConfig {
    fn default() -> Self {
        Self {
            memory_threshold: 80.0,
            check_interval: Duration::from_secs(60),
            memory_limit: None,
            enable_memory_limit: false,
        }
    }
}

/// 内存监控器
pub struct MemoryMonitor {
    config: MemoryMonitorConfig,
    system: Arc<Mutex<System>>,
    is_running: Arc<Mutex<bool>>,
    memory_usage_callback: MemoryUsageCallback,
    memory_limit_callback: MemoryLimitCallback,
}

impl MemoryMonitor {
    /// 创建新的内存监控器
    pub fn new(config: MemoryMonitorConfig) -> Self {
        let system = Arc::new(Mutex::new(System::new_all()));
        // 初始化系统信息
        if let Ok(mut sys) = system.lock() { sys.refresh_all(); }

        Self {
            config,
            system,
            is_running: Arc::new(Mutex::new(false)),
            memory_usage_callback: Arc::new(Mutex::new(None)),
            memory_limit_callback: Arc::new(Mutex::new(None)),
        }
    }

    /// 设置内存使用回调函数
    pub fn set_memory_usage_callback<F>(&self, callback: F)
    where
        F: Fn(f32) + Send + Sync + 'static,
    {
        if let Ok(mut cb) = self.memory_usage_callback.lock() {
            *cb = Some(Box::new(callback));
        }
    }

    /// 设置内存限制回调函数
    pub fn set_memory_limit_callback<F>(&self, callback: F)
    where
        F: Fn(u64, u64) + Send + Sync + 'static,
    {
        if let Ok(mut cb) = self.memory_limit_callback.lock() {
            *cb = Some(Box::new(callback));
        }
    }

    /// 启动内存监控
    pub fn start(&self) {
        if let Ok(mut running) = self.is_running.lock() {
            if *running {
                warn!("Memory monitor is already running");
                return;
            }
            *running = true;
        }

        let system = self.system.clone();
        let config = self.config.clone();
        let is_running = self.is_running.clone();
        let memory_usage_callback = self.memory_usage_callback.clone();
        let memory_limit_callback = self.memory_limit_callback.clone();

        thread::spawn(move || {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build();
            let runtime = match runtime { Ok(r) => r, Err(_) => return, };

            runtime.block_on(async move {
                loop {
                    // 检查是否应该停止
                    if !is_running.lock().ok().map(|r| *r).unwrap_or(true) {
                        break;
                    }

                    // 刷新系统信息
                    if let Ok(mut sys) = system.lock() {
                        sys.refresh_all();
                    }

                    // 获取内存使用信息
                    let (_total_memory, used_memory, memory_usage_percent) = {
                        let sys = match system.lock() {
                            Ok(s) => s,
                            Err(_) => continue,
                        };
                        let total = sys.total_memory();
                        let used = sys.used_memory();
                        let usage_percent = (used as f32 / total as f32) * 100.0;
                        (total, used, usage_percent)
                    };

                    // 调用内存使用回调
                    if let Ok(cb_guard) = memory_usage_callback.lock() {
                        if let Some(callback) = cb_guard.as_ref() {
                            callback(memory_usage_percent);
                        }
                    }

                    // 检查内存使用是否超过阈值
                    if memory_usage_percent > config.memory_threshold {
                        warn!("Memory usage high: {:.2}%", memory_usage_percent);
                    }

                    // 检查内存限制
                    if config.enable_memory_limit {
                        if let Some(limit) = config.memory_limit {
                            if used_memory > limit {
                                error!(
                                    "Memory limit exceeded: {} bytes (limit: {} bytes)",
                                    used_memory, limit
                                );
                                if let Ok(cb_guard) = memory_limit_callback.lock() {
                                    if let Some(callback) = cb_guard.as_ref() {
                                        callback(used_memory, limit);
                                    }
                                }
                            }
                        }
                    }

                    // 等待下一次检查
                    tokio::time::sleep(config.check_interval).await;
                }
            });
        });

        info!("Memory monitor started");
    }

    /// 停止内存监控
    pub fn stop(&self) {
        if let Ok(mut running) = self.is_running.lock() {
            *running = false;
        }
        info!("Memory monitor stopped");
    }

    /// 获取当前内存使用情况
    pub fn get_memory_usage(&self) -> (u64, u64, f32) {
        if let Ok(mut sys) = self.system.lock() {
            sys.refresh_all();
        }
        match self.system.lock() {
            Ok(info) => {
                let total_memory = info.total_memory();
                let used_memory = info.used_memory();
                let percent = (used_memory as f32 / total_memory as f32) * 100.0;
                (total_memory, used_memory, percent)
            }
            Err(_) => (0, 0, 0.0),
        }
    }

    /// 获取当前内存使用百分比
    pub fn get_memory_usage_percent(&self) -> f32 {
        let (_, _, percent) = self.get_memory_usage();
        percent
    }
}

/// 内存限制服务
pub struct MemoryLimitService {
    monitor: Arc<MemoryMonitor>,
    memory_manager: Option<Arc<super::memory_optimization::MemoryManager>>,
}

impl MemoryLimitService {
    /// 创建新的内存限制服务
    pub fn new(
        config: MemoryMonitorConfig,
        memory_manager: Option<Arc<super::memory_optimization::MemoryManager>>,
    ) -> Self {
        let monitor = Arc::new(MemoryMonitor::new(config));

        // 设置内存使用回调
        monitor.set_memory_usage_callback(|usage| {
            info!("Memory usage: {:.2}%", usage);
        });

        // 设置内存限制回调
        let memory_manager_clone = memory_manager.clone();
        monitor.set_memory_limit_callback(move |used, limit| {
            warn!(
                "Memory limit exceeded: {} bytes (limit: {} bytes)",
                used, limit
            );

            // 尝试强制垃圾回收
            if let Some(manager) = &memory_manager_clone {
                info!("Forcing garbage collection...");
                manager.force_gc();
            }
        });

        Self {
            monitor,
            memory_manager,
        }
    }

    /// 启动服务
    pub fn start(&self) {
        self.monitor.start();
        info!("Memory limit service started");
    }

    /// 停止服务
    pub fn stop(&self) {
        self.monitor.stop();
        info!("Memory limit service stopped");
    }

    /// 获取内存监控器
    pub fn get_monitor(&self) -> Arc<MemoryMonitor> {
        self.monitor.clone()
    }

    /// 获取内存管理器
    pub fn get_memory_manager(&self) -> Option<Arc<super::memory_optimization::MemoryManager>> {
        self.memory_manager.clone()
    }
}
