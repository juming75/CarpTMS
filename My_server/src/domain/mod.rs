//! / 领域层,包含核心业务逻辑和实体定义

pub mod ddd;
pub mod device_aggregate;
pub mod entities;
pub mod event_sourced_repositories;
pub mod order_aggregate;
pub mod repositories;
pub mod use_cases;
pub mod user_aggregate;
pub mod vehicle_aggregate;
pub mod vehicle_event_sourced_repository;

#[cfg(test)]
mod domain_tests;
