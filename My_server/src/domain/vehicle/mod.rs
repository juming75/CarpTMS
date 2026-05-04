//! Vehicle 微服务领域模块
//!
//! 采用 DDD + CQRS 架构，处理车辆管理业务
//!
//! ## 模块结构
//!
//! - `aggregate.rs` - 聚合根，包含业务规则校验
//! - `events.rs` - 领域事件
//! - `commands.rs` - 命令对象（CQRS Write）
//! - `queries.rs` - 查询对象（CQRS Read）
//! - `event_logger.rs` - 事件日志（轻量级审计）

pub mod aggregate;
pub mod commands;
pub mod event_logger;
pub mod events;
pub mod queries;

// 重新导出常用类型
pub use aggregate::VehicleAggregate;
pub use commands::*;
pub use event_logger::{VehicleEventLogEntry, VehicleEventLogger};
pub use events::*;
pub use queries::*;
