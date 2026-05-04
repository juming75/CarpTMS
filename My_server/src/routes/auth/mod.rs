//! 认证路由模块
//!
//! 子模块：
//! - [`handlers`]：认证业务处理器（登录/刷新/登出/用户信息/改密）
//! - [`cookies`]：Cookie 工具函数

pub mod cookies;
pub mod handlers;

pub use cookies::secure_cookie;
pub use handlers::{
    change_password, configure_auth_routes, get_current_user, get_current_user_by_token, login,
    logout, refresh_token,
};
