// Ansible 运维模块（仅在 remote-ops feature 启用时编译）
// 提供与 Ansible 的集成接口

#[cfg(feature = "remote-ops")]
pub mod executor;
#[cfg(feature = "remote-ops")]
pub mod inventory;
#[cfg(feature = "remote-ops")]
pub mod models;
#[cfg(feature = "remote-ops")]
pub mod routes;

// routes.rs 使用 inventory::AnsibleExecutor（含 run_playbook/run_command/ping 方法）
#[cfg(feature = "remote-ops")]
pub use inventory::AnsibleExecutor;
// executor.rs 提供后台任务管理功能
#[cfg(feature = "remote-ops")]
pub use executor::AnsibleExecutor as TaskExecutor;
#[cfg(feature = "remote-ops")]
pub use models::*;
