//! / 消息路由器
// 实现 TCP 到 WebSocket 的消息路由,以及 WebSocket 指令到 TCP 设备的下发

use actix::prelude::*;
use log::{debug, error, info, warn};

use super::converter::{JT808ToUnifiedConverter, UnifiedToJT808Converter};
use super::tcp_session::TcpSessionManager;
use super::types::{
    MessageRouterError, MessageType, TcpDeviceMessage, UnifiedMessage, WebSocketCommandMessage,
};
use crate::gateway::websocket_server::UnifiedMessage as WsMessage;
use crate::gateway::websocket_server::{TopicManager, WsSessionRegistry};
use crate::protocols::jt808::JT808Parser;

// 将 UnifiedMessage (Router) 转换为 WsMessage
fn convert_router_to_ws_message(msg: &UnifiedMessage) -> WsMessage {
    WsMessage {
        msg_type: format!("{:?}", msg.msg_type).to_lowercase(),
        msg_id: Some(msg.msg_id.clone()),
        device_id: msg.device_id.clone(),
        command: msg.command.clone(),
        payload: msg.payload.clone(),
        timestamp: msg.timestamp.timestamp(),
    }
}

/// 消息路由器 Actor
pub struct MessageRouter {
    /// TCP 会话管理器
    tcp_session_manager: Addr<TcpSessionManager>,
    /// WebSocket 会话注册表
    ws_session_registry: Option<std::sync::Arc<WsSessionRegistry>>,
    /// Topic 管理器
    topic_manager: Option<std::sync::Arc<TopicManager>>,
}

impl Clone for MessageRouter {
    fn clone(&self) -> Self {
        Self {
            tcp_session_manager: self.tcp_session_manager.clone(),
            ws_session_registry: self.ws_session_registry.clone(),
            topic_manager: self.topic_manager.clone(),
        }
    }
}

impl MessageRouter {
    pub fn new(tcp_session_manager: Addr<TcpSessionManager>) -> Self {
        Self {
            tcp_session_manager,
            ws_session_registry: None,
            topic_manager: None,
        }
    }

    /// 设置 WebSocket 会话注册表
    pub fn with_ws_registry(mut self, registry: std::sync::Arc<WsSessionRegistry>) -> Self {
        self.ws_session_registry = Some(registry);
        self
    }

    /// 设置 Topic 管理器
    pub fn with_topic_manager(
        mut self,
        manager: std::sync::Arc<crate::gateway::websocket_server::TopicManager>,
    ) -> Self {
        self.topic_manager = Some(manager);
        self
    }

    /// 处理 TCP 设备数据
    async fn handle_tcp_device_data(
        &self,
        msg: TcpDeviceMessage,
    ) -> Result<(), MessageRouterError> {
        debug!(
            "Processing TCP device data: device_id={}, protocol={}, data_len={}",
            msg.device_id,
            msg.protocol,
            msg.raw_data.len()
        );

        // 根据协议类型解析数据
        let unified_msg = match msg.protocol.as_str() {
            "JT808" => self.parse_jt808_data(&msg).await?,
            "GB" | "BSJ" | "DB44" => self.parse_legacy_data(&msg).await?,
            _ => {
                return Err(MessageRouterError::ConversionFailed(format!(
                    "Unknown protocol: {}",
                    msg.protocol
                )));
            }
        };

        // 推送到 WebSocket
        if let Some(ref registry) = self.ws_session_registry {
            if let Some(ref topic_manager) = self.topic_manager {
                if let Some(topic) = unified_msg.get_topic() {
                    let registry_clone = registry.clone();
                    let topic_manager_clone = topic_manager.clone();
                    let topic_clone = topic.clone();
                    let ws_msg = convert_router_to_ws_message(&unified_msg);

                    tokio::spawn(async move {
                        registry_clone
                            .broadcast_to_topic(topic_manager_clone, &topic_clone, &ws_msg)
                            .await;
                    });

                    debug!("Published message to topic: {}", topic);
                }
            }
        }

        // 保存到数据库 (后续实现)
        // self.save_to_database(&unified_msg).await?;

        Ok(())
    }

    /// 解析 JT808 数据
    async fn parse_jt808_data(
        &self,
        msg: &TcpDeviceMessage,
    ) -> Result<UnifiedMessage, MessageRouterError> {
        // 使用 JT808 解析器解析数据
        match JT808Parser::parse_frame(&msg.raw_data) {
            Ok(frame) => {
                debug!(
                    "JT808 frame parsed: msg_id=0x{:04X}, phone={}",
                    frame.msg_id, frame.phone
                );

                // 根据消息类型转换为统一消息
                match frame.msg_id {
                    0x0200 => {
                        // 位置信息汇报
                        match JT808Parser::parse_body(frame.msg_id, &frame.body) {
                            Ok(location) => JT808ToUnifiedConverter::convert_location_report(
                                frame.phone,
                                &location,
                            )
                            .map_err(MessageRouterError::ConversionFailed),
                            Err(e) => Err(MessageRouterError::ConversionFailed(format!(
                                "Failed to parse 0x0200 body: {:?}",
                                e
                            ))),
                        }
                    }
                    0x1201 => {
                        // 报警上传 (简化实现,暂时返回通用报警)
                        JT808ToUnifiedConverter::convert_alarm(
                            frame.phone,
                            "alarm".to_string(),
                            2,
                            "设备报警".to_string(),
                        )
                        .map_err(MessageRouterError::ConversionFailed)
                    }
                    0x0704 => {
                        // 数据上传应答 (简化实现)
                        JT808ToUnifiedConverter::convert_sensor_data(
                            frame.phone,
                            "sensor_data".to_string(),
                            serde_json::json!({
                                "data_length": frame.body.len(),
                                "data_hex": hex::encode(&frame.body)
                            }),
                        )
                        .map_err(MessageRouterError::ConversionFailed)
                    }
                    _ => {
                        debug!("Unhandled JT808 message ID: 0x{:04X}", frame.msg_id);
                        Err(MessageRouterError::ConversionFailed(format!(
                            "Unhandled message ID: 0x{:04X}",
                            frame.msg_id
                        )))
                    }
                }
            }
            Err(e) => Err(MessageRouterError::ConversionFailed(format!(
                "Failed to parse JT808 data: {}",
                e
            ))),
        }
    }

    /// 解析旧协议数据 (GB/BSJ/DB44)
    async fn parse_legacy_data(
        &self,
        msg: &TcpDeviceMessage,
    ) -> Result<UnifiedMessage, MessageRouterError> {
        // 简化实现:直接将原始数据转换为统一消息
        warn!(
            "Parsing legacy protocol data ({}), simplified implementation",
            msg.protocol
        );

        let payload = serde_json::json!({
            "protocol": msg.protocol,
            "data_hex": hex::encode(&msg.raw_data),
            "data_length": msg.raw_data.len(),
        });

        Ok(UnifiedMessage::new(
            MessageType::Data,
            crate::infrastructure::message_router::types::MessageSource::Tcp,
            Some(msg.device_id.clone()),
            Some("legacy_data".to_string()),
            payload,
        ))
    }

    /// 处理 WebSocket 指令
    async fn handle_websocket_command(
        &self,
        msg: WebSocketCommandMessage,
    ) -> Result<UnifiedMessage, MessageRouterError> {
        debug!(
            "Processing WebSocket command: client_id={}, target_device={}, command={}",
            msg.client_id, msg.target_device_id, msg.command
        );

        // 检查目标设备是否在线
        // if !self.tcp_session_manager.is_online(&msg.target_device_id).await {
        //     return Err(MessageRouterError::TargetNotFound(format!(
        //         "Device not online: {}",
        //         msg.target_device_id
        //     ));
        // }

        // 解析 WebSocket 指令并转换为 JT808 协议帧
        let (msg_id, data) =
            UnifiedToJT808Converter::parse_websocket_command(&msg.command, &msg.params)
                .map_err(MessageRouterError::ConversionFailed)?;

        debug!(
            "Converted WebSocket command to JT808: msg_id=0x{:04X}, data_len={}",
            msg_id,
            data.len()
        );

        // 下发指令到 TCP 设备 (通过会话管理器)
        // 创建发送命令消息
        let send_cmd = super::tcp_session::SendTcpCommand {
            device_id: msg.target_device_id.clone(),
            command_data: data.clone(),
        };

        // 通过 Actor 发送消息并等待响应
        match self.tcp_session_manager.send(send_cmd).await {
            Ok(Ok((flow_no, Some(addr)))) => {
                debug!(
                    "Command sent to device {} at {} with flow_no {}",
                    msg.target_device_id, addr, flow_no
                );
                Ok(UnifiedMessage::command_response(
                    Some(msg.target_device_id.clone()),
                    msg.command.clone(),
                    true,
                    format!("Command sent successfully, flow_no: {}", flow_no),
                ))
            }
            Ok(Ok((flow_no, None))) => {
                debug!(
                    "Command queued for device {} with flow_no {} (no active connection)",
                    msg.target_device_id, flow_no
                );
                Ok(UnifiedMessage::command_response(
                    Some(msg.target_device_id.clone()),
                    msg.command.clone(),
                    true,
                    format!("Command queued, flow_no: {}", flow_no),
                ))
            }
            Ok(Err(e)) => {
                error!(
                    "Failed to send command to device {}: {}",
                    msg.target_device_id, e
                );
                Ok(UnifiedMessage::command_response(
                    Some(msg.target_device_id.clone()),
                    msg.command.clone(),
                    false,
                    format!("Failed to send command: {}", e),
                ))
            }
            Err(e) => {
                error!(
                    "Actor error when sending command to device {}: {}",
                    msg.target_device_id, e
                );
                Ok(UnifiedMessage::command_response(
                    Some(msg.target_device_id.clone()),
                    msg.command.clone(),
                    false,
                    format!("Actor error: {}", e),
                ))
            }
        }
    }
}

impl Actor for MessageRouter {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        info!("Message router started");
    }
}

impl Handler<TcpDeviceMessage> for MessageRouter {
    type Result = ResponseActFuture<Self, ()>;

    fn handle(&mut self, msg: TcpDeviceMessage, _ctx: &mut Self::Context) -> Self::Result {
        let router = self.clone();

        let future = async move {
            match router.handle_tcp_device_data(msg).await {
                Ok(_) => {}
                Err(e) => {
                    log::error!("Failed to handle TCP device data: {:?}", e);
                }
            }
        };

        Box::pin(actix::fut::wrap_future(future))
    }
}

impl Handler<WebSocketCommandMessage> for MessageRouter {
    type Result = ResponseActFuture<Self, ()>;

    fn handle(&mut self, msg: WebSocketCommandMessage, _ctx: &mut Self::Context) -> Self::Result {
        let router = self.clone();

        let future = async move {
            match router.handle_websocket_command(msg).await {
                Ok(_) => {}
                Err(e) => {
                    log::error!("Failed to handle WebSocket command: {:?}", e);
                }
            }
        };

        Box::pin(actix::fut::wrap_future(future))
    }
}

// Note: MessageRouterTrait implementation removed due to Actix Actor lifetime compatibility issues.
// Direct method calls are used instead.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_router_creation() {
        let manager_addr = TcpSessionManager::default().start();
        let router = MessageRouter::new(manager_addr);

        assert!(router.ws_session_registry.is_none());
        assert!(router.topic_manager.is_none());
    }
}
