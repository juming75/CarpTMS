//! Service Registry module
//! Handles service registration, discovery, and metadata management

use std::sync::Arc;
use tokio::sync::RwLock;
use log::{info, warn, debug};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

/// Service metadata
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServiceMetadata {
    pub service_name: String,
    pub version: String,
    pub tags: Vec<String>,
    pub endpoints: Vec<ServiceEndpoint>,
    pub health_check: Option<HealthCheckConfig>,
    pub env: String,
    pub region: String,
    pub zone: String,
}

/// Service endpoint
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServiceEndpoint {
    pub name: String,
    pub url: String,
    pub method: String,
    pub timeout: u64, // milliseconds
}

/// Health check configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    pub endpoint: String,
    pub interval: u64, // seconds
    pub timeout: u64, // milliseconds
    pub threshold: u32, // failure threshold
}

/// Service registration request
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServiceRegistration {
    pub service_id: String,
    pub service_name: String,
    pub address: String,
    pub port: u16,
    pub metadata: ServiceMetadata,
}

/// Service registry
pub struct ServiceRegistry {
    services: Arc<RwLock<HashMap<String, ServiceRegistration>>>, // service_id -> registration
    service_index: Arc<RwLock<HashMap<String, HashSet<String>>>>, // service_name -> service_ids
    heartbeats: Arc<RwLock<HashMap<String, Instant>>>, // service_id -> last heartbeat
    metadata_index: Arc<RwLock<HashMap<String, HashSet<String>>>>, // tag -> service_ids
}

impl ServiceRegistry {
    /// Create a new service registry
    pub fn new() -> Self {
        Self {
            services: Arc::new(RwLock::new(HashMap::new())),
            service_index: Arc::new(RwLock::new(HashMap::new())),
            heartbeats: Arc::new(RwLock::new(HashMap::new())),
            metadata_index: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a service
    pub async fn register(&self, registration: ServiceRegistration) {
        let mut services = self.services.write().await;
        let mut service_index = self.service_index.write().await;
        let mut heartbeats = self.heartbeats.write().await;
        let mut metadata_index = self.metadata_index.write().await;

        // Add to services map
        services.insert(registration.service_id.clone(), registration.clone());

        // Update service index
        service_index
            .entry(registration.service_name.clone())
            .or_insert(HashSet::new())
            .insert(registration.service_id.clone());

        // Update heartbeat
        heartbeats.insert(registration.service_id.clone(), Instant::now());

        // Update metadata index
        for tag in &registration.metadata.tags {
            metadata_index
                .entry(tag.clone())
                .or_insert(HashSet::new())
                .insert(registration.service_id.clone());
        }

        info!("Registered service: {} ({}:{})", 
              registration.service_id, registration.address, registration.port);
    }

    /// Unregister a service
    pub async fn unregister(&self, service_id: &str) {
        let mut services = self.services.write().await;
        let mut service_index = self.service_index.write().await;
        let mut heartbeats = self.heartbeats.write().await;
        let mut metadata_index = self.metadata_index.write().await;

        // Get service information before removing
        let service = services.get(service_id);
        if let Some(service) = service {
            // Remove from service index
            if let Some(ids) = service_index.get_mut(&service.service_name) {
                ids.remove(service_id);
                if ids.is_empty() {
                    service_index.remove(&service.service_name);
                }
            }

            // Remove from metadata index
            for tag in &service.metadata.tags {
                if let Some(ids) = metadata_index.get_mut(tag) {
                    ids.remove(service_id);
                    if ids.is_empty() {
                        metadata_index.remove(tag);
                    }
                }
            }
        }

        // Remove from services and heartbeats
        services.remove(service_id);
        heartbeats.remove(service_id);

        info!("Unregistered service: {}", service_id);
    }

    /// Update heartbeat for a service
    pub async fn update_heartbeat(&self, service_id: &str) -> bool {
        let mut heartbeats = self.heartbeats.write().await;
        let mut services = self.services.write().await;

        if services.contains_key(service_id) {
            heartbeats.insert(service_id.to_string(), Instant::now());
            true
        } else {
            false
        }
    }

    /// Get service by ID
    pub async fn get_service(&self, service_id: &str) -> Option<ServiceRegistration> {
        let services = self.services.read().await;
        services.get(service_id).cloned()
    }

    /// Get services by name
    pub async fn get_services_by_name(&self, service_name: &str) -> Vec<ServiceRegistration> {
        let services = self.services.read().await;
        let service_index = self.service_index.read().await;

        if let Some(ids) = service_index.get(service_name) {
            ids.iter()
                .filter_map(|id| services.get(id))
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get services by tag
    pub async fn get_services_by_tag(&self, tag: &str) -> Vec<ServiceRegistration> {
        let services = self.services.read().await;
        let metadata_index = self.metadata_index.read().await;

        if let Some(ids) = metadata_index.get(tag) {
            ids.iter()
                .filter_map(|id| services.get(id))
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get all services
    pub async fn get_all_services(&self) -> Vec<ServiceRegistration> {
        let services = self.services.read().await;
        services.values().cloned().collect()
    }

    /// Cleanup expired services
    pub async fn cleanup_expired_services(&self, timeout: Duration) {
        let now = Instant::now();
        let mut heartbeats = self.heartbeats.write().await;
        let mut services = self.services.write().await;
        let mut service_index = self.service_index.write().await;
        let mut metadata_index = self.metadata_index.write().await;

        let expired: Vec<String> = heartbeats
            .iter()
            .filter(|(_, last_heartbeat)| now.duration_since(**last_heartbeat) > timeout)
            .map(|(id, _)| id.clone())
            .collect();

        for id in expired {
            warn!("Service {} has expired, removing from registry", id);
            
            // Get service information before removing
            let service = services.get(&id);
            if let Some(service) = service {
                // Remove from service index
                if let Some(ids) = service_index.get_mut(&service.service_name) {
                    ids.remove(&id);
                    if ids.is_empty() {
                        service_index.remove(&service.service_name);
                    }
                }

                // Remove from metadata index
                for tag in &service.metadata.tags {
                    if let Some(ids) = metadata_index.get_mut(tag) {
                        ids.remove(&id);
                        if ids.is_empty() {
                            metadata_index.remove(tag);
                        }
                    }
                }
            }

            // Remove from services and heartbeats
            services.remove(&id);
            heartbeats.remove(&id);
        }
    }

    /// Start cleanup task
    pub async fn start_cleanup_task(&self, interval: Duration, timeout: Duration) {
        let registry = self.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(interval);
            
            loop {
                interval.tick().await;
                registry.cleanup_expired_services(timeout).await;
            }
        });

        info!("Started service registry cleanup task with interval {:?} and timeout {:?}", interval, timeout);
    }
}

impl Default for ServiceRegistry {
    fn default() -> Self {
        Self::new()
    }
}
