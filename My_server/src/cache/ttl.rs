use std::time::Duration;

// 缓存过期时间配置
pub struct CacheTtlConfig {
    // 用户相关缓存
    pub user_profile: Duration,
    pub user_session: Duration,
    pub user_permissions: Duration,

    // 车辆相关缓存
    pub vehicle_info: Duration,
    pub vehicle_location: Duration,
    pub vehicle_status: Duration,

    // 设备相关缓存
    pub device_info: Duration,
    pub device_status: Duration,

    // 系统相关缓存
    pub system_config: Duration,
    pub system_stats: Duration,

    // 业务相关缓存
    pub order_info: Duration,
    pub route_info: Duration,
    pub shipment_info: Duration,

    // 热点数据缓存
    pub hot_data: Duration,

    // 一般数据缓存
    pub default: Duration,
}

impl Default for CacheTtlConfig {
    fn default() -> Self {
        Self {
            // 用户相关缓存
            user_profile: Duration::from_secs(3600), // 1小时
            user_session: Duration::from_secs(7200), // 2小时
            user_permissions: Duration::from_secs(1800), // 30分钟

            // 车辆相关缓存
            vehicle_info: Duration::from_secs(1800), // 30分钟
            vehicle_location: Duration::from_secs(30), // 30秒
            vehicle_status: Duration::from_secs(60), // 1分钟

            // 设备相关缓存
            device_info: Duration::from_secs(1800), // 30分钟
            device_status: Duration::from_secs(60), // 1分钟

            // 系统相关缓存
            system_config: Duration::from_secs(7200), // 2小时
            system_stats: Duration::from_secs(300),   // 5分钟

            // 业务相关缓存
            order_info: Duration::from_secs(600),    // 10分钟
            route_info: Duration::from_secs(1800),   // 30分钟
            shipment_info: Duration::from_secs(600), // 10分钟

            // 热点数据缓存
            hot_data: Duration::from_secs(600), // 10分钟

            // 一般数据缓存
            default: Duration::from_secs(300), // 5分钟
        }
    }
}

impl CacheTtlConfig {
    // 根据数据类型获取TTL
    pub fn get_ttl(&self, data_type: CacheDataType) -> Duration {
        match data_type {
            CacheDataType::UserProfile => self.user_profile,
            CacheDataType::UserSession => self.user_session,
            CacheDataType::UserPermissions => self.user_permissions,
            CacheDataType::VehicleInfo => self.vehicle_info,
            CacheDataType::VehicleLocation => self.vehicle_location,
            CacheDataType::VehicleStatus => self.vehicle_status,
            CacheDataType::DeviceInfo => self.device_info,
            CacheDataType::DeviceStatus => self.device_status,
            CacheDataType::SystemConfig => self.system_config,
            CacheDataType::SystemStats => self.system_stats,
            CacheDataType::OrderInfo => self.order_info,
            CacheDataType::RouteInfo => self.route_info,
            CacheDataType::ShipmentInfo => self.shipment_info,
            CacheDataType::HotData => self.hot_data,
            CacheDataType::Default => self.default,
        }
    }
}

// 缓存数据类型
pub enum CacheDataType {
    // 用户相关
    UserProfile,
    UserSession,
    UserPermissions,

    // 车辆相关
    VehicleInfo,
    VehicleLocation,
    VehicleStatus,

    // 设备相关
    DeviceInfo,
    DeviceStatus,

    // 系统相关
    SystemConfig,
    SystemStats,

    // 业务相关
    OrderInfo,
    RouteInfo,
    ShipmentInfo,

    // 热点数据
    HotData,

    // 默认
    Default,
}

// 从环境变量加载TTL配置
pub fn load_ttl_config_from_env() -> CacheTtlConfig {
    let mut config = CacheTtlConfig::default();

    // 尝试从环境变量加载TTL设置
    if let Ok(ttl) = std::env::var("CACHE_TTL_USER_PROFILE") {
        if let Ok(secs) = ttl.parse::<u64>() {
            config.user_profile = Duration::from_secs(secs);
        }
    }

    if let Ok(ttl) = std::env::var("CACHE_TTL_USER_SESSION") {
        if let Ok(secs) = ttl.parse::<u64>() {
            config.user_session = Duration::from_secs(secs);
        }
    }

    if let Ok(ttl) = std::env::var("CACHE_TTL_VEHICLE_LOCATION") {
        if let Ok(secs) = ttl.parse::<u64>() {
            config.vehicle_location = Duration::from_secs(secs);
        }
    }

    if let Ok(ttl) = std::env::var("CACHE_TTL_VEHICLE_STATUS") {
        if let Ok(secs) = ttl.parse::<u64>() {
            config.vehicle_status = Duration::from_secs(secs);
        }
    }

    if let Ok(ttl) = std::env::var("CACHE_TTL_DEFAULT") {
        if let Ok(secs) = ttl.parse::<u64>() {
            config.default = Duration::from_secs(secs);
        }
    }

    config
}

// 全局TTL配置实例
lazy_static::lazy_static! {
    pub static ref GLOBAL_TTL_CONFIG: CacheTtlConfig = load_ttl_config_from_env();
}

// 获取默认TTL配置
pub fn get_default_ttl_config() -> &'static CacheTtlConfig {
    &GLOBAL_TTL_CONFIG
}

// 根据数据类型获取TTL
pub fn get_ttl_for_data(data_type: CacheDataType) -> Duration {
    GLOBAL_TTL_CONFIG.get_ttl(data_type)
}



