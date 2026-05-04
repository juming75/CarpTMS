//! /! 数据库备份和恢复工具
//!
//! 提供数据库备份、恢复、验证等功能

use chrono::{DateTime, Local};
use sha2::{Digest, Sha256};
use sqlx::{postgres::PgPool, Row};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, SystemTime};
use tracing::{error, info};

/// 备份配置
#[derive(Debug, Clone)]
pub struct BackupConfig {
    /// 备份目录
    pub backup_dir: PathBuf,
    /// 数据库连接字符串
    pub database_url: String,
    /// 备份保留天数
    pub retention_days: i32,
    /// 是否压缩备份
    pub compress: bool,
    /// 备份超时时间(秒)
    pub timeout_seconds: u64,
}

impl Default for BackupConfig {
    fn default() -> Self {
        Self {
            backup_dir: PathBuf::from("./backups"),
            database_url: String::new(),
            retention_days: 30,
            compress: true,
            timeout_seconds: 3600,
        }
    }
}

/// 备份信息
#[derive(Debug, Clone)]
pub struct BackupInfo {
    pub id: i32,
    pub backup_time: DateTime<Local>,
    pub backup_type: String,
    pub file_path: PathBuf,
    pub size_bytes: u64,
    pub checksum: String,
    pub status: BackupStatus,
}

/// 备份状态
#[derive(Debug, Clone)]
pub enum BackupStatus {
    Running,
    Completed,
    Failed(String),
}

impl std::fmt::Display for BackupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BackupStatus::Running => write!(f, "running"),
            BackupStatus::Completed => write!(f, "completed"),
            BackupStatus::Failed(_) => write!(f, "failed"),
        }
    }
}

/// 数据库备份管理器
pub struct BackupManager {
    config: BackupConfig,
}

impl BackupManager {
    /// 创建新的备份管理器
    pub fn new(config: BackupConfig) -> Self {
        // 确保备份目录存在
        if !config.backup_dir.exists() {
            fs::create_dir_all(&config.backup_dir).expect("Failed to create backup directory");
        }

        Self { config }
    }

    /// 执行完整数据库备份
    pub async fn full_backup(&self, pool: &PgPool) -> Result<BackupInfo, BackupError> {
        let timestamp = Local::now().format("%Y%m%d_%H%M%S");
        let filename = format!("full_backup_{}.sql", timestamp);
        let file_path = self.config.backup_dir.join(&filename);

        info!("Starting full database backup to {:?}", file_path);

        // 记录备份开始
        let backup_id = self.record_backup_start(pool, "full", &file_path).await?;

        // 执行pg_dump命令
        let result = self.run_pg_dump(&file_path).await;

        match result {
            Ok(()) => {
                // 计算文件大小和校验和
                let metadata = fs::metadata(&file_path)?;
                let size_bytes = metadata.len();
                let checksum = self.calculate_checksum(&file_path).await?;

                // 压缩备份文件
                let final_path = if self.config.compress {
                    self.compress_backup(&file_path).await?
                } else {
                    file_path
                };

                // 更新备份记录
                self.record_backup_complete(pool, backup_id, size_bytes, &checksum, None)
                    .await?;

                info!(
                    "Full backup completed: {:?}, size: {} bytes",
                    final_path, size_bytes
                );

                Ok(BackupInfo {
                    id: backup_id,
                    backup_time: Local::now(),
                    backup_type: "full".to_string(),
                    file_path: final_path,
                    size_bytes,
                    checksum,
                    status: BackupStatus::Completed,
                })
            }
            Err(e) => {
                let error_msg = e.to_string();
                self.record_backup_complete(pool, backup_id, 0, "", Some(&error_msg))
                    .await?;
                error!("Full backup failed: {}", error_msg);
                Err(e)
            }
        }
    }

    /// 执行指定表的备份
    pub async fn table_backup(
        &self,
        pool: &PgPool,
        tables: &[String],
    ) -> Result<BackupInfo, BackupError> {
        let timestamp = Local::now().format("%Y%m%d_%H%M%S");
        let table_names = tables.join("_");
        let filename = format!("table_backup_{}_{}.sql", table_names, timestamp);
        let file_path = self.config.backup_dir.join(&filename);

        info!("Starting table backup for {:?} to {:?}", tables, file_path);

        let backup_id = self.record_backup_start(pool, "table", &file_path).await?;

        // 构建pg_dump命令,只备份指定表
        let mut args = vec![
            "--data-only".to_string(),
            "--inserts".to_string(),
            format!("--file={}", file_path.display()),
        ];

        for table in tables {
            args.push(format!("--table={}", table));
        }

        args.push(self.config.database_url.clone());

        let result = self.run_pg_dump_with_args(&args).await;

        match result {
            Ok(()) => {
                let metadata = fs::metadata(&file_path)?;
                let size_bytes = metadata.len();
                let checksum = self.calculate_checksum(&file_path).await?;

                let final_path = if self.config.compress {
                    self.compress_backup(&file_path).await?
                } else {
                    file_path
                };

                self.record_backup_complete(pool, backup_id, size_bytes, &checksum, None)
                    .await?;

                info!("Table backup completed: {:?}", final_path);

                Ok(BackupInfo {
                    id: backup_id,
                    backup_time: Local::now(),
                    backup_type: "table".to_string(),
                    file_path: final_path,
                    size_bytes,
                    checksum,
                    status: BackupStatus::Completed,
                })
            }
            Err(e) => {
                let error_msg = e.to_string();
                self.record_backup_complete(pool, backup_id, 0, "", Some(&error_msg))
                    .await?;
                Err(e)
            }
        }
    }

    /// 执行pg_dump命令
    async fn run_pg_dump(&self, output_path: &Path) -> Result<(), BackupError> {
        let args = vec![
            "--verbose".to_string(),
            "--no-owner".to_string(),
            "--no-privileges".to_string(),
            format!("--file={}", output_path.display()),
            self.config.database_url.clone(),
        ];

        self.run_pg_dump_with_args(&args).await
    }

    /// 执行pg_dump命令(带参数)
    async fn run_pg_dump_with_args(&self, args: &[String]) -> Result<(), BackupError> {
        let output = Command::new("pg_dump").args(args).output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(BackupError::PgDumpError(stderr.to_string()));
        }

        Ok(())
    }

    /// 压缩备份文件
    async fn compress_backup(&self, file_path: &Path) -> Result<PathBuf, BackupError> {
        let compressed_path = file_path.with_extension("sql.gz");

        info!("Compressing backup file to {:?}", compressed_path);

        let output = Command::new("gzip").arg("-c").arg(file_path).output()?;

        if !output.status.success() {
            return Err(BackupError::CompressionError(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        fs::write(&compressed_path, &output.stdout)?;
        fs::remove_file(file_path)?;

        Ok(compressed_path)
    }

    /// 计算文件校验和
    async fn calculate_checksum(&self, file_path: &Path) -> Result<String, BackupError> {
        let content = fs::read(file_path)?;
        let mut hasher = Sha256::new();
        hasher.update(&content);
        let result = hasher.finalize();
        Ok(format!("{:x}", result))
    }

    /// 记录备份开始
    async fn record_backup_start(
        &self,
        pool: &PgPool,
        backup_type: &str,
        file_path: &Path,
    ) -> Result<i32, BackupError> {
        let result = sqlx::query(
            r#"
            INSERT INTO db_backup_info (backup_type, backup_file_path, status, retention_days)
            VALUES ($1, $2, 'running', $3)
            RETURNING id
            "#,
        )
        .bind(backup_type)
        .bind(file_path.to_string_lossy().to_string())
        .bind(self.config.retention_days)
        .fetch_one(pool)
        .await?;

        let id: i32 = result.get(0);
        Ok(id)
    }

    /// 记录备份完成
    async fn record_backup_complete(
        &self,
        pool: &PgPool,
        backup_id: i32,
        size_bytes: u64,
        checksum: &str,
        error: Option<&str>,
    ) -> Result<(), BackupError> {
        let status = if error.is_some() {
            "failed"
        } else {
            "completed"
        };

        sqlx::query(
            r#"
            UPDATE db_backup_info 
            SET backup_size_bytes = $1, checksum = $2, status = $3, error_message = $4
            WHERE id = $5
            "#,
        )
        .bind(size_bytes as i64)
        .bind(checksum)
        .bind(status)
        .bind(error)
        .bind(backup_id)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// 从备份文件恢复数据库
    pub async fn restore(&self, _pool: &PgPool, backup_file: &Path) -> Result<(), BackupError> {
        info!("Starting database restore from {:?}", backup_file);

        // 验证备份文件
        if !backup_file.exists() {
            return Err(BackupError::FileNotFound(backup_file.to_path_buf()));
        }

        // 如果是压缩文件,先解压
        let sql_file = if backup_file.extension().map(|e| e == "gz").unwrap_or(false) {
            self.decompress_backup(backup_file).await?
        } else {
            backup_file.to_path_buf()
        };

        // 执行psql命令恢复
        let output = Command::new("psql")
            .arg(&self.config.database_url)
            .arg("-f")
            .arg(&sql_file)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(BackupError::RestoreError(stderr.to_string()));
        }

        // 清理临时解压文件
        if sql_file != backup_file.to_path_buf() {
            fs::remove_file(&sql_file)?;
        }

        info!("Database restore completed successfully");
        Ok(())
    }

    /// 解压备份文件
    async fn decompress_backup(&self, file_path: &Path) -> Result<PathBuf, BackupError> {
        let output_path = file_path.with_extension("");

        info!("Decompressing backup file to {:?}", output_path);

        let output = Command::new("gunzip").arg("-c").arg(file_path).output()?;

        if !output.status.success() {
            return Err(BackupError::DecompressionError(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        fs::write(&output_path, &output.stdout)?;

        Ok(output_path)
    }

    /// 验证备份文件完整性
    pub async fn verify_backup(
        &self,
        backup_file: &Path,
        expected_checksum: &str,
    ) -> Result<bool, BackupError> {
        if !backup_file.exists() {
            return Ok(false);
        }

        let actual_checksum = self.calculate_checksum(backup_file).await?;
        Ok(actual_checksum == expected_checksum)
    }

    /// 清理过期备份
    pub async fn cleanup_old_backups(&self, pool: &PgPool) -> Result<u64, BackupError> {
        info!(
            "Cleaning up old backups (retention: {} days)",
            self.config.retention_days
        );

        let result = sqlx::query(
            r#"
            DELETE FROM db_backup_info 
            WHERE backup_time < CURRENT_TIMESTAMP - ($1 || ' days')::INTERVAL
            RETURNING id
            "#,
        )
        .bind(self.config.retention_days)
        .fetch_all(pool)
        .await?;

        let deleted_count = result.len() as u64;

        // 同时清理文件系统中的旧备份
        self.cleanup_backup_files().await?;

        info!("Cleaned up {} old backups", deleted_count);
        Ok(deleted_count)
    }

    /// 清理文件系统中的备份文件
    async fn cleanup_backup_files(&self) -> Result<(), BackupError> {
        let cutoff_time = SystemTime::now()
            - Duration::from_secs((self.config.retention_days as u64) * 24 * 60 * 60);

        for entry in fs::read_dir(&self.config.backup_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                let metadata = entry.metadata()?;
                if let Ok(modified) = metadata.modified() {
                    if modified < cutoff_time {
                        info!("Removing old backup file: {:?}", path);
                        fs::remove_file(&path)?;
                    }
                }
            }
        }

        Ok(())
    }

    /// 获取备份列表
    pub async fn list_backups(&self, pool: &PgPool) -> Result<Vec<BackupInfo>, BackupError> {
        let rows = sqlx::query(
            r#"
            SELECT id, backup_time, backup_type, backup_file_path, 
                   backup_size_bytes, checksum, status, error_message
            FROM db_backup_info
            ORDER BY backup_time DESC
            "#,
        )
        .fetch_all(pool)
        .await?;

        let mut backups = Vec::new();
        for row in rows {
            let status_str: String = row.get(6);
            let error_msg: Option<String> = row.get(7);

            let status = match status_str.as_str() {
                "running" => BackupStatus::Running,
                "completed" => BackupStatus::Completed,
                "failed" => BackupStatus::Failed(error_msg.unwrap_or_default()),
                _ => BackupStatus::Failed("Unknown status".to_string()),
            };

            backups.push(BackupInfo {
                id: row.get(0),
                backup_time: row.get(1),
                backup_type: row.get(2),
                file_path: PathBuf::from(row.get::<String, _>(3)),
                size_bytes: row.get::<i64, _>(4) as u64,
                checksum: row.get(5),
                status,
            });
        }

        Ok(backups)
    }
}

/// 备份错误类型
#[derive(Debug)]
pub enum BackupError {
    IoError(std::io::Error),
    SqlxError(sqlx::Error),
    PgDumpError(String),
    CompressionError(String),
    DecompressionError(String),
    RestoreError(String),
    FileNotFound(PathBuf),
    ChecksumMismatch,
}

impl std::fmt::Display for BackupError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BackupError::IoError(e) => write!(f, "IO error: {}", e),
            BackupError::SqlxError(e) => write!(f, "Database error: {}", e),
            BackupError::PgDumpError(msg) => write!(f, "pg_dump error: {}", msg),
            BackupError::CompressionError(msg) => write!(f, "Compression error: {}", msg),
            BackupError::DecompressionError(msg) => write!(f, "Decompression error: {}", msg),
            BackupError::RestoreError(msg) => write!(f, "Restore error: {}", msg),
            BackupError::FileNotFound(path) => write!(f, "File not found: {:?}", path),
            BackupError::ChecksumMismatch => write!(f, "Checksum mismatch"),
        }
    }
}

impl std::error::Error for BackupError {}

impl From<std::io::Error> for BackupError {
    fn from(e: std::io::Error) -> Self {
        BackupError::IoError(e)
    }
}

impl From<sqlx::Error> for BackupError {
    fn from(e: sqlx::Error) -> Self {
        BackupError::SqlxError(e)
    }
}
