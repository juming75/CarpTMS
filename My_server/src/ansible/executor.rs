//! Ansible Playbook 执行引擎（任务编排）
//! 管理异步 Playbook 执行任务，提供状态追踪
//! 实际的 Ansible 调用由 inventory::AnsibleExecutor 处理

use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use super::models::PlaybookExecution;

/// 异步任务执行器（管理后台运行的任务状态）
pub struct AnsibleExecutor {
    running_tasks: Arc<RwLock<HashMap<Uuid, PlaybookExecution>>>,
    ansible_playbook: String,
    playbook_dir: String,
    inventory_dir: String,
}

impl AnsibleExecutor {
    pub fn new(base_dir: String) -> Self {
        Self {
            running_tasks: Arc::new(RwLock::new(HashMap::new())),
            ansible_playbook: which_ansible_playbook(),
            playbook_dir: format!("{}/playbooks", base_dir),
            inventory_dir: format!("{}/inventory", base_dir),
        }
    }

    /// 检查 Ansible 可用性
    pub async fn check_availability(&self) -> Result<String, String> {
        let output = tokio::process::Command::new(&self.ansible_playbook)
            .arg("--version")
            .output()
            .await
            .map_err(|e| format!("无法找到 ansible-playbook: {}", e))?;
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err("ansible-playbook 不可用".to_string())
        }
    }

    /// 扫描 Playbook 文件列表
    pub async fn scan_playbooks(&self) -> Vec<super::models::PlaybookInfo> {
        let mut playbooks = Vec::new();
        let dir = std::path::Path::new(&self.playbook_dir);
        if !dir.exists() { return playbooks; }

        for entry in std::fs::read_dir(dir).into_iter().flatten() {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.extension().map_or(false, |e| e == "yaml" || e == "yml") {
                    let name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("unknown").to_string();
                    playbooks.push(super::models::PlaybookInfo {
                        id: Uuid::new_v4(),
                        name,
                        description: None,
                        path: path.to_string_lossy().to_string(),
                        tags: Vec::new(),
                        created_at: Utc::now(),
                    });
                }
            }
        }
        playbooks
    }

    /// 异步执行 Playbook（返回任务 ID）
    pub async fn execute_playbook(
        &self,
        playbook_path: &str,
        inventory: Option<&str>,
        extra_vars: Option<&serde_json::Value>,
        tags: Option<&[String]>,
        limit: Option<&str>,
    ) -> Result<Uuid, String> {
        let task_id = Uuid::new_v4();
        let execution = PlaybookExecution {
            id: task_id,
            playbook_id: Uuid::nil(),
            playbook_name: std::path::Path::new(playbook_path)
                .file_stem().and_then(|s| s.to_str()).unwrap_or("unknown").to_string(),
            status: "running".to_string(),
            progress: 0.0,
            started_at: Some(Utc::now()),
            finished_at: None,
            executed_by: Uuid::nil(),
            log: String::new(),
            result: None,
        };

        {
            let mut tasks = self.running_tasks.write().await;
            tasks.insert(task_id, execution);
        }

        let running_tasks = self.running_tasks.clone();
        let pp = playbook_path.to_string();
        let inv = inventory.map(|s| s.to_string());
        let ev = extra_vars.cloned();
        let tg = tags.map(|v| v.to_vec());
        let lm = limit.map(|s| s.to_string());
        let ap = self.ansible_playbook.clone();

        tokio::spawn(async move {
            let mut cmd = tokio::process::Command::new(&ap);
            cmd.arg(&pp).arg("-i").arg(inv.as_deref().unwrap_or("inventory/hosts.yaml")).arg("-v");
            if let Some(ref v) = ev { cmd.arg("--extra-vars").arg(v.to_string()); }
            if let Some(ref t) = tg { cmd.arg("--tags").arg(t.join(",")); }
            if let Some(ref l) = lm { cmd.arg("--limit").arg(l); }

            let output = cmd.output().await;
            let mut tasks = running_tasks.write().await;
            if let Some(ref mut task) = tasks.get_mut(&task_id) {
                match output {
                    Ok(out) => {
                        let log = String::from_utf8_lossy(&out.stdout).to_string();
                        let err_log = String::from_utf8_lossy(&out.stderr).to_string();
                        task.log = if err_log.is_empty() { log } else { format!("{}\n{}", log, err_log) };
                        task.status = if out.status.success() { "success".into() } else { "failed".into() };
                        task.progress = 100.0;
                        task.finished_at = Some(Utc::now());
                        task.result = Some(serde_json::json!({
                            "exit_code": out.status.code(),
                            "stdout_length": out.stdout.len(),
                            "stderr_length": out.stderr.len(),
                        }));
                    }
                    Err(e) => {
                        task.status = "failed".into();
                        task.log = format!("执行失败: {}", e);
                        task.finished_at = Some(Utc::now());
                    }
                }
            }
        });

        Ok(task_id)
    }

    /// 获取执行状态
    pub async fn get_execution_status(&self, task_id: Uuid) -> Option<PlaybookExecution> {
        let tasks = self.running_tasks.read().await;
        tasks.get(&task_id).cloned()
    }
}

fn which_ansible_playbook() -> String {
    for path in &[
        "/usr/bin/ansible-playbook", "/usr/local/bin/ansible-playbook",
        "/opt/ansible/bin/ansible-playbook", "ansible-playbook",
    ] {
        if std::path::Path::new(path).exists() { return path.to_string(); }
    }
    "ansible-playbook".to_string()
}
