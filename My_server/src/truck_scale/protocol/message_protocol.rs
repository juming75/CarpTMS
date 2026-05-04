//! / 统一消息协议
// 用于 Truck Scale 协议适配器与 CarpTMS 核心服务之间的通信
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// 消息类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageType {
    /// 登录请求
    Login,
    /// 登录响应
    LoginResponse,
    /// 登出请求
    Logout,
    /// 心跳
    Heartbeat,
    /// 车辆数据查询
    QueryVehicle,
    /// 车辆数据响应
    VehicleData,
    /// 用户数据查询
    QueryUser,
    /// 用户数据响应
    UserData,
    /// 车组数据查询
    QueryVehicleGroup,
    /// 车组数据响应
    VehicleGroupData,
    /// 用户组数据查询
    QueryUserGroup,
    /// 用户组数据响应
    UserGroupData,
    /// 数据上报(称重数据)
    DataReport,
    /// 数据上报响应
    DataReportResponse,
    /// 错误消息
    Error,
    /// 通知消息
    Notification,
}

/// 消息优先级
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum MessagePriority {
    Low = 0,
    #[default]
    Normal = 1,
    High = 2,
}

/// 统一消息头
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageHeader {
    /// 消息ID(唯一标识)
    pub message_id: String,
    /// 消息类型
    pub message_type: MessageType,
    /// 时间戳
    pub timestamp: DateTime<Utc>,
    /// 会话ID
    pub session_id: Option<String>,
    /// 用户ID
    pub user_id: Option<String>,
    /// 设备ID(终端编号)
    pub device_id: Option<String>,
    /// 协议类型(BSJ/YW/GBT32960/DB44/TF_CarManager)
    pub protocol_type: Option<String>,
    /// 消息优先级
    pub priority: MessagePriority,
    /// 消息版本
    pub version: String,
}

impl Default for MessageHeader {
    fn default() -> Self {
        Self {
            message_id: Uuid::new_v4().to_string(),
            message_type: MessageType::Notification,
            timestamp: Utc::now(),
            session_id: None,
            user_id: None,
            device_id: None,
            protocol_type: None,
            priority: MessagePriority::default(),
            version: "1.0".to_string(),
        }
    }
}

impl MessageHeader {
    /// 创建新的消息头
    pub fn new(message_type: MessageType) -> Self {
        Self {
            message_type,
            ..Default::default()
        }
    }

    /// 设置会话ID
    pub fn with_session_id(mut self, session_id: String) -> Self {
        self.session_id = Some(session_id);
        self
    }

    /// 设置用户ID
    pub fn with_user_id(mut self, user_id: String) -> Self {
        self.user_id = Some(user_id);
        self
    }

    /// 设置设备ID
    pub fn with_device_id(mut self, device_id: String) -> Self {
        self.device_id = Some(device_id);
        self
    }

    /// 设置协议类型
    pub fn with_protocol_type(mut self, protocol_type: String) -> Self {
        self.protocol_type = Some(protocol_type);
        self
    }

    /// 设置优先级
    pub fn with_priority(mut self, priority: MessagePriority) -> Self {
        self.priority = priority;
        self
    }
}

/// 统一消息体
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "message_type", content = "data")]
pub enum MessageBody {
    /// 登录请求
    Login(LoginRequest),
    /// 登录响应
    LoginResponse(LoginResponse),
    /// 登出请求
    Logout(LogoutRequest),
    /// 心跳
    Heartbeat(Heartbeat),
    /// 车辆数据查询
    QueryVehicle(QueryVehicleRequest),
    /// 车辆数据响应
    VehicleData(VehicleData),
    /// 用户数据查询
    QueryUser(QueryUserRequest),
    /// 用户数据响应
    UserData(UserData),
    /// 车组数据查询
    QueryVehicleGroup(QueryVehicleGroupRequest),
    /// 车组数据响应
    VehicleGroupData(VehicleGroupData),
    /// 用户组数据查询
    QueryUserGroup(QueryUserGroupRequest),
    /// 用户组数据响应
    UserGroupData(UserGroupData),
    /// 数据上报
    DataReport(DataReport),
    /// 数据上报响应
    DataReportResponse(DataReportResponse),
    /// 错误消息
    Error(ErrorMessage),
    /// 通知消息
    Notification(NotificationMessage),
}

/// 登录请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
    pub client_type: String, // client_type: "truck_scale", "web", "mobile"
    pub device_info: Option<DeviceInfo>,
}

/// 登录响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    pub success: bool,
    pub user_id: Option<String>,
    pub user_name: Option<String>,
    pub user_type: Option<i32>,
    pub session_id: Option<String>,
    pub permissions: Option<Vec<String>>,
    pub vehicle_groups: Option<Vec<String>>,
    pub error_code: Option<i32>,
    pub error_message: Option<String>,
}

/// 登出请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogoutRequest {
    pub session_id: String,
}

/// 心跳
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Heartbeat {
    pub timestamp: DateTime<Utc>,
    pub status: String,
}

/// 设备信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub device_id: String,
    pub device_type: String,
    pub os_version: Option<String>,
    pub app_version: Option<String>,
}

/// 车辆数据查询请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryVehicleRequest {
    pub vehicle_id: Option<String>,
    pub plate_no: Option<String>,
    pub group_id: Option<String>,
    pub page: Option<i32>,
    pub page_size: Option<i32>,
}

/// 车辆数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VehicleData {
    pub vehicles: Vec<VehicleInfo>,
    pub total: Option<i32>,
    pub page: Option<i32>,
    pub page_size: Option<i32>,
}

/// 车辆信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VehicleInfo {
    pub vehicle_id: String,
    pub plate_no: String,
    pub terminal_no: Option<String>,
    pub sim_no: Option<String>,
    pub group_id: Option<String>,
    pub driver_name: Option<String>,
    pub driver_tel: Option<String>,
    pub vehicle_type: Option<String>,
    pub vehicle_brand: Option<String>,
    pub vehicle_model: Option<String>,
    pub max_weight: Option<f64>,
    pub tare_weight: Option<f64>,
    pub status: Option<i32>,
}

/// 用户数据查询请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryUserRequest {
    pub user_id: Option<String>,
    pub user_name: Option<String>,
    pub group_id: Option<String>,
    pub user_type: Option<i32>,
    pub page: Option<i32>,
    pub page_size: Option<i32>,
}

/// 用户数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserData {
    pub users: Vec<UserInfo>,
    pub total: Option<i32>,
    pub page: Option<i32>,
    pub page_size: Option<i32>,
}

/// 用户信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub user_id: String,
    pub user_name: String,
    pub real_name: Option<String>,
    pub user_type: i32,
    pub group_id: Option<String>,
    pub company: Option<String>,
    pub mobile: Option<String>,
    pub email: Option<String>,
    pub status: i32,
    pub permissions: Option<Vec<String>>,
}

/// 车组数据查询请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryVehicleGroupRequest {
    pub group_id: Option<String>,
    pub parent_id: Option<String>,
    pub include_children: Option<bool>,
}

/// 车组数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VehicleGroupData {
    pub groups: Vec<VehicleGroupInfo>,
}

/// 车组信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VehicleGroupInfo {
    pub group_id: String,
    pub parent_id: Option<String>,
    pub group_name: String,
    pub contact_people: Option<String>,
    pub contact_tel: Option<String>,
    pub children: Option<Vec<VehicleGroupInfo>>,
}

/// 用户组数据查询请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryUserGroupRequest {
    pub group_id: Option<String>,
    pub user_type: Option<i32>,
}

/// 用户组数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserGroupData {
    pub groups: Vec<UserGroupInfo>,
}

/// 用户组信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserGroupInfo {
    pub group_id: String,
    pub group_name: String,
    pub user_type: i32,
    pub permission: Option<String>,
    pub user_count: Option<i32>,
}

/// 数据上报
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataReport {
    pub report_type: String, // "weighing", "monitoring", "alarm"
    pub device_id: String,
    pub data: serde_json::Value,
    pub timestamp: DateTime<Utc>,
}

/// 数据上报响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataReportResponse {
    pub success: bool,
    pub report_id: Option<String>,
    pub error_code: Option<i32>,
    pub error_message: Option<String>,
}

/// 错误消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorMessage {
    pub error_code: i32,
    pub error_message: String,
    pub details: Option<String>,
}

/// 通知消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationMessage {
    pub notification_type: String,
    pub title: String,
    pub content: String,
    pub data: Option<serde_json::Value>,
}

/// 统一消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedMessage {
    /// 消息头
    pub header: MessageHeader,
    /// 消息体
    pub body: MessageBody,
    /// 扩展字段(可选)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extensions: Option<HashMap<String, serde_json::Value>>,
}

impl UnifiedMessage {
    /// 创建新的消息
    pub fn new(message_type: MessageType, body: MessageBody) -> Self {
        Self {
            header: MessageHeader::new(message_type),
            body,
            extensions: None,
        }
    }

    /// 创建登录请求消息
    pub fn login(username: String, password: String, client_type: String) -> Self {
        Self::new(
            MessageType::Login,
            MessageBody::Login(LoginRequest {
                username,
                password,
                client_type,
                device_info: None,
            }),
        )
    }

    /// 创建登录响应消息
    pub fn login_response(
        success: bool,
        user_id: Option<String>,
        user_name: Option<String>,
    ) -> Self {
        Self::new(
            MessageType::LoginResponse,
            MessageBody::LoginResponse(LoginResponse {
                success,
                user_id,
                user_name,
                user_type: None,
                session_id: None,
                permissions: None,
                vehicle_groups: None,
                error_code: None,
                error_message: None,
            }),
        )
    }

    /// 创建心跳消息
    pub fn heartbeat(session_id: String) -> Self {
        Self::new(
            MessageType::Heartbeat,
            MessageBody::Heartbeat(Heartbeat {
                timestamp: Utc::now(),
                status: "ok".to_string(),
            }),
        )
        .with_session_id(session_id)
    }

    /// 设置会话ID
    pub fn with_session_id(mut self, session_id: String) -> Self {
        self.header.session_id = Some(session_id);
        self
    }

    /// 设置用户ID
    pub fn with_user_id(mut self, user_id: String) -> Self {
        self.header.user_id = Some(user_id);
        self
    }

    /// 设置设备ID
    pub fn with_device_id(mut self, device_id: String) -> Self {
        self.header.device_id = Some(device_id);
        self
    }

    /// 设置协议类型
    pub fn with_protocol_type(mut self, protocol_type: String) -> Self {
        self.header.protocol_type = Some(protocol_type);
        self
    }

    /// 序列化为JSON
    pub fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string(self)?)
    }

    /// 从JSON反序列化
    pub fn from_json(json: &str) -> Result<Self> {
        Ok(serde_json::from_str(json)?)
    }

    /// 序列化为字节
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        Ok(serde_json::to_vec(self)?)
    }

    /// 从字节反序列化
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        Ok(serde_json::from_slice(bytes)?)
    }
}

/// 消息序列化器
pub struct MessageSerializer;

impl MessageSerializer {
    /// 序列化消息为JSON字符串
    pub fn serialize_to_json(message: &UnifiedMessage) -> Result<String> {
        message.to_json()
    }

    /// 从JSON字符串反序列化消息
    pub fn deserialize_from_json(json: &str) -> Result<UnifiedMessage> {
        UnifiedMessage::from_json(json)
    }

    /// 序列化消息为字节
    pub fn serialize_to_bytes(message: &UnifiedMessage) -> Result<Vec<u8>> {
        message.to_bytes()
    }

    /// 从字节反序列化消息
    pub fn deserialize_from_bytes(bytes: &[u8]) -> Result<UnifiedMessage> {
        UnifiedMessage::from_bytes(bytes)
    }
}

impl Default for MessageSerializer {
    fn default() -> Self {
        Self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_serialization() {
        let message = UnifiedMessage::login(
            "test_user".to_string(),
            "test_pass".to_string(),
            "truck_scale".to_string(),
        );

        let json = message.to_json();
        assert!(json.is_ok(), "JSON serialization failed: {:?}", json.err());
        let json = json.unwrap();
        println!("Serialized JSON: {}", json);

        let deserialized = UnifiedMessage::from_json(&json);
        assert!(
            deserialized.is_ok(),
            "JSON deserialization failed: {:?}",
            deserialized.err()
        );
        let deserialized = deserialized.unwrap();
        assert_eq!(message.header.message_id, deserialized.header.message_id);
        assert_eq!(
            message.header.message_type,
            deserialized.header.message_type
        );
    }

    #[test]
    fn test_heartbeat_message() {
        let message = UnifiedMessage::heartbeat("session_123".to_string());

        let json = message.to_json();
        assert!(json.is_ok(), "JSON serialization failed: {:?}", json.err());
        let json = json.unwrap();

        let deserialized = UnifiedMessage::from_json(&json);
        assert!(
            deserialized.is_ok(),
            "JSON deserialization failed: {:?}",
            deserialized.err()
        );
        let deserialized = deserialized.unwrap();

        assert_eq!(
            deserialized.header.session_id,
            Some("session_123".to_string())
        );
    }
}
