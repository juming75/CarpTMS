//! / 领域层,包含核心业务逻辑和实体定义

pub mod ddd;
pub mod device_aggregate;
pub mod entities;
pub mod event_logger;
pub mod event_sourced_repositories;
pub mod order_aggregate;
pub mod repositories;
pub mod use_cases;
pub mod user_aggregate;
pub mod vehicle; // 车辆微服务领域模块（DDD + CQRS）
pub mod vehicle_aggregate;
pub mod vehicle_event_sourced_repository;
pub mod vehicle_group; // 车组微服务领域模块（DDD + CQRS）
pub mod weighing; // 称重微服务领域模块（DDD + CQRS） // 统一领域事件日志服务（轻量级审计）

#[cfg(test)]
mod domain_tests;
