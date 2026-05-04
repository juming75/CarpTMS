//! 称重微服务领域模块
//!
//! 采用 DDD + CQRS 架构，支持高并发称重数据处理
//!
//! ## 模块结构
//!
//! - `aggregate.rs` - 聚合根，包含业务规则校验
//! - `events.rs` - 领域事件，支持事件溯源
//! - `commands.rs` - 命令对象（CQRS Write）
//! - `queries.rs` - 查询对象（CQRS Read）
//! - `event_logger.rs` - 事件日志（轻量级审计）
//!
//! ## 扩展计划
//!
//! - `event_sourcing.rs` - 事件溯源仓库（后续实现）
//! - `saga.rs` - 分布式事务编排

pub mod aggregate;
pub mod commands;
pub mod event_logger;
pub mod events;
pub mod queries;

// 重新导出常用类型
pub use aggregate::{AggregateError, WeighingAggregate};
pub use commands::*;
pub use event_logger::{EventLogEntry, EventLogger};
pub use events::*;
pub use queries::*;
