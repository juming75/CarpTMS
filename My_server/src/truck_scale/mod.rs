//! / Truck Scale 3.5 协议适配模块
//
// 此模块为 Truck Scale 3.5 客户端提供协议适配服务
// 支持以下协议:
// - BSJ 协议
// - YW 协议
// - GB/T 32960 协议
// - DB44 协议
// - TF_CarManager 协议

pub mod auth;
pub mod config;
// pub mod crud; // TODO: crud module removed, will be re-added later
pub mod db;
pub mod handlers;
pub mod integration;
pub mod models;
pub mod protocol;
pub mod tcp_gateway;
pub mod transformers;
pub mod websocket;

pub use config::TruckScaleConfig;
// pub use crud::{TruckScaleCrud, WeighingRecord};
pub use db::TruckScaleDb;
pub use integration::ServiceIntegration;
pub use protocol::builder::ProtocolBuilder;
pub use protocol::message_protocol::{MessageSerializer, UnifiedMessage};
pub use protocol::parser::ProtocolParser;
pub use tcp_gateway::server::TruckScaleServer;
pub use websocket::WebSocketMessageForwarder;

#[cfg(test)]
mod tests {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/truck_scale/db_test.rs"
    ));
}
