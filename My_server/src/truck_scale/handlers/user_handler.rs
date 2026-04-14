//! / 用户管理处理器
use crate::truck_scale::db::TruckScaleDb;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

/// 用户类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserType {
    SuperAdmin = 0, // 超级管理员
    Admin = 1,      // 管理员
    Operator = 2,   // 操作员
    User = 3,       // 普通用户
}

impl UserType {
    pub fn from_i32(value: i32) -> Self {
        match value {
            0 => UserType::SuperAdmin,
            1 => UserType::Admin,
            2 => UserType::Operator,
            _ => UserType::User,
        }
    }

    pub fn to_i32(&self) -> i32 {
        *self as i32
    }
}

/// 用户信息(43个字段)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    // 基本信息
    pub user_id: String,   // 用户ID
    pub user_name: String, // 用户名
    pub password: String,  // 密码(bcrypt哈希)
    pub real_name: String, // 真实姓名
    pub user_type: i32,    // 用户类型

    // 组织信息
    pub group_id: String,   // 所属用户组
    pub company: String,    // 所属公司
    pub department: String, // 部门

    // 联系信息
    pub tel: String,     // 电话
    pub mobile: String,  // 手机
    pub email: String,   // 邮箱
    pub address: String, // 地址

    // 权限信息
    pub permission: String,     // 权限字符串(逗号分隔)
    pub veh_group_list: String, // 可管理的车组列表(逗号分隔)

    // 账户状态
    pub status: i32,             // 状态:0=正常,1=禁用,2=锁定
    pub expiration_time: String, // 过期时间

    // 其他字段(共43个)
    pub title: String,               // 职位
    pub id_card: String,             // 身份证号
    pub id_card_expire_date: String, // 身份证到期日期
    pub education: String,           // 学历
    pub birth_date: String,          // 出生日期
    pub gender: i32,                 // 性别:0=男,1=女
    pub avatar: String,              // 头像
    pub signature: String,           // 签名

    // 登录信息
    pub last_login_time: String, // 最后登录时间
    pub last_login_ip: String,   // 最后登录IP
    pub login_count: i32,        // 登录次数

    // 系统字段
    pub remark: String,      // 备注
    pub create_time: String, // 创建时间
    pub update_time: String, // 更新时间
    pub create_by: String,   // 创建人
    pub update_by: String,   // 更新人
}

/// 用户组信息(4个字段)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserGroup {
    pub group_id: String,   // 用户组ID
    pub group_name: String, // 用户组名称
    pub user_type: i32,     // 用户类型
    pub permission: String, // 权限字符串
}

/// 用户管理处理器
pub struct UserHandler {
    db: TruckScaleDb,
}

impl UserHandler {
    /// 创建新的用户管理处理器
    pub fn new(pool: PgPool) -> Self {
        Self {
            db: TruckScaleDb::new(pool.into()),
        }
    }

    /// 从数据库操作器创建
    pub fn from_db(db: TruckScaleDb) -> Self {
        Self { db }
    }

    /// 查询用户信息
    pub async fn query_user(&self, user_id: &str) -> Result<Option<UserInfo>> {
        let user_data = self.db.query_user(user_id).await?;
        Ok(user_data.and_then(|data| serde_json::from_value(data).ok()))
    }

    /// 根据用户名查询用户
    pub async fn query_user_by_name(&self, user_name: &str) -> Result<Option<UserInfo>> {
        let user_data = self.db.query_user_by_name(user_name).await?;
        Ok(user_data.and_then(|data| serde_json::from_value(data).ok()))
    }

    /// 添加用户
    pub async fn add_user(&self, user: UserInfo) -> Result<String> {
        let user_data = serde_json::to_value(&user)?;
        self.db.add_user(user_data).await
    }

    /// 更新用户
    pub async fn update_user(&self, user: UserInfo) -> Result<()> {
        let user_data = serde_json::to_value(&user)?;
        self.db.update_user(user_data).await
    }

    /// 删除用户
    pub async fn delete_user(&self, user_id: &str, delete_by: &str) -> Result<()> {
        self.db.delete_user(user_id, delete_by).await
    }

    /// 查询用户组
    pub async fn query_user_group(&self, group_id: &str) -> Result<Option<UserGroup>> {
        let group_data = self.db.query_user_group(group_id).await?;
        group_data
            .map(|g| serde_json::from_value(g).map_err(anyhow::Error::from))
            .transpose()
    }

    /// 添加用户组
    pub async fn add_user_group(&self, group: UserGroup) -> Result<String> {
        let group_data = serde_json::to_value(&group)?;
        self.db.add_user_group(group_data).await
    }

    /// 更新用户组
    pub async fn update_user_group(&self, group: UserGroup) -> Result<()> {
        let group_data = serde_json::to_value(&group)?;
        self.db.update_vehicle_group(group_data).await
    }

    /// 删除用户组
    pub async fn delete_user_group(&self, group_id: &str) -> Result<()> {
        // 假设当前用户为admin,实际应该从上下文获取
        self.db.delete_user_group(group_id, "admin").await
    }

    /// 查询用户列表
    pub async fn query_user_list(&self, page: i32, page_size: i32) -> Result<Vec<UserInfo>> {
        // 计算偏移量
        let offset = (page - 1) * page_size;

        // 从数据库查询用户列表
        let users_data = self.db.query_user_list(offset, page_size).await?;

        // 转换为 UserInfo 类型
        let users = users_data
            .into_iter()
            .filter_map(|data| serde_json::from_value(data).ok())
            .collect();

        Ok(users)
    }
}

// impl Default for UserHandler {
//     fn default() -> Self {
//         Self::new()
//     }
// }
