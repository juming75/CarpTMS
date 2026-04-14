//! / 用户领域实体

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

/// 用户实体
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct User {
    /// 用户ID
    pub user_id: i32,
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
    /// 最后登录时间
    pub last_login_time: Option<NaiveDateTime>,
    /// 创建时间
    pub create_time: NaiveDateTime,
    /// 更新时间
    pub update_time: Option<NaiveDateTime>,
}

/// 用户创建实体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserCreate {
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

/// 用户更新实体
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UserUpdate {
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
    /// 最后登录时间
    pub last_login_time: Option<NaiveDateTime>,
}

/// 用户查询条件实体
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UserQuery {
    /// 页码
    pub page: Option<i32>,
    /// 每页大小
    pub page_size: Option<i32>,
    /// 用户名
    pub user_name: Option<String>,
    /// 真实姓名
    pub full_name: Option<String>,
    /// 状态
    pub status: Option<i16>,
    /// 用户组ID
    pub user_group_id: Option<i32>,
}
