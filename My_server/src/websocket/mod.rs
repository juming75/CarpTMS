//! / WebSocket 优化模块

pub mod optimized;

pub use optimized::{
    BatchConfig, HeartbeatManager, MessageBatcher, WsConnectionPool, WsOptimizedMessage,
    WsPushOptimizer,
};






