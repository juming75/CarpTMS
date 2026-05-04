// Ansible 执行器
// 负责执行 Ansible 命令并解析结果

use tokio::process::Command;
use chrono::Utc;

use super::models::*;

pub struct AnsibleExecutor {
    ansible_dir: String,
    timeout_seconds: u64,
}

impl AnsibleExecutor {
    pub fn new(ansible_dir: String) -> Self {
        Self {
            ansible_dir,
            timeout_seconds: 3600, // 默认 1 小时超时
        }
    }

    /// 执行 Playbook
    pub async fn run_playbook(&self, request: &PlaybookRequest) -> Result<PlaybookResult, String> {
        let execution_id = uuid::Uuid::new_v4().to_string();
        let started_at = Utc::now();
        
        log::info!("执行 Playbook: {} (ID: {})", request.playbook, execution_id);
        
        // 构建命令
        let mut cmd = Command::new("ansible-playbook");
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        
        // 基本参数
        cmd.arg(&request.playbook);
        cmd.arg("-i").arg(&request.inventory);
        cmd.arg("--diff"); // 显示差异
        
        // Dry-run 模式
        if request.check_mode {
            cmd.arg("--check");
        }
        
        // 限制主机
        if let Some(ref limit) = request.limit {
            cmd.arg("--limit").arg(limit);
        }
        
        // 额外变量
        if let Some(ref extra_vars) = request.extra_vars {
            let vars_str = serde_json::to_string(extra_vars)
                .map_err(|e| format!("序列化 extra_vars 失败: {}", e))?;
            cmd.arg("-e").arg(vars_str);
        }
        
        // 标签
        if let Some(ref tags) = request.tags {
            cmd.arg("--tags").arg(tags.join(","));
        }
        
        // 输出 JSON 格式
        cmd.arg("-o"); // One-line output
        cmd.arg("--module-path").arg(format!("{}/library", self.ansible_dir));
        
        // 执行命令
        let start_time = std::time::Instant::now();
        let output = cmd.output().await
            .map_err(|e| format!("执行 ansible-playbook 失败: {}", e))?;
        let _duration_ms = start_time.elapsed().as_millis() as u64;
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        
        // 解析结果
        let results = self.parse_playbook_output(&stdout);
        let summary = self.generate_summary(&results);
        
        let status = if output.status.success() {
            ExecutionStatus::Success
        } else {
            ExecutionStatus::Failed
        };
        
        log::info!("Playbook 执行完成: {} - {:?}", execution_id, status);
        
        Ok(PlaybookResult {
            execution_id,
            playbook: request.playbook.clone(),
            status,
            started_at,
            finished_at: Some(Utc::now()),
            results,
            summary,
            logs: format!("STDOUT:\n{}\n\nSTDERR:\n{}", stdout, stderr),
        })
    }

    /// 执行快速命令
    pub async fn run_command(&self, request: &QuickCommand) -> Result<Vec<TaskResult>, String> {
        log::info!("执行快速命令: {} -m {} -a '{}'", 
            request.hosts, request.module, request.args);
        
        let mut cmd = Command::new("ansible");
        cmd.arg(request.hosts.clone());
        cmd.arg("-i").arg(&request.inventory);
        cmd.arg("-m").arg(&request.module);
        cmd.arg("-a").arg(&request.args);
        cmd.arg("-o"); // One-line output
        
        let output = cmd.output().await
            .map_err(|e| format!("执行 ansible 命令失败: {}", e))?;
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        let results = self.parse_ansible_output(&stdout);
        
        Ok(results)
    }

    /// 测试主机连通性
    pub async fn ping(&self, inventory: &str, hosts: &str) -> Result<Vec<TaskResult>, String> {
        log::info!("测试主机连通性: {}", hosts);
        
        let mut cmd = Command::new("ansible");
        cmd.arg(hosts);
        cmd.arg("-i").arg(inventory);
        cmd.arg("-m").arg("ping");
        cmd.arg("-o");
        
        let output = cmd.output().await
            .map_err(|e| format!("执行 ansible ping 失败: {}", e))?;
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        let results = self.parse_ansible_output(&stdout);
        
        Ok(results)
    }

    /// 解析 Playbook 输出
    fn parse_playbook_output(&self, output: &str) -> Vec<TaskResult> {
        let mut results = Vec::new();
        
        for line in output.lines() {
            if line.trim().is_empty() {
                continue;
            }
            
            // 简单解析 Ansible 输出
            let parts: Vec<&str> = line.splitn(3, '|').collect();
            if parts.len() >= 2 {
                let host = parts[0].trim().to_string();
                let status_str = parts[1].trim();
                
                let (status, changed) = match status_str {
                    s if s.contains("SUCCESS") || s.contains("ok") => (TaskStatus::Ok, false),
                    s if s.contains("CHANGED") => (TaskStatus::Changed, true),
                    s if s.contains("FAILED") => (TaskStatus::Failed, false),
                    s if s.contains("SKIPPED") => (TaskStatus::Skipped, false),
                    s if s.contains("UNREACHABLE") => (TaskStatus::Unreachable, false),
                    _ => (TaskStatus::Ok, false),
                };
                
                results.push(TaskResult {
                    host,
                    task_name: "playbook_task".to_string(),
                    status,
                    changed,
                    output: Some(line.to_string()),
                    error: None,
                    duration_ms: 0,
                });
            }
        }
        
        results
    }

    /// 解析 Ansible 命令输出
    fn parse_ansible_output(&self, output: &str) -> Vec<TaskResult> {
        let mut results = Vec::new();
        
        for line in output.lines() {
            if line.trim().is_empty() {
                continue;
            }
            
            let parts: Vec<&str> = line.split("|").collect();
            if parts.len() >= 2 {
                let host = parts[0].trim().to_string();
                let status_str = parts.get(1).unwrap_or(&"").trim();
                
                let (status, changed) = match status_str {
                    s if s.contains("SUCCESS") || s.contains("ok") => (TaskStatus::Ok, false),
                    s if s.contains("CHANGED") => (TaskStatus::Changed, true),
                    s if s.contains("FAILED") => (TaskStatus::Failed, false),
                    s if s.contains("UNREACHABLE") => (TaskStatus::Unreachable, false),
                    _ => (TaskStatus::Ok, false),
                };
                
                results.push(TaskResult {
                    host,
                    task_name: "ansible_command".to_string(),
                    status,
                    changed,
                    output: Some(line.to_string()),
                    error: None,
                    duration_ms: 0,
                });
            }
        }
        
        results
    }

    /// 生成执行摘要
    fn generate_summary(&self, results: &[TaskResult]) -> ExecutionSummary {
        let total = results.len();
        let ok = results.iter().filter(|r| r.status == TaskStatus::Ok).count();
        let changed = results.iter().filter(|r| r.changed).count();
        let failed = results.iter().filter(|r| r.status == TaskStatus::Failed).count();
        let skipped = results.iter().filter(|r| r.status == TaskStatus::Skipped).count();
        let unreachable = results.iter().filter(|r| r.status == TaskStatus::Unreachable).count();
        
        ExecutionSummary {
            total,
            ok,
            changed,
            failed,
            skipped,
            unreachable,
        }
    }

    /// 检查 Ansible 是否可用
    pub async fn check_availability(&self) -> Result<bool, String> {
        let output = Command::new("ansible-playbook")
            .arg("--version")
            .output()
            .await
            .map_err(|e| format!("Ansible 不可用: {}", e))?;
        
        if output.status.success() {
            let version = String::from_utf8_lossy(&output.stdout);
            log::info!("Ansible 版本: {}", version.lines().next().unwrap_or("未知"));
            Ok(true)
        } else {
            Err("Ansible 不可用".to_string())
        }
    }
}
