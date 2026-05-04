//! /! 监控和追踪初始化模块

use tracing::info;

/// 初始化监控和追踪系统
pub fn init_monitoring() {
    info!("Initializing monitoring and tracing...");

    // 日志系统已经在utils::log模块中初始化
    // 这里可以添加其他监控相关的初始化代码

    info!("Monitoring and tracing initialized successfully");
}
