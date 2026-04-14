//! Health Checker module
//! Handles health checking of services and updates their status

use std::sync::Arc;
use tokio::sync::RwLock;
use log::{info, warn, debug};
use std::time::{Duration, Instant};
use reqwest::Client;

/// Health check result
#[derive(Clone, Debug)]
pub enum HealthCheckResult {
    Healthy,
    Unhealthy(String), // error message
    Timeout,
}

/// Health status
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HealthStatus {
    Healthy,
    Unhealthy,
    Unknown,
    Starting,
}

/// Health check configuration
#[derive(Clone, Debug)]
pub struct HealthCheckConfig {
    pub endpoint: String,
    pub interval: Duration,
    pub timeout: Duration,
    pub failure_threshold: u32,
    pub success_threshold: u32,
}

/// Service health information
#[derive(Clone, Debug)]
pub struct ServiceHealth {
    pub service_id: String,
    pub status: HealthStatus,
    pub last_check: Instant,
    pub failure_count: u32,
    pub success_count: u32,
    pub last_error: Option<String>,
}

/// Health checker
pub struct HealthChecker {
    client: Client,
    service_health: Arc<RwLock<std::collections::HashMap<String, ServiceHealth>>>,
    configs: Arc<RwLock<std::collections::HashMap<String, HealthCheckConfig>>>,
}

impl HealthChecker {
    /// Create a new health checker
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            service_health: Arc::new(RwLock::new(std::collections::HashMap::new())),
            configs: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// Add a service for health checking
    pub async fn add_service(&self, service_id: String, config: HealthCheckConfig) {
        let mut service_health = self.service_health.write().await;
        let mut configs = self.configs.write().await;

        service_health.insert(service_id.clone(), ServiceHealth {
            service_id: service_id.clone(),
            status: HealthStatus::Starting,
            last_check: Instant::now(),
            failure_count: 0,
            success_count: 0,
            last_error: None,
        });

        configs.insert(service_id, config);

        info!("Added service {} for health checking", service_id);
    }

    /// Remove a service from health checking
    pub async fn remove_service(&self, service_id: &str) {
        let mut service_health = self.service_health.write().await;
        let mut configs = self.configs.write().await;

        service_health.remove(service_id);
        configs.remove(service_id);

        info!("Removed service {} from health checking", service_id);
    }

    /// Check health of a service
    pub async fn check_health(&self, service_id: &str, base_url: &str) -> HealthCheckResult {
        let configs = self.configs.read().await;
        let config = match configs.get(service_id) {
            Some(config) => config,
            None => {
                return HealthCheckResult::Unhealthy("No health check config for service".to_string());
            }
        };

        let url = format!("{}{}", base_url, config.endpoint);
        debug!("Checking health of service {} at {}", service_id, url);

        match tokio::time::timeout(config.timeout, self.client.get(&url).send()).await {
            Ok(Ok(response)) => {
                if response.status().is_success() {
                    HealthCheckResult::Healthy
                } else {
                    HealthCheckResult::Unhealthy(format!("HTTP status: {}", response.status()))
                }
            }
            Ok(Err(e)) => {
                HealthCheckResult::Unhealthy(format!("Request error: {:?}", e))
            }
            Err(_) => {
                HealthCheckResult::Timeout
            }
        }
    }

    /// Update service health status based on check result
    pub async fn update_health_status(&self, service_id: &str, result: HealthCheckResult) {
        let mut service_health = self.service_health.write().await;
        let mut configs = self.configs.write().await;

        let health = match service_health.get_mut(service_id) {
            Some(health) => health,
            None => return,
        };

        let config = match configs.get(service_id) {
            Some(config) => config,
            None => return,
        };

        match result {
            HealthCheckResult::Healthy => {
                health.success_count += 1;
                health.failure_count = 0;
                health.last_error = None;

                if health.status != HealthStatus::Healthy && health.success_count >= config.success_threshold {
                    health.status = HealthStatus::Healthy;
                    info!("Service {} is now healthy", service_id);
                }
            }
            HealthCheckResult::Unhealthy(error) => {
                health.failure_count += 1;
                health.success_count = 0;
                health.last_error = Some(error);

                if health.status != HealthStatus::Unhealthy && health.failure_count >= config.failure_threshold {
                    health.status = HealthStatus::Unhealthy;
                    warn!("Service {} is now unhealthy", service_id);
                }
            }
            HealthCheckResult::Timeout => {
                health.failure_count += 1;
                health.success_count = 0;
                health.last_error = Some("Health check timeout".to_string());

                if health.status != HealthStatus::Unhealthy && health.failure_count >= config.failure_threshold {
                    health.status = HealthStatus::Unhealthy;
                    warn!("Service {} is now unhealthy (timeout)", service_id);
                }
            }
        }

        health.last_check = Instant::now();
    }

    /// Get service health status
    pub async fn get_service_health(&self, service_id: &str) -> Option<ServiceHealth> {
        let service_health = self.service_health.read().await;
        service_health.get(service_id).cloned()
    }

    /// Get all service health statuses
    pub async fn get_all_service_health(&self) -> std::collections::HashMap<String, ServiceHealth> {
        self.service_health.read().await.clone()
    }

    /// Start health checking task
    pub async fn start_health_checking(&self) {
        let checker = self.clone();

        tokio::spawn(async move {
            loop {
                // Check each service
                let service_health = checker.service_health.read().await;
                let configs = checker.configs.read().await;

                for (service_id, health) in service_health.iter() {
                    if let Some(config) = configs.get(service_id) {
                        // Check if it's time to check health
                        let elapsed = Instant::now().duration_since(health.last_check);
                        if elapsed >= config.interval {
                            // In a real implementation, you would get the service URL from the registry
                            let base_url = "http://localhost:8080"; // Placeholder
                            let result = checker.check_health(service_id, base_url).await;
                            checker.update_health_status(service_id, result).await;
                        }
                    }
                }

                // Sleep for a short time to avoid busy looping
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        });

        info!("Started health checking task");
    }
}

impl Default for HealthChecker {
    fn default() -> Self {
        Self::new()
    }
}
