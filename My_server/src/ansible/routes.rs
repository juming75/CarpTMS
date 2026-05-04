// Ansible API 路由
// 提供 RESTful API 接口

use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

use super::{AnsibleExecutor, models::*};

/// 应用状态
pub struct AppState {
    pub executor: Arc<RwLock<AnsibleExecutor>>,
}

/// 初始化 Ansible 路由配置
pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    let ansible_dir = std::env::var("ANSIBLE_DIR")
        .unwrap_or_else(|_| "../ansible-ops".to_string());
    
    let executor = AnsibleExecutor::new(ansible_dir);
    let state = web::Data::new(AppState {
        executor: Arc::new(RwLock::new(executor)),
    });
    
    cfg.app_data(state)
        .route("/api/ansible/health", web::get().to(health_check))
        .route("/api/ansible/playbooks", web::get().to(list_playbooks))
        .route("/api/ansible/playbook/execute", web::post().to(execute_playbook))
        .route("/api/ansible/playbook/{id}/status", web::get().to(get_execution_status))
        .route("/api/ansible/hosts", web::get().to(list_hosts))
        .route("/api/ansible/groups", web::get().to(list_groups))
        .route("/api/ansible/command", web::post().to(execute_command))
        .route("/api/ansible/ping", web::post().to(ping_hosts))
        .route("/api/ansible/inventory", web::get().to(get_inventory))
        .route("/api/ansible/history", web::get().to(get_execution_history));
}

/// 健康检查
async fn health_check(state: web::Data<AppState>) -> impl Responder {
    let executor = state.executor.read().await;
    match executor.check_availability().await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "status": "healthy",
            "message": "Ansible 服务正常"
        })),
        Err(e) => HttpResponse::ServiceUnavailable().json(serde_json::json!({
            "status": "unhealthy",
            "message": e
        })),
    }
}

/// 列出可用的 Playbooks
async fn list_playbooks() -> impl Responder {
    let playbooks = vec![
        PlaybookInfo {
            name: "system-init".to_string(),
            path: "playbooks/system/system-init.yaml".to_string(),
            description: "系统初始化配置".to_string(),
            category: "system".to_string(),
        },
        PlaybookInfo {
            name: "health-check".to_string(),
            path: "playbooks/system/health-check.yaml".to_string(),
            description: "服务器健康检查".to_string(),
            category: "system".to_string(),
        },
        PlaybookInfo {
            name: "deploy".to_string(),
            path: "playbooks/application/deploy.yaml".to_string(),
            description: "应用部署".to_string(),
            category: "application".to_string(),
        },
    ];
    
    HttpResponse::Ok().json(serde_json::json!({
        "code": 200,
        "data": playbooks
    }))
}

/// Playbook 信息
#[derive(Serialize)]
struct PlaybookInfo {
    name: String,
    path: String,
    description: String,
    category: String,
}

/// 执行 Playbook 请求
#[derive(Deserialize)]
pub struct ExecutePlaybookRequest {
    pub playbook: String,
    pub inventory: String,
    pub extra_vars: Option<serde_json::Value>,
    pub limit: Option<String>,
    pub check_mode: Option<bool>,
    pub tags: Option<Vec<String>>,
}

/// 执行 Playbook
async fn execute_playbook(
    state: web::Data<AppState>,
    body: web::Json<ExecutePlaybookRequest>,
) -> impl Responder {
    let executor = state.executor.read().await;
    
    let request = PlaybookRequest {
        playbook: body.playbook.clone(),
        inventory: body.inventory.clone(),
        extra_vars: body.extra_vars.clone(),
        limit: body.limit.clone(),
        check_mode: body.check_mode.unwrap_or(false),
        tags: body.tags.clone(),
    };
    
    match executor.run_playbook(&request).await {
        Ok(result) => HttpResponse::Ok().json(serde_json::json!({
            "code": 200,
            "data": result
        })),
        Err(e) => {
            log::error!("执行 Playbook 失败: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "code": 500,
                "message": e
            }))
        }
    }
}

/// 获取执行状态
async fn get_execution_status(
    path: web::Path<String>,
) -> impl Responder {
    let execution_id = path.into_inner();
    
    // TODO: 从数据库或缓存获取执行状态
    HttpResponse::Ok().json(serde_json::json!({
        "code": 200,
        "data": {
            "execution_id": execution_id,
            "status": "completed"
        }
    }))
}

/// 列出主机
async fn list_hosts() -> impl Responder {
    let hosts = vec![
        Host {
            id: "web01".to_string(),
            name: "web01".to_string(),
            ansible_host: "192.168.1.101".to_string(),
            ansible_port: Some(22),
            ansible_user: "ansible".to_string(),
            groups: vec!["web_servers".to_string()],
            variables: serde_json::json!({}),
            status: HostStatus::Online,
        },
        Host {
            id: "db01".to_string(),
            name: "db01".to_string(),
            ansible_host: "192.168.1.201".to_string(),
            ansible_port: Some(22),
            ansible_user: "ansible".to_string(),
            groups: vec!["db_servers".to_string()],
            variables: serde_json::json!({}),
            status: HostStatus::Online,
        },
    ];
    
    HttpResponse::Ok().json(serde_json::json!({
        "code": 200,
        "data": hosts
    }))
}

/// 列出服务器组
async fn list_groups() -> impl Responder {
    let groups = vec![
        ServerGroup {
            id: "web_servers".to_string(),
            name: "Web 服务器组".to_string(),
            hosts: vec![],
            variables: serde_json::json!({}),
        },
        ServerGroup {
            id: "db_servers".to_string(),
            name: "数据库服务器组".to_string(),
            hosts: vec![],
            variables: serde_json::json!({}),
        },
        ServerGroup {
            id: "cache_servers".to_string(),
            name: "缓存服务器组".to_string(),
            hosts: vec![],
            variables: serde_json::json!({}),
        },
    ];
    
    HttpResponse::Ok().json(serde_json::json!({
        "code": 200,
        "data": groups
    }))
}

/// 执行快速命令请求
#[derive(Deserialize)]
pub struct ExecuteCommandRequest {
    pub hosts: String,
    pub module: String,
    pub args: String,
    pub inventory: String,
}

/// 执行快速命令
async fn execute_command(
    state: web::Data<AppState>,
    body: web::Json<ExecuteCommandRequest>,
) -> impl Responder {
    let executor = state.executor.read().await;
    
    let request = QuickCommand {
        hosts: body.hosts.clone(),
        module: body.module.clone(),
        args: body.args.clone(),
        inventory: body.inventory.clone(),
    };
    
    match executor.run_command(&request).await {
        Ok(results) => HttpResponse::Ok().json(serde_json::json!({
            "code": 200,
            "data": results
        })),
        Err(e) => {
            log::error!("执行命令失败: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "code": 500,
                "message": e
            }))
        }
    }
}

/// Ping 主机请求
#[derive(Deserialize)]
pub struct PingRequest {
    pub hosts: String,
    pub inventory: String,
}

/// Ping 主机
async fn ping_hosts(
    state: web::Data<AppState>,
    body: web::Json<PingRequest>,
) -> impl Responder {
    let executor = state.executor.read().await;
    
    match executor.ping(&body.inventory, &body.hosts).await {
        Ok(results) => HttpResponse::Ok().json(serde_json::json!({
            "code": 200,
            "data": results
        })),
        Err(e) => {
            log::error!("Ping 失败: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "code": 500,
                "message": e
            }))
        }
    }
}

/// 获取库存信息
async fn get_inventory() -> impl Responder {
    let inventory = InventorySource {
        id: "prod".to_string(),
        name: "生产环境".to_string(),
        path: "inventory/prod/hosts.yaml".to_string(),
        source_type: InventorySourceType::File,
        last_updated: chrono::Utc::now(),
    };
    
    HttpResponse::Ok().json(serde_json::json!({
        "code": 200,
        "data": inventory
    }))
}

/// 获取执行历史
async fn get_execution_history() -> impl Responder {
    // TODO: 从数据库获取历史记录
    let history: Vec<ExecutionHistory> = vec![];
    
    HttpResponse::Ok().json(serde_json::json!({
        "code": 200,
        "data": history
    }))
}
