//! /! 服务发现模块
//!
//! 实现服务发现、过滤和选择功能

use super::models::ServiceInfo;
use super::registry::ServiceRegistry;
use std::net::SocketAddr;
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;

/// 负载均衡策略
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadBalancingStrategy {
    /// 轮询
    RoundRobin,
    /// 随机
    Random,
    /// 最少连接
    LeastConnections,
    /// 权重
    Weighted,
    /// IP哈希
    IpHash,
}

/// 服务发现器
pub struct ServiceDiscovery {
    /// 服务注册表
    registry: Arc<ServiceRegistry>,
    /// 负载均衡策略
    load_balancing_strategy: LoadBalancingStrategy,
    /// 轮询计数器
    round_robin_counter: Arc<AtomicUsize>,
}

impl ServiceDiscovery {
    /// 创建新的服务发现器
    pub fn new(registry: Arc<ServiceRegistry>, strategy: LoadBalancingStrategy) -> Self {
        Self {
            registry,
            load_balancing_strategy: strategy,
            round_robin_counter: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
        }
    }

    /// 发现服务
    pub async fn discover_services(&self, service_name: &str) -> Vec<ServiceInfo> {
        self.registry.get_services_by_name(service_name).await
    }

    /// 发现健康的服务
    pub async fn discover_healthy_services(&self, service_name: &str) -> Vec<ServiceInfo> {
        let services = self.registry.get_services_by_name(service_name).await;
        services
            .into_iter()
            .filter(|service| service.is_healthy())
            .collect()
    }

    /// 发现可用的服务
    pub async fn discover_available_services(&self, service_name: &str) -> Vec<ServiceInfo> {
        let services = self.registry.get_services_by_name(service_name).await;
        services
            .into_iter()
            .filter(|service| service.is_available())
            .collect()
    }

    /// 根据标签发现服务
    pub async fn discover_services_by_tag(&self, tag: &str) -> Vec<ServiceInfo> {
        self.registry.get_services_by_tag(tag).await
    }

    /// 根据标签发现健康的服务
    pub async fn discover_healthy_services_by_tag(&self, tag: &str) -> Vec<ServiceInfo> {
        let services = self.registry.get_services_by_tag(tag).await;
        services
            .into_iter()
            .filter(|service| service.is_healthy())
            .collect()
    }

    /// 选择一个服务
    pub async fn select_service(&self, service_name: &str) -> Option<ServiceInfo> {
        let services = self.discover_available_services(service_name).await;
        if services.is_empty() {
            return None;
        }

        match self.load_balancing_strategy {
            LoadBalancingStrategy::RoundRobin => self.select_round_robin(&services),
            LoadBalancingStrategy::Random => self.select_random(&services),
            LoadBalancingStrategy::Weighted => self.select_weighted(&services),
            // 其他策略可以在这里实现
            _ => self.select_round_robin(&services),
        }
    }

    /// 轮询选择服务
    fn select_round_robin(&self, services: &[ServiceInfo]) -> Option<ServiceInfo> {
        if services.is_empty() {
            return None;
        }

        let index = self
            .round_robin_counter
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let selected_index = index % services.len();
        services.get(selected_index).cloned()
    }

    /// 随机选择服务
    fn select_random(&self, services: &[ServiceInfo]) -> Option<ServiceInfo> {
        if services.is_empty() {
            return None;
        }

        let index = rand::random::<usize>() % services.len();
        services.get(index).cloned()
    }

    /// 权重选择服务
    fn select_weighted(&self, services: &[ServiceInfo]) -> Option<ServiceInfo> {
        if services.is_empty() {
            return None;
        }

        // 计算总权重
        let total_weight: u32 = services.iter().map(|s| s.weight).sum();
        if total_weight == 0 {
            return self.select_random(services);
        }

        // 生成随机数
        let mut random = rand::random::<u32>() % total_weight;

        // 根据权重选择服务
        for service in services {
            if random < service.weight {
                return Some(service.clone());
            }
            random -= service.weight;
        }

        // 以防万一
        services.first().cloned()
    }

    /// 获取服务地址
    pub async fn get_service_address(&self, service_name: &str) -> Option<SocketAddr> {
        self.select_service(service_name)
            .await
            .map(|service| service.address)
    }

    /// 获取所有服务地址
    pub async fn get_all_service_addresses(&self, service_name: &str) -> Vec<SocketAddr> {
        let services = self.discover_available_services(service_name).await;
        services
            .into_iter()
            .map(|service| service.address)
            .collect()
    }

    /// 设置负载均衡策略
    pub fn set_load_balancing_strategy(&mut self, strategy: LoadBalancingStrategy) {
        self.load_balancing_strategy = strategy;
    }

    /// 获取负载均衡策略
    pub fn load_balancing_strategy(&self) -> LoadBalancingStrategy {
        self.load_balancing_strategy
    }

    /// 获取服务注册表
    pub fn registry(&self) -> Arc<ServiceRegistry> {
        self.registry.clone()
    }
}
