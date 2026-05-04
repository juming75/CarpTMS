//! Blue-Green Deployment Module
//!
//! Provides blue-green deployment capabilities for zero-downtime deployments
//! including traffic switching, health checks, and rollback mechanisms.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

/// Deployment error types
#[derive(Error, Debug)]
pub enum DeploymentError {
    #[error("Health check failed: {0}")]
    HealthCheckFailed(String),

    #[error("Traffic switch failed: {0}")]
    TrafficSwitchFailed(String),

    #[error("Rollback failed: {0}")]
    RollbackFailed(String),

    #[error("Deployment timeout: {0}")]
    Timeout(String),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Load balancer error: {0}")]
    LoadBalancer(String),
}

/// Deployment environment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeploymentEnvironment {
    Blue,
    Green,
}

impl std::fmt::Display for DeploymentEnvironment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeploymentEnvironment::Blue => write!(f, "blue"),
            DeploymentEnvironment::Green => write!(f, "green"),
        }
    }
}

/// Deployment status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeploymentStatus {
    Pending,
    InProgress,
    HealthChecking,
    Ready,
    Live,
    Failed,
    RollingBack,
    RolledBack,
}

impl std::fmt::Display for DeploymentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeploymentStatus::Pending => write!(f, "pending"),
            DeploymentStatus::InProgress => write!(f, "in_progress"),
            DeploymentStatus::HealthChecking => write!(f, "health_checking"),
            DeploymentStatus::Ready => write!(f, "ready"),
            DeploymentStatus::Live => write!(f, "live"),
            DeploymentStatus::Failed => write!(f, "failed"),
            DeploymentStatus::RollingBack => write!(f, "rolling_back"),
            DeploymentStatus::RolledBack => write!(f, "rolled_back"),
        }
    }
}

/// Deployment configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentConfig {
    pub application_name: String,
    pub blue_endpoint: String,
    pub green_endpoint: String,
    pub health_check_path: String,
    pub health_check_timeout: Duration,
    pub health_check_interval: Duration,
    pub health_check_retries: u32,
    pub traffic_switch_timeout: Duration,
    pub rollback_timeout: Duration,
    pub deployment_timeout: Duration,
    pub max_concurrent_requests: u32,
    pub circuit_breaker_threshold: f32,
    pub enable_canary_deployment: bool,
    pub canary_percentage: f32,
}

impl Default for DeploymentConfig {
    fn default() -> Self {
        Self {
            application_name: "CarpTMS".to_string(),
            blue_endpoint: "http://blue.CarpTMS.local:8080".to_string(),
            green_endpoint: "http://green.CarpTMS.local:8080".to_string(),
            health_check_path: "/health".to_string(),
            health_check_timeout: Duration::from_secs(30),
            health_check_interval: Duration::from_secs(5),
            health_check_retries: 3,
            traffic_switch_timeout: Duration::from_secs(60),
            rollback_timeout: Duration::from_secs(30),
            deployment_timeout: Duration::from_secs(300),
            max_concurrent_requests: 1000,
            circuit_breaker_threshold: 0.95,
            enable_canary_deployment: true,
            canary_percentage: 0.1,
        }
    }
}

/// Deployment instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentInstance {
    pub id: String,
    pub environment: DeploymentEnvironment,
    pub version: String,
    pub status: DeploymentStatus,
    pub endpoint: String,
    pub health_status: HealthStatus,
    pub traffic_percentage: f32,
    pub deployment_time: chrono::DateTime<chrono::Utc>,
    pub last_health_check: Option<chrono::DateTime<chrono::Utc>>,
}

/// Health status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    Unknown,
    Healthy,
    Unhealthy,
    Degraded,
}

/// Health check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult {
    pub status: HealthStatus,
    pub response_time: Duration,
    pub status_code: u16,
    pub error_message: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Traffic routing rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficRoutingRule {
    pub environment: DeploymentEnvironment,
    pub percentage: f32,
    pub conditions: Vec<RoutingCondition>,
}

/// Routing condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingCondition {
    pub field: String,
    pub operator: String,
    pub value: String,
}

/// Load balancer trait
#[async_trait]
pub trait LoadBalancer: Send + Sync {
    /// Update routing rules
    async fn update_routing(&self, rules: Vec<TrafficRoutingRule>) -> Result<(), DeploymentError>;

    /// Get current routing rules
    async fn get_routing_rules(&self) -> Result<Vec<TrafficRoutingRule>, DeploymentError>;

    /// Get traffic statistics
    async fn get_traffic_stats(&self) -> Result<TrafficStats, DeploymentError>;
}

/// Traffic statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficStats {
    pub total_requests: u64,
    pub blue_requests: u64,
    pub green_requests: u64,
    pub blue_success_rate: f32,
    pub green_success_rate: f32,
    pub blue_response_time: Duration,
    pub green_response_time: Duration,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Health checker trait
#[async_trait]
pub trait HealthChecker: Send + Sync {
    /// Check health of an endpoint
    async fn check_health(&self, endpoint: &str) -> Result<HealthCheckResult, DeploymentError>;

    /// Check health of multiple endpoints
    async fn check_multiple_endpoints(
        &self,
        endpoints: &[String],
    ) -> Result<Vec<HealthCheckResult>, DeploymentError>;
}

/// HTTP health checker
pub struct HttpHealthChecker {
    client: reqwest::Client,
}

impl HttpHealthChecker {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap();

        Self { client }
    }
}

impl Default for HttpHealthChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl HealthChecker for HttpHealthChecker {
    async fn check_health(&self, endpoint: &str) -> Result<HealthCheckResult, DeploymentError> {
        let start = std::time::Instant::now();

        match self.client.get(endpoint).send().await {
            Ok(response) => {
                let response_time = start.elapsed();
                let status_code = response.status().as_u16();

                let health_status = if response.status().is_success() {
                    HealthStatus::Healthy
                } else if response.status().is_server_error() {
                    HealthStatus::Unhealthy
                } else {
                    HealthStatus::Degraded
                };

                Ok(HealthCheckResult {
                    status: health_status,
                    response_time,
                    status_code,
                    error_message: None,
                    timestamp: chrono::Utc::now(),
                })
            }
            Err(e) => {
                let response_time = start.elapsed();

                Ok(HealthCheckResult {
                    status: HealthStatus::Unhealthy,
                    response_time,
                    status_code: 0,
                    error_message: Some(e.to_string()),
                    timestamp: chrono::Utc::now(),
                })
            }
        }
    }

    async fn check_multiple_endpoints(
        &self,
        endpoints: &[String],
    ) -> Result<Vec<HealthCheckResult>, DeploymentError> {
        let mut results = Vec::new();

        for endpoint in endpoints {
            let result = self.check_health(endpoint).await?;
            results.push(result);
        }

        Ok(results)
    }
}

/// Simple load balancer implementation
pub struct SimpleLoadBalancer {
    routing_rules: Arc<RwLock<Vec<TrafficRoutingRule>>>,
    stats: Arc<RwLock<TrafficStats>>,
}

impl SimpleLoadBalancer {
    pub fn new() -> Self {
        Self {
            routing_rules: Arc::new(RwLock::new(Vec::new())),
            stats: Arc::new(RwLock::new(TrafficStats {
                total_requests: 0,
                blue_requests: 0,
                green_requests: 0,
                blue_success_rate: 1.0,
                green_success_rate: 1.0,
                blue_response_time: Duration::from_millis(100),
                green_response_time: Duration::from_millis(100),
                timestamp: chrono::Utc::now(),
            })),
        }
    }
}

impl Default for SimpleLoadBalancer {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl LoadBalancer for SimpleLoadBalancer {
    async fn update_routing(&self, rules: Vec<TrafficRoutingRule>) -> Result<(), DeploymentError> {
        let mut routing_rules = self.routing_rules.write().await;
        *routing_rules = rules;
        Ok(())
    }

    async fn get_routing_rules(&self) -> Result<Vec<TrafficRoutingRule>, DeploymentError> {
        let routing_rules = self.routing_rules.read().await;
        Ok(routing_rules.clone())
    }

    async fn get_traffic_stats(&self) -> Result<TrafficStats, DeploymentError> {
        let stats = self.stats.read().await;
        Ok(stats.clone())
    }
}

/// Blue-green deployment manager
pub struct BlueGreenDeploymentManager {
    config: DeploymentConfig,
    blue_instance: Arc<RwLock<Option<DeploymentInstance>>>,
    green_instance: Arc<RwLock<Option<DeploymentInstance>>>,
    current_environment: Arc<RwLock<DeploymentEnvironment>>,
    load_balancer: Arc<dyn LoadBalancer>,
    health_checker: Arc<dyn HealthChecker>,
    deployment_history: Arc<RwLock<Vec<DeploymentInstance>>>,
}

impl BlueGreenDeploymentManager {
    /// Create a new blue-green deployment manager
    pub async fn new(
        config: DeploymentConfig,
        load_balancer: Arc<dyn LoadBalancer>,
        health_checker: Arc<dyn HealthChecker>,
    ) -> Result<Self, DeploymentError> {
        let manager = Self {
            config,
            blue_instance: Arc::new(RwLock::new(None)),
            green_instance: Arc::new(RwLock::new(None)),
            current_environment: Arc::new(RwLock::new(DeploymentEnvironment::Blue)),
            load_balancer,
            health_checker,
            deployment_history: Arc::new(RwLock::new(Vec::new())),
        };

        // Initialize with current deployment
        manager.initialize_current_deployment().await?;

        Ok(manager)
    }

    /// Initialize current deployment
    async fn initialize_current_deployment(&self) -> Result<(), DeploymentError> {
        // Create initial blue deployment
        let blue_instance = DeploymentInstance {
            id: uuid::Uuid::new_v4().to_string(),
            environment: DeploymentEnvironment::Blue,
            version: "1.0.0".to_string(),
            status: DeploymentStatus::Live,
            endpoint: self.config.blue_endpoint.clone(),
            health_status: HealthStatus::Unknown,
            traffic_percentage: 100.0,
            deployment_time: chrono::Utc::now(),
            last_health_check: None,
        };

        *self.blue_instance.write().await = Some(blue_instance);

        // Set initial routing to blue
        let routing_rules = vec![TrafficRoutingRule {
            environment: DeploymentEnvironment::Blue,
            percentage: 100.0,
            conditions: vec![],
        }];

        self.load_balancer.update_routing(routing_rules).await?;

        Ok(())
    }

    /// Deploy new version
    pub async fn deploy(
        &self,
        version: String,
        environment: DeploymentEnvironment,
    ) -> Result<(), DeploymentError> {
        info!(
            "Starting deployment of version {} to {} environment",
            version, environment
        );

        // Create deployment instance
        let instance = DeploymentInstance {
            id: uuid::Uuid::new_v4().to_string(),
            environment,
            version: version.clone(),
            status: DeploymentStatus::InProgress,
            endpoint: match environment {
                DeploymentEnvironment::Blue => self.config.blue_endpoint.clone(),
                DeploymentEnvironment::Green => self.config.green_endpoint.clone(),
            },
            health_status: HealthStatus::Unknown,
            traffic_percentage: 0.0,
            deployment_time: chrono::Utc::now(),
            last_health_check: None,
        };

        // Store deployment instance
        match environment {
            DeploymentEnvironment::Blue => {
                *self.blue_instance.write().await = Some(instance.clone());
            }
            DeploymentEnvironment::Green => {
                *self.green_instance.write().await = Some(instance.clone());
            }
        }

        // Perform health checks
        self.perform_health_checks(environment).await?;

        // Switch traffic
        self.switch_traffic(environment, 100.0).await?;

        // Update current environment
        *self.current_environment.write().await = environment;

        // Add to history
        self.deployment_history.write().await.push(instance);

        info!(
            "Deployment of version {} to {} environment completed successfully",
            version, environment
        );
        Ok(())
    }

    /// Perform health checks
    async fn perform_health_checks(
        &self,
        environment: DeploymentEnvironment,
    ) -> Result<(), DeploymentError> {
        info!("Performing health checks for {} environment", environment);

        let instance = match environment {
            DeploymentEnvironment::Blue => self.blue_instance.read().await.clone(),
            DeploymentEnvironment::Green => self.green_instance.read().await.clone(),
        };

        if let Some(instance) = instance {
            let health_url = format!("{}{}", instance.endpoint, self.config.health_check_path);

            for i in 0..self.config.health_check_retries {
                info!(
                    "Health check attempt {} for {} environment",
                    i + 1,
                    environment
                );

                match self.health_checker.check_health(&health_url).await {
                    Ok(result) => {
                        if result.status == HealthStatus::Healthy {
                            info!("Health check passed for {} environment", environment);
                            return Ok(());
                        } else {
                            warn!(
                                "Health check failed for {} environment: {:?}",
                                environment, result
                            );
                        }
                    }
                    Err(e) => {
                        error!("Health check error for {} environment: {}", environment, e);
                    }
                }

                if i < self.config.health_check_retries - 1 {
                    tokio::time::sleep(self.config.health_check_interval).await;
                }
            }

            return Err(DeploymentError::HealthCheckFailed(format!(
                "Health checks failed after {} attempts for {} environment",
                self.config.health_check_retries, environment
            )));
        }

        Err(DeploymentError::Configuration(
            "Deployment instance not found".to_string(),
        ))
    }

    /// Switch traffic between environments
    async fn switch_traffic(
        &self,
        environment: DeploymentEnvironment,
        percentage: f32,
    ) -> Result<(), DeploymentError> {
        info!(
            "Switching traffic to {} environment with {}%",
            environment, percentage
        );

        let routing_rules = vec![TrafficRoutingRule {
            environment,
            percentage,
            conditions: vec![],
        }];

        self.load_balancer.update_routing(routing_rules).await?;

        info!("Traffic switch completed successfully");
        Ok(())
    }

    /// Rollback to previous version
    pub async fn rollback(&self) -> Result<(), DeploymentError> {
        info!("Starting rollback");

        let current_env = *self.current_environment.read().await;
        let previous_env = match current_env {
            DeploymentEnvironment::Blue => DeploymentEnvironment::Green,
            DeploymentEnvironment::Green => DeploymentEnvironment::Blue,
        };

        // Check if previous environment is healthy
        self.perform_health_checks(previous_env).await?;

        // Switch traffic back to previous environment
        self.switch_traffic(previous_env, 100.0).await?;

        // Update current environment
        *self.current_environment.write().await = previous_env;

        info!("Rollback completed successfully");
        Ok(())
    }

    /// Get current deployment status
    pub async fn get_status(&self) -> Result<DeploymentStatus, DeploymentError> {
        let current_env = *self.current_environment.read().await;

        let instance = match current_env {
            DeploymentEnvironment::Blue => self.blue_instance.read().await.clone(),
            DeploymentEnvironment::Green => self.green_instance.read().await.clone(),
        };

        Ok(instance
            .map(|i| i.status)
            .unwrap_or(DeploymentStatus::Failed))
    }

    /// Get deployment statistics
    pub async fn get_stats(&self) -> Result<TrafficStats, DeploymentError> {
        self.load_balancer.get_traffic_stats().await
    }

    /// Get deployment history
    pub async fn get_deployment_history(&self) -> Result<Vec<DeploymentInstance>, DeploymentError> {
        let history = self.deployment_history.read().await;
        Ok(history.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_deployment_config() {
        let config = DeploymentConfig::default();
        assert_eq!(config.application_name, "CarpTMS");
        assert_eq!(config.health_check_retries, 3);
    }

    #[tokio::test]
    async fn test_health_checker() {
        let health_checker = HttpHealthChecker::new();

        // Test with invalid endpoint
        let result = health_checker
            .check_health("http://invalid.endpoint:9999/health")
            .await
            .unwrap();
        assert_eq!(result.status, HealthStatus::Unhealthy);
        assert!(result.error_message.is_some());
    }

    #[tokio::test]
    async fn test_load_balancer() {
        let load_balancer = SimpleLoadBalancer::new();

        let routing_rules = vec![TrafficRoutingRule {
            environment: DeploymentEnvironment::Blue,
            percentage: 100.0,
            conditions: vec![],
        }];

        load_balancer.update_routing(routing_rules).await.unwrap();

        let rules = load_balancer.get_routing_rules().await.unwrap();
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].environment, DeploymentEnvironment::Blue);
        assert_eq!(rules[0].percentage, 100.0);
    }

    #[tokio::test]
    async fn test_deployment_manager_initialization() {
        let config = DeploymentConfig::default();
        let load_balancer = Arc::new(SimpleLoadBalancer::new());
        let health_checker = Arc::new(HttpHealthChecker::new());

        let manager = BlueGreenDeploymentManager::new(config, load_balancer, health_checker)
            .await
            .unwrap();

        let status = manager.get_status().await.unwrap();
        assert_eq!(status, DeploymentStatus::Live);

        let history = manager.get_deployment_history().await.unwrap();
        assert_eq!(history.len(), 1);
    }
}
