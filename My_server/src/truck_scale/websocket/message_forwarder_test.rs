//! / WebSocket 消息转发器单元测试
use crate::truck_scale::protocol::message_protocol::*;
use crate::truck_scale::websocket::message_forwarder::*;

#[cfg(test)]
mod message_forwarder_tests {
    use super::*;

    #[tokio::test]
    async fn test_websocket_forwarder_creation() {
        let forwarder = WebSocketMessageForwarder::new();
        assert_eq!(forwarder.connection_count().await, 0);
    }

    #[tokio::test]
    async fn test_register_single_connection() {
        let forwarder = WebSocketMessageForwarder::new();
        let _rx = forwarder
            .register_connection(
                "conn_1".to_string(),
                "session_1".to_string(),
                Some("user_1".to_string()),
                Some("device_1".to_string()),
            )
            .await;

        assert_eq!(forwarder.connection_count().await, 1);
        assert_eq!(forwarder.session_connection_count("session_1").await, 1);
    }

    #[tokio::test]
    async fn test_register_multiple_connections() {
        let forwarder = WebSocketMessageForwarder::new();

        let _rx1 = forwarder
            .register_connection(
                "conn_1".to_string(),
                "session_1".to_string(),
                Some("user_1".to_string()),
                Some("device_1".to_string()),
            )
            .await;

        let _rx2 = forwarder
            .register_connection(
                "conn_2".to_string(),
                "session_2".to_string(),
                Some("user_2".to_string()),
                Some("device_2".to_string()),
            )
            .await;

        assert_eq!(forwarder.connection_count().await, 2);
        assert_eq!(forwarder.session_connection_count("session_1").await, 1);
        assert_eq!(forwarder.session_connection_count("session_2").await, 1);
    }

    #[tokio::test]
    async fn test_unregister_connection() {
        let forwarder = WebSocketMessageForwarder::new();
        let _rx = forwarder
            .register_connection("conn_1".to_string(), "session_1".to_string(), None, None)
            .await;

        assert_eq!(forwarder.connection_count().await, 1);

        forwarder.unregister_connection("conn_1").await;
        assert_eq!(forwarder.connection_count().await, 0);
    }

    #[tokio::test]
    async fn test_unregister_nonexistent_connection() {
        let forwarder = WebSocketMessageForwarder::new();
        // 不应该 panic
        forwarder.unregister_connection("nonexistent").await;
        assert_eq!(forwarder.connection_count().await, 0);
    }

    #[tokio::test]
    async fn test_send_to_session() {
        let forwarder = WebSocketMessageForwarder::new();
        let mut rx = forwarder
            .register_connection("conn_1".to_string(), "session_1".to_string(), None, None)
            .await;

        let message = UnifiedMessage::heartbeat("session_1".to_string());
        forwarder
            .send_to_session("session_1", message)
            .await
            .unwrap();

        let received = rx.recv().await.unwrap();
        assert_eq!(received.header.session_id, Some("session_1".to_string()));
    }

    #[tokio::test]
    async fn test_send_to_nonexistent_session() {
        let forwarder = WebSocketMessageForwarder::new();
        let message = UnifiedMessage::heartbeat("session_1".to_string());

        let result = forwarder
            .send_to_session("nonexistent_session", message)
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_send_to_user() {
        let forwarder = WebSocketMessageForwarder::new();

        let _rx1 = forwarder
            .register_connection(
                "conn_1".to_string(),
                "session_1".to_string(),
                Some("user_1".to_string()),
                None,
            )
            .await;

        let mut rx2 = forwarder
            .register_connection(
                "conn_2".to_string(),
                "session_2".to_string(),
                Some("user_1".to_string()), // 同一用户
                None,
            )
            .await;

        let message = UnifiedMessage::heartbeat("session_1".to_string());
        forwarder.send_to_user("user_1", message).await.unwrap();

        // 两个连接都应该接收到消息
        let _ = rx2.recv().await.unwrap();
        // conn_1 没有接收器,但消息已发送
    }

    #[tokio::test]
    async fn test_send_to_device() {
        let forwarder = WebSocketMessageForwarder::new();
        let mut rx = forwarder
            .register_connection(
                "conn_1".to_string(),
                "session_1".to_string(),
                None,
                Some("device_1".to_string()),
            )
            .await;

        let message = UnifiedMessage::heartbeat("session_1".to_string());
        forwarder.send_to_device("device_1", message).await.unwrap();

        let received = rx.recv().await.unwrap();
        assert_eq!(received.header.session_id, Some("session_1".to_string()));
    }

    #[tokio::test]
    async fn test_send_to_nonexistent_device() {
        let forwarder = WebSocketMessageForwarder::new();
        let message = UnifiedMessage::heartbeat("session_1".to_string());

        let result = forwarder
            .send_to_device("nonexistent_device", message)
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_broadcast_message() {
        let forwarder = WebSocketMessageForwarder::new();

        let mut rx1 = forwarder
            .register_connection("conn_1".to_string(), "session_1".to_string(), None, None)
            .await;

        let mut rx2 = forwarder
            .register_connection("conn_2".to_string(), "session_2".to_string(), None, None)
            .await;

        let message = UnifiedMessage::heartbeat("session_1".to_string());
        forwarder.broadcast(message).await.unwrap();

        // 两个连接都应该接收到消息
        let _ = rx1.recv().await.unwrap();
        let _ = rx2.recv().await.unwrap();
    }

    #[tokio::test]
    async fn test_broadcast_with_multiple_connections() {
        let forwarder = WebSocketMessageForwarder::new();

        let mut rxs = Vec::new();
        for i in 0..10 {
            let rx = forwarder
                .register_connection(format!("conn_{}", i), format!("session_{}", i), None, None)
                .await;
            rxs.push(rx);
        }

        assert_eq!(forwarder.connection_count().await, 10);

        let message = UnifiedMessage::heartbeat("session_1".to_string());
        forwarder.broadcast(message).await.unwrap();

        // 所有10个连接都应该接收到消息
        let mut received_count = 0;
        for mut rx in rxs {
            if rx.recv().await.is_some() {
                received_count += 1;
            }
        }
        assert_eq!(received_count, 10);
    }

    #[tokio::test]
    async fn test_multiple_connections_same_session() {
        let forwarder = WebSocketMessageForwarder::new();

        let _rx1 = forwarder
            .register_connection("conn_1".to_string(), "session_1".to_string(), None, None)
            .await;

        let _rx2 = forwarder
            .register_connection(
                "conn_2".to_string(),
                "session_1".to_string(), // 同一会话
                None,
                None,
            )
            .await;

        assert_eq!(forwarder.session_connection_count("session_1").await, 2);
        assert_eq!(forwarder.connection_count().await, 2);
    }

    #[tokio::test]
    async fn test_send_to_session_multiple_connections() {
        let forwarder = WebSocketMessageForwarder::new();

        let mut rx1 = forwarder
            .register_connection("conn_1".to_string(), "session_1".to_string(), None, None)
            .await;

        let _rx2 = forwarder
            .register_connection("conn_2".to_string(), "session_1".to_string(), None, None)
            .await;

        let message = UnifiedMessage::heartbeat("session_1".to_string());
        forwarder
            .send_to_session("session_1", message)
            .await
            .unwrap();

        // 只有一个连接应该接收到(第一个匹配的)
        let _ = rx1.recv().await.unwrap();
    }

    #[tokio::test]
    async fn test_message_serialization_in_broadcast() {
        let forwarder = WebSocketMessageForwarder::new();

        let mut rx = forwarder
            .register_connection("conn_1".to_string(), "session_1".to_string(), None, None)
            .await;

        let original = UnifiedMessage::login(
            "test_user".to_string(),
            "test_pass".to_string(),
            "web".to_string(),
        );

        forwarder.broadcast(original.clone()).await.unwrap();

        let received = rx.recv().await.unwrap();
        assert_eq!(original.header.message_id, received.header.message_id);
        assert_eq!(original.header.message_type, received.header.message_type);
    }

    #[tokio::test]
    async fn test_connection_cleanup() {
        let forwarder = WebSocketMessageForwarder::new();

        let _rx = forwarder
            .register_connection(
                "conn_1".to_string(),
                "session_1".to_string(),
                Some("user_1".to_string()),
                Some("device_1".to_string()),
            )
            .await;

        assert_eq!(forwarder.connection_count().await, 1);

        forwarder.unregister_connection("conn_1").await;

        assert_eq!(forwarder.connection_count().await, 0);
        assert_eq!(forwarder.session_connection_count("session_1").await, 0);
    }

    #[tokio::test]
    async fn test_empty_forwarder_broadcast() {
        let forwarder = WebSocketMessageForwarder::new();
        let message = UnifiedMessage::heartbeat("session_1".to_string());

        // 不应该 panic,即使没有连接
        let result = forwarder.broadcast(message).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_connection_with_partial_info() {
        let forwarder = WebSocketMessageForwarder::new();
        let mut rx = forwarder
            .register_connection(
                "conn_1".to_string(),
                "session_1".to_string(),
                None, // 无用户ID
                None, // 无设备ID
            )
            .await;

        let message = UnifiedMessage::heartbeat("session_1".to_string());
        forwarder
            .send_to_session("session_1", message)
            .await
            .unwrap();

        let received = rx.recv().await.unwrap();
        assert_eq!(received.header.session_id, Some("session_1".to_string()));
    }

    #[tokio::test]
    async fn test_large_number_of_connections() {
        let forwarder = WebSocketMessageForwarder::new();

        // 注册100个连接
        for i in 0..100 {
            let _rx = forwarder
                .register_connection(
                    format!("conn_{}", i),
                    format!("session_{}", i % 10), // 10个不同的会话
                    None,
                    None,
                )
                .await;
        }

        assert_eq!(forwarder.connection_count().await, 100);
        assert_eq!(forwarder.session_connection_count("session_5").await, 10);
    }

    #[tokio::test]
    async fn test_message_order_in_broadcast() {
        let forwarder = WebSocketMessageForwarder::new();

        let mut rx1 = forwarder
            .register_connection("conn_1".to_string(), "session_1".to_string(), None, None)
            .await;

        let mut rx2 = forwarder
            .register_connection("conn_2".to_string(), "session_2".to_string(), None, None)
            .await;

        // 发送多个消息
        for i in 0..5 {
            let message = UnifiedMessage::heartbeat(format!("session_{}", i % 2 + 1));
            forwarder.broadcast(message).await.unwrap();
        }

        // 接收消息
        for _ in 0..5 {
            let _ = rx1.recv().await.unwrap();
            let _ = rx2.recv().await.unwrap();
        }
    }
}
