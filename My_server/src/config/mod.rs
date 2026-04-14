//! /! 配置管理模块

pub mod architecture;
pub mod hot_reload;
pub mod unified;

// 重新导出统一配置管理
pub use unified::manager as config_manager;
pub use unified::UnifiedConfig;

// 重新导出架构配置
pub use architecture::{ArchitectureConfig, ArchitectureMode, BoundedContextConfig, MicroserviceConfig};

// 重新导出热切换模块
pub use hot_reload::{
    ConfigChangeEvent, ConfigChangeListener, ConfigManager, GracefulShutdown, ValidationResult,
};
