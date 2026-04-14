//! 用户 DTO

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::domain::entities::user::User;

/// 用户 DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserDto {
    /// 用户ID
    pub user_id: i32,
    /// 用户名
    pub user_name: String,
    /// 真实姓名
    pub full_name: String,
    /// 电话号码
    pub phone_number: Option<String>,
    /// 邮箱
    pub email: Option<String>,
    /// 用户组ID
    pub user_group_id: i32,
    /// 组织ID
    pub organization_id: Option<String>,
    /// 用户组名称
    pub user_group_name: Option<String>,
    /// 状态
    pub status: i16,
    /// 最后登录时间
    pub last_login_time: Option<NaiveDateTime>,
    /// 创建时间
    pub create_time: NaiveDateTime,
    /// 更新时间
    pub update_time: Option<NaiveDateTime>,
}

/// 从领域实体转换为 DTO
impl From<User> for UserDto {
    fn from(user: User) -> Self {
        Self {
            user_id: user.user_id,
            user_name: user.user_name,
            full_name: user.full_name,
            phone_number: user.phone_number,
            email: user.email,
            user_group_id: user.user_group_id,
            organization_id: None, // Not in User entity
            user_group_name: None, // Not in User entity
            status: user.status,
            last_login_time: user.last_login_time,
            create_time: user.create_time,
            update_time: user.update_time,
        }
    }
}

/// 用户简要 DTO（用于列表显示）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSummaryDto {
    /// 用户ID
    pub user_id: i32,
    /// 用户名
    pub user_name: String,
    /// 真实姓名
    pub full_name: String,
    /// 状态
    pub status: i16,
}

impl From<User> for UserSummaryDto {
    fn from(user: User) -> Self {
        Self {
            user_id: user.user_id,
            user_name: user.user_name,
            full_name: user.full_name,
            status: user.status,
        }
    }
}

/// 用户登录 DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserLoginDto {
    /// 用户名
    pub user_name: String,
    /// 密码
    pub password: String,
}

/// 用户登录响应 DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserLoginResponseDto {
    /// 用户信息
    pub user: UserDto,
    /// 访问令牌
    pub access_token: String,
    /// 刷新令牌
    pub refresh_token: Option<String>,
    /// 过期时间（秒）
    pub expires_in: i64,
}

/// 用户创建 DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserCreateDto {
    /// 用户名
    pub user_name: String,
    /// 密码
    pub password: String,
    /// 真实姓名
    pub full_name: String,
    /// 电话号码
    pub phone_number: Option<String>,
    /// 邮箱
    pub email: Option<String>,
    /// 用户组ID
    pub user_group_id: i32,
    /// 状态
    pub status: i16,
}

/// 用户更新 DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserUpdateDto {
    /// 密码
    pub password: Option<String>,
    /// 真实姓名
    pub full_name: Option<String>,
    /// 电话号码
    pub phone_number: Option<String>,
    /// 邮箱
    pub email: Option<String>,
    /// 用户组ID
    pub user_group_id: Option<i32>,
    /// 状态
    pub status: Option<i16>,
}
