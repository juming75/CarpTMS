//! 路由适配层模块
//!
//! 提供根据架构模式选择不同处理方式的适配层。
//! 支持：
//! - 单体模式：调用 application 层服务
//! - 微服务模式：通过 gRPC 调用远程服务

pub mod monolith_handler;
pub mod micro_handler;

use std::sync::Arc;
use async_trait::async_trait;
use actix_web::{HttpResponse, web};
use serde_json::Value;

use crate::config::ArchitectureMode;
use crate::errors::AppResult;

pub use monolith_handler::MonolithRouteHandler;
pub use micro_handler::MicroserviceRouteHandler;

/// 路由适配器 trait
/// 定义统一的请求处理接口，由不同架构模式的处理器实现
#[async_trait]
pub trait RouteAdapter: Send + Sync {
    /// 获取适配器名称
    fn name(&self) -> &str;

    /// 获取支持的架构模式
    fn supported_mode(&self) -> ArchitectureMode;

    /// 处理 GET 请求
    async fn handle_get(
        &self,
        path: &str,
        user_id: Option<i32>,
        organization_id: Option<i32>,
        query: web::Query<Value>,
    ) -> AppResult<HttpResponse>;

    /// 处理 POST 请求
    async fn handle_post(
        &self,
        path: &str,
        user_id: Option<i32>,
        organization_id: Option<i32>,
        body: web::Json<Value>,
    ) -> AppResult<HttpResponse>;

    /// 处理 PUT 请求
    async fn handle_put(
        &self,
        path: &str,
        user_id: Option<i32>,
        organization_id: Option<i32>,
        body: web::Json<Value>,
    ) -> AppResult<HttpResponse>;

    /// 处理 DELETE 请求
    async fn handle_delete(
        &self,
        path: &str,
        user_id: Option<i32>,
        organization_id: Option<i32>,
    ) -> AppResult<HttpResponse>;

    /// 处理 PATCH 请求
    async fn handle_patch(
        &self,
        path: &str,
        user_id: Option<i32>,
        organization_id: Option<i32>,
        body: web::Json<Value>,
    ) -> AppResult<HttpResponse>;

    /// 健康检查
    async fn health_check(&self) -> AppResult<bool>;
}

/// 路由适配器工厂
pub struct RouteAdapterFactory {
    /// 数据库连接池
    pool: sqlx::PgPool,
}

impl RouteAdapterFactory {
    /// 创建新的工厂
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }

    /// 根据架构模式创建适配器
    pub fn create_adapter(&self, mode: ArchitectureMode) -> Arc<dyn RouteAdapter> {
        match mode {
            ArchitectureMode::MonolithDDD => {
                Arc::new(MonolithRouteHandler::new(self.pool.clone()))
            }
            ArchitectureMode::MicroDDD => {
                Arc::new(MicroserviceRouteHandler::new())
            }
        }
    }
}

/// 路由适配器管理器
/// 支持运行时切换架构模式
pub struct RouteAdapterManager {
    /// 当前适配器
    current_adapter: Arc<dyn RouteAdapter>,
    /// 当前架构模式
    current_mode: ArchitectureMode,
    /// 适配器工厂
    factory: RouteAdapterFactory,
}

impl RouteAdapterManager {
    /// 创建新的管理器
    pub fn new(pool: sqlx::PgPool, initial_mode: ArchitectureMode) -> Self {
        let factory = RouteAdapterFactory::new(pool);
        let current_adapter = factory.create_adapter(initial_mode);
        
        Self {
            current_adapter,
            current_mode: initial_mode,
            factory,
        }
    }

    /// 获取当前适配器
    pub fn current(&self) -> Arc<dyn RouteAdapter> {
        self.current_adapter.clone()
    }

    /// 获取当前架构模式
    pub fn current_mode(&self) -> ArchitectureMode {
        self.current_mode
    }

    /// 切换架构模式
    pub fn switch_mode(&mut self, new_mode: ArchitectureMode) -> AppResult<()> {
        if self.current_mode == new_mode {
            return Ok(());
        }
        
        log::info!(
            "Switching architecture mode from {} to {}",
            self.current_mode,
            new_mode
        );
        
        let new_adapter = self.factory.create_adapter(new_mode);
        self.current_adapter = new_adapter;
        self.current_mode = new_mode;
        
        Ok(())
    }
}

/// 路由上下文信息
#[derive(Debug, Clone)]
pub struct RouteContext {
    /// 请求路径
    pub path: String,
    /// HTTP 方法
    pub method: String,
    /// 用户 ID（如果已认证）
    pub user_id: Option<i32>,
    /// 组织 ID
    pub organization_id: Option<i32>,
    /// 请求 ID
    pub request_id: String,
    /// 架构模式
    pub mode: ArchitectureMode,
}

impl RouteContext {
    /// 创建新的路由上下文
    pub fn new(path: &str, method: &str, mode: ArchitectureMode) -> Self {
        Self {
            path: path.to_string(),
            method: method.to_string(),
            user_id: None,
            organization_id: None,
            request_id: uuid::Uuid::new_v4().to_string(),
            mode,
        }
    }

    /// 设置用户信息
    pub fn with_user(mut self, user_id: i32, org_id: Option<i32>) -> Self {
        self.user_id = Some(user_id);
        self.organization_id = org_id;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_route_context() {
        let context = RouteContext {
            path: "/api/vehicles".to_string(),
            method: "GET".to_string(),
            user_id: Some(1),
            organization_id: Some(100),
            request_id: "req-123".to_string(),
            mode: ArchitectureMode::MonolithDDD,
        };
        
        assert_eq!(context.path, "/api/vehicles");
        assert_eq!(context.method, "GET");
        assert_eq!(context.user_id, Some(1));
    }
}
