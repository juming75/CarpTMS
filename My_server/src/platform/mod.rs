//! Platform Layer - Core platform services and abstractions
//!
//! This module provides unified platform-level services including:
//! - Configuration management
//! - Caching abstractions
//! - Security services
//! - Protocol management
//! - Event handling
//! - Multi-client coordination
//! - Hierarchical alert system
//! - Driving behavior scoring
//! - Secure transport (HTTPS/WSS)
//! - Alert push notifications
//! - Connection diagnostic
//! - Data sync verification
//! - High availability architecture
//! - Automated testing support

pub mod cache;
pub mod config;
pub mod security;
pub mod protocols;
pub mod protocols_extended;
pub mod hierarchical_alert;
pub mod multi_client;
pub mod driving_behavior_scorer;
pub mod secure_transport;
pub mod alert_push;
pub mod connection_diagnostic;
pub mod data_sync_verification;
pub mod high_availability;
pub mod automated_testing;

pub use cache::{CacheManager, CacheConfig, CacheProvider, CacheError};
pub use config::{ConfigManager, ConfigProvider, ConfigError};
pub use security::{SecurityManager, SecurityConfig, SecurityError};
pub use protocols::{ProtocolManager, ProtocolParser, ProtocolError};
pub use protocols_extended::{ExtendedProtocolManager, ExtendedProtocolError};
pub use hierarchical_alert::{HierarchicalAlertManager, AlertLevel, AlertType, AlertRule, create_hierarchical_alert_manager};
pub use multi_client::{MultiClientCoordinator, ClientType, UserInfo, Enterprise, create_multi_client_coordinator};
pub use driving_behavior_scorer::{DrivingBehaviorScorer, DrivingEvent, DrivingScoreRecord, ScoreGrade, create_driving_behavior_scorer};
pub use secure_transport::{SecureTransportConfig, TlsConfig, TlsVersion, HttpsServerConfig, WssConfig, create_secure_transport_config, create_production_secure_config};
pub use alert_push::{AlertPushManager, AlertChannel, AlertPushConfig, PushStats, create_alert_push_manager};
pub use connection_diagnostic::{ConnectionDiagnosticManager, DiagnosticConfig, DeviceDiagnosticReport, DiagnosticStatus, DiagnosticCheck, create_connection_diagnostic_manager, create_connection_diagnostic_manager_with_config};
pub use data_sync_verification::{DataSyncManager, GpsPosition, CoordinateSystem, SyncStatus, SyncVerificationResult, CoordinateConfig, create_data_sync_manager, wgs84_to_gcj02, gcj02_to_wgs84, haversine_distance};
pub use high_availability::{ClusterManager, ClusterConfig, ClusterNode, ClusterStats, NodeStatus, NodeRole, LoadBalanceStrategy, create_cluster_manager, create_cluster_manager_with_config};
pub use automated_testing::{TestManager, TestConfig, TestResult, TestSuite, PerformanceTestResult, create_test_manager, create_test_manager_with_config};

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