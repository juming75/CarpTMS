//! / 车辆缓存模块
// 提供车辆相关数据的高性能缓存

use crate::cache::{CacheKey, CacheManager};
use crate::models::{Vehicle, VehicleRealtimeLocation};
use anyhow::{anyhow, Result};
use futures::future::join_all;
use std::time::Duration;
use tracing::{debug, info, warn};

/// 批量操作配置
pub struct BatchConfig {
    /// 最大批量大小
    pub max_batch_size: usize,
    /// 并发限制
    pub max_concurrent: usize,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            max_batch_size: 1000,
            max_concurrent: 50,
        }
    }
}

/// 车辆缓存服务
pub struct VehicleCache {
    cache: CacheManager,
    config: BatchConfig,
}

impl VehicleCache {
    /// 创建新的车辆缓存实例
    pub fn new(cache: CacheManager) -> Self {
        Self {
            cache,
            config: BatchConfig::default(),
        }
    }

    /// 创建带配置的车辆缓存实例
    pub fn with_config(cache: CacheManager, config: BatchConfig) -> Self {
        Self { cache, config }
    }

    /// 从环境变量创建 (异步)
    pub async fn from_env() -> Result<Self> {
        Ok(Self {
            cache: CacheManager::from_env().await?,
            config: BatchConfig::default(),
        })
    }

    /// 设置车辆基本信息缓存 (TTL: 30分钟)
    pub async fn set_vehicle(&self, vehicle: &Vehicle) -> Result<()> {
        let key = CacheKey::vehicle(vehicle.vehicle_id.to_string()).to_string();
        let data = serde_json::to_vec(vehicle)?;
        self.cache
            .set(&key, &data, Some(Duration::from_secs(1800)))
            .await?;
        debug!("Cached vehicle info: {}", vehicle.vehicle_id);
        Ok(())
    }

    /// 获取车辆基本信息缓存
    pub async fn get_vehicle(&self, vehicle_id: i32) -> Result<Option<Vehicle>> {
        let key = CacheKey::vehicle(vehicle_id.to_string()).to_string();
        match self.cache.get(&key).await? {
            Some(data) => {
                let vehicle: Vehicle = serde_json::from_slice(&data)?;
                Ok(Some(vehicle))
            }
            None => Ok(None),
        }
    }

    /// 批量获取车辆信息 (并发优化)
    pub async fn get_vehicles(&self, vehicle_ids: &[i32]) -> Result<Vec<Option<Vehicle>>> {
        if vehicle_ids.is_empty() {
            return Ok(Vec::new());
        }

        if vehicle_ids.len() > self.config.max_batch_size {
            return Err(anyhow!(
                "Batch size too large: {} > {}",
                vehicle_ids.len(),
                self.config.max_batch_size
            ));
        }

        let futures: Vec<_> = vehicle_ids.iter().map(|&id| self.get_vehicle(id)).collect();
        let results = join_all(futures).await;
        results.into_iter().collect()
    }

    /// 批量获取车辆信息 (Redis MGET优化 - 更高性能)
    pub async fn get_vehicles_batch(&self, vehicle_ids: &[i32]) -> Result<Vec<Option<Vehicle>>> {
        if vehicle_ids.is_empty() {
            return Ok(Vec::new());
        }

        if vehicle_ids.len() > self.config.max_batch_size {
            return Err(anyhow!(
                "Batch size too large: {} > {}",
                vehicle_ids.len(),
                self.config.max_batch_size
            ));
        }

        let keys: Vec<String> = vehicle_ids
            .iter()
            .map(|id| CacheKey::vehicle(id.to_string()).to_string())
            .collect();

        let results = self
            .cache
            .mget(&keys.iter().map(|s| s.as_str()).collect::<Vec<_>>())
            .await?;
        let mut vehicles = Vec::with_capacity(results.len());
        for result in results {
            match result {
                Some(data) => match serde_json::from_slice::<Vehicle>(&data) {
                    Ok(v) => vehicles.push(Some(v)),
                    Err(_) => vehicles.push(None),
                },
                None => vehicles.push(None),
            }
        }
        Ok(vehicles)
    }

    /// 设置车辆实时位置缓存 (TTL: 30秒)
    pub async fn set_vehicle_realtime_location(
        &self,
        vehicle_id: i32,
        location: &VehicleRealtimeLocation,
    ) -> Result<()> {
        let key = CacheKey::vehicle(vehicle_id.to_string()).to_string() + ":location";
        let data = serde_json::to_vec(location)?;
        self.cache
            .set(&key, &data, Some(Duration::from_secs(30)))
            .await?;
        debug!(
            "Cached vehicle {} location: ({}, {})",
            vehicle_id, location.latitude, location.longitude
        );
        Ok(())
    }

    /// 获取车辆实时位置缓存
    pub async fn get_vehicle_realtime_location(
        &self,
        vehicle_id: i32,
    ) -> Result<Option<VehicleRealtimeLocation>> {
        let key = CacheKey::vehicle(vehicle_id.to_string()).to_string() + ":location";
        match self.cache.get(&key).await? {
            Some(data) => {
                let location: VehicleRealtimeLocation = serde_json::from_slice(&data)?;
                Ok(Some(location))
            }
            None => Ok(None),
        }
    }

    /// 批量获取车辆实时位置 (并发优化)
    pub async fn get_vehicle_locations(
        &self,
        vehicle_ids: &[i32],
    ) -> Result<Vec<Option<VehicleRealtimeLocation>>> {
        if vehicle_ids.is_empty() {
            return Ok(Vec::new());
        }

        if vehicle_ids.len() > self.config.max_batch_size {
            return Err(anyhow!(
                "Batch size too large: {} > {}",
                vehicle_ids.len(),
                self.config.max_batch_size
            ));
        }

        let futures: Vec<_> = vehicle_ids
            .iter()
            .map(|&id| self.get_vehicle_realtime_location(id))
            .collect();
        let results = join_all(futures).await;
        results.into_iter().collect()
    }

    /// 批量获取车辆实时位置 (Redis MGET优化 - 更高性能)
    pub async fn get_vehicle_locations_batch(
        &self,
        vehicle_ids: &[i32],
    ) -> Result<Vec<Option<VehicleRealtimeLocation>>> {
        if vehicle_ids.is_empty() {
            return Ok(Vec::new());
        }

        if vehicle_ids.len() > self.config.max_batch_size {
            return Err(anyhow!(
                "Batch size too large: {} > {}",
                vehicle_ids.len(),
                self.config.max_batch_size
            ));
        }

        let keys: Vec<String> = vehicle_ids
            .iter()
            .map(|id| format!("{}:location", CacheKey::vehicle(id.to_string())))
            .collect();

        let results = self
            .cache
            .mget(&keys.iter().map(|s| s.as_str()).collect::<Vec<_>>())
            .await?;
        let mut locations = Vec::with_capacity(results.len());
        for result in results {
            match result {
                Some(data) => match serde_json::from_slice::<VehicleRealtimeLocation>(&data) {
                    Ok(loc) => locations.push(Some(loc)),
                    Err(_) => locations.push(None),
                },
                None => locations.push(None),
            }
        }
        Ok(locations)
    }

    /// 设置GPS历史数据缓存 (TTL: 5分钟)
    pub async fn set_gps_history<T>(&self, vehicle_id: i32, history: Vec<T>) -> Result<()>
    where
        T: serde::Serialize,
    {
        if history.is_empty() {
            return Ok(());
        }

        let key = CacheKey::vehicle(vehicle_id.to_string()).to_string() + ":gps_history";
        let data = serde_json::to_vec(&history)?;
        self.cache
            .set(&key, &data, Some(Duration::from_secs(300)))
            .await?;
        debug!(
            "Cached {} GPS history records for vehicle {}",
            history.len(),
            vehicle_id
        );
        Ok(())
    }

    /// 删除车辆的所有缓存
    pub async fn delete_vehicle_cache(&self, vehicle_id: i32) -> Result<()> {
        let pattern = format!("vehicle:{}:*", vehicle_id);
        self.cache.del_pattern(&pattern).await?;
        info!("Deleted cache entries for vehicle {}", vehicle_id);
        Ok(())
    }

    /// 删除特定车辆信息的缓存
    pub async fn delete_vehicle_info(&self, vehicle_id: i32) -> Result<()> {
        let key = CacheKey::vehicle(vehicle_id.to_string());
        self.cache.del(&key.to_string()).await?;
        info!("Deleted vehicle info cache: {}", vehicle_id);
        Ok(())
    }

    /// 删除车辆位置缓存
    pub async fn delete_vehicle_location(&self, vehicle_id: i32) -> Result<()> {
        let key = CacheKey::vehicle(vehicle_id.to_string()).to_string() + ":location";
        self.cache.del(&key).await?;
        Ok(())
    }

    /// 预热车辆缓存 (加载常用数据到缓存)
    pub async fn preheat_vehicles(&self, pool: &sqlx::PgPool, vehicle_ids: &[i32]) -> Result<u32> {
        if vehicle_ids.is_empty() {
            return Ok(0);
        }

        let mut cached_count = 0;

        // 缓存车辆基本信息
        let ids_str = vehicle_ids
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<_>>()
            .join(",");
        let query = format!(
            "SELECT * FROM vehicles WHERE vehicle_id IN ({}) AND status = 1",
            ids_str
        );

        let vehicles: Vec<Vehicle> = sqlx::query_as(&query)
            .fetch_all(pool)
            .await
            .unwrap_or_default();

        for vehicle in vehicles {
            if let Err(e) = self.set_vehicle(&vehicle).await {
                warn!("Failed to cache vehicle {}: {}", vehicle.vehicle_id, e);
            } else {
                cached_count += 1;
            }
        }

        info!(
            "Preheated {} vehicle info entries out of {} requested",
            cached_count,
            vehicle_ids.len()
        );

        Ok(cached_count)
    }

    /// 清空所有车辆缓存 (危险操作!)
    pub async fn flush_all(&self) -> Result<()> {
        self.cache.flush_all().await?;
        warn!("Flushed all vehicle cache!");
        Ok(())
    }
}

/// 默认实现（无 Redis，仅内存缓存）
impl Default for VehicleCache {
    fn default() -> Self {
        Self {
            cache: CacheManager::default(),
            config: BatchConfig::default(),
        }
    }
}
