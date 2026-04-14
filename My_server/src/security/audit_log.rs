//! 审计日志模块
//! 提供安全审计日志记录功能

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use std::fs::File;
use std::io::Write;
use std::path::Path;

/// 审计日志级别
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub enum AuditLogLevel {
    /// 调试级别
    Debug,
    /// 信息级别
    Info,
    /// 警告级别
    Warning,
    /// 错误级别
    Error,
    /// 严重级别
    Critical,
}

/// 审计日志条目
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AuditLogEntry {
    /// 时间戳
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// 日志级别
    pub level: AuditLogLevel,
    /// 消息
    pub message: String,
    /// 详细信息
    pub details: Option<serde_json::Value>,
    /// 来源
    pub source: String,
    /// 用户
    pub user: Option<String>,
    /// IP地址
    pub ip_address: Option<String>,
}

/// 审计日志记录器
pub struct AuditLogger {
    /// 日志文件
    log_file: Mutex<Option<File>>,
    /// 日志级别
    log_level: AuditLogLevel,
}

impl AuditLogger {
    /// 创建新的审计日志记录器
    pub async fn new() -> Result<Self, anyhow::Error> {
        // 创建日志目录
        let log_dir = Path::new("logs");
        if !log_dir.exists() {
            std::fs::create_dir_all(log_dir)?;
        }
        
        // 创建日志文件
        let log_file_path = log_dir.join(format!("audit_{}.log", chrono::Local::now().format("%Y%m%d")));
        let log_file = File::create(log_file_path)?;
        
        Ok(Self {
            log_file: Mutex::new(Some(log_file)),
            log_level: AuditLogLevel::Info,
        })
    }

    /// 记录审计日志
    pub async fn log(&self, level: AuditLogLevel, message: &str, details: Option<serde_json::Value>) {
        // 检查日志级别
        if self.should_log(level) {
            let entry = AuditLogEntry {
                timestamp: chrono::Utc::now(),
                level,
                message: message.to_string(),
                details,
                source: "carptms_server".to_string(),
                user: None,
                ip_address: None,
            };
            
            // 写入日志文件
            if let Ok(mut log_file) = self.log_file.lock().await {
                if let Some(file) = log_file.as_mut() {
                    let log_entry = serde_json::to_string(&entry).unwrap();
                    writeln!(file, "{}", log_entry).unwrap();
                    file.flush().unwrap();
                }
            }
            
            // 同时输出到控制台
            self.log_to_console(&entry);
        }
    }

    /// 记录带用户和IP的审计日志
    pub async fn log_with_context(&self, level: AuditLogLevel, message: &str, details: Option<serde_json::Value>, user: Option<String>, ip_address: Option<String>) {
        // 检查日志级别
        if self.should_log(level) {
            let entry = AuditLogEntry {
                timestamp: chrono::Utc::now(),
                level,
                message: message.to_string(),
                details,
                source: "carptms_server".to_string(),
                user,
                ip_address,
            };
            
            // 写入日志文件
            if let Ok(mut log_file) = self.log_file.lock().await {
                if let Some(file) = log_file.as_mut() {
                    let log_entry = serde_json::to_string(&entry).unwrap();
                    writeln!(file, "{}", log_entry).unwrap();
                    file.flush().unwrap();
                }
            }
            
            // 同时输出到控制台
            self.log_to_console(&entry);
        }
    }

    /// 检查是否应该记录该级别的日志
    fn should_log(&self, level: AuditLogLevel) -> bool {
        match (&self.log_level, &level) {
            (AuditLogLevel::Debug, _) => true,
            (AuditLogLevel::Info, AuditLogLevel::Info | AuditLogLevel::Warning | AuditLogLevel::Error | AuditLogLevel::Critical) => true,
            (AuditLogLevel::Warning, AuditLogLevel::Warning | AuditLogLevel::Error | AuditLogLevel::Critical) => true,
            (AuditLogLevel::Error, AuditLogLevel::Error | AuditLogLevel::Critical) => true,
            (AuditLogLevel::Critical, AuditLogLevel::Critical) => true,
            _ => false,
        }
    }

    /// 输出到控制台
    fn log_to_console(&self, entry: &AuditLogEntry) {
        let level_str = match entry.level {
            AuditLogLevel::Debug => "DEBUG",
            AuditLogLevel::Info => "INFO",
            AuditLogLevel::Warning => "WARNING",
            AuditLogLevel::Error => "ERROR",
            AuditLogLevel::Critical => "CRITICAL",
        };
        
        let timestamp = entry.timestamp.format("%Y-%m-%d %H:%M:%S");
        let user_str = entry.user.as_ref().map(|u| format!(" [{}]", u)).unwrap_or_else(|| "".to_string());
        let ip_str = entry.ip_address.as_ref().map(|ip| format!(" [{}]", ip)).unwrap_or_else(|| "".to_string());
        
        println!("[{}] [{}]{}{} {}", timestamp, level_str, user_str, ip_str, entry.message);
        
        if let Some(details) = &entry.details {
            println!("Details: {}", details);
        }
    }

    /// 设置日志级别
    pub fn set_log_level(&mut self, level: AuditLogLevel) {
        self.log_level = level;
    }

    /// 轮换日志文件
    pub async fn rotate_log(&self) -> Result<(), anyhow::Error> {
        let log_dir = Path::new("logs");
        if !log_dir.exists() {
            std::fs::create_dir_all(log_dir)?;
        }
        
        let log_file_path = log_dir.join(format!("audit_{}.log", chrono::Local::now().format("%Y%m%d")));
        let log_file = File::create(log_file_path)?;
        
        let mut log_file_guard = self.log_file.lock().await;
        *log_file_guard = Some(log_file);
        
        Ok(())
    }
}
