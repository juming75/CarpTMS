//! / 协议模块
pub mod bsj;
pub mod builder;
pub mod car_manager;
pub mod compression;
pub mod db44;
pub mod encoding;
pub mod gbt32960;
pub mod message_protocol;
pub mod parser;
pub mod yw;

#[cfg(test)]
mod message_protocol_test;

pub use message_protocol::{
    DataReport, DataReportResponse, ErrorMessage, Heartbeat, LoginRequest, LoginResponse,
    LogoutRequest, MessageBody, MessageHeader, MessagePriority, MessageSerializer, MessageType,
    NotificationMessage, QueryUserGroupRequest, QueryUserRequest, QueryVehicleGroupRequest,
    QueryVehicleRequest, UnifiedMessage, UserData, UserGroupData, UserGroupInfo, UserInfo,
    VehicleData, VehicleGroupData, VehicleGroupInfo, VehicleInfo,
};
