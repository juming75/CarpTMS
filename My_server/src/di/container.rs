//! /! 依赖注入容器实现

use log::{error, info, warn};
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};

/// 生命周期类型
#[allow(dead_code)]
enum Lifetime {
    Singleton, // 单例模式
    Transient, // 瞬态模式
}

/// 依赖工厂类型
pub type DependencyFactory = Arc<dyn Fn(&Container) -> Arc<dyn Any + Send + Sync> + Send + Sync>;

/// 依赖注册信息
struct DependencyRegistration {
    lifetime: Lifetime,
    factory: DependencyFactory,
    instance: Option<Arc<dyn Any + Send + Sync>>, // 单例实例
}

/// 依赖注入容器
#[derive(Clone)]
pub struct Container {
    registrations: Arc<RwLock<HashMap<TypeId, DependencyRegistration>>>,
    parent: Option<Arc<Container>>,
}

// 实现 Send 和 Sync trait
unsafe impl Send for Container {}
unsafe impl Sync for Container {}

impl Default for Container {
    fn default() -> Self {
        Self::new()
    }
}

impl Container {
    /// 创建新的依赖注入容器
    pub fn new() -> Self {
        Self {
            registrations: Arc::new(RwLock::new(HashMap::new())),
            parent: None,
        }
    }

    /// 创建带有父容器的依赖注入容器
    pub fn with_parent(parent: Arc<Container>) -> Self {
        Self {
            registrations: Arc::new(RwLock::new(HashMap::new())),
            parent: Some(parent),
        }
    }

    /// 注册单例依赖
    pub fn register_singleton<T: Send + Sync + 'static, F>(&self, factory: F)
    where
        F: Fn(&Container) -> Arc<T> + 'static + Send + Sync,
    {
        let type_id = TypeId::of::<T>();
        let registration = DependencyRegistration {
            lifetime: Lifetime::Singleton,
            factory: Arc::new(move |container| {
                let instance = factory(container);
                instance as Arc<dyn Any + Send + Sync>
            }),
            instance: None,
        };

        if let Ok(mut regs) = self.registrations.write() {
            regs.insert(type_id, registration);
            info!(
                "Registered singleton dependency for type: {:?}",
                std::any::type_name::<T>()
            );
        } else {
            error!("Failed to acquire write lock for singleton registration");
        }
    }

    /// 注册瞬态依赖
    pub fn register_transient<T: Send + Sync + 'static, F>(&self, factory: F)
    where
        F: Fn(&Container) -> Arc<T> + 'static + Send + Sync,
    {
        let type_id = TypeId::of::<T>();
        let registration = DependencyRegistration {
            lifetime: Lifetime::Transient,
            factory: Arc::new(move |container| {
                let instance = factory(container);
                instance as Arc<dyn Any + Send + Sync>
            }),
            instance: None,
        };

        if let Ok(mut regs) = self.registrations.write() {
            regs.insert(type_id, registration);
            info!(
                "Registered transient dependency for type: {:?}",
                std::any::type_name::<T>()
            );
        } else {
            error!("Failed to acquire write lock for transient registration");
        }
    }

    /// 解析依赖
    pub fn resolve<T: Send + Sync + 'static>(&self) -> Option<Arc<T>> {
        let type_id = TypeId::of::<T>();

        // 首先在当前容器中查找
        if let Ok(regs) = self.registrations.read() {
            if let Some(registration) = regs.get(&type_id) {
                return self.resolve_registration::<T>(registration);
            }
        }

        // 如果当前容器中没有,在父容器中查找
        if let Some(parent) = &self.parent {
            return parent.resolve::<T>();
        }

        warn!(
            "Dependency not found for type: {:?}",
            std::any::type_name::<T>()
        );
        None
    }

    /// 从注册信息中解析依赖
    fn resolve_registration<T: Send + Sync + 'static>(
        &self,
        registration: &DependencyRegistration,
    ) -> Option<Arc<T>> {
        match registration.lifetime {
            Lifetime::Singleton => {
                // 如果单例实例已存在,直接返回
                if let Some(instance) = &registration.instance {
                    return self.downcast_any_to::<T>(instance.clone());
                }

                // 否则创建新实例
                let instance = (registration.factory)(self);
                let instance_clone = instance.clone();

                // 更新单例实例
                if let Ok(mut registrations) = self.registrations.write() {
                    if let Some(reg) = registrations.get_mut(&TypeId::of::<T>()) {
                        reg.instance = Some(instance);
                    }
                }

                self.downcast_any_to::<T>(instance_clone)
            }
            Lifetime::Transient => {
                // 每次都创建新实例
                let instance = (registration.factory)(self);
                self.downcast_any_to::<T>(instance)
            }
        }
    }

    /// 将 Any 类型转换为指定类型 T
    fn downcast_any_to<T: Send + Sync + 'static>(
        &self,
        any: Arc<dyn Any + Send + Sync>,
    ) -> Option<Arc<T>> {
        // 尝试直接 downcast
        if let Ok(instance) = any.downcast::<T>() {
            return Some(instance);
        }

        None
    }

    /// 检查依赖是否已注册
    pub fn is_registered<T: Send + Sync + 'static>(&self) -> bool {
        let type_id = TypeId::of::<T>();

        // 首先在当前容器中查找
        if let Ok(regs) = self.registrations.read() {
            if regs.contains_key(&type_id) {
                return true;
            }
        }

        // 如果当前容器中没有,在父容器中查找
        if let Some(parent) = &self.parent {
            return parent.is_registered::<T>();
        }

        false
    }

    /// 移除依赖注册
    pub fn unregister<T: Send + Sync + 'static>(&self) {
        let type_id = TypeId::of::<T>();
        if let Ok(mut regs) = self.registrations.write() {
            regs.remove(&type_id);
            info!(
                "Unregistered dependency for type: {:?}",
                std::any::type_name::<T>()
            );
        }
    }

    /// 清空所有依赖注册
    pub fn clear(&self) {
        if let Ok(mut regs) = self.registrations.write() {
            regs.clear();
            info!("Cleared all dependencies");
        }
    }

    /// 获取注册的依赖数量
    pub fn registration_count(&self) -> usize {
        self.registrations.read().ok().map(|r| r.len()).unwrap_or(0)
    }
}

/// 模块 trait,用于组织依赖注册
pub trait Module {
    /// 注册模块依赖
    fn register(&self, container: &Container);
}

/// 容器构建器
pub struct ContainerBuilder {
    container: Container,
}

impl Default for ContainerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ContainerBuilder {
    /// 创建新的容器构建器
    pub fn new() -> Self {
        Self {
            container: Container::new(),
        }
    }

    /// 创建带有父容器的构建器
    pub fn with_parent(parent: Arc<Container>) -> Self {
        Self {
            container: Container::with_parent(parent),
        }
    }

    /// 注册单例依赖
    pub fn register_singleton<T: Send + Sync + 'static, F>(self, factory: F) -> Self
    where
        F: Fn(&Container) -> Arc<T> + 'static + Send + Sync,
    {
        self.container.register_singleton(factory);
        self
    }

    /// 注册瞬态依赖
    pub fn register_transient<T: Send + Sync + 'static, F>(self, factory: F) -> Self
    where
        F: Fn(&Container) -> Arc<T> + 'static + Send + Sync,
    {
        self.container.register_transient(factory);
        self
    }

    /// 注册模块
    pub fn register_module<M: Module>(self, module: M) -> Self {
        module.register(&self.container);
        self
    }

    /// 构建容器
    pub fn build(self) -> Arc<Container> {
        Arc::new(self.container)
    }
}

/// 全局容器管理
#[derive(Default)]
struct GlobalContainer {
    container: Option<Arc<Container>>,
}

lazy_static::lazy_static! {
    static ref GLOBAL_CONTAINER: Mutex<GlobalContainer> = Mutex::new(GlobalContainer::default());
}

/// 初始化全局容器
pub fn init_global_container(container: Arc<Container>) {
    if let Ok(mut global) = GLOBAL_CONTAINER.lock() {
        global.container = Some(container);
        info!("Global container initialized");
    } else {
        error!("Failed to acquire global container lock");
    }
}

/// 获取全局容器
pub fn get_global_container() -> Option<Arc<Container>> {
    GLOBAL_CONTAINER.lock().ok()?.container.clone()
}

/// 从全局容器解析依赖
pub fn resolve<T: Send + Sync + 'static>() -> Option<Arc<T>> {
    if let Some(container) = get_global_container() {
        container.resolve::<T>()
    } else {
        warn!("Global container not initialized");
        None
    }
}

/// 检查全局容器中是否注册了依赖
pub fn is_registered<T: Send + Sync + 'static>() -> bool {
    if let Some(container) = get_global_container() {
        container.is_registered::<T>()
    } else {
        false
    }
}

// ==================== 自适应 AI 管理器支持 ====================

#[cfg(feature = "ai")]
use crate::ai::adaptive::AdaptiveAiManager;

/// 获取全局自适应 AI 管理器
#[cfg(feature = "ai")]
pub async fn get_adaptive_ai_manager() -> Result<Arc<AdaptiveAiManager>, String> {
    // 尝试从容器解析
    if let Some(manager) = resolve::<AdaptiveAiManager>() {
        return Ok(manager);
    }

    // 如果容器中不存在，创建并注册新的管理器
    if let Some(container) = get_global_container() {
        let manager = Arc::new(AdaptiveAiManager::new());

        // 注册为单例
        let manager_clone = manager.clone();
        container.register_singleton::<AdaptiveAiManager, _>(move |_| manager_clone.clone());

        return Ok(manager);
    }

    Err("全局容器未初始化".to_string())
}

/// AI 功能不可用时的占位函数
#[cfg(not(feature = "ai"))]
pub async fn get_adaptive_ai_manager() -> Result<Arc<()>, String> {
    Err("AI功能未启用，请启用 ai feature".to_string())
}
