//! /! 服务器最终设置模块
//! 
//! 包含 main.rs 中除核心初始化外的所有服务初始化逻辑

use log::{info, warn};
use std::sync::Arc;
use std::time::Duration;
use sysinfo::SystemExt;

use crate::config::unified::UnifiedConfig;
use crate::init::AppConfig;
use ::redis::Client as RedisClient;
use crate::domain::use_cases::driver::{DriverUseCases, DriverRepository};
use crate::domain::use_cases::department::{DepartmentUseCases, DepartmentRepository};
use crate::domain::use_cases::auth::{AuthUseCases, AuthRepository};
use crate::domain::use_cases::organization::{OrganizationUseCases, OrganizationRepository};
use crate::domain::use_cases::statistic::{StatisticUseCases, StatisticRepository};
use crate::domain::use_cases::vehicle::{VehicleUseCases, VehicleRepository};
use crate::infrastructure::repositories::driver_repository::PgDriverRepository;
use crate::infrastructure::repositories::department_repository::DepartmentRepositoryImpl;
use crate::infrastructure::repositories::auth_repository::AuthRepositoryImpl;
use crate::infrastructure::repositories::organization_repository::OrganizationRepositoryImpl;
use crate::infrastructure::repositories::statistic_repository::StatisticRepositoryImpl;
use crate::infrastructure::repositories::vehicle_repository::PgVehicleRepository;
use crate::infrastructure::repositories::location_repository::PgLocationRepository;
use crate::application::services::location_service::LocationServiceImpl;
use crate::infrastructure::repositories::location_repository::LocationRepository;
use crate::application::services::alert_service::AlertApplicationService;
use crate::domain::use_cases::user::{UserUseCases, UserRepository};
use crate::infrastructure::repositories::user_repository::PgUserRepository;
use crate::application::services::user_service::UserServiceImpl;
use crate::domain::use_cases::device::{DeviceUseCases, DeviceRepository};
use crate::infrastructure::repositories::device_repository::PgDeviceRepository;
use crate::application::services::device_service::DeviceServiceImpl;
use crate::domain::use_cases::sync::SyncUseCases;
use crate::domain::repositories::sync::SyncRepository;
use crate::domain::repositories::SqlxSyncRepository;
use crate::application::services::role_service::RoleApplicationService;
use crate::domain::use_cases::vehicle_group::VehicleGroupUseCases;
use crate::domain::use_cases::vehicle_group::VehicleGroupRepository;
use crate::infrastructure::repositories::vehicle_group_repository::PgVehicleGroupRepository;
use crate::domain::use_cases::openapi_platform::{OpenapiPlatformUseCases, OpenapiPlatformRepository};
use crate::infrastructure::repositories::openapi_platform_repository::OpenapiPlatformRepositoryImpl;
use crate::application::services::settings_service::SettingsApplicationService;

/// 服务器运行时状态
#[derive(Clone)]
pub struct ServerState {
    pub pool: sqlx::PgPool,
    pub config: UnifiedConfig,
    pub datasource_manager: Arc<crate::bff::datasources::DataSourceManager>,
    pub vehicle_aggregator: Arc<crate::bff::services::VehicleAggregator>,
    pub report_service: Arc<crate::bff::reports::ReportService>,
    pub template_engine: Arc<crate::bff::templates::ReportTemplateEngine>,
    pub video_service: Arc<crate::video::VideoService>,
    pub ai_service_state: Arc<crate::ai::routes::AiServiceState>,
    pub ws_app_state: Arc<crate::gateway::websocket_server::WsAppState>,
    pub driver_service: Arc<DriverUseCases>,
    pub department_service: Arc<DepartmentUseCases>,
    pub auth_service: Arc<AuthUseCases>,
    pub organization_service: Arc<OrganizationUseCases>,
    pub statistic_service: Arc<StatisticUseCases>,
    pub vehicle_service: Arc<VehicleUseCases>,
    pub location_service: Arc<LocationServiceImpl>,
    pub alert_service: Arc<AlertApplicationService>,
    pub user_service: Arc<UserServiceImpl>,
    pub device_service: Arc<DeviceServiceImpl>,
    pub sync_service: Arc<SyncUseCases>,
    pub order_service: Arc<crate::application::services::order_service::OrderApplicationService>,
    pub finance_service: Arc<dyn crate::application::services::finance_service::FinanceService>,
    pub role_service: Arc<RoleApplicationService>,
    pub vehicle_group_service: Arc<VehicleGroupUseCases>,
    pub openapi_platform_service: Arc<crate::domain::use_cases::openapi_platform::OpenapiPlatformUseCases>,
    pub system_monitor_service: Arc<crate::application::services::system_monitor_service::SystemMonitorService>,
    pub audit_log_service: Arc<crate::application::services::audit_log_service::AuditLogService>,
    pub settings_service: Arc<SettingsApplicationService>,
}

/// 执行所有服务初始化
pub async fn finalize_setup(
    app_config: &AppConfig,
    redis_client: &Arc<RedisClient>,
) -> Result<ServerState, Box<dyn std::error::Error>> {
    let pool = (*app_config.pool).clone();
    let config = app_config.config.clone();

    // 检查数据库连接
    info!("Checking database connection...");
    match sqlx::query("SELECT 1").execute(&pool).await {
        Ok(_) => info!("Database connection successful"),
        Err(e) => warn!(
            "Database connection failed: {:?}, continuing without database",
            e
        ),
    }

    // 启动所有服务
    crate::init::start_all_services(app_config, redis_client).await;

    // TODO: 微服务初始化 - 需要重新实现
    // if let Err(e) = crate::microservices::initialize_microservices().await {
    //     warn!("Failed to initialize microservices: {}", e);
    // } else {
    //     info!("Microservices initialized successfully");
    // }

    // 启动 Client API 服务器 (Port 9808)
    start_client_api_server();

    // 初始化 Redis(供 BFF 缓存层使用)
    if let Err(e) = crate::redis::init_redis().await {
        warn!(
            "Redis initialization failed, BFF cache will be disabled: {}",
            e
        );
    } else {
        info!("Redis connection pool initialized for BFF cache layer");
    }

    // 初始化BFF层数据源管理器
    let datasource_manager = Arc::new(crate::bff::datasources::DataSourceManager::new(
        Arc::new(pool.clone()),
        None,
        true,
    ));

    // 初始化BFF车辆聚合服务
    let vehicle_aggregator = Arc::new(crate::bff::services::VehicleAggregator::new(
        datasource_manager.clone(),
        Arc::new(pool.clone()),
        None,
    ));

    // 初始化BFF报表服务
    let report_service = Arc::new(crate::bff::reports::ReportService::new(Arc::new(
        pool.clone(),
    )));

    // 初始化BFF报表模板引擎
    let template_engine = match crate::bff::templates::ReportTemplateEngine::new("") {
        Ok(engine) => Arc::new(engine),
        Err(e) => return Err(format!("Failed to initialize template engine: {}", e).into()),
    };

    // 初始化视频服务
    let video_service = crate::init::services::start_video_service().await;

    // 初始化内存监控服务（仅在生产环境）
    if !cfg!(debug_assertions) {
        init_memory_monitor();
    }

    // 初始化资源告警服务（仅在生产环境）
    if !cfg!(debug_assertions) {
        let _ = init_resource_alert().await;
    }

    // 初始化WebSocket连接管理器
    let ws_manager = Arc::new(crate::gateway::websocket_manager::WebSocketManager::new(
        crate::gateway::websocket_manager::WebSocketManagerConfig::default(),
    ));
    ws_manager.start();

    // 初始化WebSocket应用状态
    let ws_app_state = Arc::new(crate::gateway::websocket_server::WsAppState::new());

    // 初始化AI服务（仅在需要时）
    let ai_service_state = if std::env::var("AI_SERVICE_ENABLED")
        .ok()
        .and_then(|s| s.parse::<bool>().ok())
        .unwrap_or(false) {
        init_ai_service()
    } else {
        // 创建一个空的AI服务状态
        let config = crate::ai::deepseek::DeepSeekConfig::default();
        let client = crate::ai::deepseek::DeepSeekClient::new(config.clone());
        Arc::new(crate::ai::routes::AiServiceState {
            coder_service: crate::ai::coder::DeepSeekCoderService::new(client.clone()),
            v3_service: crate::ai::v3::DeepSeekV3Service::new(client),
        })
    };

    // 初始化司机服务
    let driver_repository: Arc<dyn DriverRepository + Send + Sync> = Arc::new(PgDriverRepository::new(Arc::new(pool.clone())));
    let driver_service = Arc::new(DriverUseCases::new(driver_repository));

    // 初始化部门服务
    let department_repository: Arc<dyn DepartmentRepository + Send + Sync> = Arc::new(DepartmentRepositoryImpl::new(Arc::new(pool.clone())));
    let department_service = Arc::new(DepartmentUseCases::new(department_repository));

    // 初始化认证服务
    let auth_repository: Arc<dyn AuthRepository + Send + Sync> = Arc::new(AuthRepositoryImpl::new(Arc::new(pool.clone())));
    let auth_service = Arc::new(AuthUseCases::new(auth_repository));

    // 初始化组织单位服务
    let organization_repository: Arc<dyn OrganizationRepository + Send + Sync> = Arc::new(OrganizationRepositoryImpl::new(Arc::new(pool.clone())));
    let organization_service = Arc::new(OrganizationUseCases::new(organization_repository));

    // 初始化统计分析服务
    let statistic_repository: Arc<dyn StatisticRepository + Send + Sync> = Arc::new(StatisticRepositoryImpl::new(Arc::new(pool.clone())));
    let statistic_service = Arc::new(StatisticUseCases::new(statistic_repository));

    // 初始化车辆服务
    let vehicle_repository: Arc<dyn VehicleRepository + Send + Sync> = Arc::new(PgVehicleRepository::new(Arc::new(pool.clone())));
    let vehicle_service = Arc::new(VehicleUseCases::new(vehicle_repository));

    // 初始化位置服务
    let location_repository: Arc<dyn LocationRepository + Send + Sync> = Arc::new(PgLocationRepository::new(Arc::new(pool.clone())));
    let location_service = Arc::new(LocationServiceImpl::new(location_repository));

    // 初始化报警服务
    let alert_service = Arc::new(AlertApplicationService::new(pool.clone()));

    // 初始化用户服务
    let user_repository: Arc<dyn UserRepository + Send + Sync> = Arc::new(PgUserRepository::new(Arc::new(pool.clone())));
    let user_use_cases = Arc::new(UserUseCases::new(user_repository));
    let user_service = Arc::new(UserServiceImpl::new(user_use_cases));

    // 初始化设备服务
    let device_repository: Arc<dyn DeviceRepository + Send + Sync> = Arc::new(PgDeviceRepository::new(Arc::new(pool.clone())));
    let device_use_cases = Arc::new(DeviceUseCases::new(device_repository));
    let device_service = Arc::new(DeviceServiceImpl::new(device_use_cases));

    // 初始化同步服务
    let sync_repository: Arc<dyn SyncRepository + Send + Sync> = Arc::new(SqlxSyncRepository::new(pool.clone()));
    let sync_service = Arc::new(SyncUseCases::new(sync_repository));

    // 初始化订单服务
    let order_service = Arc::new(crate::application::services::order_service::OrderApplicationService::new(pool.clone()));

    // 初始化财务管理服务
    let finance_repository: Arc<dyn crate::domain::use_cases::finance::FinanceRepository + Send + Sync> = Arc::new(crate::infrastructure::repositories::finance_repository::FinanceRepositoryImpl::new(Arc::new(pool.clone())));
    let finance_service: Arc<dyn crate::application::services::finance_service::FinanceService> = Arc::new(crate::application::services::finance_service::FinanceServiceImpl::new(finance_repository));

    // 初始化角色服务
    let role_service = Arc::new(RoleApplicationService::new(pool.clone()));

    // 初始化车队服务
    let vehicle_group_repository: Arc<dyn VehicleGroupRepository + Send + Sync> = Arc::new(PgVehicleGroupRepository::new(Arc::new(pool.clone())));
    let vehicle_group_service = Arc::new(VehicleGroupUseCases::new(vehicle_group_repository));

    // 初始化 OpenAPI 平台服务
    let openapi_platform_repository: Arc<dyn OpenapiPlatformRepository + Send + Sync> = Arc::new(OpenapiPlatformRepositoryImpl::new(Arc::new(pool.clone())));
    let openapi_platform_service = Arc::new(OpenapiPlatformUseCases::new(openapi_platform_repository));

    // 初始化系统监控服务
    let system_monitor_config = crate::application::services::system_monitor_service::SystemMonitorConfig {
        check_interval: 10,
        max_processes: 100,
        enable_details: true,
    };
    let system_monitor_service = Arc::new(crate::application::services::system_monitor_service::SystemMonitorService::new(system_monitor_config));
    system_monitor_service.start();

    // 初始化审计日志服务
    let audit_log_repository = crate::infrastructure::repositories::audit_log_repository::AuditLogRepository::new(Arc::new(pool.clone()));
    let audit_log_service = Arc::new(crate::application::services::audit_log_service::AuditLogService::new(audit_log_repository));

    // 初始化设置服务
    let settings_service = Arc::new(SettingsApplicationService::new(pool.clone()));

    info!("All services initialized successfully");

    Ok(ServerState {
        pool,
        config,
        datasource_manager,
        vehicle_aggregator,
        report_service,
        template_engine,
        video_service,
        ai_service_state,
        ws_app_state,
        driver_service,
        department_service,
        auth_service,
        organization_service,
        statistic_service,
        vehicle_service,
        location_service,
        alert_service,
        user_service,
        device_service,
        sync_service,
        order_service,
        finance_service,
        role_service,
        vehicle_group_service,
        openapi_platform_service,
        system_monitor_service,
        audit_log_service,
        settings_service,
    })
}

/// 启动 Client API 服务器
fn start_client_api_server() {
    info!("Starting Client API server on 0.0.0.0:9808...");
    std::thread::spawn(|| {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                let addr: std::net::SocketAddr = "0.0.0.0:9808"
                    .parse()
                    .expect("Failed to parse client API server address");
                let mut server = crate::gateway::client_api_server::ClientApiServer::new(addr, 1000);
                if let Err(e) = server.start().await {
                    log::error!("Client API server failed to start: {:?}", e);
                    return;
                }
                server.run().await;
            });
    });
    info!("Client API server started successfully on 0.0.0.0:9808");
}

/// 初始化内存监控服务
fn init_memory_monitor() {
    use crate::performance::{MemoryLimitService, MemoryManager, MemoryMonitorConfig};

    // 获取系统总内存,设置为 80% 作为内存限制阈值
    let sys = sysinfo::System::new_all();
    let total_memory = sys.total_memory();
    // 使用系统内存的 80% 作为限制
    let memory_limit = total_memory * 80 / 100;
    let memory_manager = Arc::new(MemoryManager::new(memory_limit as usize));
    let config = MemoryMonitorConfig {
        memory_threshold: 80.0,
        check_interval: Duration::from_secs(30),
        memory_limit: Some(memory_limit),
        enable_memory_limit: true,
    };
    let service = Arc::new(MemoryLimitService::new(config, Some(memory_manager)));
    service.start();
    info!(
        "Memory monitoring service started (system total: {} GB, limit: {:.1} GB)",
        total_memory as f64 / (1024.0 * 1024.0 * 1024.0),
        memory_limit as f64 / (1024.0 * 1024.0 * 1024.0)
    );
}

/// 初始化资源告警服务
async fn init_resource_alert() {
    use crate::performance::{ResourceAlertConfig, ResourceAlertService};

    let service = Arc::new(ResourceAlertService::new(ResourceAlertConfig::default()));
    service.start().await;
    info!("Resource alert service started");
}

/// 初始化 AI 服务
fn init_ai_service() -> Arc<crate::ai::routes::AiServiceState> {
    use crate::ai::{coder::DeepSeekCoderService, deepseek::DeepSeekClient, v3::DeepSeekV3Service};

    let config = crate::ai::deepseek::DeepSeekConfig::default();
    let client = DeepSeekClient::new(config.clone());
    let coder_service = DeepSeekCoderService::new(client);

    let v3_config = crate::ai::deepseek::DeepSeekConfig {
        model: "deepseek-v3:7b".to_string(),
        ..config
    };
    let v3_client = DeepSeekClient::new(v3_config);
    let v3_service = DeepSeekV3Service::new(v3_client);

    Arc::new(crate::ai::routes::AiServiceState {
        coder_service,
        v3_service,
    })
}
