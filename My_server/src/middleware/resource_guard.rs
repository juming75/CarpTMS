use actix_web::{dev::ServiceRequest, error::ErrorForbidden};
use log::{info, warn};

use super::permission_checker::Role;

/// 资源守卫
pub struct ResourceGuard;

impl ResourceGuard {
    /// 检查敏感资源访问权限
    pub fn check_sensitive_resource(req: &ServiceRequest, user_role: Role) -> Result<(), actix_web::Error> {
        let path = req.path().to_string();
        
        // 对于敏感数据,即使角色匹配,也需要额外检查
        if path.starts_with("/api/users") || path.starts_with("/api/roles") {
            // 只有管理员可以访问用户管理和角色管理
            if user_role != Role::Admin {
                warn!(
                    "Sensitive resource access denied for user: role: {:?}, path: {}",
                    user_role, path
                );
                return Err(ErrorForbidden(
                    "权限不足,无法访问敏感资源",
                ));
            }
            info!(
                "Sensitive resource access granted for user: role: {:?}, path: {}",
                user_role, path
            );
        }
        
        Ok(())
    }
}



