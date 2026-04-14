//! /! 监控和追踪系统初始化

/// 初始化监控和追踪系统
///
/// 初始化以下组件:
/// - 日志系统 (根据环境变量配置)
/// - 分布式追踪 (OpenTelemetry)
/// - Prometheus监控指标
pub async fn init_monitoring() -> Result<(), String> {
    // 初始化日志系统
    let log_config = crate::utils::log::load_log_config_from_env();
    if let Err(e) = crate::utils::log::init_logging(log_config).await {
        return Err(format!("Failed to initialize logging: {:?}", e));
    }

    // 初始化分布式追踪
    if let Err(e) = crate::telemetry::init_telemetry("tms_server") {
        log::warn!("Failed to initialize telemetry: {:?}", e);
    } else {
        log::info!("Telemetry initialized successfully");
    }

    // 初始化监控指标
    crate::metrics::init_metrics();
    
    Ok(())
}
