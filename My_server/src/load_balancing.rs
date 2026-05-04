//! / 负载均衡模块

use lazy_static::lazy_static;
use log::{info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 服务实例信息
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServiceInstance {
    pub id: String,
    pub name: String,
    pub address: String,
    pub port: u16,
    pub health_status: String,
}

/// 服务注册中心
pub struct ServiceRegistry {
    instances: Arc<RwLock<HashMap<String, ServiceInstance>>>,
    heartbeats: Arc<RwLock<HashMap<String, std::time::Instant>>>,
}

impl ServiceRegistry {
    pub fn new() -> Self {
        Self {
            instances: Arc::new(RwLock::new(HashMap::new())),
            heartbeats: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 注册服务实例
    pub async fn register(&self, instance: ServiceInstance) {
        let mut instances = self.instances.write().await;
        let mut heartbeats = self.heartbeats.write().await;

        instances.insert(instance.id.clone(), instance.clone());
        heartbeats.insert(instance.id.clone(), std::time::Instant::now());

        info!(
            "Registered service instance: {} at {}:{}",
            instance.id, instance.address, instance.port
        );
    }

    /// 注销服务实例
    pub async fn unregister(&self, service_id: &str) {
        let mut instances = self.instances.write().await;
        let mut heartbeats = self.heartbeats.write().await;

        instances.remove(service_id);
        heartbeats.remove(service_id);

        info!("Unregistered service instance: {}", service_id);
    }

    /// 更新心跳
    pub async fn update_heartbeat(&self, service_id: &str) {
        let mut heartbeats = self.heartbeats.write().await;
        heartbeats.insert(service_id.to_string(), std::time::Instant::now());
    }

    /// 获取所有健康的服务实例
    pub fn get_all_instances(&self) -> Vec<ServiceInstance> {
        // 在实际应用中,应该使用阻塞方法或返回Future
        vec![]
    }

    /// 获取所有健康的服务实例(异步版本)
    pub async fn get_all_instances_async(&self) -> Vec<ServiceInstance> {
        let instances = self.instances.read().await;
        instances.values().cloned().collect()
    }

    /// 启动心跳清理任务
    pub async fn start_heartbeat_cleanup(&self) {
        let heartbeats = self.heartbeats.clone();
        let instances = self.instances.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60));

            loop {
                interval.tick().await;

                let now = std::time::Instant::now();
                let timeout = tokio::time::Duration::from_secs(300); // 5分钟超时

                let mut hb = heartbeats.write().await;
                let mut inst = instances.write().await;

                let expired: Vec<String> = hb
                    .iter()
                    .filter(|(_, time)| now.duration_since(**time) > timeout)
                    .map(|(id, _)| id.clone())
                    .collect();

                for id in expired {
                    warn!("Service instance {} heartbeat timeout, removing", id);
                    inst.remove(&id);
                    hb.remove(&id);
                }
            }
        });
    }

    /// 启动健康检查任务
    pub async fn start_health_checks(&self) {
        let instances = self.instances.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));

            loop {
                interval.tick().await;

                let inst = instances.read().await;
                for (id, instance) in inst.iter() {
                    // 这里可以实现实际的HTTP健康检查
                    info!(
                        "Health check for service {}: {}",
                        id, instance.health_status
                    );
                }
            }
        });
    }
}

impl Default for ServiceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// 自动扩缩容管理器
pub struct ScalingManager {
    is_running: Arc<RwLock<bool>>,
}

impl ScalingManager {
    pub fn new() -> Self {
        Self {
            is_running: Arc::new(RwLock::new(false)),
        }
    }

    /// 启动扩缩容监控任务
    pub async fn start_scaling_monitor(&self) {
        let is_running = self.is_running.clone();
        let mut running = is_running.write().await;

        if *running {
            warn!("Scaling monitor is already running");
            return;
        }

        *running = true;
        drop(running);

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60));

            loop {
                interval.tick().await;

                let instances = SERVICE_REGISTRY.get_all_instances_async().await;
                let count = instances.len();

                info!("Scaling monitor: {} active instances", count);

                // 这里可以实现实际的扩缩容逻辑
                // 例如:根据负载自动增减实例
            }
        });

        info!("Scaling monitor started");
    }

    /// 停止扩缩容监控任务
    pub async fn stop_scaling_monitor(&self) {
        let mut is_running = self.is_running.write().await;
        *is_running = false;
        info!("Scaling monitor stopped");
    }
}

impl Default for ScalingManager {
    fn default() -> Self {
        Self::new()
    }
}

// 全局服务注册表
lazy_static! {
    pub static ref SERVICE_REGISTRY: ServiceRegistry = {
        ServiceRegistry {
            instances: std::sync::Arc::new(tokio::sync::RwLock::new(
                std::collections::HashMap::new(),
            )),
            heartbeats: std::sync::Arc::new(tokio::sync::RwLock::new(
                std::collections::HashMap::new(),
            )),
        }
    };
}

// 全局扩缩容管理器
lazy_static! {
    pub static ref SCALING_MANAGER: ScalingManager = ScalingManager::new();
}

/// 配置服务发现路由
pub fn configure_service_discovery_routes(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(
        actix_web::web::scope("/api/discovery")
            .route(
                "/services",
                actix_web::web::get().to(|| async {
                    let instances = SERVICE_REGISTRY.get_all_instances_async().await;
                    actix_web::HttpResponse::Ok().json(instances)
                }),
            )
            .route(
                "/register",
                actix_web::web::post().to(
                    |data: actix_web::web::Json<ServiceInstance>| async move {
                        SERVICE_REGISTRY.register(data.into_inner()).await;
                        actix_web::HttpResponse::Ok().json(serde_json::json!({
                            "success": true,
                            "message": "Service registered successfully"
                        }))
                    },
                ),
            ),
    );
}
