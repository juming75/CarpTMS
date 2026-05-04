//! 架构配置模块
//!
//! 定义 CarpTMS 支持的两种架构模式：
//! - MonolithDDD: 单体应用 + DDD 架构
//! - MicroDDD: 微服务 + DDD 架构

use serde::{Deserialize, Serialize};
use std::fmt;

/// 架构模式枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum ArchitectureMode {
    /// 单体应用 + DDD 架构
    /// 所有服务在同一个进程中运行，使用领域驱动设计
    #[default]
    MonolithDDD,

    /// 微服务 + DDD 架构
    /// 服务拆分为独立微服务，每个服务采用领域驱动设计
    MicroDDD,
}

impl fmt::Display for ArchitectureMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ArchitectureMode::MonolithDDD => write!(f, "monolith_ddd"),
            ArchitectureMode::MicroDDD => write!(f, "micro_ddd"),
        }
    }
}

impl std::str::FromStr for ArchitectureMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "monolith_ddd" | "monolith-ddd" | "monolithddd" => Ok(Self::MonolithDDD),
            "micro_ddd" | "micro-ddd" | "microddd" => Ok(Self::MicroDDD),
            _ => Err(format!(
                "Unknown architecture mode: {}. Valid options: monolith_ddd, micro_ddd",
                s
            )),
        }
    }
}

impl ArchitectureMode {
    /// 是否为单体架构
    pub fn is_monolith(&self) -> bool {
        matches!(self, ArchitectureMode::MonolithDDD)
    }

    /// 是否为微服务架构
    pub fn is_microservice(&self) -> bool {
        matches!(self, ArchitectureMode::MicroDDD)
    }

    /// 是否使用 DDD 架构
    pub fn is_ddd(&self) -> bool {
        matches!(
            self,
            ArchitectureMode::MonolithDDD | ArchitectureMode::MicroDDD
        )
    }

    /// 获取架构描述
    pub fn description(&self) -> &'static str {
        match self {
            ArchitectureMode::MonolithDDD => "单体应用架构，所有服务在同一进程，采用领域驱动设计",
            ArchitectureMode::MicroDDD => "微服务架构，服务独立部署，每个服务采用领域驱动设计",
        }
    }

    /// 获取推荐的数据库连接池大小
    pub fn recommended_db_connections(&self) -> u32 {
        match self {
            ArchitectureMode::MonolithDDD => 20,
            ArchitectureMode::MicroDDD => 10, // 每个服务独立连接池
        }
    }

    /// 获取推荐的 Redis 连接池大小
    pub fn recommended_redis_connections(&self) -> u32 {
        match self {
            ArchitectureMode::MonolithDDD => 10,
            ArchitectureMode::MicroDDD => 5,
        }
    }
}

/// 架构配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitectureConfig {
    /// 当前架构模式
    #[serde(default)]
    pub mode: ArchitectureMode,

    /// 服务名称（微服务模式下标识当前服务）
    #[serde(default)]
    pub service_name: Option<String>,

    /// 服务端口（微服务模式下可覆盖全局端口）
    #[serde(default)]
    pub service_port: Option<u16>,

    /// 是否启用服务发现（微服务模式）
    #[serde(default)]
    pub enable_service_discovery: bool,

    /// 是否启用分布式追踪（微服务模式推荐）
    #[serde(default = "default_enable_distributed_tracing")]
    pub enable_distributed_tracing: bool,

    /// 是否启用事件驱动通信（DDD 模式推荐）
    #[serde(default)]
    pub enable_event_driven: bool,

    /// 是否启用领域事件持久化
    #[serde(default)]
    pub persist_domain_events: bool,

    /// 限界上下文配置（DDD 模式）
    #[serde(default)]
    pub bounded_contexts: Vec<BoundedContextConfig>,

    /// 微服务配置（微服务模式）
    #[serde(default)]
    pub microservices: Vec<MicroserviceConfig>,
}

fn default_enable_distributed_tracing() -> bool {
    true
}

/// 限界上下文配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundedContextConfig {
    /// 上下文名称
    pub name: String,

    /// 上下文描述
    #[serde(default)]
    pub description: Option<String>,

    /// 聚合根列表
    #[serde(default)]
    pub aggregates: Vec<String>,

    /// 领域服务列表
    #[serde(default)]
    pub domain_services: Vec<String>,

    /// 应用服务列表
    #[serde(default)]
    pub application_services: Vec<String>,
}

/// 微服务配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MicroserviceConfig {
    /// 服务名称
    pub name: String,

    /// 服务地址
    pub host: String,

    /// 服务端口
    pub port: u16,

    /// 健康检查路径
    #[serde(default = "default_health_check_path")]
    pub health_check_path: String,

    /// 服务标签
    #[serde(default)]
    pub tags: Vec<String>,
}

fn default_health_check_path() -> String {
    "/api/health".to_string()
}

impl Default for ArchitectureConfig {
    fn default() -> Self {
        Self {
            mode: ArchitectureMode::default(),
            service_name: None,
            service_port: None,
            enable_service_discovery: false,
            enable_distributed_tracing: default_enable_distributed_tracing(),
            enable_event_driven: false,
            persist_domain_events: false,
            bounded_contexts: vec![],
            microservices: vec![],
        }
    }
}

impl ArchitectureConfig {
    /// 创建新的架构配置
    pub fn new(mode: ArchitectureMode) -> Self {
        Self {
            mode,
            ..Self::default()
        }
    }

    /// 创建单体 DDD 配置
    pub fn monolith_ddd() -> Self {
        Self {
            mode: ArchitectureMode::MonolithDDD,
            enable_event_driven: true,
            persist_domain_events: true,
            bounded_contexts: default_bounded_contexts(),
            ..Self::default()
        }
    }

    /// 创建微服务 DDD 配置
    pub fn micro_ddd() -> Self {
        Self {
            mode: ArchitectureMode::MicroDDD,
            enable_service_discovery: true,
            enable_distributed_tracing: true,
            enable_event_driven: true,
            persist_domain_events: true,
            bounded_contexts: default_bounded_contexts(),
            microservices: default_microservices(),
            ..Self::default()
        }
    }

    /// 获取当前服务名称
    pub fn current_service_name(&self) -> &str {
        self.service_name.as_deref().unwrap_or(match self.mode {
            ArchitectureMode::MonolithDDD => "carptms",
            ArchitectureMode::MicroDDD => "unknown-service",
        })
    }

    /// 获取当前服务端口
    pub fn current_service_port(&self, default_port: u16) -> u16 {
        self.service_port.unwrap_or(default_port)
    }

    /// 验证配置
    pub fn validate(&self) -> Result<(), String> {
        // 微服务模式下必须指定服务名称
        if self.mode.is_microservice() && self.service_name.is_none() {
            return Err("service_name is required in microservice mode".to_string());
        }

        // 验证微服务配置
        for service in &self.microservices {
            if service.name.is_empty() {
                return Err("microservice name cannot be empty".to_string());
            }
            if service.port == 0 {
                return Err(format!("microservice {} port cannot be 0", service.name));
            }
        }

        // 验证限界上下文配置
        for context in &self.bounded_contexts {
            if context.name.is_empty() {
                return Err("bounded context name cannot be empty".to_string());
            }
        }

        Ok(())
    }

    /// 根据模式调整数据库连接池大小
    pub fn adjust_db_config(&self, current_max: u32) -> u32 {
        self.mode.recommended_db_connections().min(current_max)
    }

    /// 根据模式调整 Redis 连接池大小
    pub fn adjust_redis_config(&self, current_max: u32) -> u32 {
        self.mode.recommended_redis_connections().min(current_max)
    }
}

/// 默认限界上下文
fn default_bounded_contexts() -> Vec<BoundedContextConfig> {
    vec![
        BoundedContextConfig {
            name: "transportation".to_string(),
            description: Some("运输管理上下文".to_string()),
            aggregates: vec![
                "Vehicle".to_string(),
                "Driver".to_string(),
                "Trip".to_string(),
            ],
            domain_services: vec!["VehicleService".to_string(), "TripService".to_string()],
            application_services: vec!["TransportationAppService".to_string()],
        },
        BoundedContextConfig {
            name: "cargo".to_string(),
            description: Some("货物管理上下文".to_string()),
            aggregates: vec!["Cargo".to_string(), "Container".to_string()],
            domain_services: vec!["CargoService".to_string()],
            application_services: vec!["CargoAppService".to_string()],
        },
        BoundedContextConfig {
            name: "billing".to_string(),
            description: Some("计费管理上下文".to_string()),
            aggregates: vec!["Invoice".to_string(), "Payment".to_string()],
            domain_services: vec!["BillingService".to_string()],
            application_services: vec!["BillingAppService".to_string()],
        },
    ]
}

/// 默认微服务配置
fn default_microservices() -> Vec<MicroserviceConfig> {
    vec![
        MicroserviceConfig {
            name: "vehicle-service".to_string(),
            host: "0.0.0.0".to_string(),
            port: 8083,
            health_check_path: "/api/health".to_string(),
            tags: vec!["vehicle".to_string(), "transportation".to_string()],
        },
        MicroserviceConfig {
            name: "cargo-service".to_string(),
            host: "0.0.0.0".to_string(),
            port: 8084,
            health_check_path: "/api/health".to_string(),
            tags: vec!["cargo".to_string()],
        },
        MicroserviceConfig {
            name: "trip-service".to_string(),
            host: "0.0.0.0".to_string(),
            port: 8085,
            health_check_path: "/api/health".to_string(),
            tags: vec!["trip".to_string(), "transportation".to_string()],
        },
        MicroserviceConfig {
            name: "billing-service".to_string(),
            host: "0.0.0.0".to_string(),
            port: 8086,
            health_check_path: "/api/health".to_string(),
            tags: vec!["billing".to_string()],
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_architecture_mode_from_str() {
        assert_eq!(
            ArchitectureMode::from_str("monolith_ddd").unwrap(),
            ArchitectureMode::MonolithDDD
        );
        assert_eq!(
            ArchitectureMode::from_str("micro_ddd").unwrap(),
            ArchitectureMode::MicroDDD
        );
    }

    #[test]
    fn test_architecture_mode_helpers() {
        let monolith = ArchitectureMode::MonolithDDD;
        assert!(monolith.is_monolith());
        assert!(!monolith.is_microservice());
        assert!(monolith.is_ddd());

        let micro_ddd = ArchitectureMode::MicroDDD;
        assert!(!micro_ddd.is_monolith());
        assert!(micro_ddd.is_microservice());
        assert!(micro_ddd.is_ddd());
    }

    #[test]
    fn test_architecture_config_defaults() {
        let config = ArchitectureConfig::monolith_ddd();
        assert_eq!(config.mode, ArchitectureMode::MonolithDDD);
        assert!(config.enable_event_driven);
        assert!(config.persist_domain_events);
        assert!(!config.bounded_contexts.is_empty());

        let config = ArchitectureConfig::micro_ddd();
        assert_eq!(config.mode, ArchitectureMode::MicroDDD);
        assert!(config.enable_event_driven);
        assert!(!config.bounded_contexts.is_empty());
        assert!(!config.microservices.is_empty());
    }

    #[test]
    fn test_config_validation() {
        let mut config = ArchitectureConfig::micro_ddd();
        assert!(config.validate().is_err()); // 缺少 service_name

        config.service_name = Some("vehicle-service".to_string());
        assert!(config.validate().is_ok());
    }
}
