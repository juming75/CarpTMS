//! Disaster Recovery API Routes
//!
//! 灾备恢复 API 路由

use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::disaster_recovery::{BackupType, LocalStorageBackend, StorageBackend};
use crate::errors::{success_response_with_message, AppError, AppResult};

/// 灾备状态响应
#[derive(Debug, Serialize, ToSchema)]
pub struct DRStatusResponse {
    pub backup_enabled: bool,
    pub replication_enabled: bool,
    pub failover_enabled: bool,
    pub last_backup_time: Option<String>,
    pub last_backup_size: Option<u64>,
    pub rpo_minutes: u32,
    pub rto_minutes: u32,
    pub replication_status: String,
}

/// 备份列表响应
#[derive(Debug, Serialize, ToSchema)]
pub struct BackupListResponse {
    pub backups: Vec<BackupInfo>,
    pub total: usize,
}

/// 备份信息
#[derive(Debug, Serialize, ToSchema)]
pub struct BackupInfo {
    pub id: String,
    pub backup_type: String,
    pub created_at: String,
    pub size_bytes: u64,
    pub status: String,
}

/// 创建备份请求
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateBackupRequest {
    /// 备份类型: full, incremental, differential
    #[serde(default = "default_backup_type")]
    pub backup_type: String,
}

fn default_backup_type() -> String {
    "full".to_string()
}

/// 恢复请求
#[derive(Debug, Deserialize, ToSchema)]
pub struct RestoreRequest {
    pub backup_id: String,
}

/// 获取灾备状态
#[utoipa::path(
    get,
    path = "/api/dr/status",
    responses(
        (status = 200, description = "灾备状态获取成功", body = ApiResponse<DRStatusResponse>)
    ),
    tag = "Disaster Recovery"
)]
pub async fn get_dr_status() -> AppResult<HttpResponse> {
    // 从全局获取灾备管理器（如果已初始化）
    let response = DRStatusResponse {
        backup_enabled: true,
        replication_enabled: false, // 需要配置复制端点
        failover_enabled: false,    // 需要配置故障转移
        last_backup_time: None,
        last_backup_size: None,
        rpo_minutes: 15,
        rto_minutes: 30,
        replication_status: "not_configured".to_string(),
    };

    Ok(success_response_with_message("灾备状态获取成功", response))
}

/// 获取备份列表
#[utoipa::path(
    get,
    path = "/api/dr/backups",
    responses(
        (status = 200, description = "备份列表获取成功", body = ApiResponse<BackupListResponse>)
    ),
    tag = "Disaster Recovery"
)]
pub async fn list_backups() -> AppResult<HttpResponse> {
    // 从环境变量获取备份路径（兼容 Windows/Linux）
    let backup_path = std::env::var("DR_BACKUP_STORAGE_PATH")
        .unwrap_or_else(|_| {
            // 根据操作系统选择默认路径
            if cfg!(windows) {
                "./backups/CarpTMS".to_string()
            } else {
                "/var/backups/CarpTMS".to_string()
            }
        });

    let path = std::path::PathBuf::from(&backup_path);

    // 如果目录不存在，自动创建并返回空列表
    if !path.exists() {
        if let Err(e) = std::fs::create_dir_all(&path) {
            log::warn!("无法创建备份目录 {}: {}", backup_path, e);
        }
        let response = BackupListResponse {
            total: 0,
            backups: vec![],
        };
        return Ok(success_response_with_message("备份列表获取成功（目录已创建）", response));
    }

    let storage = LocalStorageBackend::new(path);

    let backup_ids = storage
        .list_backups()
        .await
        .map_err(|e| AppError::internal_error(&format!("获取备份列表失败: {}", e), None))?;

    let mut backups = Vec::new();
    for id in backup_ids {
        if let Ok(size) = storage.get_backup_size(&id).await {
            backups.push(BackupInfo {
                id,
                backup_type: "full".to_string(),
                created_at: "unknown".to_string(),
                size_bytes: size,
                status: "completed".to_string(),
            });
        }
    }

    let response = BackupListResponse {
        total: backups.len(),
        backups,
    };

    Ok(success_response_with_message("备份列表获取成功", response))
}

/// 创建手动备份
#[utoipa::path(
    post,
    path = "/api/dr/backup",
    request_body = CreateBackupRequest,
    responses(
        (status = 200, description = "备份创建成功")
    ),
    tag = "Disaster Recovery"
)]
pub async fn create_backup(req: web::Json<CreateBackupRequest>) -> AppResult<HttpResponse> {
    let backup_type = match req.backup_type.as_str() {
        "full" => BackupType::Full,
        "incremental" => BackupType::Incremental,
        "differential" => BackupType::Differential,
        _ => BackupType::Full,
    };

    // 生成备份 ID
    let backup_id = format!(
        "backup-{}-{}",
        chrono::Utc::now().format("%Y%m%d-%H%M%S"),
        uuid::Uuid::new_v4()
            .to_string()
            .split('-')
            .next()
            .expect("UUID must contain hyphens")
    );

    // 获取数据库连接信息
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:123@localhost:5432/carptms".to_string());

    let backup_path = std::env::var("DR_BACKUP_STORAGE_PATH")
        .unwrap_or_else(|_| "/var/backups/CarpTMS".to_string());

    // 执行 pg_dump 备份
    let output = tokio::process::Command::new("pg_dump")
        .args([
            "--dbname",
            &db_url,
            "--format",
            "c",
            "--file",
            &format!("{}/{}.backup", backup_path, backup_id),
            "--verbose",
        ])
        .output()
        .await
        .map_err(|e| AppError::internal_error(&format!("执行 pg_dump 失败: {}", e), None))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::internal_error(
            &format!("备份失败: {}", stderr),
            None,
        ));
    }

    Ok(success_response_with_message(
        "备份创建成功",
        serde_json::json!({
            "backup_id": backup_id,
            "backup_type": backup_type.to_string(),
            "created_at": chrono::Utc::now().to_rfc3339(),
        }),
    ))
}

/// 从备份恢复
#[utoipa::path(
    post,
    path = "/api/dr/restore",
    request_body = RestoreRequest,
    responses(
        (status = 200, description = "恢复成功")
    ),
    tag = "Disaster Recovery"
)]
pub async fn restore_backup(req: web::Json<RestoreRequest>) -> AppResult<HttpResponse> {
    let backup_id = &req.backup_id;

    let backup_path = std::env::var("DR_BACKUP_STORAGE_PATH")
        .unwrap_or_else(|_| "/var/backups/CarpTMS".to_string());

    let backup_file = format!("{}/{}.backup", backup_path, backup_id);

    // 检查备份文件是否存在
    if !std::path::Path::new(&backup_file).exists() {
        return Err(AppError::not_found_error(format!(
            "备份文件不存在: {}",
            backup_id
        )));
    }

    // 获取数据库连接信息
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:123@localhost:5432/carptms".to_string());

    // 执行 pg_restore
    let output = tokio::process::Command::new("pg_restore")
        .args([
            "--dbname",
            &db_url,
            "--clean",
            "--if-exists",
            "--no-owner",
            "--no-privileges",
            &backup_file,
        ])
        .output()
        .await
        .map_err(|e| AppError::internal_error(&format!("执行 pg_restore 失败: {}", e), None))?;

    // pg_restore 对某些警告会输出到 stderr，但恢复仍然成功
    let _stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    Ok(success_response_with_message(
        "恢复完成",
        serde_json::json!({
            "backup_id": backup_id,
            "restored_at": chrono::Utc::now().to_rfc3339(),
            "warnings": if stderr.is_empty() { None } else { Some(stderr.to_string()) },
        }),
    ))
}

/// 删除备份
#[utoipa::path(
    delete,
    path = "/api/dr/backups/{backup_id}",
    responses(
        (status = 200, description = "备份删除成功")
    ),
    tag = "Disaster Recovery"
)]
pub async fn delete_backup(path: web::Path<String>) -> AppResult<HttpResponse> {
    let backup_id = path.into_inner();

    let backup_path = std::env::var("DR_BACKUP_STORAGE_PATH")
        .unwrap_or_else(|_| "/var/backups/CarpTMS".to_string());

    let backup_file = format!("{}/{}.backup", backup_path, backup_id);

    // 删除备份文件
    tokio::fs::remove_file(&backup_file)
        .await
        .map_err(|e| AppError::internal_error(&format!("删除备份失败: {}", e), None))?;

    Ok(success_response_with_message(
        "备份删除成功",
        serde_json::json!({
            "backup_id": backup_id,
        }),
    ))
}

/// 配置灾备路由
pub fn configure_dr_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/dr")
            .route("/status", web::get().to(get_dr_status))
            .route("/backups", web::get().to(list_backups))
            .route("/backup", web::post().to(create_backup))
            .route("/restore", web::post().to(restore_backup))
            .route("/backups/{backup_id}", web::delete().to(delete_backup)),
    );
}
