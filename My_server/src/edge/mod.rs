//! Edge Computing Module
//! Offloads processing to edge devices to reduce central server load

use log::info;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

/// Edge device status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EdgeDeviceStatus {
    Online,
    Offline,
    Busy,
    Error,
}

/// Edge device type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EdgeDeviceType {
    VehicleTerminal,
    RoadsideUnit,
    Gateway,
    SensorNode,
}

/// Edge device
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeDevice {
    pub id: String,
    pub name: String,
    pub device_type: EdgeDeviceType,
    pub status: EdgeDeviceStatus,
    pub ip_address: String,
    pub port: u16,
    pub last_heartbeat: String,
    pub compute_capacity: u32,
    pub memory: u32,
    pub storage: u32,
    pub supported_tasks: Vec<String>,
}

/// Edge task type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EdgeTaskType {
    LocationProcessing,
    VideoAnalysis,
    AlarmProcessing,
    DataAggregation,
    PredictiveAnalysis,
}

impl std::fmt::Display for EdgeTaskType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EdgeTaskType::LocationProcessing => write!(f, "LocationProcessing"),
            EdgeTaskType::VideoAnalysis => write!(f, "VideoAnalysis"),
            EdgeTaskType::AlarmProcessing => write!(f, "AlarmProcessing"),
            EdgeTaskType::DataAggregation => write!(f, "DataAggregation"),
            EdgeTaskType::PredictiveAnalysis => write!(f, "PredictiveAnalysis"),
        }
    }
}

/// Edge task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeTask {
    pub id: String,
    pub task_type: EdgeTaskType,
    pub data: serde_json::Value,
    pub priority: u32,
    pub created_at: String,
    pub estimated_duration: u32,
    pub assigned_device_id: Option<String>,
    pub status: EdgeTaskStatus,
    pub result: Option<serde_json::Value>,
}

/// Edge task status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EdgeTaskStatus {
    Pending,
    Assigned,
    Running,
    Completed,
    Failed,
    Timeout,
}

/// Edge computing service
pub struct EdgeComputingService {
    devices: Arc<RwLock<HashMap<String, EdgeDevice>>>,
    tasks: Arc<RwLock<HashMap<String, EdgeTask>>>,
    task_tx: mpsc::Sender<EdgeTask>,
}

impl Default for EdgeComputingService {
    fn default() -> Self {
        Self::new()
    }
}

impl EdgeComputingService {
    pub fn new() -> Self {
        let (task_tx, mut task_rx) = mpsc::channel(1000);

        let tasks = Arc::new(RwLock::new(HashMap::new()));
        let tasks_clone = tasks.clone();

        let devices = Arc::new(RwLock::new(HashMap::new()));
        let devices_clone = devices.clone();

        tokio::spawn(async move {
            while let Some(task) = task_rx.recv().await {
                Self::process_task(task, devices_clone.clone(), tasks_clone.clone()).await;
            }
        });

        Self {
            devices,
            tasks,
            task_tx,
        }
    }

    async fn process_task(
        task: EdgeTask,
        devices: Arc<RwLock<HashMap<String, EdgeDevice>>>,
        tasks: Arc<RwLock<HashMap<String, EdgeTask>>>,
    ) {
        match Self::allocate_task(&task, &devices).await {
            Ok(device_id) => {
                info!("Task {} allocated to device {}", task.id, device_id);

                let mut tasks_guard = tasks.write().await;
                if let Some(t) = tasks_guard.get_mut(&task.id) {
                    t.assigned_device_id = Some(device_id.clone());
                    t.status = EdgeTaskStatus::Assigned;
                }
                drop(tasks_guard);

                let result = Self::execute_task(&task, &device_id).await;

                let mut tasks_guard = tasks.write().await;
                if let Some(t) = tasks_guard.get_mut(&task.id) {
                    match result {
                        Ok(output) => {
                            t.status = EdgeTaskStatus::Completed;
                            t.result = Some(output);
                            info!("Task {} completed", task.id);
                        }
                        Err(e) => {
                            t.status = EdgeTaskStatus::Failed;
                            info!("Task {} failed: {}", task.id, e);
                        }
                    }
                }
            }
            Err(e) => {
                info!("Task allocation failed: {}", e);
                let mut tasks_guard = tasks.write().await;
                if let Some(t) = tasks_guard.get_mut(&task.id) {
                    t.status = EdgeTaskStatus::Failed;
                }
            }
        }
    }

    async fn allocate_task(
        task: &EdgeTask,
        devices: &Arc<RwLock<HashMap<String, EdgeDevice>>>,
    ) -> Result<String, EdgeError> {
        let devices_guard = devices.read().await;

        let available_devices: Vec<&EdgeDevice> = devices_guard
            .values()
            .filter(|d| d.status == EdgeDeviceStatus::Online)
            .filter(|d| d.supported_tasks.contains(&task.task_type.to_string()))
            .collect();

        if available_devices.is_empty() {
            return Err(EdgeError::DeviceUnavailable(
                "No available devices".to_string(),
            ));
        }

        let best_device = available_devices
            .iter()
            .max_by_key(|d| d.compute_capacity)
            .unwrap();

        Ok(best_device.id.clone())
    }

    async fn execute_task(
        task: &EdgeTask,
        _device_id: &str,
    ) -> Result<serde_json::Value, EdgeError> {
        tokio::time::sleep(tokio::time::Duration::from_secs(
            task.estimated_duration as u64,
        ))
        .await;
        Ok(serde_json::json!({"result": "success", "data": task.data}))
    }

    pub async fn register_device(&self, device: EdgeDevice) -> Result<(), EdgeError> {
        let device_id = device.id.clone();
        let mut devices = self.devices.write().await;
        devices.insert(device_id.clone(), device);
        info!("Edge device {} registered successfully", device_id);
        Ok(())
    }

    pub async fn update_device_status(
        &self,
        device_id: &str,
        status: EdgeDeviceStatus,
    ) -> Result<(), EdgeError> {
        let mut devices = self.devices.write().await;
        match devices.get_mut(device_id) {
            Some(device) => {
                device.status = status.clone();
                device.last_heartbeat = chrono::Utc::now().to_rfc3339();
                info!("Device {} status updated to {:?}", device_id, status);
                Ok(())
            }
            None => Err(EdgeError::DeviceNotFound(device_id.to_string())),
        }
    }

    pub async fn submit_task(&self, task: EdgeTask) -> Result<String, EdgeError> {
        let task_id = task.id.clone();
        let mut tasks = self.tasks.write().await;
        tasks.insert(task_id.clone(), task.clone());
        drop(tasks);

        self.task_tx
            .send(task)
            .await
            .map_err(|e| EdgeError::TaskAllocationFailed(e.to_string()))?;

        Ok(task_id)
    }

    pub async fn get_task_status(&self, task_id: &str) -> Result<EdgeTaskStatus, EdgeError> {
        let tasks = self.tasks.read().await;
        match tasks.get(task_id) {
            Some(task) => Ok(task.status.clone()),
            None => Err(EdgeError::TaskNotFound(task_id.to_string())),
        }
    }

    pub async fn get_task_result(
        &self,
        task_id: &str,
    ) -> Result<Option<serde_json::Value>, EdgeError> {
        let tasks = self.tasks.read().await;
        match tasks.get(task_id) {
            Some(task) => Ok(task.result.clone()),
            None => Err(EdgeError::TaskNotFound(task_id.to_string())),
        }
    }

    pub async fn get_all_devices(&self) -> Vec<EdgeDevice> {
        let devices = self.devices.read().await;
        devices.values().cloned().collect()
    }

    pub async fn get_all_tasks(&self) -> Vec<EdgeTask> {
        let tasks = self.tasks.read().await;
        tasks.values().cloned().collect()
    }
}

/// Edge computing error
#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
pub enum EdgeError {
    #[error("Device not found: {0}")]
    DeviceNotFound(String),
    #[error("Task not found: {0}")]
    TaskNotFound(String),
    #[error("Device unavailable: {0}")]
    DeviceUnavailable(String),
    #[error("Task allocation failed: {0}")]
    TaskAllocationFailed(String),
    #[error("Task execution failed: {0}")]
    TaskExecutionFailed(String),
    #[error("Network error: {0}")]
    NetworkError(String),
}
