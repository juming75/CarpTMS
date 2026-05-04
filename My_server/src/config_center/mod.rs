//! /! 配置中心模块
//!
//! 提供动态配置管理功能,支持配置的存储、读取、更新和监听

mod manager;
mod models;
mod storage;
mod watcher;

pub use manager::{ConfigCreateParams, ConfigManager, ConfigUpdateParams};
pub use models::{
    ConfigChangeEvent, ConfigChangeEventType, ConfigEntry, ConfigKey, ConfigStatus, ConfigType,
    ConfigValue,
};
pub use storage::{
    create_storage, CompositeConfigStorage, ConfigStorage, FileConfigStorage, MemoryConfigStorage,
    RedisConfigStorage, StorageConfig, StorageType,
};
pub use watcher::{ConfigWatcher, ConfigWatcherManager};
