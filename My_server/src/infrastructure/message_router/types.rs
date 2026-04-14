//! / 统一消息类型定义
// 定义跨 TCP 和 WebSocket 的统一消息格式

use actix::Message;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 消息来源
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MessageSource {
    /// 来自 TCP 设备
    Tcp,
    /// 来自 WebSocket 客户端
    WebSocket,
    /// 来自内部系统
    Internal,
}

/// 消息类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MessageType {
    /// 数据消息 (位置、报警、传感器数据)
    Data,
    /// 通知消息 (系统通知、报警通知)
    Notification,
    /// 命令消息 (下发指令)
    Command,
    /// 响应消息 (命令响应)
    Response,
    /// 确认消息 (ACK)
    Ack,
}

/// 消息优先级
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum MessagePriority {
    /// 低优先级
    Low = 0,
    /// 普通优先级
    #[default]
    Normal = 1,
    /// 高优先级
    High = 2,
    /// 紧急优先级
    Critical = 3,
}

/// 统一消息结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedMessage {
    /// 消息ID (唯一标识)
    pub msg_id: String,

    /// 消息类型
    pub msg_type: MessageType,

    /// 消息来源
    pub source: MessageSource,

    /// 设备ID
    pub device_id: Option<String>,

    /// 命令类型 (如: location_report, alarm, take_photo)
    pub command: Option<String>,

    /// 消息优先级
    pub priority: MessagePriority,

    /// 消息载荷 (JSON 格式)
    pub payload: serde_json::Value,

    /// 扩展字段 (用于存储自定义数据)
    pub extra: HashMap<String, serde_json::Value>,

    /// 时间戳
    pub timestamp: DateTime<Utc>,

    /// 可靠性标志 (是否需要确认)
    pub reliable: bool,
}

impl UnifiedMessage {
    /// 创建新的统一消息
    pub fn new(
        msg_type: MessageType,
        source: MessageSource,
        device_id: Option<String>,
        command: Option<String>,
        payload: serde_json::Value,
    ) -> Self {
        Self {
            msg_id: uuid::Uuid::new_v4().to_string(),
            msg_type,
            source,
            device_id,
            command,
            priority: MessagePriority::default(),
            payload,
            extra: HashMap::new(),
            timestamp: Utc::now(),
            reliable: false,
        }
    }

    /// 设置消息优先级
    pub fn with_priority(mut self, priority: MessagePriority) -> Self {
        self.priority = priority;
        self
    }

    /// 设置可靠性标志
    pub fn with_reliable(mut self, reliable: bool) -> Self {
        self.reliable = reliable;
        self
    }

    /// 添加扩展字段
    pub fn with_extra(mut self, key: String, value: serde_json::Value) -> Self {
        self.extra.insert(key, value);
        self
    }

    /// 创建位置数据消息
    pub fn location_data(
        device_id: String,
        latitude: f64,
        longitude: f64,
        speed: f64,
        direction: f64,
        sensor_data: Option<serde_json::Value>,
    ) -> Self {
        let payload = serde_json::json!({
            "latitude": latitude,
            "longitude": longitude,
            "speed": speed,
            "direction": direction,
            "sensor_data": sensor_data.unwrap_or(serde_json::json!({})),
        });

        Self::new(
            MessageType::Data,
            MessageSource::Tcp,
            Some(device_id),
            Some("location_report".to_string()),
            payload,
        )
        .with_priority(MessagePriority::Normal)
    }

    /// 创建报警数据消息
    pub fn alarm_data(
        device_id: String,
        alarm_type: String,
        alarm_level: i32,
        description: String,
    ) -> Self {
        let payload = serde_json::json!({
            "alarm_type": alarm_type,
            "alarm_level": alarm_level,
            "description": description,
        });

        Self::new(
            MessageType::Notification,
            MessageSource::Tcp,
            Some(device_id),
            Some("alarm".to_string()),
            payload,
        )
        .with_priority(MessagePriority::High)
    }

    /// 创建传感器数据消息
    pub fn sensor_data(
        device_id: String,
        sensor_type: String,
        sensor_value: serde_json::Value,
    ) -> Self {
        let payload = serde_json::json!({
            "sensor_type": sensor_type,
            "sensor_value": sensor_value,
        });

        Self::new(
            MessageType::Data,
            MessageSource::Tcp,
            Some(device_id),
            Some("sensor_update".to_string()),
            payload,
        )
        .with_priority(MessagePriority::Low)
    }

    /// 创建设备状态消息
    pub fn device_status(device_id: String, status: String, online: bool) -> Self {
        let payload = serde_json::json!({
            "status": status,
            "online": online,
        });

        Self::new(
            MessageType::Data,
            MessageSource::Tcp,
            Some(device_id),
            Some("device_status".to_string()),
            payload,
        )
        .with_priority(MessagePriority::Normal)
    }

    /// 创建命令响应消息
    pub fn command_response(
        device_id: Option<String>,
        command: String,
        success: bool,
        message: String,
    ) -> Self {
        let payload = serde_json::json!({
            "success": success,
            "message": message,
        });

        Self::new(
            MessageType::Response,
            MessageSource::Internal,
            device_id,
            Some(command),
            payload,
        )
        .with_priority(MessagePriority::Normal)
    }

    /// 创建确认消息
    pub fn ack(original_msg_id: String, success: bool, message: String) -> Self {
        let payload = serde_json::json!({
            "original_msg_id": original_msg_id,
            "success": success,
            "message": message,
        });

        Self::new(
            MessageType::Ack,
            MessageSource::Internal,
            None,
            Some("ack".to_string()),
            payload,
        )
        .with_priority(MessagePriority::Normal)
    }

    /// 获取 Topic 字符串
    pub fn get_topic(&self) -> Option<String> {
        match (&self.msg_type, &self.command, &self.device_id) {
            (MessageType::Data, Some(cmd), Some(device_id)) => {
                Some(format!("vehicle:{}:{}", device_id, cmd))
            }
            (MessageType::Notification, Some(cmd), Some(device_id)) => {
                if cmd == "alarm" {
                    Some(format!("vehicle:{}:alarm", device_id))
                } else {
                    Some(format!("vehicle:{}:{}", device_id, cmd))
                }
            }
            (MessageType::Data, Some(cmd), None) => Some(format!("system:{}", cmd)),
            _ => None,
        }
    }

    /// 序列化为 JSON 字符串
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// 从 JSON 字符串反序列化
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

impl Default for UnifiedMessage {
    fn default() -> Self {
        Self::new(
            MessageType::Data,
            MessageSource::Internal,
            None,
            None,
            serde_json::json!({}),
        )
    }
}

/// TCP 设备消息
#[derive(Message)]
#[rtype(result = "()")]
pub struct TcpDeviceMessage {
    /// 设备ID
    pub device_id: String,
    /// 协议类型 (JT808/GB/BSJ/DB44)
    pub protocol: String,
    /// 原始协议数据
    pub raw_data: Vec<u8>,
    /// 解析后的统一消息
    pub unified_msg: Option<UnifiedMessage>,
    /// 接收时间
    pub received_at: DateTime<Utc>,
}

/// WebSocket 客户端消息
#[derive(Message)]
#[rtype(result = "()")]
pub struct WebSocketCommandMessage {
    /// 客户端ID
    pub client_id: String,
    /// 目标设备ID
    pub target_device_id: String,
    /// 命令类型
    pub command: String,
    /// 命令参数
    pub params: serde_json::Value,
    /// 发送时间
    pub sent_at: DateTime<Utc>,
}

/// 消息路由请求
#[derive(Message)]
#[rtype(result = "Result<(), MessageRouterError>")]
pub struct RouteMessageRequest {
    /// 要路由的消息
    pub message: UnifiedMessage,
    /// 目标 Topic (可选,如果不指定则自动推断)
    pub target_topic: Option<String>,
}

/// 消息路由错误
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageRouterError {
    /// 消息格式错误
    InvalidMessageFormat(String),
    /// 目标不存在
    TargetNotFound(String),
    /// 消息转换失败
    ConversionFailed(String),
    /// 推送失败
    PushFailed(String),
    /// 其他错误
    Other(String),
}

// 自动实现 From<String> 用于 ? 操作符
impl From<String> for MessageRouterError {
    fn from(s: String) -> Self {
        MessageRouterError::Other(s)
    }
}

impl From<&str> for MessageRouterError {
    fn from(s: &str) -> Self {
        MessageRouterError::Other(s.to_string())
    }
}

impl std::fmt::Display for MessageRouterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidMessageFormat(msg) => write!(f, "Invalid message format: {}", msg),
            Self::TargetNotFound(target) => write!(f, "Target not found: {}", target),
            Self::ConversionFailed(msg) => write!(f, "Conversion failed: {}", msg),
            Self::PushFailed(msg) => write!(f, "Push failed: {}", msg),
            Self::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for MessageRouterError {}

/// 消息路由器 Trait
#[async_trait::async_trait]
pub trait MessageRouterTrait: Send + Sync {
    /// 路由消息
    async fn route_message(&self, message: &UnifiedMessage) -> Result<(), MessageRouterError>;

    /// 向指定 Topic 推送消息
    async fn publish_to_topic(
        &self,
        topic: &str,
        message: &UnifiedMessage,
    ) -> Result<(), MessageRouterError>;

    /// 向指定客户端推送消息
    async fn send_to_client(
        &self,
        client_id: &str,
        message: &UnifiedMessage,
    ) -> Result<(), MessageRouterError>;
}
