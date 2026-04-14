//! /! 协议管理模块
//!
//! 负责协议版本检测、协议自动协商、动态端口分配和协议动态扩展

use actix_web::{web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

// 协议版本信息
#[derive(Debug, Serialize, Deserialize)]
pub struct ProtocolVersion {
    pub version: String,
    pub supported: bool,
    pub features: Vec<String>,
}

// 协议协商请求
#[derive(Debug, Deserialize, Serialize)]
pub struct ProtocolNegotiationRequest {
    pub client_versions: Vec<String>,
    pub client_features: Vec<String>,
}

// 协议协商响应
#[derive(Debug, Serialize, Deserialize)]
pub struct ProtocolNegotiationResponse {
    pub selected_version: String,
    pub supported_features: Vec<String>,
    pub server_features: Vec<String>,
}

// 动态端口分配请求
#[derive(Debug, Deserialize, Serialize)]
pub struct DynamicPortRequest {
    pub protocol: String,
    pub version: String,
    pub required_features: Vec<String>,
}

// 动态端口分配响应
#[derive(Debug, Serialize, Deserialize)]
pub struct DynamicPortResponse {
    pub port: u16,
    pub protocol: String,
    pub version: String,
    pub expires_at: u64, // 过期时间戳
}

// 协议扩展信息
#[derive(Debug, Serialize, Deserialize)]
pub struct ProtocolExtension {
    pub name: String,
    pub version: String,
    pub status: String,
    pub features: Vec<String>,
}

// 协议管理状态
pub struct ProtocolManager {
    pub supported_versions: Arc<RwLock<HashMap<String, ProtocolVersion>>>,
    pub allocated_ports: Arc<RwLock<HashMap<u16, DynamicPortResponse>>>,
    pub extensions: Arc<RwLock<Vec<ProtocolExtension>>>,
    pub next_port: Arc<RwLock<u16>>,
}

impl Default for ProtocolManager {
    fn default() -> Self {
        let mut supported_versions = HashMap::new();
        supported_versions.insert(
            "1.0".to_string(),
            ProtocolVersion {
                version: "1.0".to_string(),
                supported: true,
                features: vec!["basic_auth", "json_api", "websocket"]
                    .into_iter()
                    .map(|s| s.to_string())
                    .collect(),
            },
        );
        supported_versions.insert(
            "2.0".to_string(),
            ProtocolVersion {
                version: "2.0".to_string(),
                supported: true,
                features: vec![
                    "basic_auth",
                    "json_api",
                    "websocket",
                    "streaming",
                    "batch_operations",
                ]
                .into_iter()
                .map(|s| s.to_string())
                .collect(),
            },
        );

        Self {
            supported_versions: Arc::new(RwLock::new(supported_versions)),
            allocated_ports: Arc::new(RwLock::new(HashMap::new())),
            extensions: Arc::new(RwLock::new(vec![
                ProtocolExtension {
                    name: "jt808".to_string(),
                    version: "1.0".to_string(),
                    status: "active".to_string(),
                    features: vec!["gps_tracking", "command_execution"]
                        .into_iter()
                        .map(|s| s.to_string())
                        .collect(),
                },
                ProtocolExtension {
                    name: "gb28181".to_string(),
                    version: "1.0".to_string(),
                    status: "active".to_string(),
                    features: vec!["video_streaming", "device_management"]
                        .into_iter()
                        .map(|s| s.to_string())
                        .collect(),
                },
            ])),
            next_port: Arc::new(RwLock::new(9000)),
        }
    }
}

// 全局协议管理器实例
pub static PROTOCOL_MANAGER: once_cell::sync::Lazy<ProtocolManager> =
    once_cell::sync::Lazy::new(ProtocolManager::default);

// 获取支持的协议版本
pub async fn get_supported_versions() -> HttpResponse {
    let manager = &PROTOCOL_MANAGER;
    let versions = manager.supported_versions.read().await;

    let extensions = manager.extensions.read().await;
    let response = serde_json::json!({
        "success": true,
        "data": {
            "versions": versions.values().collect::<Vec<_>>(),
            "extensions": *extensions
        }
    });
    HttpResponse::Ok().json(response)
}

// 协议自动协商
pub async fn negotiate_protocol(req: web::Json<ProtocolNegotiationRequest>) -> HttpResponse {
    let manager = &PROTOCOL_MANAGER;
    let versions = manager.supported_versions.read().await;

    // 查找客户端支持的最高版本
    let mut selected_version = "1.0".to_string();
    let mut max_version = 0.0;

    for client_version in &req.client_versions {
        if let Ok(version_num) = client_version.parse::<f32>() {
            if version_num > max_version && versions.contains_key(client_version) {
                max_version = version_num;
                selected_version = client_version.clone();
            }
        }
    }

    let selected_proto = match versions.get(&selected_version) {
        Some(proto) => proto,
        None => {
            let response = serde_json::json!({
                "success": false,
                "error": format!("Protocol version '{}' not found", selected_version)
            });
            return HttpResponse::BadRequest().json(response);
        }
    };

    // 计算支持的特性
    let supported_features: Vec<String> = selected_proto
        .features
        .iter()
        .filter(|feature| req.client_features.contains(feature))
        .cloned()
        .collect();

    let response = serde_json::json!({
        "success": true,
        "data": ProtocolNegotiationResponse {
            selected_version,
            supported_features,
            server_features: selected_proto.features.clone()
        }
    });
    HttpResponse::Ok().json(response)
}

// 动态端口分配
pub async fn allocate_port(req: web::Json<DynamicPortRequest>) -> HttpResponse {
    let manager = &PROTOCOL_MANAGER;

    // 检查协议版本是否支持
    let versions = manager.supported_versions.read().await;
    if !versions.contains_key(&req.version) {
        let response = serde_json::json!({
            "success": false,
            "error": "Unsupported protocol version"
        });
        return HttpResponse::BadRequest().json(response);
    }

    // 分配端口
    let mut next_port = manager.next_port.write().await;
    let port = *next_port;
    *next_port += 1;

    // 记录端口分配
    let mut allocated_ports = manager.allocated_ports.write().await;
    let expires_at = (chrono::Utc::now().timestamp() + 3600) as u64; // 1小时过期

    allocated_ports.insert(
        port,
        DynamicPortResponse {
            port,
            protocol: req.protocol.clone(),
            version: req.version.clone(),
            expires_at,
        },
    );

    let response = serde_json::json!({
        "success": true,
        "data": DynamicPortResponse {
            port,
            protocol: req.protocol.clone(),
            version: req.version.clone(),
            expires_at
        }
    });
    HttpResponse::Ok().json(response)
}

// 获取协议扩展列表
pub async fn get_extensions() -> HttpResponse {
    let manager = &PROTOCOL_MANAGER;
    let extensions = manager.extensions.read().await;

    let response = serde_json::json!({
        "success": true,
        "data": *extensions
    });
    HttpResponse::Ok().json(response)
}

// 协议版本检测中间件
pub async fn protocol_version_detect(req: HttpRequest) -> HttpResponse {
    // 从请求头中检测协议版本
    let version = req
        .headers()
        .get("X-Protocol-Version")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("1.0");

    let manager = &PROTOCOL_MANAGER;
    let versions = manager.supported_versions.read().await;

    let supported = versions.contains_key(version);

    let response = serde_json::json!({
        "success": true,
        "data": {
            "detected_version": version,
            "supported": supported,
            "available_versions": versions.keys().collect::<Vec<_>>()
        }
    });
    HttpResponse::Ok().json(response)
}

// 配置协议管理路由
pub fn configure_protocol_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/protocol")
            .route("/versions", web::get().to(get_supported_versions))
            .route("/negotiate", web::post().to(negotiate_protocol))
            .route("/allocate-port", web::post().to(allocate_port))
            .route("/extensions", web::get().to(get_extensions))
            .route("/detect", web::get().to(protocol_version_detect)),
    );
}
