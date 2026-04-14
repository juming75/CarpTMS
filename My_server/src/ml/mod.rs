//! Machine Learning Integration Module
//!
//! Provides machine learning capabilities for:
//! - Anomaly detection in vehicle behavior
//! - Predictive maintenance
//! - Route optimization
//! - Fuel consumption prediction
//! - Driver behavior analysis

use async_trait::async_trait;
use ndarray::{Array1, Array2, ArrayView1};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;

/// Machine learning error types
#[derive(Error, Debug)]
pub enum MLError {
    #[error("Model not found: {0}")]
    ModelNotFound(String),

    #[error("Model training error: {0}")]
    TrainingError(String),

    #[error("Prediction error: {0}")]
    PredictionError(String),

    #[error("Data preprocessing error: {0}")]
    PreprocessingError(String),

    #[error("Model serialization error: {0}")]
    SerializationError(String),

    #[error("Feature extraction error: {0}")]
    FeatureExtractionError(String),
}

/// ML model types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum MLModelType {
    AnomalyDetection,
    PredictiveMaintenance,
    RouteOptimization,
    FuelConsumption,
    DriverBehavior,
    TrafficPrediction,
    WeatherImpact,
}

impl std::fmt::Display for MLModelType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MLModelType::AnomalyDetection => write!(f, "Anomaly Detection"),
            MLModelType::PredictiveMaintenance => write!(f, "Predictive Maintenance"),
            MLModelType::RouteOptimization => write!(f, "Route Optimization"),
            MLModelType::FuelConsumption => write!(f, "Fuel Consumption"),
            MLModelType::DriverBehavior => write!(f, "Driver Behavior"),
            MLModelType::TrafficPrediction => write!(f, "Traffic Prediction"),
            MLModelType::WeatherImpact => write!(f, "Weather Impact"),
        }
    }
}

/// Vehicle telemetry data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VehicleTelemetry {
    pub vehicle_id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub latitude: f64,
    pub longitude: f64,
    pub speed: f32,
    pub engine_rpm: f32,
    pub fuel_level: f32,
    pub coolant_temp: f32,
    pub oil_pressure: f32,
    pub battery_voltage: f32,
    pub odometer: f64,
    pub engine_hours: f32,
    pub gps_accuracy: f32,
    pub altitude: f32,
    pub heading: f32,
    pub satellites: u8,
}

/// Driver behavior data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriverBehavior {
    pub driver_id: String,
    pub vehicle_id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub harsh_acceleration: f32,
    pub harsh_braking: f32,
    pub harsh_cornering: f32,
    pub speeding_duration: f32,
    pub idle_time: f32,
    pub fuel_efficiency: f32,
    pub seatbelt_usage: f32,
    pub phone_usage: f32,
}

/// Route data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteData {
    pub route_id: String,
    pub origin: (f64, f64),
    pub destination: (f64, f64),
    pub waypoints: Vec<(f64, f64)>,
    pub distance: f32,
    pub estimated_time: f32,
    pub traffic_conditions: Vec<TrafficCondition>,
    pub weather_conditions: Vec<WeatherCondition>,
}

/// Traffic condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficCondition {
    pub location: (f64, f64),
    pub severity: f32, // 0.0 to 1.0
    pub speed_limit: f32,
    pub current_speed: f32,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Weather condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherCondition {
    pub location: (f64, f64),
    pub temperature: f32,
    pub humidity: f32,
    pub wind_speed: f32,
    pub wind_direction: f32,
    pub precipitation: f32,
    pub visibility: f32,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Anomaly detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyResult {
    pub is_anomaly: bool,
    pub anomaly_score: f32,
    pub anomaly_type: String,
    pub confidence: f32,
    pub explanation: String,
    pub recommendations: Vec<String>,
}

/// Predictive maintenance prediction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaintenancePrediction {
    pub component: String,
    pub failure_probability: f32,
    pub estimated_time_to_failure: f32, // in hours
    pub maintenance_urgency: MaintenanceUrgency,
    pub estimated_cost: f32,
    pub recommended_action: String,
}

/// Maintenance urgency levels
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum MaintenanceUrgency {
    Low,
    Medium,
    High,
    Critical,
}

/// ML model trait
#[async_trait]
pub trait MLModel: Send + Sync {
    /// Train the model with data
    async fn train(
        &mut self,
        training_data: &Array2<f32>,
        labels: &Array1<f32>,
    ) -> Result<(), MLError>;

    /// Make predictions
    async fn predict(&self, features: &Array2<f32>) -> Result<Array1<f32>, MLError>;

    /// Get model type
    fn model_type(&self) -> MLModelType;

    /// Get model version
    fn version(&self) -> &str;

    /// Save model to bytes
    async fn save(&self) -> Result<Vec<u8>, MLError>;

    /// Load model from bytes
    async fn load(&mut self, data: &[u8]) -> Result<(), MLError>;
}

/// Simple anomaly detection model using Isolation Forest-like approach
pub struct AnomalyDetectionModel {
    version: String,
    threshold: f32,
    normal_samples: Vec<Array1<f32>>,
}

impl AnomalyDetectionModel {
    pub fn new(threshold: f32) -> Self {
        Self {
            version: "1.0.0".to_string(),
            threshold,
            normal_samples: Vec::new(),
        }
    }

    /// Calculate Euclidean distance between two vectors
    fn euclidean_distance(&self, a: &ArrayView1<f32>, b: &ArrayView1<f32>) -> f32 {
        a.iter()
            .zip(b.iter())
            .map(|(x, y)| (x - y).powi(2))
            .sum::<f32>()
            .sqrt()
    }

    /// Calculate anomaly score based on distance to normal samples
    fn calculate_anomaly_score(&self, sample: &ArrayView1<f32>) -> f32 {
        if self.normal_samples.is_empty() {
            return 0.0;
        }

        let distances: Vec<f32> = self
            .normal_samples
            .iter()
            .map(|normal| self.euclidean_distance(sample, &normal.view()))
            .collect();

        // Calculate average distance to k nearest neighbors
        let k = 5.min(distances.len());
        let mut sorted_distances = distances;
        sorted_distances.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        sorted_distances.iter().take(k).sum::<f32>() / k as f32
    }
}

#[async_trait]
impl MLModel for AnomalyDetectionModel {
    async fn train(
        &mut self,
        training_data: &Array2<f32>,
        _labels: &Array1<f32>,
    ) -> Result<(), MLError> {
        // Store normal samples for comparison
        self.normal_samples.clear();

        for row in training_data.rows() {
            self.normal_samples.push(row.to_owned());
        }

        Ok(())
    }

    async fn predict(&self, features: &Array2<f32>) -> Result<Array1<f32>, MLError> {
        let mut predictions = Vec::new();

        for sample in features.rows() {
            let anomaly_score = self.calculate_anomaly_score(&sample);
            predictions.push(if anomaly_score > self.threshold {
                1.0
            } else {
                0.0
            });
        }

        Ok(Array1::from(predictions))
    }

    fn model_type(&self) -> MLModelType {
        MLModelType::AnomalyDetection
    }

    fn version(&self) -> &str {
        &self.version
    }

    async fn save(&self) -> Result<Vec<u8>, MLError> {
        // Simple serialization - in production, use proper serialization
        let serialized = format!("{},{}", self.version, self.threshold);
        Ok(serialized.into_bytes())
    }

    async fn load(&mut self, data: &[u8]) -> Result<(), MLError> {
        let serialized = String::from_utf8(data.to_vec())
            .map_err(|e| MLError::SerializationError(e.to_string()))?;

        let parts: Vec<&str> = serialized.split(',').collect();
        if parts.len() != 2 {
            return Err(MLError::SerializationError("Invalid format".to_string()));
        }

        self.version = parts[0].to_string();
        self.threshold = parts[1]
            .parse::<f32>()
            .map_err(|e| MLError::SerializationError(e.to_string()))?;

        Ok(())
    }
}

/// Predictive maintenance model using simple regression
#[allow(dead_code)]
pub struct PredictiveMaintenanceModel {
    version: String,
    component_thresholds: HashMap<String, f32>,
    component_weights: HashMap<String, f32>,
}

impl Default for PredictiveMaintenanceModel {
    fn default() -> Self {
        Self::new()
    }
}

impl PredictiveMaintenanceModel {
    pub fn new() -> Self {
        let mut component_thresholds = HashMap::new();
        let mut component_weights = HashMap::new();

        // Default thresholds and weights for common components
        component_thresholds.insert("engine".to_string(), 0.7);
        component_thresholds.insert("transmission".to_string(), 0.6);
        component_thresholds.insert("brakes".to_string(), 0.8);
        component_thresholds.insert("tires".to_string(), 0.5);
        component_thresholds.insert("battery".to_string(), 0.4);

        component_weights.insert("engine".to_string(), 0.3);
        component_weights.insert("transmission".to_string(), 0.25);
        component_weights.insert("brakes".to_string(), 0.2);
        component_weights.insert("tires".to_string(), 0.15);
        component_weights.insert("battery".to_string(), 0.1);

        Self {
            version: "1.0.0".to_string(),
            component_thresholds,
            component_weights,
        }
    }

    /// Calculate component health based on sensor data
    fn calculate_component_health(&self, telemetry: &VehicleTelemetry) -> HashMap<String, f32> {
        let mut health_scores = HashMap::new();

        // Engine health based on RPM, temperature, and oil pressure
        let engine_health = if telemetry.engine_rpm > 4000.0
            || telemetry.coolant_temp > 100.0
            || telemetry.oil_pressure < 20.0
        {
            0.3
        } else if telemetry.engine_rpm > 3000.0
            || telemetry.coolant_temp > 90.0
            || telemetry.oil_pressure < 30.0
        {
            0.6
        } else {
            0.9
        };
        health_scores.insert("engine".to_string(), engine_health);

        // Battery health based on voltage
        let battery_health = if telemetry.battery_voltage < 11.5 {
            0.2
        } else if telemetry.battery_voltage < 12.0 {
            0.5
        } else if telemetry.battery_voltage > 14.5 {
            0.3
        } else {
            0.9
        };
        health_scores.insert("battery".to_string(), battery_health);

        // Tire health based on speed and time (simplified)
        let tire_health = if telemetry.speed > 80.0 { 0.7 } else { 0.85 };
        health_scores.insert("tires".to_string(), tire_health);

        // Brake health based on speed patterns (simplified)
        let brake_health = if telemetry.speed < 5.0 && telemetry.engine_rpm > 800.0 {
            0.6 // Possible brake drag
        } else {
            0.9
        };
        health_scores.insert("brakes".to_string(), brake_health);

        health_scores
    }
}

#[async_trait]
impl MLModel for PredictiveMaintenanceModel {
    async fn train(
        &mut self,
        training_data: &Array2<f32>,
        labels: &Array1<f32>,
    ) -> Result<(), MLError> {
        // Simple training - adjust thresholds based on training data
        // In a real implementation, this would involve proper statistical analysis

        if training_data.nrows() == 0 {
            return Err(MLError::TrainingError("Empty training data".to_string()));
        }

        // Update thresholds based on training data (simplified)
        for (_component, threshold) in self.component_thresholds.iter_mut() {
            *threshold = 0.5 + (labels.mean().unwrap_or(0.5) * 0.3);
        }

        Ok(())
    }

    async fn predict(&self, features: &Array2<f32>) -> Result<Array1<f32>, MLError> {
        // Convert features to telemetry data for component health calculation
        let mut predictions = Vec::new();

        for sample in features.rows() {
            if sample.len() < 10 {
                return Err(MLError::PredictionError(
                    "Insufficient features".to_string(),
                ));
            }

            // Create a simplified telemetry object from features
            let telemetry = VehicleTelemetry {
                vehicle_id: "predict".to_string(),
                timestamp: chrono::Utc::now(),
                latitude: 0.0,
                longitude: 0.0,
                speed: sample[0],
                engine_rpm: sample[1],
                fuel_level: sample[2],
                coolant_temp: sample[3],
                oil_pressure: sample[4],
                battery_voltage: sample[5],
                odometer: sample[6] as f64,
                engine_hours: sample[7],
                gps_accuracy: sample[8],
                altitude: sample[9],
                heading: 0.0,
                satellites: 0,
            };

            let health_scores = self.calculate_component_health(&telemetry);
            let overall_health = health_scores.values().sum::<f32>() / health_scores.len() as f32;

            predictions.push(overall_health);
        }

        Ok(Array1::from(predictions))
    }

    fn model_type(&self) -> MLModelType {
        MLModelType::PredictiveMaintenance
    }

    fn version(&self) -> &str {
        &self.version
    }

    async fn save(&self) -> Result<Vec<u8>, MLError> {
        // Simple serialization
        let serialized = format!("{},{}", self.version, self.component_thresholds.len());
        Ok(serialized.into_bytes())
    }

    async fn load(&mut self, _data: &[u8]) -> Result<(), MLError> {
        // Simple deserialization - in production, use proper serialization
        // Keep default thresholds for now
        Ok(())
    }
}

/// ML model manager that coordinates all ML models
pub struct MLModelManager {
    models: Arc<RwLock<HashMap<MLModelType, Arc<dyn MLModel>>>>,
    anomaly_model: Arc<AnomalyDetectionModel>,
    maintenance_model: Arc<PredictiveMaintenanceModel>,
}

impl MLModelManager {
    /// Create a new ML model manager
    pub async fn new() -> Result<Self, MLError> {
        let anomaly_model = Arc::new(AnomalyDetectionModel::new(0.5));
        let maintenance_model = Arc::new(PredictiveMaintenanceModel::new());

        let mut models: HashMap<MLModelType, Arc<dyn MLModel>> = HashMap::new();
        models.insert(MLModelType::AnomalyDetection, anomaly_model.clone());
        models.insert(
            MLModelType::PredictiveMaintenance,
            maintenance_model.clone(),
        );

        Ok(Self {
            models: Arc::new(RwLock::new(models)),
            anomaly_model,
            maintenance_model,
        })
    }

    /// Detect anomalies in vehicle telemetry
    pub async fn detect_anomalies(
        &self,
        telemetry: &VehicleTelemetry,
    ) -> Result<AnomalyResult, MLError> {
        // Convert telemetry to feature vector
        let features = Array2::from_shape_vec(
            (1, 10),
            vec![
                telemetry.speed,
                telemetry.engine_rpm,
                telemetry.fuel_level,
                telemetry.coolant_temp,
                telemetry.oil_pressure,
                telemetry.battery_voltage,
                telemetry.odometer as f32,
                telemetry.engine_hours,
                telemetry.gps_accuracy,
                telemetry.altitude,
            ],
        )
        .map_err(|e| MLError::PreprocessingError(e.to_string()))?;

        let predictions = self.anomaly_model.predict(&features).await?;
        let anomaly_score = predictions[0];

        Ok(AnomalyResult {
            is_anomaly: anomaly_score > 0.5,
            anomaly_score,
            anomaly_type: if anomaly_score > 0.7 {
                "High Risk".to_string()
            } else {
                "Medium Risk".to_string()
            },
            confidence: anomaly_score,
            explanation: "Anomaly detected based on sensor data patterns".to_string(),
            recommendations: vec![
                "Check vehicle sensors".to_string(),
                "Review recent driving patterns".to_string(),
                "Schedule diagnostic check".to_string(),
            ],
        })
    }

    /// Predict maintenance needs
    pub async fn predict_maintenance(
        &self,
        telemetry: &VehicleTelemetry,
    ) -> Result<Vec<MaintenancePrediction>, MLError> {
        // Convert telemetry to feature vector
        let features = Array2::from_shape_vec(
            (1, 10),
            vec![
                telemetry.speed,
                telemetry.engine_rpm,
                telemetry.fuel_level,
                telemetry.coolant_temp,
                telemetry.oil_pressure,
                telemetry.battery_voltage,
                telemetry.odometer as f32,
                telemetry.engine_hours,
                telemetry.gps_accuracy,
                telemetry.altitude,
            ],
        )
        .map_err(|e| MLError::PreprocessingError(e.to_string()))?;

        let predictions = self.maintenance_model.predict(&features).await?;
        let overall_health = predictions[0];

        // Generate maintenance predictions based on overall health
        let mut predictions = Vec::new();

        if overall_health < 0.3 {
            predictions.push(MaintenancePrediction {
                component: "Engine".to_string(),
                failure_probability: 0.8,
                estimated_time_to_failure: 24.0,
                maintenance_urgency: MaintenanceUrgency::Critical,
                estimated_cost: 2500.0,
                recommended_action: "Immediate inspection required".to_string(),
            });
        } else if overall_health < 0.5 {
            predictions.push(MaintenancePrediction {
                component: "Battery".to_string(),
                failure_probability: 0.6,
                estimated_time_to_failure: 72.0,
                maintenance_urgency: MaintenanceUrgency::High,
                estimated_cost: 150.0,
                recommended_action: "Replace battery soon".to_string(),
            });
        }

        Ok(predictions)
    }

    /// Analyze driver behavior
    pub async fn analyze_driver_behavior(
        &self,
        behavior: &DriverBehavior,
    ) -> Result<HashMap<String, f32>, MLError> {
        let mut scores = HashMap::new();

        // Calculate driver safety score
        let safety_score = 100.0
            - (behavior.harsh_acceleration * 20.0
                + behavior.harsh_braking * 20.0
                + behavior.harsh_cornering * 15.0
                + behavior.speeding_duration * 10.0
                + behavior.phone_usage * 30.0);
        scores.insert("safety_score".to_string(), safety_score.max(0.0));

        // Calculate efficiency score
        let efficiency_score =
            50.0 + (behavior.fuel_efficiency * 30.0 + (1.0 - behavior.idle_time / 100.0) * 20.0);
        scores.insert("efficiency_score".to_string(), efficiency_score.min(100.0));

        // Calculate overall score
        let overall_score = (safety_score + efficiency_score) / 2.0;
        scores.insert("overall_score".to_string(), overall_score);

        Ok(scores)
    }

    /// Add a new model
    pub async fn add_model(
        &self,
        model_type: MLModelType,
        model: Arc<dyn MLModel>,
    ) -> Result<(), MLError> {
        let mut models = self.models.write().await;
        models.insert(model_type, model);
        Ok(())
    }

    /// Remove a model
    pub async fn remove_model(&self, model_type: &MLModelType) -> Result<(), MLError> {
        let mut models = self.models.write().await;
        models.remove(model_type);
        Ok(())
    }

    /// Get list of available models
    pub async fn get_available_models(&self) -> Vec<MLModelType> {
        let models = self.models.read().await;
        models.keys().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_anomaly_detection() {
        let mut model = AnomalyDetectionModel::new(0.5);

        // Training data
        let training_data = Array2::from_shape_vec(
            (3, 10),
            vec![
                60.0, 2000.0, 50.0, 85.0, 40.0, 13.5, 100000.0, 2000.0, 5.0, 100.0, 55.0, 1800.0,
                45.0, 82.0, 38.0, 13.2, 95000.0, 1900.0, 4.5, 95.0, 65.0, 2200.0, 55.0, 88.0, 42.0,
                13.8, 105000.0, 2100.0, 5.5, 105.0,
            ],
        )
        .unwrap();

        let labels = Array1::from(vec![0.0, 0.0, 0.0]); // Normal samples
        model.train(&training_data, &labels).await.unwrap();

        // Test normal data
        let normal_data = Array2::from_shape_vec(
            (1, 10),
            vec![
                58.0, 1900.0, 48.0, 84.0, 39.0, 13.4, 98000.0, 1950.0, 4.8, 98.0,
            ],
        )
        .unwrap();

        let predictions = model.predict(&normal_data).await.unwrap();
        assert_eq!(predictions.len(), 1);
        assert_eq!(predictions[0], 0.0);
    }

    #[tokio::test]
    async fn test_predictive_maintenance() {
        let model_manager = MLModelManager::new().await.unwrap();

        let telemetry = VehicleTelemetry {
            vehicle_id: "test_vehicle".to_string(),
            timestamp: chrono::Utc::now(),
            latitude: 40.7128,
            longitude: -74.0060,
            speed: 65.0,
            engine_rpm: 2500.0,
            fuel_level: 45.0,
            coolant_temp: 95.0,
            oil_pressure: 35.0,
            battery_voltage: 12.5,
            odometer: 100000.0,
            engine_hours: 2000.0,
            gps_accuracy: 5.0,
            altitude: 100.0,
            heading: 180.0,
            satellites: 8,
        };

        let predictions = model_manager.predict_maintenance(&telemetry).await.unwrap();
        assert!(!predictions.is_empty());
    }

    #[tokio::test]
    async fn test_driver_behavior_analysis() {
        let model_manager = MLModelManager::new().await.unwrap();

        let behavior = DriverBehavior {
            driver_id: "test_driver".to_string(),
            vehicle_id: "test_vehicle".to_string(),
            timestamp: chrono::Utc::now(),
            harsh_acceleration: 0.1,
            harsh_braking: 0.2,
            harsh_cornering: 0.05,
            speeding_duration: 0.15,
            idle_time: 0.25,
            fuel_efficiency: 0.8,
            seatbelt_usage: 0.95,
            phone_usage: 0.05,
        };

        let scores = model_manager
            .analyze_driver_behavior(&behavior)
            .await
            .unwrap();
        assert!(scores.contains_key("safety_score"));
        assert!(scores.contains_key("efficiency_score"));
        assert!(scores.contains_key("overall_score"));
    }
}

// 导出子模块
pub mod deployment;
pub mod truck_scale;
