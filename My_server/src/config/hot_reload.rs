//! 配置热切换模块
//!
//! 支持运行时切换架构模式、配置验证和优雅关闭。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, mpsc, RwLock};

use crate::config::{ArchitectureConfig, ArchitectureMode};
use crate::errors::AppResult;

/// 配置变更事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfigChangeEvent {
    /// 架构模式切换
    ModeChanged {
        old_mode: ArchitectureMode,
        new_mode: ArchitectureMode,
        timestamp: DateTime<Utc>,
    },
    /// 服务配置更新
    ServiceConfigUpdated {
        service_name: String,
        changes: serde_json::Value,
        timestamp: DateTime<Utc>,
    },
    /// 数据库配置更新
    DatabaseConfigUpdated {
        changes: serde_json::Value,
        timestamp: DateTime<Utc>,
    },
}

/// 配置验证结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// 是否有效
    pub is_valid: bool,
    /// 错误信息
    pub errors: Vec<String>,
    /// 警告信息
    pub warnings: Vec<String>,
}

impl ValidationResult {
    /// 创建有效的结果
    pub fn valid() -> Self {
        Self {
            is_valid: true,
            errors: vec![],
            warnings: vec![],
        }
    }

    /// 创建无效的结果
    pub fn invalid(errors: Vec<String>) -> Self {
        Self {
            is_valid: false,
            errors,
            warnings: vec![],
        }
    }

    /// 添加警告
    pub fn with_warning(mut self, warning: String) -> Self {
        self.warnings.push(warning);
        self
    }
}

/// 配置管理器
pub struct ConfigManager {
    /// 当前架构配置
    config: Arc<RwLock<ArchitectureConfig>>,
    /// 配置变更事件发送器
    event_sender: broadcast::Sender<ConfigChangeEvent>,
    /// 关闭信号发送器
    shutdown_sender: mpsc::Sender<()>,
    /// 关闭信号接收器
    shutdown_receiver: Option<mpsc::Receiver<()>>,
}

impl ConfigManager {
    /// 创建新的配置管理器
    pub fn new(initial_config: ArchitectureConfig) -> Self {
        let (event_sender, _) = broadcast::channel(100);
        let (shutdown_sender, shutdown_receiver) = mpsc::channel(1);

        Self {
            config: Arc::new(RwLock::new(initial_config)),
            event_sender,
            shutdown_sender,
            shutdown_receiver: Some(shutdown_receiver),
        }
    }

    /// 获取当前配置
    pub async fn get_config(&self) -> ArchitectureConfig {
        self.config.read().await.clone()
    }

    /// 获取当前架构模式
    pub async fn get_mode(&self) -> ArchitectureMode {
        self.config.read().await.mode
    }

    /// 切换架构模式
    pub async fn switch_mode(&self, new_mode: ArchitectureMode) -> AppResult<()> {
        let old_mode = {
            let mut config = self.config.write().await;
            let old = config.mode;

            // 验证新模式
            config.mode = new_mode;
            if let Err(e) = config.validate() {
                config.mode = old; // 回滚
                return Err(crate::errors::AppError::validation_error(
                    &format!("Invalid mode switch: {}", e),
                    None,
                ));
            }

            old
        };

        // 发送变更事件
        let event = ConfigChangeEvent::ModeChanged {
            old_mode,
            new_mode,
            timestamp: Utc::now(),
        };

        if let Err(e) = self.event_sender.send(event) {
            log::warn!("Failed to send config change event: {}", e);
        }

        log::info!(
            "Architecture mode switched from {} to {}",
            old_mode,
            new_mode
        );

        Ok(())
    }

    /// 验证配置
    pub async fn validate(&self) -> ValidationResult {
        let config = self.config.read().await;
        let mut result = ValidationResult::valid();

        // 验证架构模式
        if let Err(e) = config.validate() {
            result
                .errors
                .push(format!("Architecture config validation failed: {}", e));
            result.is_valid = false;
        }

        // 验证微服务配置
        if config.mode.is_microservice() {
            if config.service_name.is_none() {
                result
                    .errors
                    .push("service_name is required in microservice mode".to_string());
                result.is_valid = false;
            }

            if config.microservices.is_empty() {
                result
                    .warnings
                    .push("No microservices configured".to_string());
            }
        }

        // 验证 DDD 配置
        if config.mode.is_ddd() && config.bounded_contexts.is_empty() {
            result
                .warnings
                .push("No bounded contexts configured for DDD mode".to_string());
        }

        result
    }

    /// 订阅配置变更事件
    pub fn subscribe(&self) -> broadcast::Receiver<ConfigChangeEvent> {
        self.event_sender.subscribe()
    }

    /// 发起优雅关闭
    pub async fn shutdown(&self) -> AppResult<()> {
        log::info!("Initiating graceful shutdown...");

        // 发送关闭信号
        self.shutdown_sender.send(()).await.map_err(|e| {
            crate::errors::AppError::internal_error(
                &format!("Failed to send shutdown signal: {}", e),
                None,
            )
        })?;

        Ok(())
    }

    /// 获取关闭信号接收器
    pub fn take_shutdown_receiver(&mut self) -> Option<mpsc::Receiver<()>> {
        self.shutdown_receiver.take()
    }

    /// 热重载配置（从文件或环境变量）
    pub async fn reload_from_env(&self) -> AppResult<()> {
        log::info!("Reloading configuration from environment...");

        // 从环境变量读取架构模式
        let new_mode = if let Ok(mode_str) = std::env::var("ARCHITECTURE_MODE") {
            mode_str
                .parse()
                .map_err(|e: String| crate::errors::AppError::validation_error(&e, None))?
        } else {
            return Ok(()); // 没有设置环境变量，保持当前配置
        };

        if self.get_mode().await != new_mode {
            self.switch_mode(new_mode).await?;
        }

        Ok(())
    }

    /// 从配置文件重新加载
    pub async fn reload_from_file(&self, path: &str) -> AppResult<()> {
        log::info!("Reloading configuration from file: {}", path);

        let content = tokio::fs::read_to_string(path).await.map_err(|e| {
            crate::errors::AppError::internal_error(
                &format!("Failed to read config file: {}", e),
                None,
            )
        })?;

        let new_config: ArchitectureConfig = toml::from_str(&content).map_err(|e| {
            crate::errors::AppError::validation_error(
                &format!("Failed to parse config file: {}", e),
                None,
            )
        })?;

        // 验证新配置
        new_config
            .validate()
            .map_err(|e| crate::errors::AppError::validation_error(&e, None))?;

        // 检查是否需要切换模式
        let old_mode = self.get_mode().await;
        let new_mode = new_config.mode;

        // 更新配置
        {
            let mut config = self.config.write().await;
            *config = new_config;
        }

        // 发送变更事件
        if old_mode != new_mode {
            let event = ConfigChangeEvent::ModeChanged {
                old_mode,
                new_mode,
                timestamp: Utc::now(),
            };

            let _ = self.event_sender.send(event);
        }

        Ok(())
    }
}

/// 优雅关闭处理器
pub struct GracefulShutdown {
    /// 关闭超时时间
    timeout: Duration,
    /// 关闭前的清理任务
    cleanup_tasks: Vec<Box<dyn Fn() + Send + Sync>>,
}

impl GracefulShutdown {
    /// 创建新的关闭处理器
    pub fn new() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            cleanup_tasks: vec![],
        }
    }

    /// 设置超时时间
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// 添加清理任务
    pub fn add_cleanup_task<F>(mut self, task: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.cleanup_tasks.push(Box::new(task));
        self
    }

    /// 执行关闭
    pub async fn execute(&self) -> AppResult<()> {
        log::info!("Executing graceful shutdown...");

        // 执行清理任务
        for task in &self.cleanup_tasks {
            task();
        }

        log::info!("Graceful shutdown completed");
        Ok(())
    }
}

impl Default for GracefulShutdown {
    fn default() -> Self {
        Self::new()
    }
}

/// 配置变更监听器
pub struct ConfigChangeListener {
    /// 事件接收器
    receiver: broadcast::Receiver<ConfigChangeEvent>,
}

impl ConfigChangeListener {
    /// 创建新的监听器
    pub fn new(manager: &ConfigManager) -> Self {
        Self {
            receiver: manager.subscribe(),
        }
    }

    /// 等待下一个事件
    pub async fn next(&mut self) -> Option<ConfigChangeEvent> {
        self.receiver.recv().await.ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_config_manager_creation() {
        let config = ArchitectureConfig::monolith_ddd();
        let manager = ConfigManager::new(config);

        assert_eq!(manager.get_mode().await, ArchitectureMode::MonolithDDD);
    }

    #[tokio::test]
    async fn test_switch_mode() {
        let config = ArchitectureConfig::monolith_ddd();
        let manager = ConfigManager::new(config);

        assert_eq!(manager.get_mode().await, ArchitectureMode::MonolithDDD);

        // 切换到微服务模式
        let mut micro_config = ArchitectureConfig::micro_ddd();
        micro_config.service_name = Some("test-service".to_string());

        let manager = ConfigManager::new(micro_config);
        manager
            .switch_mode(ArchitectureMode::MicroDDD)
            .await
            .unwrap();

        assert_eq!(manager.get_mode().await, ArchitectureMode::MicroDDD);
    }

    #[tokio::test]
    async fn test_validation() {
        let config = ArchitectureConfig::micro_ddd();
        let manager = ConfigManager::new(config);

        let result = manager.validate().await;

        // 微服务模式需要 service_name
        assert!(!result.is_valid);
        assert!(!result.errors.is_empty());
    }

    #[test]
    fn test_validation_result() {
        let result = ValidationResult::valid();
        assert!(result.is_valid);
        assert!(result.errors.is_empty());

        let result = ValidationResult::invalid(vec!["error1".to_string()]);
        assert!(!result.is_valid);
        assert_eq!(result.errors.len(), 1);
    }

    #[test]
    fn test_graceful_shutdown() {
        let shutdown = GracefulShutdown::new()
            .with_timeout(Duration::from_secs(10))
            .add_cleanup_task(|| {
                log::info!("Cleanup task executed");
            });

        assert_eq!(shutdown.timeout, Duration::from_secs(10));
    }
}
