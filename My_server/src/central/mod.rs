//! / 中心服务模块
// 负责管理和协调各个组件的工作

pub mod config;
pub mod manager;
pub mod service;

use actix::prelude::*;

// 中心服务启动消息
#[derive(Message)]
#[rtype(result = "Result<(), std::io::Error>")]
pub struct StartCentralService {
    pub config: config::CentralConfig,
}

// 中心服务停止消息
#[derive(Message)]
#[rtype(result = "Result<(), std::io::Error>")]
pub struct StopCentralService;

// 设备注册消息
#[derive(Message)]
#[rtype(result = "Result<(), std::io::Error>")]
pub struct RegisterDevice {
    pub device_id: String,
    pub protocol: String,
    pub addr: std::net::SocketAddr,
}

// 设备注销消息
#[derive(Message)]
#[rtype(result = "Result<(), std::io::Error>")]
pub struct UnregisterDevice {
    pub device_id: String,
    pub reason: String,
}

// 设备数据消息
#[derive(Message)]
#[rtype(result = "Result<(), std::io::Error>")]
pub struct DeviceData {
    pub device_id: String,
    pub protocol: String,
    pub data: Vec<u8>,
    pub timestamp: std::time::SystemTime,
}

// 系统状态消息
#[derive(Message)]
#[rtype(result = "Result<service::SystemStatus, std::io::Error>")]
pub struct GetSystemStatus;

// 数据转发消息
#[derive(Message)]
#[rtype(result = "Result<(), std::io::Error>")]
pub struct ForwardData {
    pub device_id: String,
    pub data: Vec<u8>,
    pub target: String,
}
