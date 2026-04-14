//! / 网关模块
// 负责处理GPRS、TCP、UDP和WebSocket设备的连接、数据接收和发送

pub mod api_gateway;
pub mod client_api_server;
pub mod gprs_client;
pub mod gprs_server;
pub mod handlers;
pub mod router;
pub mod tcp_server;
pub mod udp_server;
pub mod websocket_manager;
pub mod websocket_server;

// 导出 WebSocket 路由
pub use websocket_server::websocket_index_route;

// 导出路由器和处理器
pub use router::{ArchitectureRouter, RouteRule, RouteTarget, RouterStats};
pub use handlers::{
    ApiResponse, HandlerFactory, MonolithOrderHandler, MonolithVehicleHandler,
    MicroserviceOrderHandler, MicroserviceVehicleHandler, RequestContext, RequestHandler,
};

use actix::prelude::*;
use std::net::SocketAddr;

// 网关配置信息
#[derive(Debug, Clone)]
pub struct GatewayConfig {
    pub gprs_server_addr: SocketAddr,
    pub tcp_server_addr: SocketAddr,
    pub udp_server_addr: SocketAddr,
    pub heartbeat_interval: u64,
    pub max_connections: usize,
}

impl Default for GatewayConfig {
    fn default() -> Self {
        Self {
            gprs_server_addr: "0.0.0.0:8888".parse().expect("hardcoded addr"),
            tcp_server_addr: "0.0.0.0:9999".parse().expect("hardcoded addr"),
            udp_server_addr: "0.0.0.0:7777".parse().expect("hardcoded addr"),
            heartbeat_interval: 30,
            max_connections: 1000,
        }
    }
}

// 网关启动消息
#[derive(Message)]
#[rtype(result = "Result<(), std::io::Error>")]
pub struct StartGateway {
    pub config: GatewayConfig,
}

// 网关停止消息
#[derive(Message)]
#[rtype(result = "Result<(), std::io::Error>")]
pub struct StopGateway;

// 设备连接消息
#[derive(Message)]
#[rtype(result = "()")]
pub struct DeviceConnected {
    pub device_id: String,
    pub addr: SocketAddr,
    pub protocol: String,
}

// 设备断开连接消息
#[derive(Message)]
#[rtype(result = "()")]
pub struct DeviceDisconnected {
    pub device_id: String,
    pub reason: String,
}

// 设备数据消息
#[derive(Message)]
#[rtype(result = "()")]
pub struct DeviceData {
    pub device_id: String,
    pub data: Vec<u8>,
    pub protocol: String,
}
