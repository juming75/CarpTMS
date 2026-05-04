//! 远程运维数据模型

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ═══════════ 服务器管理 ═══════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerStatus {
    Online,
    Offline,
    Connecting,
    Error,
    Maintenance,
}

impl Default for ServerStatus {
    fn default() -> Self { ServerStatus::Offline }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Server {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub host: String,
    pub port: i32,
    pub username: String,
    pub password: Option<String>,
    pub private_key: Option<String>,
    pub private_key_passphrase: Option<String>,
    pub status: String,
    pub group_id: Option<Uuid>,
    pub tags: Option<serde_json::Value>,
    pub os_type: Option<String>,
    pub last_connected_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Uuid,
    pub organization_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateServerRequest {
    pub name: String,
    pub description: Option<String>,
    pub host: String,
    pub port: Option<i32>,
    pub username: String,
    pub password: Option<String>,
    pub private_key: Option<String>,
    pub private_key_passphrase: Option<String>,
    pub group_id: Option<Uuid>,
    pub tags: Option<serde_json::Value>,
    pub os_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateServerRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub host: Option<String>,
    pub port: Option<i32>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub private_key: Option<String>,
    pub private_key_passphrase: Option<String>,
    pub group_id: Option<Uuid>,
    pub tags: Option<serde_json::Value>,
    pub os_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub host: String,
    pub port: i32,
    pub username: String,
    pub status: String,
    pub group_id: Option<Uuid>,
    pub group_name: Option<String>,
    pub tags: Option<serde_json::Value>,
    pub os_type: Option<String>,
    pub last_connected_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl From<Server> for ServerResponse {
    fn from(s: Server) -> Self {
        Self {
            id: s.id,
            name: s.name,
            description: s.description,
            host: s.host,
            port: s.port,
            username: s.username,
            status: s.status.clone(),
            group_id: s.group_id,
            group_name: None,
            tags: s.tags,
            os_type: s.os_type,
            last_connected_at: s.last_connected_at,
            created_at: s.created_at,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ServerListQuery {
    pub keyword: Option<String>,
    pub group_id: Option<Uuid>,
    pub status: Option<String>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

// ═══════════ 服务器组 ═══════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerGroup {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub parent_id: Option<Uuid>,
    pub server_count: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateServerGroupRequest {
    pub name: String,
    pub description: Option<String>,
    pub parent_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateServerGroupRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub parent_id: Option<Uuid>,
}

// ═══════════ 命令执行 ═══════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteCommandRequest {
    pub server_id: Uuid,
    pub command: String,
    pub working_dir: Option<String>,
    pub timeout_secs: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandResult {
    pub id: Uuid,
    pub server_id: Uuid,
    pub server_name: Option<String>,
    pub command: String,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
    pub status: CommandStatus,
    pub duration_ms: u64,
    pub executed_by: Uuid,
    pub executed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommandStatus {
    Pending,
    Running,
    Success,
    Failed,
    Timeout,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchExecuteRequest {
    pub server_ids: Vec<Uuid>,
    pub command: String,
    pub parallel: Option<bool>,
    pub timeout_secs: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchExecuteResult {
    pub results: Vec<ServerCommandResult>,
    pub total: usize,
    pub success: usize,
    pub failed: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerCommandResult {
    pub server_id: Uuid,
    pub server_name: Option<String>,
    pub host: String,
    pub success: bool,
    pub command: String,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
    pub duration_ms: u64,
}

// ═══════════ 连接测试 ═══════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestConnectionRequest {
    pub host: String,
    pub port: Option<i32>,
    pub username: String,
    pub password: Option<String>,
    pub private_key: Option<String>,
    pub private_key_passphrase: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestConnectionResult {
    pub success: bool,
    pub message: String,
    pub duration_ms: u64,
    pub os_type: Option<String>,
    pub hostname: Option<String>,
}

// ═══════════ 文件操作 ═══════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListDirectoryRequest {
    pub server_id: Uuid,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub name: String,
    pub path: String,
    pub file_type: String,
    pub size: i64,
    pub permissions: String,
    pub modified_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileTransferRequest {
    pub server_id: Uuid,
    pub remote_path: String,
    pub content: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileOperation {
    Read,
    Write,
    Delete,
    List,
}

// ═══════════ 服务器指标 ═══════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerMetrics {
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub disk_usage: f64,
    pub load_average: Vec<f64>,
    pub uptime_seconds: u64,
    pub network_rx_bytes: u64,
    pub network_tx_bytes: u64,
    pub process_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T: Serialize> {
    pub items: Vec<T>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
    pub total_pages: i64,
}

// ═══════════ Ansible Playbook ═══════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybookInfo {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub path: String,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutePlaybookRequest {
    pub playbook_id: Uuid,
    pub inventory_id: Option<Uuid>,
    pub server_ids: Option<Vec<Uuid>>,
    pub extra_vars: Option<serde_json::Value>,
    pub tags: Option<Vec<String>>,
    pub limit: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybookExecution {
    pub id: Uuid,
    pub playbook_id: Uuid,
    pub playbook_name: String,
    pub status: String,
    pub progress: f64,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
    pub executed_by: Uuid,
    pub log: String,
    pub result: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnsibleHost {
    pub name: String,
    pub ip: String,
    pub port: i32,
    pub username: String,
    pub groups: Vec<String>,
    pub vars: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnsibleGroup {
    pub name: String,
    pub description: Option<String>,
    pub hosts: Vec<String>,
    pub children: Vec<String>,
    pub vars: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnsibleInventory {
    pub name: String,
    pub hosts: Vec<AnsibleHost>,
    pub groups: Vec<AnsibleGroup>,
}
