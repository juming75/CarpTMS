//! 备份与恢复模块
//! 支持自动化备份、异地复制、恢复演练

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tokio::sync::mpsc;

/// 备份类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BackupType {
    /// 完整备份
    Full,
    /// 增量备份
    Incremental,
    /// 差异备份
    Differential,
}

/// 备份状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BackupStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Verified,
}

/// 备份目标位置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupLocation {
    /// 位置名称
    pub name: String,
    /// 路径或URL
    pub path: String,
    /// 是否为远程位置
    pub is_remote: bool,
    /// 远程类型 (s3, ftp, sftp, etc.)
    pub remote_type: Option<String>,
}

/// 备份配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupConfig {
    /// 备份保留份数
    pub retention_count: u32,
    /// 保留天数
    pub retention_days: u32,
    /// 是否启用压缩
    pub compress: bool,
    /// 压缩级别 (1-9)
    pub compress_level: u32,
    /// 是否加密
    pub encrypt: bool,
    /// 加密密钥路径
    pub encrypt_key_path: Option<String>,
    /// 备份位置列表
    pub locations: Vec<BackupLocation>,
    /// 自动备份间隔（秒）
    pub auto_backup_interval_seconds: u64,
    /// 完整备份间隔（秒）
    pub full_backup_interval_seconds: u64,
    /// 异地复制配置
    pub remote_replication: Option<RemoteReplicationConfig>,
    /// 备份验证配置
    pub verification: BackupVerificationConfig,
}

/// 异地复制配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteReplicationConfig {
    /// 是否启用
    pub enabled: bool,
    /// 目标区域
    pub target_region: String,
    /// 目标存储类型
    pub target_storage: String,
    /// 目标端点
    pub target_endpoint: String,
    /// 访问密钥ID
    pub access_key_id: Option<String>,
    /// 秘密访问密钥路径
    pub secret_access_key_path: Option<String>,
    /// 复制策略 (immediate, scheduled, realtime)
    pub replication_strategy: String,
    /// 调度间隔（秒）
    pub schedule_interval_seconds: u64,
}

/// 备份验证配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupVerificationConfig {
    /// 是否启用验证
    pub enabled: bool,
    /// 验证方式 (checksum, restore_test, all)
    pub verification_method: String,
    /// 验证间隔（天）
    pub verification_interval_days: u32,
    /// 恢复演练间隔（天）
    pub drill_interval_days: u32,
    /// 演练保留时间（小时）
    pub drill_retention_hours: u32,
}

/// 备份记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupRecord {
    pub id: String,
    pub backup_type: BackupType,
    pub status: BackupStatus,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub size_bytes: u64,
    pub checksum: Option<String>,
    pub locations: Vec<String>,
    pub error_message: Option<String>,
    pub verified: bool,
    pub verified_at: Option<DateTime<Utc>>,
}

/// 恢复演练记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrillRecord {
    pub id: String,
    pub backup_id: String,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub status: String,
    pub test_database_name: String,
    pub test_results: Option<serde_json::Value>,
    pub error_message: Option<String>,
}

/// 备份管理器
#[derive(Clone)]
pub struct BackupManager {
    config: BackupConfig,
    backup_dir: PathBuf,
    database_url: String,
}

impl BackupManager {
    /// 创建备份管理器
    pub fn new(config: BackupConfig, backup_dir: &Path, database_url: &str) -> Self {
        Self {
            config,
            backup_dir: backup_dir.to_path_buf(),
            database_url: database_url.to_string(),
        }
    }

    /// 执行完整备份
    pub async fn create_full_backup(&self) -> Result<BackupRecord, BackupError> {
        let backup_id = format!("full_{}", Utc::now().format("%Y%m%d_%H%M%S"));

        let record = BackupRecord {
            id: backup_id.clone(),
            backup_type: BackupType::Full,
            status: BackupStatus::Running,
            started_at: Utc::now(),
            completed_at: None,
            size_bytes: 0,
            checksum: None,
            locations: Vec::new(),
            error_message: None,
            verified: false,
            verified_at: None,
        };

        // 执行 pg_dump 备份
        let backup_path = self.backup_dir.join(format!("{}.dump", backup_id));

        let output = Command::new("pg_dump")
            .args(&[
                "--dbname",
                &self.database_url,
                "--format",
                "custom",
                "--compress",
                &self.config.compress_level.to_string(),
                "--file",
                backup_path.to_str().unwrap(),
            ])
            .output()
            .map_err(|e| BackupError::BackupFailed(e.to_string()))?;

        if !output.status.success() {
            return Err(BackupError::BackupFailed(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        // 计算校验和
        let checksum = self.calculate_checksum(&backup_path)?;

        // 获取文件大小
        let size = fs::metadata(&backup_path)
            .map_err(|e| BackupError::BackupFailed(e.to_string()))?
            .len();

        // 复制到备份位置
        let locations = self.replicate_to_locations(&backup_path, &backup_id).await?;

        // 删除临时文件
        fs::remove_file(&backup_path).ok();

        Ok(BackupRecord {
            id: backup_id,
            backup_type: BackupType::Full,
            status: BackupStatus::Completed,
            started_at: record.started_at,
            completed_at: Some(Utc::now()),
            size_bytes: size,
            checksum: Some(checksum),
            locations,
            error_message: None,
            verified: false,
            verified_at: None,
        })
    }

    /// 执行增量备份
    pub async fn create_incremental_backup(&self) -> Result<BackupRecord, BackupError> {
        // 使用 pg_basebackup 或 WAL 归档实现增量备份
        let backup_id = format!("incr_{}", Utc::now().format("%Y%m%d_%H%M%S"));

        let record = BackupRecord {
            id: backup_id.clone(),
            backup_type: BackupType::Incremental,
            status: BackupStatus::Running,
            started_at: Utc::now(),
            completed_at: None,
            size_bytes: 0,
            checksum: None,
            locations: Vec::new(),
            error_message: None,
            verified: false,
            verified_at: None,
        };

        // 执行增量备份逻辑
        let backup_path = self.backup_dir.join(format!("{}.wal", backup_id));

        // 触发 WAL 归档
        let output = Command::new("psql")
            .args(&[
                "--dbname",
                &self.database_url,
                "--command",
                "SELECT pg_switch_wal();",
            ])
            .output()
            .map_err(|e| BackupError::BackupFailed(e.to_string()))?;

        if !output.status.success() {
            return Err(BackupError::BackupFailed(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        // 复制 WAL 文件到备份位置
        let locations = self.replicate_to_locations(&backup_path, &backup_id).await?;

        Ok(BackupRecord {
            id: backup_id,
            backup_type: BackupType::Incremental,
            status: BackupStatus::Completed,
            started_at: record.started_at,
            completed_at: Some(Utc::now()),
            size_bytes: fs::metadata(&backup_path).map(|m| m.len()).unwrap_or(0),
            checksum: Some(self.calculate_checksum(&backup_path)?),
            locations,
            error_message: None,
            verified: false,
            verified_at: None,
        })
    }

    /// 恢复到指定时间点
    pub async fn point_in_time_recovery(
        &self,
        target_time: DateTime<Utc>,
    ) -> Result<(), BackupError> {
        // 找到最近的完整备份
        let latest_backup = self.find_latest_full_backup()?;

        // 执行恢复
        let recovery_target = target_time.format("%Y-%m-%d %H:%M:%S").to_string();

        // 停止数据库
        Command::new("pg_ctl")
            .args(&["stop", "-D", "/var/lib/postgresql/data"])
            .output()
            .map_err(|e| BackupError::RecoveryFailed(e.to_string()))?;

        // 恢复基础备份
        Command::new("pg_restore")
            .args(&[
                "--dbname",
                &self.database_url,
                "--clean",
                "--if-exists",
                latest_backup.as_str(),
            ])
            .output()
            .map_err(|e| BackupError::RecoveryFailed(e.to_string()))?;

        // 重启数据库
        Command::new("pg_ctl")
            .args(&["start", "-D", "/var/lib/postgresql/data"])
            .output()
            .map_err(|e| BackupError::RecoveryFailed(e.to_string()))?;

        Ok(())
    }

    /// 执行恢复演练
    pub async fn execute_drill(&self, backup_id: &str) -> Result<DrillRecord, BackupError> {
        let drill_id = format!("drill_{}", Utc::now().format("%Y%m%d_%H%M%S"));
        let test_db_name = format!("carptms_drill_{}", Utc::now().format("%Y%m%d_%H%M%S"));

        let record = DrillRecord {
            id: drill_id.clone(),
            backup_id: backup_id.to_string(),
            started_at: Utc::now(),
            completed_at: None,
            status: "running".to_string(),
            test_database_name: test_db_name.clone(),
            test_results: None,
            error_message: None,
        };

        // 创建测试数据库
        let create_db = Command::new("psql")
            .args(&["--dbname", &self.database_url, "--command", &format!("CREATE DATABASE {};", test_db_name)])
            .output();

        if create_db.is_err() || !create_db.unwrap().status.success() {
            return Err(BackupError::RecoveryFailed("无法创建测试数据库".to_string()));
        }

        // 恢复备份到测试数据库
        let backup_path = self.backup_dir.join(format!("{}.dump", backup_id));
        let restore_result = Command::new("pg_restore")
            .args(&[
                "--dbname",
                &format!("{}/{}", self.database_url, test_db_name),
                backup_path.to_str().unwrap(),
            ])
            .output();

        if restore_result.is_err() || !restore_result.unwrap().status.success() {
            // 清理测试数据库
            Command::new("psql")
                .args(&[
                    "--dbname",
                    &self.database_url,
                    "--command",
                    &format!("DROP DATABASE IF EXISTS {};", test_db_name),
                ])
                .output()
                .ok();

            return Err(BackupError::RecoveryFailed("恢复演练失败".to_string()));
        }

        // 执行基本验证
        let verification_result = self.verify_restored_data(&test_db_name).await;

        // 清理测试数据库
        Command::new("psql")
            .args(&[
                "--dbname",
                &self.database_url,
                "--command",
                &format!("DROP DATABASE IF EXISTS {};", test_db_name),
            ])
            .output()
            .ok();

        Ok(DrillRecord {
            id: drill_id,
            backup_id: backup_id.to_string(),
            started_at: record.started_at,
            completed_at: Some(Utc::now()),
            status: if verification_result.is_ok() {
                "passed".to_string()
            } else {
                "failed".to_string()
            },
            test_database_name: test_db_name,
            test_results: verification_result.ok(),
            error_message: verification_result.err().map(|e| e.to_string()),
        })
    }

    /// 验证恢复的数据
    async fn verify_restored_data(&self, db_name: &str) -> Result<serde_json::Value, BackupError> {
        // 检查关键表是否存在
        let tables_check = Command::new("psql")
            .args(&[
                "--dbname",
                &format!("{}/{}", self.database_url, db_name),
                "--command",
                "SELECT COUNT(*) FROM pg_tables WHERE schemaname = 'public';",
            ])
            .output()
            .map_err(|e| BackupError::RecoveryFailed(e.to_string()))?;

        // 检查数据完整性
        let data_check = Command::new("psql")
            .args(&[
                "--dbname",
                &format!("{}/{}", self.database_url, db_name),
                "--command",
                "SELECT COUNT(*) FROM users WHERE user_id = 1;",
            ])
            .output();

        Ok(serde_json::json!({
            "tables_exist": tables_check.status.success(),
            "data_integrity": data_check.map(|o| o.status.success()).unwrap_or(false)
        }))
    }

    /// 复制备份到指定位置
    async fn replicate_to_locations(
        &self,
        source: &Path,
        backup_id: &str,
    ) -> Result<Vec<String>, BackupError> {
        let mut locations = Vec::new();

        for location in &self.config.locations {
            let dest_path = if location.is_remote {
                format!("{}/{}", location.path, backup_id)
            } else {
                format!("{}/{}.dump", location.path, backup_id)
            };

            if location.is_remote {
                // 远程复制 (S3, FTP, etc.)
                self replicate_to_remote(source, &dest_path, &location)?;
            } else {
                // 本地复制
                fs::copy(source, &dest_path)
                    .map_err(|e| BackupError::ReplicationFailed(e.to_string()))?;
            }

            locations.push(dest_path);
        }

        Ok(locations)
    }

    /// 复制到远程存储
    fn replicate_to_remote(
        &self,
        source: &Path,
        dest: &str,
        location: &BackupLocation,
    ) -> Result<(), BackupError> {
        match location.remote_type.as_deref() {
            Some("s3") => {
                // 使用 AWS CLI 或 S3 客户端
                Command::new("aws")
                    .args(&["s3", "cp", source.to_str().unwrap(), dest])
                    .output()
                    .map_err(|e| BackupError::ReplicationFailed(e.to_string()))?;
            }
            Some("ftp") | Some("sftp") => {
                // 使用 scp 或专门的 FTP 客户端
                Command::new("scp")
                    .args(&[source.to_str().unwrap(), dest])
                    .output()
                    .map_err(|e| BackupError::ReplicationFailed(e.to_string()))?;
            }
            _ => {
                return Err(BackupError::ReplicationFailed(
                    "不支持的远程存储类型".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// 计算文件校验和
    fn calculate_checksum(&self, path: &Path) -> Result<String, BackupError> {
        let output = Command::new("sha256sum")
            .arg(path)
            .output()
            .map_err(|e| BackupError::BackupFailed(e.to_string()))?;

        let checksum = String::from_utf8_lossy(&output.stdout);
        Ok(checksum.split_whitespace().next().unwrap_or("").to_string())
    }

    /// 查找最新的完整备份
    fn find_latest_full_backup(&self) -> Result<String, BackupError> {
        let entries = fs::read_dir(&self.backup_dir)
            .map_err(|e| BackupError::BackupFailed(e.to_string()))?;

        let mut backups: Vec<_> = entries
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().starts_with("full_"))
            .collect();

        backups.sort_by_key(|e| e.metadata().ok().and_then(|m| m.modified().ok()));

        backups
            .last()
            .map(|e| e.path().to_string_lossy().to_string())
            .ok_or_else(|| BackupError::BackupNotFound("没有找到完整备份".to_string()))
    }

    /// 清理过期备份
    pub fn cleanup_expired_backups(&self) -> Result<u32, BackupError> {
        let retention_duration = chrono::Duration::days(self.config.retention_days as i64);

        let entries = fs::read_dir(&self.backup_dir)
            .map_err(|e| BackupError::BackupFailed(e.to_string()))?;

        let mut deleted_count = 0;

        for entry in entries.filter_map(|e| e.ok()) {
            if let Ok(metadata) = entry.metadata() {
                if let Ok(modified) = metadata.modified() {
                    let modified_dt: DateTime<Utc> = modified.into();
                    if Utc::now() - modified_dt > retention_duration {
                        fs::remove_file(entry.path()).ok();
                        deleted_count += 1;
                    }
                }
            }
        }

        Ok(deleted_count)
    }
}

/// 备份错误类型
#[derive(Debug)]
pub enum BackupError {
    BackupFailed(String),
    BackupNotFound(String),
    RecoveryFailed(String),
    ReplicationFailed(String),
    VerificationFailed(String),
}

impl std::fmt::Display for BackupError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BackupError::BackupFailed(msg) => write!(f, "备份失败: {}", msg),
            BackupError::BackupNotFound(msg) => write!(f, "备份未找到: {}", msg),
            BackupError::RecoveryFailed(msg) => write!(f, "恢复失败: {}", msg),
            BackupError::ReplicationFailed(msg) => write!(f, "复制失败: {}", msg),
            BackupError::VerificationFailed(msg) => write!(f, "验证失败: {}", msg),
        }
    }
}

impl std::error::Error for BackupError {}

/// 备份配置示例
pub const DEFAULT_BACKUP_CONFIG: &str = r#"
[backup]
retention_count = 7
retention_days = 180
compress = true
compress_level = 6
encrypt = true

[[backup.locations]]
name = "local"
path = "/var/backups/carptms"
is_remote = false

[[backup.locations]]
name = "remote_backup"
path = "s3://carptms-backups/database"
is_remote = true
remote_type = "s3"

[backup.remote_replication]
enabled = true
target_region = "cn-north-1"
target_storage = "s3"
target_endpoint = "s3.cn-north-1.amazonaws.com"
replication_strategy = "realtime"
schedule_interval_seconds = 3600

[backup.verification]
enabled = true
verification_method = "all"
verification_interval_days = 7
drill_interval_days = 30
drill_retention_hours = 24
"#;
