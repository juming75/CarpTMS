use log::{info, warn};

use crate::utils::permissions::{has_permission, Action, Resource, Role as PermRole};

/// 角色枚举
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Role {
    Admin,
    Manager,
    User,
    Guest,
}

/// 从字符串转换为Role
pub fn role_from_str(role_str: &str) -> Role {
    // 去除字符串两端的空格,确保匹配准确性
    let role = role_str.trim();

    // 转换为小写,用于不区分大小写的匹配
    let lowercase_role = role.to_lowercase();

    // 打印角色匹配日志,方便调试
    info!(
        "Role mapping: original='{}', trimmed='{}', lowercase='{}'",
        role_str, role, lowercase_role
    );

    // 使用精确匹配或starts_with检查，避免contains导致的误判
    // 例如 "vip_admin_user" 不应该被识别为 Admin
    if lowercase_role == "admin"
        || lowercase_role.starts_with("admin_")
        || lowercase_role.contains("管理员")
    {
        info!("Role mapping result: Admin");
        Role::Admin
    } else if lowercase_role == "manager"
        || lowercase_role.starts_with("manager_")
        || lowercase_role.contains("经理")
    {
        info!("Role mapping result: Manager");
        Role::Manager
    } else if lowercase_role == "user"
        || lowercase_role.starts_with("user_")
        || lowercase_role.contains("普通用户")
    {
        info!("Role mapping result: User");
        Role::User
    } else {
        // 兜底逻辑:如果角色名以"admin"或"manager"开头,也给予相应权限
        let role_chars: Vec<char> = lowercase_role.chars().collect();
        let role_prefix: String = if role_chars.len() >= 5 {
            role_chars[0..5].iter().collect()
        } else {
            lowercase_role.clone()
        };

        if role_prefix == "admin" {
            info!("Role mapping result: Admin (prefix)");
            Role::Admin
        } else if role_prefix == "manag" {
            info!("Role mapping result: Manager (prefix)");
            Role::Manager
        } else if role_prefix == "user" {
            info!("Role mapping result: User (prefix)");
            Role::User
        } else {
            info!("Role mapping result: Guest");
            Role::Guest
        }
    }
}

/// 权限检查中间件
#[derive(Clone)]
pub struct PermissionChecker {
    required_role: Role,
    resource: Option<Resource>,
    action: Option<Action>,
}

impl PermissionChecker {
    pub fn new(required_role: Role) -> Self {
        Self {
            required_role,
            resource: None,
            action: None,
        }
    }

    // 设置资源
    pub fn resource(mut self, resource: Resource) -> Self {
        self.resource = Some(resource);
        self
    }

    // 设置操作
    pub fn action(mut self, action: Action) -> Self {
        self.action = Some(action);
        self
    }

    // 快捷方法:设置资源为字符串
    pub fn resource_str(mut self, resource_str: &str) -> Self {
        self.resource = Some(crate::utils::permissions::resource_from_str(resource_str));
        self
    }

    // 快捷方法:设置操作为字符串
    pub fn action_str(mut self, action_str: &str) -> Self {
        self.action = Some(crate::utils::permissions::action_from_str(action_str));
        self
    }

    pub fn admin() -> Self {
        Self::new(Role::Admin)
    }

    pub fn manager() -> Self {
        Self::new(Role::Manager)
    }

    pub fn user() -> Self {
        Self::new(Role::User)
    }

    pub fn guest() -> Self {
        Self::new(Role::Guest)
    }

    /// 检查角色权限
    pub fn check_role_permission(&self, user_role: Role) -> bool {
        match (user_role, self.required_role) {
            // 管理员可以访问所有资源
            (Role::Admin, _) => true,
            // 经理不能访问Admin资源（安全修复）
            (Role::Manager, Role::Admin) => false,
            // 经理可以访问经理、用户和访客资源
            (Role::Manager, Role::Manager) => true,
            (Role::Manager, Role::User) => true,
            (Role::Manager, Role::Guest) => true,
            // 用户可以访问用户和访客资源
            (Role::User, Role::User) => true,
            (Role::User, Role::Guest) => true,
            // 访客只能访问访客资源
            (Role::Guest, Role::Guest) => true,
            // 其他情况权限不足
            _ => false,
        }
    }

    /// 检查资源和操作权限
    pub fn check_resource_permission(&self, user_role: Role) -> Result<(), String> {
        // 管理员直接通过所有权限检查
        if user_role == Role::Admin {
            return Ok(());
        }

        if let (Some(resource), Some(action)) = (&self.resource, &self.action) {
            // 非管理员需要进行细粒度权限检查
            // 将中间件的Role转换为权限模块的Role
            let perm_role = match user_role {
                Role::Admin => PermRole::Admin,
                Role::Manager => PermRole::Manager,
                Role::User => PermRole::User,
                Role::Guest => PermRole::Guest,
            };

            // 检查资源和操作权限
            let has_perm = has_permission(perm_role, *resource, *action);
            info!(
                "Resource permission check: role={:?}, perm_role={:?}, resource={:?}, action={:?}, has_perm={}",
                user_role, perm_role, resource, action, has_perm
            );
            if !has_perm {
                warn!(
                    "Resource permission denied: role={:?}, perm_role={:?}, resource={:?}, action={:?}",
                    user_role, perm_role, resource, action
                );
                return Err("权限不足,无法访问该资源".to_string());
            }
            info!(
                "Resource permission granted: role={:?}, resource={:?}, action={:?}",
                perm_role, resource, action
            );
        }

        Ok(())
    }
}
