//! /! 多因素认证中间件

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error,
};
use futures::future::{ready, LocalBoxFuture, Ready};
use log::{info, warn};
use std::sync::Arc;

/// 多因素认证配置
#[derive(Clone)]
pub struct MultiFactorAuthConfig {
    /// 需要多因素认证的操作列表
    pub sensitive_operations: Vec<String>,
    /// 需要多因素认证的路径列表
    pub sensitive_paths: Vec<String>,
}

/// 多因素认证中间件
#[derive(Clone)]
pub struct MultiFactorAuthMiddleware {
    config: MultiFactorAuthConfig,
}

impl MultiFactorAuthMiddleware {
    /// 创建新的多因素认证中间件
    pub fn new(config: MultiFactorAuthConfig) -> Self {
        Self { config }
    }

}

impl Default for MultiFactorAuthMiddleware {
    fn default() -> Self {
        Self::new(MultiFactorAuthConfig {
            sensitive_operations: vec![
                "login".to_string(),
                "change_password".to_string(),
                "reset_password".to_string(),
                "update_profile".to_string(),
                "delete_account".to_string(),
                "add_user".to_string(),
                "delete_user".to_string(),
                "update_user_role".to_string(),
            ],
            sensitive_paths: vec![
                "/api/auth/login".to_string(),
                "/api/auth/change_password".to_string(),
                "/api/auth/reset_password".to_string(),
                "/api/user/update".to_string(),
                "/api/user/delete".to_string(),
                "/api/user/add".to_string(),
                "/api/user/delete".to_string(),
                "/api/user/role".to_string(),
            ],
        })
    }
}

impl MultiFactorAuthMiddleware {

    /// 检查是否需要多因素认证
    fn requires_mfa(&self, req: &ServiceRequest) -> bool {
        let path = req.path();
        let _method = req.method().as_str();

        // 检查路径是否在敏感路径列表中
        if self.config.sensitive_paths.contains(&path.to_string()) {
            return true;
        }

        // 检查操作是否在敏感操作列表中
        // 这里可以根据实际情况从请求中提取操作信息
        // 暂时简单实现,检查路径中是否包含敏感操作
        for operation in &self.config.sensitive_operations {
            if path.contains(operation) {
                return true;
            }
        }

        false
    }

    /// 验证多因素认证
    fn verify_mfa(&self, req: &ServiceRequest) -> bool {
        // 从请求中获取MFA令牌
        let mfa_token = req
            .headers()
            .get("X-MFA-Token")
            .and_then(|h| h.to_str().ok());

        match mfa_token {
            Some(token) => {
                // 这里应该实现实际的MFA令牌验证逻辑
                // 暂时简单实现,检查令牌是否为"123456"
                token == "123456"
            }
            None => {
                // 没有提供MFA令牌
                false
            }
        }
    }
}

// 中间件转换实现
impl<S, B> Transform<S, ServiceRequest> for MultiFactorAuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = MultiFactorAuthMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(MultiFactorAuthMiddlewareService {
            service: Arc::new(service),
            mfa: self.clone(),
        }))
    }
}

// 中间件服务结构体
pub struct MultiFactorAuthMiddlewareService<S> {
    service: Arc<S>,
    mfa: MultiFactorAuthMiddleware,
}

impl<S, B> Service<ServiceRequest> for MultiFactorAuthMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = Arc::clone(&self.service);
        let mfa = self.mfa.clone();

        Box::pin(async move {
            // 检查是否需要多因素认证
            if mfa.requires_mfa(&req) {
                // 验证多因素认证
                if !mfa.verify_mfa(&req) {
                    warn!("MFA verification failed for path: {}", req.path());
                    return Err(actix_web::error::ErrorUnauthorized(
                        "MFA verification required",
                    ));
                }
                info!("MFA verification successful for path: {}", req.path());
            }

            // 继续处理请求
            let res = service.call(req).await?;
            Ok(res)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::test::TestRequest;

    #[test]
    fn test_requires_mfa() {
        let middleware = MultiFactorAuthMiddleware::default();

        // 测试敏感路径
        let req1 = TestRequest::default()
            .uri("/api/auth/login")
            .to_srv_request();
        assert!(middleware.requires_mfa(&req1));

        // 测试非敏感路径
        let req2 = TestRequest::default()
            .uri("/api/user/profile")
            .to_srv_request();
        assert!(!middleware.requires_mfa(&req2));
    }

    #[test]
    fn test_verify_mfa() {
        let middleware = MultiFactorAuthMiddleware::default();

        // 测试有效的MFA令牌
        let req1 = TestRequest::default()
            .insert_header(("X-MFA-Token", "123456"))
            .to_srv_request();
        assert!(middleware.verify_mfa(&req1));

        // 测试无效的MFA令牌
        let req2 = TestRequest::default()
            .insert_header(("X-MFA-Token", "invalid"))
            .to_srv_request();
        assert!(!middleware.verify_mfa(&req2));

        // 测试没有MFA令牌
        let req3 = TestRequest::default().to_srv_request();
        assert!(!middleware.verify_mfa(&req3));
    }
}
