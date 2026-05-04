//! / BFF (Backend for Frontend) 模块
// 用于为前端提供统一的数据聚合服务

pub mod datasources;
pub mod export;
pub mod models;
pub mod optimization;
pub mod realtime;
pub mod reports;
pub mod routes;
pub mod services;
pub mod templates;

#[cfg(test)]
pub mod tests;

pub use export::*;
pub use models::*;
pub use optimization::*;
pub use realtime::*;
pub use reports::*;
pub use services::*;
pub use templates::*;
