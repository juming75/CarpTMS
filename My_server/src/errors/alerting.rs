use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, SystemTime},
};
use tokio::sync::{mpsc, RwLock};
use tracing::{error, info, warn};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertConfig {
    pub enabled: bool,
    pub severity_threshold: AlertSeverity,
    pub rate_limit: Duration,
    pub max_alerts_per_interval: u32,
    pub aggregation_window: Duration,
    pub escalation_delay: Duration,
    pub enable_auto_recovery: bool,
}

impl Default for AlertConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            severity_threshold: AlertSeverity::Warning,
            rate_limit: Duration::from_secs(60),
            max_alerts_per_interval: 10,
            aggregation_window: Duration::from_secs(300),
            escalation_delay: Duration::from_secs(1800),
            enable_auto_recovery: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info = 0,
    Warning = 1,
    Error = 2,
    Critical = 3,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRule {
    pub id: String,
    pub name: String,
    pub description: String,
    pub condition: AlertCondition,
    pub severity: AlertSeverity,
    pub notification_channels: Vec<String>,
    pub enabled: bool,
    pub cooldown: Duration,
    pub tags: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertCondition {
    ErrorRate {
        threshold: f64,
        window: Duration,
    },
    ResponseTime {
        threshold: Duration,
        window: Duration,
    },
    StatusCode {
        codes: Vec<u16>,
        threshold: u32,
        window: Duration,
    },
    Custom {
        metric: String,
        operator: String,
        value: f64,
        window: Duration,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertEvent {
    pub id: String,
    pub rule_id: String,
    pub severity: AlertSeverity,
    pub title: String,
    pub description: String,
    pub timestamp: DateTime<Utc>,
    pub context: HashMap<String, String>,
    pub acknowledged: bool,
    pub acknowledged_by: Option<String>,
    pub acknowledged_at: Option<DateTime<Utc>>,
    pub resolved: bool,
    pub resolved_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationChannel {
    pub id: String,
    pub name: String,
    pub channel_type: NotificationType,
    pub config: HashMap<String, String>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationType {
    Email,
    Webhook,
    Slack,
    Teams,
    Sms,
    Log,
}

pub struct ErrorAlertManager {
    config: AlertConfig,
    rules: Arc<RwLock<HashMap<String, AlertRule>>>,
    channels: Arc<RwLock<HashMap<String, NotificationChannel>>>,
    active_alerts: Arc<RwLock<HashMap<String, AlertEvent>>>,
    alert_history: Arc<RwLock<Vec<AlertEvent>>>,
    metrics: Arc<RwLock<AlertMetrics>>,
    event_tx: mpsc::Sender<AlertEvent>,
    event_rx: Option<mpsc::Receiver<AlertEvent>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertMetrics {
    pub total_alerts: u64,
    pub active_alerts: u64,
    pub acknowledged_alerts: u64,
    pub resolved_alerts: u64,
    pub alerts_by_severity: HashMap<AlertSeverity, u64>,
    pub alerts_by_rule: HashMap<String, u64>,
    pub notification_success_rate: f64,
    pub average_resolution_time: Duration,
}

impl ErrorAlertManager {
    pub fn new(config: AlertConfig) -> Self {
        let (event_tx, event_rx) = mpsc::channel(1000);
        let initial_metrics = AlertMetrics {
            total_alerts: 0,
            active_alerts: 0,
            acknowledged_alerts: 0,
            resolved_alerts: 0,
            alerts_by_severity: HashMap::new(),
            alerts_by_rule: HashMap::new(),
            notification_success_rate: 0.0,
            average_resolution_time: Duration::from_secs(0),
        };

        Self {
            config,
            rules: Arc::new(RwLock::new(HashMap::new())),
            channels: Arc::new(RwLock::new(HashMap::new())),
            active_alerts: Arc::new(RwLock::new(HashMap::new())),
            alert_history: Arc::new(RwLock::new(Vec::new())),
            metrics: Arc::new(RwLock::new(initial_metrics)),
            event_tx,
            event_rx: Some(event_rx),
        }
    }

    pub async fn start(&mut self) -> Result<(), AlertError> {
        if !self.config.enabled {
            info!("Error alerting is disabled");
            return Ok(());
        }

        let mut event_rx = self.event_rx.take().ok_or(AlertError::AlreadyStarted)?;
        let rules = self.rules.clone();
        let channels = self.channels.clone();
        let active_alerts = self.active_alerts.clone();
        let alert_history = self.alert_history.clone();
        let metrics = self.metrics.clone();
        let config = self.config.clone();

        tokio::spawn(async move {
            while let Some(event) = event_rx.recv().await {
                Self::process_alert_event(
                    event,
                    &rules,
                    &channels,
                    &active_alerts,
                    &alert_history,
                    &metrics,
                    &config,
                )
                .await;
            }
        });

        info!("Error alert manager started");
        Ok(())
    }

    pub async fn add_rule(&self, rule: AlertRule) -> Result<(), AlertError> {
        let rule_id = rule.id.clone();
        let mut rules = self.rules.write().await;
        rules.insert(rule.id.clone(), rule);
        info!("Added alert rule: {}", rule_id);
        Ok(())
    }

    pub async fn remove_rule(&self, rule_id: &str) -> Result<(), AlertError> {
        let mut rules = self.rules.write().await;
        rules.remove(rule_id);
        info!("Removed alert rule: {}", rule_id);
        Ok(())
    }

    pub async fn add_notification_channel(
        &self,
        channel: NotificationChannel,
    ) -> Result<(), AlertError> {
        let channel_id = channel.id.clone();
        let mut channels = self.channels.write().await;
        channels.insert(channel.id.clone(), channel);
        info!("Added notification channel: {}", channel_id);
        Ok(())
    }

    pub async fn remove_notification_channel(&self, channel_id: &str) -> Result<(), AlertError> {
        let mut channels = self.channels.write().await;
        channels.remove(channel_id);
        info!("Removed notification channel: {}", channel_id);
        Ok(())
    }

    pub async fn trigger_alert(
        &self,
        rule_id: &str,
        title: &str,
        description: &str,
        context: HashMap<String, String>,
    ) -> Result<(), AlertError> {
        let rules = self.rules.read().await;

        if let Some(rule) = rules.get(rule_id) {
            if !rule.enabled {
                return Ok(());
            }

            let event = AlertEvent {
                id: Uuid::new_v4().to_string(),
                rule_id: rule_id.to_string(),
                severity: rule.severity,
                title: title.to_string(),
                description: description.to_string(),
                timestamp: Utc::now(),
                context,
                acknowledged: false,
                acknowledged_by: None,
                acknowledged_at: None,
                resolved: false,
                resolved_at: None,
            };

            self.event_tx
                .send(event)
                .await
                .map_err(|_| AlertError::EventQueueFull)?;
            Ok(())
        } else {
            Err(AlertError::RuleNotFound(rule_id.to_string()))
        }
    }

    pub async fn acknowledge_alert(
        &self,
        alert_id: &str,
        acknowledged_by: &str,
    ) -> Result<(), AlertError> {
        let mut active_alerts = self.active_alerts.write().await;

        if let Some(alert) = active_alerts.get_mut(alert_id) {
            alert.acknowledged = true;
            alert.acknowledged_by = Some(acknowledged_by.to_string());
            alert.acknowledged_at = Some(Utc::now());

            info!("Alert {} acknowledged by {}", alert_id, acknowledged_by);
            Ok(())
        } else {
            Err(AlertError::AlertNotFound(alert_id.to_string()))
        }
    }

    pub async fn resolve_alert(&self, alert_id: &str) -> Result<(), AlertError> {
        let mut active_alerts = self.active_alerts.write().await;

        if let Some(alert) = active_alerts.remove(alert_id) {
            let mut alert_history = self.alert_history.write().await;
            let mut resolved_alert = alert;
            resolved_alert.resolved = true;
            resolved_alert.resolved_at = Some(Utc::now());
            alert_history.push(resolved_alert.clone());

            // Update metrics
            let mut metrics = self.metrics.write().await;
            metrics.active_alerts = active_alerts.len() as u64;
            metrics.resolved_alerts += 1;

            if let Some(resolution_time) = resolved_alert.resolved_at {
                let duration = resolution_time - resolved_alert.timestamp;
                let current_avg = metrics.average_resolution_time.as_secs_f64();
                let total_resolved = metrics.resolved_alerts;
                metrics.average_resolution_time = Duration::from_secs_f64(
                    (current_avg * (total_resolved - 1) as f64 + duration.num_seconds() as f64)
                        / total_resolved as f64,
                );
            }

            info!("Alert {} resolved", alert_id);
            Ok(())
        } else {
            Err(AlertError::AlertNotFound(alert_id.to_string()))
        }
    }

    pub async fn get_active_alerts(&self) -> Vec<AlertEvent> {
        let active_alerts = self.active_alerts.read().await;
        active_alerts.values().cloned().collect()
    }

    pub async fn get_alert_history(&self, limit: usize) -> Vec<AlertEvent> {
        let alert_history = self.alert_history.read().await;
        alert_history.iter().rev().take(limit).cloned().collect()
    }

    pub async fn get_metrics(&self) -> AlertMetrics {
        self.metrics.read().await.clone()
    }

    pub async fn check_alert_conditions(
        &self,
        metrics: HashMap<String, f64>,
    ) -> Result<Vec<String>, AlertError> {
        let rules = self.rules.read().await;
        let mut triggered_rules = Vec::new();

        for (rule_id, rule) in rules.iter() {
            if !rule.enabled {
                continue;
            }

            if let Some(triggered_reason) = self.evaluate_rule(rule, &metrics).await {
                triggered_rules.push(rule_id.clone());

                // Trigger alert
                self.trigger_alert(
                    rule_id,
                    &format!("Alert triggered: {}", rule.name),
                    &triggered_reason,
                    HashMap::new(),
                )
                .await?;
            }
        }

        Ok(triggered_rules)
    }

    // Private methods
    async fn process_alert_event(
        event: AlertEvent,
        rules: &Arc<RwLock<HashMap<String, AlertRule>>>,
        channels: &Arc<RwLock<HashMap<String, NotificationChannel>>>,
        active_alerts: &Arc<RwLock<HashMap<String, AlertEvent>>>,
        alert_history: &Arc<RwLock<Vec<AlertEvent>>>,
        metrics: &Arc<RwLock<AlertMetrics>>,
        config: &AlertConfig,
    ) {
        // Check if we should process this alert based on severity threshold
        if event.severity < config.severity_threshold {
            return;
        }

        // Check rate limiting
        if !Self::check_rate_limit(event.rule_id.clone(), config).await {
            warn!("Rate limit exceeded for rule: {}", event.rule_id);
            return;
        }

        // Add to active alerts
        let mut active_alerts_guard = active_alerts.write().await;
        active_alerts_guard.insert(event.id.clone(), event.clone());

        // Add to history
        let mut history = alert_history.write().await;
        history.push(event.clone());

        // Update metrics
        let mut metrics_guard = metrics.write().await;
        metrics_guard.total_alerts += 1;
        metrics_guard.active_alerts = active_alerts_guard.len() as u64;
        *metrics_guard
            .alerts_by_severity
            .entry(event.severity)
            .or_insert(0) += 1;
        *metrics_guard
            .alerts_by_rule
            .entry(event.rule_id.clone())
            .or_insert(0) += 1;

        // Send notifications
        if let Some(rule) = rules.read().await.get(&event.rule_id) {
            for channel_id in &rule.notification_channels {
                if let Some(channel) = channels.read().await.get(channel_id) {
                    if channel.enabled {
                        if let Err(e) = Self::send_notification(&event, channel).await {
                            error!(
                                "Failed to send notification to channel {}: {}",
                                channel_id, e
                            );
                        }
                    }
                }
            }
        }

        info!(
            "Alert processed: {} - Severity: {:?} - Rule: {}",
            event.title, event.severity, event.rule_id
        );
    }

    async fn check_rate_limit(rule_id: String, config: &AlertConfig) -> bool {
        // Simple rate limiting implementation
        // In a real system, you'd use Redis or similar for distributed rate limiting
        use once_cell::sync::Lazy;
        use std::sync::Mutex;

        static RATE_LIMITER: Lazy<Mutex<std::collections::HashMap<String, (u32, SystemTime)>>> = 
            Lazy::new(|| Mutex::new(std::collections::HashMap::new()));

        let now = SystemTime::now();
        if let Ok(mut rate_limiter) = RATE_LIMITER.lock() {
            if let Some((count, last_reset)) = rate_limiter.get(&rule_id) {
                let count = *count;
                let last_reset = *last_reset;

                if now.duration_since(last_reset).ok() < Some(config.rate_limit) {
                    if count >= config.max_alerts_per_interval {
                        return false;
                    }
                    rate_limiter.insert(rule_id, (count + 1, last_reset));
                } else {
                    rate_limiter.insert(rule_id, (1, now));
                }
            } else {
                rate_limiter.insert(rule_id, (1, now));
            }
        }

        true
    }

    async fn evaluate_rule(
        &self,
        rule: &AlertRule,
        metrics: &HashMap<String, f64>,
    ) -> Option<String> {
        match &rule.condition {
            AlertCondition::ErrorRate { threshold, window } => {
                if let Some(error_rate) = metrics.get("error_rate") {
                    if *error_rate > *threshold {
                        return Some(format!(
                            "Error rate {} exceeds threshold {} over {:?}",
                            error_rate, threshold, window
                        ));
                    }
                }
            }
            AlertCondition::ResponseTime { threshold, window } => {
                if let Some(response_time) = metrics.get("response_time") {
                    if *response_time > threshold.as_millis() as f64 {
                        return Some(format!(
                            "Response time {}ms exceeds threshold {}ms over {:?}",
                            response_time,
                            threshold.as_millis(),
                            window
                        ));
                    }
                }
            }
            AlertCondition::StatusCode {
                codes: _,
                threshold,
                window,
            } => {
                if let Some(status_count) = metrics.get("status_5xx") {
                    if *status_count > *threshold as f64 {
                        return Some(format!(
                            "Status code errors {} exceed threshold {} over {:?}",
                            status_count, threshold, window
                        ));
                    }
                }
            }
            AlertCondition::Custom {
                metric,
                operator,
                value,
                window,
            } => {
                if let Some(metric_value) = metrics.get(metric) {
                    let triggered = match operator.as_str() {
                        ">" => *metric_value > *value,
                        ">=" => *metric_value >= *value,
                        "<" => *metric_value < *value,
                        "<=" => *metric_value <= *value,
                        "==" => (*metric_value - value).abs() < f64::EPSILON,
                        "!=" => (*metric_value - value).abs() > f64::EPSILON,
                        _ => false,
                    };

                    if triggered {
                        return Some(format!(
                            "Metric {} {} {} over {:?}",
                            metric, operator, value, window
                        ));
                    }
                }
            }
        }
        None
    }

    async fn send_notification(
        event: &AlertEvent,
        channel: &NotificationChannel,
    ) -> Result<(), AlertError> {
        match channel.channel_type {
            NotificationType::Email => {
                info!(
                    "Sending email notification for alert {} to channel {}",
                    event.id, channel.id
                );
                // Implement email sending logic
                Ok(())
            }
            NotificationType::Webhook => {
                info!(
                    "Sending webhook notification for alert {} to channel {}",
                    event.id, channel.id
                );
                // Implement webhook sending logic
                Ok(())
            }
            NotificationType::Slack => {
                info!(
                    "Sending Slack notification for alert {} to channel {}",
                    event.id, channel.id
                );
                // Implement Slack notification logic
                Ok(())
            }
            NotificationType::Teams => {
                info!(
                    "Sending Teams notification for alert {} to channel {}",
                    event.id, channel.id
                );
                // Implement Teams notification logic
                Ok(())
            }
            NotificationType::Sms => {
                info!(
                    "Sending SMS notification for alert {} to channel {}",
                    event.id, channel.id
                );
                // Implement SMS sending logic
                Ok(())
            }
            NotificationType::Log => {
                match event.severity {
                    AlertSeverity::Critical => error!("CRITICAL ALERT: {}", event.description),
                    AlertSeverity::Error => error!("ERROR ALERT: {}", event.description),
                    AlertSeverity::Warning => warn!("WARNING ALERT: {}", event.description),
                    AlertSeverity::Info => info!("INFO ALERT: {}", event.description),
                }
                Ok(())
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AlertError {
    #[error("Rule not found: {0}")]
    RuleNotFound(String),

    #[error("Alert not found: {0}")]
    AlertNotFound(String),

    #[error("Channel not found: {0}")]
    ChannelNotFound(String),

    #[error("Event queue is full")]
    EventQueueFull,

    #[error("Alert manager already started")]
    AlreadyStarted,

    #[error("Notification failed: {0}")]
    NotificationFailed(String),

    #[error("Configuration error: {0}")]
    ConfigurationError(String),
}

// Convenience functions
pub async fn create_error_alert_manager() -> ErrorAlertManager {
    let config = AlertConfig::default();
    ErrorAlertManager::new(config)
}

pub async fn create_error_alert_manager_with_config(config: AlertConfig) -> ErrorAlertManager {
    ErrorAlertManager::new(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_alert_manager_creation() {
        let manager = create_error_alert_manager().await;
        let metrics = manager.get_metrics().await;

        assert_eq!(metrics.total_alerts, 0);
        assert_eq!(metrics.active_alerts, 0);
    }

    #[tokio::test]
    async fn test_alert_rule_management() {
        let manager = create_error_alert_manager().await;

        let rule = AlertRule {
            id: "test_rule".to_string(),
            name: "Test Rule".to_string(),
            description: "Test alert rule".to_string(),
            condition: AlertCondition::ErrorRate {
                threshold: 0.1,
                window: Duration::from_secs(300),
            },
            severity: AlertSeverity::Warning,
            notification_channels: vec!["log_channel".to_string()],
            enabled: true,
            cooldown: Duration::from_secs(300),
            tags: HashMap::new(),
        };

        manager.add_rule(rule).await.unwrap();

        let channel = NotificationChannel {
            id: "log_channel".to_string(),
            name: "Log Channel".to_string(),
            channel_type: NotificationType::Log,
            config: HashMap::new(),
            enabled: true,
        };

        manager.add_notification_channel(channel).await.unwrap();

        // Test triggering alert
        let mut metrics = HashMap::new();
        metrics.insert("error_rate".to_string(), 0.15);

        let triggered = manager.check_alert_conditions(metrics).await.unwrap();
        assert!(!triggered.is_empty());
    }

    #[tokio::test]
    async fn test_alert_acknowledgment_and_resolution() {
        let mut manager = create_error_alert_manager().await;
        manager.start().await.unwrap();

        // Add a rule and channel
        let rule = AlertRule {
            id: "test_rule".to_string(),
            name: "Test Rule".to_string(),
            description: "Test alert rule".to_string(),
            condition: AlertCondition::ErrorRate {
                threshold: 0.05,
                window: Duration::from_secs(300),
            },
            severity: AlertSeverity::Warning,
            notification_channels: vec!["log_channel".to_string()],
            enabled: true,
            cooldown: Duration::from_secs(300),
            tags: HashMap::new(),
        };

        manager.add_rule(rule).await.unwrap();

        let channel = NotificationChannel {
            id: "log_channel".to_string(),
            name: "Log Channel".to_string(),
            channel_type: NotificationType::Log,
            config: HashMap::new(),
            enabled: true,
        };

        manager.add_notification_channel(channel).await.unwrap();

        // Trigger an alert
        let mut metrics = HashMap::new();
        metrics.insert("error_rate".to_string(), 0.1);

        manager.check_alert_conditions(metrics).await.unwrap();

        // Wait a bit for processing
        tokio::time::sleep(Duration::from_millis(100)).await;

        let active_alerts = manager.get_active_alerts().await;
        assert!(!active_alerts.is_empty());

        let alert_id = &active_alerts[0].id;

        // Acknowledge the alert
        manager
            .acknowledge_alert(alert_id, "test_user")
            .await
            .unwrap();

        // Resolve the alert
        manager.resolve_alert(alert_id).await.unwrap();

        let updated_active_alerts = manager.get_active_alerts().await;
        assert!(updated_active_alerts.is_empty());

        let metrics = manager.get_metrics().await;
        assert_eq!(metrics.resolved_alerts, 1);
    }
}





