//! / 用户仓库测试

use sqlx::{PgPool, Executor}; 
use anyhow::Result;
use std::sync::Arc;

use crate::domain::entities::user::{User, UserCreate, UserQuery, UserUpdate};
use crate::infrastructure::repositories::user_repository::PgUserRepository;
use crate::domain::use_cases::user::UserRepository;

// 测试前的设置:创建测试数据库连接
async fn setup_test_db() -> PgPool {
    // 使用测试数据库连接
    let pool = PgPool::connect("postgresql://postgres:password@localhost:5432/test_tms_db")
        .await
        .expect("Failed to connect to test database");
    
    // 创建测试所需的表
    pool.execute(r#"
        CREATE TABLE IF NOT EXISTS users (
            user_id SERIAL PRIMARY KEY,
            user_name VARCHAR(50) UNIQUE NOT NULL,
            password VARCHAR(255) NOT NULL,
            full_name VARCHAR(100) NOT NULL,
            phone VARCHAR(20),
            email VARCHAR(100),
            role VARCHAR(20) NOT NULL,
            status VARCHAR(20) NOT NULL DEFAULT 'active',
            create_time TIMESTAMP NOT NULL DEFAULT NOW(),
            update_time TIMESTAMP
        );
    "#).await.expect("Failed to create test tables");
    
    pool
}

// 测试后的清理:删除测试数据
async fn cleanup_test_db(pool: &PgPool) {
    pool.execute("TRUNCATE TABLE users RESTART IDENTITY CASCADE").await.unwrap();
}

// 测试用例:创建用户
#[tokio::test]
async fn test_create_user() -> Result<()> {
    // 设置测试环境
    let pool = setup_test_db().await;
    let repo = PgUserRepository::new(Arc::new(pool.clone()));
    
    // 准备测试数据
    let user_create = UserCreate {
        user_name: "test_user".to_string(),
        password: "password123".to_string(),
        full_name: "测试用户".to_string(),
        phone: Some("13800138000".to_string()),
        email: Some("test@example.com".to_string()),
        role: "admin".to_string(),
    };
    
    // 执行测试
    let result = repo.create_user(user_create).await?;
    
    // 验证结果
    assert!(result.user_id > 0);
    assert_eq!(result.user_name, "test_user");
    assert_eq!(result.full_name, "测试用户");
    assert_eq!(result.role, "admin");
    
    // 清理测试数据
    cleanup_test_db(&pool).await;
    
    Ok(())
}

// 测试用例:获取用户列表
#[tokio::test]
async fn test_get_users() -> Result<()> {
    // 设置测试环境
    let pool = setup_test_db().await;
    let repo = PgUserRepository::new(Arc::new(pool.clone()));
    
    // 创建测试数据
    let user_create = UserCreate {
        user_name: "test_user2".to_string(),
        password: "password123".to_string(),
        full_name: "测试用户2".to_string(),
        phone: Some("13800138000".to_string()),
        email: Some("test@example.com".to_string()),
        role: "admin".to_string(),
    };
    repo.create_user(user_create).await?;
    
    // 执行测试
    let query = UserQuery {
        user_name: Some("test".to_string()),
        ..Default::default()
    };
    let result = repo.get_users(query).await?;
    
    // 验证结果
    assert!(result.0.len() > 0);
    assert!(result.1 > 0);
    
    // 清理测试数据
    cleanup_test_db(&pool).await;
    
    Ok(())
}

// 测试用例:获取单个用户
#[tokio::test]
async fn test_get_user() -> Result<()> {
    // 设置测试环境
    let pool = setup_test_db().await;
    let repo = PgUserRepository::new(Arc::new(pool.clone()));
    
    // 创建测试数据
    let user_create = UserCreate {
        user_name: "test_user3".to_string(),
        password: "password123".to_string(),
        full_name: "测试用户3".to_string(),
        phone: Some("13800138000".to_string()),
        email: Some("test@example.com".to_string()),
        role: "admin".to_string(),
    };
    let created_user = repo.create_user(user_create).await?;
    
    // 执行测试
    let result = repo.get_user(created_user.user_id).await?;
    
    // 验证结果
    assert!(result.is_some());
    let user = result.unwrap();
    assert_eq!(user.user_id, created_user.user_id);
    assert_eq!(user.user_name, "test_user3");
    
    // 清理测试数据
    cleanup_test_db(&pool).await;
    
    Ok(())
}

// 测试用例:根据用户名获取用户
#[tokio::test]
async fn test_get_user_by_name() -> Result<()> {
    // 设置测试环境
    let pool = setup_test_db().await;
    let repo = PgUserRepository::new(Arc::new(pool.clone()));
    
    // 创建测试数据
    let user_create = UserCreate {
        user_name: "test_user4".to_string(),
        password: "password123".to_string(),
        full_name: "测试用户4".to_string(),
        phone: Some("13800138000".to_string()),
        email: Some("test@example.com".to_string()),
        role: "admin".to_string(),
    };
    repo.create_user(user_create).await?;
    
    // 执行测试
    let result = repo.get_user_by_name("test_user4").await?;
    
    // 验证结果
    assert!(result.is_some());
    let user = result.unwrap();
    assert_eq!(user.user_name, "test_user4");
    
    // 清理测试数据
    cleanup_test_db(&pool).await;
    
    Ok(())
}

// 测试用例:更新用户
#[tokio::test]
async fn test_update_user() -> Result<()> {
    // 设置测试环境
    let pool = setup_test_db().await;
    let repo = PgUserRepository::new(Arc::new(pool.clone()));
    
    // 创建测试数据
    let user_create = UserCreate {
        user_name: "test_user5".to_string(),
        password: "password123".to_string(),
        full_name: "测试用户5".to_string(),
        phone: Some("13800138000".to_string()),
        email: Some("test@example.com".to_string()),
        role: "admin".to_string(),
    };
    let created_user = repo.create_user(user_create).await?;
    
    // 执行测试
    let user_update = UserUpdate {
        full_name: Some("更新后的测试用户".to_string()),
        phone: Some("13900139000".to_string()),
        ..Default::default()
    };
    let result = repo.update_user(created_user.user_id, user_update).await?;
    
    // 验证结果
    assert!(result.is_some());
    let updated_user = result.unwrap();
    assert_eq!(updated_user.user_id, created_user.user_id);
    assert_eq!(updated_user.full_name, "更新后的测试用户");
    assert_eq!(updated_user.phone.unwrap(), "13900139000");
    
    // 清理测试数据
    cleanup_test_db(&pool).await;
    
    Ok(())
}

// 测试用例:删除用户
#[tokio::test]
async fn test_delete_user() -> Result<()> {
    // 设置测试环境
    let pool = setup_test_db().await;
    let repo = PgUserRepository::new(Arc::new(pool.clone()));
    
    // 创建测试数据
    let user_create = UserCreate {
        user_name: "test_user6".to_string(),
        password: "password123".to_string(),
        full_name: "测试用户6".to_string(),
        phone: Some("13800138000".to_string()),
        email: Some("test@example.com".to_string()),
        role: "admin".to_string(),
    };
    let created_user = repo.create_user(user_create).await?;
    
    // 执行测试
    let result = repo.delete_user(created_user.user_id).await?;
    
    // 验证结果
    assert!(result);
    
    // 验证用户已被删除
    let get_result = repo.get_user(created_user.user_id).await?;
    assert!(get_result.is_none());
    
    // 清理测试数据
    cleanup_test_db(&pool).await;
    
    Ok(())
}






