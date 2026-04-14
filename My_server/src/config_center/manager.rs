//! /! 配置管理器模块
//!
//! 实现配置的核心管理功能,包括配置的增删改查、版本管理、状态管理等

use super::models::{
    ConfigChangeEvent, ConfigChangeEventType, ConfigEntry, ConfigKey, ConfigStatus, ConfigType,
    ConfigValue,
};
use super::storage::ConfigStorage;
use super::watcher::{ConfigWatcher, ConfigWatcherManager};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

/// 配置创建参数
pub struct ConfigCreateParams<'a> {
    pub namespace: &'a str,
    pub key: &'a str,
    pub value: ConfigValue,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub config_type: ConfigType,
    pub changed_by: Option<String>,
}

/// 配置更新参数
pub struct ConfigUpdateParams {
    pub value: Option<ConfigValue>,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
    pub changed_by: Option<String>,
}

/// 配置管理器
pub struct ConfigManager {
    /// 配置存储
    storage: Arc<dyn ConfigStorage + Send + Sync>,
    /// 配置监听器管理器
    watcher_manager: Arc<ConfigWatcherManager>,
    /// 配置缓存
    cache: Arc<RwLock<HashMap<String, ConfigEntry>>>,
    /// 缓存过期时间
    cache_expiration: Duration,
    /// 配置版本管理
    version_manager: Arc<RwLock<HashMap<String, u64>>>,
}

impl ConfigManager {
    /// 创建新的配置管理器
    pub fn new(
        storage: Arc<dyn ConfigStorage + Send + Sync>,
        watcher_manager: Arc<ConfigWatcherManager>,
        cache_expiration: Duration,
    ) -> Self {
        Self {
            storage,
            watcher_manager,
            cache: Arc::new(RwLock::new(HashMap::new())),
            cache_expiration,
            version_manager: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 创建默认的配置管理器
    pub fn with_default_config() -> Result<Self, String> {
        let storage = Arc::new(super::storage::MemoryConfigStorage::new(None));
        let watcher_manager = Arc::new(ConfigWatcherManager::new(Duration::from_secs(10)));
        Ok(Self::new(
            storage,
            watcher_manager,
            Duration::from_secs(300),
        ))
    }

    /// 启动配置管理器
    pub fn start(&mut self) {
        if let Some(wm) = Arc::get_mut(&mut self.watcher_manager) {
            wm.start();
        }
    }

    /// 停止配置管理器
    pub async fn stop(&mut self) {
        if let Some(wm) = Arc::get_mut(&mut self.watcher_manager) {
            wm.stop().await;
        }
    }

    /// 创建配置
    pub async fn create_config(
        &self,
        params: ConfigCreateParams<'_>,
    ) -> Result<ConfigEntry, String> {
        let config_key = ConfigKey::new(params.namespace, params.key);

        // 检查配置是否已存在
        if self.storage.exists(&config_key)? {
            return Err(format!("Config already exists: {}", config_key.full_key()));
        }

        // 生成版本号
        let version = self.generate_version(params.namespace, params.key)?;

        // 创建配置条目
        let entry = ConfigEntry {
            key: config_key.clone(),
            value: params.value.clone(),
            created_at: Instant::now(),
            updated_at: Instant::now(),
            description: params.description,
            tags: params.tags,
            version: version.to_string(),
            status: ConfigStatus::Active,
            config_type: params.config_type,
        };

        // 存储配置
        self.storage.store(&entry)?;

        // 更新缓存
        self.update_cache(&entry);

        // 发送配置变更事件
        let event = ConfigChangeEvent {
            event_type: ConfigChangeEventType::Created,
            key: config_key,
            old_value: None,
            new_value: Some(params.value),
            timestamp: Instant::now(),
            reason: Some("Config created".to_string()),
            changed_by: params.changed_by,
        };
        self.watcher_manager.broadcast_event(event).await;

        Ok(entry)
    }

    /// 获取配置
    pub async fn get_config(
        &self,
        namespace: &str,
        key: &str,
    ) -> Result<Option<ConfigEntry>, String> {
        let config_key = ConfigKey::new(namespace, key);

        // 先从缓存获取
        if let Some(entry) = self.get_from_cache(&config_key) {
            return Ok(Some(entry));
        }

        // 从存储获取
        let entry = self.storage.get(&config_key)?;

        // 更新缓存
        if let Some(entry) = &entry {
            self.update_cache(entry);
        }

        Ok(entry)
    }

    /// 更新配置
    pub async fn update_config(
        &self,
        namespace: &str,
        key: &str,
        params: ConfigUpdateParams,
    ) -> Result<ConfigEntry, String> {
        let config_key = ConfigKey::new(namespace, key);

        // 获取现有配置
        let old_entry = self
            .storage
            .get(&config_key)?
            .ok_or_else(|| format!("Config not found: {}", config_key.full_key()))?;

        // 生成新版本号
        let version = self.generate_version(namespace, key)?;

        // 创建更新后的配置条目
        let mut entry = old_entry.clone();
        let new_value = params.value;
        if let Some(value) = &new_value {
            entry.value = value.clone();
        }
        if let Some(description) = params.description {
            entry.description = Some(description);
        }
        if let Some(tags) = params.tags {
            entry.tags = tags;
        }
        entry.updated_at = Instant::now();
        entry.version = version.to_string();

        // 存储配置
        self.storage.store(&entry)?;

        // 更新缓存
        self.update_cache(&entry);

        // 发送配置变更事件
        let event = ConfigChangeEvent {
            event_type: ConfigChangeEventType::Updated,
            key: config_key,
            old_value: Some(old_entry.value),
            new_value,
            timestamp: Instant::now(),
            reason: Some("Config updated".to_string()),
            changed_by: params.changed_by,
        };
        self.watcher_manager.broadcast_event(event).await;

        Ok(entry)
    }

    /// 删除配置
    pub async fn delete_config(&self, namespace: &str, key: &str) -> Result<bool, String> {
        let config_key = ConfigKey::new(namespace, key);

        // 获取现有配置
        let old_entry = self.storage.get(&config_key)?;
        if old_entry.is_none() {
            return Ok(false);
        }

        // 删除配置
        let result = self.storage.delete(&config_key)?;

        // 从缓存中删除
        self.remove_from_cache(&config_key);

        // 发送配置变更事件
        if result {
            let event = ConfigChangeEvent {
                event_type: ConfigChangeEventType::Deleted,
                key: config_key,
                old_value: old_entry.map(|e| e.value),
                new_value: None,
                timestamp: Instant::now(),
                reason: Some("Config deleted".to_string()),
                changed_by: None,
            };
            self.watcher_manager.broadcast_event(event).await;
        }

        Ok(result)
    }

    /// 列出所有配置
    pub async fn list_configs(&self) -> Result<Vec<ConfigEntry>, String> {
        let entries = self.storage.list()?;

        // 更新缓存
        for entry in &entries {
            self.update_cache(entry);
        }

        Ok(entries)
    }

    /// 搜索配置
    pub async fn search_configs(
        &self,
        namespace: Option<&str>,
        key_pattern: Option<&str>,
    ) -> Result<Vec<ConfigEntry>, String> {
        let entries = self.storage.search(namespace, key_pattern)?;

        // 更新缓存
        for entry in &entries {
            self.update_cache(entry);
        }

        Ok(entries)
    }

    /// 更改配置状态
    pub async fn change_config_status(
        &self,
        namespace: &str,
        key: &str,
        status: ConfigStatus,
        changed_by: Option<String>,
    ) -> Result<ConfigEntry, String> {
        let config_key = ConfigKey::new(namespace, key);

        // 获取现有配置
        let mut entry = self
            .storage
            .get(&config_key)?
            .ok_or_else(|| format!("Config not found: {}", config_key.full_key()))?;

        // 更新状态
        entry.status = status;
        entry.updated_at = Instant::now();

        // 存储配置
        self.storage.store(&entry)?;

        // 更新缓存
        self.update_cache(&entry);

        // 发送配置变更事件
        let event = ConfigChangeEvent {
            event_type: ConfigChangeEventType::StatusChanged,
            key: config_key,
            old_value: None,
            new_value: None,
            timestamp: Instant::now(),
            reason: Some(format!("Config status changed to {:?}", status)),
            changed_by,
        };
        self.watcher_manager.broadcast_event(event).await;

        Ok(entry)
    }

    /// 生成版本号
    fn generate_version(&self, namespace: &str, key: &str) -> Result<u64, String> {
        let key = format!("{}/{}", namespace, key);
        if let Ok(mut version_manager) = self.version_manager.write() {
            let version = version_manager.entry(key).or_insert(0);
            *version += 1;
            return Ok(*version);
        }
        Err("Failed to acquire version manager lock".to_string())
    }

    /// 从缓存获取配置
    fn get_from_cache(&self, key: &ConfigKey) -> Option<ConfigEntry> {
        self.cache.read().ok()?.get(&key.full_key()).cloned()
    }

    /// 更新缓存
    fn update_cache(&self, entry: &ConfigEntry) {
        if let Ok(mut cache) = self.cache.write() {
            cache.insert(entry.key.full_key(), entry.clone());
        }
    }

    /// 从缓存中删除配置
    fn remove_from_cache(&self, key: &ConfigKey) {
        if let Ok(mut cache) = self.cache.write() {
            cache.remove(&key.full_key());
        }
    }

    /// 清理过期缓存
    pub fn cleanup_cache(&self) {
        if let Ok(mut cache) = self.cache.write() {
            let now = Instant::now();
            cache.retain(|_, entry| now.duration_since(entry.updated_at) <= self.cache_expiration);
        }
    }

    /// 备份配置
    pub async fn backup_configs(&self, backup_path: &str) -> Result<(), String> {
        self.storage.backup(backup_path)
    }

    /// 恢复配置
    pub async fn restore_configs(
        &self,
        backup_path: &str,
        changed_by: Option<String>,
    ) -> Result<(), String> {
        self.storage.restore(backup_path)?;

        // 清除缓存
        if let Ok(mut cache) = self.cache.write() { cache.clear(); }

        // 发送配置变更事件
        let entries = self.storage.list()?;
        for entry in entries {
            let event = ConfigChangeEvent {
                event_type: ConfigChangeEventType::Created,
                key: entry.key,
                old_value: None,
                new_value: Some(entry.value),
                timestamp: Instant::now(),
                reason: Some("Config restored".to_string()),
                changed_by: changed_by.clone(),
            };
            self.watcher_manager.broadcast_event(event).await;
        }

        Ok(())
    }

    /// 获取配置存储
    pub fn storage(&self) -> Arc<dyn ConfigStorage + Send + Sync> {
        self.storage.clone()
    }

    /// 获取配置监听器
    pub fn watcher(&self) -> Arc<ConfigWatcher> {
        self.watcher_manager.watcher()
    }

    /// 获取缓存过期时间
    pub fn cache_expiration(&self) -> Duration {
        self.cache_expiration
    }

    /// 设置缓存过期时间
    pub fn set_cache_expiration(&mut self, expiration: Duration) {
        self.cache_expiration = expiration;
    }

    /// 获取缓存大小
    pub fn cache_size(&self) -> usize {
        self.cache.read().ok().map(|c| c.len()).unwrap_or(0)
    }

    /// 清除缓存
    pub fn clear_cache(&self) {
        if let Ok(mut cache) = self.cache.write() { cache.clear(); }
    }
}
