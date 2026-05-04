//! / 资源类型枚举
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Resource {
    User,
    Vehicle,
    VehicleGroup,
    Weighing,
    Report,
    Sync,
    Role,
    Device,
    Statistics,
    Alert,
    Settings,
    Services,
    RemoteOps, // 远程运维资源 - 需要高级权限
    All,
}

// 操作类型枚举
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Action {
    Create,
    Read,
    Update,
    Delete,
    All,
}

// 角色枚举(与middleware::auth中的Role保持一致)
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Role {
    Admin,
    Manager,
    User,
    Guest,
}

// 从字符串转换为Resource
pub fn resource_from_str(resource_str: &str) -> Resource {
    match resource_str.to_lowercase().as_str() {
        "user" => Resource::User,
        "vehicle" => Resource::Vehicle,
        "vehicle_group" => Resource::VehicleGroup,
        "weighing" => Resource::Weighing,
        "report" => Resource::Report,
        "sync" => Resource::Sync,
        "role" => Resource::Role,
        "device" => Resource::Device,
        "statistics" => Resource::Statistics,
        "alert" => Resource::Alert,
        "alerts" => Resource::Alert,
        "settings" => Resource::Settings,
        "services" => Resource::Services,
        "remote_ops" | "remoteops" => Resource::RemoteOps,
        "all" => Resource::All,
        _ => Resource::All,
    }
}

// 从字符串转换为Action
pub fn action_from_str(action_str: &str) -> Action {
    match action_str.to_lowercase().as_str() {
        "create" => Action::Create,
        "read" => Action::Read,
        "update" => Action::Update,
        "delete" => Action::Delete,
        "all" => Action::All,
        _ => Action::Read,
    }
}

// 权限检查函数
pub fn has_permission(role: Role, resource: Resource, action: Action) -> bool {
    // 定义权限映射:角色 -> 资源 -> 允许的操作
    match role {
        Role::Admin => {
            // 管理员可以对所有资源执行所有操作
            true
        }
        Role::Manager => {
            match (resource, action) {
                // 经理可以管理车辆、车组和称重数据
                (Resource::Vehicle, _) => true,
                (Resource::VehicleGroup, _) => true,
                (Resource::Weighing, _) => true,
                (Resource::Report, _) => true,
                (Resource::Sync, _) => true,
                // 经理可以管理设备
                (Resource::Device, _) => true,
                // 经理可以查看和管理统计数据
                (Resource::Statistics, _) => true,
                // 经理可以查看和管理报警
                (Resource::Alert, _) => true,
                // 经理可以查看和更新设置
                (Resource::Settings, Action::Read) => true,
                (Resource::Settings, Action::Update) => true,
                // 经理可以查看服务状态
                (Resource::Services, Action::Read) => true,
                // 经理可以查看用户,但不能创建或删除用户
                (Resource::User, Action::Read) => true,
                // 经理不能管理角色
                // 经理不能访问远程运维（安全限制）
                _ => false,
            }
        }
        Role::User => {
            match (resource, action) {
                // 用户可以查看和更新自己的信息
                (Resource::User, Action::Read) => true,
                (Resource::User, Action::Update) => true,
                // 用户可以查看车辆、车组和称重数据
                (Resource::Vehicle, Action::Read) => true,
                (Resource::VehicleGroup, Action::Read) => true,
                (Resource::Weighing, Action::Read) => true,
                // 用户可以生成报告
                (Resource::Report, Action::Read) => true,
                (Resource::Report, Action::Create) => true,
                // 用户可以同步数据
                (Resource::Sync, _) => true,
                // 用户可以查看设备信息
                (Resource::Device, Action::Read) => true,
                // 用户可以查看统计数据
                (Resource::Statistics, Action::Read) => true,
                // 用户可以查看报警
                (Resource::Alert, Action::Read) => true,
                (Resource::Alert, Action::Update) => true,
                // 用户可以查看设置
                (Resource::Settings, Action::Read) => true,
                // 用户可以查看服务状态
                (Resource::Services, Action::Read) => true,
                // 用户不能管理角色
                // 用户不能访问远程运维（安全限制）
                _ => false,
            }
        }
        Role::Guest => {
            // 访客只能查看公开资源
            match (resource, action) {
                (Resource::Vehicle, Action::Read) => true,
                (Resource::VehicleGroup, Action::Read) => true,
                // 访客可以查看设备信息
                (Resource::Device, Action::Read) => true,
                // 访客不能查看统计数据
                // 访客不能访问远程运维
                _ => false,
            }
        }
    }
}

// 从字符串转换为当前模块的Role
pub fn from_middleware_role(role: &str) -> Role {
    match role {
        "Admin" => Role::Admin,
        "Manager" => Role::Manager,
        "User" => Role::User,
        _ => Role::Guest,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // 测试资源转换函数
    #[test]
    fn test_resource_from_str() {
        assert_eq!(resource_from_str("user"), Resource::User);
        assert_eq!(resource_from_str("Vehicle"), Resource::Vehicle);
        assert_eq!(resource_from_str("VEHICLE_GROUP"), Resource::VehicleGroup);
        assert_eq!(resource_from_str("weighing"), Resource::Weighing);
        assert_eq!(resource_from_str("report"), Resource::Report);
        assert_eq!(resource_from_str("sync"), Resource::Sync);
        assert_eq!(resource_from_str("role"), Resource::Role);
        assert_eq!(resource_from_str("device"), Resource::Device);
        assert_eq!(resource_from_str("statistics"), Resource::Statistics);
        assert_eq!(resource_from_str("all"), Resource::All);
        // 测试未知资源
        assert_eq!(resource_from_str("unknown"), Resource::All);
        assert_eq!(resource_from_str(""), Resource::All);
    }

    // 测试操作转换函数
    #[test]
    fn test_action_from_str() {
        assert_eq!(action_from_str("create"), Action::Create);
        assert_eq!(action_from_str("READ"), Action::Read);
        assert_eq!(action_from_str("Update"), Action::Update);
        assert_eq!(action_from_str("DELETE"), Action::Delete);
        assert_eq!(action_from_str("all"), Action::All);
        // 测试未知操作
        assert_eq!(action_from_str("unknown"), Action::Read);
        assert_eq!(action_from_str(""), Action::Read);
    }

    // 测试权限检查函数 - 管理员权限
    #[test]
    fn test_has_permission_admin() {
        // 管理员可以执行所有操作
        assert!(has_permission(Role::Admin, Resource::User, Action::Create));
        assert!(has_permission(
            Role::Admin,
            Resource::Vehicle,
            Action::Delete
        ));
        assert!(has_permission(Role::Admin, Resource::Role, Action::All));
        assert!(has_permission(Role::Admin, Resource::All, Action::All));
    }

    // 测试权限检查函数 - 经理权限
    #[test]
    fn test_has_permission_manager() {
        // 经理可以管理车辆、车组和称重数据
        assert!(has_permission(
            Role::Manager,
            Resource::Vehicle,
            Action::Create
        ));
        assert!(has_permission(
            Role::Manager,
            Resource::VehicleGroup,
            Action::Delete
        ));
        assert!(has_permission(
            Role::Manager,
            Resource::Weighing,
            Action::Update
        ));

        // 经理可以查看用户,但不能创建用户
        assert!(has_permission(Role::Manager, Resource::User, Action::Read));
        assert!(!has_permission(
            Role::Manager,
            Resource::User,
            Action::Create
        ));

        // 经理不能管理角色
        assert!(!has_permission(
            Role::Manager,
            Resource::Role,
            Action::Create
        ));
    }

    // 测试权限检查函数 - 用户权限
    #[test]
    fn test_has_permission_user() {
        // 用户可以查看和更新自己的信息
        assert!(has_permission(Role::User, Resource::User, Action::Read));
        assert!(has_permission(Role::User, Resource::User, Action::Update));
        assert!(!has_permission(Role::User, Resource::User, Action::Delete));

        // 用户可以查看车辆、车组和称重数据
        assert!(has_permission(Role::User, Resource::Vehicle, Action::Read));
        assert!(has_permission(
            Role::User,
            Resource::VehicleGroup,
            Action::Read
        ));
        assert!(has_permission(Role::User, Resource::Weighing, Action::Read));
        assert!(!has_permission(
            Role::User,
            Resource::Vehicle,
            Action::Delete
        ));

        // 用户可以生成报告
        assert!(has_permission(Role::User, Resource::Report, Action::Create));
        assert!(has_permission(Role::User, Resource::Report, Action::Read));

        // 用户不能管理角色
        assert!(!has_permission(Role::User, Resource::Role, Action::Create));
    }

    // 测试权限检查函数 - 访客权限
    #[test]
    fn test_has_permission_guest() {
        // 访客只能查看公开资源
        assert!(has_permission(Role::Guest, Resource::Vehicle, Action::Read));
        assert!(has_permission(
            Role::Guest,
            Resource::VehicleGroup,
            Action::Read
        ));
        assert!(has_permission(Role::Guest, Resource::Device, Action::Read));

        // 访客不能执行修改操作
        assert!(!has_permission(
            Role::Guest,
            Resource::Vehicle,
            Action::Create
        ));
        assert!(!has_permission(Role::Guest, Resource::User, Action::Read));
        assert!(!has_permission(
            Role::Guest,
            Resource::Weighing,
            Action::Read
        ));
    }

    // 测试角色转换函数
    #[test]
    fn test_from_middleware_role() {
        assert_eq!(from_middleware_role("Admin"), Role::Admin);
        assert_eq!(from_middleware_role("Manager"), Role::Manager);
        assert_eq!(from_middleware_role("User"), Role::User);
        assert_eq!(from_middleware_role("Guest"), Role::Guest);
        assert_eq!(from_middleware_role("Unknown"), Role::Guest);
    }
}
