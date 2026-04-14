//! /! 服务发现模块
//!
//! 提供服务注册、发现和健康检查功能,支持微服务架构的服务管理

mod discovery;
mod health_check;
mod models;
mod registry;

pub use discovery::{LoadBalancingStrategy, ServiceDiscovery};
pub use health_check::HealthChecker;
pub use models::{ServiceHealth, ServiceInfo, ServiceInfoBuilder, ServiceStatus};
pub use registry::ServiceRegistry;
