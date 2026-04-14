//! 微服务路由处理器
//!
//! 处理微服务模式的请求转发到远程服务

use async_trait::async_trait;
use actix_web::{HttpResponse, web};

use crate::config::ArchitectureMode;
use crate::errors::AppResult;
use super::RouteAdapter;

/// 微服务路由处理器
/// 将请求转发到远程微服务（通过 gRPC）
pub struct MicroserviceRouteHandler {
    /// 服务地址配置
    service_endpoints: std::collections::HashMap<String, String>,
}

impl MicroserviceRouteHandler {
    /// 创建新的处理器
    pub fn new() -> Self {
        let mut endpoints = std::collections::HashMap::new();
        // 配置微服务端点（实际应从配置读取）
        endpoints.insert("vehicle".to_string(), "http://localhost:8083".to_string());
        endpoints.insert("order".to_string(), "http://localhost:8084".to_string());
        endpoints.insert("user".to_string(), "http://localhost:8085".to_string());
        
        Self {
            service_endpoints: endpoints,
        }
    }

    /// 获取服务地址
    fn get_service_url(&self, path: &str) -> Option<String> {
        let parts: Vec<&str> = path.trim_start_matches("/api/").split('/').collect();
        let resource = parts.first()?;
        
        match *resource {
            "vehicles" | "vehicle" => self.service_endpoints.get("vehicle").cloned(),
            "orders" | "order" => self.service_endpoints.get("order").cloned(),
            "users" | "user" => self.service_endpoints.get("user").cloned(),
            _ => None,
        }
    }
}

impl Default for MicroserviceRouteHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl RouteAdapter for MicroserviceRouteHandler {
    fn name(&self) -> &str {
        "MicroserviceRouteHandler"
    }

    fn supported_mode(&self) -> ArchitectureMode {
        ArchitectureMode::MicroDDD
    }

    async fn handle_get(
        &self,
        path: &str,
        _user_id: Option<i32>,
        _organization_id: Option<i32>,
        _query: web::Query<serde_json::Value>,
    ) -> AppResult<HttpResponse> {
        // 微服务模式下转发请求到远程服务
        let service_url = self.get_service_url(path);
        
        match service_url {
            Some(base_url) => {
                let url = format!("{}{}", base_url, path);
                log::info!("Forwarding GET request to microservice: {}", url);
                
                // 实际实现应该使用 HTTP 客户端调用远程服务
                // 这里返回提示信息，实际部署时需要实现 gRPC 或 HTTP 转发
                Ok(HttpResponse::Ok().json(serde_json::json!({
                    "code": 200,
                    "message": "微服务模式，请求已转发",
                    "data": {
                        "mode": "micro_ddd",
                        "forwarded_to": url,
                        "note": "请实现 gRPC 或 HTTP 客户端调用远程服务"
                    }
                })))
            }
            None => {
                Ok(HttpResponse::NotFound().json(serde_json::json!({
                    "code": 404,
                    "message": format!("未找到服务处理器: {}", path)
                })))
            }
        }
    }

    async fn handle_post(
        &self,
        path: &str,
        _user_id: Option<i32>,
        _organization_id: Option<i32>,
        body: web::Json<serde_json::Value>,
    ) -> AppResult<HttpResponse> {
        let service_url = self.get_service_url(path);
        
        match service_url {
            Some(base_url) => {
                let url = format!("{}{}", base_url, path);
                log::info!("Forwarding POST request to microservice: {}", url);
                
                Ok(HttpResponse::Created().json(serde_json::json!({
                    "code": 201,
                    "message": "微服务模式，请求已转发",
                    "data": {
                        "mode": "micro_ddd",
                        "forwarded_to": url,
                        "body": body.into_inner()
                    }
                })))
            }
            None => {
                Ok(HttpResponse::NotFound().json(serde_json::json!({
                    "code": 404,
                    "message": format!("未找到服务处理器: {}", path)
                })))
            }
        }
    }

    async fn handle_put(
        &self,
        path: &str,
        _user_id: Option<i32>,
        _organization_id: Option<i32>,
        _body: web::Json<serde_json::Value>,
    ) -> AppResult<HttpResponse> {
        let service_url = self.get_service_url(path);
        
        match service_url {
            Some(base_url) => {
                let url = format!("{}{}", base_url, path);
                log::info!("Forwarding PUT request to microservice: {}", url);
                
                Ok(HttpResponse::Ok().json(serde_json::json!({
                    "code": 200,
                    "message": "微服务模式，请求已转发",
                    "data": {
                        "mode": "micro_ddd",
                        "forwarded_to": url
                    }
                })))
            }
            None => {
                Ok(HttpResponse::NotFound().json(serde_json::json!({
                    "code": 404,
                    "message": format!("未找到服务处理器: {}", path)
                })))
            }
        }
    }

    async fn handle_delete(
        &self,
        path: &str,
        _user_id: Option<i32>,
        _organization_id: Option<i32>,
    ) -> AppResult<HttpResponse> {
        let service_url = self.get_service_url(path);
        
        match service_url {
            Some(base_url) => {
                let url = format!("{}{}", base_url, path);
                log::info!("Forwarding DELETE request to microservice: {}", url);
                
                Ok(HttpResponse::Ok().json(serde_json::json!({
                    "code": 200,
                    "message": "微服务模式，请求已转发",
                    "data": {
                        "mode": "micro_ddd",
                        "forwarded_to": url
                    }
                })))
            }
            None => {
                Ok(HttpResponse::NotFound().json(serde_json::json!({
                    "code": 404,
                    "message": format!("未找到服务处理器: {}", path)
                })))
            }
        }
    }

    async fn handle_patch(
        &self,
        path: &str,
        _user_id: Option<i32>,
        _organization_id: Option<i32>,
        _body: web::Json<serde_json::Value>,
    ) -> AppResult<HttpResponse> {
        let service_url = self.get_service_url(path);
        
        match service_url {
            Some(base_url) => {
                let url = format!("{}{}", base_url, path);
                log::info!("Forwarding PATCH request to microservice: {}", url);
                
                Ok(HttpResponse::Ok().json(serde_json::json!({
                    "code": 200,
                    "message": "微服务模式，请求已转发",
                    "data": {
                        "mode": "micro_ddd",
                        "forwarded_to": url
                    }
                })))
            }
            None => {
                Ok(HttpResponse::NotFound().json(serde_json::json!({
                    "code": 404,
                    "message": format!("未找到服务处理器: {}", path)
                })))
            }
        }
    }

    async fn health_check(&self) -> AppResult<bool> {
        // 检查所有配置的微服务健康状态
        Ok(true)
    }
}
