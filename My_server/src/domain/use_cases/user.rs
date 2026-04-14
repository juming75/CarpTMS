//! / 用户领域用例

use std::sync::Arc;

use crate::domain::entities::user::{User, UserCreate, UserQuery, UserUpdate};
use crate::utils::password::hash_password;

/// 用户仓库接口
#[async_trait::async_trait]
pub trait UserRepository: Send + Sync {
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

/// 用户用例结构
#[derive(Clone)]
pub struct UserUseCases {
    user_repository: Arc<dyn UserRepository + Send + Sync>,
}

impl UserUseCases {
    /// 创建用户用例实例
    pub fn new(user_repository: Arc<dyn UserRepository>) -> Self {
        Self { user_repository }
    }

    /// 获取用户列表用例
    pub async fn get_users(&self, query: UserQuery) -> Result<(Vec<User>, i64), anyhow::Error> {
        // 业务逻辑:可以在这里添加额外的业务规则,例如权限检查等
        let (users, total) = self.user_repository.get_users(query).await?;

        // 解密敏感信息
        use futures::stream::{self, StreamExt};

        let decrypted_users: Vec<User> = stream::iter(users)
            .map(|user| async move {
                let decrypted_phone: Option<String> =
                    if let Some(phone) = user.phone_number.as_ref() {
                        crate::utils::encryption::decrypt_string(phone).await.ok()
                    } else {
                        None
                    };

                let decrypted_email: Option<String> = if let Some(email) = user.email.as_ref() {
                    crate::utils::encryption::decrypt_string(email).await.ok()
                } else {
                    None
                };

                User {
                    user_id: user.user_id,
                    user_name: user.user_name,
                    password: user.password, // 密码保持哈希状态
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
            .buffer_unordered(10) // 并行处理10个用户
            .collect()
            .await;

        Ok((decrypted_users, total))
    }

    /// 获取单个用户用例
    pub async fn get_user(&self, user_id: i32) -> Result<Option<User>, anyhow::Error> {
        // 业务逻辑:可以在这里添加额外的业务规则,例如权限检查等
        let user = self.user_repository.get_user(user_id).await?;

        // 解密敏感信息
        let decrypted_user = if let Some(user) = user {
            let decrypted_phone: Option<String> = if let Some(phone) = user.phone_number.as_ref() {
                crate::utils::encryption::decrypt_string(phone).await.ok()
            } else {
                None
            };

            let decrypted_email: Option<String> = if let Some(email) = user.email.as_ref() {
                crate::utils::encryption::decrypt_string(email).await.ok()
            } else {
                None
            };

            Some(User {
                user_id: user.user_id,
                user_name: user.user_name,
                password: user.password, // 密码保持哈希状态
                full_name: user.full_name,
                phone_number: decrypted_phone,
                email: decrypted_email,
                user_group_id: user.user_group_id,
                status: user.status,
                last_login_time: user.last_login_time,
                create_time: user.create_time,
                update_time: user.update_time,
            })
        } else {
            None
        };

        Ok(decrypted_user)
    }

    /// 根据用户名获取用户用例
    pub async fn get_user_by_name(&self, user_name: &str) -> Result<Option<User>, anyhow::Error> {
        // 业务逻辑:可以在这里添加额外的业务规则,例如权限检查等
        if user_name.is_empty() {
            return Err(anyhow::anyhow!("用户名不能为空"));
        }

        let user = self.user_repository.get_user_by_name(user_name).await?;

        // 解密敏感信息
        let decrypted_user = if let Some(user) = user {
            let decrypted_phone: Option<String> = if let Some(phone) = user.phone_number.as_ref() {
                crate::utils::encryption::decrypt_string(phone).await.ok()
            } else {
                None
            };

            let decrypted_email: Option<String> = if let Some(email) = user.email.as_ref() {
                crate::utils::encryption::decrypt_string(email).await.ok()
            } else {
                None
            };

            Some(User {
                user_id: user.user_id,
                user_name: user.user_name,
                password: user.password, // 密码保持哈希状态
                full_name: user.full_name,
                phone_number: decrypted_phone,
                email: decrypted_email,
                user_group_id: user.user_group_id,
                status: user.status,
                last_login_time: user.last_login_time,
                create_time: user.create_time,
                update_time: user.update_time,
            })
        } else {
            None
        };

        Ok(decrypted_user)
    }

    /// 创建用户用例
    pub async fn create_user(&self, user: UserCreate) -> Result<User, anyhow::Error> {
        // 业务逻辑:数据验证
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
        if let Some(existing_user) = self
            .user_repository
            .get_user_by_name(&user.user_name)
            .await?
        {
            return Err(anyhow::anyhow!("用户名已存在: {}", existing_user.user_name));
        }

        // 处理密码哈希
        let hashed_password = hash_password(&user.password)?;

        // 加密敏感信息
        let encrypted_phone: Option<String> = if let Some(phone) = user.phone_number.as_ref() {
            if !phone.is_empty() {
                crate::utils::encryption::encrypt_string(phone).await.ok()
            } else {
                None
            }
        } else {
            None
        };

        let encrypted_email: Option<String> = if let Some(email) = user.email.as_ref() {
            if !email.is_empty() {
                crate::utils::encryption::encrypt_string(email).await.ok()
            } else {
                None
            }
        } else {
            None
        };

        // 创建加密后的用户数据
        let encrypted_user = UserCreate {
            user_name: user.user_name,
            password: hashed_password,
            full_name: user.full_name,
            phone_number: encrypted_phone,
            email: encrypted_email,
            user_group_id: user.user_group_id,
            status: user.status,
        };

        // 调用仓库创建用户
        let created_user = self.user_repository.create_user(encrypted_user).await?;

        // 解密敏感信息用于返回
        let decrypted_phone: Option<String> =
            if let Some(phone) = created_user.phone_number.as_ref() {
                crate::utils::encryption::decrypt_string(phone).await.ok()
            } else {
                None
            };

        let decrypted_email: Option<String> = if let Some(email) = created_user.email.as_ref() {
            crate::utils::encryption::decrypt_string(email).await.ok()
        } else {
            None
        };

        // 返回解密后的用户信息
        Ok(User {
            user_id: created_user.user_id,
            user_name: created_user.user_name,
            password: created_user.password, // 密码保持哈希状态
            full_name: created_user.full_name,
            phone_number: decrypted_phone,
            email: decrypted_email,
            user_group_id: created_user.user_group_id,
            status: created_user.status,
            last_login_time: created_user.last_login_time,
            create_time: created_user.create_time,
            update_time: created_user.update_time,
        })
    }

    /// 更新用户用例
    pub async fn update_user(
        &self,
        user_id: i32,
        user: UserUpdate,
    ) -> Result<Option<User>, anyhow::Error> {
        // 业务逻辑:可以在这里添加额外的业务规则,例如数据验证、权限检查等
        if let Some(password) = &user.password {
            if password.len() < 6 {
                return Err(anyhow::anyhow!("密码长度不能少于6个字符"));
            }
        }

        // 处理密码哈希和敏感数据加密
        let hashed_password = user.password.as_ref().and_then(|password| {
            if !password.is_empty() {
                hash_password(password).ok()
            } else {
                None
            }
        });

        // 加密敏感信息
        let encrypted_phone: Option<String> = if let Some(phone) = user.phone_number.as_ref() {
            if !phone.is_empty() {
                crate::utils::encryption::encrypt_string(phone).await.ok()
            } else {
                None
            }
        } else {
            None
        };

        let encrypted_email: Option<String> = if let Some(email) = user.email.as_ref() {
            if !email.is_empty() {
                crate::utils::encryption::encrypt_string(email).await.ok()
            } else {
                None
            }
        } else {
            None
        };

        // 创建加密后的用户更新数据
        let encrypted_user_update = UserUpdate {
            password: hashed_password,
            full_name: user.full_name,
            phone_number: encrypted_phone,
            email: encrypted_email,
            user_group_id: user.user_group_id,
            status: user.status,
            last_login_time: user.last_login_time,
        };

        // 调用仓库更新用户
        let updated_user = self
            .user_repository
            .update_user(user_id, encrypted_user_update)
            .await?;

        // 解密敏感信息用于返回
        let decrypted_user = if let Some(user) = updated_user {
            let decrypted_phone: Option<String> = if let Some(phone) = user.phone_number.as_ref() {
                crate::utils::encryption::decrypt_string(phone).await.ok()
            } else {
                None
            };

            let decrypted_email: Option<String> = if let Some(email) = user.email.as_ref() {
                crate::utils::encryption::decrypt_string(email).await.ok()
            } else {
                None
            };

            Some(User {
                user_id: user.user_id,
                user_name: user.user_name,
                password: user.password, // 密码保持哈希状态
                full_name: user.full_name,
                phone_number: decrypted_phone,
                email: decrypted_email,
                user_group_id: user.user_group_id,
                status: user.status,
                last_login_time: user.last_login_time,
                create_time: user.create_time,
                update_time: user.update_time,
            })
        } else {
            None
        };

        Ok(decrypted_user)
    }

    /// 删除用户用例
    pub async fn delete_user(&self, user_id: i32) -> Result<bool, anyhow::Error> {
        // 业务逻辑:可以在这里添加额外的业务规则,例如权限检查、关联数据检查等
        // 例如:检查该用户是否是最后一个管理员用户,如果是则不允许删除

        // 调用仓库删除用户
        self.user_repository.delete_user(user_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDateTime;
    use std::sync::Arc;

    struct MockUserRepo {
        users: Vec<User>,
    }

    #[async_trait::async_trait]
    impl UserRepository for MockUserRepo {
        async fn get_users(&self, _query: UserQuery) -> Result<(Vec<User>, i64), anyhow::Error> {
            Ok((self.users.clone(), self.users.len() as i64))
        }

        async fn get_user(&self, user_id: i32) -> Result<Option<User>, anyhow::Error> {
            Ok(self.users.iter().find(|u| u.user_id == user_id).cloned())
        }

        async fn get_user_by_name(&self, user_name: &str) -> Result<Option<User>, anyhow::Error> {
            Ok(self
                .users
                .iter()
                .find(|u| u.user_name == user_name)
                .cloned())
        }

        async fn create_user(&self, user: UserCreate) -> Result<User, anyhow::Error> {
            let now =
                NaiveDateTime::parse_from_str("2026-01-13 10:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
            let new_user = User {
                user_id: self.users.len() as i32 + 1,
                user_name: user.user_name,
                password: user.password,
                full_name: user.full_name,
                phone_number: user.phone_number,
                email: user.email,
                user_group_id: user.user_group_id,
                status: user.status,
                last_login_time: None,
                create_time: now,
                update_time: None,
            };
            Ok(new_user)
        }

        async fn update_user(
            &self,
            user_id: i32,
            user: UserUpdate,
        ) -> Result<Option<User>, anyhow::Error> {
            if let Some(mut existing_user) = self.get_user(user_id).await? {
                if let Some(password) = user.password {
                    existing_user.password = password;
                }
                if let Some(full_name) = user.full_name {
                    existing_user.full_name = full_name;
                }
                if let Some(phone_number) = user.phone_number {
                    existing_user.phone_number = Some(phone_number);
                }
                if let Some(email) = user.email {
                    existing_user.email = Some(email);
                }
                if let Some(user_group_id) = user.user_group_id {
                    existing_user.user_group_id = user_group_id;
                }
                if let Some(status) = user.status {
                    existing_user.status = status;
                }
                if let Some(last_login_time) = user.last_login_time {
                    existing_user.last_login_time = Some(last_login_time);
                }
                Ok(Some(existing_user))
            } else {
                Ok(None)
            }
        }

        async fn delete_user(&self, user_id: i32) -> Result<bool, anyhow::Error> {
            Ok(self.users.iter().any(|u| u.user_id == user_id))
        }
    }

    // 测试用例:获取用户列表
    #[tokio::test]
    async fn test_get_users() {
        // 准备测试数据
        let now =
            NaiveDateTime::parse_from_str("2026-01-13 10:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
        let users = vec![
            User {
                user_id: 1,
                user_name: "admin".to_string(),
                password: "password123".to_string(),
                full_name: "管理员".to_string(),
                phone_number: Some("13800138000".to_string()),
                email: Some("admin@example.com".to_string()),
                user_group_id: 1,
                status: 1,
                last_login_time: None,
                create_time: now,
                update_time: None,
            },
            User {
                user_id: 2,
                user_name: "user1".to_string(),
                password: "password456".to_string(),
                full_name: "用户1".to_string(),
                phone_number: None,
                email: Some("user1@example.com".to_string()),
                user_group_id: 2,
                status: 1,
                last_login_time: None,
                create_time: now,
                update_time: None,
            },
        ];

        // 创建模拟仓库
        let mock_repo = Arc::new(MockUserRepo {
            users: users.clone(),
        });

        // 创建用例实例
        let use_cases = UserUseCases::new(mock_repo);

        // 执行测试
        let query = UserQuery::default();
        let result = use_cases.get_users(query).await;

        // 验证结果
        assert!(result.is_ok());
        let (result_users, result_total) = result.unwrap();
        assert_eq!(result_users, users);
        assert_eq!(result_total, 2);
    }

    // 测试用例:获取单个用户
    #[tokio::test]
    async fn test_get_user() {
        // 准备测试数据
        let now =
            NaiveDateTime::parse_from_str("2026-01-13 10:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
        let user = User {
            user_id: 1,
            user_name: "admin".to_string(),
            password: "password123".to_string(),
            full_name: "管理员".to_string(),
            phone_number: Some("13800138000".to_string()),
            email: Some("admin@example.com".to_string()),
            user_group_id: 1,
            status: 1,
            last_login_time: None,
            create_time: now,
            update_time: None,
        };

        // 创建模拟仓库
        let mock_repo = Arc::new(MockUserRepo {
            users: vec![user.clone()],
        });

        // 创建用例实例
        let use_cases = UserUseCases::new(mock_repo);

        // 执行测试
        let result = use_cases.get_user(1).await;

        // 验证结果
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some(user));
    }

    // 测试用例:根据用户名获取用户
    #[tokio::test]
    async fn test_get_user_by_name() {
        // 准备测试数据
        let now =
            NaiveDateTime::parse_from_str("2026-01-13 10:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
        let user = User {
            user_id: 1,
            user_name: "admin".to_string(),
            password: "password123".to_string(),
            full_name: "管理员".to_string(),
            phone_number: Some("13800138000".to_string()),
            email: Some("admin@example.com".to_string()),
            user_group_id: 1,
            status: 1,
            last_login_time: None,
            create_time: now,
            update_time: None,
        };

        // 创建模拟仓库
        let mock_repo = Arc::new(MockUserRepo {
            users: vec![user.clone()],
        });

        // 创建用例实例
        let use_cases = UserUseCases::new(mock_repo);

        // 执行测试
        let result = use_cases.get_user_by_name("admin").await;

        // 验证结果
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some(user));
    }

    // 测试用例:创建用户成功
    #[tokio::test]
    async fn test_create_user_success() {
        // 准备测试数据
        let user_create = UserCreate {
            user_name: "newuser".to_string(),
            password: "password123".to_string(),
            full_name: "新用户".to_string(),
            phone_number: Some("13800138000".to_string()),
            email: Some("newuser@example.com".to_string()),
            user_group_id: 2,
            status: 1,
        };

        // 创建模拟仓库
        let mock_repo = Arc::new(MockUserRepo { users: Vec::new() });

        // 创建用例实例
        let use_cases = UserUseCases::new(mock_repo);

        // 执行测试
        let result = use_cases.create_user(user_create).await;

        // 验证结果
        assert!(result.is_ok());
        let user = result.unwrap();
        assert_eq!(user.user_name, "newuser");
        assert_eq!(user.full_name, "新用户");
    }

    // 测试用例:创建用户失败 - 用户名已存在
    #[tokio::test]
    async fn test_create_user_duplicate_username() {
        // 准备测试数据
        let now =
            NaiveDateTime::parse_from_str("2026-01-13 10:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
        let existing_user = User {
            user_id: 1,
            user_name: "existing".to_string(),
            password: "password123".to_string(),
            full_name: "现有用户".to_string(),
            phone_number: Some("13800138000".to_string()),
            email: Some("existing@example.com".to_string()),
            user_group_id: 2,
            status: 1,
            last_login_time: None,
            create_time: now,
            update_time: None,
        };

        let user_create = UserCreate {
            user_name: "existing".to_string(),
            password: "password456".to_string(),
            full_name: "重复用户".to_string(),
            phone_number: Some("13900139000".to_string()),
            email: Some("duplicate@example.com".to_string()),
            user_group_id: 2,
            status: 1,
        };

        // 创建模拟仓库
        let mock_repo = Arc::new(MockUserRepo {
            users: vec![existing_user.clone()],
        });

        // 创建用例实例
        let use_cases = UserUseCases::new(mock_repo);

        // 执行测试
        let result = use_cases.create_user(user_create).await;

        // 验证结果
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("用户名已存在"));
    }

    // 测试用例:创建用户失败 - 用户名太短
    #[tokio::test]
    async fn test_create_user_short_username() {
        // 准备测试数据
        let user_create = UserCreate {
            user_name: "ab".to_string(),
            password: "password123".to_string(),
            full_name: "短用户名用户".to_string(),
            phone_number: Some("13800138000".to_string()),
            email: Some("short@example.com".to_string()),
            user_group_id: 2,
            status: 1,
        };

        // 创建模拟仓库
        let mock_repo = Arc::new(MockUserRepo { users: Vec::new() });

        // 创建用例实例
        let use_cases = UserUseCases::new(mock_repo);

        // 执行测试
        let result = use_cases.create_user(user_create).await;

        // 验证结果
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "用户名长度不能少于3个字符");
    }

    // 测试用例:创建用户失败 - 密码太短
    #[tokio::test]
    async fn test_create_user_short_password() {
        // 准备测试数据
        let user_create = UserCreate {
            user_name: "shortpass".to_string(),
            password: "123".to_string(),
            full_name: "短密码用户".to_string(),
            phone_number: Some("13800138000".to_string()),
            email: Some("shortpass@example.com".to_string()),
            user_group_id: 2,
            status: 1,
        };

        // 创建模拟仓库
        let mock_repo = Arc::new(MockUserRepo { users: Vec::new() });

        // 创建用例实例
        let use_cases = UserUseCases::new(mock_repo);

        // 执行测试
        let result = use_cases.create_user(user_create).await;

        // 验证结果
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "密码长度不能少于6个字符");
    }

    // 测试用例:更新用户
    #[tokio::test]
    async fn test_update_user() {
        // 准备测试数据
        let now =
            NaiveDateTime::parse_from_str("2026-01-13 10:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
        let user = User {
            user_id: 1,
            user_name: "admin".to_string(),
            password: "password123".to_string(),
            full_name: "管理员".to_string(),
            phone_number: Some("13800138000".to_string()),
            email: Some("admin@example.com".to_string()),
            user_group_id: 1,
            status: 1,
            last_login_time: None,
            create_time: now,
            update_time: None,
        };

        let user_update = UserUpdate {
            password: Some("newpassword456".to_string()),
            full_name: Some("更新管理员".to_string()),
            ..Default::default()
        };

        // 创建模拟仓库
        let mock_repo = Arc::new(MockUserRepo {
            users: vec![user.clone()],
        });

        // 创建用例实例
        let use_cases = UserUseCases::new(mock_repo);

        // 执行测试
        let result = use_cases.update_user(1, user_update).await;

        // 验证结果
        assert!(result.is_ok());
        let updated_user = result.unwrap().unwrap();
        assert_eq!(updated_user.full_name, "更新管理员");
        assert_eq!(updated_user.password, "newpassword456");
    }

    // 测试用例:删除用户
    #[tokio::test]
    async fn test_delete_user() {
        // 准备测试数据
        let now =
            NaiveDateTime::parse_from_str("2026-01-13 10:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
        let user = User {
            user_id: 1,
            user_name: "admin".to_string(),
            password: "password123".to_string(),
            full_name: "管理员".to_string(),
            phone_number: Some("13800138000".to_string()),
            email: Some("admin@example.com".to_string()),
            user_group_id: 1,
            status: 1,
            last_login_time: None,
            create_time: now,
            update_time: None,
        };

        // 创建模拟仓库
        let mock_repo = Arc::new(MockUserRepo {
            users: vec![user.clone()],
        });

        // 创建用例实例
        let use_cases = UserUseCases::new(mock_repo);

        // 执行测试
        let result = use_cases.delete_user(1).await;

        // 验证结果
        assert!(result.is_ok());
        assert!(result.unwrap());
    }
}
