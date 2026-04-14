//! /! 模型部署与监控模块
//!
//! 实现模型的实时部署、版本管理和监控功能

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

use crate::ml::MLModel;

/// 模型部署状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModelDeploymentStatus {
    /// 部署中
    Deploying,
    /// 部署成功
    Deployed,
    /// 部署失败
    Failed(String),
    /// 已停止
    Stopped,
}

/// 模型版本信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelVersion {
    /// 版本号
    pub version: String,
    /// 模型类型
    pub model_type: String,
    /// 部署时间
    pub deploy_time: chrono::DateTime<chrono::Utc>,
    /// 状态
    pub status: ModelDeploymentStatus,
    /// 模型大小(字节)
    pub size: usize,
    /// 描述
    pub description: String,
}

/// 模型监控指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetrics {
    /// 模型版本
    pub version: String,
    /// 预测次数
    pub prediction_count: usize,
    /// 平均预测时间(毫秒)
    pub avg_prediction_time: f64,
    /// 准确率
    pub accuracy: f64,
    /// 错误率
    pub error_rate: f64,
    /// 最后预测时间
    pub last_prediction_time: chrono::DateTime<chrono::Utc>,
    /// 内存使用(MB)
    pub memory_usage: f64,
}

/// 模型部署管理器
pub struct ModelDeploymentManager {
    /// 模型版本集合
    models: Arc<RwLock<std::collections::HashMap<String, Arc<dyn MLModel>>>>,
    /// 模型版本信息
    model_versions: Arc<RwLock<std::collections::HashMap<String, ModelVersion>>>,
    /// 模型监控指标
    model_metrics: Arc<RwLock<std::collections::HashMap<String, ModelMetrics>>>,
    /// 部署状态
    is_running: Arc<AtomicBool>,
    /// 监控任务句柄
    monitor_task: Option<tokio::task::JoinHandle<()>>,
}

impl ModelDeploymentManager {
    /// 创建新的模型部署管理器
    pub fn new() -> Self {
        Self {
            models: Arc::new(RwLock::new(std::collections::HashMap::new())),
            model_versions: Arc::new(RwLock::new(std::collections::HashMap::new())),
            model_metrics: Arc::new(RwLock::new(std::collections::HashMap::new())),
            is_running: Arc::new(AtomicBool::new(false)),
            monitor_task: None,
        }
    }

    /// 部署模型
    pub async fn deploy_model(&self, version: &str, model: Arc<dyn MLModel>) -> Result<(), String> {
        log::info!("Deploying model version: {}", version);

        // 更新模型状态为部署中
        let model_version = ModelVersion {
            version: version.to_string(),
            model_type: model.model_type().to_string(),
            deploy_time: chrono::Utc::now(),
            status: ModelDeploymentStatus::Deploying,
            size: 0, // 实际应用中应该计算模型大小
            description: "Model deployment".to_string(),
        };

        let mut model_versions = self.model_versions.write().await;
        model_versions.insert(version.to_string(), model_version);

        // 部署模型
        let mut models = self.models.write().await;
        models.insert(version.to_string(), model);

        // 更新模型状态为部署成功
        let mut model_versions = self.model_versions.write().await;
        if let Some(mv) = model_versions.get_mut(version) {
            mv.status = ModelDeploymentStatus::Deployed;
        }

        // 初始化监控指标
        let metrics = ModelMetrics {
            version: version.to_string(),
            prediction_count: 0,
            avg_prediction_time: 0.0,
            accuracy: 0.0,
            error_rate: 0.0,
            last_prediction_time: chrono::Utc::now(),
            memory_usage: 0.0,
        };

        let mut model_metrics = self.model_metrics.write().await;
        model_metrics.insert(version.to_string(), metrics);

        log::info!("Model version {} deployed successfully", version);
        Ok(())
    }

    /// 获取模型
    pub async fn get_model(&self, version: &str) -> Option<Arc<dyn MLModel>> {
        let models = self.models.read().await;
        models.get(version).cloned()
    }

    /// 获取当前活跃模型
    pub async fn get_active_model(&self) -> Option<Arc<dyn MLModel>> {
        let models = self.models.read().await;
        let model_versions = self.model_versions.read().await;

        // 查找最新的部署成功的模型
        let mut latest_version = None;
        let mut latest_time = chrono::DateTime::from_timestamp(0, 0).expect("epoch timestamp always valid");

        for (version, mv) in &*model_versions {
            if let ModelDeploymentStatus::Deployed = mv.status {
                if mv.deploy_time > latest_time {
                    latest_time = mv.deploy_time;
                    latest_version = Some(version.clone());
                }
            }
        }

        if let Some(version) = latest_version {
            models.get(&version).cloned()
        } else {
            None
        }
    }

    /// 停止模型
    pub async fn stop_model(&self, version: &str) -> Result<(), String> {
        log::info!("Stopping model version: {}", version);

        let mut model_versions = self.model_versions.write().await;
        if let Some(mv) = model_versions.get_mut(version) {
            mv.status = ModelDeploymentStatus::Stopped;
        }

        log::info!("Model version {} stopped successfully", version);
        Ok(())
    }

    /// 启动监控任务
    pub async fn start_monitoring(&mut self, interval: Duration) {
        if self.is_running.load(Ordering::SeqCst) {
            log::warn!("Monitoring already running");
            return;
        }

        self.is_running.store(true, Ordering::SeqCst);

        let manager = self.clone();
        let task = tokio::spawn(async move {
            while manager.is_running.load(Ordering::SeqCst) {
                manager.monitor_models().await;
                tokio::time::sleep(interval).await;
            }
        });

        self.monitor_task = Some(task);
        log::info!("Model monitoring started");
    }

    /// 停止监控任务
    pub async fn stop_monitoring(&mut self) {
        self.is_running.store(false, Ordering::SeqCst);

        if let Some(task) = self.monitor_task.take() {
            task.await.ok();
        }

        log::info!("Model monitoring stopped");
    }

    /// 监控模型
    async fn monitor_models(&self) {
        let models = self.models.read().await;
        let mut model_metrics = self.model_metrics.write().await;

        for version in (*models).keys() {
            if let Some(metrics) = model_metrics.get_mut(version) {
                // 更新内存使用
                metrics.memory_usage = self.get_memory_usage();

                // 记录监控日志
                log::debug!("Model {} metrics: prediction_count={}, avg_prediction_time={:.2}ms, accuracy={:.2}%, memory_usage={:.2}MB", 
                    version, metrics.prediction_count, metrics.avg_prediction_time, metrics.accuracy * 100.0, metrics.memory_usage);
            }
        }
    }

    /// 记录预测结果
    pub async fn record_prediction(
        &self,
        version: &str,
        prediction_time: f64,
        accuracy: Option<f64>,
    ) {
        let mut model_metrics = self.model_metrics.write().await;
        if let Some(metrics) = model_metrics.get_mut(version) {
            // 更新预测次数
            metrics.prediction_count += 1;

            // 更新平均预测时间
            metrics.avg_prediction_time = (metrics.avg_prediction_time
                * (metrics.prediction_count - 1) as f64
                + prediction_time)
                / metrics.prediction_count as f64;

            // 更新准确率
            if let Some(acc) = accuracy {
                metrics.accuracy = (metrics.accuracy * (metrics.prediction_count - 1) as f64 + acc)
                    / metrics.prediction_count as f64;
                metrics.error_rate = 1.0 - metrics.accuracy;
            }

            // 更新最后预测时间
            metrics.last_prediction_time = chrono::Utc::now();
        }
    }

    /// 获取内存使用
    fn get_memory_usage(&self) -> f64 {
        // 简化实现,实际应用中应该使用系统API获取内存使用
        0.0
    }

    /// 获取模型版本信息
    pub async fn get_model_versions(&self) -> Vec<ModelVersion> {
        let model_versions = self.model_versions.read().await;
        model_versions.values().cloned().collect()
    }

    /// 获取模型监控指标
    pub async fn get_model_metrics(&self, version: &str) -> Option<ModelMetrics> {
        let model_metrics = self.model_metrics.read().await;
        model_metrics.get(version).cloned()
    }
}

impl Clone for ModelDeploymentManager {
    fn clone(&self) -> Self {
        Self {
            models: self.models.clone(),
            model_versions: self.model_versions.clone(),
            model_metrics: self.model_metrics.clone(),
            is_running: self.is_running.clone(),
            monitor_task: None, // 不克隆任务句柄
        }
    }
}

impl Default for ModelDeploymentManager {
    fn default() -> Self {
        Self::new()
    }
}
