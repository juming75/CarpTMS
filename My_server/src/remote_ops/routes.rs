//! 远程运维 API 路由
//!
//! 提供服务器管理、命令执行、文件管理、Ansible Playbook 执行等 API

use actix_web::{web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::errors::{AppError, AppResult};
use crate::remote_ops::models::*;
use crate::remote_ops::ssh::{SshConfig, SshConnection, SshConnectionManager};
use crate::remote_ops::websocket;

/// 获取模拟用户 ID（生产环境应从 JWT 解析）
fn current_user_id() -> Uuid {
    Uuid::nil()
}

/// 获取模拟组织 ID
fn current_org_id() -> Uuid {
    Uuid::nil()
}

lazy_static::lazy_static! {
    static ref SSH_MANAGER: SshConnectionManager = SshConnectionManager::new();
}

fn ok_json<T: Serialize>(data: T) -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({ "code": 200, "message": "success", "data": data }))
}

fn err_msg(code: u16, msg: &str) -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({ "code": code, "message": msg }))
}

// ═══════════ 服务器管理 ═══════════

pub async fn create_server(
    pool: web::Data<sqlx::PgPool>,
    body: web::Json<CreateServerRequest>,
) -> AppResult<HttpResponse> {
    let user_id = current_user_id();
    let org_id = current_org_id();
    let server = crate::remote_ops::db::create_server(pool.get_ref(), body.into_inner(), user_id, org_id).await?;
    Ok(ok_json(ServerResponse::from(server)))
}

pub async fn list_servers(
    pool: web::Data<sqlx::PgPool>,
    query: web::Query<ServerListQuery>,
) -> AppResult<HttpResponse> {
    let org_id = current_org_id();
    let result = crate::remote_ops::db::list_servers(pool.get_ref(), query.into_inner(), org_id).await?;
    Ok(ok_json(result))
}

pub async fn get_server(
    pool: web::Data<sqlx::PgPool>,
    path: web::Path<Uuid>,
) -> AppResult<HttpResponse> {
    let server = crate::remote_ops::db::get_server(pool.get_ref(), path.into_inner()).await?;
    match server {
        Some(s) => Ok(ok_json(ServerResponse::from(s))),
        None => Ok(err_msg(404, "服务器不存在")),
    }
}

pub async fn update_server(
    pool: web::Data<sqlx::PgPool>,
    path: web::Path<Uuid>,
    body: web::Json<UpdateServerRequest>,
) -> AppResult<HttpResponse> {
    let server = crate::remote_ops::db::update_server(pool.get_ref(), path.into_inner(), body.into_inner()).await?;
    match server {
        Some(s) => Ok(ok_json(ServerResponse::from(s))),
        None => Ok(err_msg(404, "服务器不存在")),
    }
}

pub async fn delete_server(
    pool: web::Data<sqlx::PgPool>,
    path: web::Path<Uuid>,
) -> AppResult<HttpResponse> {
    crate::remote_ops::db::delete_server(pool.get_ref(), path.into_inner()).await?;
    Ok(ok_json("删除成功"))
}

// ═══════════ 连接测试 ═══════════

pub async fn test_connection(
    body: web::Json<TestConnectionRequest>,
) -> AppResult<HttpResponse> {
    let start = std::time::Instant::now();
    let config = SshConfig {
        host: body.host.clone(),
        port: body.port.unwrap_or(22) as u16,
        username: body.username.clone(),
        password: body.password.clone(),
        private_key: body.private_key.clone(),
        private_key_passphrase: body.private_key_passphrase.clone(),
        timeout_secs: 10,
    };

    let conn = SshConnection {
        session_id: Uuid::new_v4(),
        server_id: Uuid::nil(),
        config,
        created_at: chrono::Utc::now(),
    };

    match conn.execute_command("uname -a", 10).await {
        Ok((stdout, _stderr, exit_code)) => {
            let duration_ms = start.elapsed().as_millis() as u64;
            Ok(ok_json(TestConnectionResult {
                success: exit_code == Some(0),
                message: if exit_code == Some(0) { "连接成功".into() } else { "命令执行异常".into() },
                duration_ms,
                os_type: Some(stdout.trim().to_string()),
                hostname: None,
            }))
        }
        Err(e) => {
            let duration_ms = start.elapsed().as_millis() as u64;
            Ok(ok_json(TestConnectionResult {
                success: false,
                message: format!("连接失败: {}", e),
                duration_ms,
                os_type: None,
                hostname: None,
            }))
        }
    }
}

// ═══════════ 命令执行 ═══════════

pub async fn execute_command(
    pool: web::Data<sqlx::PgPool>,
    path: web::Path<Uuid>,
    body: web::Json<ExecuteCommandRequest>,
) -> AppResult<HttpResponse> {
    let server_id = path.into_inner();
    let server = crate::remote_ops::db::get_server(pool.get_ref(), server_id).await?
        .ok_or_else(|| AppError::resource_not_found("服务器不存在"))?;

    let config = SshConfig {
        host: server.host.clone(),
        port: server.port as u16,
        username: server.username.clone(),
        password: server.password.clone(),
        private_key: server.private_key.clone(),
        private_key_passphrase: server.private_key_passphrase.clone(),
        timeout_secs: body.timeout_secs.unwrap_or(30),
    };

    let start = std::time::Instant::now();
    let (stdout, stderr, exit_code) = SSH_MANAGER.execute(
        server_id, config, &body.command, body.timeout_secs.unwrap_or(30),
    ).await.map_err(|e| AppError::internal_error(&format!("命令执行失败: {}", e), None))?;
    let duration_ms = start.elapsed().as_millis() as u64;

    Ok(ok_json(CommandResult {
        id: Uuid::new_v4(),
        server_id,
        server_name: Some(server.name),
        command: body.command.clone(),
        stdout,
        stderr,
        exit_code,
        status: if exit_code == Some(0) { CommandStatus::Success } else { CommandStatus::Failed },
        duration_ms,
        executed_by: current_user_id(),
        executed_at: chrono::Utc::now(),
    }))
}

pub async fn batch_execute(
    pool: web::Data<sqlx::PgPool>,
    body: web::Json<BatchExecuteRequest>,
) -> AppResult<HttpResponse> {
    let mut results = Vec::new();
    for server_id in &body.server_ids {
        let server = match crate::remote_ops::db::get_server(pool.get_ref(), *server_id).await? {
            Some(s) => s,
            None => continue,
        };
        let config = SshConfig {
            host: server.host.clone(),
            port: server.port as u16,
            username: server.username.clone(),
            password: server.password.clone(),
            private_key: server.private_key.clone(),
            private_key_passphrase: server.private_key_passphrase.clone(),
            timeout_secs: body.timeout_secs.unwrap_or(30),
        };

        let start = std::time::Instant::now();
        let result = SSH_MANAGER.execute(*server_id, config, &body.command, body.timeout_secs.unwrap_or(30)).await;
        let duration_ms = start.elapsed().as_millis() as u64;

        match result {
            Ok((stdout, stderr, exit_code)) => {
                results.push(ServerCommandResult {
                    server_id: *server_id,
                    server_name: Some(server.name),
                    host: server.host,
                    success: exit_code == Some(0),
                    command: body.command.clone(),
                    stdout,
                    stderr,
                    exit_code,
                    duration_ms,
                });
            }
            Err(e) => {
                results.push(ServerCommandResult {
                    server_id: *server_id,
                    server_name: Some(server.name),
                    host: server.host,
                    success: false,
                    command: body.command.clone(),
                    stdout: String::new(),
                    stderr: format!("{}", e),
                    exit_code: Some(-1),
                    duration_ms,
                });
            }
        }
    }

    let success = results.iter().filter(|r| r.success).count();
    let failed = results.len() - success;
    Ok(ok_json(BatchExecuteResult {
        total: results.len(),
        success,
        failed,
        results,
    }))
}

pub async fn get_command_history(
    pool: web::Data<sqlx::PgPool>,
    path: web::Path<Uuid>,
) -> AppResult<HttpResponse> {
    let history = crate::remote_ops::db::get_command_history(pool.get_ref(), path.into_inner()).await?;
    Ok(ok_json(history))
}

// ═══════════ 服务器组管理 ═══════════

pub async fn list_groups(
    pool: web::Data<sqlx::PgPool>,
) -> AppResult<HttpResponse> {
    let groups = crate::remote_ops::db::list_groups(pool.get_ref(), current_org_id()).await?;
    Ok(ok_json(groups))
}

pub async fn create_group(
    pool: web::Data<sqlx::PgPool>,
    body: web::Json<CreateServerGroupRequest>,
) -> AppResult<HttpResponse> {
    let group = crate::remote_ops::db::create_group(pool.get_ref(), body.into_inner(), current_org_id()).await?;
    Ok(ok_json(group))
}

pub async fn update_group(
    pool: web::Data<sqlx::PgPool>,
    path: web::Path<Uuid>,
    body: web::Json<UpdateServerGroupRequest>,
) -> AppResult<HttpResponse> {
    let group = crate::remote_ops::db::update_group(pool.get_ref(), path.into_inner(), body.into_inner()).await?;
    match group {
        Some(g) => Ok(ok_json(g)),
        None => Ok(err_msg(404, "服务器组不存在")),
    }
}

pub async fn delete_group(
    pool: web::Data<sqlx::PgPool>,
    path: web::Path<Uuid>,
) -> AppResult<HttpResponse> {
    crate::remote_ops::db::delete_group(pool.get_ref(), path.into_inner()).await?;
    Ok(ok_json("删除成功"))
}

// ═══════════ 服务器指标 ═══════════

pub async fn get_server_metrics(
    pool: web::Data<sqlx::PgPool>,
    path: web::Path<Uuid>,
) -> AppResult<HttpResponse> {
    let server_id = path.into_inner();
    let server = crate::remote_ops::db::get_server(pool.get_ref(), server_id).await?
        .ok_or_else(|| AppError::resource_not_found("服务器不存在"))?;

    let config = SshConfig {
        host: server.host.clone(),
        port: server.port as u16,
        username: server.username.clone(),
        password: server.password.clone(),
        private_key: server.private_key.clone(),
        private_key_passphrase: server.private_key_passphrase.clone(),
        timeout_secs: 15,
    };

    let metrics_cmd = r#"echo "{\"cpu\":$(top -bn1 | grep 'Cpu(s)' | awk '{print $2+$4}'),\"mem\":$(free | grep Mem | awk '{print $3/$2 * 100.0}'),\"disk\":$(df / | tail -1 | awk '{print $5}' | sed 's/%//'),\"uptime\":$(cat /proc/uptime | awk '{print $1}'),\"load\":$(cat /proc/loadavg | awk '{print $1,$2,$3}'),\"procs\":$(ps aux | wc -l)}""#;

    let conn = SshConnection {
        session_id: Uuid::new_v4(),
        server_id,
        config,
        created_at: chrono::Utc::now(),
    };

    let metrics = match conn.execute_command(metrics_cmd, 15).await {
        Ok((stdout, _stderr, _exit)) => {
            serde_json::from_str::<ServerMetrics>(&stdout).unwrap_or_default()
        }
        Err(_) => ServerMetrics::default(),
    };

    Ok(ok_json(metrics))
}

// ═══════════ 文件操作 ═══════════

pub async fn list_directory(
    pool: web::Data<sqlx::PgPool>,
    body: web::Json<ListDirectoryRequest>,
) -> AppResult<HttpResponse> {
    let server = crate::remote_ops::db::get_server(pool.get_ref(), body.server_id).await?
        .ok_or_else(|| AppError::resource_not_found("服务器不存在"))?;

    let config = SshConfig {
        host: server.host, port: server.port as u16,
        username: server.username, password: server.password,
        private_key: server.private_key, private_key_passphrase: server.private_key_passphrase,
        timeout_secs: 10,
    };

    let cmd = format!("ls -la '{}'", body.path.replace('\'', "'\\''"));

    let conn = SshConnection {
        session_id: Uuid::new_v4(),
        server_id: body.server_id,
        config,
        created_at: chrono::Utc::now(),
    };

    let (stdout, _stderr, _exit) = conn.execute_command(&cmd, 10).await
        .map_err(|e| AppError::internal_error(&format!("列出目录失败: {}", e), None))?;
    let files: Vec<FileInfo> = stdout.lines().skip(1).filter_map(|line| {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 9 { return None; }
        Some(FileInfo {
            name: parts.iter().skip(8).cloned().collect::<Vec<_>>().join(" "),
            path: format!("{}/{}", body.path, parts.iter().skip(8).cloned().collect::<Vec<_>>().join(" ")),
            file_type: if parts[0].starts_with('d') { "directory".into() } else { "file".into() },
            size: parts[4].parse().unwrap_or(0),
            permissions: parts[0].to_string(),
            modified_at: Some(format!("{} {} {}", parts[5], parts[6], parts[7])),
        })
    }).collect();

    Ok(ok_json(files))
}

pub async fn read_file(
    pool: web::Data<sqlx::PgPool>,
    body: web::Json<FileTransferRequest>,
) -> AppResult<HttpResponse> {
    let server = crate::remote_ops::db::get_server(pool.get_ref(), body.server_id).await?
        .ok_or_else(|| AppError::resource_not_found("服务器不存在"))?;

    let config = SshConfig {
        host: server.host, port: server.port as u16,
        username: server.username, password: server.password,
        private_key: server.private_key, private_key_passphrase: server.private_key_passphrase,
        timeout_secs: 10,
    };

    let cmd = format!("cat '{}'", body.remote_path.replace('\'', "'\\''"));
    let conn = SshConnection {
        session_id: Uuid::new_v4(),
        server_id: body.server_id,
        config,
        created_at: chrono::Utc::now(),
    };

    let (stdout, _stderr, _exit) = conn.execute_command(&cmd, 10).await?;
    Ok(ok_json(serde_json::json!({ "content": stdout })))
}

// ═══════════ 路由配置 ═══════════

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/remote-ops")
            // 服务器管理
            .route("/servers", web::get().to(list_servers))
            .route("/servers", web::post().to(create_server))
            .route("/servers/test-connection", web::post().to(test_connection))
            .route("/servers/{id}", web::get().to(get_server))
            .route("/servers/{id}", web::put().to(update_server))
            .route("/servers/{id}", web::delete().to(delete_server))
            // 命令执行
            .route("/servers/{id}/execute", web::post().to(execute_command))
            .route("/servers/{id}/history", web::get().to(get_command_history))
            .route("/servers/{id}/metrics", web::get().to(get_server_metrics))
            // 文件操作
            .route("/files/list", web::post().to(list_directory))
            .route("/files/read", web::post().to(read_file))
            // 批量操作
            .route("/commands/batch", web::post().to(batch_execute))
            // 服务器组
            .route("/groups", web::get().to(list_groups))
            .route("/groups", web::post().to(create_group))
            .route("/groups/{id}", web::put().to(update_group))
            .route("/groups/{id}", web::delete().to(delete_group))
            // 状态
            .route("/status", web::get().to(remote_ops_status)),
    );
}

/// 远程运维状态
async fn remote_ops_status() -> AppResult<HttpResponse> {
    Ok(ok_json(serde_json::json!({
        "status": "running",
        "version": env!("CARGO_PKG_VERSION"),
        "features": ["ssh", "file_management", "command_execution", "batch"],
    })))
}
