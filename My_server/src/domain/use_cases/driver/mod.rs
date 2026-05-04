//! 司机用例模块
//!
//! ## 模块结构
//!
//! - `repository.rs` - 仓库接口定义
//! - `service.rs` - 业务逻辑实现
//! - `tests.rs` - 单元测试
//!
//! ## 扩展计划
//!
//! - `stats.rs` - 统计服务（待添加）

pub mod repository;
pub mod service;

pub use repository::DriverRepository;
pub use service::DriverUseCases;
