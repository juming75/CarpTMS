//! Routing module
//! Handles service routing and request forwarding

use std::sync::Arc;
use tokio::sync::RwLock;
use log::{info, warn, debug};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

/// Route rule
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RouteRule {
    pub id: String,
    pub service_name: String,
    pub path_pattern: String,
    pub method: Option<String>,
    pub headers: HashMap<String, String>,
    pub priority: u32,
    pub destination: RouteDestination,
    pub timeout: Option<Duration>,
    pub retries: Option<u32>,
    pub retry_delay: Option<Duration>,
}

/// Route destination
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum RouteDestination {
    Service { service_name: String, version: Option<String> },
    URL { url: String },
    Weighted {
        destinations: Vec<(RouteDestination, u32)>, // (destination, weight)
    },
}

/// Route match result
#[derive(Clone, Debug)]
pub struct RouteMatch {
    pub rule: RouteRule,
    pub matched_path: String,
    pub parameters: HashMap<String, String>,
}

/// Service router
pub struct ServiceRouter {
    routes: Arc<RwLock<Vec<RouteRule>>>,
    route_index: Arc<RwLock<HashMap<String, Vec<RouteRule>>>>, // service name -> routes
}

impl ServiceRouter {
    /// Create a new service router
    pub fn new() -> Self {
        Self {
            routes: Arc::new(RwLock::new(Vec::new())),
            route_index: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Add a route rule
    pub async fn add_route(&self, rule: RouteRule) {
        let mut routes = self.routes.write().await;
        let mut route_index = self.route_index.write().await;

        // Add to routes
        routes.push(rule.clone());

        // Update route index
        route_index
            .entry(rule.service_name.clone())
            .or_insert(Vec::new())
            .push(rule.clone());

        // Sort routes by priority (highest first)
        routes.sort_by(|a, b| b.priority.cmp(&a.priority));

        info!("Added route rule for service {} with priority {}", rule.service_name, rule.priority);
    }

    /// Remove a route rule
    pub async fn remove_route(&self, rule_id: &str) {
        let mut routes = self.routes.write().await;
        let mut route_index = self.route_index.write().await;

        // Find and remove the rule
        if let Some(index) = routes.iter().position(|r| r.id == rule_id) {
            let rule = routes.remove(index);

            // Remove from route index
            if let Some(rules) = route_index.get_mut(&rule.service_name) {
                if let Some(rule_index) = rules.iter().position(|r| r.id == rule_id) {
                    rules.remove(rule_index);
                    if rules.is_empty() {
                        route_index.remove(&rule.service_name);
                    }
                }
            }

            info!("Removed route rule: {}", rule_id);
        }
    }

    /// Match a request to a route
    pub async fn match_route(&self, service_name: &str, path: &str, method: &str, headers: &HashMap<String, String>) -> Option<RouteMatch> {
        let routes = self.routes.read().await;

        // Filter routes for the service
        let service_routes: Vec<&RouteRule> = routes
            .iter()
            .filter(|r| r.service_name == service_name)
            .collect();

        // Try to match each route
        for rule in service_routes {
            if self.matches_rule(rule, path, method, headers) {
                // Extract parameters from path
                let parameters = self.extract_parameters(&rule.path_pattern, path);

                return Some(RouteMatch {
                    rule: rule.clone(),
                    matched_path: path.to_string(),
                    parameters,
                });
            }
        }

        None
    }

    /// Check if a request matches a route rule
    fn matches_rule(&self, rule: &RouteRule, path: &str, method: &str, headers: &HashMap<String, String>) -> bool {
        // Check method
        if let Some(rule_method) = &rule.method {
            if rule_method != method {
                return false;
            }
        }

        // Check path pattern
        if !self.matches_path(&rule.path_pattern, path) {
            return false;
        }

        // Check headers
        for (key, value) in &rule.headers {
            if let Some(header_value) = headers.get(key) {
                if header_value != value {
                    return false;
                }
            } else {
                return false;
            }
        }

        true
    }

    /// Check if a path matches a pattern
    fn matches_path(&self, pattern: &str, path: &str) -> bool {
        // Simplified path matching
        // In a real implementation, you would use a more sophisticated pattern matching
        pattern == path || pattern.ends_with("*") && path.starts_with(&pattern[..pattern.len()-1])
    }

    /// Extract parameters from path
    fn extract_parameters(&self, pattern: &str, path: &str) -> HashMap<String, String> {
        // Simplified parameter extraction
        // In a real implementation, you would use a more sophisticated parameter extraction
        HashMap::new()
    }

    /// Get all routes
    pub async fn get_all_routes(&self) -> Vec<RouteRule> {
        self.routes.read().await.clone()
    }

    /// Get routes by service name
    pub async fn get_routes_by_service(&self, service_name: &str) -> Vec<RouteRule> {
        let route_index = self.route_index.read().await;
        route_index.get(service_name).cloned().unwrap_or(Vec::new())
    }

    /// Update a route rule
    pub async fn update_route(&self, rule: RouteRule) {
        let mut routes = self.routes.write().await;
        let mut route_index = self.route_index.write().await;

        // Find and update the rule
        if let Some(index) = routes.iter().position(|r| r.id == rule.id) {
            let old_rule = routes.remove(index);
            routes.push(rule.clone());

            // Update route index
            if let Some(rules) = route_index.get_mut(&old_rule.service_name) {
                if let Some(rule_index) = rules.iter().position(|r| r.id == rule.id) {
                    rules.remove(rule_index);
                    if rules.is_empty() {
                        route_index.remove(&old_rule.service_name);
                    }
                }
            }

            route_index
                .entry(rule.service_name.clone())
                .or_insert(Vec::new())
                .push(rule.clone());

            // Sort routes by priority
            routes.sort_by(|a, b| b.priority.cmp(&a.priority));

            info!("Updated route rule: {}", rule.id);
        } else {
            // If rule doesn't exist, add it
            self.add_route(rule).await;
        }
    }

    /// Clear all routes
    pub async fn clear_routes(&self) {
        let mut routes = self.routes.write().await;
        let mut route_index = self.route_index.write().await;

        routes.clear();
        route_index.clear();

        info!("Cleared all routes");
    }
}

impl Default for ServiceRouter {
    fn default() -> Self {
        Self::new()
    }
}
