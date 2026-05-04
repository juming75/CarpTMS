//! / 用户应用服务

use std::sync::Arc;

use crate::domain::entities::user::{User, UserCreate, UserQuery, UserUpdate};
use crate::domain::use_cases::user::UserUseCases;

/// 用户服务接口
#[async_trait::async_trait]
pub trait UserService: Send + Sync {
    /// 获取用户列表
    async fn get_users(&self, query: UserQuery) -> Result<(Vec<User>, i64), anyhow::Error>;

    /// 获取单个用户
    async fn get_user(&self, user_id: i32) -> Result<Option<User>, anyhow::Error>;

    /// 根据用户名获取用户
    async fn get_user_by_name(&self, user_name: &str) -> Result<Option<User>, anyhow::Error>;

    /// 创建用户
    async fn create_user(&self, user: UserCreate) -> Result<User, anyhow::Error>;

    /// 更新用户
    async fn update_user(
        &self,
        user_id: i32,
        user: UserUpdate,
    ) -> Result<Option<User>, anyhow::Error>;

    /// 删除用户
    async fn delete_user(&self, user_id: i32) -> Result<bool, anyhow::Error>;
}

/// 用户服务实现
#[derive(Clone)]
pub struct UserServiceImpl {
    user_use_cases: Arc<UserUseCases>,
}

impl UserServiceImpl {
    /// 创建用户服务实例
    pub fn new(user_use_cases: Arc<UserUseCases>) -> Self {
        Self { user_use_cases }
    }
}

#[async_trait::async_trait]
impl UserService for UserServiceImpl {
    /// 获取用户列表
    async fn get_users(&self, query: UserQuery) -> Result<(Vec<User>, i64), anyhow::Error> {
        self.user_use_cases.get_users(query).await
    }

    /// 获取单个用户
    async fn get_user(&self, user_id: i32) -> Result<Option<User>, anyhow::Error> {
        self.user_use_cases.get_user(user_id).await
    }

    /// 根据用户名获取用户
    async fn get_user_by_name(&self, user_name: &str) -> Result<Option<User>, anyhow::Error> {
        self.user_use_cases.get_user_by_name(user_name).await
    }

    /// 创建用户
    async fn create_user(&self, user: UserCreate) -> Result<User, anyhow::Error> {
        self.user_use_cases.create_user(user).await
    }

    /// 更新用户
    async fn update_user(
        &self,
        user_id: i32,
        user: UserUpdate,
    ) -> Result<Option<User>, anyhow::Error> {
        self.user_use_cases.update_user(user_id, user).await
    }

    /// 删除用户
    async fn delete_user(&self, user_id: i32) -> Result<bool, anyhow::Error> {
        self.user_use_cases.delete_user(user_id).await
    }
}
