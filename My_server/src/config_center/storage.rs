//! /! 配置存储模块
//!
//! 实现配置的持久化存储,支持多种存储后端

use super::models::{ConfigEntry, ConfigKey};
use serde_json;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

/// 配置存储接口
pub trait ConfigStorage {
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
