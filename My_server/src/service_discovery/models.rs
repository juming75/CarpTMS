//! /! 服务发现模型
//!
//! 定义服务注册和发现相关的数据模型

use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::time::{Duration, Instant};

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

/// 为 Duration 类型实现序列化和反序列化的辅助模块
mod duration_serde {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::time::Duration;

    /// 将 Duration 序列化为毫秒数
    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64(duration.as_millis() as u64)
    }

    /// 从毫秒数反序列化为 Duration
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let millis = u64::deserialize(deserializer)?;
        Ok(Duration::from_millis(millis))
    }
}

/// 服务状态
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ServiceStatus {
    /// 服务健康
    Healthy,
    /// 服务警告
    Warning,
    /// 服务不健康
    Unhealthy,
    /// 服务未知状态
    Unknown,
}

/// 服务健康信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceHealth {
    /// 服务状态
    pub status: ServiceStatus,
    /// 最后检查时间
    #[serde(with = "instant_serde")]
    pub last_check: Instant,
    /// 健康检查详情
    pub details: Option<String>,
    /// 响应时间(毫秒)
    pub response_time: Option<u64>,
}

/// 服务信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    /// 服务ID
    pub service_id: String,
    /// 服务名称
    pub service_name: String,
    /// 服务版本
    pub version: String,
    /// 服务地址
    pub address: SocketAddr,
    /// 服务标签
    pub tags: Vec<String>,
    /// 服务元数据
    pub metadata: serde_json::Value,
    /// 注册时间
    #[serde(with = "instant_serde")]
    pub registered_at: Instant,
    /// 最后更新时间
    #[serde(with = "instant_serde")]
    pub last_updated: Instant,
    /// 健康状态
    pub health: ServiceHealth,
    /// 服务权重
    pub weight: u32,
    /// 服务超时时间
    #[serde(with = "duration_serde")]
    pub timeout: Duration,
}

/// 服务信息创建参数
pub struct ServiceInfoBuilder {
    pub service_id: String,
    pub service_name: String,
    pub version: String,
    pub address: SocketAddr,
    pub tags: Vec<String>,
    pub metadata: serde_json::Value,
    pub weight: u32,
    pub timeout: Duration,
}

impl ServiceInfo {
    /// 创建新的服务信息
    pub fn new(builder: ServiceInfoBuilder) -> Self {
        let now = Instant::now();
        Self {
            service_id: builder.service_id,
            service_name: builder.service_name,
            version: builder.version,
            address: builder.address,
            tags: builder.tags,
            metadata: builder.metadata,
            registered_at: now,
            last_updated: now,
            health: ServiceHealth {
                status: ServiceStatus::Unknown,
                last_check: now,
                details: None,
                response_time: None,
            },
            weight: builder.weight,
            timeout: builder.timeout,
        }
    }

    /// 更新服务健康状态
    pub fn update_health(&mut self, health: ServiceHealth) {
        self.health = health;
        self.last_updated = Instant::now();
    }

    /// 更新服务地址
    pub fn update_address(&mut self, address: SocketAddr) {
        self.address = address;
        self.last_updated = Instant::now();
    }

    /// 检查服务是否健康
    pub fn is_healthy(&self) -> bool {
        self.health.status == ServiceStatus::Healthy
    }

    /// 检查服务是否可用
    pub fn is_available(&self) -> bool {
        matches!(
            self.health.status,
            ServiceStatus::Healthy | ServiceStatus::Warning
        )
    }
}
