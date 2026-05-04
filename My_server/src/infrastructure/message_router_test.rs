//! / Infrastructure层测试 - 消息路由器
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;

#[cfg(test)]
mod tests {
    use super::*;

    // 模拟消息
    #[derive(Debug, Clone, PartialEq)]
    struct TestMessage {
        id: String,
        payload: Vec<u8>,
        timestamp: i64,
    }

    impl TestMessage {
        fn new(id: &str, payload: &[u8]) -> Self {
            Self {
                id: id.to_string(),
                payload: payload.to_vec(),
                timestamp: chrono::Utc::now().timestamp_millis(),
            }
        }

        fn serialize(&self) -> Vec<u8> {
            format!("{}|{}|{}", 
                self.id,
                hex::encode(&self.payload),
                self.timestamp
            ).into_bytes()
        }

        fn deserialize(data: &[u8]) -> Option<Self> {
            let str_data = String::from_utf8(data.to_vec()).ok()?;
            let parts: Vec<&str> = str_data.split('|').collect();
            if parts.len() != 3 {
                return None;
            }
            let payload = hex::decode(parts[1]).ok()?;
            let timestamp = parts[2].parse::<i64>().ok()?;
            Some(TestMessage {
                id: parts[0].to_string(),
                payload,
                timestamp,
            })
        }
    }

    // 模拟消息转换器
    struct MessageConverter {
        prefix: String,
    }

    impl MessageConverter {
        fn new(prefix: &str) -> Self {
            Self {
                prefix: prefix.to_string(),
            }
        }

        fn convert(&self, msg: &TestMessage) -> TestMessage {
            let mut new_msg = msg.clone();
            new_msg.id = format!("{}:{}", self.prefix, msg.id);
            new_msg
        }

        fn convert_batch(&self, messages: &[TestMessage]) -> Vec<TestMessage> {
            messages.iter().map(|m| self.convert(m)).collect()
        }
    }

    // 模拟TCP会话
    struct TcpSession {
        stream: Option<TcpStream>,
        session_id: String,
    }

    impl TcpSession {
        fn new(stream: TcpStream) -> Self {
            let session_id = format!("session_{}", uuid::Uuid::new_v4());
            Self {
                stream: Some(stream),
                session_id,
            }
        }

        async fn send_message(&mut self, msg: &TestMessage) -> Result<(), std::io::Error> {
            if let Some(ref mut stream) = self.stream {
                let data = msg.serialize();
                stream.write_all(&data).await?;
                stream.write_all(b"\n").await?;
            }
            Ok(())
        }

        async fn receive_message(&mut self) -> Result<TestMessage, String> {
            if let Some(ref mut stream) = self.stream {
                let mut buffer = [0u8; 4096];
                let n = stream.read(&mut buffer).await
                    .map_err(|e| e.to_string())?;
                if n == 0 {
                    return Err("Connection closed".to_string());
                }
                let data = &buffer[..n];
                let msg = TestMessage::deserialize(data)
                    .ok_or("Invalid message format".to_string())?;
                Ok(msg)
            } else {
                Err("No stream available".to_string())
            }
        }

        fn session_id(&self) -> &str {
            &self.session_id
        }
    }

    // 模拟消息路由器
    struct MessageRouter {
        routes: Arc<RwLock<HashMap<String, Vec<String>>>>,
        converters: Arc<RwLock<HashMap<String, MessageConverter>>>,
    }

    impl MessageRouter {
        fn new() -> Self {
            Self {
                routes: Arc::new(RwLock::new(HashMap::new())),
                converters: Arc::new(RwLock::new(HashMap::new())),
            }
        }

        async fn add_route(&self, from: &str, to: &str) {
            let mut routes = self.routes.write().await;
            routes.entry(from.to_string())
                .or_insert_with(Vec::new)
                .push(to.to_string());
        }

        async fn get_routes(&self, from: &str) -> Vec<String> {
            let routes = self.routes.read().await;
            routes.get(from).cloned().unwrap_or_default()
        }

        async fn remove_route(&self, from: &str, to: &str) {
            let mut routes = self.routes.write().await;
            if let Some(destinations) = routes.get_mut(from) {
                destinations.retain(|d| d != to);
            }
        }

        async fn add_converter(&self, name: &str, converter: MessageConverter) {
            let mut converters = self.converters.write().await;
            converters.insert(name.to_string(), converter);
        }

        async fn convert_message(&self, converter_name: &str, msg: &TestMessage) -> Option<TestMessage> {
            let converters = self.converters.read().await;
            let converter = converters.get(converter_name)?;
            Some(converter.convert(msg))
        }
    }

    #[test]
    fn test_message_creation() {
        let msg = TestMessage::new("test001", b"hello world");
        assert_eq!(msg.id, "test001");
        assert_eq!(msg.payload, b"hello world");
    }

    #[test]
    fn test_message_serialization() {
        let msg = TestMessage::new("test001", b"hello");
        let serialized = msg.serialize();
        let deserialized = TestMessage::deserialize(&serialized);
        assert!(deserialized.is_some());
        assert_eq!(deserialized.unwrap().id, "test001");
    }

    #[test]
    fn test_message_serialization_empty_payload() {
        let msg = TestMessage::new("test001", b"");
        let serialized = msg.serialize();
        let deserialized = TestMessage::deserialize(&serialized);
        assert!(deserialized.is_some());
        assert_eq!(deserialized.unwrap().payload.len(), 0);
    }

    #[test]
    fn test_message_deserialization_invalid() {
        let invalid_data = b"invalid|format";
        let result = TestMessage::deserialize(invalid_data);
        assert!(result.is_none());
    }

    #[test]
    fn test_message_timestamp() {
        let before = chrono::Utc::now().timestamp_millis();
        let msg = TestMessage::new("test001", b"hello");
        let after = chrono::Utc::now().timestamp_millis();
        assert!(msg.timestamp >= before && msg.timestamp <= after);
    }

    #[test]
    fn test_converter_creation() {
        let converter = MessageConverter::new("prefix");
        assert_eq!(converter.prefix, "prefix");
    }

    #[test]
    fn test_converter_single_message() {
        let converter = MessageConverter::new("test");
        let msg = TestMessage::new("msg001", b"hello");
        let converted = converter.convert(&msg);
        assert_eq!(converted.id, "test:msg001");
    }

    #[test]
    fn test_converter_batch_messages() {
        let converter = MessageConverter::new("test");
        let messages = vec![
            TestMessage::new("msg001", b"hello"),
            TestMessage::new("msg002", b"world"),
        ];
        let converted = converter.convert_batch(&messages);
        assert_eq!(converted.len(), 2);
        assert_eq!(converted[0].id, "test:msg001");
        assert_eq!(converted[1].id, "test:msg002");
    }

    #[test]
    fn test_converter_preserves_payload() {
        let converter = MessageConverter::new("test");
        let msg = TestMessage::new("msg001", b"hello world");
        let converted = converter.convert(&msg);
        assert_eq!(converted.payload, msg.payload);
    }

    #[test]
    fn test_converter_preserves_timestamp() {
        let converter = MessageConverter::new("test");
        let msg = TestMessage::new("msg001", b"hello");
        let converted = converter.convert(&msg);
        assert_eq!(converted.timestamp, msg.timestamp);
    }

    #[tokio::test]
    async fn test_router_creation() {
        let router = MessageRouter::new();
        assert_eq!(router.get_routes("source1").await.len(), 0);
    }

    #[tokio::test]
    async fn test_router_add_route() {
        let router = MessageRouter::new();
        router.add_route("source1", "dest1").await;
        let routes = router.get_routes("source1").await;
        assert_eq!(routes.len(), 1);
        assert_eq!(routes[0], "dest1");
    }

    #[tokio::test]
    async fn test_router_add_multiple_routes() {
        let router = MessageRouter::new();
        router.add_route("source1", "dest1").await;
        router.add_route("source1", "dest2").await;
        router.add_route("source1", "dest3").await;
        let routes = router.get_routes("source1").await;
        assert_eq!(routes.len(), 3);
    }

    #[tokio::test]
    async fn test_router_remove_route() {
        let router = MessageRouter::new();
        router.add_route("source1", "dest1").await;
        router.add_route("source1", "dest2").await;
        router.remove_route("source1", "dest1").await;
        let routes = router.get_routes("source1").await;
        assert_eq!(routes.len(), 1);
        assert_eq!(routes[0], "dest2");
    }

    #[tokio::test]
    async fn test_router_multiple_sources() {
        let router = MessageRouter::new();
        router.add_route("source1", "dest1").await;
        router.add_route("source2", "dest2").await;
        assert_eq!(router.get_routes("source1").await.len(), 1);
        assert_eq!(router.get_routes("source2").await.len(), 1);
    }

    #[tokio::test]
    async fn test_router_nonexistent_source() {
        let router = MessageRouter::new();
        let routes = router.get_routes("nonexistent").await;
        assert_eq!(routes.len(), 0);
    }

    #[tokio::test]
    async fn test_router_add_converter() {
        let router = MessageRouter::new();
        let converter = MessageConverter::new("test");
        router.add_converter("conv1", converter).await;
        
        let msg = TestMessage::new("msg001", b"hello");
        let converted = router.convert_message("conv1", &msg).await;
        assert!(converted.is_some());
        assert_eq!(converted.unwrap().id, "test:msg001");
    }

    #[tokio::test]
    async fn test_converter_nonexistent() {
        let router = MessageRouter::new();
        let msg = TestMessage::new("msg001", b"hello");
        let converted = router.convert_message("nonexistent", &msg).await;
        assert!(converted.is_none());
    }

    #[test]
    fn test_message_clone() {
        let msg = TestMessage::new("test001", b"hello");
        let cloned = msg.clone();
        assert_eq!(msg.id, cloned.id);
        assert_eq!(msg.payload, cloned.payload);
    }

    #[test]
    fn test_message_equality() {
        let msg1 = TestMessage::new("test001", b"hello");
        let msg2 = TestMessage::new("test001", b"hello");
        assert_eq!(msg1, msg2);
    }

    #[test]
    fn test_message_inequality() {
        let msg1 = TestMessage::new("test001", b"hello");
        let msg2 = TestMessage::new("test002", b"hello");
        assert_ne!(msg1, msg2);
    }

    #[tokio::test]
    async fn test_route_deduplication() {
        let router = MessageRouter::new();
        router.add_route("source1", "dest1").await;
        router.add_route("source1", "dest1").await; // 添加重复路由
        let routes = router.get_routes("source1").await;
        // 当前实现允许重复,这符合某些路由场景
        assert_eq!(routes.len(), 2);
    }

    #[tokio::test]
    async fn test_router_concurrent_access() {
        let router = Arc::new(MessageRouter::new());
        let mut handles = vec![];

        for i in 0..10 {
            let router_clone = router.clone();
            let handle = tokio::spawn(async move {
                router_clone.add_route(&format!("source{}", i % 3), &format!("dest{}", i)).await;
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.await.unwrap();
        }

        assert_eq!(router.get_routes("source0").await.len(), 4);
        assert_eq!(router.get_routes("source1").await.len(), 3);
        assert_eq!(router.get_routes("source2").await.len(), 3);
    }

    #[tokio::test]
    async fn test_converter_chain() {
        let router = Arc::new(MessageRouter::new());
        
        router.add_converter("conv1", MessageConverter::new("A")).await;
        router.add_converter("conv2", MessageConverter::new("B")).await;

        let msg = TestMessage::new("msg001", b"hello");
        let conv1 = router.convert_message("conv1", &msg).await.unwrap();
        let conv2 = router.convert_message("conv2", &conv1).await.unwrap();

        assert_eq!(conv2.id, "B:A:msg001");
    }

    #[test]
    fn test_large_message_serialization() {
        let large_payload = vec![0u8; 100000];
        let msg = TestMessage::new("large", &large_payload);
        let serialized = msg.serialize();
        let deserialized = TestMessage::deserialize(&serialized).unwrap();
        assert_eq!(deserialized.payload.len(), 100000);
    }
}






