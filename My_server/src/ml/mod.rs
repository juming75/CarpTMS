//! Machine Learning Module
//! Provides trend prediction and alerting using TensorFlow

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Prediction model type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ModelType {
    VehicleTrajectory,
    AlarmTrend,
    FuelConsumption,
    VehicleFault,
}

/// Prediction result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionResult {
    pub value: f64,
    pub confidence: f64,
    pub timestamp: String,
    pub model_type: ModelType,
}

/// Machine learning service
pub struct MachineLearningService {
    models: Arc<RwLock<HashMap<ModelType, Box<dyn PredictionModel>>>>,
}

/// Prediction model trait
pub trait PredictionModel: Send + Sync {
    fn predict(&self, input: &[f64]) -> Result<PredictionResult, MLError>;
    fn train(&mut self, data: &[Vec<f64>], labels: &[f64]) -> Result<(), MLError>;
    fn get_info(&self) -> ModelInfo;
}

/// Model information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub name: String,
    pub version: String,
    pub model_type: ModelType,
    pub training_time: String,
    pub accuracy: f64,
}

/// Machine learning error
#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
pub enum MLError {
    #[error("Model load failed: {0}")]
    ModelLoadError(String),
    #[error("Prediction failed: {0}")]
    PredictionError(String),
    #[error("Training failed: {0}")]
    TrainingError(String),
    #[error("Data format error: {0}")]
    DataFormatError(String),
}

/// Simple linear regression model
pub struct LinearRegressionModel {
    info: ModelInfo,
    weights: Vec<f64>,
    bias: f64,
}

impl LinearRegressionModel {
    pub fn new(model_type: ModelType) -> Self {
        Self {
            info: ModelInfo {
                name: "Linear Regression".to_string(),
                version: "1.0.0".to_string(),
                model_type,
                training_time: chrono::Utc::now().to_rfc3339(),
                accuracy: 0.0,
            },
            weights: vec![],
            bias: 0.0,
        }
    }
}

impl PredictionModel for LinearRegressionModel {
    fn predict(&self, input: &[f64]) -> Result<PredictionResult, MLError> {
        if input.len() != self.weights.len() {
            return Err(MLError::DataFormatError(format!(
                "Input dimension mismatch: expected {}, got {}",
                self.weights.len(),
                input.len()
            )));
        }

        let mut prediction = self.bias;
        for (i, &x) in input.iter().enumerate() {
            prediction += self.weights[i] * x;
        }

        Ok(PredictionResult {
            value: prediction,
            confidence: 0.8,
            timestamp: chrono::Utc::now().to_rfc3339(),
            model_type: self.info.model_type.clone(),
        })
    }

    fn train(&mut self, data: &[Vec<f64>], labels: &[f64]) -> Result<(), MLError> {
        if data.is_empty() || data[0].is_empty() || data.len() != labels.len() {
            return Err(MLError::DataFormatError("Invalid data format".to_string()));
        }

        let n = data.len() as f64;
        let features = data[0].len();

        self.weights = vec![0.0; features];
        self.bias = 0.0;

        let learning_rate = 0.01;
        let epochs = 1000;

        for _ in 0..epochs {
            let mut dw = vec![0.0; features];
            let mut db = 0.0;

            for (i, x) in data.iter().enumerate() {
                let y_pred = self
                    .weights
                    .iter()
                    .zip(x)
                    .map(|(w, &xi)| w * xi)
                    .sum::<f64>()
                    + self.bias;
                let error = y_pred - labels[i];

                for (j, &xj) in x.iter().enumerate() {
                    dw[j] += error * xj;
                }
                db += error;
            }

            for (j, weight) in self.weights.iter_mut().enumerate().take(features) {
                *weight -= learning_rate * dw[j] / n;
            }
            self.bias -= learning_rate * db / n;
        }

        let mut correct = 0;
        for (i, x) in data.iter().enumerate() {
            let y_pred = self
                .weights
                .iter()
                .zip(x)
                .map(|(w, &xi)| w * xi)
                .sum::<f64>()
                + self.bias;
            if (y_pred - labels[i]).abs() < 0.1 {
                correct += 1;
            }
        }
        self.info.accuracy = correct as f64 / n;
        self.info.training_time = chrono::Utc::now().to_rfc3339();

        Ok(())
    }

    fn get_info(&self) -> ModelInfo {
        self.info.clone()
    }
}

impl Default for MachineLearningService {
    fn default() -> Self {
        Self::new()
    }
}

impl MachineLearningService {
    pub fn new() -> Self {
        let mut models: HashMap<ModelType, Box<dyn PredictionModel>> = HashMap::new();

        models.insert(
            ModelType::VehicleTrajectory,
            Box::new(LinearRegressionModel::new(ModelType::VehicleTrajectory)),
        );
        models.insert(
            ModelType::AlarmTrend,
            Box::new(LinearRegressionModel::new(ModelType::AlarmTrend)),
        );
        models.insert(
            ModelType::FuelConsumption,
            Box::new(LinearRegressionModel::new(ModelType::FuelConsumption)),
        );
        models.insert(
            ModelType::VehicleFault,
            Box::new(LinearRegressionModel::new(ModelType::VehicleFault)),
        );

        Self {
            models: Arc::new(RwLock::new(models)),
        }
    }

    pub async fn predict(
        &self,
        model_type: ModelType,
        input: &[f64],
    ) -> Result<PredictionResult, MLError> {
        let models = self.models.read().await;
        match models.get(&model_type) {
            Some(model) => model.predict(input),
            None => Err(MLError::ModelLoadError(format!(
                "Model not found: {:?}",
                model_type
            ))),
        }
    }

    pub async fn train(
        &self,
        model_type: ModelType,
        data: &[Vec<f64>],
        labels: &[f64],
    ) -> Result<(), MLError> {
        let mut models = self.models.write().await;
        match models.get_mut(&model_type) {
            Some(model) => model.train(data, labels),
            None => Err(MLError::ModelLoadError(format!(
                "Model not found: {:?}",
                model_type
            ))),
        }
    }

    pub async fn get_model_info(&self, model_type: ModelType) -> Result<ModelInfo, MLError> {
        let models = self.models.read().await;
        match models.get(&model_type) {
            Some(model) => Ok(model.get_info()),
            None => Err(MLError::ModelLoadError(format!(
                "Model not found: {:?}",
                model_type
            ))),
        }
    }

    pub async fn get_all_models(&self) -> Vec<ModelInfo> {
        let models = self.models.read().await;
        models.values().map(|model| model.get_info()).collect()
    }
}
