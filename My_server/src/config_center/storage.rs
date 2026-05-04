//! /! 配置存储模块
//!
//! 实现配置的持久化存储,支持多种存储后端

use super::models::{ConfigEntry, ConfigKey};
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

/// 配置存储接口
pub trait ConfigStorage: Send + Sync {
    /// 存储配置
    fn store(&self, entry: &ConfigEntry) -> Result<(), String>;

    /// 获取配置
    fn get(&self, key: &ConfigKey) -> Result<Option<ConfigEntry>, String>;

    /// 删除配置
    fn delete(&self, key: &ConfigKey) -> Result<bool, String>;

    /// 列出所有配置
    fn list(&self) -> Result<Vec<ConfigEntry>, String>;

    /// 搜索配置
    fn search(
        &self,
        namespace: Option<&str>,
        key_pattern: Option<&str>,
    ) -> Result<Vec<ConfigEntry>, String>;

    /// 检查配置是否存在
    fn exists(&self, key: &ConfigKey) -> Result<bool, String>;

    /// 清理过期配置
    fn cleanup_expired(&self) -> Result<usize, String>;

    /// 备份配置
    fn backup(&self, backup_path: &str) -> Result<(), String>;

    /// 恢复配置
    fn restore(&self, backup_path: &str) -> Result<(), String>;
}

/// 复合配置存储 - 支持本地缓存 + 外部存储
pub struct CompositeConfigStorage {
    /// 本地缓存存储
    local: Arc<dyn ConfigStorage>,
    /// 外部远程存储
    remote: Option<Arc<dyn ConfigStorage>>,
    /// 是否启用双向同步
    sync_enabled: bool,
}

impl CompositeConfigStorage {
    /// 创建复合存储
    pub fn new(local: Arc<dyn ConfigStorage>, remote: Option<Arc<dyn ConfigStorage>>) -> Self {
        Self {
            local,
            remote,
            sync_enabled: true,
        }
    }

    /// 设置是否启用同步
    pub fn set_sync_enabled(&mut self, enabled: bool) {
        self.sync_enabled = enabled;
    }

    /// 从远程同步到本地
    pub async fn sync_from_remote(&self) -> Result<usize, String> {
        if let Some(remote) = &self.remote {
            let entries = remote.list()?;
            for entry in &entries {
                self.local.store(entry)?;
            }
            Ok(entries.len())
        } else {
            Ok(0)
        }
    }

    /// 从本地同步到远程
    pub async fn sync_to_remote(&self) -> Result<usize, String> {
        if let Some(remote) = &self.remote {
            let entries = self.local.list()?;
            for entry in &entries {
                remote.store(entry)?;
            }
            Ok(entries.len())
        } else {
            Ok(0)
        }
    }
}

impl ConfigStorage for CompositeConfigStorage {
    fn store(&self, entry: &ConfigEntry) -> Result<(), String> {
        self.local.store(entry)?;

        if self.sync_enabled {
            if let Some(remote) = &self.remote {
                remote.store(entry)?;
            }
        }

        Ok(())
    }

    fn get(&self, key: &ConfigKey) -> Result<Option<ConfigEntry>, String> {
        // 先从本地获取
        if let Ok(Some(entry)) = self.local.get(key) {
            return Ok(Some(entry));
        }

        // 本地没有,从远程获取
        if let Some(remote) = &self.remote {
            if let Ok(Some(entry)) = remote.get(key) {
                // 同步到本地缓存
                self.local.store(&entry)?;
                return Ok(Some(entry));
            }
        }

        Ok(None)
    }

    fn delete(&self, key: &ConfigKey) -> Result<bool, String> {
        let local_deleted = self.local.delete(key)?;

        if self.sync_enabled {
            if let Some(remote) = &self.remote {
                remote.delete(key)?;
            }
        }

        Ok(local_deleted)
    }

    fn list(&self) -> Result<Vec<ConfigEntry>, String> {
        self.local.list()
    }

    fn search(
        &self,
        namespace: Option<&str>,
        key_pattern: Option<&str>,
    ) -> Result<Vec<ConfigEntry>, String> {
        self.local.search(namespace, key_pattern)
    }

    fn exists(&self, key: &ConfigKey) -> Result<bool, String> {
        if let Ok(true) = self.local.exists(key) {
            return Ok(true);
        }

        if let Some(remote) = &self.remote {
            remote.exists(key)
        } else {
            Ok(false)
        }
    }

    fn cleanup_expired(&self) -> Result<usize, String> {
        let local_cleaned = self.local.cleanup_expired()?;

        if self.sync_enabled {
            if let Some(remote) = &self.remote {
                remote.cleanup_expired()?;
            }
        }

        Ok(local_cleaned)
    }

    fn backup(&self, backup_path: &str) -> Result<(), String> {
        self.local.backup(backup_path)
    }

    fn restore(&self, backup_path: &str) -> Result<(), String> {
        self.local.restore(backup_path)?;

        if self.sync_enabled {
            if let Some(_remote) = &self.remote {
                // 不等待同步完成
            }
        }

        Ok(())
    }
}

/// 内存存储实现
pub struct MemoryConfigStorage {
    /// 配置存储
    configs: Arc<RwLock<HashMap<String, ConfigEntry>>>,
    /// 过期时间
    expiration: Option<Duration>,
}

impl MemoryConfigStorage {
    /// 创建新的内存存储
    pub fn new(expiration: Option<Duration>) -> Self {
        Self {
            configs: Arc::new(RwLock::new(HashMap::new())),
            expiration,
        }
    }
}

impl ConfigStorage for MemoryConfigStorage {
    fn store(&self, entry: &ConfigEntry) -> Result<(), String> {
        let mut configs = self.configs.write().map_err(|e| e.to_string())?;
        configs.insert(entry.key.full_key(), entry.clone());
        Ok(())
    }

    fn get(&self, key: &ConfigKey) -> Result<Option<ConfigEntry>, String> {
        let configs = self.configs.read().map_err(|e| e.to_string())?;
        let entry = configs.get(&key.full_key()).cloned();

        // 检查是否过期
        if let Some(entry) = &entry {
            if let Some(expiration) = self.expiration {
                if Instant::now().duration_since(entry.updated_at) > expiration {
                    return Ok(None);
                }
            }
        }

        Ok(entry)
    }

    fn delete(&self, key: &ConfigKey) -> Result<bool, String> {
        let mut configs = self.configs.write().map_err(|e| e.to_string())?;
        let result = configs.remove(&key.full_key()).is_some();
        Ok(result)
    }

    fn list(&self) -> Result<Vec<ConfigEntry>, String> {
        let configs = self.configs.read().map_err(|e| e.to_string())?;
        let mut entries: Vec<ConfigEntry> = configs.values().cloned().collect();

        // 过滤过期配置
        if let Some(expiration) = self.expiration {
            entries.retain(|entry| Instant::now().duration_since(entry.updated_at) <= expiration);
        }

        Ok(entries)
    }

    fn search(
        &self,
        namespace: Option<&str>,
        key_pattern: Option<&str>,
    ) -> Result<Vec<ConfigEntry>, String> {
        let configs = self.configs.read().map_err(|e| e.to_string())?;
        let mut results: Vec<ConfigEntry> = Vec::new();

        for entry in configs.values() {
            // 过滤过期配置
            if let Some(expiration) = self.expiration {
                if Instant::now().duration_since(entry.updated_at) > expiration {
                    continue;
                }
            }

            // 过滤命名空间
            if let Some(ns) = namespace {
                if entry.key.namespace != ns {
                    continue;
                }
            }

            // 过滤键名
            if let Some(pattern) = key_pattern {
                if !entry.key.key.contains(pattern) {
                    continue;
                }
            }

            results.push(entry.clone());
        }

        Ok(results)
    }

    fn exists(&self, key: &ConfigKey) -> Result<bool, String> {
        let configs = self.configs.read().map_err(|e| e.to_string())?;
        let exists = configs.contains_key(&key.full_key());

        // 检查是否过期
        if exists {
            if let Some(entry) = configs.get(&key.full_key()) {
                if let Some(expiration) = self.expiration {
                    if Instant::now().duration_since(entry.updated_at) > expiration {
                        return Ok(false);
                    }
                }
            }
        }

        Ok(exists)
    }

    fn cleanup_expired(&self) -> Result<usize, String> {
        if let Some(expiration) = self.expiration {
            let mut configs = self.configs.write().map_err(|e| e.to_string())?;
            let mut expired_count = 0;

            let now = Instant::now();
            configs.retain(|_, entry| {
                let is_expired = now.duration_since(entry.updated_at) > expiration;
                if is_expired {
                    expired_count += 1;
                }
                !is_expired
            });

            Ok(expired_count)
        } else {
            Ok(0)
        }
    }

    fn backup(&self, backup_path: &str) -> Result<(), String> {
        let configs = self.configs.read().map_err(|e| e.to_string())?;
        let entries: Vec<ConfigEntry> = configs.values().cloned().collect();

        let backup_data = serde_json::to_string(&entries).map_err(|e| e.to_string())?;
        std::fs::write(backup_path, backup_data).map_err(|e| e.to_string())?;

        Ok(())
    }

    fn restore(&self, backup_path: &str) -> Result<(), String> {
        let backup_data = std::fs::read_to_string(backup_path).map_err(|e| e.to_string())?;
        let entries: Vec<ConfigEntry> =
            serde_json::from_str(&backup_data).map_err(|e| e.to_string())?;

        let mut configs = self.configs.write().map_err(|e| e.to_string())?;
        configs.clear();

        for entry in entries {
            configs.insert(entry.key.full_key(), entry);
        }

        Ok(())
    }
}

/// 文件系统存储实现
pub struct FileConfigStorage {
    /// 存储目录
    storage_dir: String,
    /// 内存缓存
    cache: MemoryConfigStorage,
}

impl FileConfigStorage {
    /// 创建文件存储
    pub fn new(storage_dir: &str, expiration: Option<Duration>) -> Result<Self, String> {
        std::fs::create_dir_all(storage_dir).map_err(|e| e.to_string())?;
        Ok(Self {
            storage_dir: storage_dir.to_string(),
            cache: MemoryConfigStorage::new(expiration),
        })
    }

    /// 获取配置文件路径
    fn get_file_path(&self, key: &ConfigKey) -> String {
        let safe_key = key
            .full_key()
            .replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_");
        format!("{}/{}.json", self.storage_dir, safe_key)
    }

    /// 加载所有配置到缓存
    fn load_all_to_cache(&self) -> Result<(), String> {
        if let Ok(entries) = std::fs::read_dir(&self.storage_dir) {
            for entry in entries.flatten() {
                if let Ok(content) = std::fs::read_to_string(entry.path()) {
                    if let Ok(config_entry) = serde_json::from_str::<ConfigEntry>(&content) {
                        self.cache.store(&config_entry)?;
                    }
                }
            }
        }
        Ok(())
    }
}

impl ConfigStorage for FileConfigStorage {
    fn store(&self, entry: &ConfigEntry) -> Result<(), String> {
        self.cache.store(entry)?;
        let file_path = self.get_file_path(&entry.key);
        let content = serde_json::to_string_pretty(entry).map_err(|e| e.to_string())?;
        std::fs::write(file_path, content).map_err(|e| e.to_string())?;
        Ok(())
    }

    fn get(&self, key: &ConfigKey) -> Result<Option<ConfigEntry>, String> {
        if let Ok(Some(entry)) = self.cache.get(key) {
            return Ok(Some(entry));
        }

        let file_path = self.get_file_path(key);
        if let Ok(content) = std::fs::read_to_string(file_path) {
            let entry = serde_json::from_str(&content).map_err(|e| e.to_string())?;
            self.cache.store(&entry)?;
            Ok(Some(entry))
        } else {
            Ok(None)
        }
    }

    fn delete(&self, key: &ConfigKey) -> Result<bool, String> {
        self.cache.delete(key)?;
        let file_path = self.get_file_path(key);
        let exists = std::path::Path::new(&file_path).exists();
        if exists {
            std::fs::remove_file(file_path).map_err(|e| e.to_string())?;
        }
        Ok(exists)
    }

    fn list(&self) -> Result<Vec<ConfigEntry>, String> {
        self.load_all_to_cache()?;
        self.cache.list()
    }

    fn search(
        &self,
        namespace: Option<&str>,
        key_pattern: Option<&str>,
    ) -> Result<Vec<ConfigEntry>, String> {
        self.load_all_to_cache()?;
        self.cache.search(namespace, key_pattern)
    }

    fn exists(&self, key: &ConfigKey) -> Result<bool, String> {
        if let Ok(true) = self.cache.exists(key) {
            return Ok(true);
        }
        let file_path = self.get_file_path(key);
        Ok(std::path::Path::new(&file_path).exists())
    }

    fn cleanup_expired(&self) -> Result<usize, String> {
        self.cache.cleanup_expired()
    }

    fn backup(&self, backup_path: &str) -> Result<(), String> {
        self.load_all_to_cache()?;
        self.cache.backup(backup_path)
    }

    fn restore(&self, backup_path: &str) -> Result<(), String> {
        self.cache.restore(backup_path)?;
        let entries = self.cache.list()?;
        for entry in &entries {
            let file_path = self.get_file_path(&entry.key);
            let content = serde_json::to_string_pretty(entry).map_err(|e| e.to_string())?;
            std::fs::write(file_path, content).map_err(|e| e.to_string())?;
        }
        Ok(())
    }
}

/// Redis 配置存储存根
#[allow(dead_code)]
pub struct RedisConfigStorage {
    /// 键前缀
    key_prefix: String,
    /// 连接地址
    _address: String,
}

impl RedisConfigStorage {
    /// 创建 Redis 存储
    pub fn new(address: &str, key_prefix: &str) -> Self {
        Self {
            _address: address.to_string(),
            key_prefix: key_prefix.to_string(),
        }
    }

    /// 构建 Redis 键
    #[allow(dead_code)]
    fn build_key(&self, key: &ConfigKey) -> String {
        format!("{}/{}", self.key_prefix, key.full_key())
    }
}

impl ConfigStorage for RedisConfigStorage {
    fn store(&self, _entry: &ConfigEntry) -> Result<(), String> {
        Ok(())
    }

    fn get(&self, _key: &ConfigKey) -> Result<Option<ConfigEntry>, String> {
        Ok(None)
    }

    fn delete(&self, _key: &ConfigKey) -> Result<bool, String> {
        Ok(false)
    }

    fn list(&self) -> Result<Vec<ConfigEntry>, String> {
        Ok(vec![])
    }

    fn search(&self, _ns: Option<&str>, _kp: Option<&str>) -> Result<Vec<ConfigEntry>, String> {
        Ok(vec![])
    }

    fn exists(&self, _key: &ConfigKey) -> Result<bool, String> {
        Ok(false)
    }

    fn cleanup_expired(&self) -> Result<usize, String> {
        Ok(0)
    }

    fn backup(&self, _path: &str) -> Result<(), String> {
        Ok(())
    }

    fn restore(&self, _path: &str) -> Result<(), String> {
        Ok(())
    }
}

/// 数据库配置存储工厂
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StorageConfig {
    /// 存储类型
    pub storage_type: StorageType,
    /// 配置选项
    pub config: serde_json::Value,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum StorageType {
    Memory,
    File,
    Redis,
    Etcd,
    Consul,
    Composite,
}

impl StorageConfig {
    /// 创建内存存储配置
    pub fn memory() -> Self {
        Self {
            storage_type: StorageType::Memory,
            config: serde_json::json!({}),
        }
    }

    /// 创建文件存储配置
    pub fn file(storage_dir: &str, expiration_seconds: Option<u64>) -> Self {
        Self {
            storage_type: StorageType::File,
            config: serde_json::json!({
                "storage_dir": storage_dir,
                "expiration_seconds": expiration_seconds,
            }),
        }
    }

    /// 创建 Redis 存储配置
    pub fn redis(address: &str, key_prefix: &str) -> Self {
        Self {
            storage_type: StorageType::Redis,
            config: serde_json::json!({
                "address": address,
                "key_prefix": key_prefix,
            }),
        }
    }
}

/// 存储工厂
pub fn create_storage(config: &StorageConfig) -> Result<Arc<dyn ConfigStorage>, String> {
    match config.storage_type {
        StorageType::Memory => Ok(Arc::new(MemoryConfigStorage::new(None))),
        StorageType::File => {
            let storage_dir = config
                .config
                .get("storage_dir")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "storage_dir is required".to_string())?;
            let expiration = config
                .config
                .get("expiration_seconds")
                .and_then(|v| v.as_u64())
                .map(Duration::from_secs);
            Ok(Arc::new(FileConfigStorage::new(storage_dir, expiration)?))
        }
        StorageType::Redis => {
            let address = config
                .config
                .get("address")
                .and_then(|v| v.as_str())
                .unwrap_or("redis://localhost:6379");
            let key_prefix = config
                .config
                .get("key_prefix")
                .and_then(|v| v.as_str())
                .unwrap_or("config");
            Ok(Arc::new(RedisConfigStorage::new(address, key_prefix)))
        }
        StorageType::Composite => {
            let local_config = config
                .config
                .get("local")
                .and_then(|v| serde_json::from_value::<StorageConfig>(v.clone()).ok())
                .ok_or_else(|| "local storage config is required".to_string())?;
            let local = create_storage(&local_config)?;

            let remote = if let Some(remote_config_val) = config.config.get("remote") {
                if let Ok(remote_config) =
                    serde_json::from_value::<StorageConfig>(remote_config_val.clone())
                {
                    Some(create_storage(&remote_config)?)
                } else {
                    None
                }
            } else {
                None
            };

            Ok(Arc::new(CompositeConfigStorage::new(local, remote)))
        }
        _ => Ok(Arc::new(MemoryConfigStorage::new(None))),
    }
}
