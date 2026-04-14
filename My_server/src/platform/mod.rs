//! Platform Layer - Core platform services and abstractions
//!
//! This module provides unified platform-level services including:
//! - Configuration management
//! - Caching abstractions
//! - Security services
//! - Protocol management
//! - Event handling

pub mod cache;
pub mod config;
pub mod security;
pub mod protocols;
pub mod protocols_extended;

pub use cache::{CacheManager, CacheConfig, CacheProvider, CacheError};
pub use config::{ConfigManager, ConfigProvider, ConfigError};
pub use security::{SecurityManager, SecurityConfig, SecurityError};
pub use protocols::{ProtocolManager, ProtocolParser, ProtocolError};
pub use protocols_extended::{ExtendedProtocolManager, ExtendedProtocolError};

use std::sync::Arc;
use thiserror::Error;

/// Unified platform error type
#[derive(Error, Debug)]
pub enum PlatformError {
    #[error("Cache error: {0}")]
    Cache(#[from] CacheError),
    
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),
    
    #[error("Security error: {0}")]
    Security(#[from] SecurityError),
    
    #[error("Protocol error: {0}")]
    Protocol(#[from] ProtocolError),
    
    #[error("Extended protocol error: {0}")]
    ExtendedProtocol(#[from] ExtendedProtocolError),
    
    #[error("Platform initialization error: {0}")]
    Initialization(String),
}

/// Platform context that holds all platform services
pub struct PlatformContext {
    pub cache: Arc<CacheManager>,
    pub config: Arc<ConfigManager>,
    pub security: Arc<SecurityManager>,
    pub protocols: Arc<ProtocolManager>,
    pub extended_protocols: Arc<ExtendedProtocolManager>,
}

impl PlatformContext {
    /// Create a new platform context with all services initialized
    pub async fn new() -> Result<Self, PlatformError> {
        // Initialize configuration first
        let config = Arc::new(ConfigManager::new().await?);
        
        // Initialize other services with configuration
        let cache = Arc::new(CacheManager::new(config.clone()).await?);
        let security = Arc::new(SecurityManager::new(config.clone()).await?);
        let protocols = Arc::new(ProtocolManager::new(config.clone()).await?);
        let extended_protocols = Arc::new(ExtendedProtocolManager::new(config.clone()).await?);
        
        Ok(Self {
            cache,
            config,
            security,
            protocols,
            extended_protocols,
        })
    }
}