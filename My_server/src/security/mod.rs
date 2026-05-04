//! 安全审计模块
//! 提供安全日志记录、漏洞扫描和安全配置检查等功能

mod audit_log;
pub mod jwt_blacklist;
mod security_config;
mod threat_detection;
mod vulnerability_scanner;

pub use audit_log::{AuditLogEntry, AuditLogLevel, AuditLogger};
pub use security_config::{SecurityConfig, SecurityLevel};
pub use threat_detection::{ThreatDetector, ThreatEvent};
pub use vulnerability_scanner::{Vulnerability, VulnerabilityScanner};

/// 安全管理器
/// 整合所有安全相关功能
pub struct SecurityManager {
    /// 审计日志记录器
    audit_logger: AuditLogger,
    /// 漏洞扫描器
    vulnerability_scanner: VulnerabilityScanner,
    /// 安全配置
    security_config: SecurityConfig,
    /// 威胁检测器
    threat_detector: ThreatDetector,
}

impl SecurityManager {
    /// 创建新的安全管理器
    pub async fn new() -> Result<Self, anyhow::Error> {
        Ok(Self {
            audit_logger: AuditLogger::new().await?,
            vulnerability_scanner: VulnerabilityScanner::new().await?,
            security_config: SecurityConfig::load().await?,
            threat_detector: ThreatDetector::new().await?,
        })
    }

    /// 记录审计日志
    pub async fn log_audit(
        &self,
        level: AuditLogLevel,
        message: &str,
        details: Option<serde_json::Value>,
    ) {
        self.audit_logger.log(level, message, details).await;
    }

    /// 执行漏洞扫描
    pub async fn run_vulnerability_scan(&self) -> Result<Vec<Vulnerability>, anyhow::Error> {
        self.vulnerability_scanner.scan().await
    }

    /// 检查安全配置
    pub async fn check_security_config(&self) -> Result<Vec<String>, anyhow::Error> {
        self.security_config.check().await
    }

    /// 检测威胁
    pub async fn detect_threats(&self) -> Result<Vec<ThreatEvent>, anyhow::Error> {
        self.threat_detector.detect().await
    }

    /// 生成安全报告
    pub async fn generate_security_report(&self) -> Result<serde_json::Value, anyhow::Error> {
        let vulnerabilities = self.run_vulnerability_scan().await?;
        let config_issues = self.check_security_config().await?;
        let threats = self.detect_threats().await?;

        Ok(serde_json::json!({
            "timestamp": chrono::Utc::now(),
            "vulnerabilities": vulnerabilities,
            "config_issues": config_issues,
            "threats": threats,
            "security_level": self.security_config.get_level(),
        }))
    }

    /// 执行安全评估
    pub async fn run_security_assessment(&self) -> Result<serde_json::Value, anyhow::Error> {
        self.log_audit(AuditLogLevel::Info, "开始安全评估", None)
            .await;

        let report = self.generate_security_report().await?;

        self.log_audit(AuditLogLevel::Info, "安全评估完成", Some(report.clone()))
            .await;

        Ok(report)
    }
}
