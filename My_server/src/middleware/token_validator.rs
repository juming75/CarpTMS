use actix_web::{dev::ServiceRequest, error::ErrorUnauthorized};
use log::{info, warn};

use crate::utils::jwt::verify_token;

/// 令牌验证器
pub struct TokenValidator;

impl TokenValidator {
    /// 验证令牌
    pub fn validate_token(req: &ServiceRequest) -> Result<crate::utils::jwt::Claims, actix_web::Error> {
        let path = req.path().to_string();
        
        // 服务状态检查接口允许无token访问
        if path == "/api/services/status" {
            return Err(ErrorUnauthorized("Token not required for this endpoint"));
        }
        
        // 从请求头获取令牌
        let token = req
            .headers()
            .get("Authorization")
            .and_then(|h| h.to_str().ok())
            .and_then(|auth| auth.strip_prefix("Bearer ").map(|s| s.to_string()));

        match token {
            Some(token) => {
                // 验证真实令牌
                match verify_token(&token) {
                    Ok(claims) => {
                        info!(
                            "Token verification successful for user: {}, role: {}, path: {}",
                            claims.sub, claims.role, path
                        );
                        Ok(claims)
                    },
                    Err(e) => {
                        // 令牌无效
                        warn!("Invalid token for path: {}, error: {:?}", path, e);
                        Err(ErrorUnauthorized("Invalid or expired token"))
                    }
                }
            }
            None => {
                // 没有提供令牌
                warn!("No token provided for path: {}", path);
                Err(ErrorUnauthorized("Token not provided"))
            }
        }
    }
}



