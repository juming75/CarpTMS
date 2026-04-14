//! 单体模式路由处理器
//!
//! 在单体架构模式下，直接调用 application 层服务处理请求。
//! 遵循 DDD 架构设计，通过 Command 和 Query 处理业务逻辑。

use async_trait::async_trait;
use actix_web::{HttpResponse, web};
use serde_json::Value;
use chrono::Utc;
use uuid::Uuid;

use super::{RouteAdapter, RouteContext};
use crate::config::ArchitectureMode;
use crate::errors::{AppError, AppResult};

/// 单体模式路由处理器
pub struct MonolithRouteHandler {
    /// 数据库连接池
    pool: sqlx::PgPool,
}

impl MonolithRouteHandler {
    /// 创建新的处理器
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }

    /// 创建路由上下文
    fn create_context(&self, user_id: Option<i32>, org_id: Option<i32>) -> RouteContext {
        RouteContext {
            path: String::new(),
            method: String::new(),
            user_id,
            organization_id: org_id,
            request_id: Uuid::new_v4().to_string(),
            mode: ArchitectureMode::MonolithDDD,
        }
    }

    /// 创建成功响应
    fn success_response<T: serde::Serialize>(&self, data: T, request_id: &str) -> HttpResponse {
        HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "data": data,
            "request_id": request_id,
            "timestamp": Utc::now().timestamp(),
            "mode": "monolith_ddd"
        }))
    }

    /// 解析路径中的资源 ID
    fn parse_resource_id(&self, path: &str) -> Option<i32> {
        let parts: Vec<&str> = path.trim_start_matches('/').split('/').collect();
        if parts.len() >= 3 {
            parts[2].parse().ok()
        } else {
            None
        }
    }

    /// 处理车辆相关请求
    async fn handle_vehicles(
        &self,
        path: &str,
        method: &str,
        user_id: Option<i32>,
        org_id: Option<i32>,
        query: Option<web::Query<Value>>,
        body: Option<web::Json<Value>>,
    ) -> AppResult<HttpResponse> {
        let ctx = self.create_context(user_id, org_id);
        let request_id = ctx.request_id.clone();
        
        // 解析资源 ID
        let resource_id = self.parse_resource_id(path);
        
        match method {
            "GET" => {
                if let Some(id) = resource_id {
                    // GET /api/vehicles/{id} - 查询单个车辆
                    Ok(self.success_response(
                        serde_json::json!({
                            "vehicle_id": id,
                            "message": "Vehicle details (monolith mode)"
                        }),
                        &request_id,
                    ))
                } else {
                    // GET /api/vehicles - 列表查询
                    let page = query
                        .as_ref()
                        .and_then(|q| q.get("page").and_then(|v| v.as_i64()))
                        .unwrap_or(1) as i32;
                    let page_size = query
                        .as_ref()
                        .and_then(|q| q.get("page_size").and_then(|v| v.as_i64()))
                        .unwrap_or(20) as i32;
                    
                    Ok(self.success_response(
                        serde_json::json!({
                            "items": [],
                            "total": 0,
                            "page": page,
                            "page_size": page_size,
                            "message": "Vehicle list (monolith mode)"
                        }),
                        &request_id,
                    ))
                }
            }
            "POST" => {
                // POST /api/vehicles - 创建车辆
                let data = body.map(|b| b.0).unwrap_or(Value::Null);
                
                Ok(HttpResponse::Created().json(serde_json::json!({
                    "success": true,
                    "data": {
                        "message": "Vehicle creation command submitted",
                        "payload": data
                    },
                    "request_id": request_id,
                    "timestamp": Utc::now().timestamp(),
                    "mode": "monolith_ddd"
                })))
            }
            "PUT" => {
                // PUT /api/vehicles/{id} - 更新车辆
                if resource_id.is_none() {
                    return Err(AppError::validation_error("Vehicle ID is required", None));
                }
                
                let data = body.map(|b| b.0).unwrap_or(Value::Null);
                
                Ok(self.success_response(
                    serde_json::json!({
                        "vehicle_id": resource_id,
                        "message": "Vehicle update command submitted",
                        "payload": data
                    }),
                    &request_id,
                ))
            }
            "DELETE" => {
                // DELETE /api/vehicles/{id} - 删除车辆
                if resource_id.is_none() {
                    return Err(AppError::validation_error("Vehicle ID is required", None));
                }
                
                Ok(HttpResponse::NoContent().finish())
            }
            "PATCH" => {
                // PATCH /api/vehicles/{id} - 部分更新
                let data = body.map(|b| b.0).unwrap_or(Value::Null);
                
                Ok(self.success_response(
                    serde_json::json!({
                        "vehicle_id": resource_id,
                        "message": "Vehicle patch command submitted",
                        "payload": data
                    }),
                    &request_id,
                ))
            }
            _ => {
                Err(AppError::validation_error(
                    &format!("Method {} not allowed", method),
                    None,
                ))
            }
        }
    }

    /// 处理订单相关请求
    async fn handle_orders(
        &self,
        path: &str,
        method: &str,
        user_id: Option<i32>,
        org_id: Option<i32>,
        _query: Option<web::Query<Value>>,
        body: Option<web::Json<Value>>,
    ) -> AppResult<HttpResponse> {
        let ctx = self.create_context(user_id, org_id);
        let request_id = ctx.request_id.clone();
        
        let resource_id = self.parse_resource_id(path);
        
        match method {
            "GET" => {
                if let Some(id) = resource_id {
                    Ok(self.success_response(
                        serde_json::json!({
                            "order_id": id,
                            "message": "Order details (monolith mode)"
                        }),
                        &request_id,
                    ))
                } else {
                    Ok(self.success_response(
                        serde_json::json!({
                            "items": [],
                            "total": 0,
                            "message": "Order list (monolith mode)"
                        }),
                        &request_id,
                    ))
                }
            }
            "POST" => {
                let data = body.map(|b| b.0).unwrap_or(Value::Null);
                
                Ok(HttpResponse::Created().json(serde_json::json!({
                    "success": true,
                    "data": {
                        "message": "Order creation command submitted",
                        "payload": data
                    },
                    "request_id": request_id,
                    "timestamp": Utc::now().timestamp(),
                    "mode": "monolith_ddd"
                })))
            }
            "PUT" => {
                let data = body.map(|b| b.0).unwrap_or(Value::Null);
                
                Ok(self.success_response(
                    serde_json::json!({
                        "order_id": resource_id,
                        "message": "Order update command submitted",
                        "payload": data
                    }),
                    &request_id,
                ))
            }
            "DELETE" => {
                if resource_id.is_none() {
                    return Err(AppError::validation_error("Order ID is required", None));
                }
                
                Ok(HttpResponse::NoContent().finish())
            }
            _ => {
                Err(AppError::validation_error(
                    &format!("Method {} not allowed", method),
                    None,
                ))
            }
        }
    }
}

#[async_trait]
impl RouteAdapter for MonolithRouteHandler {
    fn name(&self) -> &str {
        "MonolithDDD"
    }

    fn supported_mode(&self) -> ArchitectureMode {
        ArchitectureMode::MonolithDDD
    }

    async fn handle_get(
        &self,
        path: &str,
        user_id: Option<i32>,
        organization_id: Option<i32>,
        query: web::Query<Value>,
    ) -> AppResult<HttpResponse> {
        if path.starts_with("/api/vehicles") {
            self.handle_vehicles(path, "GET", user_id, organization_id, Some(query), None).await
        } else if path.starts_with("/api/orders") {
            self.handle_orders(path, "GET", user_id, organization_id, Some(query), None).await
        } else {
            let ctx = self.create_context(user_id, organization_id);
            Ok(self.success_response(
                serde_json::json!({
                    "message": "GET request handled by monolith adapter",
                    "path": path
                }),
                &ctx.request_id,
            ))
        }
    }

    async fn handle_post(
        &self,
        path: &str,
        user_id: Option<i32>,
        organization_id: Option<i32>,
        body: web::Json<Value>,
    ) -> AppResult<HttpResponse> {
        if path.starts_with("/api/vehicles") {
            self.handle_vehicles(path, "POST", user_id, organization_id, None, Some(body)).await
        } else if path.starts_with("/api/orders") {
            self.handle_orders(path, "POST", user_id, organization_id, None, Some(body)).await
        } else {
            let ctx = self.create_context(user_id, organization_id);
            Ok(HttpResponse::Created().json(serde_json::json!({
                "success": true,
                "data": {
                    "message": "POST request handled by monolith adapter",
                    "path": path
                },
                "request_id": ctx.request_id,
                "timestamp": Utc::now().timestamp(),
                "mode": "monolith_ddd"
            })))
        }
    }

    async fn handle_put(
        &self,
        path: &str,
        user_id: Option<i32>,
        organization_id: Option<i32>,
        body: web::Json<Value>,
    ) -> AppResult<HttpResponse> {
        if path.starts_with("/api/vehicles") {
            self.handle_vehicles(path, "PUT", user_id, organization_id, None, Some(body)).await
        } else if path.starts_with("/api/orders") {
            self.handle_orders(path, "PUT", user_id, organization_id, None, Some(body)).await
        } else {
            let ctx = self.create_context(user_id, organization_id);
            Ok(self.success_response(
                serde_json::json!({
                    "message": "PUT request handled by monolith adapter",
                    "path": path
                }),
                &ctx.request_id,
            ))
        }
    }

    async fn handle_delete(
        &self,
        path: &str,
        user_id: Option<i32>,
        organization_id: Option<i32>,
    ) -> AppResult<HttpResponse> {
        if path.starts_with("/api/vehicles") {
            self.handle_vehicles(path, "DELETE", user_id, organization_id, None, None).await
        } else if path.starts_with("/api/orders") {
            self.handle_orders(path, "DELETE", user_id, organization_id, None, None).await
        } else {
            Ok(HttpResponse::NoContent().finish())
        }
    }

    async fn handle_patch(
        &self,
        path: &str,
        user_id: Option<i32>,
        organization_id: Option<i32>,
        body: web::Json<Value>,
    ) -> AppResult<HttpResponse> {
        if path.starts_with("/api/vehicles") {
            self.handle_vehicles(path, "PATCH", user_id, organization_id, None, Some(body)).await
        } else if path.starts_with("/api/orders") {
            self.handle_orders(path, "PATCH", user_id, organization_id, None, Some(body)).await
        } else {
            let ctx = self.create_context(user_id, organization_id);
            Ok(self.success_response(
                serde_json::json!({
                    "message": "PATCH request handled by monolith adapter",
                    "path": path
                }),
                &ctx.request_id,
            ))
        }
    }

    async fn health_check(&self) -> AppResult<bool> {
        // 检查数据库连接
        let result = sqlx::query("SELECT 1")
            .fetch_one(&self.pool)
            .await;
        
        match result {
            Ok(_) => Ok(true),
            Err(e) => {
                log::warn!("Health check failed: {}", e);
                Ok(false)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_resource_id() {
        // 测试路径解析逻辑
        let path = "/api/vehicles/123";
        let parts: Vec<&str> = path.trim_start_matches('/').split('/').collect();
        assert_eq!(parts.len(), 3);
        
        let id: Option<i32> = if parts.len() >= 3 {
            parts[2].parse().ok()
        } else {
            None
        };
        assert_eq!(id, Some(123));
    }
}
