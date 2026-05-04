//! / WebSocket 消息转发器
// 用于在 Truck Scale 协议适配器和 CarpTMS 核心服务之间转发 WebSocket 消息
use crate::truck_scale::integration::WebSocketSender;
use crate::truck_scale::protocol::message_protocol::*;
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

/// WebSocket 连接信息
#[derive(Clone)]
struct WebSocketConnection {
    /// 会话ID
    session_id: String,
    /// 用户ID
    user_id: Option<String>,
    /// 设备ID
    device_id: Option<String>,
    /// 发送通道
    sender: mpsc::UnboundedSender<UnifiedMessage>,
}

/// WebSocket 消息转发器
pub struct WebSocketMessageForwarder {
    /// 所有连接
    connections: Arc<RwLock<HashMap<String, WebSocketConnection>>>,
    /// 消息广播通道
    broadcast_tx: mpsc::UnboundedSender<UnifiedMessage>,
}

impl WebSocketMessageForwarder {
    /// 创建新的 WebSocket 消息转发器
    pub fn new() -> Self {
        let (broadcast_tx, mut broadcast_rx) = mpsc::unbounded_channel::<UnifiedMessage>();
        let connections = Arc::new(RwLock::new(HashMap::<String, WebSocketConnection>::new()));

        // 启动广播任务
        let connections_clone = connections.clone();
        tokio::spawn(async move {
            while let Some(message) = broadcast_rx.recv().await {
                // 广播消息到所有连接
                let connections = connections_clone.read().await;
                for conn in connections.values() {
                    let _ = conn.sender.send(message.clone());
                }
            }
        });

        Self {
            connections,
            broadcast_tx,
        }
    }

    /// 注册 WebSocket 连接
    pub async fn register_connection(
        &self,
        connection_id: String,
        session_id: String,
        user_id: Option<String>,
        device_id: Option<String>,
    ) -> mpsc::UnboundedReceiver<UnifiedMessage> {
        let (sender, receiver) = mpsc::unbounded_channel::<UnifiedMessage>();

        let connection = WebSocketConnection {
            session_id,
            user_id,
            device_id,
            sender,
        };

        self.connections
            .write()
            .await
            .insert(connection_id.clone(), connection);

        receiver
    }

    /// 注销 WebSocket 连接
    pub async fn unregister_connection(&self, connection_id: &str) {
        self.connections.write().await.remove(connection_id);
    }

    /// 广播消息到所有连接
    pub async fn broadcast(&self, message: UnifiedMessage) -> Result<()> {
        self.broadcast_tx.send(message)?;
        Ok(())
    }

    /// 发送消息到指定会话
    pub async fn send_to_session(&self, session_id: &str, message: UnifiedMessage) -> Result<()> {
        let connections = self.connections.read().await;

        for conn in connections.values() {
            if conn.session_id == session_id {
                conn.sender.send(message)?;
                return Ok(());
            }
        }

        Err(anyhow::anyhow!("Session not found: {}", session_id))
    }

    /// 发送消息到指定用户
    pub async fn send_to_user(&self, user_id: &str, message: UnifiedMessage) -> Result<()> {
        let connections = self.connections.read().await;

        for conn in connections.values() {
            if let Some(uid) = &conn.user_id {
                if uid == user_id {
                    let _ = conn.sender.send(message.clone());
                }
            }
        }

        Ok(())
    }

    /// 发送消息到指定设备
    pub async fn send_to_device(&self, device_id: &str, message: UnifiedMessage) -> Result<()> {
        let connections = self.connections.read().await;

        for conn in connections.values() {
            if let Some(did) = &conn.device_id {
                if did == device_id {
                    conn.sender.send(message)?;
                    return Ok(());
                }
            }
        }

        Err(anyhow::anyhow!("Device not found: {}", device_id))
    }

    /// 获取连接数量
    pub async fn connection_count(&self) -> usize {
        self.connections.read().await.len()
    }

    /// 获取指定会话的连接数量
    pub async fn session_connection_count(&self, session_id: &str) -> usize {
        let connections = self.connections.read().await;
        connections
            .values()
            .filter(|c| c.session_id == session_id)
            .count()
    }
}

/// 实现 WebSocketSender trait
#[async_trait::async_trait]
impl WebSocketSender for WebSocketMessageForwarder {
    async fn send_to_all(&self, message: UnifiedMessage) -> Result<()> {
        self.broadcast(message).await
    }

    async fn send_to_session(&self, session_id: &str, message: UnifiedMessage) -> Result<()> {
        self.send_to_session(session_id, message).await
    }
}

impl Default for WebSocketMessageForwarder {
    fn default() -> Self {
        Self::new()
    }
}

/// WebSocket 消息处理器
pub struct WebSocketMessageHandler {
    forwarder: Arc<WebSocketMessageForwarder>,
    integration: Arc<dyn MessageHandler + Send + Sync>,
}

/// 消息处理器 trait
#[async_trait::async_trait]
pub trait MessageHandler {
    /// 处理传入的 WebSocket 消息
    async fn handle_message(&self, message: UnifiedMessage) -> Result<UnifiedMessage>;
}

impl WebSocketMessageHandler {
    /// 创建新的 WebSocket 消息处理器
    pub fn new(
        forwarder: Arc<WebSocketMessageForwarder>,
        integration: Arc<dyn MessageHandler + Send + Sync>,
    ) -> Self {
        Self {
            forwarder,
            integration,
        }
    }

    /// 处理 WebSocket 消息
    pub async fn handle_websocket_message(
        &self,
        message: UnifiedMessage,
    ) -> Result<UnifiedMessage> {
        // 处理消息
        let response = self.integration.handle_message(message.clone()).await?;

        // 根据消息类型决定是否广播
        match &message.body {
            MessageBody::DataReport(data_report) => {
                // 数据上报消息广播到所有连接
                let notification = NotificationMessage {
                    notification_type: format!("data_report_{}", data_report.report_type),
                    title: "数据上报通知".to_string(),
                    content: format!(
                        "设备 {} 上报了 {} 数据",
                        data_report.device_id, data_report.report_type
                    ),
                    data: Some(data_report.data.clone()),
                };

                let broadcast_msg = UnifiedMessage::new(
                    MessageType::Notification,
                    MessageBody::Notification(notification),
                )
                .with_session_id(message.header.session_id.clone().unwrap_or_default())
                .with_device_id(data_report.device_id.clone());

                let _ = self.forwarder.broadcast(broadcast_msg).await;
            }
            MessageBody::Heartbeat(_) => {
                // 心跳消息不广播
            }
            _ => {
                // 其他消息返回给发送者
            }
        }

        Ok(response)
    }

    /// 获取转发器
    pub fn forwarder(&self) -> Arc<WebSocketMessageForwarder> {
        self.forwarder.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_websocket_forwarder() {
        let forwarder = WebSocketMessageForwarder::new();

        // 注册连接
        let mut rx = forwarder
            .register_connection(
                "conn_1".to_string(),
                "session_1".to_string(),
                Some("user_1".to_string()),
                Some("device_1".to_string()),
            )
            .await;

        // 发送消息到会话
        let message = UnifiedMessage::heartbeat("session_1".to_string());
        forwarder
            .send_to_session("session_1", message)
            .await
            .unwrap();

        // 接收消息
        let received = rx.recv().await.unwrap();
        assert_eq!(received.header.session_id, Some("session_1".to_string()));
    }

    #[tokio::test]
    async fn test_broadcast() {
        let forwarder = WebSocketMessageForwarder::new();

        // 注册多个连接
        let mut rx1 = forwarder
            .register_connection("conn_1".to_string(), "session_1".to_string(), None, None)
            .await;
        let mut rx2 = forwarder
            .register_connection("conn_2".to_string(), "session_2".to_string(), None, None)
            .await;

        // 广播消息
        let message = UnifiedMessage::heartbeat("session_1".to_string());
        forwarder.broadcast(message).await.unwrap();

        // 两个连接都应该接收到消息
        let _ = rx1.recv().await.unwrap();
        let _ = rx2.recv().await.unwrap();
    }
}
