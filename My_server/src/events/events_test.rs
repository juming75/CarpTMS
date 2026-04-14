//! / 事件驱动架构测试

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc;

    #[tokio::test]
    async fn test_event_creation() {
        let event = DomainEvent::VehicleOnline {
            vehicle_id: "V123".to_string(),
            timestamp: chrono::Utc::now(),
        };
        assert_eq!(event.event_type(), "VehicleOnline");
    }

    #[tokio::test]
    async fn test_event_dispatch() {
        let dispatcher = EventDispatcher::new();
        let (tx, mut rx) = mpsc::channel(100);
        
        let handler = TestEventHandler { sender: tx };
        dispatcher.register_handler(Box::new(handler));
        
        let event = DomainEvent::VehicleOnline {
            vehicle_id: "V123".to_string(),
            timestamp: chrono::Utc::now(),
        };
        
        dispatcher.dispatch(event).await.unwrap();
        
        let received = rx.recv().await;
        assert!(received.is_some());
    }

    #[tokio::test]
    async fn test_multiple_event_handlers() {
        let dispatcher = EventDispatcher::new();
        let (tx1, mut rx1) = mpsc::channel(100);
        let (tx2, mut rx2) = mpsc::channel(100);
        
        dispatcher.register_handler(Box::new(TestEventHandler { sender: tx1 }));
        dispatcher.register_handler(Box::new(TestEventHandler { sender: tx2 }));
        
        let event = DomainEvent::VehicleOnline {
            vehicle_id: "V123".to_string(),
            timestamp: chrono::Utc::now(),
        };
        
        dispatcher.dispatch(event).await.unwrap();
        
        let received1 = rx1.recv().await;
        let received2 = rx2.recv().await;
        assert!(received1.is_some());
        assert!(received2.is_some());
    }

    #[tokio::test]
    async fn test_event_serialization() {
        let event = DomainEvent::VehicleOnline {
            vehicle_id: "V123".to_string(),
            timestamp: chrono::Utc::now(),
        };
        
        let json = serde_json::to_string(&event).unwrap();
        let deserialized: DomainEvent = serde_json::from_str(&json).unwrap();
        
        match deserialized {
            DomainEvent::VehicleOnline { vehicle_id, .. } => {
                assert_eq!(vehicle_id, "V123");
            }
            _ => panic!("Wrong event type"),
        }
    }

    // 测试辅助结构
    struct TestEventHandler {
        sender: mpsc::Sender<DomainEvent>,
    }

    #[async_trait::async_trait]
    impl EventHandler for TestEventHandler {
        async fn handle(&self, event: &DomainEvent) -> Result<(), AppError> {
            self.sender.send(event.clone()).await.unwrap();
            Ok(())
        }

        fn event_types(&self) -> Vec<&'static str> {
            vec!["VehicleOnline"]
        }
    }
}






