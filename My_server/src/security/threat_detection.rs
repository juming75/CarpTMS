//! 威胁检测模块
//! 提供威胁检测功能

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::time::{Duration, Instant};

/// 威胁类型
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub enum ThreatType {
    /// 暴力破解
    BruteForce,
    /// SQL注入
    SQLInjection,
    /// 跨站脚本
    XSS,
    /// 命令注入
    CommandInjection,
    /// 敏感信息访问
    SensitiveInformationAccess,
    /// 异常访问模式
    AbnormalAccessPattern,
    /// 其他
    Other,
}

/// 威胁事件
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ThreatEvent {
    /// 事件ID
    pub id: String,
    /// 威胁类型
    pub r#type: ThreatType,
    /// 威胁描述
    pub description: String,
    /// 威胁级别
    pub severity: String,
    /// 发生时间
    pub occurred_at: chrono::DateTime<chrono::Utc>,
    /// 源IP地址
    pub source_ip: String,
    /// 目标资源
    pub target_resource: String,
    /// 详细信息
    pub details: serde_json::Value,
}

/// 访问记录
#[derive(Debug, Clone)]
pub struct AccessRecord {
    /// 访问时间
    pub timestamp: Instant,
    /// 访问路径
    pub path: String,
    /// 访问方法
    pub method: String,
    /// 状态码
    pub status_code: u16,
}

/// 威胁检测器
pub struct ThreatDetector {
    /// IP访问记录
    ip_access_records: Mutex<HashMap<String, Vec<AccessRecord>>>,
    /// 敏感路径
    sensitive_paths: Vec<String>,
    /// 可疑模式
    suspicious_patterns: Vec<(ThreatType, String)>,
}

impl ThreatDetector {
    /// 创建新的威胁检测器
    pub async fn new() -> Result<Self, anyhow::Error> {
        Ok(Self {
            ip_access_records: Mutex::new(HashMap::new()),
            sensitive_paths: vec![
                "/api/auth/login".to_string(),
                "/api/users".to_string(),
                "/api/vehicles".to_string(),
                "/api/orders".to_string(),
                "/api/devices".to_string(),
                "/api/drivers".to_string(),
            ],
            suspicious_patterns: vec![
                (ThreatType::SQLInjection, r#"' OR 1=1"#.to_string()),
                (ThreatType::SQLInjection, r#"UNION SELECT"#.to_string()),
                (ThreatType::XSS, r#"<script"#.to_string()),
                (ThreatType::XSS, r#"javascript:"#.to_string()),
                (ThreatType::CommandInjection, r#"; rm -rf"#.to_string()),
                (ThreatType::CommandInjection, r#"| cat /etc/passwd"#.to_string()),
            ],
        })
    }

    /// 检测威胁
    pub async fn detect(&self) -> Result<Vec<ThreatEvent>, anyhow::Error> {
        let mut threats = Vec::new();
        
        // 检测暴力破解
        self.detect_brute_force(&mut threats).await;
        
        // 检测SQL注入
        self.detect_sql_injection(&mut threats).await;
        
        // 检测XSS
        self.detect_xss(&mut threats).await;
        
        // 检测命令注入
        self.detect_command_injection(&mut threats).await;
        
        // 检测敏感信息访问
        self.detect_sensitive_information_access(&mut threats).await;
        
        // 检测异常访问模式
        self.detect_abnormal_access_pattern(&mut threats).await;
        
        Ok(threats)
    }

    /// 记录访问
    pub async fn record_access(&self, ip: &str, path: &str, method: &str, status_code: u16) {
        let mut ip_access_records = self.ip_access_records.lock().await;
        
        let records = ip_access_records.entry(ip.to_string()).or_insert(Vec::new());
        records.push(AccessRecord {
            timestamp: Instant::now(),
            path: path.to_string(),
            method: method.to_string(),
            status_code,
        });
        
        // 清理过期记录（保留最近1小时）
        let now = Instant::now();
        records.retain(|record| now.duration_since(record.timestamp) < Duration::from_hours(1));
    }

    /// 检测暴力破解
    async fn detect_brute_force(&self, threats: &mut Vec<ThreatEvent>) {
        let ip_access_records = self.ip_access_records.lock().await;
        
        for (ip, records) in &*ip_access_records {
            // 统计最近5分钟内的登录失败次数
            let now = Instant::now();
            let login_failures = records.iter()
                .filter(|record| 
                    record.path == "/api/auth/login" && 
                    record.status_code == 401 && 
                    now.duration_since(record.timestamp) < Duration::from_minutes(5)
                )
                .count();
            
            if login_failures > 5 {
                threats.push(ThreatEvent {
                    id: uuid::Uuid::new_v4().to_string(),
                    r#type: ThreatType::BruteForce,
                    description: "暴力破解尝试".to_string(),
                    severity: "high".to_string(),
                    occurred_at: chrono::Utc::now(),
                    source_ip: ip.to_string(),
                    target_resource: "/api/auth/login".to_string(),
                    details: serde_json::json!({
                        "login_failures": login_failures,
                        "time_window": "5 minutes"
                    }),
                });
            }
        }
    }

    /// 检测SQL注入
    async fn detect_sql_injection(&self, threats: &mut Vec<ThreatEvent>) {
        // 这里可以添加SQL注入检测逻辑
        // 例如检查请求参数中是否包含SQL注入攻击特征
    }

    /// 检测XSS
    async fn detect_xss(&self, threats: &mut Vec<ThreatEvent>) {
        // 这里可以添加XSS检测逻辑
        // 例如检查请求参数中是否包含XSS攻击特征
    }

    /// 检测命令注入
    async fn detect_command_injection(&self, threats: &mut Vec<ThreatEvent>) {
        // 这里可以添加命令注入检测逻辑
        // 例如检查请求参数中是否包含命令注入攻击特征
    }

    /// 检测敏感信息访问
    async fn detect_sensitive_information_access(&self, threats: &mut Vec<ThreatEvent>) {
        let ip_access_records = self.ip_access_records.lock().await;
        
        for (ip, records) in &*ip_access_records {
            // 统计最近10分钟内的敏感路径访问次数
            let now = Instant::now();
            let sensitive_accesses = records.iter()
                .filter(|record| 
                    self.sensitive_paths.contains(&record.path) && 
                    now.duration_since(record.timestamp) < Duration::from_minutes(10)
                )
                .count();
            
            if sensitive_accesses > 20 {
                threats.push(ThreatEvent {
                    id: uuid::Uuid::new_v4().to_string(),
                    r#type: ThreatType::SensitiveInformationAccess,
                    description: "敏感信息访问异常".to_string(),
                    severity: "medium".to_string(),
                    occurred_at: chrono::Utc::now(),
                    source_ip: ip.to_string(),
                    target_resource: "multiple".to_string(),
                    details: serde_json::json!({
                        "sensitive_accesses": sensitive_accesses,
                        "time_window": "10 minutes"
                    }),
                });
            }
        }
    }

    /// 检测异常访问模式
    async fn detect_abnormal_access_pattern(&self, threats: &mut Vec<ThreatEvent>) {
        let ip_access_records = self.ip_access_records.lock().await;
        
        for (ip, records) in &*ip_access_records {
            // 检查是否存在快速连续访问
            let now = Instant::now();
            let recent_accesses = records.iter()
                .filter(|record| now.duration_since(record.timestamp) < Duration::from_seconds(10))
                .count();
            
            if recent_accesses > 50 {
                threats.push(ThreatEvent {
                    id: uuid::Uuid::new_v4().to_string(),
                    r#type: ThreatType::AbnormalAccessPattern,
                    description: "异常访问模式".to_string(),
                    severity: "medium".to_string(),
                    occurred_at: chrono::Utc::now(),
                    source_ip: ip.to_string(),
                    target_resource: "multiple".to_string(),
                    details: serde_json::json!({
                        "accesses_per_10_seconds": recent_accesses
                    }),
                });
            }
        }
    }

    /// 添加敏感路径
    pub fn add_sensitive_path(&mut self, path: String) {
        self.sensitive_paths.push(path);
    }

    /// 添加可疑模式
    pub fn add_suspicious_pattern(&mut self, threat_type: ThreatType, pattern: String) {
        self.suspicious_patterns.push((threat_type, pattern));
    }
}
