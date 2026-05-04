//! / 服务发现模块测试

use super::*;
use std::net::SocketAddr;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_info_builder() {
        let builder = ServiceInfoBuilder {
            service_id: "test-service-1".to_string(),
            service_name: "test-service".to_string(),
            version: "1.0.0".to_string(),
            address: "127.0.0.1:8080".parse().unwrap(),
            tags: vec!["test", "development"],
            metadata: serde_json::Value::Object(serde_json::Map::new()),
            weight: 100,
            timeout: std::time::Duration::from_secs(5),
        };

        let service_info = ServiceInfo::new(builder);
        assert_eq!(service_info.service_id, "test-service-1");
        assert_eq!(service_info.service_name, "test-service");
        assert_eq!(service_info.version, "1.0.0");
        assert_eq!(service_info.address, "127.0.0.1:8080".parse().unwrap());
        assert_eq!(service_info.tags, vec!["test", "development"]);
        assert_eq!(service_info.weight, 100);
    }

    #[tokio::test]
    async fn test_service_registry() {
        let registry = ServiceRegistry::new();
        let builder = ServiceInfoBuilder {
            service_id: "test-service-1".to_string(),
            service_name: "test-service".to_string(),
            version: "1.0.0".to_string(),
            address: "127.0.0.1:8080".parse().unwrap(),
            tags: vec!["test"],
            metadata: serde_json::Value::Object(serde_json::Map::new()),
            weight: 100,
            timeout: std::time::Duration::from_secs(5),
        };
        let service_info = ServiceInfo::new(builder);

        // 注册服务
        registry.register(service_info.clone()).unwrap();

        // 发现服务
        let services = registry.get_services_by_name("test-service").await;
        assert_eq!(services.len(), 1);
        assert_eq!(services[0].service_id, "test-service-1");

        // 注销服务
        registry.unregister("test-service-1").unwrap();
        let services = registry.get_services_by_name("test-service").await;
        assert_eq!(services.len(), 0);
    }

    #[tokio::test]
    async fn test_load_balancing() {
        let registry = Arc::new(ServiceRegistry::new());

        // 注册多个服务实例
        for i in 1..=3 {
            let builder = ServiceInfoBuilder {
                service_id: format!("test-service-{}", i),
                service_name: "test-service".to_string(),
                version: "1.0.0".to_string(),
                address: format!("127.0.0.1:{}", 8080 + i).parse().unwrap(),
                tags: vec!["test"],
                metadata: serde_json::Value::Object(serde_json::Map::new()),
                weight: 100,
                timeout: std::time::Duration::from_secs(5),
            };
            let service_info = ServiceInfo::new(builder);
            registry.register(service_info).unwrap();
        }

        // 测试轮询负载均衡
        let discovery = ServiceDiscovery::new(registry, LoadBalancingStrategy::RoundRobin);
        
        // 第一次应该选择第一个服务
        let first_service = discovery.select_service("test-service").await.unwrap();
        assert_eq!(first_service.service_id, "test-service-1");
        
        // 第二次应该选择第二个服务
        let second_service = discovery.select_service("test-service").await.unwrap();
        assert_eq!(second_service.service_id, "test-service-2");
        
        // 第三次应该选择第三个服务
        let third_service = discovery.select_service("test-service").await.unwrap();
        assert_eq!(third_service.service_id, "test-service-3");
        
        // 第四次应该回到第一个服务
        let fourth_service = discovery.select_service("test-service").await.unwrap();
        assert_eq!(fourth_service.service_id, "test-service-1");
    }

    #[tokio::test]
    async fn test_health_check() {
        let registry = Arc::new(ServiceRegistry::new());
        let health_checker = HealthChecker::new(Duration::from_secs(10));

        let builder = ServiceInfoBuilder {
            service_id: "test-service-1".to_string(),
            service_name: "test-service".to_string(),
            version: "1.0.0".to_string(),
            address: "127.0.0.1:8080".parse().unwrap(),
            tags: vec!["test"],
            metadata: serde_json::Value::Object(serde_json::Map::new()),
            weight: 100,
            timeout: std::time::Duration::from_secs(5),
        };
        let service_info = ServiceInfo::new(builder);

        registry.register(service_info.clone()).unwrap();

        // 手动触发健康检查
        health_checker.check_health(&registry).await;

        // 验证服务状态
        let services = registry.get_services_by_name("test-service").await;
        assert_eq!(services.len(), 1);
        assert!(matches!(services[0].health.status, ServiceStatus::Unknown));
    }
}






