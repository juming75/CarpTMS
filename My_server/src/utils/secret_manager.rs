//! / 密钥管理模块
// 提供安全的密钥存储和访问

use log::warn;
use std::env;
use thiserror::Error;
use tokio::fs;

#[derive(Debug, Error)]
pub enum SecretError {
    #[error("Environment variable not found: {0}")]
    EnvVarNotFound(String),

    #[error("Secret file not found: {0}")]
    SecretFileNotFound(String),

    #[error("Invalid secret format: {0}")]
    InvalidFormat(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Production security violation: {0}")]
    ProductionSecurityError(String),
}

/// 检查是否处于生产模式
fn is_production() -> bool {
    env::var("PRODUCTION_MODE")
        .map(|v| v.eq_ignore_ascii_case("true") || v == "1")
        .unwrap_or(false)
}

/// 密钥管理器
pub struct SecretManager;

impl SecretManager {
    /// 从环境变量获取 JWT 密钥
    pub async fn get_jwt_secret() -> Result<String, SecretError> {
        let secret = match Self::get_env_var("JWT_SECRET") {
            Ok(secret) => secret,
            Err(_) => match Self::get_secret_from_file("JWT_SECRET_FILE").await {
                Ok(secret) => secret,
                Err(_) => {
                    // 开发环境默认密钥(仅用于开发！)
                    warn!("Using default JWT secret. Change this in production!");
                    "dev_jwt_secret_change_in_production_123456".to_string()
                }
            },
        };

        // 生产环境安全检查
        if is_production() {
            if secret.contains("dev") || secret.contains("change") || secret.contains("default") {
                log::error!("FATAL: 生产环境使用了不安全的 JWT_SECRET！请立即设置安全的密钥。");
                return Err(SecretError::ProductionSecurityError(
                    "生产环境使用了不安全的 JWT_SECRET，请设置符合安全要求的密钥".to_string(),
                ));
            }
            if secret.len() < 64 {
                log::error!("FATAL: 生产环境 JWT_SECRET 长度不足 64 字符！");
                return Err(SecretError::ProductionSecurityError(
                    "JWT_SECRET 长度不足，请使用至少 64 字符的安全密钥".to_string(),
                ));
            }
            log::info!("JWT_SECRET 通过生产环境安全检查");
        } else if secret.contains("dev") || secret.contains("change") {
            warn!("⚠️  使用默认 JWT_SECRET，仅限开发环境使用！");
        }

        Ok(secret)
    }

    /// 从环境变量获取加密密钥
    pub async fn get_encryption_key() -> Result<[u8; 32], SecretError> {
        let key_str = match Self::get_env_var("ENCRYPTION_KEY") {
            Ok(key) => key,
            Err(_) => Self::get_secret_from_file("ENCRYPTION_KEY_FILE").await?,
        };

        // 从十六进制字符串解码
        let key_bytes = hex::decode(&key_str).map_err(|_| {
            SecretError::InvalidFormat("Encryption key must be hex string".to_string())
        })?;

        if key_bytes.len() != 32 {
            return Err(SecretError::InvalidFormat(
                "Encryption key must be exactly 32 bytes (64 hex chars)".to_string(),
            ));
        }

        let mut key = [0u8; 32];
        key.copy_from_slice(&key_bytes);
        Ok(key)
    }

    /// 从环境变量获取数据库密码
    pub async fn get_database_password() -> Result<String, SecretError> {
        match Self::get_env_var("DATABASE_PASSWORD") {
            Ok(password) => Ok(password),
            Err(_) => match Self::get_secret_from_file("DATABASE_PASSWORD_FILE").await {
                Ok(password) => Ok(password),
                Err(_) => Ok("tms_default_password".to_string()),
            },
        }
    }

    /// 从环境变量获取 Redis 密钥
    pub async fn get_redis_password() -> Result<String, SecretError> {
        match Self::get_env_var("REDIS_PASSWORD") {
            Ok(password) => Ok(password),
            Err(_) => match Self::get_secret_from_file("REDIS_PASSWORD_FILE").await {
                Ok(password) => Ok(password),
                Err(_) => Ok("".to_string()),
            },
        }
    }

    /// 生成安全的随机密钥(用于初始配置)
    pub fn generate_secret(length: usize) -> String {
        use rand::RngCore;
        let mut bytes = vec![0u8; length];
        rand::thread_rng().fill_bytes(&mut bytes);
        hex::encode(&bytes)
    }

    /// 从环境变量获取值
    fn get_env_var(var_name: &str) -> Result<String, SecretError> {
        env::var(var_name).map_err(|_| SecretError::EnvVarNotFound(var_name.to_string()))
    }

    /// 从文件获取密钥
    async fn get_secret_from_file(env_var: &str) -> Result<String, SecretError> {
        let file_path = Self::get_env_var(env_var)?;

        fs::read_to_string(&file_path)
            .await
            .map(|s| s.trim().to_string())
            .map_err(|_| SecretError::SecretFileNotFound(file_path))
    }

    /// 检查是否使用了默认/不安全的密钥
    pub async fn check_insecure_secrets() -> Vec<SecretWarning> {
        let warnings = Vec::new();

        // 检查 JWT 密钥
        if let Ok(secret) = Self::get_jwt_secret().await {
            if secret.contains("default") || secret.contains("change") || secret.len() < 32 {
                warn!("JWT_SECRET may be insecure");
            }
        }

        // 检查加密密钥
        if Self::get_encryption_key().await.is_err() {
            warn!("ENCRYPTION_KEY not properly configured");
        }

        // 检查数据库密码
        if let Ok(password) = Self::get_database_password().await {
            if password == "tms_default_password" {
                warn!("Using default database password, this is insecure");
            }
        }

        warnings
    }

    /// 检查是否使用了默认/不安全的密钥(阻塞版本)
    pub fn check_insecure_secrets_blocking() -> Vec<SecretWarning> {
        tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap()
            .block_on(Self::check_insecure_secrets())
    }
}

/// 密钥警告
#[derive(Debug, Clone)]
pub struct SecretWarning {
    pub secret_type: String,
    pub message: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_secret() {
        let secret = SecretManager::generate_secret(32);
        assert_eq!(secret.len(), 64); // 32 bytes = 64 hex chars
    }

    #[tokio::test]
    async fn test_secret_file_not_found() {
        env::set_var("TEST_SECRET_FILE", "/nonexistent/file.txt");
        let result = SecretManager::get_secret_from_file("TEST_SECRET_FILE").await;
        assert!(result.is_err());
    }
}
