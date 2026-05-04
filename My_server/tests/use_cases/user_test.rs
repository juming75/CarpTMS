//! 用户用例集成测试
//!
//! 独立的集成测试，不依赖内嵌测试模块

use std::sync::Arc;
use carptms::domain::use_cases::user::UserUseCases;
use carptms::domain::entities::user::{User, UserCreate, UserQuery, UserUpdate};
use carptms::domain::use_cases::user::repository::UserRepository;
use chrono::NaiveDateTime;

#[allow(dead_code)]
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
        Ok(User {
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
        })
    }

    async fn update_user(
        &self,
        user_id: i32,
        user: UserUpdate,
    ) -> Result<Option<User>, anyhow::Error> {
        if let Some(mut existing) = self.get_user(user_id).await? {
            if let Some(pwd) = user.password {
                existing.password = pwd;
            }
            if let Some(name) = user.full_name {
                existing.full_name = name;
            }
            if let Some(phone) = user.phone_number {
                existing.phone_number = Some(phone);
            }
            if let Some(email) = user.email {
                existing.email = Some(email);
            }
            if let Some(gid) = user.user_group_id {
                existing.user_group_id = gid;
            }
            if let Some(status) = user.status {
                existing.status = status;
            }
            if let Some(time) = user.last_login_time {
                existing.last_login_time = Some(time);
            }
            Ok(Some(existing))
        } else {
            Ok(None)
        }
    }

    async fn delete_user(&self, user_id: i32) -> Result<bool, anyhow::Error> {
        Ok(self.users.iter().any(|u| u.user_id == user_id))
    }
}

#[tokio::test]
async fn test_create_user_success() {
    let user_create = UserCreate {
        user_name: "newuser".to_string(),
        password: "password123".to_string(),
        full_name: "新用户".to_string(),
        phone_number: Some("13800138000".to_string()),
        email: Some("newuser@example.com".to_string()),
        user_group_id: 2,
        status: 1,
    };

    let mock_repo = Arc::new(MockUserRepo { users: vec![] });
    let use_cases = UserUseCases::new(mock_repo);
    let result: Result<User, anyhow::Error> = use_cases.create_user(user_create).await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap().user_name, "newuser");
}

#[tokio::test]
async fn test_create_user_short_username() {
    let user_create = UserCreate {
        user_name: "ab".to_string(),
        password: "password123".to_string(),
        full_name: "短用户名用户".to_string(),
        phone_number: None,
        email: None,
        user_group_id: 2,
        status: 1,
    };

    let mock_repo = Arc::new(MockUserRepo { users: vec![] });
    let use_cases = UserUseCases::new(mock_repo);
    let result: Result<User, anyhow::Error> = use_cases.create_user(user_create).await;

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().to_string(), "用户名长度不能少于3个字符");
}

#[tokio::test]
async fn test_create_user_short_password() {
    let user_create = UserCreate {
        user_name: "shortpass".to_string(),
        password: "123".to_string(),
        full_name: "短密码用户".to_string(),
        phone_number: None,
        email: None,
        user_group_id: 2,
        status: 1,
    };

    let mock_repo = Arc::new(MockUserRepo { users: vec![] });
    let use_cases = UserUseCases::new(mock_repo);
    let result: Result<User, anyhow::Error> = use_cases.create_user(user_create).await;

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().to_string(), "密码长度不能少于6个字符");
}
