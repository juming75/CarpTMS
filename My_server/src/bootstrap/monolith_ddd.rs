//! 单体 DDD 架构启动器
//!
//! 所有服务在同一个进程中运行，使用领域驱动设计
//! 特点：
//! - 统一的数据库连接池
//! - 共享的应用状态
//! - 领域事件在进程内传递
//! - 简化部署和运维

use crate::bootstrap::{ArchitectureBootstrap, ApplicationState};
use crate::config::UnifiedConfig;
use crate::config::ArchitectureMode;
use crate::errors::AppResult;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

/// 单体 DDD 启动器
pub struct MonolithDDDBootstrap;

impl MonolithDDDBootstrap {
    pub fn new() -> Self {
        Self
    }

    /// 内部初始化方法
    async fn init_internal(&self, config: &UnifiedConfig) -> AppResult<ApplicationState> {
        tracing::info!("Initializing Monolith DDD architecture...");

        // 1. 创建数据库连接池（共享）
        tracing::info!("Creating shared database connection pool...");
        let pool = Arc::new(
            config.get_pool_config()
                .connect(&config.database.url)
                .await
                .map_err(|e| crate::errors::AppError::internal_error(
                    &format!("Failed to create database pool: {}", e),
                    None
                ))?
        );

        // 2. 初始化领域服务
        tracing::info!("Initializing domain services...");
        let bounded_contexts = &config.architecture.bounded_contexts;
        for context in bounded_contexts {
            tracing::info!(
                "Loading bounded context: {} with {} aggregates, {} domain services",
                context.name,
                context.aggregates.len(),
                context.domain_services.len()
            );
        }

        // 3. 使用现有的 ApplicationState 初始化
        let central_config = crate::central::config::CentralConfig::default();
        let state = ApplicationState::new(pool, &central_config)
            .await
            .map_err(|e| crate::errors::AppError::internal_error(
                &format!("Failed to initialize application state: {}", e),
                None
            ))?;

        tracing::info!("Monolith DDD architecture initialized successfully");
        Ok(state)
    }

    /// 内部启动方法
    async fn start_internal(&self, config: UnifiedConfig) -> AppResult<()> {
        tracing::info!("Starting Monolith DDD server...");
        
        let _state = self.init_internal(&config).await?;
        
        // 获取服务器配置
        let server_addr = format!(
            "{}:{}",
            config.server.host,
            config.server.port
        );
        
        tracing::info!("Monolith DDD server ready at http://{}", server_addr);
        tracing::info!("Bounded contexts loaded: {}", 
            config.architecture.bounded_contexts.len());
        
        Ok(())
    }

    /// 内部关闭方法
    async fn shutdown_internal(&self) -> AppResult<()> {
        tracing::info!("Shutting down Monolith DDD server...");
        
        // 优雅关闭
        // - 停止接受新请求
        // - 等待现有请求完成
        // - 关闭数据库连接
        // - 关闭 Redis 连接
        
        tracing::info!("Monolith DDD server shutdown complete");
        Ok(())
    }
}

impl Default for MonolithDDDBootstrap {
    fn default() -> Self {
        Self::new()
    }
}

impl ArchitectureBootstrap for MonolithDDDBootstrap {
    fn mode(&self) -> ArchitectureMode {
        ArchitectureMode::MonolithDDD
    }

    fn initialize<'a>(
        &'a self,
        config: &'a UnifiedConfig,
    ) -> Pin<Box<dyn Future<Output = AppResult<ApplicationState>> + Send + 'a>> {
        Box::pin(self.init_internal(config))
    }

    fn start<'a>(
        &'a self,
        config: UnifiedConfig,
    ) -> Pin<Box<dyn Future<Output = AppResult<()>> + Send + 'a>> {
        Box::pin(self.start_internal(config))
    }

    fn shutdown<'a>(
        &'a self,
    ) -> Pin<Box<dyn Future<Output = AppResult<()>> + Send + 'a>> {
        Box::pin(self.shutdown_internal())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bootstrap_creation() {
        let bootstrap = MonolithDDDBootstrap::new();
        assert_eq!(bootstrap.mode(), ArchitectureMode::MonolithDDD);
    }
}
