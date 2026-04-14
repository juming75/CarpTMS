//! / 实时数据缓存模块
// 使用 Redis 实现高性能数据缓存

pub mod realtime_cache;
pub mod trajectory_cache;

pub use realtime_cache::{RealtimeCache, CacheKey, CacheValue};
pub use trajectory_cache::{TrajectoryCache, TrajectoryPoint};






