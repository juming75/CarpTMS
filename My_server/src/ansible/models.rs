// Ansible 数据模型

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// 主机信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Host {
    pub id: String,
    pub name: String,
    pub ansible_host: String,
    pub ansible_port: Option<u16>,
    pub ansible_user: String,
    pub groups: Vec<String>,
    pub variables: serde_json::Value,
    pub status: HostStatus,
}

/// 主机状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum HostStatus {
    Online,
    Offline,
    Unreachable,
    Unknown,
}

/// 服务器组
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerGroup {
    pub id: String,
    pub name: String,
    pub hosts: Vec<Host>,
    pub variables: serde_json::Value,
}

/// Playbook 执行请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybookRequest {
    pub playbook: String,           // playbook 路径
    pub inventory: String,          // inventory 路径
    pub extra_vars: Option<serde_json::Value>,
    pub limit: Option<String>,      // 限制主机
    pub check_mode: bool,           // Dry-run 模式
    pub tags: Option<Vec<String>>,  // 执行指定标签
}

/// 任务执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    pub host: String,
    pub task_name: String,
    pub status: TaskStatus,
    pub changed: bool,
    pub output: Option<String>,
    pub error: Option<String>,
    pub duration_ms: u64,
}

/// 任务状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
    Ok,
    Changed,
    Failed,
    Skipped,
    Unreachable,
}

/// Playbook 执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybookResult {
    pub execution_id: String,
    pub playbook: String,
    pub status: ExecutionStatus,
    pub started_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
    pub results: Vec<TaskResult>,
    pub summary: ExecutionSummary,
    pub logs: String,
}

/// 执行状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ExecutionStatus {
    Pending,
    Running,
    Success,
    Failed,
    Cancelled,
    Timeout,
}

/// 执行摘要
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionSummary {
    pub total: usize,
    pub ok: usize,
    pub changed: usize,
    pub failed: usize,
    pub skipped: usize,
    pub unreachable: usize,
}

/// 快速命令执行
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuickCommand {
    pub hosts: String,           // 主机选择器，如 "web_servers" 或 "all"
    pub module: String,          // 模块名，如 "shell", "yum", "copy"
    pub args: String,            // 模块参数
    pub inventory: String,
}

/// 执行历史记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionHistory {
    pub id: String,
    pub user_id: String,
    pub user_name: String,
    pub playbook_name: String,
    pub status: ExecutionStatus,
    pub hosts_count: usize,
    pub started_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
    pub summary: Option<ExecutionSummary>,
}

/// 库存源
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventorySource {
    pub id: String,
    pub name: String,
    pub path: String,
    pub source_type: InventorySourceType,
    pub last_updated: DateTime<Utc>,
}

/// 库存源类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum InventorySourceType {
    File,
    Dynamic,
    Manual,
}
