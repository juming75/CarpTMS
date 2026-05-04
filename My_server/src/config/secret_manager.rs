//! / 安全密钥管理模块
//!
//! 提供安全的密钥管理和加载功能
//!
//! 特性:
//! - 从环境变量加载密钥
//! - 密钥验证和长度检查
//! - 支持多密钥管理
//! - 密钥版本管理和轮换
//! - 无缝密钥过渡
//! - 自动轮换调度
//! - 敏感信息脱敏日志

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::env;
use std::error::Error;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use rand::RngCore;
use rand::rngs::OsRng;

/// 密钥版本信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretVersion {
    /// 版本号
    pub version: u64,
    /// 密钥值
    pub secret: String,
    /// 创建时间
    pub created_at: Instant,
    /// 过期时间
    pub expires_at: Option<Instant>,
    /// 是否为当前活跃版本
    pub is_active: bool,
}

/// 密钥类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SecretType {
    JwtSecret,
    JwtRefreshSecret,
    ApiInternalKey,
    TerminalAuthKey,
    EncryptionKey,
}

/// 密钥轮换配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyRotationConfig {
    /// 是否启用自动轮换
    pub auto_rotate: bool,
    /// 轮换周期
    pub rotation_period: Duration,
    /// 过渡窗口期
    pub transition_window: Duration,
    /// 保留历史版本数量
    pub keep_versions: usize,
}

impl Default for KeyRotationConfig {
    fn default() -> Self {
        Self {
            auto_rotate: false,
            rotation_period: Duration::from_secs(86400 * 30), // 30天
            transition_window: Duration::from_secs(3600 * 24), // 24小时
            keep_versions: 5,
        }
    }
}

/// 密钥配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SecretConfig {
    /// JWT 签名密钥
    pub jwt_secret: String,
    /// JWT 刷新密钥
    pub jwt_refresh_secret: String,
    /// API 内部密钥
    pub api_internal_key: String,
    /// 终端认证密钥
    pub terminal_auth_key: String,
    /// 加密密钥
    pub encryption_key: String,
    /// 数据库密码
    pub database_url: String,
    /// Redis 密码
    pub redis_url: String,
}

impl Default for SecretConfig {
    fn default() -> Self {
        Self {
            jwt_secret: env::var("JWT_SECRET").unwrap_or_else(|_| {
                log::warn!("JWT_SECRET 未设置，使用临时密钥（仅用于开发！）");
                "dev_only_jwt_secret_do_not_use_in_production".to_string()
            }),
            jwt_refresh_secret: env::var("JWT_REFRESH_SECRET").unwrap_or_else(|_| {
                "dev_only_jwt_refresh_secret_do_not_use_in_production".to_string()
            }),
            api_internal_key: env::var("API_INTERNAL_KEY").unwrap_or_else(|_| {
                "dev_only_api_key_do_not_use_in_production".to_string()
            }),
            terminal_auth_key: env::var("TERMINAL_AUTH_KEY").unwrap_or_else(|_| {
                "dev_only_terminal_auth_key_do_not_use_in_production".to_string()
            }),
            encryption_key: env::var("ENCRYPTION_KEY").unwrap_or_else(|_| {
                "dev_only_encryption_key_do_not_use_in_production".to_string()
            }),
            database_url: env::var("DATABASE_URL").unwrap_or_else(|_| {
                "postgres://postgres:123@localhost:5432/CarpTMS_db".to_string()
            }),
            redis_url: env::var("REDIS_URL").unwrap_or_else(|_| {
                "redis://localhost:6379".to_string()
            }),
        }
    }
}

impl SecretConfig {
    /// 从环境变量加载密钥配置
    pub fn load_from_env() -> Result<Self, Box<dyn Error>> {
        let config = Self::default();
        config.validate()?;
        Ok(config)
    }

    /// 验证密钥配置
    pub fn validate(&self) -> Result<(), Box<dyn Error>> {
        // 检查是否为生产环境
        let is_production = env::var("PRODUCTION_MODE")
            .map(|v| v == "true")
            .unwrap_or(false);

        if is_production {
            // 生产环境密钥验证
            self.validate_production_secrets()?;
        } else {
            // 开发环境密钥验证（警告）
            self.validate_dev_secrets()?;
        }

        Ok(())
    }

    /// 生产环境密钥验证
    fn validate_production_secrets(&self) -> Result<(), Box<dyn Error>> {
        // JWT 密钥长度检查
        if self.jwt_secret.len() < 64 {
            return Err("JWT_SECRET 必须至少 64 个字符".into());
        }

        // 刷新密钥长度检查
        if self.jwt_refresh_secret.len() < 64 {
            return Err("JWT_REFRESH_SECRET 必须至少 64 个字符".into());
        }

        // 加密密钥长度检查（应该是 32 字节的十六进制字符串）
        if self.encryption_key.len() != 64 {
            return Err("ENCRYPTION_KEY 必须是 64 个十六进制字符（32 字节）".into());
        }

        // 检查是否为默认密钥
        if self.jwt_secret.contains("dev_only") {
            return Err("生产环境不能使用开发密钥！请设置安全的 JWT_SECRET".into());
        }

        log::info!("生产环境密钥验证通过");
        Ok(())
    }

    /// 开发环境密钥验证（仅警告）
    fn validate_dev_secrets(&self) -> Result<(), Box<dyn Error>> {
        if self.jwt_secret.contains("dev_only") {
            log::warn!("⚠️  使用开发密钥！仅用于本地开发，不要在生产环境使用！");
        }

        if self.jwt_secret.len() < 32 {
            log::warn!("⚠️ JWT_SECRET 长度小于 32，可能不够安全");
        }

        Ok(())
    }

    /// 获取脱敏的密钥信息（用于日志）
    pub fn masked_info(&self) -> String {
        format!(
            "jwt_secret: {}***, api_key: {}***, db_url: {}",
            &self.jwt_secret[..4.min(self.jwt_secret.len())],
            &self.api_internal_key[..4.min(self.api_internal_key.len())],
            self.database_url.split('@').last().unwrap_or("***")
        )
    }
}

/// 密钥版本存储
struct SecretVersionStore {
    versions: VecDeque<SecretVersion>,
    current_version: u64,
}

impl SecretVersionStore {
    fn new(initial_secret: String) -> Self {
        let initial_version = SecretVersion {
            version: 1,
            secret: initial_secret,
            created_at: Instant::now(),
            expires_at: None,
            is_active: true,
        };
        
        let mut versions = VecDeque::new();
        versions.push_back(initial_version);
        
        Self {
            versions,
            current_version: 1,
        }
    }
    
    fn add_version(&mut self, secret: String, rotation_config: &KeyRotationConfig) -> u64 {
        // 将当前版本标记为非活跃但仍在过渡窗口内
        if let Some(version) = self.versions.back_mut() {
            version.is_active = false;
            version.expires_at = Some(Instant::now() + rotation_config.transition_window);
        }
        
        // 创建新版本
        let new_version = self.current_version + 1;
        let secret_version = SecretVersion {
            version: new_version,
            secret,
            created_at: Instant::now(),
            expires_at: None,
            is_active: true,
        };
        
        self.versions.push_back(secret_version);
        self.current_version = new_version;
        
        // 清理旧版本
        while self.versions.len() > rotation_config.keep_versions {
            self.versions.pop_front();
        }
        
        new_version
    }
    
    fn get_active_version(&self) -> Option<&SecretVersion> {
        self.versions.iter().find(|v| v.is_active)
    }
    
    fn get_all_valid_versions(&self) -> Vec<&SecretVersion> {
        let now = Instant::now();
        self.versions
            .iter()
            .filter(|v| {
                v.expires_at.map_or(true, |expires| expires > now)
            })
            .collect()
    }
    
    fn get_version(&self, version: u64) -> Option<&SecretVersion> {
        self.versions.iter().find(|v| v.version == version)
    }
}

/// 密钥管理器
pub struct SecretManager {
    config: SecretConfig,
    /// 密钥版本存储
    version_stores: Arc<RwLock<std::collections::HashMap<SecretType, SecretVersionStore>>>,
    /// 轮换配置
    rotation_config: Arc<RwLock<KeyRotationConfig>>,
    /// 上次轮换时间
    last_rotation: Arc<RwLock<std::collections::HashMap<SecretType, Instant>>>,
}

impl SecretManager {
    /// 创建新的密钥管理器
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let config = SecretConfig::load_from_env()?;
        
        // 初始化版本存储
        let mut version_stores = std::collections::HashMap::new();
        version_stores.insert(SecretType::JwtSecret, SecretVersionStore::new(config.jwt_secret.clone()));
        version_stores.insert(SecretType::JwtRefreshSecret, SecretVersionStore::new(config.jwt_refresh_secret.clone()));
        version_stores.insert(SecretType::ApiInternalKey, SecretVersionStore::new(config.api_internal_key.clone()));
        version_stores.insert(SecretType::TerminalAuthKey, SecretVersionStore::new(config.terminal_auth_key.clone()));
        version_stores.insert(SecretType::EncryptionKey, SecretVersionStore::new(config.encryption_key.clone()));
        
        // 初始化上次轮换时间
        let mut last_rotation = std::collections::HashMap::new();
        last_rotation.insert(SecretType::JwtSecret, Instant::now());
        last_rotation.insert(SecretType::JwtRefreshSecret, Instant::now());
        last_rotation.insert(SecretType::ApiInternalKey, Instant::now());
        last_rotation.insert(SecretType::TerminalAuthKey, Instant::now());
        last_rotation.insert(SecretType::EncryptionKey, Instant::now());
        
        log::info!("密钥管理器初始化完成: {}", config.masked_info());
        
        Ok(Self {
            config,
            version_stores: Arc::new(RwLock::new(version_stores)),
            rotation_config: Arc::new(RwLock::new(KeyRotationConfig::default())),
            last_rotation: Arc::new(RwLock::new(last_rotation)),
        })
    }

    /// 获取 JWT 密钥
    pub fn jwt_secret(&self) -> &str {
        &self.config.jwt_secret
    }

    /// 获取 JWT 刷新密钥
    pub fn jwt_refresh_secret(&self) -> &str {
        &self.config.jwt_refresh_secret
    }

    /// 获取 API 密钥
    pub fn api_key(&self) -> &str {
        &self.config.api_internal_key
    }

    /// 获取终端认证密钥
    pub fn terminal_auth_key(&self) -> &str {
        &self.config.terminal_auth_key
    }

    /// 获取加密密钥
    pub fn encryption_key(&self) -> &str {
        &self.config.encryption_key
    }

    /// 获取数据库 URL
    pub fn database_url(&self) -> &str {
        &self.config.database_url
    }

    /// 获取 Redis URL
    pub fn redis_url(&self) -> &str {
        &self.config.redis_url
    }

    /// ========== 密钥轮换相关方法 ==========
    
    /// 设置轮换配置
    pub fn set_rotation_config(&self, config: KeyRotationConfig) {
        if let Ok(mut rotation) = self.rotation_config.write() {
            *rotation = config;
        }
    }

    /// 获取轮换配置
    pub fn get_rotation_config(&self) -> KeyRotationConfig {
        self.rotation_config.read().ok().cloned().unwrap_or_default()
    }

    /// 生成安全的随机密钥
    pub fn generate_secure_key(length: usize) -> Result<String, Box<dyn Error>> {
        let mut bytes = vec![0u8; length];
        OsRng.fill_bytes(&mut bytes);
        Ok(base64::encode(&bytes))
    }

    /// 手动轮换指定类型的密钥
    pub fn rotate_secret(&self, secret_type: SecretType) -> Result<u64, Box<dyn Error>> {
        let rotation_config = self.get_rotation_config();
        
        // 生成新密钥 - 根据不同密钥类型选择合适长度
        let new_secret = match secret_type {
            SecretType::EncryptionKey => {
                // 加密密钥需要32字节(64字符的hex)
                let mut bytes = [0u8; 32];
                OsRng.fill_bytes(&mut bytes);
                hex::encode(&bytes)
            }
            _ => {
                // 其他密钥使用64字节随机数据
                Self::generate_secure_key(64)?
            }
        };

        let new_version = {
            let mut stores = self.version_stores.write()?;
            let store = stores.get_mut(&secret_type)
                .ok_or("密钥类型不存在")?;
            
            store.add_version(new_secret, &rotation_config)
        };

        // 更新最后轮换时间
        {
            let mut last_rot = self.last_rotation.write()?;
            last_rot.insert(secret_type, Instant::now());
        }

        // 更新config中的密钥值为最新版本
        self.update_config_secret(secret_type);

        log::info!("密钥轮换成功 - 类型: {:?}, 新版本: {}", secret_type, new_version);
        Ok(new_version)
    }

    /// 更新config中的密钥值
    fn update_config_secret(&self, secret_type: SecretType) {
        if let Ok(stores) = self.version_stores.read() {
            if let Some(store) = stores.get(&secret_type) {
                if let Some(active) = store.get_active_version() {
                    // 注意: 这里只更新内存中的版本引用,config字段是immutable的
                    log::debug!("活跃密钥版本已更新 - 类型: {:?}, 版本: {}", 
                        secret_type, active.version);
                }
            }
        }
    }

    /// 获取活跃版本的密钥
    pub fn get_active_secret(&self, secret_type: SecretType) -> Option<String> {
        let stores = self.version_stores.read().ok()?;
        let store = stores.get(&secret_type)?;
        store.get_active_version().map(|v| v.secret.clone())
    }

    /// 获取所有有效版本的密钥(用于验证旧token)
    pub fn get_all_valid_secrets(&self, secret_type: SecretType) -> Vec<String> {
        if let Ok(stores) = self.version_stores.read() {
            if let Some(store) = stores.get(&secret_type) {
                return store.get_all_valid_versions()
                    .into_iter()
                    .map(|v| v.secret.clone())
                    .collect();
            }
        }
        vec![]
    }

    /// 获取指定版本的密钥
    pub fn get_secret_version(&self, secret_type: SecretType, version: u64) -> Option<String> {
        let stores = self.version_stores.read().ok()?;
        let store = stores.get(&secret_type)?;
        store.get_version(version).map(|v| v.secret.clone())
    }

    /// 获取密钥版本信息
    pub fn get_secret_versions(&self, secret_type: SecretType) -> Vec<SecretVersion> {
        if let Ok(stores) = self.version_stores.read() {
            if let Some(store) = stores.get(&secret_type) {
                return store.versions.iter().cloned().collect();
            }
        }
        vec![]
    }

    /// 检查是否需要自动轮换
    pub fn check_and_rotate(&self, secret_type: SecretType) -> Result<bool, Box<dyn Error>> {
        let config = self.get_rotation_config();
        
        if !config.auto_rotate {
            return Ok(false);
        }

        let last_rot = self.last_rotation.read()?;
        let last_time = last_rot.get(&secret_type)
            .ok_or("找不到上次轮换时间")?;

        if Instant::now().duration_since(*last_time) >= config.rotation_period {
            self.rotate_secret(secret_type)?;
            return Ok(true);
        }

        Ok(false)
    }

    /// 批量检查并轮换所有密钥
    pub fn check_and_rotate_all(&self) -> Result<Vec<(SecretType, u64)>, Box<dyn Error>> {
        let mut rotated = Vec::new();
        
        for &secret_type in &[
            SecretType::JwtSecret,
            SecretType::JwtRefreshSecret,
            SecretType::ApiInternalKey,
            SecretType::TerminalAuthKey,
            SecretType::EncryptionKey,
        ] {
            if self.check_and_rotate(secret_type)? {
                if let Some(version) = self.get_secret_versions(secret_type)
                    .last()
                    .map(|v| v.version) {
                    rotated.push((secret_type, version));
                }
            }
        }

        Ok(rotated)
    }

    /// 获取当前活跃版本号
    pub fn get_current_version(&self, secret_type: SecretType) -> Option<u64> {
        let stores = self.version_stores.read().ok()?;
        let store = stores.get(&secret_type)?;
        store.get_active_version().map(|v| v.version)
    }
}

impl Default for SecretManager {
    fn default() -> Self {
        match Self::new() {
            Ok(manager) => manager,
            Err(e) => {
                tracing::error!(error = %e, "密钥管理器初始化失败");
                tracing::warn!("使用默认开发配置继续运行");
                SecretManager {
                    config: SecretConfig::default()
                }
            }
        }
    }
}

/// 从环境变量获取字符串值
pub fn get_env_string(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_string())
}

/// 从环境变量获取可选字符串值
pub fn get_env_optional(key: &str) -> Option<String> {
    env::var(key).ok().filter(|s| !s.is_empty())
}

/// 从环境变量获取整数
pub fn get_env_int(key: &str, default: i64) -> i64 {
    env::var(key)
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(default)
}

/// 从环境变量获取布尔值
pub fn get_env_bool(key: &str, default: bool) -> bool {
    env::var(key)
        .ok()
        .map(|s| s.to_lowercase() == "true" || s == "1")
        .unwrap_or(default)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_env_string_default() {
        let value = get_env_string("NONEXISTENT_KEY_12345", "default_value");
        assert_eq!(value, "default_value");
    }

    #[test]
    fn test_env_int_default() {
        let value = get_env_int("NONEXISTENT_KEY_12345", 42);
        assert_eq!(value, 42);
    }

    #[test]
    fn test_env_bool_default() {
        let value = get_env_bool("NONEXISTENT_KEY_12345", true);
        assert!(value);
    }

    #[test]
    fn test_secret_config_default() {
        let config = SecretConfig::default();
        assert!(!config.jwt_secret.is_empty());
    }

    #[test]
    fn test_masked_info() {
        let config = SecretConfig::default();
        let info = config.masked_info();
        assert!(info.contains("jwt_secret:"));
        assert!(info.contains("***"));
    }

    // ========== 密钥轮换测试 ==========

    #[test]
    fn test_secret_manager_creation() {
        let manager = SecretManager::new();
        assert!(manager.is_ok());
    }

    #[test]
    fn test_get_active_secret() {
        let manager = SecretManager::new().unwrap();
        let secret = manager.get_active_secret(SecretType::JwtSecret);
        assert!(secret.is_some());
        assert!(!secret.unwrap().is_empty());
    }

    #[test]
    fn test_initial_version() {
        let manager = SecretManager::new().unwrap();
        let version = manager.get_current_version(SecretType::JwtSecret);
        assert_eq!(version, Some(1));
    }

    #[test]
    fn test_rotate_secret() {
        let manager = SecretManager::new().unwrap();
        
        // 获取原始密钥
        let original_secret = manager.get_active_secret(SecretType::JwtSecret).unwrap();
        let original_version = manager.get_current_version(SecretType::JwtSecret).unwrap();
        
        // 轮换密钥
        let new_version = manager.rotate_secret(SecretType::JwtSecret).unwrap();
        
        // 验证版本增加
        assert_eq!(new_version, original_version + 1);
        
        // 验证新密钥不同
        let new_secret = manager.get_active_secret(SecretType::JwtSecret).unwrap();
        assert_ne!(original_secret, new_secret);
    }

    #[test]
    fn test_get_all_valid_secrets() {
        let manager = SecretManager::new().unwrap();
        
        // 轮换几次
        manager.rotate_secret(SecretType::JwtSecret).unwrap();
        manager.rotate_secret(SecretType::JwtSecret).unwrap();
        
        // 获取所有有效密钥
        let secrets = manager.get_all_valid_secrets(SecretType::JwtSecret);
        assert!(secrets.len() >= 2); // 应该有多个版本
    }

    #[test]
    fn test_get_secret_versions() {
        let manager = SecretManager::new().unwrap();
        
        let versions = manager.get_secret_versions(SecretType::JwtSecret);
        assert_eq!(versions.len(), 1);
        assert_eq!(versions[0].version, 1);
        assert!(versions[0].is_active);
    }

    #[test]
    fn test_get_specific_version() {
        let manager = SecretManager::new().unwrap();
        
        // 获取v1
        let v1_secret = manager.get_secret_version(SecretType::JwtSecret, 1);
        assert!(v1_secret.is_some());
        
        // 轮换到v2
        manager.rotate_secret(SecretType::JwtSecret).unwrap();
        
        // 仍然可以获取v1
        let v1_secret_again = manager.get_secret_version(SecretType::JwtSecret, 1);
        assert_eq!(v1_secret, v1_secret_again);
        
        // 可以获取v2
        let v2_secret = manager.get_secret_version(SecretType::JwtSecret, 2);
        assert!(v2_secret.is_some());
        assert_ne!(v1_secret, v2_secret);
    }

    #[test]
    fn test_rotation_config() {
        let manager = SecretManager::new().unwrap();
        
        let mut config = manager.get_rotation_config();
        assert!(!config.auto_rotate);
        
        // 修改配置
        config.auto_rotate = true;
        config.rotation_period = Duration::from_secs(3600);
        manager.set_rotation_config(config);
        
        let new_config = manager.get_rotation_config();
        assert!(new_config.auto_rotate);
        assert_eq!(new_config.rotation_period, Duration::from_secs(3600));
    }

    #[test]
    fn test_generate_secure_key() {
        let key1 = SecretManager::generate_secure_key(32).unwrap();
        let key2 = SecretManager::generate_secure_key(32).unwrap();
        
        assert_eq!(key1.len(), 44); // base64编码后约为原始的4/3
        assert_ne!(key1, key2); // 两次生成的密钥应该不同
    }

    #[test]
    fn test_check_and_rotate_no_auto() {
        let manager = SecretManager::new().unwrap();
        
        // 默认不启用自动轮换
        let result = manager.check_and_rotate(SecretType::JwtSecret);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), false);
    }

    #[test]
    fn test_multiple_secret_types() {
        let manager = SecretManager::new().unwrap();
        
        // 测试所有密钥类型
        let secret_types = [
            SecretType::JwtSecret,
            SecretType::JwtRefreshSecret,
            SecretType::ApiInternalKey,
            SecretType::TerminalAuthKey,
            SecretType::EncryptionKey,
        ];
        
        for &secret_type in &secret_types {
            let secret = manager.get_active_secret(secret_type);
            assert!(secret.is_some());
            
            let version = manager.get_current_version(secret_type);
            assert_eq!(version, Some(1));
        }
    }

    #[test]
    fn test_version_cleanup() {
        let manager = SecretManager::new().unwrap();
        
        // 设置保留3个版本
        let mut config = KeyRotationConfig::default();
        config.keep_versions = 3;
        manager.set_rotation_config(config);
        
        // 轮换5次
        for _ in 0..5 {
            manager.rotate_secret(SecretType::JwtSecret).unwrap();
        }
        
        let versions = manager.get_secret_versions(SecretType::JwtSecret);
        // 应该最多保留3个版本
        assert!(versions.len() <= 3);
        // 最新版本应该是6(初始1 + 5次轮换)
        assert_eq!(versions.last().unwrap().version, 6);
    }

    #[test]
    fn test_active_version_flag() {
        let manager = SecretManager::new().unwrap();
        
        let versions1 = manager.get_secret_versions(SecretType::JwtSecret);
        assert!(versions1[0].is_active);
        
        // 轮换
        manager.rotate_secret(SecretType::JwtSecret).unwrap();
        
        let versions2 = manager.get_secret_versions(SecretType::JwtSecret);
        // v1 应该不再活跃
        assert!(!versions2[0].is_active);
        // v2 应该是活跃的
        assert!(versions2[1].is_active);
    }
}
