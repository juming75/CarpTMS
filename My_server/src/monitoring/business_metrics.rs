use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::Arc,
    time::Duration,
};
use tokio::sync::{mpsc, RwLock};
use tracing::{error, info, warn};

/// Business metrics monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessMetricsConfig {
    pub system_name: String,
    pub metrics: HashMap<String, BusinessMetricConfig>,
    pub aggregation_interval: Duration,
    pub retention_period: Duration,
    pub export_format: ExportFormat,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessMetricConfig {
    pub name: String,
    pub metric_type: BusinessMetricType,
    pub aggregation_type: AggregationType,
    pub unit: String,
    pub description: String,
    pub tags: Vec<String>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum BusinessMetricType {
    Counter,
    Gauge,
    Histogram,
    Rate,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum AggregationType {
    Sum,
    Average,
    Max,
    Min,
    Count,
    Percentile(f64),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ExportFormat {
    Json,
    Prometheus,
    InfluxDB,
    Custom,
}

/// Default business metrics for CarpTMS system
impl Default for BusinessMetricsConfig {
    fn default() -> Self {
        let mut metrics = HashMap::new();

        // User-related metrics
        metrics.insert("user_registrations".to_string(), BusinessMetricConfig {
            name: "user_registrations".to_string(),
            metric_type: BusinessMetricType::Counter,
            aggregation_type: AggregationType::Sum,
            unit: "users".to_string(),
            description: "Total number of user registrations".to_string(),
            tags: vec!["source".to_string(), "plan_type".to_string()],
            enabled: true,
        });

        metrics.insert("active_users".to_string(), BusinessMetricConfig {
            name: "active_users".to_string(),
            metric_type: BusinessMetricType::Gauge,
            aggregation_type: AggregationType::Average,
            unit: "users".to_string(),
            description: "Number of active users in the system".to_string(),
            tags: vec!["time_period".to_string()],
            enabled: true,
        });

        // Transaction-related metrics
        metrics.insert("orders_created".to_string(), BusinessMetricConfig {
            name: "orders_created".to_string(),
            metric_type: BusinessMetricType::Counter,
            aggregation_type: AggregationType::Sum,
            unit: "orders".to_string(),
            description: "Total number of orders created".to_string(),
            tags: vec!["order_type".to_string(), "status".to_string()],
            enabled: true,
        });

        metrics.insert("order_value".to_string(), BusinessMetricConfig {
            name: "order_value".to_string(),
            metric_type: BusinessMetricType::Histogram,
            aggregation_type: AggregationType::Average,
            unit: "USD".to_string(),
            description: "Order value distribution".to_string(),
            tags: vec!["currency".to_string(), "order_type".to_string()],
            enabled: true,
        });

        // Revenue metrics
        metrics.insert("revenue".to_string(), BusinessMetricConfig {
            name: "revenue".to_string(),
            metric_type: BusinessMetricType::Counter,
            aggregation_type: AggregationType::Sum,
            unit: "USD".to_string(),
            description: "Total revenue generated".to_string(),
            tags: vec!["currency".to_string(), "product_category".to_string()],
            enabled: true,
        });

        metrics.insert("revenue_per_user".to_string(), BusinessMetricConfig {
            name: "revenue_per_user".to_string(),
            metric_type: BusinessMetricType::Gauge,
            aggregation_type: AggregationType::Average,
            unit: "USD".to_string(),
            description: "Average revenue per user".to_string(),
            tags: vec!["user_segment".to_string()],
            enabled: true,
        });

        // API usage metrics
        metrics.insert("api_requests".to_string(), BusinessMetricConfig {
            name: "api_requests".to_string(),
            metric_type: BusinessMetricType::Counter,
            aggregation_type: AggregationType::Sum,
            unit: "requests".to_string(),
            description: "Total API requests".to_string(),
            tags: vec!["endpoint".to_string(), "method".to_string(), "status".to_string()],
            enabled: true,
        });

        metrics.insert("api_response_time".to_string(), BusinessMetricConfig {
            name: "api_response_time".to_string(),
            metric_type: BusinessMetricType::Histogram,
            aggregation_type: AggregationType::Percentile(95.0),
            unit: "milliseconds".to_string(),
            description: "API response time (95th percentile)".to_string(),
            tags: vec!["endpoint".to_string()],
            enabled: true,
        });

        // Error metrics
        metrics.insert("api_errors".to_string(), BusinessMetricConfig {
            name: "api_errors".to_string(),
            metric_type: BusinessMetricType::Counter,
            aggregation_type: AggregationType::Sum,
            unit: "errors".to_string(),
            description: "Total API errors".to_string(),
            tags: vec!["endpoint".to_string(), "error_type".to_string()],
            enabled: true,
        });

        // System health metrics
        metrics.insert("system_uptime".to_string(), BusinessMetricConfig {
            name: "system_uptime".to_string(),
            metric_type: BusinessMetricType::Gauge,
            aggregation_type: AggregationType::Average,
            unit: "percent".to_string(),
            description: "System uptime percentage".to_string(),
            tags: vec!["component".to_string()],
            enabled: true,
        });

        // Customer satisfaction metrics
        metrics.insert("customer_satisfaction".to_string(), BusinessMetricConfig {
            name: "customer_satisfaction".to_string(),
            metric_type: BusinessMetricType::Gauge,
            aggregation_type: AggregationType::Average,
            unit: "score".to_string(),
            description: "Customer satisfaction score (1-10)".to_string(),
            tags: vec!["survey_type".to_string()],
            enabled: true,
        });

        Self {
            system_name: "CarpTMS".to_string(),
            metrics,
            aggregation_interval: Duration::from_secs(300), // 5 minutes
            retention_period: Duration::from_secs(86400 * 30), // 30 days
            export_format: ExportFormat::Json,
            enabled: true,
        }
    }
}

pub struct BusinessMetricsMonitor {
    config: BusinessMetricsConfig,
    metrics_data: Arc<RwLock<HashMap<String, Vec<MetricPoint>>>>,
    aggregations: Arc<RwLock<HashMap<String, AggregationResult>>>,
    event_tx: mpsc::Sender<MetricEvent>,
    event_rx: Option<mpsc::Receiver<MetricEvent>>,
}

#[derive(Debug, Clone)]
pub struct MetricPoint {
    pub value: f64,
    pub timestamp: DateTime<Utc>,
    pub tags: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricEvent {
    pub metric_name: String,
    pub value: f64,
    pub timestamp: DateTime<Utc>,
    pub tags: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregationResult {
    pub metric_name: String,
    pub aggregated_value: f64,
    pub aggregation_type: AggregationType,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub data_points: usize,
    pub tags: HashMap<String, String>,
}

impl BusinessMetricsMonitor {
    pub fn new(config: BusinessMetricsConfig) -> Self {
        let (event_tx, event_rx) = mpsc::channel(1000);
        
        Self {
            config,
            metrics_data: Arc::new(RwLock::new(HashMap::new())),
            aggregations: Arc::new(RwLock::new(HashMap::new())),
            event_tx,
            event_rx: Some(event_rx),
        }
    }

    /// Start the metrics monitoring service
    pub async fn start(&mut self) -> Result<(), BusinessMetricsError> {
        if !self.config.enabled {
            info!("Business metrics monitoring is disabled");
            return Ok(());
        }

        let mut event_rx = self.event_rx.take().ok_or(BusinessMetricsError::AlreadyStarted)?;
        let config = self.config.clone();
        let metrics_data = self.metrics_data.clone();
        let aggregations = self.aggregations.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(config.aggregation_interval);
            
            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        // Perform aggregation
                        if let Err(e) = Self::perform_aggregation(
                            &config,
                            &metrics_data,
                            &aggregations,
                        ).await {
                            error!("Failed to perform aggregation: {}", e);
                        }
                        
                        // Clean up old data
                        if let Err(e) = Self::cleanup_old_data(
                            &config,
                            &metrics_data,
                        ).await {
                            error!("Failed to cleanup old data: {}", e);
                        }
                    }
                    Some(event) = event_rx.recv() => {
                        // Store metric event
                        if let Err(e) = Self::store_metric_event(
                            &config,
                            &metrics_data,
                            event,
                        ).await {
                            error!("Failed to store metric event: {}", e);
                        }
                    }
                    else => {
                        break;
                    }
                }
            }
        });

        info!("Business metrics monitor started");
        Ok(())
    }

    /// Record a business metric
    pub async fn record_metric(
        &self,
        metric_name: &str,
        value: f64,
        tags: HashMap<String, String>,
    ) -> Result<(), BusinessMetricsError> {
        if !self.config.enabled {
            return Ok(());
        }

        if !self.config.metrics.contains_key(metric_name) {
            return Err(BusinessMetricsError::UnknownMetric(metric_name.to_string()));
        }

        let event = MetricEvent {
            metric_name: metric_name.to_string(),
            value,
            timestamp: Utc::now(),
            tags,
        };

        self.event_tx.send(event).await
            .map_err(|_| BusinessMetricsError::EventQueueFull)?;

        Ok(())
    }

    /// Get current aggregation results
    pub async fn get_aggregations(&self) -> HashMap<String, AggregationResult> {
        self.aggregations.read().await.clone()
    }

    /// Get metric data for a specific metric
    pub async fn get_metric_data(&self, metric_name: &str) -> Vec<MetricPoint> {
        let data = self.metrics_data.read().await;
        data.get(metric_name).cloned().unwrap_or_default()
    }

    /// Get metric configuration
    pub fn get_metric_config(&self, metric_name: &str) -> Option<&BusinessMetricConfig> {
        self.config.metrics.get(metric_name)
    }

    /// Export metrics in specified format
    pub async fn export_metrics(&self) -> Result<String, BusinessMetricsError> {
        let aggregations = self.aggregations.read().await;
        
        match self.config.export_format {
            ExportFormat::Json => {
                serde_json::to_string_pretty(&*aggregations)
                    .map_err(|e| BusinessMetricsError::SerializationError(e.to_string()))
            }
            ExportFormat::Prometheus => {
                Self::format_prometheus_metrics(&aggregations)
            }
            ExportFormat::InfluxDB => {
                Self::format_influxdb_metrics(&aggregations)
            }
            ExportFormat::Custom => {
                Err(BusinessMetricsError::CustomFormatNotImplemented)
            }
        }
    }

    /// Get system health summary
    pub async fn get_health_summary(&self) -> HealthSummary {
        let aggregations = self.aggregations.read().await;
        let mut summary = HealthSummary::default();

        for (metric_name, aggregation) in aggregations.iter() {
            match metric_name.as_str() {
                "system_uptime" => summary.uptime_percentage = aggregation.aggregated_value,
                "api_errors" => summary.total_errors = aggregation.aggregated_value as u64,
                "api_requests" => summary.total_requests = aggregation.aggregated_value as u64,
                "revenue" => summary.total_revenue = aggregation.aggregated_value,
                "active_users" => summary.active_users = aggregation.aggregated_value as u64,
                _ => {}
            }
        }

        // Calculate error rate if possible
        if summary.total_requests > 0 {
            summary.error_rate = (summary.total_errors as f64 / summary.total_requests as f64) * 100.0;
        }

        summary.timestamp = Utc::now();
        summary
    }

    // Private methods
    async fn store_metric_event(
        config: &BusinessMetricsConfig,
        metrics_data: &Arc<RwLock<HashMap<String, Vec<MetricPoint>>>>,
        event: MetricEvent,
    ) -> Result<(), BusinessMetricsError> {
        let mut data = metrics_data.write().await;
        
        let point = MetricPoint {
            value: event.value,
            timestamp: event.timestamp,
            tags: event.tags,
        };

        data.entry(event.metric_name).or_insert_with(Vec::new).push(point);

        Ok(())
    }

    async fn perform_aggregation(
        config: &BusinessMetricsConfig,
        metrics_data: &Arc<RwLock<HashMap<String, Vec<MetricPoint>>>>,
        aggregations: &Arc<RwLock<HashMap<String, AggregationResult>>>,
    ) -> Result<(), BusinessMetricsError> {
        let mut data = metrics_data.write().await;
        let mut aggregations_guard = aggregations.write().await;

        let now = Utc::now();
        let start_time = now - chrono::Duration::from_std(config.aggregation_interval)
            .expect("aggregation interval out of range");

        for (metric_name, metric_config) in &config.metrics {
            if !metric_config.enabled {
                continue;
            }

            if let Some(points) = data.get_mut(metric_name) {
                // Filter points within the aggregation window
                let window_points: Vec<&MetricPoint> = points
                    .iter()
                    .filter(|p| p.timestamp >= start_time && p.timestamp <= now)
                    .collect();

                if window_points.is_empty() {
                    continue;
                }

                // Perform aggregation based on type
                let aggregated_value = match metric_config.aggregation_type {
                    AggregationType::Sum => window_points.iter().map(|p| p.value).sum(),
                    AggregationType::Average => {
                        let sum: f64 = window_points.iter().map(|p| p.value).sum();
                        sum / window_points.len() as f64
                    }
                    AggregationType::Max => window_points.iter().map(|p| p.value).fold(f64::NEG_INFINITY, f64::max),
                    AggregationType::Min => window_points.iter().map(|p| p.value).fold(f64::INFINITY, f64::min),
                    AggregationType::Count => window_points.len() as f64,
                    AggregationType::Percentile(p) => {
                        let mut values: Vec<f64> = window_points.iter().map(|p| p.value).collect();
                        values.sort_by(|a, b| a.partial_cmp(b)
                            .expect("NaN value encountered in percentile calculation"));
                        
                        let index = ((p / 100.0) * (values.len() - 1) as f64) as usize;
                        values.get(index).copied().unwrap_or(0.0)
                    }
                };

                let result = AggregationResult {
                    metric_name: metric_name.clone(),
                    aggregated_value,
                    aggregation_type: metric_config.aggregation_type,
                    start_time,
                    end_time: now,
                    data_points: window_points.len(),
                    tags: HashMap::new(), // Could aggregate tags if needed
                };

                aggregations_guard.insert(metric_name.clone(), result);

                // Remove processed points
                points.retain(|p| p.timestamp > now);
            }
        }

        Ok(())
    }

    async fn cleanup_old_data(
        config: &BusinessMetricsConfig,
        metrics_data: &Arc<RwLock<HashMap<String, Vec<MetricPoint>>>>,
    ) -> Result<(), BusinessMetricsError> {
        let mut data = metrics_data.write().await;
        let cutoff_time = Utc::now() - chrono::Duration::from_std(config.retention_period)
            .expect("retention period out of range");

        for points in data.values_mut() {
            points.retain(|p| p.timestamp > cutoff_time);
        }

        Ok(())
    }

    fn format_prometheus_metrics(aggregations: &HashMap<String, AggregationResult>) -> Result<String, BusinessMetricsError> {
        let mut output = String::new();
        
        for (metric_name, result) in aggregations {
            let metric_type = match result.aggregation_type {
                AggregationType::Counter => "counter",
                AggregationType::Gauge => "gauge",
                AggregationType::Histogram => "histogram",
                _ => "gauge",
            };

            output.push_str(&format!("# TYPE {} {}\n", metric_name, metric_type));
            output.push_str(&format!(
                "{} {} {}\n",
                metric_name,
                result.aggregated_value,
                result.end_time.timestamp()
            ));
        }

        Ok(output)
    }

    fn format_influxdb_metrics(aggregations: &HashMap<String, AggregationResult>) -> Result<String, BusinessMetricsError> {
        let mut output = String::new();
        
        for (metric_name, result) in aggregations {
            output.push_str(&format!(
                "{},aggregation={} value={} {}\n",
                metric_name,
                format!("{:?}", result.aggregation_type).to_lowercase(),
                result.aggregated_value,
                result.end_time.timestamp_nanos()
            ));
        }

        Ok(output)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthSummary {
    pub uptime_percentage: f64,
    pub total_requests: u64,
    pub total_errors: u64,
    pub error_rate: f64,
    pub total_revenue: f64,
    pub active_users: u64,
    pub timestamp: DateTime<Utc>,
}

impl Default for HealthSummary {
    fn default() -> Self {
        Self {
            uptime_percentage: 100.0,
            total_requests: 0,
            total_errors: 0,
            error_rate: 0.0,
            total_revenue: 0.0,
            active_users: 0,
            timestamp: Utc::now(),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum BusinessMetricsError {
    #[error("Unknown metric: {0}")]
    UnknownMetric(String),
    
    #[error("Event queue is full")]
    EventQueueFull,
    
    #[error("Metrics monitoring already started")]
    AlreadyStarted,
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Custom format not implemented")]
    CustomFormatNotImplemented,
}

/// Create default business metrics monitor
pub fn create_business_metrics_monitor() -> BusinessMetricsMonitor {
    BusinessMetricsMonitor::new(BusinessMetricsConfig::default())
}

/// Create business metrics monitor with custom configuration
pub fn create_business_metrics_monitor_with_config(config: BusinessMetricsConfig) -> BusinessMetricsMonitor {
    BusinessMetricsMonitor::new(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_business_metrics_monitor_creation() {
        let monitor = create_business_metrics_monitor();
        let aggregations = monitor.get_aggregations().await;
        
        assert!(aggregations.is_empty());
    }

    #[tokio::test]
    async fn test_metric_recording() {
        let monitor = create_business_metrics_monitor();
        let mut tags = HashMap::new();
        tags.insert("source".to_string(), "web".to_string());
        
        let result = monitor.record_metric("user_registrations", 1.0, tags).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_unknown_metric() {
        let monitor = create_business_metrics_monitor();
        let tags = HashMap::new();
        
        let result = monitor.record_metric("unknown_metric", 1.0, tags).await;
        assert!(result.is_err());
        
        match result {
            Err(BusinessMetricsError::UnknownMetric(_)) => {},
            _ => {
                log::warn!("Unexpected metric error type in business_metrics");
                assert!(false, "Expected UnknownMetric error");
            },
        }
    }

    #[tokio::test]
    async fn test_aggregation_sum() {
        let mut monitor = create_business_metrics_monitor();
        monitor.start().await.unwrap();
        
        let mut tags = HashMap::new();
        tags.insert("source".to_string(), "web".to_string());
        
        // Record multiple values
        monitor.record_metric("user_registrations", 1.0, tags.clone()).await.unwrap();
        monitor.record_metric("user_registrations", 2.0, tags.clone()).await.unwrap();
        monitor.record_metric("user_registrations", 3.0, tags).await.unwrap();
        
        // Wait for aggregation
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        let aggregations = monitor.get_aggregations().await;
        let user_registrations = aggregations.get("user_registrations");
        
        assert!(user_registrations.is_some());
        assert_eq!(user_registrations.unwrap().aggregated_value, 6.0); // 1 + 2 + 3
    }

    #[tokio::test]
    async fn test_aggregation_average() {
        let mut monitor = create_business_metrics_monitor();
        monitor.start().await.unwrap();
        
        let mut tags = HashMap::new();
        tags.insert("time_period".to_string(), "daily".to_string());
        
        // Record multiple values
        monitor.record_metric("active_users", 100.0, tags.clone()).await.unwrap();
        monitor.record_metric("active_users", 150.0, tags.clone()).await.unwrap();
        monitor.record_metric("active_users", 200.0, tags).await.unwrap();
        
        // Wait for aggregation
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        let aggregations = monitor.get_aggregations().await;
        let active_users = aggregations.get("active_users");
        
        assert!(active_users.is_some());
        assert_eq!(active_users.unwrap().aggregated_value, 150.0); // (100 + 150 + 200) / 3
    }

    #[tokio::test]
    async fn test_export_json_format() {
        let mut monitor = create_business_metrics_monitor();
        monitor.start().await.unwrap();
        
        let mut tags = HashMap::new();
        tags.insert("source".to_string(), "web".to_string());
        
        monitor.record_metric("user_registrations", 5.0, tags).await.unwrap();
        
        // Wait for aggregation
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        let export_result = monitor.export_metrics().await;
        assert!(export_result.is_ok());
        
        let json_output = export_result.unwrap();
        assert!(json_output.contains("user_registrations"));
    }

    #[tokio::test]
    async fn test_health_summary() {
        let mut monitor = create_business_metrics_monitor();
        monitor.start().await.unwrap();
        
        let mut tags = HashMap::new();
        tags.insert("component".to_string(), "api".to_string());
        
        monitor.record_metric("system_uptime", 99.5, tags.clone()).await.unwrap();
        monitor.record_metric("api_requests", 1000.0, tags.clone()).await.unwrap();
        monitor.record_metric("api_errors", 10.0, tags).await.unwrap();
        
        // Wait for aggregation
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        let health_summary = monitor.get_health_summary().await;
        
        assert_eq!(health_summary.uptime_percentage, 99.5);
        assert_eq!(health_summary.total_requests, 1000);
        assert_eq!(health_summary.total_errors, 10);
        assert_eq!(health_summary.error_rate, 1.0); // 10/1000 * 100
    }
}




