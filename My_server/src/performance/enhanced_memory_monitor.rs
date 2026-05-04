//! Enhanced Memory Monitoring Module
//!
//! Provides advanced memory usage monitoring, alerting, and management capabilities

use crate::performance::memory_optimization::MemoryManager;
use log::{error, info, warn};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use sysinfo::System;

/// 内存使用历史记录
#[derive(Debug, Clone)]
pub struct MemoryUsageHistory {
    /// 时间戳
    pub timestamp: std::time::Instant,
    /// 内存使用百分比
    pub usage_percent: f32,
    /// 总内存(字节)
    pub total_memory: u64,
    /// 已使用内存(字节)
    pub used_memory: u64,
}

/// 内存监控配置
#[derive(Debug, Clone)]
pub struct EnhancedMemoryMonitorConfig {
    /// 内存使用警告阈值(百分比)
    pub warning_threshold: f32,
    /// 内存使用严重阈值(百分比)
    pub critical_threshold: f32,
    /// 检查间隔
    pub check_interval: Duration,
    /// 内存限制(字节)
    pub memory_limit: Option<u64>,
    /// 是否启用内存限制
    pub enable_memory_limit: bool,
    /// 历史记录保留时间
    pub history_retention: Duration,
    /// 警报冷却时间
    pub alert_cooldown: Duration,
}

impl Default for EnhancedMemoryMonitorConfig {
    fn default() -> Self {
        Self {
            warning_threshold: 70.0,
            critical_threshold: 85.0,
            check_interval: Duration::from_secs(30),
            memory_limit: Some(8 * 1024 * 1024 * 1024), // 8GB
            enable_memory_limit: true,
            history_retention: Duration::from_secs(24 * 60 * 60),
            alert_cooldown: Duration::from_secs(5 * 60),
        }
    }
}

/// 内存警报级别
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MemoryAlertLevel {
    /// 正常
    Normal,
    /// 警告
    Warning,
    /// 严重
    Critical,
    /// 超限
    ExceededLimit,
}

/// 内存监控器
pub struct EnhancedMemoryMonitor {
    config: EnhancedMemoryMonitorConfig,
    system: Arc<Mutex<System>>,
    is_running: Arc<Mutex<bool>>,
    usage_history: Arc<Mutex<Vec<MemoryUsageHistory>>>,
    last_alert: Arc<Mutex<Option<std::time::Instant>>>,
    current_alert_level: Arc<Mutex<MemoryAlertLevel>>,
    memory_manager: Option<Arc<MemoryManager>>,
}

impl EnhancedMemoryMonitor {
    /// 创建新的内存监控器
    pub fn new(
        config: EnhancedMemoryMonitorConfig,
        memory_manager: Option<Arc<MemoryManager>>,
    ) -> Self {
        let system = Arc::new(Mutex::new(System::new_all()));
        // 初始化系统信息
        if let Ok(mut sys) = system.lock() {
            sys.refresh_all();
        }

        Self {
            config,
            system,
            is_running: Arc::new(Mutex::new(false)),
            usage_history: Arc::new(Mutex::new(Vec::new())),
            last_alert: Arc::new(Mutex::new(None)),
            current_alert_level: Arc::new(Mutex::new(MemoryAlertLevel::Normal)),
            memory_manager,
        }
    }

    /// 启动内存监控
    pub fn start(&self) {
        if let Ok(mut running) = self.is_running.lock() {
            if *running {
                warn!("Enhanced memory monitor is already running");
                return;
            }
            *running = true;
        }

        let system = self.system.clone();
        let config = self.config.clone();
        let is_running = self.is_running.clone();
        let usage_history = self.usage_history.clone();
        let last_alert = self.last_alert.clone();
        let current_alert_level = self.current_alert_level.clone();
        let memory_manager = self.memory_manager.clone();

        thread::spawn(move || {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build();
            let runtime = match runtime {
                Ok(r) => r,
                Err(_) => return,
            };

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
                    let (total_memory, used_memory, usage_percent) = {
                        let sys = match system.lock() {
                            Ok(s) => s,
                            Err(_) => continue,
                        };
                        let total = sys.total_memory();
                        let used = sys.used_memory();
                        let usage_percent = (used as f32 / total as f32) * 100.0;
                        (total, used, usage_percent)
                    };

                    // 记录内存使用历史
                    if let Ok(mut history) = usage_history.lock() {
                        let now = std::time::Instant::now();
                        history.push(MemoryUsageHistory {
                            timestamp: now,
                            usage_percent,
                            total_memory,
                            used_memory,
                        });
                        // 清理过期历史记录
                        history.retain(|entry| {
                            now.duration_since(entry.timestamp) < config.history_retention
                        });
                    }

                    // 确定警报级别
                    let alert_level = {
                        if config.enable_memory_limit {
                            if let Some(limit) = config.memory_limit {
                                if used_memory > limit {
                                    MemoryAlertLevel::ExceededLimit
                                } else {
                                    Self::determine_alert_level(
                                        usage_percent,
                                        config.warning_threshold,
                                        config.critical_threshold,
                                    )
                                }
                            } else {
                                Self::determine_alert_level(
                                    usage_percent,
                                    config.warning_threshold,
                                    config.critical_threshold,
                                )
                            }
                        } else {
                            Self::determine_alert_level(
                                usage_percent,
                                config.warning_threshold,
                                config.critical_threshold,
                            )
                        }
                    };

                    // 检查警报级别变化
                    if let (Ok(mut current_level), Ok(mut last_alert_time)) =
                        (current_alert_level.lock(), last_alert.lock())
                    {
                        let now = std::time::Instant::now();
                        // 检查是否需要发送警报
                        if alert_level != *current_level
                            || (*current_level != MemoryAlertLevel::Normal
                                && last_alert_time
                                    .as_ref()
                                    .map(|t| now.duration_since(*t) > config.alert_cooldown)
                                    .unwrap_or(true))
                        {
                            Self::send_alert(
                                &alert_level,
                                used_memory,
                                total_memory,
                                config.memory_limit,
                            );
                            *last_alert_time = Some(now);
                            *current_level = alert_level.clone();
                        }
                    }

                    // 处理内存限制超限
                    if alert_level == MemoryAlertLevel::ExceededLimit {
                        error!(
                            "Memory limit exceeded: {} bytes (limit: {:?} bytes)",
                            used_memory, config.memory_limit
                        );

                        // 尝试强制垃圾回收
                        if let Some(manager) = &memory_manager {
                            info!("Forcing garbage collection...");
                            manager.force_gc();
                        }

                        // 暂停非关键后台任务
                        Self::pause_non_critical_tasks().await;
                    }

                    // 等待下一次检查
                    tokio::time::sleep(config.check_interval).await;
                }
            });
        });

        info!("Enhanced memory monitor started");
    }

    /// 停止内存监控
    pub fn stop(&self) {
        if let Ok(mut running) = self.is_running.lock() {
            *running = false;
        }
        info!("Enhanced memory monitor stopped");
    }

    /// 获取当前内存使用情况
    pub fn get_memory_usage(&self) -> (u64, u64, f32) {
        if let Ok(mut sys) = self.system.lock() {
            sys.refresh_all();
        }
        match self.system.lock() {
            Ok(system_info) => {
                let total_memory = system_info.total_memory();
                let used_memory = system_info.used_memory();
                let memory_usage_percent = (used_memory as f32 / total_memory as f32) * 100.0;
                (total_memory, used_memory, memory_usage_percent)
            }
            Err(_) => (0, 0, 0.0),
        }
    }

    /// 获取内存使用历史
    pub fn get_usage_history(&self) -> Vec<MemoryUsageHistory> {
        self.usage_history
            .lock()
            .ok()
            .map(|h| h.clone())
            .unwrap_or_default()
    }

    /// 获取当前警报级别
    pub fn get_current_alert_level(&self) -> MemoryAlertLevel {
        self.current_alert_level
            .lock()
            .ok()
            .map(|l| l.clone())
            .unwrap_or(MemoryAlertLevel::Normal)
    }

    /// 确定警报级别
    fn determine_alert_level(
        usage_percent: f32,
        warning_threshold: f32,
        critical_threshold: f32,
    ) -> MemoryAlertLevel {
        if usage_percent >= critical_threshold {
            MemoryAlertLevel::Critical
        } else if usage_percent >= warning_threshold {
            MemoryAlertLevel::Warning
        } else {
            MemoryAlertLevel::Normal
        }
    }

    /// 发送警报
    fn send_alert(
        alert_level: &MemoryAlertLevel,
        used_memory: u64,
        total_memory: u64,
        memory_limit: Option<u64>,
    ) {
        match alert_level {
            MemoryAlertLevel::Normal => {
                info!(
                    "Memory usage back to normal: {:.2}% ({} MB / {} MB)",
                    (used_memory as f32 / total_memory as f32) * 100.0,
                    used_memory / 1024 / 1024,
                    total_memory / 1024 / 1024
                );
            }
            MemoryAlertLevel::Warning => {
                warn!(
                    "Memory usage warning: {:.2}% ({} MB / {} MB)",
                    (used_memory as f32 / total_memory as f32) * 100.0,
                    used_memory / 1024 / 1024,
                    total_memory / 1024 / 1024
                );
            }
            MemoryAlertLevel::Critical => {
                error!(
                    "Memory usage critical: {:.2}% ({} MB / {} MB)",
                    (used_memory as f32 / total_memory as f32) * 100.0,
                    used_memory / 1024 / 1024,
                    total_memory / 1024 / 1024
                );
            }
            MemoryAlertLevel::ExceededLimit => {
                error!(
                    "Memory limit exceeded: {} MB (limit: {:?} MB)",
                    used_memory / 1024 / 1024,
                    memory_limit.map(|l| l / 1024 / 1024)
                );
            }
        }
    }

    /// 暂停非关键后台任务
    async fn pause_non_critical_tasks() {
        // 这里可以实现暂停非关键后台任务的逻辑
        // 例如:减少后台工作线程数量,延迟非紧急任务等
        info!("Pausing non-critical background tasks due to high memory usage");
    }
}

/// 增强内存监控服务
pub struct EnhancedMemoryMonitoringService {
    monitor: Arc<EnhancedMemoryMonitor>,
    memory_manager: Option<Arc<MemoryManager>>,
}

impl EnhancedMemoryMonitoringService {
    /// 创建新的内存监控服务
    pub fn new(
        config: EnhancedMemoryMonitorConfig,
        memory_manager: Option<Arc<MemoryManager>>,
    ) -> Self {
        let monitor = Arc::new(EnhancedMemoryMonitor::new(config, memory_manager.clone()));

        Self {
            monitor,
            memory_manager,
        }
    }

    /// 启动服务
    pub fn start(&self) {
        self.monitor.start();
        info!("Enhanced memory monitoring service started");
    }

    /// 停止服务
    pub fn stop(&self) {
        self.monitor.stop();
        info!("Enhanced memory monitoring service stopped");
    }

    /// 获取内存监控器
    pub fn get_monitor(&self) -> Arc<EnhancedMemoryMonitor> {
        self.monitor.clone()
    }

    /// 获取内存管理器
    pub fn get_memory_manager(&self) -> Option<Arc<MemoryManager>> {
        self.memory_manager.clone()
    }
}
