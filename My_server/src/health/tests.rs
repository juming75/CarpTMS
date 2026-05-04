//! /! 健康检查模块测试

use super::enhanced::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_checker_creation() {
        let checker = EnhancedHealthChecker::new();
        let config = checker.get_config();
        assert!(config.enable_all_checks);
        assert_eq!(config.check_interval_seconds, 30);
    }

    #[test]
    fn test_update_config() {
        let checker = EnhancedHealthChecker::new();
        
        let mut new_config = DynamicHealthConfig::default();
        new_config.check_interval_seconds = 60;
        new_config.enable_all_checks = false;
        new_config.enabled_checks = vec!["cpu".to_string(), "memory".to_string()];
        
        checker.set_config(new_config.clone());
        
        let retrieved_config = checker.get_config();
        assert_eq!(retrieved_config.check_interval_seconds, 60);
        assert_eq!(retrieved_config.enabled_checks.len(), 2);
    }

    #[test]
    fn test_custom_thresholds() {
        let checker = EnhancedHealthChecker::new();
        
        // 设置自定义阈值
        checker.update_threshold("cpu", 50.0, 70.0);
        
        // 检查阈值是否被应用
        let threshold = checker.get_threshold("cpu");
        assert_eq!(threshold.warning, 50.0);
        assert_eq!(threshold.critical, 70.0);
        
        // 其他检查项应该使用默认阈值
        let memory_threshold = checker.get_threshold("memory");
        assert_eq!(memory_threshold.warning, 80.0);
        assert_eq!(memory_threshold.critical, 90.0);
    }

    #[tokio::test]
    async fn test_health_check_execution() {
        let checker = EnhancedHealthChecker::new();
        
        let status = checker.check_health().await;
        
        assert_eq!(status.service, "tms_server");
        assert_eq!(status.version, "1.1.0");
        assert!(!status.hostname.is_empty());
        assert!(status.checks.contains_key("cpu"));
        assert!(status.checks.contains_key("memory"));
        assert!(status.checks.contains_key("disk"));
        assert!(status.dependencies.contains_key("database"));
        assert!(status.dependencies.contains_key("redis"));
    }

    #[test]
    fn test_check_result_status_determination() {
        let checker = EnhancedHealthChecker::new();
        let thresholds = ThresholdConfig {
            warning: 70.0,
            critical: 85.0,
        };
        
        // 正常情况
        let ok_result = checker.perform_check("test", 50.0, thresholds.clone());
        assert_eq!(ok_result.status, "ok");
        
        // 警告情况
        let warn_result = checker.perform_check("test", 75.0, thresholds.clone());
        assert_eq!(warn_result.status, "warning");
        
        // 严重情况
        let critical_result = checker.perform_check("test", 90.0, thresholds);
        assert_eq!(critical_result.status, "critical");
    }

    #[test]
    fn test_default_threshold_config() {
        let config = DynamicHealthConfig::default();
        assert!(config.enable_all_checks);
        assert!(config.enabled_checks.contains(&"cpu".to_string()));
        assert!(config.enabled_checks.contains(&"memory".to_string()));
        assert!(config.enabled_checks.contains(&"disk".to_string()));
        assert!(!config.notification_config.enabled);
    }

    #[test]
    fn test_global_health_checker() {
        let checker1 = get_enhanced_health_checker();
        let checker2 = get_enhanced_health_checker();
        
        // 应该返回同一个实例
        checker1.set_config(DynamicHealthConfig::default());
        let config1 = checker1.get_config();
        let config2 = checker2.get_config();
        
        assert_eq!(config1.check_interval_seconds, config2.check_interval_seconds);
    }
}
