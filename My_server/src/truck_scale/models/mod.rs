//! / 数据模型模块
// 统一的消息格式

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 统一消息格式(协议无关)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedMessage {
    pub message_id: String,                           // 消息ID(UUID)
    pub message_type: MessageType,                    // 消息类型
    pub timestamp: DateTime<Utc>,                     // 时间戳
    pub source: String,                               // 消息源(协议名称)
    pub vehicle_id: Option<String>,                   // 车辆ID
    pub user_id: Option<String>,                      // 用户ID
    pub data: MessageData,                            // 消息数据
    pub metadata: HashMap<String, serde_json::Value>, // 元数据
}

/// 消息类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    // 实时数据
    RealtimeLocation, // 实时位置
    RealtimeStatus,   // 实时状态
    RealtimeAlarm,    // 实时告警

    // 车辆管理
    VehicleInfoQuery,   // 车辆信息查询
    VehicleInfoUpdate,  // 车辆信息更新
    VehicleGroupQuery,  // 车组查询
    VehicleGroupUpdate, // 车组更新

    // 用户管理
    UserInfoQuery,    // 用户信息查询
    UserInfoUpdate,   // 用户信息更新
    UserGroupQuery,   // 用户组查询
    UserGroupUpdate,  // 用户组更新
    PermissionQuery,  // 权限查询
    PermissionUpdate, // 权限更新

    // 系统事件
    Login,          // 登录
    Logout,         // 登出
    Heartbeat,      // 心跳
    SessionTimeout, // 会话超时
}

/// 消息数据
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MessageData {
    VehicleInfo(serde_json::Value),  // 车辆信息
    UserInfo(serde_json::Value),     // 用户信息
    RealtimeData(serde_json::Value), // 实时数据
    AlarmInfo(serde_json::Value),    // 告警信息
    Empty,                           // 空数据
}
