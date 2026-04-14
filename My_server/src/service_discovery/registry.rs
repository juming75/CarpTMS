//! /! 服务注册模块
//!
//! 实现服务注册、注销和管理功能

use super::models::{ServiceHealth, ServiceInfo};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use tokio::time::sleep;
use rand::Rng;

/// 服务ID集合类型
pub type ServiceIdSet = HashSet<String>;
/// 服务版本索引类型
pub type VersionIndex = HashMap<String, HashMap<String, ServiceIdSet>>;
/// 服务名称索引类型
pub type ServiceNameIndex = HashMap<String, ServiceIdSet>;
/// 服务标签索引类型
pub type TagIndex = HashMap<String, ServiceIdSet>;

/// 服务注册表
pub struct ServiceRegistry {
    /// 服务存储
    services: Arc<RwLock<HashMap<String, ServiceInfo>>>,
    /// 服务名称索引
    service_name_index: Arc<RwLock<ServiceNameIndex>>,
    /// 服务标签索引
    tag_index: Arc<RwLock<TagIndex>>,
    /// 清理间隔
    cleanup_interval: Duration,
    /// 服务超时时间
    service_timeout: Duration,
    /// 服务版本索引
    version_index: Arc<RwLock<VersionIndex>>,
    /// 服务依赖关系
    dependencies: Arc<RwLock<HashMap<String, ServiceIdSet>>>,
    /// 服务元数据
    metadata: Arc<RwLock<HashMap<String, HashMap<String, String>>>>,
}

impl ServiceRegistry {
    /// 创建新的服务注册表
    pub fn new(cleanup_interval: Duration, service_timeout: Duration) -> Self {
        let registry = Self {
            services: Arc::new(RwLock::new(HashMap::new())),
            service_name_index: Arc::new(RwLock::new(HashMap::new())),
            tag_index: Arc::new(RwLock::new(HashMap::new())),
            version_index: Arc::new(RwLock::new(HashMap::new())),
            dependencies: Arc::new(RwLock::new(HashMap::new())),
            metadata: Arc::new(RwLock::new(HashMap::new())),
            cleanup_interval,
            service_timeout,
        };

        // 启动清理任务
        registry.start_cleanup_task();

        registry
    }

    /// 启动清理任务
    fn start_cleanup_task(&self) {
        let services = self.services.clone();
        let service_name_index = self.service_name_index.clone();
        let tag_index = self.tag_index.clone();
        let version_index = self.version_index.clone();
        let dependencies = self.dependencies.clone();
        let metadata = self.metadata.clone();
        let cleanup_interval = self.cleanup_interval;
        let service_timeout = self.service_timeout;

        tokio::spawn(async move {
            loop {
                sleep(cleanup_interval).await;
                Self::cleanup_expired_services(
                    services.clone(),
                    service_name_index.clone(),
                    tag_index.clone(),
                    version_index.clone(),
                    dependencies.clone(),
                    metadata.clone(),
                    service_timeout,
                );
            }
        });
    }

    /// 清理过期服务
    fn cleanup_expired_services(
        services: Arc<RwLock<HashMap<String, ServiceInfo>>>,
        service_name_index: Arc<RwLock<ServiceNameIndex>>,
        tag_index: Arc<RwLock<TagIndex>>,
        version_index: Arc<RwLock<VersionIndex>>,
        dependencies: Arc<RwLock<HashMap<String, ServiceIdSet>>>,
        metadata: Arc<RwLock<HashMap<String, HashMap<String, String>>>>,
        service_timeout: Duration,
    ) {
        let now = Instant::now();
        let mut expired_service_ids: Vec<String> = Vec::new();

        // 找出过期的服务
        {
            if let Ok(services) = services.read() {
                for (service_id, service) in services.iter() {
                    if now.duration_since(service.last_updated) > service_timeout {
                        expired_service_ids.push(service_id.clone());
                    }
                }
            }
        }

        // 移除过期的服务
        for service_id in expired_service_ids {
            let service_info = {
                if let Ok(mut services) = services.write() {
                    services.remove(&service_id)
                } else {
                    continue;
                }
            };

            if let Some(service) = service_info {
                // 从名称索引中移除
                {
                    if let Ok(mut name_index) = service_name_index.write() {
                        if let Some(ids) = name_index.get_mut(&service.service_name) {
                            ids.remove(&service_id);
                            if ids.is_empty() {
                                name_index.remove(&service.service_name);
                            }
                        }
                    }
                }

                // 从标签索引中移除
                {
                    if let Ok(mut tag_index) = tag_index.write() {
                        for tag in &service.tags {
                            if let Some(ids) = tag_index.get_mut(tag) {
                                ids.remove(&service_id);
                                if ids.is_empty() {
                                    tag_index.remove(tag);
                                }
                            }
                        }
                    }
                }

                // 从版本索引中移除
                {
                    if let Ok(mut version_index) = version_index.write() {
                        if let Some(versions) = version_index.get_mut(&service.service_name) {
                            // 先收集需要删除的版本
                            let mut versions_to_remove = vec![];
                            for (version, ids) in versions.iter_mut() {
                                ids.remove(&service_id);
                                if ids.is_empty() {
                                    versions_to_remove.push(version.clone());
                                }
                            }
                            // 然后删除空的版本
                            for version in versions_to_remove {
                                versions.remove(&version);
                            }
                            if versions.is_empty() {
                                version_index.remove(&service.service_name);
                            }
                        }
                    }
                }

                // 从依赖关系中移除
                {
                    if let Ok(mut dependencies) = dependencies.write() {
                        dependencies.remove(&service_id);
                    }
                }

                // 从元数据中移除
                {
                    if let Ok(mut metadata) = metadata.write() {
                        metadata.remove(&service_id);
                    }
                }

                log::info!("Cleaned up expired service: {}", service_id);
            }
        }
    }

    /// 注册服务
    pub async fn register_service(&self, service_info: ServiceInfo) -> Result<(), String> {
        let service_id = service_info.service_id.clone();
        let service_name = service_info.service_name.clone();
        let tags = service_info.tags.clone();
        let version = service_info.version.clone();

        // 添加服务到注册表
        {
            let mut services = self.services.write().map_err(|e| e.to_string())?;
            services.insert(service_id.clone(), service_info);
        }

        // 更新名称索引
        {
            let mut name_index = self.service_name_index.write().map_err(|e| e.to_string())?;
            name_index
                .entry(service_name.clone())
                .or_default()
                .insert(service_id.clone());
        }

        // 更新标签索引
        {
            let mut tag_index = self.tag_index.write().map_err(|e| e.to_string())?;
            for tag in tags {
                tag_index.entry(tag).or_default().insert(service_id.clone());
            }
        }

        // 更新版本索引
        {
            let mut version_index = self.version_index.write().map_err(|e| e.to_string())?;
            version_index
                .entry(service_name.clone())
                .or_default()
                .entry(version)
                .or_default()
                .insert(service_id.clone());
        }

        // 初始化依赖关系
        {
            let mut dependencies = self.dependencies.write().map_err(|e| e.to_string())?;
            dependencies.insert(service_id.clone(), HashSet::new());
        }

        // 初始化元数据
        {
            let mut metadata = self.metadata.write().map_err(|e| e.to_string())?;
            metadata.insert(service_id.clone(), HashMap::new());
        }

        log::info!("Service registered: {}", service_id);
        Ok(())
    }

    /// 注销服务
    pub async fn unregister_service(&self, service_id: &str) -> Result<(), String> {
        let service_info = {
            let mut services = self.services.write().map_err(|e| e.to_string())?;
            services.remove(service_id)
        };

        if let Some(service) = service_info {
            // 从名称索引中移除
            {
                let mut name_index = self.service_name_index.write().map_err(|e| e.to_string())?;
                if let Some(ids) = name_index.get_mut(&service.service_name) {
                    ids.remove(service_id);
                    if ids.is_empty() {
                        name_index.remove(&service.service_name);
                    }
                }
            }

            // 从标签索引中移除
            {
                let mut tag_index = self.tag_index.write().map_err(|e| e.to_string())?;
                for tag in &service.tags {
                    if let Some(ids) = tag_index.get_mut(tag) {
                        ids.remove(service_id);
                        if ids.is_empty() {
                            tag_index.remove(tag);
                        }
                    }
                }
            }

            // 从版本索引中移除
            {
                let mut version_index = self.version_index.write().map_err(|e| e.to_string())?;
                if let Some(versions) = version_index.get_mut(&service.service_name) {
                    let version = &service.version;
                    if let Some(ids) = versions.get_mut(version) {
                        ids.remove(service_id);
                        if ids.is_empty() {
                            versions.remove(version);
                        }
                    }
                    if versions.is_empty() {
                        version_index.remove(&service.service_name);
                    }
                }
            }

            // 从依赖关系中移除
            {
                let mut dependencies = self.dependencies.write().map_err(|e| e.to_string())?;
                dependencies.remove(service_id);
            }

            // 从元数据中移除
            {
                let mut metadata = self.metadata.write().map_err(|e| e.to_string())?;
                metadata.remove(service_id);
            }

            log::info!("Service unregistered: {}", service_id);
            Ok(())
        } else {
            Err(format!("Service not found: {}", service_id))
        }
    }

    /// 更新服务健康状态
    pub async fn update_service_health(
        &self,
        service_id: &str,
        health: ServiceHealth,
    ) -> Result<(), String> {
        let mut services = self.services.write().map_err(|e| e.to_string())?;
        if let Some(service) = services.get_mut(service_id) {
            service.update_health(health);
            log::debug!("Updated service health: {}", service_id);
            Ok(())
        } else {
            Err(format!("Service not found: {}", service_id))
        }
    }

    /// 获取服务信息
    pub async fn get_service(&self, service_id: &str) -> Option<ServiceInfo> {
        self.services.read().ok()?.get(service_id).cloned()
    }

    /// 根据服务名称获取服务
    pub async fn get_services_by_name(&self, service_name: &str) -> Vec<ServiceInfo> {
        if let (Ok(name_index), Ok(services)) = (self.service_name_index.read(), self.services.read()) {
            if let Some(ids) = name_index.get(service_name) {
                return ids.iter()
                    .filter_map(|id| services.get(id))
                    .cloned()
                    .collect();
            }
        }
        Vec::new()
    }

    /// 根据标签获取服务
    pub async fn get_services_by_tag(&self, tag: &str) -> Vec<ServiceInfo> {
        if let (Ok(tag_index), Ok(services)) = (self.tag_index.read(), self.services.read()) {
            if let Some(ids) = tag_index.get(tag) {
                return ids.iter()
                    .filter_map(|id| services.get(id))
                    .cloned()
                    .collect();
            }
        }
        Vec::new()
    }

    /// 获取所有服务
    pub async fn get_all_services(&self) -> Vec<ServiceInfo> {
        self.services.read().ok()
            .map(|s| s.values().cloned().collect())
            .unwrap_or_default()
    }

    /// 获取健康的服务
    pub async fn get_healthy_services(&self) -> Vec<ServiceInfo> {
        self.services.read().ok()
            .map(|s| s.values().filter(|service| service.is_healthy()).cloned().collect())
            .unwrap_or_default()
    }

    /// 获取可用的服务
    pub async fn get_available_services(&self) -> Vec<ServiceInfo> {
        self.services.read().ok()
            .map(|s| s.values().filter(|service| service.is_available()).cloned().collect())
            .unwrap_or_default()
    }

    /// 获取服务数量
    pub async fn get_service_count(&self) -> usize {
        self.services.read().ok().map(|s| s.len()).unwrap_or(0)
    }

    /// 根据服务名称和版本获取服务
    pub async fn get_services_by_version(&self, service_name: &str, version: &str) -> Vec<ServiceInfo> {
        if let (Ok(version_index), Ok(services)) = (self.version_index.read(), self.services.read()) {
            if let Some(versions) = version_index.get(service_name) {
                if let Some(ids) = versions.get(version) {
                    return ids.iter()
                        .filter_map(|id| services.get(id))
                        .cloned()
                        .collect();
                }
            }
        }
        Vec::new()
    }

    /// 设置服务元数据
    pub async fn set_service_metadata(&self, service_id: &str, key: &str, value: &str) -> Result<(), String> {
        let mut metadata = self.metadata.write().map_err(|e| e.to_string())?;
        if let Some(service_metadata) = metadata.get_mut(service_id) {
            service_metadata.insert(key.to_string(), value.to_string());
            log::debug!("Set metadata for service {}: {}={}", service_id, key, value);
            Ok(())
        } else {
            Err(format!("Service not found: {}", service_id))
        }
    }

    /// 获取服务元数据
    pub async fn get_service_metadata(&self, service_id: &str, key: &str) -> Option<String> {
        self.metadata.read().ok()?
            .get(service_id)?
            .get(key)
            .cloned()
    }

    /// 获取服务的所有元数据
    pub async fn get_service_all_metadata(&self, service_id: &str) -> Option<HashMap<String, String>> {
        self.metadata.read().ok()?.get(service_id).cloned()
    }

    /// 添加服务依赖
    pub async fn add_service_dependency(&self, service_id: &str, dependency_id: &str) -> Result<(), String> {
        let mut dependencies = self.dependencies.write().map_err(|e| e.to_string())?;
        if let Some(service_deps) = dependencies.get_mut(service_id) {
            service_deps.insert(dependency_id.to_string());
            log::debug!("Added dependency {} for service {}", dependency_id, service_id);
            Ok(())
        } else {
            Err(format!("Service not found: {}", service_id))
        }
    }

    /// 移除服务依赖
    pub async fn remove_service_dependency(&self, service_id: &str, dependency_id: &str) -> Result<(), String> {
        let mut dependencies = self.dependencies.write().map_err(|e| e.to_string())?;
        if let Some(service_deps) = dependencies.get_mut(service_id) {
            service_deps.remove(dependency_id);
            log::debug!("Removed dependency {} for service {}", dependency_id, service_id);
            Ok(())
        } else {
            Err(format!("Service not found: {}", service_id))
        }
    }

    /// 获取服务的依赖
    pub async fn get_service_dependencies(&self, service_id: &str) -> Option<HashSet<String>> {
        self.dependencies.read().ok()?.get(service_id).cloned()
    }

    /// 负载均衡选择服务（随机选择）
    pub async fn select_service(&self, service_name: &str) -> Option<ServiceInfo> {
        let services = self.get_services_by_name(service_name).await;
        let healthy_services: Vec<ServiceInfo> = services
            .into_iter()
            .filter(|service| service.is_healthy())
            .collect();

        if healthy_services.is_empty() {
            None
        } else {
            let mut rng = rand::thread_rng();
            let index = rng.gen_range(0..healthy_services.len());
            Some(healthy_services[index].clone())
        }
    }

    /// 负载均衡选择服务（基于版本）
    pub async fn select_service_by_version(&self, service_name: &str, version: &str) -> Option<ServiceInfo> {
        let services = self.get_services_by_version(service_name, version).await;
        let healthy_services: Vec<ServiceInfo> = services
            .into_iter()
            .filter(|service| service.is_healthy())
            .collect();

        if healthy_services.is_empty() {
            None
        } else {
            let mut rng = rand::thread_rng();
            let index = rng.gen_range(0..healthy_services.len());
            Some(healthy_services[index].clone())
        }
    }
}
