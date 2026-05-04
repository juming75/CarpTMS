//! 安全配置模块
//! 提供安全配置检查功能

use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;
use std::path::Path;

/// 安全级别
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub enum SecurityLevel {
    /// 低安全级别
    Low,
    /// 中安全级别
    Medium,
    /// 高安全级别
    High,
    /// 最高安全级别
    Critical,
}

/// 安全配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SecurityConfig {
    /// 安全级别
    security_level: SecurityLevel,
    /// 是否启用HTTPS
    https_enabled: bool,
    /// 是否启用CORS
    cors_enabled: bool,
    /// 是否启用CSRF保护
    csrf_protection_enabled: bool,
    /// 是否启用速率限制
    rate_limiting_enabled: bool,
    /// 是否启用审计日志
    audit_logging_enabled: bool,
    /// 密码复杂度要求
    password_complexity: PasswordComplexity,
    /// 会话超时时间（秒）
    session_timeout: u64,
    /// JWT过期时间（秒）
    jwt_expiration: u64,
}

/// 密码复杂度要求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PasswordComplexity {
    /// 最小长度
    min_length: u32,
    /// 是否要求包含大写字母
    require_uppercase: bool,
    /// 是否要求包含小写字母
    require_lowercase: bool,
    /// 是否要求包含数字
    require_digit: bool,
    /// 是否要求包含特殊字符
    require_special: bool,
}

impl SecurityConfig {
    /// 从文件加载安全配置
    pub async fn load() -> Result<Self, anyhow::Error> {
        let config_path = Path::new(r"config\security.toml");

        if config_path.exists() {
            let mut file = File::open(config_path)?;
            let mut content = String::new();
            file.read_to_string(&mut content)?;

            let config: SecurityConfig = toml::from_str(&content)?;
            Ok(config)
        } else {
            // 使用默认配置
            Ok(Self::default())
        }
    }

    /// 检查安全配置
    pub async fn check(&self) -> Result<Vec<String>, anyhow::Error> {
        let mut issues = Vec::new();

        // 检查HTTPS配置
        if !self.https_enabled {
            issues.push("未启用HTTPS，建议在生产环境中启用".to_string());
        }

        // 检查CORS配置
        if !self.cors_enabled {
            issues.push("未启用CORS，可能会影响前端跨域请求".to_string());
        }

        // 检查CSRF保护
        if !self.csrf_protection_enabled {
            issues.push("未启用CSRF保护，可能会受到跨站请求伪造攻击".to_string());
        }

        // 检查速率限制
        if !self.rate_limiting_enabled {
            issues.push("未启用速率限制，可能会受到DoS攻击".to_string());
        }

        // 检查审计日志
        if !self.audit_logging_enabled {
            issues.push("未启用审计日志，可能无法追踪安全事件".to_string());
        }

        // 检查密码复杂度
        if self.password_complexity.min_length < 8 {
            issues.push("密码最小长度不足，建议至少8位".to_string());
        }

        // 检查会话超时
        if self.session_timeout > 3600 {
            issues.push("会话超时时间过长，建议不超过1小时".to_string());
        }

        // 检查JWT过期时间
        if self.jwt_expiration > 3600 {
            issues.push("JWT过期时间过长，建议不超过1小时".to_string());
        }

        Ok(issues)
    }

    /// 获取安全级别
    pub fn get_level(&self) -> SecurityLevel {
        self.security_level.clone()
    }

    /// 设置安全级别
    pub fn set_level(&mut self, level: SecurityLevel) {
        self.security_level = level;
    }

    /// 启用HTTPS
    pub fn enable_https(&mut self) {
        self.https_enabled = true;
    }

    /// 启用CORS
    pub fn enable_cors(&mut self) {
        self.cors_enabled = true;
    }

    /// 启用CSRF保护
    pub fn enable_csrf_protection(&mut self) {
        self.csrf_protection_enabled = true;
    }

    /// 启用速率限制
    pub fn enable_rate_limiting(&mut self) {
        self.rate_limiting_enabled = true;
    }

    /// 启用审计日志
    pub fn enable_audit_logging(&mut self) {
        self.audit_logging_enabled = true;
    }
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            security_level: SecurityLevel::Medium,
            https_enabled: false,
            cors_enabled: true,
            csrf_protection_enabled: true,
            rate_limiting_enabled: true,
            audit_logging_enabled: true,
            password_complexity: PasswordComplexity {
                min_length: 8,
                require_uppercase: true,
                require_lowercase: true,
                require_digit: true,
                require_special: true,
            },
            session_timeout: 3600,
            jwt_expiration: 3600,
        }
    }
}
