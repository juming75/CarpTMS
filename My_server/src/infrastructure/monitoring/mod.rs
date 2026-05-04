//! 系统监控模块
//!
//! 提供系统指标收集和架构自动切换功能

pub mod architecture_switcher;
pub mod system_monitor;

pub use system_monitor::{
    BusinessMetrics, DatabaseMetrics, PerformanceMetrics, ResourceMetrics, SystemMetrics,
    SystemMonitor,
};

pub use architecture_switcher::{
    ArchitectureSwitcher, SwitchingConfig, SwitchingDecision, SwitchingHistory,
};

use crate::config::ArchitectureMode;
use std::sync::Arc;

/// 监控管理器
pub struct MonitoringManager {
    /// 系统监控器
    pub system_monitor: Arc<SystemMonitor>,
    /// 架构切换器
    pub architecture_switcher: Option<Arc<ArchitectureSwitcher>>,
}

impl MonitoringManager {
    /// 创建新的监控管理器
    pub fn new(
        db_pool: Option<sqlx::PgPool>,
        initial_mode: ArchitectureMode,
        switching_config: Option<SwitchingConfig>,
    ) -> Self {
        let system_monitor = Arc::new(SystemMonitor::new(db_pool));

        let architecture_switcher = switching_config.map(|config| {
            Arc::new(ArchitectureSwitcher::new(
                initial_mode,
                system_monitor.clone(),
                config,
            ))
        });

        Self {
            system_monitor,
            architecture_switcher,
        }
    }

    /// 启动监控
    pub async fn start(&self) {
        // 启动系统监控
        self.system_monitor.start().await;

        // 启动架构切换器（如果启用）
        if let Some(switcher) = &self.architecture_switcher {
            switcher.start().await;
        }

        tracing::info!("Monitoring manager started");
    }

    /// 停止监控
    pub async fn stop(&self) {
        self.system_monitor.stop().await;

        if let Some(switcher) = &self.architecture_switcher {
            switcher.stop().await;
        }

        tracing::info!("Monitoring manager stopped");
    }

    /// 获取当前推荐的架构模式
    pub async fn get_recommended_mode(&self) -> ArchitectureMode {
        if let Some(switcher) = &self.architecture_switcher {
            switcher.get_current_mode().await
        } else {
            ArchitectureMode::MonolithDDD
        }
    }
}

/// 默认切换配置
pub fn default_switching_config() -> SwitchingConfig {
    SwitchingConfig::default()
}

/// 保守的切换配置（更难触发切换）
pub fn conservative_switching_config() -> SwitchingConfig {
    SwitchingConfig {
        enabled: true,
        microservice_threshold: 80.0,     // 负载超过80%才考虑切换
        monolith_threshold: 20.0,         // 负载低于20%才回退
        consecutive_threshold: 5,         // 连续5次超过阈值
        check_interval_secs: 600,         // 每10分钟检查一次
        min_runtime_minutes: 60,          // 最少运行1小时
        data_volume_threshold: 5_000_000, // 500万记录
        qps_threshold: 5000.0,            // 每秒5000请求
        response_time_threshold: 1000.0,  // 1000ms响应时间
    }
}

/// 激进的切换配置（更容易触发切换）
pub fn aggressive_switching_config() -> SwitchingConfig {
    SwitchingConfig {
        enabled: true,
        microservice_threshold: 50.0,   // 负载超过50%就考虑切换
        monolith_threshold: 40.0,       // 负载低于40%就回退
        consecutive_threshold: 2,       // 连续2次超过阈值
        check_interval_secs: 180,       // 每3分钟检查一次
        min_runtime_minutes: 15,        // 最少运行15分钟
        data_volume_threshold: 500_000, // 50万记录
        qps_threshold: 500.0,           // 每秒500请求
        response_time_threshold: 300.0, // 300ms响应时间
    }
}
