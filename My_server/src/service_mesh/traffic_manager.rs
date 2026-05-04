//! Traffic Manager module
//! Handles advanced traffic management, load balancing, and routing

use std::sync::Arc;
use tokio::sync::RwLock;
use log::{info, warn, debug};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

/// Service instance information
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServiceInstance {
    pub id: String,
    pub name: String,
    pub address: String,
    pub port: u16,
    pub health_status: String,
    pub version: String,
    pub tags: Vec<String>,
}

/// Load balancing strategy
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LoadBalancingStrategy {
    RoundRobin,
    LeastConnections,
    Random,
    Weighted,
}

/// Traffic management rules
#[derive(Clone, Debug)]
pub struct TrafficRule {
    pub service_name: String,
    pub priority: u32,
    pub condition: String, // Can be a JSON expression or simple string
    pub action: TrafficAction,
}

/// Traffic action
#[derive(Clone, Debug)]
pub enum TrafficAction {
    RouteToVersion(String),
    RouteToTag(String),
    RateLimit(u32), // requests per second
    CircuitBreak(u32), // failure threshold
    Timeout(u64), // milliseconds
}

/// Connection count for least connections strategy
#[derive(Clone, Debug)]
pub struct ConnectionCount {
    pub instance_id: String,
    pub count: u32,
    pub last_update: Instant,
}

/// Traffic manager
pub struct TrafficManager {
    services: Arc<RwLock<HashMap<String, Vec<ServiceInstance>>>>, // service name -> instances
    connection_counts: Arc<RwLock<HashMap<String, ConnectionCount>>>, // instance id -> connection count
    traffic_rules: Arc<RwLock<Vec<TrafficRule>>>,
    load_balancing_strategy: LoadBalancingStrategy,
    round_robin_state: Arc<RwLock<HashMap<String, usize>>>, // service name -> current index
}

impl TrafficManager {
    /// Create a new traffic manager
    pub fn new(strategy: LoadBalancingStrategy) -> Self {
        Self {
            services: Arc::new(RwLock::new(HashMap::new())),
            connection_counts: Arc::new(RwLock::new(HashMap::new())),
            traffic_rules: Arc::new(RwLock::new(Vec::new())),
            load_balancing_strategy: strategy,
            round_robin_state: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a service instance
    pub async fn register_service(&self, instance: ServiceInstance) {
        let mut services = self.services.write().await;
        let mut connection_counts = self.connection_counts.write().await;

        services
            .entry(instance.name.clone())
            .or_insert(Vec::new())
            .push(instance.clone());

        connection_counts.insert(instance.id.clone(), ConnectionCount {
            instance_id: instance.id.clone(),
            count: 0,
            last_update: Instant::now(),
        });

        info!("Registered service instance: {} ({}:{}) for service {}", 
              instance.id, instance.address, instance.port, instance.name);
    }

    /// Unregister a service instance
    pub async fn unregister_service(&self, service_id: &str) {
        let mut services = self.services.write().await;
        let mut connection_counts = self.connection_counts.write().await;

        // Find and remove the instance from all services
        for (service_name, instances) in services.iter_mut() {
            if let Some(index) = instances.iter().position(|i| i.id == service_id) {
                instances.remove(index);
                info!("Unregistered service instance: {} from service {}", service_id, service_name);
                break;
            }
        }

        connection_counts.remove(service_id);
    }

    /// Update service health status
    pub async fn update_service_health(&self, service_id: &str, health_status: String) {
        let mut services = self.services.write().await;

        for (_, instances) in services.iter_mut() {
            if let Some(instance) = instances.iter_mut().find(|i| i.id == service_id) {
                instance.health_status = health_status.clone();
                info!("Updated health status for service instance {}: {}", service_id, health_status);
                break;
            }
        }
    }

    /// Add a traffic rule
    pub async fn add_traffic_rule(&self, rule: TrafficRule) {
        let mut rules = self.traffic_rules.write().await;
        rules.push(rule);
        // Sort rules by priority (highest first)
        rules.sort_by(|a, b| b.priority.cmp(&a.priority));
        info!("Added traffic rule with priority {}", rule.priority);
    }

    /// Get a service instance based on load balancing strategy
    pub async fn get_service_instance(&self, service_name: &str) -> Option<ServiceInstance> {
        let services = self.services.read().await;
        let instances = services.get(service_name)?;
        
        // Filter healthy instances
        let healthy_instances: Vec<&ServiceInstance> = instances
            .iter()
            .filter(|i| i.health_status == "healthy")
            .collect();

        if healthy_instances.is_empty() {
            warn!("No healthy instances found for service {}", service_name);
            return None;
        }

        let selected_instance = match self.load_balancing_strategy {
            LoadBalancingStrategy::RoundRobin => self.select_round_robin(service_name, &healthy_instances).await,
            LoadBalancingStrategy::LeastConnections => self.select_least_connections(&healthy_instances).await,
            LoadBalancingStrategy::Random => self.select_random(&healthy_instances).await,
            LoadBalancingStrategy::Weighted => self.select_weighted(&healthy_instances).await,
        };

        selected_instance
    }

    /// Select instance using round robin strategy
    async fn select_round_robin(&self, service_name: &str, instances: &[&ServiceInstance]) -> Option<ServiceInstance> {
        let mut state = self.round_robin_state.write().await;
        let current_index = state.entry(service_name.to_string()).or_insert(0);
        
        let instance = instances[*current_index % instances.len()].clone();
        *current_index += 1;
        
        Some(instance)
    }

    /// Select instance using least connections strategy
    async fn select_least_connections(&self, instances: &[&ServiceInstance]) -> Option<ServiceInstance> {
        let connection_counts = self.connection_counts.read().await;
        
        let mut min_count = u32::MAX;
        let mut selected_instance = None;
        
        for instance in instances {
            if let Some(count) = connection_counts.get(&instance.id) {
                if count.count < min_count {
                    min_count = count.count;
                    selected_instance = Some(instance.clone());
                }
            } else {
                // If no connection count, assume it's zero
                return Some(instance.clone());
            }
        }
        
        selected_instance
    }

    /// Select instance using random strategy
    async fn select_random(&self, instances: &[&ServiceInstance]) -> Option<ServiceInstance> {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let index = rng.gen_range(0..instances.len());
        Some(instances[index].clone())
    }

    /// Select instance using weighted strategy
    async fn select_weighted(&self, instances: &[&ServiceInstance]) -> Option<ServiceInstance> {
        // Simplified weighted selection based on instance tags
        // In a real implementation, you would have explicit weights
        let instance = instances[0].clone();
        Some(instance)
    }

    /// Increment connection count for an instance
    pub async fn increment_connection(&self, instance_id: &str) {
        let mut connection_counts = self.connection_counts.write().await;
        if let Some(count) = connection_counts.get_mut(instance_id) {
            count.count += 1;
            count.last_update = Instant::now();
        }
    }

    /// Decrement connection count for an instance
    pub async fn decrement_connection(&self, instance_id: &str) {
        let mut connection_counts = self.connection_counts.write().await;
        if let Some(count) = connection_counts.get_mut(instance_id) {
            if count.count > 0 {
                count.count -= 1;
            }
            count.last_update = Instant::now();
        }
    }

    /// Get all services
    pub async fn get_all_services(&self) -> HashMap<String, Vec<ServiceInstance>> {
        self.services.read().await.clone()
    }

    /// Get service instances by name
    pub async fn get_service_instances(&self, service_name: &str) -> Option<Vec<ServiceInstance>> {
        let services = self.services.read().await;
        services.get(service_name).cloned()
    }
}

impl Default for TrafficManager {
    fn default() -> Self {
        Self::new(LoadBalancingStrategy::RoundRobin)
    }
}
