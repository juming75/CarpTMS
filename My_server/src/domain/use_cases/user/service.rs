//! 用户用例实现

use std::sync::Arc;

use crate::domain::entities::user::{User, UserCreate, UserQuery, UserUpdate};
use crate::domain::use_cases::user::repository::UserRepository;
use crate::utils::password::hash_password;

/// 用户用例结构
#[derive(Clone)]
pub struct UserUseCases {
    user_repository: Arc<dyn UserRepository + Send + Sync>,
}

impl UserUseCases {
    pub fn new(user_repository: Arc<dyn UserRepository>) -> Self {
        Self { user_repository }
    }

    /// 获取用户列表
    pub async fn get_users(&self, query: UserQuery) -> Result<(Vec<User>, i64), anyhow::Error> {
        let (users, total) = self.user_repository.get_users(query).await?;

        // 解密敏感信息
        use futures::stream::{self, StreamExt};
        let decrypted_users: Vec<User> = stream::iter(users)
            .map(|user| async move {
                let decrypted_phone = if let Some(phone) = user.phone_number.as_ref() {
                    crate::utils::encryption::decrypt_string(phone).await.ok()
                } else {
                    None
                };
                let decrypted_email = if let Some(email) = user.email.as_ref() {
                    crate::utils::encryption::decrypt_string(email).await.ok()
                } else {
                    None
                };

                User {
                    user_id: user.user_id,
                    user_name: user.user_name,
                    password: user.password,
                    full_name: user.full_name,
                    phone_number: decrypted_phone,
                    email: decrypted_email,
                    user_group_id: user.user_group_id,
                    status: user.status,
                    last_login_time: user.last_login_time,
                    create_time: user.create_time,
                    update_time: user.update_time,
                }
            })
            .buffer_unordered(10)
            .collect()
            .await;

        Ok((decrypted_users, total))
    }

    /// 获取单个用户
    pub async fn get_user(&self, user_id: i32) -> Result<Option<User>, anyhow::Error> {
        let user = self.user_repository.get_user(user_id).await?;
        Ok(self.decrypt_user(user).await)
    }

    /// 根据用户名获取用户
    pub async fn get_user_by_name(&self, user_name: &str) -> Result<Option<User>, anyhow::Error> {
        if user_name.is_empty() {
            return Err(anyhow::anyhow!("用户名不能为空"));
        }
        let user = self.user_repository.get_user_by_name(user_name).await?;
        Ok(self.decrypt_user(user).await)
    }

    /// 创建用户
    pub async fn create_user(&self, user: UserCreate) -> Result<User, anyhow::Error> {
        // 数据验证
        if user.user_name.is_empty() {
            return Err(anyhow::anyhow!("用户名不能为空"));
        }
        if user.user_name.len() < 3 {
            return Err(anyhow::anyhow!("用户名长度不能少于3个字符"));
        }
        if user.password.is_empty() {
            return Err(anyhow::anyhow!("密码不能为空"));
        }
        if user.password.len() < 6 {
            return Err(anyhow::anyhow!("密码长度不能少于6个字符"));
        }
        if user.full_name.is_empty() {
            return Err(anyhow::anyhow!("真实姓名不能为空"));
        }

        // 检查用户名是否已存在
        if let Some(existing) = self
            .user_repository
            .get_user_by_name(&user.user_name)
            .await?
        {
            return Err(anyhow::anyhow!("用户名已存在: {}", existing.user_name));
        }

        // 处理密码哈希
        let hashed_password = hash_password(&user.password)?;

        // 加密敏感信息
        let encrypted_phone = if let Some(phone) = user.phone_number.as_ref() {
            if !phone.is_empty() {
                crate::utils::encryption::encrypt_string(phone).await.ok()
            } else {
                None
            }
        } else {
            None
        };

        let encrypted_email = if let Some(email) = user.email.as_ref() {
            if !email.is_empty() {
                crate::utils::encryption::encrypt_string(email).await.ok()
            } else {
                None
            }
        } else {
            None
        };

        let encrypted_user = UserCreate {
            user_name: user.user_name,
            password: hashed_password,
            full_name: user.full_name,
            phone_number: encrypted_phone,
            email: encrypted_email,
            user_group_id: user.user_group_id,
            status: user.status,
        };

        let created = self.user_repository.create_user(encrypted_user).await?;
        self.decrypt_single_user(created).await
    }

    /// 更新用户
    pub async fn update_user(
        &self,
        user_id: i32,
        user: UserUpdate,
    ) -> Result<Option<User>, anyhow::Error> {
        if let Some(password) = &user.password {
            if password.len() < 6 {
                return Err(anyhow::anyhow!("密码长度不能少于6个字符"));
            }
        }

        let hashed_password = user.password.as_ref().and_then(|p| {
            if !p.is_empty() {
                hash_password(p).ok()
            } else {
                None
            }
        });

        let encrypted_phone = if let Some(phone) = user.phone_number.as_ref() {
            if !phone.is_empty() {
                crate::utils::encryption::encrypt_string(phone).await.ok()
            } else {
                None
            }
        } else {
            None
        };

        let encrypted_email = if let Some(email) = user.email.as_ref() {
            if !email.is_empty() {
                crate::utils::encryption::encrypt_string(email).await.ok()
            } else {
                None
            }
        } else {
            None
        };

        let encrypted_update = UserUpdate {
            password: hashed_password,
            full_name: user.full_name,
            phone_number: encrypted_phone,
            email: encrypted_email,
            user_group_id: user.user_group_id,
            status: user.status,
            last_login_time: user.last_login_time,
        };

        let updated = self
            .user_repository
            .update_user(user_id, encrypted_update)
            .await?;
        Ok(self.decrypt_user(updated).await)
    }

    /// 删除用户
    pub async fn delete_user(&self, user_id: i32) -> Result<bool, anyhow::Error> {
        self.user_repository.delete_user(user_id).await
    }

    /// 解密用户信息
    async fn decrypt_user(&self, user: Option<User>) -> Option<User> {
        self.decrypt_single_user(user?).await.ok()
    }

    async fn decrypt_single_user(&self, user: User) -> Result<User, anyhow::Error> {
        let decrypted_phone = if let Some(phone) = user.phone_number.as_ref() {
            crate::utils::encryption::decrypt_string(phone).await.ok()
        } else {
            None
        };
        let decrypted_email = if let Some(email) = user.email.as_ref() {
            crate::utils::encryption::decrypt_string(email).await.ok()
        } else {
            None
        };

        Ok(User {
            user_id: user.user_id,
            user_name: user.user_name,
            password: user.password,
            full_name: user.full_name,
            phone_number: decrypted_phone,
            email: decrypted_email,
            user_group_id: user.user_group_id,
            status: user.status,
            last_login_time: user.last_login_time,
            create_time: user.create_time,
            update_time: user.update_time,
        })
    }
}
