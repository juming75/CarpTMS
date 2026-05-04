//! HTTPS/WSS配置模块
//!
//! 实现生产环境的安全传输配置
//! 包括HTTPS和WSS（WebSocket Secure）的支持

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// TLS/SSL配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    /// 是否启用TLS
    pub enabled: bool,
    /// 证书文件路径
    pub cert_path: Option<PathBuf>,
    /// 私钥文件路径
    pub key_path: Option<PathBuf>,
    /// CA证书路径（用于双向认证）
    pub ca_cert_path: Option<PathBuf>,
    /// 是否启用客户端证书验证
    pub client_auth_enabled: bool,
    /// 支持的TLS版本
    pub min_tls_version: TlsVersion,
}

/// TLS版本
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TlsVersion {
    /// TLS 1.2
    Tls12,
    /// TLS 1.3
    Tls13,
}

impl Default for TlsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            cert_path: None,
            key_path: None,
            ca_cert_path: None,
            client_auth_enabled: false,
            min_tls_version: TlsVersion::Tls12,
        }
    }
}

/// HTTPS服务器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpsServerConfig {
    /// HTTP端口
    pub http_port: u16,
    /// HTTPS端口
    pub https_port: u16,
    /// 是否启用自动HTTP到HTTPS重定向
    pub redirect_http_to_https: bool,
    /// TLS配置
    pub tls: TlsConfig,
    /// HSTS（HTTP Strict Transport Security）配置
    pub hsts_enabled: bool,
    /// HSTS max-age（秒）
    pub hsts_max_age: u64,
}

impl Default for HttpsServerConfig {
    fn default() -> Self {
        Self {
            http_port: 8080,
            https_port: 8443,
            redirect_http_to_https: true,
            tls: TlsConfig::default(),
            hsts_enabled: true,
            hsts_max_age: 31536000, // 1年
        }
    }
}

/// WebSocket安全配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WssConfig {
    /// WebSocket端口
    pub ws_port: u16,
    /// WebSocket Secure端口
    pub wss_port: u16,
    /// 是否启用自动WS到WSS重定向
    pub redirect_ws_to_wss: bool,
    /// TLS配置
    pub tls: TlsConfig,
    /// 心跳间隔（秒）
    pub heartbeat_interval: u64,
    /// 连接超时时间（秒）
    pub connection_timeout: u64,
    /// 最大连接数
    pub max_connections: usize,
}

impl Default for WssConfig {
    fn default() -> Self {
        Self {
            ws_port: 8083,
            wss_port: 8444,
            redirect_ws_to_wss: true,
            tls: TlsConfig::default(),
            heartbeat_interval: 30,
            connection_timeout: 60,
            max_connections: 10000,
        }
    }
}

/// 安全传输配置管理器
/// 统一管理HTTPS和WSS的配置
pub struct SecureTransportConfig {
    /// HTTPS配置
    pub https: HttpsServerConfig,
    /// WSS配置
    pub wss: WssConfig,
}

impl SecureTransportConfig {
    /// 创建新的安全传输配置
    pub fn new() -> Self {
        Self {
            https: HttpsServerConfig::default(),
            wss: WssConfig::default(),
        }
    }

    /// 创建生产环境默认配置
    pub fn production_defaults() -> Self {
        Self {
            https: HttpsServerConfig {
                http_port: 80,
                https_port: 443,
                redirect_http_to_https: true,
                tls: TlsConfig {
                    enabled: true,
                    min_tls_version: TlsVersion::Tls12,
                    ..TlsConfig::default()
                },
                hsts_enabled: true,
                hsts_max_age: 31536000,
            },
            wss: WssConfig {
                ws_port: 80,
                wss_port: 443,
                redirect_ws_to_wss: true,
                tls: TlsConfig {
                    enabled: true,
                    min_tls_version: TlsVersion::Tls12,
                    ..TlsConfig::default()
                },
                heartbeat_interval: 30,
                connection_timeout: 60,
                max_connections: 10000,
            },
        }
    }

    /// 检查TLS配置是否有效
    pub fn validate_tls_config(&self) -> Result<(), String> {
        if self.https.tls.enabled {
            if self.https.tls.cert_path.is_none() {
                return Err("HTTPS enabled but cert_path not specified".to_string());
            }
            if self.https.tls.key_path.is_none() {
                return Err("HTTPS enabled but key_path not specified".to_string());
            }
        }

        if self.wss.tls.enabled {
            if self.wss.tls.cert_path.is_none() {
                return Err("WSS enabled but cert_path not specified".to_string());
            }
            if self.wss.tls.key_path.is_none() {
                return Err("WSS enabled but key_path not specified".to_string());
            }
        }

        Ok(())
    }

    /// 获取安全端口配置摘要
    pub fn get_config_summary(&self) -> String {
        format!(
            "HTTPS: {}:{}, WSS: {}:{}, TLS: {}, HSTS: {}",
            self.https.https_port,
            if self.https.redirect_http_to_https { "redirect" } else { "standalone" },
            self.wss.wss_port,
            if self.wss.redirect_ws_to_wss { "redirect" } else { "standalone" },
            if self.https.tls.enabled { "enabled" } else { "disabled" },
            if self.https.hsts_enabled { "enabled" } else { "disabled" }
        )
    }
}

impl Default for SecureTransportConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// 创建安全传输配置（便捷函数）
pub fn create_secure_transport_config() -> SecureTransportConfig {
    SecureTransportConfig::new()
}

/// 创建生产环境安全传输配置
pub fn create_production_secure_config() -> SecureTransportConfig {
    SecureTransportConfig::production_defaults()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_production_defaults() {
        let config = SecureTransportConfig::production_defaults();
        assert!(config.https.tls.enabled);
        assert!(config.wss.tls.enabled);
        assert_eq!(config.https.https_port, 443);
        assert_eq!(config.wss.wss_port, 443);
    }

    #[test]
    fn test_validate_tls_config() {
        let config = SecureTransportConfig::new();
        assert!(config.validate_tls_config().is_ok());

        let mut config_with_https = SecureTransportConfig::new();
        config_with_https.https.tls.enabled = true;
        assert!(config_with_https.validate_tls_config().is_err());
    }
}
