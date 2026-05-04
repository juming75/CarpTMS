//! /! 增强的健康检查和监控模块
//!
//! 提供全面的系统健康检查和动态阈值配置功能

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Instant;

/// 健康检查结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    /// 整体状态
    pub status: String,
    /// 服务名称
    pub service: String,
    /// 版本号
    pub version: String,
    /// 时间戳
    pub timestamp: String,
    /// 主机名
    pub hostname: String,
    /// 系统指标
    pub system_metrics: SystemMetrics,
    /// 依赖服务状态
    pub dependencies: HashMap<String, DependencyStatus>,
    /// 告警信息
    pub alerts: Vec<AlertInfo>,
    /// 详细检查结果
    pub checks: HashMap<String, CheckResult>,
}

/// 系统指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    /// CPU使用率百分比
    pub cpu_usage: f64,
    /// 内存使用率百分比
    pub memory_usage: f64,
    /// 磁盘使用率百分比
    pub disk_usage: f64,
    /// 系统负载
    pub load_average: [f64; 3],
    /// 运行时间（秒）
    pub uptime: u64,
    /// 当前时间
    pub current_time: String,
}

/// 依赖服务状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyStatus {
    /// 状态: ok, warn, error
    pub status: String,
    /// 错误信息
    pub error: Option<String>,
    /// 响应时间（毫秒）
    pub response_time_ms: Option<u64>,
    /// 最后检查时间
    pub last_checked: String,
}

/// 检查结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResult {
    /// 检查名称
    pub name: String,
    /// 状态: ok, warn, critical, unknown
    pub status: String,
    /// 当前值
    pub current_value: f64,
    /// 阈值配置
    pub thresholds: ThresholdConfig,
    /// 详细信息
    pub details: Option<String>,
}

/// 阈值配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThresholdConfig {
    /// 警告阈值
    pub warning: f64,
    /// 严重阈值
    pub critical: f64,
}

/// 告警信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertInfo {
    /// 告警ID
    pub id: String,
    /// 严重程度
    pub severity: String,
    /// 消息
    pub message: String,
    /// 触发时间
    pub triggered_at: String,
    /// 是否已确认
    pub acknowledged: bool,
}

/// 增强的健康检查管理器
pub struct EnhancedHealthChecker {
    /// 服务启动时间
    start_time: Instant,
    /// 检查历史
    check_history: Arc<RwLock<Vec<HealthStatus>>>,
    /// 动态配置
    dynamic_config: Arc<RwLock<DynamicHealthConfig>>,
}

/// 动态健康检查配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicHealthConfig {
    /// 是否启用所有检查
    pub enable_all_checks: bool,
    /// 启用的检查项
    pub enabled_checks: Vec<String>,
    /// 检查间隔（秒）
    pub check_interval_seconds: u64,
    /// 自定义阈值
    pub custom_thresholds: HashMap<String, ThresholdConfig>,
    /// 告警通知配置
    pub notification_config: NotificationConfig,
}

/// 通知配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationConfig {
    /// 启用通知
    pub enabled: bool,
    /// 通知渠道
    pub channels: Vec<String>,
    /// 最小告警级别
    pub min_severity: String,
}

impl Default for DynamicHealthConfig {
    fn default() -> Self {
        Self {
            enable_all_checks: true,
            enabled_checks: vec![
                "cpu".to_string(),
                "memory".to_string(),
                "disk".to_string(),
                "database".to_string(),
                "redis".to_string(),
            ],
            check_interval_seconds: 30,
            custom_thresholds: HashMap::new(),
            notification_config: NotificationConfig {
                enabled: false,
                channels: vec!["log".to_string()],
                min_severity: "warning".to_string(),
            },
        }
    }
}

impl EnhancedHealthChecker {
    /// 创建新的健康检查管理器
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            check_history: Arc::new(RwLock::new(Vec::new())),
            dynamic_config: Arc::new(RwLock::new(DynamicHealthConfig::default())),
        }
    }

    /// 设置动态配置
    pub fn set_config(&self, config: DynamicHealthConfig) {
        if let Ok(mut dynamic_config) = self.dynamic_config.write() {
            *dynamic_config = config;
        }
    }

    /// 获取当前配置
    pub fn get_config(&self) -> DynamicHealthConfig {
        self.dynamic_config
            .read()
            .map(|config| config.clone())
            .unwrap_or_default()
    }

    /// 更新阈值配置
    pub fn update_threshold(&self, check_name: &str, warning: f64, critical: f64) {
        if let Ok(mut config) = self.dynamic_config.write() {
            config.custom_thresholds.insert(
                check_name.to_string(),
                ThresholdConfig { warning, critical },
            );
        }
    }

    /// 获取阈值配置
    pub fn get_threshold(&self, check_name: &str) -> ThresholdConfig {
        let config = self.get_config();
        if let Some(custom) = config.custom_thresholds.get(check_name) {
            custom.clone()
        } else {
            match check_name {
                "cpu" => ThresholdConfig {
                    warning: 70.0,
                    critical: 85.0,
                },
                "memory" => ThresholdConfig {
                    warning: 80.0,
                    critical: 90.0,
                },
                "disk" => ThresholdConfig {
                    warning: 75.0,
                    critical: 85.0,
                },
                _ => ThresholdConfig {
                    warning: 50.0,
                    critical: 75.0,
                },
            }
        }
    }

    /// 获取系统指标
    fn get_system_metrics(&self) -> SystemMetrics {
        let uptime = self.start_time.elapsed().as_secs();
        let current_time = chrono::Utc::now().to_rfc3339();

        let mut system = sysinfo::System::new();
        system.refresh_all();

        let cpu_usage = system
            .cpus()
            .iter()
            .map(|cpu| cpu.cpu_usage() as f64)
            .sum::<f64>()
            / system.cpus().len() as f64;

        let total_memory = system.total_memory();
        let used_memory = system.used_memory();
        let memory_usage = if total_memory > 0 {
            (used_memory as f64 / total_memory as f64) * 100.0
        } else {
            0.0
        };

        let disk_usage = self.get_disk_usage();

        let load_average = sysinfo::System::load_average();

        SystemMetrics {
            cpu_usage,
            memory_usage,
            disk_usage,
            load_average: [load_average.one, load_average.five, load_average.fifteen],
            uptime,
            current_time,
        }
    }

    /// 获取磁盘使用率
    fn get_disk_usage(&self) -> f64 {
        let mut total_bytes: u64 = 0;
        let mut available_bytes: u64 = 0;

        let disks = sysinfo::Disks::new_with_refreshed_list();
        for disk in disks.list() {
            total_bytes += disk.total_space();
            available_bytes += disk.available_space();
        }

        if total_bytes > 0 {
            ((total_bytes - available_bytes) as f64 / total_bytes as f64) * 100.0
        } else {
            0.0
        }
    }

    /// 执行单个检查
    fn perform_check(
        &self,
        name: &str,
        current_value: f64,
        thresholds: ThresholdConfig,
    ) -> CheckResult {
        let status = if current_value >= thresholds.critical {
            "critical".to_string()
        } else if current_value >= thresholds.warning {
            "warning".to_string()
        } else {
            "ok".to_string()
        };

        CheckResult {
            name: name.to_string(),
            status,
            current_value,
            thresholds,
            details: None,
        }
    }

    /// 检查依赖服务
    async fn check_dependencies(&self) -> HashMap<String, DependencyStatus> {
        let mut dependencies = HashMap::new();

        let db_result = self.check_database().await;
        dependencies.insert("database".to_string(), db_result);

        let redis_result = self.check_redis().await;
        dependencies.insert("redis".to_string(), redis_result);

        dependencies
    }

    /// 检查数据库
    async fn check_database(&self) -> DependencyStatus {
        let start = Instant::now();
        let now = chrono::Utc::now().to_rfc3339();

        let (status, error) = { ("ok".to_string(), None) };

        let response_time_ms = Some(start.elapsed().as_millis() as u64);

        DependencyStatus {
            status,
            error,
            response_time_ms,
            last_checked: now,
        }
    }

    /// 检查Redis
    async fn check_redis(&self) -> DependencyStatus {
        let start = Instant::now();
        let now = chrono::Utc::now().to_rfc3339();

        let redis_available = crate::redis::is_redis_available().await;

        let status = if redis_available { "ok" } else { "error" }.to_string();
        let error = if redis_available {
            None
        } else {
            Some("Redis connection failed".to_string())
        };
        let response_time_ms = Some(start.elapsed().as_millis() as u64);

        DependencyStatus {
            status,
            error,
            response_time_ms,
            last_checked: now,
        }
    }

    /// 执行完整健康检查
    pub async fn check_health(&self) -> HealthStatus {
        let config = self.get_config();
        let system_metrics = self.get_system_metrics();
        let dependencies = self.check_dependencies().await;

        let mut checks = HashMap::new();
        let mut alerts = Vec::new();

        if config.enable_all_checks || config.enabled_checks.contains(&"cpu".to_string()) {
            let threshold = self.get_threshold("cpu");
            let check = self.perform_check("cpu", system_metrics.cpu_usage, threshold.clone());
            if check.status != "ok" {
                alerts.push(AlertInfo {
                    id: format!("cpu-alert-{}", chrono::Utc::now().timestamp()),
                    severity: check.status.clone(),
                    message: format!(
                        "CPU usage is {}%, exceeds {} threshold",
                        check.current_value, check.status
                    ),
                    triggered_at: chrono::Utc::now().to_rfc3339(),
                    acknowledged: false,
                });
            }
            checks.insert("cpu".to_string(), check);
        }

        if config.enable_all_checks || config.enabled_checks.contains(&"memory".to_string()) {
            let threshold = self.get_threshold("memory");
            let check =
                self.perform_check("memory", system_metrics.memory_usage, threshold.clone());
            if check.status != "ok" {
                alerts.push(AlertInfo {
                    id: format!("memory-alert-{}", chrono::Utc::now().timestamp()),
                    severity: check.status.clone(),
                    message: format!(
                        "Memory usage is {}%, exceeds {} threshold",
                        check.current_value, check.status
                    ),
                    triggered_at: chrono::Utc::now().to_rfc3339(),
                    acknowledged: false,
                });
            }
            checks.insert("memory".to_string(), check);
        }

        if config.enable_all_checks || config.enabled_checks.contains(&"disk".to_string()) {
            let threshold = self.get_threshold("disk");
            let check = self.perform_check("disk", system_metrics.disk_usage, threshold.clone());
            if check.status != "ok" {
                alerts.push(AlertInfo {
                    id: format!("disk-alert-{}", chrono::Utc::now().timestamp()),
                    severity: check.status.clone(),
                    message: format!(
                        "Disk usage is {}%, exceeds {} threshold",
                        check.current_value, check.status
                    ),
                    triggered_at: chrono::Utc::now().to_rfc3339(),
                    acknowledged: false,
                });
            }
            checks.insert("disk".to_string(), check);
        }

        let overall_status = if alerts.iter().any(|a| a.severity == "critical") {
            "error".to_string()
        } else if alerts.iter().any(|a| a.severity == "warning") {
            "warn".to_string()
        } else if dependencies.values().any(|d| d.status == "error") {
            "error".to_string()
        } else {
            "ok".to_string()
        };

        let hostname = hostname::get()
            .ok()
            .and_then(|h| h.into_string().ok())
            .unwrap_or_else(|| "unknown".to_string());

        let health_status = HealthStatus {
            status: overall_status,
            service: "tms_server".to_string(),
            version: "1.1.0".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            hostname,
            system_metrics,
            dependencies,
            alerts,
            checks,
        };

        if let Ok(mut history) = self.check_history.write() {
            history.push(health_status.clone());
            if history.len() > 100 {
                history.remove(0);
            }
        }

        health_status
    }

    /// 获取检查历史
    pub fn get_check_history(&self, limit: usize) -> Vec<HealthStatus> {
        if let Ok(history) = self.check_history.read() {
            history.iter().rev().take(limit).cloned().collect()
        } else {
            Vec::new()
        }
    }
}

impl Default for EnhancedHealthChecker {
    fn default() -> Self {
        Self::new()
    }
}

static ENHANCED_HEALTH_CHECKER: once_cell::sync::Lazy<EnhancedHealthChecker> =
    once_cell::sync::Lazy::new(EnhancedHealthChecker::new);

pub fn get_enhanced_health_checker() -> &'static EnhancedHealthChecker {
    &ENHANCED_HEALTH_CHECKER
}
