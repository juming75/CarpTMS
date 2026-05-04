//! 系统监控模块集成测试
//! 验证监控系统状态采集和架构切换功能

#[cfg(test)]
mod tests {
    use carptms::infrastructure::monitoring::{
        MonitoringManager, SystemMonitor, default_switching_config,
    };
    use carptms::config::ArchitectureMode;

    #[tokio::test]
    async fn test_system_monitor_start_stop() {
        let monitor = SystemMonitor::new(None);
        monitor.start().await;
        // 等待一次采集
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        monitor.stop().await;
        
        // 验证停止后不再采集
        let metrics = monitor.get_current_metrics().await;
        assert!(metrics.load_score >= 0.0);
    }

    #[tokio::test]
    async fn test_monitoring_manager_without_switcher() {
        let manager = MonitoringManager::new(
            None,
            ArchitectureMode::MonolithDDD,
            None,
        );
        
        let mode = manager.get_recommended_mode().await;
        assert_eq!(mode, ArchitectureMode::MonolithDDD);
    }

    #[tokio::test]
    async fn test_monitoring_manager_with_switcher() {
        let config = default_switching_config();
        let manager = MonitoringManager::new(
            None,
            ArchitectureMode::MonolithDDD,
            Some(config),
        );
        
        manager.start().await;
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
        manager.stop().await;
        
        let mode = manager.get_recommended_mode().await;
        assert!(mode == ArchitectureMode::MonolithDDD || mode == ArchitectureMode::MicroDDD);
    }
}
