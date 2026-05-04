//! / // 用户管理处理器测试

use super::setup_test_db;
use crate::truck_scale::handlers::UserHandler;
use crate::truck_scale::handlers::UserInfo;
use chrono;
use uuid;
use std::sync::Arc;

#[tokio::test]
async fn test_query_user() {
    // 设置测试数据库
    let pool = Arc::new(setup_test_db().await.unwrap());
    let handler = UserHandler::new(pool);
    
    // 测试查询不存在的用户
    let result = handler.query_user("non_existent_user").await;
    assert!(result.is_ok());
    let user = result.unwrap();
    assert!(user.is_none());
}

#[tokio::test]
async fn test_query_user_by_name() {
    // 设置测试数据库
    let pool = Arc::new(setup_test_db().await.unwrap());
    let handler = UserHandler::new(pool);
    
    // 测试查询不存在的用户
    let result = handler.query_user_by_name("non_existent_user").await;
    assert!(result.is_ok());
    let user = result.unwrap();
    assert!(user.is_none());
}

#[tokio::test]
async fn test_query_user_list() {
    // 设置测试数据库
    let pool = Arc::new(setup_test_db().await.unwrap());
    let handler = UserHandler::new(pool);
    
    // 测试查询用户列表(分页)
    let result = handler.query_user_list(1, 10).await;
    assert!(result.is_ok());
    let users = result.unwrap();
    assert!(users.is_empty() || users.len() <= 10);
}

#[tokio::test]
async fn test_add_user() {
    // 设置测试数据库
    let pool = Arc::new(setup_test_db().await.unwrap());
    let handler = UserHandler::new(pool);
    
    // 创建测试用户信息
    let test_user = UserInfo {
        user_id: format!("test_user_{}", uuid::Uuid::new_v4()),
        user_name: format!("test_user_{}", uuid::Uuid::new_v4()),
        password: "test_password".to_string(),
        real_name: "测试用户".to_string(),
        user_type: 3, // 普通用户
        group_id: "test_group".to_string(),
        company: "测试公司".to_string(),
        department: "测试部门".to_string(),
        tel: "13800138000".to_string(),
        mobile: "13900139000".to_string(),
        email: "test@example.com".to_string(),
        address: "测试地址".to_string(),
        permission: "test_permission".to_string(),
        veh_group_list: "test_veh_group".to_string(),
        status: 0, // 正常
        expiration_time: "2030-12-31".to_string(),
        title: "测试职位".to_string(),
        id_card: "110101199001011234".to_string(),
        id_card_expire_date: "2030-12-31".to_string(),
        education: "本科".to_string(),
        birth_date: "1990-01-01".to_string(),
        gender: 0, // 男
        avatar: "".to_string(),
        signature: "".to_string(),
        last_login_time: "".to_string(),
        last_login_ip: "".to_string(),
        login_count: 0,
        remark: "测试用户".to_string(),
        create_time: chrono::Local::now().to_string(),
        update_time: chrono::Local::now().to_string(),
        create_by: "test_admin".to_string(),
        update_by: "test_admin".to_string(),
    };
    
    // 测试添加用户
    let result = handler.add_user(test_user.clone()).await;
    assert!(result.is_ok());
    let user_id = result.unwrap();
    assert_eq!(user_id, test_user.user_id);
    
    // 测试查询刚添加的用户
    let result = handler.query_user(&user_id).await;
    assert!(result.is_ok());
    let user = result.unwrap();
    assert!(user.is_some());
    
    // 测试根据用户名查询刚添加的用户
    let result = handler.query_user_by_name(&test_user.user_name).await;
    assert!(result.is_ok());
    let user = result.unwrap();
    assert!(user.is_some());
}

#[tokio::test]
async fn test_update_user() {
    // 设置测试数据库
    let pool = Arc::new(setup_test_db().await.unwrap());
    let handler = UserHandler::new(pool);
    
    // 创建测试用户信息
    let mut test_user = UserInfo {
        user_id: format!("test_user_{}", uuid::Uuid::new_v4()),
        user_name: format!("test_user_{}", uuid::Uuid::new_v4()),
        password: "test_password".to_string(),
        real_name: "测试用户".to_string(),
        user_type: 3, // 普通用户
        group_id: "test_group".to_string(),
        company: "测试公司".to_string(),
        department: "测试部门".to_string(),
        tel: "13800138000".to_string(),
        mobile: "13900139000".to_string(),
        email: "test@example.com".to_string(),
        address: "测试地址".to_string(),
        permission: "test_permission".to_string(),
        veh_group_list: "test_veh_group".to_string(),
        status: 0, // 正常
        expiration_time: "2030-12-31".to_string(),
        title: "测试职位".to_string(),
        id_card: "110101199001011234".to_string(),
        id_card_expire_date: "2030-12-31".to_string(),
        education: "本科".to_string(),
        birth_date: "1990-01-01".to_string(),
        gender: 0, // 男
        avatar: "".to_string(),
        signature: "".to_string(),
        last_login_time: "".to_string(),
        last_login_ip: "".to_string(),
        login_count: 0,
        remark: "测试用户".to_string(),
        create_time: chrono::Local::now().to_string(),
        update_time: chrono::Local::now().to_string(),
        create_by: "test_admin".to_string(),
        update_by: "test_admin".to_string(),
    };
    
    // 添加用户
    let add_result = handler.add_user(test_user.clone()).await;
    assert!(add_result.is_ok());
    let user_id = add_result.unwrap();
    
    // 更新用户信息
    test_user.real_name = "更新后的用户".to_string();
    test_user.update_by = "update_admin".to_string();
    
    let update_result = handler.update_user(test_user).await;
    assert!(update_result.is_ok());
    
    // 测试查询更新后的用户
    let query_result = handler.query_user(&user_id).await;
    assert!(query_result.is_ok());
    let user = query_result.unwrap();
    assert!(user.is_some());
}

#[tokio::test]
async fn test_delete_user() {
    // 设置测试数据库
    let pool = Arc::new(setup_test_db().await.unwrap());
    let handler = UserHandler::new(pool);
    
    // 创建测试用户信息
    let test_user = UserInfo {
        user_id: format!("test_user_{}", uuid::Uuid::new_v4()),
        user_name: format!("test_user_{}", uuid::Uuid::new_v4()),
        password: "test_password".to_string(),
        real_name: "测试用户".to_string(),
        user_type: 3, // 普通用户
        group_id: "test_group".to_string(),
        company: "测试公司".to_string(),
        department: "测试部门".to_string(),
        tel: "13800138000".to_string(),
        mobile: "13900139000".to_string(),
        email: "test@example.com".to_string(),
        address: "测试地址".to_string(),
        permission: "test_permission".to_string(),
        veh_group_list: "test_veh_group".to_string(),
        status: 0, // 正常
        expiration_time: "2030-12-31".to_string(),
        title: "测试职位".to_string(),
        id_card: "110101199001011234".to_string(),
        id_card_expire_date: "2030-12-31".to_string(),
        education: "本科".to_string(),
        birth_date: "1990-01-01".to_string(),
        gender: 0, // 男
        avatar: "".to_string(),
        signature: "".to_string(),
        last_login_time: "".to_string(),
        last_login_ip: "".to_string(),
        login_count: 0,
        remark: "测试用户".to_string(),
        create_time: chrono::Local::now().to_string(),
        update_time: chrono::Local::now().to_string(),
        create_by: "test_admin".to_string(),
        update_by: "test_admin".to_string(),
    };
    
    // 添加用户
    let add_result = handler.add_user(test_user).await;
    assert!(add_result.is_ok());
    let user_id = add_result.unwrap();
    
    // 删除用户
    let delete_result = handler.delete_user(&user_id, "delete_admin").await;
    assert!(delete_result.is_ok());
    
    // 测试查询删除后的用户(应该返回 None)
    let query_result = handler.query_user(&user_id).await;
    assert!(query_result.is_ok());
    let user = query_result.unwrap();
    assert!(user.is_none());
}






