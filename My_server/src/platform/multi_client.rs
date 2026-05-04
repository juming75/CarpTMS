//! 多端协同架构模块
//!
//! 实现管理端、运营端与企业端的业务隔离和协同
//! 参考博客：从JT808/JT1078协议到多端协同：构建高并发北斗车辆监控平台的技术实践

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use log::{info, debug, warn};
use chrono::{DateTime, Utc};

/// 端类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ClientType {
    /// 管理端（平台管理员）
    Admin,
    /// 运营端（运营管理）
    Operation,
    /// 企业端（企业用户）
    Enterprise,
    /// 司机端（移动端）
    Driver,
}

impl ClientType {
    /// 获取端类型显示名称
    pub fn display_name(&self) -> &str {
        match self {
            ClientType::Admin => "管理端",
            ClientType::Operation => "运营端",
            ClientType::Enterprise => "企业端",
            ClientType::Driver => "司机端",
        }
    }

    /// 获取默认权限级别
    pub fn default_permission_level(&self) -> u8 {
        match self {
            ClientType::Admin => 100,
            ClientType::Operation => 80,
            ClientType::Enterprise => 60,
            ClientType::Driver => 40,
        }
    }
}

/// 角色信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    /// 角色ID
    pub role_id: String,
    /// 角色名称
    pub role_name: String,
    /// 角色描述
    pub description: String,
    /// 所属端类型
    pub client_type: ClientType,
    /// 权限级别
    pub permission_level: u8,
    /// 权限列表
    pub permissions: Vec<String>,
}

/// 用户信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    /// 用户ID
    pub user_id: String,
    /// 用户名
    pub username: String,
    /// 所属端类型
    pub client_type: ClientType,
    /// 角色列表
    pub roles: Vec<String>,
    /// 企业ID（企业端用户）
    pub enterprise_id: Option<String>,
    /// 状态
    pub status: UserStatus,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 最后登录时间
    pub last_login: Option<DateTime<Utc>>,
}

/// 用户状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UserStatus {
    /// 活跃
    Active,
    /// 禁用
    Disabled,
    /// 锁定
    Locked,
}

/// 企业信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Enterprise {
    /// 企业ID
    pub enterprise_id: String,
    /// 企业名称
    pub enterprise_name: String,
    /// 企业编码
    pub enterprise_code: String,
    /// 联系人
    pub contact_person: String,
    /// 联系电话
    pub contact_phone: String,
    /// 车辆数量
    pub vehicle_count: u64,
    /// 司机数量
    pub driver_count: u64,
    /// 状态
    pub status: EnterpriseStatus,
    /// 创建时间
    pub created_at: DateTime<Utc>,
}

/// 企业状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EnterpriseStatus {
    /// 正常
    Active,
    /// 停用
    Suspended,
    /// 审核中
    Pending,
}

/// 数据权限范围
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataPermission {
    /// 权限ID
    pub permission_id: String,
    /// 权限名称
    pub permission_name: String,
    /// 权限类型
    pub permission_type: PermissionType,
    /// 数据范围
    pub data_scope: DataScope,
    /// 所属端类型
    pub client_type: ClientType,
}

/// 权限类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PermissionType {
    /// 菜单权限
    Menu,
    /// 按钮权限
    Button,
    /// 数据权限
    Data,
    /// API权限
    Api,
}

/// 数据范围
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataScope {
    /// 全部数据
    All,
    /// 本部门数据
    Department,
    /// 本部门及以下数据
    DepartmentAndBelow,
    /// 仅本人数据
    SelfOnly,
    /// 自定义数据
    Custom,
}

/// 多端协同管理器
/// 管理多端用户的权限、数据隔离和协同
pub struct MultiClientCoordinator {
    /// 用户信息
    users: Arc<RwLock<HashMap<String, UserInfo>>>,
    /// 角色信息
    roles: Arc<RwLock<HashMap<String, Role>>>,
    /// 企业信息
    enterprises: Arc<RwLock<HashMap<String, Enterprise>>>,
    /// 权限信息
    permissions: Arc<RwLock<HashMap<String, DataPermission>>>,
    /// 用户-企业映射
    user_enterprise_mapping: Arc<RwLock<HashMap<String, String>>>,
}

impl MultiClientCoordinator {
    /// 创建新的多端协同管理器
    pub fn new() -> Self {
        Self {
            users: Arc::new(RwLock::new(HashMap::new())),
            roles: Arc::new(RwLock::new(HashMap::new())),
            enterprises: Arc::new(RwLock::new(HashMap::new())),
            permissions: Arc::new(RwLock::new(HashMap::new())),
            user_enterprise_mapping: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 注册用户
    pub async fn register_user(&self, user: UserInfo) -> Result<(), String> {
        let mut users = self.users.write().await;
        if users.contains_key(&user.user_id) {
            return Err(format!("User {} already exists", user.user_id));
        }
        users.insert(user.user_id.clone(), user);
        info!("User registered: {} ({:?})", user.user_id, user.client_type);
        Ok(())
    }

    /// 注册企业
    pub async fn register_enterprise(&self, enterprise: Enterprise) -> Result<(), String> {
        let mut enterprises = self.enterprises.write().await;
        if enterprises.contains_key(&enterprise.enterprise_id) {
            return Err(format!("Enterprise {} already exists", enterprise.enterprise_id));
        }
        enterprises.insert(enterprise.enterprise_id.clone(), enterprise);
        info!("Enterprise registered: {}", enterprise.enterprise_id);
        Ok(())
    }

    /// 关联用户和企业
    pub async fn link_user_to_enterprise(&self, user_id: &str, enterprise_id: &str) -> Result<(), String> {
        let mut mapping = self.user_enterprise_mapping.write().await;
        mapping.insert(user_id.to_string(), enterprise_id.to_string());
        Ok(())
    }

    /// 检查用户权限
    pub async fn check_permission(&self, user_id: &str, permission: &str) -> bool {
        let users = self.users.read().await;
        if let Some(user) = users.get(user_id) {
            // 管理端拥有所有权限
            if user.client_type == ClientType::Admin {
                return true;
            }

            // 检查用户角色的权限
            let roles = self.roles.read().await;
            for role_id in &user.roles {
                if let Some(role) = roles.get(role_id) {
                    if role.permissions.contains(&permission.to_string()) {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// 获取用户数据范围
    pub async fn get_user_data_scope(&self, user_id: &str) -> Option<DataScope> {
        let users = self.users.read().await;
        if let Some(user) = users.get(user_id) {
            let roles = self.roles.read().await;
            for role_id in &user.roles {
                if let Some(role) = roles.get(role_id) {
                    let permissions = self.permissions.read().await;
                    for perm_id in &role.permissions {
                        if let Some(perm) = permissions.get(perm_id) {
                            if perm.permission_type == PermissionType::Data {
                                return Some(perm.data_scope);
                            }
                        }
                    }
                }
            }
        }
        None
    }

    /// 获取用户可访问的企业列表
    pub async fn get_accessible_enterprises(&self, user_id: &str) -> Vec<Enterprise> {
        let users = self.users.read().await;
        if let Some(user) = users.get(user_id) {
            match user.client_type {
                ClientType::Admin => {
                    // 管理端可访问所有企业
                    let enterprises = self.enterprises.read().await;
                    enterprises.values().cloned().collect()
                }
                ClientType::Operation => {
                    // 运营端可访问指定的企业
                    let enterprises = self.enterprises.read().await;
                    enterprises.values().cloned().collect()
                }
                ClientType::Enterprise => {
                    // 企业端只能访问自己的企业
                    if let Some(enterprise_id) = &user.enterprise_id {
                        let enterprises = self.enterprises.read().await;
                        if let Some(enterprise) = enterprises.get(enterprise_id) {
                            return vec![enterprise.clone()];
                        }
                    }
                    Vec::new()
                }
                ClientType::Driver => {
                    // 司机端只能访问自己的企业
                    if let Some(enterprise_id) = &user.enterprise_id {
                        let enterprises = self.enterprises.read().await;
                        if let Some(enterprise) = enterprises.get(enterprise_id) {
                            return vec![enterprise.clone()];
                        }
                    }
                    Vec::new()
                }
            }
        } else {
            Vec::new()
        }
    }

    /// 获取用户列表
    pub async fn list_users(&self, client_type: Option<ClientType>) -> Vec<UserInfo> {
        let users = self.users.read().await;
        users.values()
            .filter(|u| {
                if let Some(ct) = client_type {
                    u.client_type == ct
                } else {
                    true
                }
            })
            .cloned()
            .collect()
    }

    /// 获取企业列表
    pub async fn list_enterprises(&self) -> Vec<Enterprise> {
        let enterprises = self.enterprises.read().await;
        enterprises.values().cloned().collect()
    }

    /// 更新用户最后登录时间
    pub async fn update_last_login(&self, user_id: &str) {
        let mut users = self.users.write().await;
        if let Some(user) = users.get_mut(user_id) {
            user.last_login = Some(Utc::now());
        }
    }
}

impl Default for MultiClientCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

/// 创建多端协同管理器（便捷函数）
pub fn create_multi_client_coordinator() -> Arc<MultiClientCoordinator> {
    Arc::new(MultiClientCoordinator::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_user_registration() {
        let coordinator = MultiClientCoordinator::new();
        let user = UserInfo {
            user_id: "user_001".to_string(),
            username: "admin".to_string(),
            client_type: ClientType::Admin,
            roles: vec!["admin_role".to_string()],
            enterprise_id: None,
            status: UserStatus::Active,
            created_at: Utc::now(),
            last_login: None,
        };
        
        assert!(coordinator.register_user(user).await.is_ok());
    }
}
