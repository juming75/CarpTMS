//! 缓存优化模块
//! 提供缓存预热、动态过期策略和缓存命中率统计等功能

use super::{CacheManager, CacheKey};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use crate::errors::AppResult;

/// 缓存统计信息
#[derive(Debug, Clone, Default)]
pub struct EnhancedCacheStats {
    /// 内存缓存大小
    pub memory_cache_size: usize,
    /// 缓存命中次数
    pub hit_count: u64,
    /// 缓存未命中次数
    pub miss_count: u64,
    /// 缓存命中率
    pub hit_rate: f64,
    /// 缓存操作总时间
    pub total_operation_time: u128,
    /// 平均操作时间
    pub avg_operation_time: f64,
}

/// 缓存访问记录
#[derive(Debug, Clone)]
pub struct CacheAccessRecord {
    /// 最后访问时间
    pub last_access: Instant,
    /// 访问次数
    pub access_count: u64,
    /// 访问频率（次/秒）
    pub access_frequency: f64,
}

/// 增强的缓存管理器
pub struct EnhancedCacheManager {
    /// 基础缓存管理器
    pub cache_manager: Arc<CacheManager>,
    /// 缓存访问记录
    access_records: Mutex<HashMap<String, CacheAccessRecord>>,
    /// 缓存统计信息
    stats: Mutex<EnhancedCacheStats>,
    /// 缓存预热数据
    warmup_data: Vec<(String, Vec<u8>, Option<Duration>)>,
}

impl EnhancedCacheManager {
    /// 创建新的增强缓存管理器
    pub fn new(cache_manager: Arc<CacheManager>) -> Self {
        Self {
            cache_manager,
            access_records: Mutex::new(HashMap::new()),
            stats: Mutex::new(EnhancedCacheStats::default()),
            warmup_data: Vec::new(),
        }
    }

    /// 添加缓存预热数据
    pub fn add_warmup_data(&mut self, key: String, value: Vec<u8>, ttl: Option<Duration>) {
        self.warmup_data.push((key, value, ttl));
    }

    /// 执行缓存预热
    pub async fn warmup(&self) -> AppResult<()> {
        for (key, value, ttl) in &self.warmup_data {
            self.cache_manager.set(key, value, *ttl).await?;
        }
        Ok(())
    }

    /// 获取缓存项（增强版）
    pub async fn get(&self, key: &str) -> AppResult<Option<Vec<u8>>> {
        let start_time = Instant::now();
        
        let result = self.cache_manager.get(key).await;
        
        let operation_time = start_time.elapsed().as_nanos();
        
        // 更新统计信息
        let mut stats = self.stats.lock().await;
        stats.total_operation_time += operation_time;
        
        match &result {
            Ok(Some(_)) => {
                stats.hit_count += 1;
            }
            Ok(None) => {
                stats.miss_count += 1;
            }
            Err(_) => {
                stats.miss_count += 1;
            }
        }
        
        stats.hit_rate = if stats.hit_count + stats.miss_count > 0 {
            stats.hit_count as f64 / (stats.hit_count + stats.miss_count) as f64
        } else {
            0.0
        };
        
        stats.avg_operation_time = if stats.hit_count + stats.miss_count > 0 {
            stats.total_operation_time as f64 / (stats.hit_count + stats.miss_count) as f64
        } else {
            0.0
        };
        
        // 更新访问记录
        if result.is_ok() {
            let mut access_records = self.access_records.lock().await;
            let now = Instant::now();
            
            if let Some(record) = access_records.get_mut(key) {
                record.last_access = now;
                record.access_count += 1;
                record.access_frequency = record.access_count as f64 / now.duration_since(record.last_access).as_secs_f64();
            } else {
                access_records.insert(key.to_string(), CacheAccessRecord {
                    last_access: now,
                    access_count: 1,
                    access_frequency: 1.0,
                });
            }
        }
        
        Ok(result?)
    }

    /// 设置缓存项（增强版）
    pub async fn set(&self, key: &str, value: &[u8], ttl: Option<Duration>) -> AppResult<()> {
        // 根据访问频率动态调整TTL
        let adjusted_ttl = self.adjust_ttl(key, ttl).await;
        
        Ok(self.cache_manager.set(key, value, adjusted_ttl).await?)
    }

    /// 调整TTL（根据访问频率）
    async fn adjust_ttl(&self, key: &str, ttl: Option<Duration>) -> Option<Duration> {
        let access_records = self.access_records.lock().await;
        
        if let Some(record) = access_records.get(key) {
            // 根据访问频率调整TTL
            // 访问频率越高，TTL越长
            let base_ttl = ttl.unwrap_or(Duration::from_secs(3600));
            let frequency = record.access_frequency;
            
            if frequency > 1.0 { // 每秒访问1次以上
                Some(base_ttl * 2)
            } else if frequency > 0.1 { // 每10秒访问1次以上
                Some(base_ttl)
            } else {
                Some(base_ttl / 2)
            }
        } else {
            ttl
        }
    }

    /// 获取缓存统计信息
    pub async fn get_stats(&self) -> EnhancedCacheStats {
        let mut stats = self.stats.lock().await.clone();
        stats.memory_cache_size = self.cache_manager.get_stats().memory_cache_size;
        stats
    }

    /// 清理过期的访问记录
    pub async fn cleanup_access_records(&self) {
        let mut access_records = self.access_records.lock().await;
        let now = Instant::now();
        
        access_records.retain(|_, record| {
            // 保留最近24小时内有访问的记录
            now.duration_since(record.last_access) < Duration::from_secs(24 * 3600)
        });
    }
}

/// 缓存预热器
pub struct CacheWarmer {
    enhanced_cache: Arc<Mutex<EnhancedCacheManager>>,
}

impl CacheWarmer {
    /// 创建新的缓存预热器
    pub fn new(enhanced_cache: Arc<Mutex<EnhancedCacheManager>>) -> Self {
        Self {
            enhanced_cache,
        }
    }

    /// 预热用户缓存
    pub async fn warmup_users(&self, user_ids: &[String]) -> AppResult<()> {
        let mut cache = self.enhanced_cache.lock().await;
        for user_id in user_ids {
            // 这里应该从数据库获取用户信息，然后添加到缓存预热数据中
            // 暂时使用模拟数据
            let user_data = serde_json::to_vec(&serde_json::json! {
                {
                    "id": user_id,
                    "username": format!("user_{}", user_id),
                    "email": format!("user_{}@example.com", user_id),
                    "role": "user"
                }
            })?;
            
            let key = CacheKey::user(user_id.clone()).to_string();
            cache.add_warmup_data(key, user_data, Some(Duration::from_secs(3600)));
        }
        
        cache.warmup().await
    }

    /// 预热车辆缓存
    pub async fn warmup_vehicles(&self, vehicle_ids: &[String]) -> AppResult<()> {
        let mut cache = self.enhanced_cache.lock().await;
        for vehicle_id in vehicle_ids {
            // 这里应该从数据库获取车辆信息，然后添加到缓存预热数据中
            // 暂时使用模拟数据
            let vehicle_data = serde_json::to_vec(&serde_json::json! {
                {
                    "id": vehicle_id,
                    "license_plate": format!("京A{}", vehicle_id),
                    "vehicle_type": "truck",
                    "model": "解放J6",
                    "status": "active"
                }
            })?;
            
            let key = CacheKey::vehicle(vehicle_id.clone()).to_string();
            cache.add_warmup_data(key, vehicle_data, Some(Duration::from_secs(3600)));
        }
        
        cache.warmup().await
    }
}
