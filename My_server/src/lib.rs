//! # CarpTMS Server Library
//!
//! Transportation Management System (TMS) 核心库

// 导出主要模块

pub mod ai;
pub mod alert;
pub mod app;
pub mod application;
pub mod bff;
pub mod bootstrap;
pub mod cache;
pub mod central;
pub mod config;
pub mod config_center;
pub mod deployment;
pub mod di;
pub mod disaster_recovery;
pub mod domain;
pub mod errors;
pub mod events;
pub mod gateway;
pub mod health;
pub mod infrastructure;
pub mod init;
pub mod load_balancing;
pub mod logging;
pub mod metrics;
// pub mod microservices; // TODO: microservices module removed, will be re-added later
pub mod middleware;
pub mod ml;
pub mod models;
pub mod performance;
pub mod protocols;
pub mod redis;
pub mod routes;
pub mod services;
pub mod schemas;
pub mod service_discovery;
pub mod sync;
pub mod telemetry;
pub mod tracing;
pub mod truck_scale;
pub mod utils;
pub mod vehicle_comm;
pub mod video;

// 导出健康检查和指标端点函数
pub use health::health_check;
pub use health::liveness_check;
pub use health::metrics_endpoint;
pub use health::readiness_check;
