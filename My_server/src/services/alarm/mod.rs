//! / 报警服务模块
// 提供报警数据的存储、查询和实时推送功能

pub mod service;
pub mod parser;
pub mod notifier;

pub use service::{AlarmService, AlarmQueryParams, AlarmStatus, AlarmLevel};
pub use parser::{AlarmParser, AlarmType};
pub use notifier::{AlarmNotifier, NotificationChannel};






