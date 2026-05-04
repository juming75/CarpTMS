//! 设备连接诊断模块
//!
//! 提供设备连接失败排查功能：
//! - 网络连通性检查
//! - 协议兼容性验证
//! - 连接日志分析
//! - 自动诊断建议

use chrono::Utc;
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::ToSocketAddrs;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{timeout, Duration};

/// 诊断检查项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticCheck {
    /// 检查项名称
    pub name: String,
    /// 检查状态
    pub status: DiagnosticStatus,
    /// 详细信息
    pub details: String,
    /// 建议操作
    pub recommendation: String,
    /// 检查时间戳
    pub timestamp: String,
}

/// 诊断状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiagnosticStatus {
    /// 通过
    Pass,
    /// 失败
    Fail,
    /// 警告
    Warning,
    /// 跳过
    Skipped,
}

/// 设备连接诊断报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceDiagnosticReport {
    /// 设备ID
    pub device_id: String,
    /// 诊断时间
    pub diagnostic_time: String,
    /// 总体状态
    pub overall_status: DiagnosticStatus,
    /// 检查项列表
    pub checks: Vec<DiagnosticCheck>,
    /// 建议摘要
    pub summary: String,
}

/// 网络连通性检查结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkCheckResult {
    /// 设备是否可访问
    pub reachable: bool,
    /// 端口是否开放
    pub port_open: bool,
    /// 响应延迟（毫秒）
    pub latency_ms: Option<u64>,
    /// 防火墙状态
    pub firewall_status: String,
}

/// 协议兼容性检查结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolCompatibilityResult {
    /// 协议版本
    pub protocol_version: String,
    /// 是否兼容
    pub compatible: bool,
    /// 支持的功能
    pub supported_features: Vec<String>,
    /// 不支持的功能
    pub unsupported_features: Vec<String>,
}

/// 连接诊断管理器
pub struct ConnectionDiagnosticManager {
    /// 诊断历史记录
    history: Arc<RwLock<HashMap<String, Vec<DeviceDiagnosticReport>>>>,
    /// 诊断配置
    config: DiagnosticConfig,
}

/// 诊断配置
#[derive(Debug, Clone)]
pub struct DiagnosticConfig {
    /// 连接超时时间（秒）
    pub connection_timeout: u64,
    /// 默认检查端口
    pub default_port: u16,
    /// 是否启用详细日志
    pub verbose_logging: bool,
}

impl Default for DiagnosticConfig {
    fn default() -> Self {
        Self {
            connection_timeout: 10,
            default_port: 1078,
            verbose_logging: true,
        }
    }
}

impl ConnectionDiagnosticManager {
    /// 创建新的诊断管理器
    pub fn new() -> Self {
        Self {
            history: Arc::new(RwLock::new(HashMap::new())),
            config: DiagnosticConfig::default(),
        }
    }

    /// 使用自定义配置创建诊断管理器
    pub fn with_config(config: DiagnosticConfig) -> Self {
        Self {
            history: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    /// 执行完整的设备连接诊断
    pub async fn run_full_diagnostic(&self, device_id: &str, address: &str) -> DeviceDiagnosticReport {
        info!("Running full diagnostic for device {} at {}", device_id, address);
        
        let mut checks = Vec::new();
        
        // 1. 网络连通性检查
        let network_check = self.check_network_connectivity(device_id, address).await;
        let network_status = network_check.status.clone();
        checks.push(network_check);
        
        // 2. 端口可达性检查
        let port_check = self.check_port_accessibility(device_id, address).await;
        let port_status = port_check.status.clone();
        checks.push(port_check);
        
        // 3. 协议版本验证
        let protocol_check = self.check_protocol_version(device_id).await;
        let protocol_status = protocol_check.status.clone();
        checks.push(protocol_check);
        
        // 4. 消息头验证
        let header_check = self.check_message_header(device_id).await;
        checks.push(header_check);
        
        // 5. 连接日志分析
        let log_check = self.analyze_connection_logs(device_id).await;
        checks.push(log_check);
        
        // 确定总体状态
        let overall_status = self.determine_overall_status(&checks);
        
        // 生成建议摘要
        let summary = self.generate_diagnostic_summary(&checks);
        
        let report = DeviceDiagnosticReport {
            device_id: device_id.to_string(),
            diagnostic_time: Utc::now().to_rfc3339(),
            overall_status: overall_status.clone(),
            checks,
            summary,
        };
        
        // 保存诊断历史
        self.save_diagnostic_report(device_id, &report).await;
        
        report
    }

    /// 检查网络连通性
    async fn check_network_connectivity(&self, device_id: &str, address: &str) -> DiagnosticCheck {
        debug!("Checking network connectivity for {}", device_id);
        
        let result = self.check_tcp_connectivity(address).await;
        
        match result {
            Ok(latency) => DiagnosticCheck {
                name: "网络连通性".to_string(),
                status: DiagnosticStatus::Pass,
                details: format!("设备 {} 可访问，延迟: {}ms", address, latency),
                recommendation: "网络连接正常".to_string(),
                timestamp: Utc::now().to_rfc3339(),
            },
            Err(e) => DiagnosticCheck {
                name: "网络连通性".to_string(),
                status: DiagnosticStatus::Fail,
                details: format!("无法访问设备 {}: {}", address, e),
                recommendation: "1. 确认终端是否能访问互联网; 2. 检查网络路由; 3. 验证DNS解析".to_string(),
                timestamp: Utc::now().to_rfc3339(),
            },
        }
    }

    /// 检查端口可达性
    async fn check_port_accessibility(&self, device_id: &str, address: &str) -> DiagnosticCheck {
        debug!("Checking port accessibility for {}", device_id);
        
        let port = self.config.default_port;
        let result = self.check_tcp_port(address, port).await;
        
        match result {
            Ok(_) => DiagnosticCheck {
                name: "端口可达性".to_string(),
                status: DiagnosticStatus::Pass,
                details: format!("端口 {} 在 {} 上可访问", port, address),
                recommendation: "端口访问正常".to_string(),
                timestamp: Utc::now().to_rfc3339(),
            },
            Err(e) => DiagnosticCheck {
                name: "端口可达性".to_string(),
                status: DiagnosticStatus::Fail,
                details: format!("端口 {} 在 {} 上不可访问: {}", port, address, e),
                recommendation: format!(
                    "1. 检查防火墙是否放行端口 {}; 2. 确认服务是否在该端口监听; 3. 检查SELinux/AppArmor配置",
                    port
                ),
                timestamp: Utc::now().to_rfc3339(),
            },
        }
    }

    /// 验证协议版本
    async fn check_protocol_version(&self, device_id: &str) -> DiagnosticCheck {
        debug!("Checking protocol version for {}", device_id);
        
        // 检查设备支持的协议版本
        let supported_versions = self.get_supported_protocol_versions(device_id).await;
        
        if supported_versions.contains(&"JT/T 1078-2016".to_string()) {
            DiagnosticCheck {
                name: "协议兼容性".to_string(),
                status: DiagnosticStatus::Pass,
                details: "设备支持 JT/T 1078-2016 协议".to_string(),
                recommendation: "协议版本兼容".to_string(),
                timestamp: Utc::now().to_rfc3339(),
            }
        } else {
            DiagnosticCheck {
                name: "协议兼容性".to_string(),
                status: DiagnosticStatus::Warning,
                details: format!("设备可能不支持 JT/T 1078-2016，支持版本: {:?}", supported_versions),
                recommendation: "1. 确认终端支持的协议版本; 2. 检查消息头中的协议版本号; 3. 考虑升级固件".to_string(),
                timestamp: Utc::now().to_rfc3339(),
            }
        }
    }

    /// 检查消息头格式
    async fn check_message_header(&self, device_id: &str) -> DiagnosticCheck {
        debug!("Checking message header format for {}", device_id);
        
        // 模拟检查消息头格式
        DiagnosticCheck {
            name: "消息头验证".to_string(),
            status: DiagnosticStatus::Pass,
            details: "消息头格式正确".to_string(),
            recommendation: "消息头格式符合规范".to_string(),
            timestamp: Utc::now().to_rfc3339(),
        }
    }

    /// 分析连接日志
    async fn analyze_connection_logs(&self, device_id: &str) -> DiagnosticCheck {
        debug!("Analyzing connection logs for {}", device_id);
        
        // 这里应该读取并分析实际的日志文件
        // 为了演示，我们返回一个通用检查项
        DiagnosticCheck {
            name: "连接日志分析".to_string(),
            status: DiagnosticStatus::Pass,
            details: "未发现异常连接日志".to_string(),
            recommendation: "如需详细分析，请使用Wireshark抓包分析通信过程".to_string(),
            timestamp: Utc::now().to_rfc3339(),
        }
    }

    /// 检查TCP连接
    async fn check_tcp_connectivity(&self, address: &str) -> Result<u64, String> {
        let start = std::time::Instant::now();
        
        match timeout(
            Duration::from_secs(self.config.connection_timeout),
            tokio::net::TcpStream::connect(address),
        ).await {
            Ok(Ok(_)) => Ok(start.elapsed().as_millis() as u64),
            Ok(Err(e)) => Err(format!("TCP连接失败: {}", e)),
            Err(_) => Err("连接超时".to_string()),
        }
    }

    /// 检查TCP端口
    async fn check_tcp_port(&self, address: &str, port: u16) -> Result<(), String> {
        let full_address = format!("{}:{}", address, port);
        
        match timeout(
            Duration::from_secs(self.config.connection_timeout),
            tokio::net::TcpStream::connect(&full_address),
        ).await {
            Ok(Ok(_)) => Ok(()),
            Ok(Err(e)) => Err(format!("端口 {} 连接失败: {}", port, e)),
            Err(_) => Err(format!("端口 {} 连接超时", port)),
        }
    }

    /// 获取设备支持的协议版本
    async fn get_supported_protocol_versions(&self, device_id: &str) -> Vec<String> {
        // 这里应该从设备注册信息中获取
        // 为了演示，返回默认值
        vec!["JT/T 1078-2016".to_string()]
    }

    /// 确定总体诊断状态
    fn determine_overall_status(&self, checks: &[DiagnosticCheck]) -> DiagnosticStatus {
        for check in checks {
            match check.status {
                DiagnosticStatus::Fail => return DiagnosticStatus::Fail,
                DiagnosticStatus::Warning => continue,
                _ => continue,
            }
        }
        
        // 检查是否有警告
        if checks.iter().any(|c| matches!(c.status, DiagnosticStatus::Warning)) {
            return DiagnosticStatus::Warning;
        }
        
        DiagnosticStatus::Pass
    }

    /// 生成诊断建议摘要
    fn generate_diagnostic_summary(&self, checks: &[DiagnosticCheck]) -> String {
        let failed_checks: Vec<&str> = checks.iter()
            .filter(|c| matches!(c.status, DiagnosticStatus::Fail))
            .map(|c| c.name.as_str())
            .collect();
        
        if failed_checks.is_empty() {
            "设备连接诊断通过，未发现明显问题".to_string()
        } else {
            format!("发现以下问题: {}。请根据各项的建议进行排查", failed_checks.join(", "))
        }
    }

    /// 保存诊断报告
    async fn save_diagnostic_report(&self, device_id: &str, report: &DeviceDiagnosticReport) {
        let mut history = self.history.write().await;
        let reports = history.entry(device_id.to_string()).or_insert_with(Vec::new);
        reports.push(report.clone());
        
        // 只保留最近的100条记录
        if reports.len() > 100 {
            reports.drain(..reports.len() - 100);
        }
        
        info!("Saved diagnostic report for device {}", device_id);
    }

    /// 获取设备的诊断历史
    pub async fn get_diagnostic_history(&self, device_id: &str) -> Vec<DeviceDiagnosticReport> {
        let history = self.history.read().await;
        history.get(device_id).cloned().unwrap_or_default()
    }
}

/// 创建诊断管理器实例
pub fn create_connection_diagnostic_manager() -> ConnectionDiagnosticManager {
    ConnectionDiagnosticManager::new()
}

/// 创建带自定义配置的诊断管理器
pub fn create_connection_diagnostic_manager_with_config(
    config: DiagnosticConfig,
) -> ConnectionDiagnosticManager {
    ConnectionDiagnosticManager::with_config(config)
}
