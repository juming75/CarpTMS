//! /! 资源使用告警模块
//!
//! 提供系统资源使用情况的监控和告警功能

use log::{info, warn};
use std::sync::Arc;
use std::time::{Duration, Instant};
use sysinfo::{CpuExt, DiskExt, System, SystemExt};
use tokio::sync::RwLock;

/// 资源告警配置
#[derive(Debug, Clone)]
pub struct ResourceAlertConfig {
    /// CPU使用率阈值 (%)
    pub cpu_threshold: f32,
    /// 内存使用率阈值 (%)
    pub memory_threshold: f32,
    /// 磁盘使用率阈值 (%)
    pub disk_threshold: f32,
    /// 告警检查间隔
    pub check_interval: Duration,
    /// 告警冷却时间
    pub cooldown_period: Duration,
}

impl Default for ResourceAlertConfig {
    fn default() -> Self {
        Self {
            cpu_threshold: 80.0,
            memory_threshold: 85.0,
            disk_threshold: 90.0,
            check_interval: Duration::from_secs(60),
            cooldown_period: Duration::from_secs(300), // 5分钟
        }
    }
}

/// 资源告警状态
#[derive(Debug, Clone, Default)]
pub struct ResourceAlertState {
    pub last_cpu_alert: Option<Instant>,
    pub last_memory_alert: Option<Instant>,
    pub last_disk_alert: Option<Instant>,
}

/// 资源告警服务
pub struct ResourceAlertService {
    config: ResourceAlertConfig,
    state: Arc<RwLock<ResourceAlertState>>,
    system: Arc<RwLock<System>>,
    is_running: Arc<RwLock<bool>>,
}

impl ResourceAlertService {
    /// 创建新的资源告警服务
    pub fn new(config: ResourceAlertConfig) -> Self {
        let mut system = System::new_all();
        system.refresh_all();

        Self {
            config,
            state: Arc::new(RwLock::new(ResourceAlertState::default())),
            system: Arc::new(RwLock::new(system)),
            is_running: Arc::new(RwLock::new(false)),
        }
    }

    /// 启动资源告警服务
    pub async fn start(&self) {
        let mut running = self.is_running.write().await;

        if *running {
            warn!("Resource alert service is already running");
            return;
        }
        *running = true;
        drop(running);

        let config = self.config.clone();
        let state = self.state.clone();
        let system = self.system.clone();
        let is_running = self.is_running.clone();

        tokio::spawn(async move {
            loop {
                if !*is_running.read().await {
                    break;
                }

                // 检查资源使用情况
                {
                    let mut sys = system.write().await;
                    sys.refresh_all();

                    // 检查CPU使用率
                    let cpus = sys.cpus();
                    let cpu_usage =
                        cpus.iter().map(|c| c.cpu_usage()).sum::<f32>() / cpus.len() as f32;
                    if cpu_usage > config.cpu_threshold {
                        let mut state = state.write().await;
                        if Self::should_alert(&state.last_cpu_alert, &config.cooldown_period) {
                            warn!(
                                "CPU usage alert: {:.2}% (threshold: {:.2}%)",
                                cpu_usage, config.cpu_threshold
                            );
                            state.last_cpu_alert = Some(Instant::now());
                        }
                    }

                    // 检查内存使用率
                    let memory_usage =
                        (sys.used_memory() as f32 / sys.total_memory() as f32) * 100.0;
                    if memory_usage > config.memory_threshold {
                        let mut state = state.write().await;
                        if Self::should_alert(&state.last_memory_alert, &config.cooldown_period) {
                            warn!(
                                "Memory usage alert: {:.2}% (threshold: {:.2}%)",
                                memory_usage, config.memory_threshold
                            );
                            state.last_memory_alert = Some(Instant::now());
                        }
                    }

                    // 检查磁盘使用率
                    if let Some(disk) = sys.disks().first() {
                        let total = disk.total_space();
                        let available = disk.available_space();
                        let used = total - available;
                        let used_percent = (used as f32 / total as f32) * 100.0;
                        if used_percent > config.disk_threshold {
                            let mut state = state.write().await;
                            if Self::should_alert(&state.last_disk_alert, &config.cooldown_period) {
                                warn!(
                                    "Disk usage alert: {:.2}% (threshold: {:.2}%)",
                                    used_percent, config.disk_threshold
                                );
                                state.last_disk_alert = Some(Instant::now());
                            }
                        }
                    }
                }

                // 等待下一次检查
                tokio::time::sleep(config.check_interval).await;
            }
        });

        info!(
            "Resource alert service started with config: {:?}",
            self.config
        );
    }

    /// 停止资源告警服务
    pub async fn stop(&self) {
        let mut running = self.is_running.write().await;
        *running = false;
        drop(running);
        info!("Resource alert service stopped");
    }

    /// 检查是否应该发送告警
    fn should_alert(last_alert: &Option<Instant>, cooldown_period: &Duration) -> bool {
        match last_alert {
            None => true,
            Some(last) => {
                // 检查是否超过冷却期
                Instant::now().duration_since(*last) >= *cooldown_period
            }
        }
    }

    /// 获取当前资源使用情况
    pub async fn get_resource_usage(&self) -> ResourceUsage {
        let mut sys = self.system.write().await;
        sys.refresh_all();

        let cpus = sys.cpus();
        let cpu_usage = cpus.iter().map(|c| c.cpu_usage()).sum::<f32>() / cpus.len() as f32;
        let memory_usage = (sys.used_memory() as f32 / sys.total_memory() as f32) * 100.0;

        let (total_disk, used_disk, disk_usage) = if let Some(disk) = sys.disks().first() {
            let total = disk.total_space();
            let available = disk.available_space();
            let used = total - available;
            let usage = (used as f32 / total as f32) * 100.0;
            (total, used, usage)
        } else {
            (0, 0, 0.0)
        };

        ResourceUsage {
            cpu_usage,
            memory_usage,
            disk_usage,
            total_memory: sys.total_memory(),
            used_memory: sys.used_memory(),
            total_disk,
            used_disk,
        }
    }
}

/// 资源使用情况
#[derive(Debug, Clone)]
pub struct ResourceUsage {
    pub cpu_usage: f32,
    pub memory_usage: f32,
    pub disk_usage: f32,
    pub total_memory: u64,
    pub used_memory: u64,
    pub total_disk: u64,
    pub used_disk: u64,
}
