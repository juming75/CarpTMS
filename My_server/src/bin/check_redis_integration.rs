use chrono::Utc;
use tms_server::cache::vehicle_cache;
use tms_server::models::VehicleRealtimeLocation;
use tms_server::redis;
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 设置日志
    tracing_subscriber::fmt::init();

    info!("Checking Redis integration progress...");

    // 检查Redis初始化
    info!("Testing Redis connection...");

    // 尝试初始化Redis
    match redis::init_redis().await {
        Ok(_) => info!("✅ Redis connection initialized successfully"),
        Err(e) => error!("❌ Failed to initialize Redis: {}", e),
    }

    // 检查Redis是否可用
    let is_available = redis::is_redis_available().await;
    if is_available {
        info!("✅ Redis is available");
    } else {
        error!("❌ Redis is not available");
    }

    // 检查车辆缓存功能
    info!("Testing vehicle cache functionality...");
    test_vehicle_cache().await?;

    info!("Redis integration check completed!");
    Ok(())
}

// 测试车辆缓存功能
async fn test_vehicle_cache() -> Result<(), Box<dyn std::error::Error>> {
    // 只有当Redis可用时才测试缓存功能
    if !redis::is_redis_available().await {
        info!("⚠️ Redis not available, skipping cache functionality test");
        return Ok(());
    }

    // 创建测试数据
    let vehicle_id = 1;
    let location = VehicleRealtimeLocation {
        id: 0,
        vehicle_id: 1,
        latitude: 39.9042,
        longitude: 116.4074,
        speed: 60.5,
        direction: 90,
        altitude: 43.0,
        accuracy: Some(10.0),
        status: 1,
        timestamp: Utc::now(),
        update_time: Utc::now(),
        created_at: Utc::now(),
    };

    // 测试设置缓存
    let cache = vehicle_cache::VehicleCache::default();
    match cache
        .set_vehicle_realtime_location(vehicle_id, &location)
        .await
    {
        Ok(_) => info!("✅ Vehicle realtime location cache set successfully"),
        Err(e) => error!("❌ Failed to set vehicle realtime location cache: {}", e),
    }

    // 测试获取缓存
    match cache.get_vehicle_realtime_location(vehicle_id).await {
        Ok(Some(cached_location)) => {
            info!("✅ Vehicle realtime location cache retrieved successfully");
            info!(
                "  Cached location: vehicle_id={}, lon={}, lat={}",
                cached_location.vehicle_id, cached_location.longitude, cached_location.latitude
            );
        }
        Ok(None) => error!("❌ Vehicle realtime location cache not found"),
        Err(e) => error!("❌ Failed to get vehicle realtime location cache: {}", e),
    }

    Ok(())
}
