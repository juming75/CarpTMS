//! /! 健康检查模块
//!
//! 实现服务健康检查和状态监控功能

use super::models::{ServiceHealth, ServiceInfo, ServiceStatus};
use super::registry::ServiceRegistry;
use reqwest::Client;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::sleep;

/// 健康检查器
pub struct HealthChecker {
    /// 服务注册表
    registry: Arc<ServiceRegistry>,
    /// 健康检查间隔
    check_interval: Duration,
    /// 健康检查超时时间
    timeout: Duration,
    /// HTTP客户端
    client: Client,
}

impl HealthChecker {
    /// 创建新的健康检查器
    pub fn new(
        registry: Arc<ServiceRegistry>,
        check_interval: Duration,
        timeout: Duration,
    ) -> Self {
        let client = Client::builder()
            .timeout(timeout)
            .build()
            .expect("Failed to create HTTP client");

        let checker = Self {
            registry,
            check_interval,
            timeout,
            client,
        };

        // 启动健康检查任务
        checker.start_health_check_task();

        checker
    }

    /// 启动健康检查任务
    fn start_health_check_task(&self) {
        let registry = self.registry.clone();
        let check_interval = self.check_interval;
        let timeout = self.timeout;
        let client = self.client.clone();

        tokio::spawn(async move {
            loop {
                sleep(check_interval).await;
                Self::perform_health_checks(registry.clone(), client.clone(), timeout).await;
            }
        });
    }

    /// 执行健康检查
    async fn perform_health_checks(
        registry: Arc<ServiceRegistry>,
        client: Client,
        timeout: Duration,
    ) {
        let services = registry.get_all_services().await;

        for service in services {
            let health = Self::check_service_health(&client, &service, timeout).await;
            let _ = registry
                .update_service_health(&service.service_id, health)
                .await;
        }
    }

    /// 检查服务健康状态
    async fn check_service_health(
        client: &Client,
        service: &ServiceInfo,
        _timeout: Duration,
    ) -> ServiceHealth {
        let start = Instant::now();

        // 尝试访问服务的健康检查端点
        let health_endpoint = format!("http://{}/health", service.address);
        let response = client.get(&health_endpoint).send().await;

        let response_time = start.elapsed().as_millis() as u64;

        match response {
            Ok(resp) if resp.status().is_success() => ServiceHealth {
                status: ServiceStatus::Healthy,
                last_check: Instant::now(),
                details: Some(format!("HTTP {}: {}", resp.status(), health_endpoint)),
                response_time: Some(response_time),
            },
            Ok(resp) => ServiceHealth {
                status: ServiceStatus::Warning,
                last_check: Instant::now(),
                details: Some(format!("HTTP {}: {}", resp.status(), health_endpoint)),
                response_time: Some(response_time),
            },
            Err(e) => ServiceHealth {
                status: ServiceStatus::Unhealthy,
                last_check: Instant::now(),
                details: Some(format!("Error: {}: {}", e, health_endpoint)),
                response_time: Some(response_time),
            },
        }
    }

    /// 手动检查服务健康状态
    pub async fn check_service(&self, service_id: &str) -> Result<ServiceHealth, String> {
        let service = self.registry.get_service(service_id).await;
        if let Some(service) = service {
            let health = Self::check_service_health(&self.client, &service, self.timeout).await;
            self.registry
                .update_service_health(service_id, health.clone())
                .await?;
            Ok(health)
        } else {
            Err(format!("Service not found: {}", service_id))
        }
    }

    /// 检查所有服务的健康状态
    pub async fn check_all_services(&self) -> Vec<(String, ServiceHealth)> {
        let services = self.registry.get_all_services().await;
        let mut results = Vec::new();

        for service in services {
            let health = Self::check_service_health(&self.client, &service, self.timeout).await;
            let _ = self
                .registry
                .update_service_health(&service.service_id, health.clone())
                .await;
            results.push((service.service_id, health));
        }

        results
    }

    /// 获取服务健康状态
    pub async fn get_service_health(&self, service_id: &str) -> Option<ServiceHealth> {
        self.registry
            .get_service(service_id)
            .await
            .map(|service| service.health)
    }

    /// 获取所有服务的健康状态
    pub async fn get_all_service_health(&self) -> Vec<(String, ServiceHealth)> {
        let services = self.registry.get_all_services().await;
        services
            .into_iter()
            .map(|service| (service.service_id, service.health))
            .collect()
    }

    /// 获取健康的服务数量
    pub async fn get_healthy_service_count(&self) -> usize {
        let services = self.registry.get_healthy_services().await;
        services.len()
    }

    /// 获取不健康的服务数量
    pub async fn get_unhealthy_service_count(&self) -> usize {
        let services = self.registry.get_all_services().await;
        services
            .into_iter()
            .filter(|service| !service.is_healthy())
            .count()
    }

    /// 获取服务注册表
    pub fn registry(&self) -> Arc<ServiceRegistry> {
        self.registry.clone()
    }

    /// 设置健康检查间隔
    pub fn set_check_interval(&mut self, interval: Duration) {
        self.check_interval = interval;
    }

    /// 设置健康检查超时时间
    pub fn set_timeout(&mut self, timeout: Duration) {
        self.timeout = timeout;
        // 更新HTTP客户端超时
        self.client = Client::builder()
            .timeout(timeout)
            .build()
            .expect("Failed to create HTTP client");
    }
}
