//! / 基础设施层,包含外部依赖实现
//!
//! 基础设施层负责：
//! - 数据库访问（repositories）
//! - 事件总线（event_bus）
//! - 消息队列（messaging）
//! - 外部服务集成
//! - 系统监控（monitoring）

pub mod db;
pub mod event_bus;
pub mod message_router;
pub mod messaging;
pub mod repositories;
pub mod monitoring;
