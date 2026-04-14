//! /! 配置中心模型
//!
//! 定义配置中心相关的数据模型

use serde::{Deserialize, Serialize};
use std::time::Instant;

/// 为 Instant 类型实现序列化和反序列化的辅助模块
mod instant_serde {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::time::{Duration, Instant};

    /// 将 Instant 序列化为秒数(从当前时间开始计算)
    pub fn serialize<S>(instant: &Instant, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let duration = instant.elapsed();
        serializer.serialize_i64(duration.as_secs() as i64)
    }

    /// 从秒数反序列化为 Instant(基于当前时间)
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Instant, D::Error>
    where
        D: Deserializer<'de>,
    {
        let seconds = i64::deserialize(deserializer)?;
        let duration = Duration::from_secs(seconds as u64);
        Ok(Instant::now() - duration)
    }
}

/// 配置键
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ConfigKey {
    /// 配置命名空间
    pub namespace: String,
    /// 配置键名
    pub key: String,
    /// 配置版本
    pub version: Option<String>,
}

impl ConfigKey {
    /// 创建新的配置键
    pub fn new(namespace: &str, key: &str) -> Self {
        Self {
            namespace: namespace.to_string(),
            key: key.to_string(),
            version: None,
        }
    }

    /// 创建带版本的配置键
    pub fn new_with_version(namespace: &str, key: &str, version: &str) -> Self {
        Self {
            namespace: namespace.to_string(),
            key: key.to_string(),
            version: Some(version.to_string()),
        }
    }

    /// 获取完整键名
    pub fn full_key(&self) -> String {
        if let Some(version) = &self.version {
            format!("{}/{}/{}", self.namespace, self.key, version)
        } else {
            format!("{}/{}", self.namespace, self.key)
        }
    }
}

/// 配置值
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfigValue {
    /// 字符串值
    String(String),
    /// 整数值
    Integer(i64),
    /// 布尔值
    Boolean(bool),
    /// 浮点数
    Float(f64),
    /// 字符串列表
    StringList(Vec<String>),
    /// 整数值列表
    IntegerList(Vec<i64>),
    /// 布尔值列表
    BooleanList(Vec<bool>),
    /// 浮点数列表
    FloatList(Vec<f64>),
    /// JSON对象
    Json(serde_json::Value),
    /// 二进制数据
    Binary(Vec<u8>),
}

impl ConfigValue {
    /// 从字符串创建配置值
    pub fn from_string(value: &str) -> Self {
        ConfigValue::String(value.to_string())
    }

    /// 从整数创建配置值
    pub fn from_integer(value: i64) -> Self {
        ConfigValue::Integer(value)
    }

    /// 从布尔值创建配置值
    pub fn from_boolean(value: bool) -> Self {
        ConfigValue::Boolean(value)
    }

    /// 从浮点数创建配置值
    pub fn from_float(value: f64) -> Self {
        ConfigValue::Float(value)
    }

    /// 从JSON创建配置值
    pub fn from_json(value: serde_json::Value) -> Self {
        ConfigValue::Json(value)
    }

    /// 尝试获取字符串值
    pub fn as_string(&self) -> Option<&str> {
        match self {
            ConfigValue::String(s) => Some(s),
            _ => None,
        }
    }

    /// 尝试获取整数值
    pub fn as_integer(&self) -> Option<i64> {
        match self {
            ConfigValue::Integer(i) => Some(*i),
            _ => None,
        }
    }

    /// 尝试获取布尔值
    pub fn as_boolean(&self) -> Option<bool> {
        match self {
            ConfigValue::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    /// 尝试获取浮点数值
    pub fn as_float(&self) -> Option<f64> {
        match self {
            ConfigValue::Float(f) => Some(*f),
            _ => None,
        }
    }

    /// 尝试获取JSON值
    pub fn as_json(&self) -> Option<&serde_json::Value> {
        match self {
            ConfigValue::Json(j) => Some(j),
            _ => None,
        }
    }
}

/// 配置条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigEntry {
    /// 配置键
    pub key: ConfigKey,
    /// 配置值
    pub value: ConfigValue,
    /// 创建时间
    #[serde(with = "instant_serde")]
    pub created_at: Instant,
    /// 最后更新时间
    #[serde(with = "instant_serde")]
    pub updated_at: Instant,
    /// 配置描述
    pub description: Option<String>,
    /// 配置标签
    pub tags: Vec<String>,
    /// 配置版本
    pub version: String,
    /// 配置状态
    pub status: ConfigStatus,
    /// 配置类型
    pub config_type: ConfigType,
}

/// 配置状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConfigStatus {
    /// 活跃
    Active,
    /// 已过期
    Expired,
    /// 已禁用
    Disabled,
    /// 草稿
    Draft,
}

/// 配置类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConfigType {
    /// 系统配置
    System,
    /// 应用配置
    Application,
    /// 服务配置
    Service,
    /// 环境配置
    Environment,
    /// 自定义配置
    Custom,
}

/// 配置变更事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigChangeEvent {
    /// 事件类型
    pub event_type: ConfigChangeEventType,
    /// 配置键
    pub key: ConfigKey,
    /// 旧值
    pub old_value: Option<ConfigValue>,
    /// 新值
    pub new_value: Option<ConfigValue>,
    /// 变更时间
    #[serde(with = "instant_serde")]
    pub timestamp: Instant,
    /// 变更原因
    pub reason: Option<String>,
    /// 变更人
    pub changed_by: Option<String>,
}

/// 配置变更事件类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConfigChangeEventType {
    /// 配置创建
    Created,
    /// 配置更新
    Updated,
    /// 配置删除
    Deleted,
    /// 配置状态变更
    StatusChanged,
    /// 配置版本变更
    VersionChanged,
}
