//! 用户仓库接口

use crate::domain::entities::user::{User, UserCreate, UserQuery, UserUpdate};

/// 用户仓库接口
#[async_trait::async_trait]
pub trait UserRepository: Send + Sync {
    async fn get_users(&self, query: UserQuery) -> Result<(Vec<User>, i64), anyhow::Error>;
    async fn get_user(&self, user_id: i32) -> Result<Option<User>, anyhow::Error>;
    async fn get_user_by_name(&self, user_name: &str) -> Result<Option<User>, anyhow::Error>;
    async fn create_user(&self, user: UserCreate) -> Result<User, anyhow::Error>;
    async fn update_user(
        &self,
        user_id: i32,
        user: UserUpdate,
    ) -> Result<Option<User>, anyhow::Error>;
    async fn delete_user(&self, user_id: i32) -> Result<bool, anyhow::Error>;
}
