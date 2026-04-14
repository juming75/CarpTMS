//! / 监控指标模块
use once_cell::sync::Lazy;
use prometheus::{
    register_gauge, register_histogram_vec, Gauge, GaugeVec, HistogramVec, IntCounterVec,
};

// API 请求计数器
pub static API_REQUESTS_TOTAL: Lazy<IntCounterVec> = Lazy::new(|| {
    prometheus::register_int_counter_vec!(
        "api_requests_total",
        "Total number of API requests",
        &["method", "endpoint", "status"]
    )
    .unwrap()
});

// API 请求延迟直方图
pub static API_REQUEST_DURATION: Lazy<HistogramVec> = Lazy::new(|| {
    register_histogram_vec!(
        "api_request_duration_seconds",
        "API request duration in seconds",
        &["method", "endpoint", "status"],
        vec![0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0]
    )
    .unwrap()
});

// 速率限制指标
pub static RATE_LIMIT_REQUESTS_TOTAL: Lazy<IntCounterVec> = Lazy::new(|| {
    prometheus::register_int_counter_vec!(
        "rate_limit_requests_total",
        "Total number of requests subject to rate limiting",
        &["key_type", "endpoint"]
    )
    .unwrap()
});

pub static RATE_LIMIT_REJECTIONS_TOTAL: Lazy<IntCounterVec> = Lazy::new(|| {
    prometheus::register_int_counter_vec!(
        "rate_limit_rejections_total",
        "Total number of requests rejected due to rate limiting",
        &["key_type", "endpoint"]
    )
    .unwrap()
});

pub static RATE_LIMIT_TOKENS_REMAINING: Lazy<GaugeVec> = Lazy::new(|| {
    prometheus::register_gauge_vec!(
        "rate_limit_tokens_remaining",
        "Number of tokens remaining in the bucket",
        &["key_type"]
    )
    .unwrap()
});

pub static RATE_LIMIT_CURRENT_USERS: Lazy<Gauge> = Lazy::new(|| {
    register_gauge!(
        "rate_limit_current_users",
        "Current number of unique keys being rate limited"
    )
    .unwrap()
});

// 数据库连接池状态指标
pub static DB_CONNECTIONS_TOTAL: Lazy<Gauge> = Lazy::new(|| {
    register_gauge!(
        "db_connections_total",
        "Total number of database connections"
    )
    .unwrap()
});

pub static DB_CONNECTIONS_IN_USE: Lazy<Gauge> = Lazy::new(|| {
    register_gauge!(
        "db_connections_in_use",
        "Number of database connections currently in use"
    )
    .unwrap()
});

pub static DB_CONNECTIONS_IDLE: Lazy<Gauge> = Lazy::new(|| {
    register_gauge!("db_connections_idle", "Number of idle database connections").unwrap()
});

// 业务指标
pub static ORDERS_TOTAL: Lazy<Gauge> =
    Lazy::new(|| register_gauge!("orders_total", "Total number of orders created").unwrap());

pub static ORDERS_PENDING: Lazy<Gauge> =
    Lazy::new(|| register_gauge!("orders_pending", "Number of pending orders").unwrap());

pub static ORDERS_IN_TRANSIT: Lazy<Gauge> =
    Lazy::new(|| register_gauge!("orders_in_transit", "Number of orders in transit").unwrap());

pub static ORDERS_COMPLETED: Lazy<Gauge> =
    Lazy::new(|| register_gauge!("orders_completed", "Number of completed orders").unwrap());

pub static VEHICLES_ONLINE: Lazy<Gauge> =
    Lazy::new(|| register_gauge!("vehicles_online", "Number of online vehicles").unwrap());

pub static VEHICLES_TOTAL: Lazy<Gauge> =
    Lazy::new(|| register_gauge!("vehicles_total", "Total number of vehicles").unwrap());

// 设备状态指标
pub static DEVICES_TOTAL: Lazy<Gauge> =
    Lazy::new(|| register_gauge!("devices_total", "Total number of devices").unwrap());

pub static DEVICES_ONLINE: Lazy<Gauge> =
    Lazy::new(|| register_gauge!("devices_online", "Number of online devices").unwrap());

pub static DEVICES_OFFLINE: Lazy<Gauge> =
    Lazy::new(|| register_gauge!("devices_offline", "Number of offline devices").unwrap());

// 称重数据指标
pub static WEIGHING_DATA_TOTAL: Lazy<Gauge> = Lazy::new(|| {
    register_gauge!(
        "weighing_data_total",
        "Total number of weighing data records"
    )
    .unwrap()
});

pub static WEIGHING_DATA_DAILY: Lazy<Gauge> = Lazy::new(|| {
    register_gauge!(
        "weighing_data_daily",
        "Number of weighing data records in the last 24 hours"
    )
    .unwrap()
});

// JWT令牌指标
pub static JWT_TOKENS_GENERATED: Lazy<IntCounterVec> = Lazy::new(|| {
    prometheus::register_int_counter_vec!(
        "jwt_tokens_generated_total",
        "Total number of JWT tokens generated",
        &["token_type"]
    )
    .unwrap()
});

pub static JWT_TOKENS_VALIDATED: Lazy<IntCounterVec> = Lazy::new(|| {
    prometheus::register_int_counter_vec!(
        "jwt_tokens_validated_total",
        "Total number of JWT tokens validated",
        &["result"]
    )
    .unwrap()
});

// API请求大小指标
pub static API_REQUEST_SIZE: Lazy<HistogramVec> = Lazy::new(|| {
    register_histogram_vec!(
        "api_request_size_bytes",
        "API request size in bytes",
        &["method", "endpoint"],
        vec![100.0, 500.0, 1000.0, 5000.0, 10000.0, 50000.0]
    )
    .unwrap()
});

pub static API_RESPONSE_SIZE: Lazy<HistogramVec> = Lazy::new(|| {
    register_histogram_vec!(
        "api_response_size_bytes",
        "API response size in bytes",
        &["method", "endpoint", "status"],
        vec![100.0, 500.0, 1000.0, 5000.0, 10000.0, 50000.0]
    )
    .unwrap()
});

// 数据库查询性能监控
pub static DB_QUERY_DURATION: Lazy<HistogramVec> = Lazy::new(|| {
    register_histogram_vec!(
        "db_query_duration_seconds",
        "Database query duration in seconds",
        &["query_type", "table"],
        vec![0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0]
    )
    .unwrap()
});

// WebSocket连接指标
pub static WEBSOCKET_CONNECTIONS: Lazy<Gauge> = Lazy::new(|| {
    register_gauge!(
        "websocket_connections_total",
        "Total number of WebSocket connections"
    )
    .unwrap()
});

pub static WEBSOCKET_MESSAGES: Lazy<IntCounterVec> = Lazy::new(|| {
    prometheus::register_int_counter_vec!(
        "websocket_messages_total",
        "Total number of WebSocket messages",
        &["direction", "message_type"]
    )
    .unwrap()
});

// 缓存指标
pub static CACHE_HITS: Lazy<IntCounterVec> = Lazy::new(|| {
    prometheus::register_int_counter_vec!(
        "cache_hits_total",
        "Total number of cache hits",
        &["cache_type", "cache_key"]
    )
    .unwrap()
});

pub static CACHE_MISSES: Lazy<IntCounterVec> = Lazy::new(|| {
    prometheus::register_int_counter_vec!(
        "cache_misses_total",
        "Total number of cache misses",
        &["cache_type", "cache_key"]
    )
    .unwrap()
});

pub static CACHE_HIT_RATE: Lazy<Gauge> =
    Lazy::new(|| register_gauge!("cache_hit_rate", "Cache hit rate percentage").unwrap());

// 服务注册中心指标
pub static SERVICE_INSTANCES: Lazy<Gauge> = Lazy::new(|| {
    register_gauge!(
        "service_instances_total",
        "Total number of registered service instances"
    )
    .unwrap()
});

pub static SERVICE_INSTANCES_HEALTHY: Lazy<Gauge> = Lazy::new(|| {
    register_gauge!(
        "service_instances_healthy",
        "Number of healthy service instances"
    )
    .unwrap()
});

pub static SERVICE_INSTANCES_UNHEALTHY: Lazy<Gauge> = Lazy::new(|| {
    register_gauge!(
        "service_instances_unhealthy",
        "Number of unhealthy service instances"
    )
    .unwrap()
});

// 扩缩容指标
pub static SCALING_EVENTS: Lazy<IntCounterVec> = Lazy::new(|| {
    prometheus::register_int_counter_vec!(
        "scaling_events_total",
        "Total number of scaling events",
        &["event_type", "service_name"]
    )
    .unwrap()
});

// 系统资源指标
pub static CPU_USAGE: Lazy<Gauge> =
    Lazy::new(|| register_gauge!("cpu_usage_percentage", "CPU usage percentage").unwrap());

pub static MEMORY_USAGE: Lazy<Gauge> =
    Lazy::new(|| register_gauge!("memory_usage_percentage", "Memory usage percentage").unwrap());

pub static DISK_USAGE: Lazy<Gauge> =
    Lazy::new(|| register_gauge!("disk_usage_percentage", "Disk usage percentage").unwrap());

// 消息队列指标
pub static MESSAGE_QUEUE_SIZE: Lazy<Gauge> =
    Lazy::new(|| register_gauge!("message_queue_size", "Number of messages in queue").unwrap());

pub static MESSAGE_PROCESSING_TIME: Lazy<HistogramVec> = Lazy::new(|| {
    register_histogram_vec!(
        "message_processing_time_seconds",
        "Message processing time in seconds",
        &["queue_name", "message_type"],
        vec![0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0]
    )
    .unwrap()
});

// Redis缓存性能指标
pub static REDIS_OPERATION_DURATION: Lazy<HistogramVec> = Lazy::new(|| {
    register_histogram_vec!(
        "redis_operation_duration_seconds",
        "Redis operation duration in seconds",
        &["operation_type", "result"],
        vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5]
    )
    .unwrap()
});

// 电路breaker状态指标
pub static CIRCUIT_BREAKER_STATE: Lazy<IntCounterVec> = Lazy::new(|| {
    prometheus::register_int_counter_vec!(
        "circuit_breaker_state_transitions_total",
        "Total number of circuit breaker state transitions",
        &["breaker_name", "from_state", "to_state"]
    )
    .unwrap()
});

pub static CIRCUIT_BREAKER_FAILURES: Lazy<IntCounterVec> = Lazy::new(|| {
    prometheus::register_int_counter_vec!(
        "circuit_breaker_failures_total",
        "Total number of circuit breaker failures",
        &["breaker_name"]
    )
    .unwrap()
});

// 并发请求数指标
pub static CONCURRENT_REQUESTS: Lazy<Gauge> =
    Lazy::new(|| register_gauge!("concurrent_requests", "Number of concurrent requests").unwrap());

// 错误率指标
pub static ERROR_RATE: Lazy<Gauge> = Lazy::new(|| {
    register_gauge!(
        "error_rate_percentage",
        "Percentage of requests resulting in errors"
    )
    .unwrap()
});

pub static ERROR_COUNT: Lazy<IntCounterVec> = Lazy::new(|| {
    prometheus::register_int_counter_vec!(
        "error_count_total",
        "Total number of errors",
        &["error_type", "component"]
    )
    .unwrap()
});

// 队列处理延迟指标
pub static QUEUE_PROCESSING_DELAY: Lazy<HistogramVec> = Lazy::new(|| {
    register_histogram_vec!(
        "queue_processing_delay_seconds",
        "Delay between message enqueue and processing",
        &["queue_name"],
        vec![0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0]
    )
    .unwrap()
});

// 详细的业务指标 - 车辆行驶里程
pub static VEHICLE_MILEAGE: Lazy<Gauge> = Lazy::new(|| {
    register_gauge!(
        "vehicle_total_mileage_km",
        "Total vehicle mileage in kilometers"
    )
    .unwrap()
});

// 详细的业务指标 - 活跃用户数
pub static ACTIVE_USERS: Lazy<Gauge> =
    Lazy::new(|| register_gauge!("active_users_total", "Number of active users").unwrap());

// 初始化监控指标
pub fn init_metrics() {
    // 注册所有指标
    prometheus::register_int_counter_vec!(
        "application_info",
        "Application information",
        &["version", "environment"]
    )
    .unwrap()
    .with_label_values(&["1.0.0", "development"])
    .inc();
}

// 监控数据库连接池状态
pub async fn monitor_db_pool(pool: &sqlx::postgres::PgPool) {
    use tokio::time::Duration;

    // 每秒更新一次数据库连接池状态
    let mut interval = tokio::time::interval(Duration::from_secs(1));

    loop {
        interval.tick().await;

        // 获取连接池状态
        let size = pool.size() as f64;
        let num_idle = pool.num_idle() as f64;
        let in_use = (size - num_idle).max(0.0);

        // 更新指标
        DB_CONNECTIONS_TOTAL.set(size);
        DB_CONNECTIONS_IDLE.set(num_idle);
        DB_CONNECTIONS_IN_USE.set(in_use);
    }
}

// 监控业务指标
pub async fn monitor_business_metrics(pool: &sqlx::postgres::PgPool) {
    use tokio::time::Duration;

    // 每5秒更新一次业务指标
    let mut interval = tokio::time::interval(Duration::from_secs(5));

    loop {
        interval.tick().await;

        // 更新订单状态指标
        update_order_metrics(pool).await;

        // 更新车辆状态指标
        update_vehicle_metrics(pool).await;

        // 更新设备状态指标
        update_device_metrics(pool).await;

        // 更新称重数据指标
        update_weighing_data_metrics(pool).await;

        // 更新服务注册中心指标
        update_service_registry_metrics().await;

        // 更新系统资源指标
        update_system_resources_metrics().await;

        // 更新缓存指标
        update_cache_metrics().await;

        // 更新活跃用户数指标
        update_active_users_metrics(pool).await;

        // 更新车辆里程指标
        update_vehicle_mileage_metrics(pool).await;

        // 更新错误率指标
        update_error_rate_metrics().await;
    }
}

// 更新订单状态指标
async fn update_order_metrics(pool: &sqlx::postgres::PgPool) {
    // 获取总订单数
    let total_orders: i32 = sqlx::query_scalar("SELECT COUNT(*) FROM orders")
        .fetch_one(pool)
        .await
        .unwrap_or(0);

    // 获取待分配订单数
    let pending_orders: i32 =
        sqlx::query_scalar("SELECT COUNT(*) FROM orders WHERE order_status = 1")
            .fetch_one(pool)
            .await
            .unwrap_or(0);

    // 获取运输中订单数
    let in_transit_orders: i32 =
        sqlx::query_scalar("SELECT COUNT(*) FROM orders WHERE order_status = 2")
            .fetch_one(pool)
            .await
            .unwrap_or(0);

    // 获取已完成订单数
    let completed_orders: i32 =
        sqlx::query_scalar("SELECT COUNT(*) FROM orders WHERE order_status = 3")
            .fetch_one(pool)
            .await
            .unwrap_or(0);

    // 更新指标
    ORDERS_TOTAL.set(total_orders as f64);
    ORDERS_PENDING.set(pending_orders as f64);
    ORDERS_IN_TRANSIT.set(in_transit_orders as f64);
    ORDERS_COMPLETED.set(completed_orders as f64);
}

// 更新车辆状态指标
async fn update_vehicle_metrics(pool: &sqlx::postgres::PgPool) {
    // 获取总车辆数
    let total_vehicles: i32 = sqlx::query_scalar("SELECT COUNT(*) FROM vehicles")
        .fetch_one(pool)
        .await
        .unwrap_or(0);

    // 获取在线车辆数(假设is_active字段表示在线状态)
    let online_vehicles: i32 =
        sqlx::query_scalar("SELECT COUNT(*) FROM vehicles WHERE is_active = true")
            .fetch_one(pool)
            .await
            .unwrap_or(0);

    // 更新指标
    VEHICLES_TOTAL.set(total_vehicles as f64);
    VEHICLES_ONLINE.set(online_vehicles as f64);
}

// 更新设备状态指标
async fn update_device_metrics(pool: &sqlx::postgres::PgPool) {
    // 获取总设备数
    let total_devices: i32 = sqlx::query_scalar("SELECT COUNT(*) FROM devices")
        .fetch_one(pool)
        .await
        .unwrap_or(0);

    // 获取在线设备数(假设status = 1表示在线)
    let online_devices: i32 = sqlx::query_scalar("SELECT COUNT(*) FROM devices WHERE status = 1")
        .fetch_one(pool)
        .await
        .unwrap_or(0);

    // 计算离线设备数
    let offline_devices = total_devices - online_devices;

    // 更新指标
    DEVICES_TOTAL.set(total_devices as f64);
    DEVICES_ONLINE.set(online_devices as f64);
    DEVICES_OFFLINE.set(offline_devices as f64);
}

// 更新称重数据指标
async fn update_weighing_data_metrics(pool: &sqlx::postgres::PgPool) {
    // 获取总称重数据记录数
    let total_weighing_data: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM weighing_data")
        .fetch_one(pool)
        .await
        .unwrap_or(0);

    // 获取最近24小时的称重数据记录数
    let daily_weighing_data: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM weighing_data WHERE weighing_time > NOW() - INTERVAL '24 HOUR'",
    )
    .fetch_one(pool)
    .await
    .unwrap_or(0);

    // 更新指标
    WEIGHING_DATA_TOTAL.set(total_weighing_data as f64);
    WEIGHING_DATA_DAILY.set(daily_weighing_data as f64);
}

// 更新服务注册中心指标
// TODO: 当实现服务注册中心后启用此函数
async fn update_service_registry_metrics() {
    // use crate::load_balancing::SERVICE_REGISTRY;

    // // 获取服务实例信息
    // let instances = SERVICE_REGISTRY.get_all_instances();
    // let total_instances = instances.len() as f64;

    // // 计算健康实例数
    // let healthy_instances = instances.iter()
    //     .filter(|instance| instance.health_status == "healthy")
    //     .count() as f64;

    // let unhealthy_instances = total_instances - healthy_instances;

    // // 更新指标
    // SERVICE_INSTANCES.set(total_instances);
    // SERVICE_INSTANCES_HEALTHY.set(healthy_instances);
    // SERVICE_INSTANCES_UNHEALTHY.set(unhealthy_instances);
}

// 更新系统资源指标
async fn update_system_resources_metrics() {
    // 简化实现:使用默认值,实际项目中可以添加sysinfo依赖来获取真实系统资源
    // CPU使用率默认值
    CPU_USAGE.set(0.0);
    // 内存使用率默认值
    MEMORY_USAGE.set(0.0);
    // 磁盘使用率默认值
    DISK_USAGE.set(0.0);
}

// 更新缓存指标
async fn update_cache_metrics() {
    // 获取缓存命中和未命中数
    let total_hits = CACHE_HITS.with_label_values(&["", ""]).get() as f64;
    let total_misses = CACHE_MISSES.with_label_values(&["", ""]).get() as f64;

    // 计算命中率
    let hit_rate = if total_hits + total_misses > 0.0 {
        (total_hits / (total_hits + total_misses)) * 100.0
    } else {
        0.0
    };

    // 更新指标
    CACHE_HIT_RATE.set(hit_rate);
}

// 更新活跃用户数指标
async fn update_active_users_metrics(pool: &sqlx::postgres::PgPool) {
    // 获取最近30分钟内活跃的用户数
    let active_users: i32 = sqlx::query_scalar(
        "SELECT COUNT(DISTINCT user_id) FROM audit_logs WHERE action_time > NOW() - INTERVAL '30 MINUTE'"
    )
    .fetch_one(pool)
    .await.unwrap_or(0);

    // 更新指标
    ACTIVE_USERS.set(active_users as f64);
}

// 更新车辆里程指标
async fn update_vehicle_mileage_metrics(pool: &sqlx::postgres::PgPool) {
    // 获取所有车辆的总行驶里程
    let total_mileage: f64 =
        sqlx::query_scalar("SELECT COALESCE(SUM(distance), 0) FROM logistics_track_lines")
            .fetch_one(pool)
            .await
            .unwrap_or(0.0);

    // 更新指标
    VEHICLE_MILEAGE.set(total_mileage);
}

// 更新错误率指标
async fn update_error_rate_metrics() {
    // 获取最近60秒内的请求总数
    let total_requests = API_REQUESTS_TOTAL.with_label_values(&["", "", ""]).get() as f64;

    // 获取最近60秒内的错误请求数
    let error_requests = API_REQUESTS_TOTAL.with_label_values(&["", "", "500"]).get() as f64;

    // 计算错误率
    let error_rate = if total_requests > 0.0 {
        (error_requests / total_requests) * 100.0
    } else {
        0.0
    };

    // 更新指标
    ERROR_RATE.set(error_rate);
}
