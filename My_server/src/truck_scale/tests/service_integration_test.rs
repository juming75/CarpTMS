//! / // 服务集成测试

use super::setup_test_db;
use crate::truck_scale::integration::ServiceIntegration;
use crate::truck_scale::protocol::message_protocol::*;
use chrono;
use std::sync::Arc;

#[tokio::test]
async fn test_handle_login() {
    // 设置测试数据库
    let pool = Arc::new(setup_test_db().await.unwrap());
    let integration = ServiceIntegration::new(pool);
    
    // 创建测试登录请求
    let login_request = LoginRequest {
        username: "test_user".to_string(),
        password: "test_password".to_string(),
        device_id: "test_device".to_string(),
    };
    
    let header = MessageHeader {
        message_type: MessageType::Login,
        session_id: Some("test_session".to_string()),
        device_id: "test_device".to_string(),
        timestamp: chrono::Utc::now(),
    };
    
    // 创建统一消息
    let message = UnifiedMessage::new(
        MessageType::Login,
        MessageBody::Login(login_request),
    );
    
    // 测试处理登录请求
    let result = integration.handle_message(message).await;
    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(matches!(response.body, MessageBody::LoginResponse(_)));
}

#[tokio::test]
async fn test_handle_heartbeat() {
    // 设置测试数据库
    let pool = Arc::new(setup_test_db().await.unwrap());
    let integration = ServiceIntegration::new(pool);
    
    // 创建测试心跳请求
    let heartbeat_request = Heartbeat {
        timestamp: chrono::Utc::now(),
        status: "ok".to_string(),
    };
    
    let header = MessageHeader {
        message_type: MessageType::Heartbeat,
        session_id: Some("test_session".to_string()),
        device_id: "test_device".to_string(),
        timestamp: chrono::Utc::now(),
    };
    
    // 创建统一消息
    let message = UnifiedMessage::new(
        MessageType::Heartbeat,
        MessageBody::Heartbeat(heartbeat_request),
    );
    
    // 测试处理心跳请求
    let result = integration.handle_message(message).await;
    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(matches!(response.body, MessageBody::Heartbeat(_)));
}






