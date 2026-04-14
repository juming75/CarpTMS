//! 系统监控服务
//! 负责监控系统进程状态和资源使用情况

use log::info;
use std::time::{Duration, UNIX_EPOCH};
use sysinfo::{System, SystemExt, ProcessExt, CpuExt, Pid, PidExt};

/// 系统监控服务配置
#[derive(Debug, Clone, Default)]
pub struct SystemMonitorConfig {
    /// 检查间隔（秒）
    pub check_interval: u64,
    /// 最大进程数量
    pub max_processes: usize,
    /// 启用详细信息
    pub enable_details: bool,
}

/// 进程信息
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProcessInfo {
    /// 进程ID
    pub pid: i32,
    /// 进程名称
    pub name: String,
    /// 进程状态
    pub status: String,
    /// CPU使用率（%）
    pub cpu_usage: f32,
    /// 内存使用量（MB）
    pub memory_usage: u64,
    /// 进程启动时间
    pub start_time: Option<std::time::SystemTime>,
}

/// 系统状态信息
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SystemStatus {
    /// 系统总内存（MB）
    pub total_memory: u64,
    /// 系统可用内存（MB）
    pub available_memory: u64,
    /// 系统CPU使用率（%）
    pub cpu_usage: f32,
    /// 系统进程数量
    pub process_count: usize,
    /// 系统负载
    pub load_avg: (f64, f64, f64),
    /// 系统启动时间
    pub boot_time: std::time::SystemTime,
    /// 系统名称
    pub system_name: String,
    /// 系统版本
    pub system_version: String,
    /// 系统内核版本
    pub kernel_version: String,
}

/// 系统监控服务
#[derive(Clone)]
pub struct SystemMonitorService {
    config: SystemMonitorConfig,
}

impl SystemMonitorService {
    /// 创建系统监控服务实例
    pub fn new(config: SystemMonitorConfig) -> Self {
        Self {
            config,
        }
    }

    /// 启动系统监控服务
    pub fn start(&self) {
        let interval = self.config.check_interval;

        std::thread::spawn(move || {
            loop {
                let mut sys = System::new_all();
                // 刷新系统信息
                sys.refresh_all();
                std::thread::sleep(Duration::from_secs(interval));
            }
        });

        info!("System monitor service started with interval: {} seconds", interval);
    }

    /// 获取系统状态
    pub fn get_system_status(&self) -> SystemStatus {
        let mut sys = System::new_all();
        sys.refresh_all();

        SystemStatus {
            total_memory: sys.total_memory() / (1024 * 1024), // 转换为MB
            available_memory: sys.available_memory() / (1024 * 1024), // 转换为MB
            cpu_usage: sys.global_cpu_info().cpu_usage(),
            process_count: sys.processes().len(),
            load_avg: (0.0, 0.0, 0.0), // 简化处理
            boot_time: UNIX_EPOCH + Duration::from_secs(sys.boot_time()),
            system_name: sys.name().unwrap_or_else(|| "Unknown".to_string()),
            system_version: sys.os_version().unwrap_or_else(|| "Unknown".to_string()),
            kernel_version: sys.kernel_version().unwrap_or_else(|| "Unknown".to_string()),
        }
    }

    /// 获取进程列表
    pub fn get_processes(&self) -> Vec<ProcessInfo> {
        let mut sys = System::new_all();
        sys.refresh_all();

        let mut processes: Vec<ProcessInfo> = sys
            .processes()
            .values()
            .take(self.config.max_processes)
            .map(|process| ProcessInfo {
                pid: process.pid().as_u32() as i32,
                name: process.name().to_string(),
                status: process.status().to_string(),
                cpu_usage: process.cpu_usage(),
                memory_usage: process.memory() / (1024 * 1024), // 转换为MB
                start_time: Some(UNIX_EPOCH + Duration::from_secs(process.start_time())),
            })
            .collect();

        // 按CPU使用率排序
        processes.sort_by(|a, b| b.cpu_usage.partial_cmp(&a.cpu_usage).unwrap());

        processes
    }

    /// 根据PID获取进程信息
    pub fn get_process_by_pid(&self, pid: i32) -> Option<ProcessInfo> {
        let mut sys = System::new_all();
        sys.refresh_all();

        sys.processes()
            .get(&Pid::from(pid as usize))
            .map(|process| ProcessInfo {
                pid: process.pid().as_u32() as i32,
                name: process.name().to_string(),
                status: process.status().to_string(),
                cpu_usage: process.cpu_usage(),
                memory_usage: process.memory() / (1024 * 1024), // 转换为MB
                start_time: Some(UNIX_EPOCH + Duration::from_secs(process.start_time())),
            })
    }

    /// 获取系统负载
    pub fn get_system_load(&self) -> (f64, f64, f64) {
        let mut sys = System::new_all();
        sys.refresh_all();
        (0.0, 0.0, 0.0) // 简化处理
    }

    /// 获取内存使用情况
    pub fn get_memory_usage(&self) -> (u64, u64) {
        let mut sys = System::new_all();
        sys.refresh_all();
        (
            sys.total_memory() / (1024 * 1024), // 转换为MB
            sys.available_memory() / (1024 * 1024), // 转换为MB
        )
    }
}

/// 应用服务接口实现
#[async_trait::async_trait]
impl crate::domain::use_cases::application_service::ApplicationService for SystemMonitorService {
    fn name(&self) -> &str {
        "system_monitor_service"
    }

    fn initialize(&self) -> anyhow::Result<()> {
        // 初始化逻辑
        self.start();
        Ok(())
    }

    async fn execute(&self, _input: serde_json::Value) -> anyhow::Result<serde_json::Value> {
        // 通用执行方法
        let system_status = self.get_system_status();
        Ok(serde_json::to_value(system_status)?)
    }
}
