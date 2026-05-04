//! 应用启动和配置模块
//!
//! 负责CarpTMS服务器的初始化和启动流程
//!
//! 支持两种架构模式：
//! - MonolithDDD: 单体应用 + DDD 架构
//! - MicroDDD: 微服务 + DDD 架构

pub mod config_loader;
pub mod micro_ddd;
pub mod monitoring;
pub mod monolith_ddd;
pub mod services;

pub use config_loader::load_config;
pub use micro_ddd::MicroDDDBootstrap;
pub use monitoring::init_monitoring;
pub use monolith_ddd::MonolithDDDBootstrap;
pub use services::ApplicationState;

use crate::config::{ArchitectureMode, UnifiedConfig};
use crate::errors::AppResult;
use std::future::Future;
use std::pin::Pin;

/// 架构启动器trait
pub trait ArchitectureBootstrap: Send + Sync {
    /// 获取架构模式
    fn mode(&self) -> ArchitectureMode;

    /// 初始化服务
    fn initialize<'a>(
        &'a self,
        config: &'a UnifiedConfig,
    ) -> Pin<Box<dyn Future<Output = AppResult<ApplicationState>> + Send + 'a>>;

    /// 启动服务
    fn start<'a>(
        &'a self,
        config: UnifiedConfig,
    ) -> Pin<Box<dyn Future<Output = AppResult<()>> + Send + 'a>>;

    /// 关闭服务
    fn shutdown<'a>(&'a self) -> Pin<Box<dyn Future<Output = AppResult<()>> + Send + 'a>>;
}

/// 根据架构模式创建启动器
pub fn create_bootstrap(mode: ArchitectureMode) -> Box<dyn ArchitectureBootstrap> {
    match mode {
        ArchitectureMode::MonolithDDD => Box::new(MonolithDDDBootstrap::new()),
        ArchitectureMode::MicroDDD => Box::new(MicroDDDBootstrap::new()),
    }
}

/// 统一启动入口
pub async fn bootstrap(config: UnifiedConfig) -> AppResult<()> {
    let mode = config.architecture.mode;
    tracing::info!("Starting CarpTMS with architecture mode: {}", mode);
    tracing::info!("Description: {}", mode.description());

    let bootstrap = create_bootstrap(mode);
    bootstrap.start(config).await
}
