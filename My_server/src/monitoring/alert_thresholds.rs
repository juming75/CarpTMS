use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::Arc,
    time::Duration,
};
use tokio::sync::RwLock;
use tracing::{error, info, warn};

/// Alert threshold configuration for different system components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertThresholdConfig {
    pub system_name: String,
    pub thresholds: HashMap<String, ComponentThreshold>,
    pub escalation_rules: Vec<EscalationRule>,
    pub notification_channels: Vec<String>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentThreshold {
    pub component_name: String,
    pub metric_name: String,
    pub warning_threshold: f64,
    pub critical_threshold: f64,
    pub evaluation_window: Duration,
    pub cooldown_period: Duration,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationRule {
    pub rule_name: String,
    pub trigger_condition: String,
    pub escalation_delay: Duration,
    pub notification_channels: Vec<String>,
    pub enabled: bool,
}

/// Default alert thresholds for common system components
impl Default for AlertThresholdConfig {
    fn default() -> Self {
        let mut thresholds = HashMap::new();
        
        // CPU thresholds
        thresholds.insert("cpu_usage".to_string(), ComponentThreshold {
            component_name: "CPU".to_string(),
            metric_name: "usage_percent".to_string(),
            warning_threshold: 70.0,
            critical_threshold: 85.0,
            evaluation_window: Duration::from_secs(300), // 5 minutes
            cooldown_period: Duration::from_secs(600),   // 10 minutes
            enabled: true,
        });

        // Memory thresholds
        thresholds.insert("memory_usage".to_string(), ComponentThreshold {
            component_name: "Memory".to_string(),
            metric_name: "usage_percent".to_string(),
            warning_threshold: 80.0,
            critical_threshold: 90.0,
            evaluation_window: Duration::from_secs(300),
            cooldown_period: Duration::from_secs(600),
            enabled: true,
        });

        // Disk thresholds
        thresholds.insert("disk_usage".to_string(), ComponentThreshold {
            component_name: "Disk".to_string(),
            metric_name: "usage_percent".to_string(),
            warning_threshold: 75.0,
            critical_threshold: 85.0,
            evaluation_window: Duration::from_secs(600),  // 10 minutes
            cooldown_period: Duration::from_secs(1800),   // 30 minutes
            enabled: true,
        });

        // Response time thresholds
        thresholds.insert("response_time".to_string(), ComponentThreshold {
            component_name: "API".to_string(),
            metric_name: "response_time_ms".to_string(),
            warning_threshold: 1000.0,  // 1 second
            critical_threshold: 3000.0, // 3 seconds
            evaluation_window: Duration::from_secs(180),   // 3 minutes
            cooldown_period: Duration::from_secs(300),   // 5 minutes
            enabled: true,
        });

        // Error rate thresholds
        thresholds.insert("error_rate".to_string(), ComponentThreshold {
            component_name: "API".to_string(),
            metric_name: "error_rate_percent".to_string(),
            warning_threshold: 5.0,   // 5%
            critical_threshold: 10.0, // 10%
            evaluation_window: Duration::from_secs(300),   // 5 minutes
            cooldown_period: Duration::from_secs(600),     // 10 minutes
            enabled: true,
        });

        // Database connection thresholds
        thresholds.insert("db_connections".to_string(), ComponentThreshold {
            component_name: "Database".to_string(),
            metric_name: "active_connections_percent".to_string(),
            warning_threshold: 70.0,
            critical_threshold: 85.0,
            evaluation_window: Duration::from_secs(120),   // 2 minutes
            cooldown_period: Duration::from_secs(300),     // 5 minutes
            enabled: true,
        });

        // Queue size thresholds
        thresholds.insert("queue_size".to_string(), ComponentThreshold {
            component_name: "Queue".to_string(),
            metric_name: "queue_size".to_string(),
            warning_threshold: 1000.0,
            critical_threshold: 5000.0,
            evaluation_window: Duration::from_secs(60),    // 1 minute
            cooldown_period: Duration::from_secs(180),     // 3 minutes
            enabled: true,
        });

        // Business metric thresholds
        thresholds.insert("failed_logins".to_string(), ComponentThreshold {
            component_name: "Security".to_string(),
            metric_name: "failed_logins_per_minute".to_string(),
            warning_threshold: 10.0,
            critical_threshold: 30.0,
            evaluation_window: Duration::from_secs(60),    // 1 minute
            cooldown_period: Duration::from_secs(300),     // 5 minutes
            enabled: true,
        });

        Self {
            system_name: "CarpTMS".to_string(),
            thresholds,
            escalation_rules: vec![
                EscalationRule {
                    rule_name: "Critical_System_Down".to_string(),
                    trigger_condition: "cpu_usage > 90 OR memory_usage > 95".to_string(),
                    escalation_delay: Duration::from_secs(300), // 5 minutes
                    notification_channels: vec!["email".to_string(), "sms".to_string()],
                    enabled: true,
                },
                EscalationRule {
                    rule_name: "Sustained_High_Load".to_string(),
                    trigger_condition: "cpu_usage > 80 AND memory_usage > 85".to_string(),
                    escalation_delay: Duration::from_secs(900), // 15 minutes
                    notification_channels: vec!["email".to_string()],
                    enabled: true,
                },
            ],
            notification_channels: vec!["email".to_string(), "slack".to_string()],
            enabled: true,
        }
    }
}

pub struct AlertThresholdManager {
    config: AlertThresholdConfig,
    current_metrics: Arc<RwLock<HashMap<String, f64>>>,
    alert_history: Arc<RwLock<Vec<AlertEvent>>>,
    last_alert_times: Arc<RwLock<HashMap<String, DateTime<Utc>>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertEvent {
    pub id: String,
    pub component: String,
    pub metric: String,
    pub severity: AlertSeverity,
    pub threshold_value: f64,
    pub current_value: f64,
    pub message: String,
    pub timestamp: DateTime<Utc>,
    pub acknowledged: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlertSeverity {
    Warning,
    Critical,
}

impl AlertThresholdManager {
    pub fn new(config: AlertThresholdConfig) -> Self {
        Self {
            config,
            current_metrics: Arc::new(RwLock::new(HashMap::new())),
            alert_history: Arc::new(RwLock::new(Vec::new())),
            last_alert_times: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Update a metric value and check if it triggers any thresholds
    pub async fn update_metric(&self, component: &str, metric: &str, value: f64) -> Vec<AlertEvent> {
        let metric_key = format!("{}_{}", component, metric);
        
        // Update current metrics
        {
            let mut metrics = self.current_metrics.write().await;
            metrics.insert(metric_key.clone(), value);
        }

        // Check thresholds
        self.check_thresholds(&metric_key, component, metric, value).await
    }

    /// Check all thresholds for a given metric
    async fn check_thresholds(
        &self,
        metric_key: &str,
        component: &str,
        metric: &str,
        value: f64,
    ) -> Vec<AlertEvent> {
        let mut triggered_alerts = Vec::new();

        if let Some(threshold) = self.config.thresholds.get(metric_key) {
            if !threshold.enabled {
                return triggered_alerts;
            }

            // Check cooldown period
            if !self.is_cooldown_passed(metric_key).await {
                return triggered_alerts;
            }

            let severity = if value >= threshold.critical_threshold {
                AlertSeverity::Critical
            } else if value >= threshold.warning_threshold {
                AlertSeverity::Warning
            } else {
                return triggered_alerts; // No threshold exceeded
            };

            let alert = AlertEvent {
                id: uuid::Uuid::new_v4().to_string(),
                component: component.to_string(),
                metric: metric.to_string(),
                severity,
                threshold_value: if severity == AlertSeverity::Critical {
                    threshold.critical_threshold
                } else {
                    threshold.warning_threshold
                },
                current_value: value,
                message: format!(
                    "{} {} is {} (threshold: {})",
                    component, metric, value,
                    if severity == AlertSeverity::Critical {
                        threshold.critical_threshold
                    } else {
                        threshold.warning_threshold
                    }
                ),
                timestamp: Utc::now(),
                acknowledged: false,
            };

            triggered_alerts.push(alert.clone());

            // Record alert and update last alert time
            {
                let mut history = self.alert_history.write().await;
                history.push(alert);

                // Keep only last 1000 alerts
                if history.len() > 1000 {
                    history.drain(0..history.len() - 1000);
                }
            }

            {
                let mut last_times = self.last_alert_times.write().await;
                last_times.insert(metric_key.to_string(), Utc::now());
            }

            info!(
                "Alert triggered: {} {} = {} ({} threshold: {})",
                component, metric, value,
                if severity == AlertSeverity::Critical { "critical" } else { "warning" },
                if severity == AlertSeverity::Critical {
                    threshold.critical_threshold
                } else {
                    threshold.warning_threshold
                }
            );
        }

        triggered_alerts
    }

    /// Check if cooldown period has passed for a metric
    async fn is_cooldown_passed(&self, metric_key: &str) -> bool {
        let last_times = self.last_alert_times.read().await;
        
        if let Some(last_time) = last_times.get(metric_key) {
            let cooldown = self.config.thresholds.get(metric_key)
                .map(|t| t.cooldown_period)
                .unwrap_or(Duration::from_secs(300));
            
            Utc::now() - *last_time >= chrono::Duration::from_std(cooldown).expect("valid Duration")
        } else {
            true // No previous alert
        }
    }

    /// Get current metrics
    pub async fn get_current_metrics(&self) -> HashMap<String, f64> {
        self.current_metrics.read().await.clone()
    }

    /// Get alert history
    pub async fn get_alert_history(&self, limit: usize) -> Vec<AlertEvent> {
        let history = self.alert_history.read().await;
        history.iter().rev().take(limit).cloned().collect()
    }

    /// Acknowledge an alert
    pub async fn acknowledge_alert(&self, alert_id: &str) -> bool {
        let mut history = self.alert_history.write().await;
        
        if let Some(alert) = history.iter_mut().find(|a| a.id == alert_id) {
            alert.acknowledged = true;
            info!("Alert {} acknowledged", alert_id);
            true
        } else {
            false
        }
    }

    /// Get threshold configuration
    pub fn get_threshold_config(&self) -> &AlertThresholdConfig {
        &self.config
    }

    /// Update threshold configuration
    pub async fn update_threshold(&mut self, metric_key: &str, threshold: ComponentThreshold) {
        self.config.thresholds.insert(metric_key.to_string(), threshold);
        info!("Updated threshold for metric: {}", metric_key);
    }

    /// Disable a threshold
    pub async fn disable_threshold(&mut self, metric_key: &str) {
        if let Some(threshold) = self.config.thresholds.get_mut(metric_key) {
            threshold.enabled = false;
            info!("Disabled threshold for metric: {}", metric_key);
        }
    }

    /// Enable a threshold
    pub async fn enable_threshold(&mut self, metric_key: &str) {
        if let Some(threshold) = self.config.thresholds.get_mut(metric_key) {
            threshold.enabled = true;
            info!("Enabled threshold for metric: {}", metric_key);
        }
    }

    /// Check escalation rules
    pub async fn check_escalation_rules(&self) -> Vec<String> {
        let mut triggered_escalations = Vec::new();
        let current_metrics = self.current_metrics.read().await;

        for rule in &self.config.escalation_rules {
            if !rule.enabled {
                continue;
            }

            // Simple condition evaluation (in real implementation, use a proper expression evaluator)
            if self.evaluate_escalation_condition(&rule.trigger_condition, &current_metrics) {
                triggered_escalations.push(rule.rule_name.clone());
                
                warn!(
                    "Escalation rule '{}' triggered: {}",
                    rule.rule_name, rule.trigger_condition
                );
            }
        }

        triggered_escalations
    }

    /// Simple condition evaluation (placeholder for real expression evaluator)
    fn evaluate_escalation_condition(&self, condition: &str, metrics: &HashMap<String, f64>) -> bool {
        // Simple AND/OR logic for demonstration
        if condition.contains(" OR ") {
            let parts: Vec<&str> = condition.split(" OR ").collect();
            parts.iter().any(|part| self.evaluate_simple_condition(part.trim(), metrics))
        } else if condition.contains(" AND ") {
            let parts: Vec<&str> = condition.split(" AND ").collect();
            parts.iter().all(|part| self.evaluate_simple_condition(part.trim(), metrics))
        } else {
            self.evaluate_simple_condition(condition, metrics)
        }
    }

    /// Evaluate simple metric comparison
    fn evaluate_simple_condition(&self, condition: &str, metrics: &HashMap<String, f64>) -> bool {
        // Parse simple conditions like "cpu_usage > 80"
        let parts: Vec<&str> = condition.split_whitespace().collect();
        if parts.len() != 3 {
            return false;
        }

        let metric_name = parts[0];
        let operator = parts[1];
        let threshold: f64 = parts[2].parse().unwrap_or(0.0);

        if let Some(&value) = metrics.get(metric_name) {
            match operator {
                ">" => value > threshold,
                ">=" => value >= threshold,
                "<" => value < threshold,
                "<=" => value <= threshold,
                "==" => (value - threshold).abs() < f64::EPSILON,
                "!=" => (value - threshold).abs() > f64::EPSILON,
                _ => false,
            }
        } else {
            false
        }
    }
}

/// Create default alert threshold manager
pub fn create_alert_threshold_manager() -> AlertThresholdManager {
    AlertThresholdManager::new(AlertThresholdConfig::default())
}

/// Create alert threshold manager with custom configuration
pub fn create_alert_threshold_manager_with_config(config: AlertThresholdConfig) -> AlertThresholdManager {
    AlertThresholdManager::new(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_threshold_manager_creation() {
        let manager = create_alert_threshold_manager();
        let metrics = manager.get_current_metrics().await;
        
        assert!(metrics.is_empty());
    }

    #[tokio::test]
    async fn test_cpu_threshold_triggering() {
        let manager = create_alert_threshold_manager();
        
        // Test warning threshold
        let alerts = manager.update_metric("CPU", "usage_percent", 75.0).await;
        assert_eq!(alerts.len(), 1);
        assert_eq!(alerts[0].severity, AlertSeverity::Warning);
        
        // Test critical threshold
        let alerts = manager.update_metric("CPU", "usage_percent", 90.0).await;
        assert_eq!(alerts.len(), 1);
        assert_eq!(alerts[0].severity, AlertSeverity::Critical);
        
        // Test normal value (no alert)
        let alerts = manager.update_metric("CPU", "usage_percent", 50.0).await;
        assert_eq!(alerts.len(), 0);
    }

    #[tokio::test]
    async fn test_memory_threshold_triggering() {
        let manager = create_alert_threshold_manager();
        
        // Test warning threshold
        let alerts = manager.update_metric("Memory", "usage_percent", 85.0).await;
        assert_eq!(alerts.len(), 1);
        assert_eq!(alerts[0].severity, AlertSeverity::Warning);
        
        // Test critical threshold
        let alerts = manager.update_metric("Memory", "usage_percent", 95.0).await;
        assert_eq!(alerts.len(), 1);
        assert_eq!(alerts[0].severity, AlertSeverity::Critical);
    }

    #[tokio::test]
    async fn test_response_time_threshold_triggering() {
        let manager = create_alert_threshold_manager();
        
        // Test warning threshold (1 second)
        let alerts = manager.update_metric("API", "response_time_ms", 1500.0).await;
        assert_eq!(alerts.len(), 1);
        assert_eq!(alerts[0].severity, AlertSeverity::Warning);
        
        // Test critical threshold (3 seconds)
        let alerts = manager.update_metric("API", "response_time_ms", 3500.0).await;
        assert_eq!(alerts.len(), 1);
        assert_eq!(alerts[0].severity, AlertSeverity::Critical);
    }

    #[tokio::test]
    async fn test_error_rate_threshold_triggering() {
        let manager = create_alert_threshold_manager();
        
        // Test warning threshold (5%)
        let alerts = manager.update_metric("API", "error_rate_percent", 7.0).await;
        assert_eq!(alerts.len(), 1);
        assert_eq!(alerts[0].severity, AlertSeverity::Warning);
        
        // Test critical threshold (10%)
        let alerts = manager.update_metric("API", "error_rate_percent", 12.0).await;
        assert_eq!(alerts.len(), 1);
        assert_eq!(alerts[0].severity, AlertSeverity::Critical);
    }

    #[tokio::test]
    async fn test_cooldown_period() {
        let manager = create_alert_threshold_manager();
        
        // First alert should trigger
        let alerts = manager.update_metric("CPU", "usage_percent", 90.0).await;
        assert_eq!(alerts.len(), 1);
        
        // Second alert immediately should not trigger (cooldown)
        let alerts = manager.update_metric("CPU", "usage_percent", 90.0).await;
        assert_eq!(alerts.len(), 0);
    }

    #[tokio::test]
    async fn test_alert_acknowledgment() {
        let manager = create_alert_threshold_manager();
        
        // Generate an alert
        let alerts = manager.update_metric("CPU", "usage_percent", 90.0).await;
        assert_eq!(alerts.len(), 1);
        
        let alert_id = &alerts[0].id;
        
        // Acknowledge the alert
        let acknowledged = manager.acknowledge_alert(alert_id).await;
        assert!(acknowledged);
        
        // Verify acknowledgment in history
        let history = manager.get_alert_history(10).await;
        let acknowledged_alert = history.iter().find(|a| a.id == *alert_id);
        assert!(acknowledged_alert.is_some());
        assert!(acknowledged_alert.unwrap().acknowledged);
    }

    #[tokio::test]
    async fn test_escalation_rule_evaluation() {
        let mut manager = create_alert_threshold_manager();
        
        // Set metrics that would trigger escalation
        manager.update_metric("CPU", "usage_percent", 95.0).await;
        manager.update_metric("Memory", "usage_percent", 90.0).await;
        
        // Check escalation rules
        let escalations = manager.check_escalation_rules().await;
        assert!(!escalations.is_empty());
        assert!(escalations.contains(&"Critical_System_Down".to_string()));
    }

    #[tokio::test]
    async fn test_threshold_enable_disable() {
        let mut manager = create_alert_threshold_manager();
        
        // Disable CPU threshold
        manager.disable_threshold("cpu_usage").await;
        
        // This should not trigger an alert
        let alerts = manager.update_metric("CPU", "usage_percent", 95.0).await;
        assert_eq!(alerts.len(), 0);
        
        // Enable it back
        manager.enable_threshold("cpu_usage").await;
        
        // Now it should trigger
        let alerts = manager.update_metric("CPU", "usage_percent", 95.0).await;
        assert_eq!(alerts.len(), 1);
    }
}




