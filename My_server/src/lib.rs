//! # CarpTMS Server Library
//!
//! Transportation Management System (TMS) 核心库

// AI 模块（需要 --features ai 启用）
#[cfg(feature = "ai")]
pub mod ai;

// 导出主要模块
pub mod alert;
pub mod app;
pub mod application;
pub mod bff;
pub mod bootstrap;
pub mod cache;
pub mod central;
pub mod config;
pub mod config_center;
pub mod database;
pub mod deployment;
pub mod devices;
pub mod di;
pub mod disaster_recovery;
pub mod dispatch;
pub mod domain;
pub mod errors;
pub mod events;
pub mod feature_flags;
pub mod gateway;
pub mod health;
pub mod infrastructure;
pub mod ansible;
#[cfg(feature = "remote-ops")]
pub mod remote_ops;
pub mod init;
pub mod load_balancing;
pub mod logging;
pub mod metrics;
pub mod microservices; // 微服务模块（DDD架构）
pub mod middleware;
pub mod ml;
pub mod map_service;
pub mod models;
pub mod performance;
pub mod protocols;
pub mod redis;
pub mod routes;
pub mod schemas;
pub mod security;
pub mod service_discovery;
pub mod services;
pub mod sync;
pub mod telemetry;
pub mod tracing;
pub mod truck_scale;
pub mod utils;
pub mod vehicle_comm;
pub mod video;
