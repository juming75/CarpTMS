//! / 依赖注入模块

// 导出容器实现
pub mod container;
pub mod example;
pub use container::{
    get_global_container, init_global_container, is_registered, resolve, Container,
    ContainerBuilder, Module,
};

use log::error;

// 导出旧的容器类型以保持兼容性
use sqlx::PgPool;
use std::sync::Arc;

use crate::domain::event_logger::DomainEventLogger; // 统一事件日志
use crate::domain::use_cases::device::{DeviceRepository, DeviceUseCases};
use crate::domain::use_cases::order::{OrderRepository, OrderUseCases};
use crate::domain::use_cases::user::{UserRepository, UserUseCases};
use crate::domain::use_cases::vehicle::{VehicleRepository, VehicleUseCases};
use crate::domain::use_cases::weighing_data::{
    WeighingDataRepository,    // 来自 repository.rs
    WeighingDataStatsUseCases, // 来自 stats.rs
    WeighingDataUseCases,      // 来自 service.rs
};
use crate::infrastructure::repositories::device_repository::PgDeviceRepository;
use crate::infrastructure::repositories::order_repository::PgOrderRepository;
use crate::infrastructure::repositories::user_repository::PgUserRepository;
use crate::infrastructure::repositories::vehicle_repository::PgVehicleRepository;
use crate::infrastructure::repositories::weighing_data_repository::PgWeighingDataRepository;

/// 依赖注入容器(兼容旧版本)
#[derive(Clone)]
pub struct DIContainer {
    pub db_pool: Arc<PgPool>,

    // 仓库
    pub device_repository: Arc<dyn DeviceRepository>,
    pub order_repository: Arc<dyn OrderRepository>,
    pub user_repository: Arc<dyn UserRepository>,
    pub vehicle_repository: Arc<dyn VehicleRepository>,
    pub weighing_data_repository: Arc<dyn WeighingDataRepository>,

    // 用例
    pub device_use_cases: DeviceUseCases,
    pub order_use_cases: OrderUseCases,
    pub user_use_cases: UserUseCases,
    pub vehicle_use_cases: VehicleUseCases,
    pub weighing_data_use_cases: WeighingDataUseCases,
    pub weighing_data_stats_use_cases: WeighingDataStatsUseCases,

    // 事件日志
    pub event_logger: Arc<DomainEventLogger>,
}

impl DIContainer {
    /// 创建依赖注入容器
    pub fn new(db_pool: Arc<PgPool>) -> Self {
        // 创建仓库实例
        let device_repository = Arc::new(PgDeviceRepository::new(db_pool.clone()));
        let order_repository = Arc::new(PgOrderRepository::new(db_pool.clone()));
        let user_repository = Arc::new(PgUserRepository::new(db_pool.clone()));
        let vehicle_repository = Arc::new(PgVehicleRepository::new(db_pool.clone()));
        let weighing_data_repository = Arc::new(PgWeighingDataRepository::new(db_pool.clone()));

        // 创建用例实例
        let device_use_cases = DeviceUseCases::new(device_repository.clone());
        let order_use_cases = OrderUseCases::new(order_repository.clone());
        let user_use_cases = UserUseCases::new(user_repository.clone());
        let vehicle_use_cases = VehicleUseCases::new(vehicle_repository.clone());
        let weighing_data_use_cases = WeighingDataUseCases::new(weighing_data_repository.clone());
        let weighing_data_stats_use_cases =
            WeighingDataStatsUseCases::new(weighing_data_repository.clone());

        // 创建统一事件日志记录器
        let event_logger = Arc::new(DomainEventLogger::new());

        // 返回容器实例
        Self {
            db_pool,

            // 仓库
            device_repository,
            order_repository,
            user_repository,
            vehicle_repository,
            weighing_data_repository,

            // 用例
            device_use_cases,
            order_use_cases,
            user_use_cases,
            vehicle_use_cases,
            weighing_data_use_cases,
            weighing_data_stats_use_cases,

            // 事件日志
            event_logger,
        }
    }
}

/// 全局依赖注入容器(兼容旧版本)
use lazy_static::lazy_static;
use std::sync::Mutex;

lazy_static! {
    static ref GLOBAL_DI_CONTAINER: Mutex<Option<DIContainer>> = Mutex::new(None);
}

/// 初始化全局依赖注入容器(兼容旧版本)
pub fn init_di_container(db_pool: Arc<PgPool>) {
    if let Ok(mut global) = GLOBAL_DI_CONTAINER.lock() {
        *global = Some(DIContainer::new(db_pool));
    } else {
        error!("Failed to acquire DI container lock");
    }
}

/// 获取全局依赖注入容器(兼容旧版本)
pub fn get_di_container() -> Option<Arc<DIContainer>> {
    GLOBAL_DI_CONTAINER
        .lock()
        .ok()
        .and_then(|guard| guard.as_ref().map(|container| Arc::new(container.clone())))
}

/// 注册默认依赖
/// 注意：当前使用旧的 DIContainer，新的 Container 注册暂未完全实现 trait object 支持
pub fn register_default_dependencies(_container: &Container, _db_pool: Arc<PgPool>) {
    // 新版 Container 对于 trait object 的支持还需要进一步开发
    // 当前使用 DIContainer (旧版) 进行依赖注入
    // 参见 init_di_container 函数
}
