# 分表分库与缓存策略设计

## 1. 分表分库策略

### 1.1 分表策略

#### 1.1.1 垂直分表
- **定义**：将表中不同业务逻辑的字段分布到不同的表中
- **适用场景**：表字段过多，或存在热点字段
- **实现方案**：
  - `vehicles`：基础信息保留，将不常用字段拆分到`vehicle_extensions`表
  - `logistics_tracks`：核心轨迹数据保留，将附加信息拆分到`track_extensions`表

#### 1.1.2 水平分表
- **定义**：将表中的数据按一定规则分布到多个结构相同的表中
- **适用场景**：表数据量过大，查询性能下降

##### 1.1.2.1 按时间分区（已实现）
- **实现**：对`logistics_tracks`、`weighing_data`、`audit_logs`按天分区
- **配置**：在`06_partition_logistics_tracks.sql`中已实现
- **优势**：
  - 提高查询性能，自动过滤无关分区
  - 方便数据管理，可按时间删除或归档数据
  - 支持并行查询，提高查询效率

##### 1.1.2.2 按车辆ID哈希分区
- **实现**：对`logistics_tracks`和`vehicle_realtime_locations`按`vehicle_id`哈希分区
- **分区数量**：根据车辆规模动态调整
  - 5000台车：8个分区
  - 50000台车：64个分区
  - 200000台车：256个分区
- **优势**：
  - 均衡数据分布，避免热点分区
  - 支持水平扩展，可动态增加分区
  - 提高并发查询性能

### 1.2 分库策略

#### 1.2.1 垂直分库
- **定义**：将不同业务模块的数据分布到不同的数据库中
- **实现方案**：
  - **主数据库**：车辆基本信息、用户信息
  - **轨迹数据库**：轨迹数据、实时位置
  - **业务数据库**：订单信息、物流信息
- **优势**：
  - 隔离业务风险，提高系统可用性
  - 优化资源分配，提高各数据库性能
  - 支持独立扩展，便于业务发展

#### 1.2.2 水平分库
- **定义**：将相同业务的数据按一定规则分布到多个数据库中
- **实现方案**：
  - **分片键**：`vehicle_id`（哈希分片）
  - **分片算法**：一致性哈希算法
  - **分片数量**：
    - 50000台车：4个分片
    - 200000台车：16个分片
    - 1000000台车：64个分片
- **优势**：
  - 解决单库性能瓶颈
  - 支持大规模扩展
  - 提高系统可用性

### 1.3 分表分库路由实现

#### 1.3.1 路由规则
- **时间分区路由**：根据`track_time`字段路由到对应的分区表
- **哈希分区路由**：根据`vehicle_id`的哈希值路由到对应的分区表
- **分库路由**：根据`vehicle_id`的哈希值路由到对应的数据库实例

#### 1.3.2 实现方式
- **应用层分片**：在应用代码中实现分片逻辑
- **中间件分片**：使用数据库中间件（如PgBouncer、Citus）实现分片
- **混合模式**：时间分区使用PostgreSQL内置分区，水平分库使用应用层分片

## 2. Redis缓存策略

### 2.1 缓存架构

#### 2.1.1 缓存层次
- **本地缓存**：使用内存缓存（如Rust的`lru-cache`）存储热点数据
- **分布式缓存**：Redis集群存储全局热点数据

#### 2.1.2 Redis部署方案
- **5000台车**：单Redis实例
- **50000台车**：Redis主从架构（1主2从）
- **200000台车**：Redis集群（3主3从）
- **1000000台车**：Redis集群（10主10从）

### 2.2 缓存设计

#### 2.2.1 缓存键设计
- **命名规范**：`业务:模块:标识符:属性`
- **示例**：
  - 车辆实时位置：`vehicle:realtime:{vehicle_id}`
  - 车辆基本信息：`vehicle:info:{vehicle_id}`
  - 轨迹摘要：`track:summary:{vehicle_id}:{date}`
  - 围栏内车辆：`fence:vehicles:{fence_id}`

#### 2.2.2 缓存数据结构
- **字符串**：存储车辆基本信息、实时位置等简单数据
- **哈希**：存储轨迹摘要、车辆状态等复杂数据
- **集合**：存储围栏内车辆ID、在线车辆ID等
- **有序集合**：存储轨迹点、车辆速度排名等

#### 2.2.3 TTL设计
- **实时位置**：1小时
- **车辆基本信息**：24小时
- **轨迹摘要**：7天
- **围栏信息**：10分钟
- **热点数据**：根据访问频率动态调整

### 2.3 缓存更新策略

#### 2.3.1 主动更新
- **数据写入时更新**：当数据写入数据库时，同时更新缓存
- **定时任务更新**：定期更新热点数据
- **事件驱动更新**：通过Kafka消息驱动缓存更新

#### 2.3.2 被动失效
- **TTL自动失效**：缓存到达过期时间后自动失效
- **缓存击穿处理**：使用互斥锁防止缓存击穿
- **缓存雪崩处理**：设置随机TTL，避免缓存同时失效

#### 2.3.3 缓存一致性
- **最终一致性**：保证数据最终一致，允许短暂不一致
- **写入策略**：
  - 先写数据库，再删缓存（推荐）
  - 或使用延迟双删策略
- **读取策略**：
  - 先读缓存，缓存未命中则读数据库，然后更新缓存

### 2.4 Redis集成实现

#### 2.4.1 依赖添加
- 在`Cargo.toml`中添加Redis依赖：
  ```toml
  redis = { version = "0.24.0", features = ["tokio-comp", "json"] }
  ```

#### 2.4.2 Redis连接池实现
```rust
// src/redis.rs
use log::{error, info};
use redis::{Client, RedisError, aio::ConnectionManager};
use std::env;
use tokio::sync::OnceCell;

static REDIS_CLIENT: OnceCell<Client> = OnceCell::const_new();
static REDIS_CONNECTION_MANAGER: OnceCell<ConnectionManager> = OnceCell::const_new();

pub async fn init_redis() -> Result<(), RedisError> {
    let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379/0".to_string());
    
    info!("Connecting to Redis: {}", redis_url);
    
    let client = Client::open(redis_url)?;
    let connection_manager = client.get_tokio_connection_manager().await?;
    
    REDIS_CLIENT.set(client).ok();
    REDIS_CONNECTION_MANAGER.set(connection_manager).ok();
    
    info!("Successfully connected to Redis");
    Ok(())
}

pub async fn get_redis_manager() -> &'static ConnectionManager {
    REDIS_CONNECTION_MANAGER.get().expect("Redis connection manager not initialized")
}
```

#### 2.4.3 缓存操作封装
```rust
// src/cache/vehicle_cache.rs
use crate::models::VehicleRealtimeLocation;
use redis::{AsyncCommands, JsonCommands};
use serde_json::json;

pub async fn set_vehicle_realtime_location(
    vehicle_id: i32,
    location: &VehicleRealtimeLocation
) -> Result<(), redis::RedisError> {
    let manager = crate::redis::get_redis_manager().await;
    let key = format!("vehicle:realtime:{}", vehicle_id);
    
    manager
        .json_set(&key, ".", &json!(location))
        .await?;
    
    // 设置TTL为1小时
    manager
        .expire(&key, 3600)
        .await?;
    
    Ok(())
}

pub async fn get_vehicle_realtime_location(
    vehicle_id: i32
) -> Result<Option<VehicleRealtimeLocation>, redis::RedisError> {
    let manager = crate::redis::get_redis_manager().await;
    let key = format!("vehicle:realtime:{}", vehicle_id);
    
    let location: Option<VehicleRealtimeLocation> = manager
        .json_get(&key, ".")
        .await?;
    
    Ok(location)
}
```

## 3. 分表分库与缓存的集成方案

### 3.1 数据访问层设计

#### 3.1.1 Repository模式
- **定义**：封装数据访问逻辑，提供统一的数据访问接口
- **实现**：
  ```rust
  pub trait TrackRepository {
      async fn save_track(&self, track: &LogisticsTrack) -> Result<(), DatabaseError>;
      async fn get_track_by_vehicle_id(
          &self, 
          vehicle_id: i32, 
          start_time: DateTime<Utc>, 
          end_time: DateTime<Utc>
      ) -> Result<Vec<LogisticsTrack>, DatabaseError>;
      // 其他方法...
  }
  ```

#### 3.1.2 分表分库路由
- **实现**：在Repository实现中，根据分区规则路由到对应的表或数据库
- **示例**：
  ```rust
  async fn get_track_by_vehicle_id(
      &self, 
      vehicle_id: i32, 
      start_time: DateTime<Utc>, 
      end_time: DateTime<Utc>
  ) -> Result<Vec<LogisticsTrack>, DatabaseError> {
      // 1. 确定时间分区
      let partition_table = self.get_partition_table(start_time).await?;
      
      // 2. 确定车辆ID分区
      let shard_db = self.get_shard_db(vehicle_id).await?;
      
      // 3. 执行查询
      let tracks = sqlx::query_as!(LogisticsTrack, 
          r#"SELECT * FROM "$1" WHERE vehicle_id = $2 AND track_time BETWEEN $3 AND $4"#, 
          partition_table, vehicle_id, start_time, end_time
      )
      .fetch_all(shard_db)
      .await?;
      
      Ok(tracks)
  }
  ```

#### 3.1.3 缓存集成
- **实现**：在Repository中集成缓存逻辑，实现缓存的自动更新和失效
- **示例**：
  ```rust
  async fn get_vehicle_realtime_location(
      &self, 
      vehicle_id: i32
  ) -> Result<Option<VehicleRealtimeLocation>, DatabaseError> {
      // 1. 尝试从缓存获取
      if let Some(location) = vehicle_cache::get_vehicle_realtime_location(vehicle_id).await? {
          return Ok(Some(location));
      }
      
      // 2. 缓存未命中，从数据库获取
      let location = sqlx::query_as!(VehicleRealtimeLocation, 
          r#"SELECT * FROM vehicle_realtime_locations WHERE vehicle_id = $1"#, 
          vehicle_id
      )
      .fetch_optional(&self.pool)
      .await?;
      
      // 3. 更新缓存
      if let Some(ref loc) = location {
          vehicle_cache::set_vehicle_realtime_location(vehicle_id, loc).await?;
      }
      
      Ok(location)
  }
  ```

### 3.2 性能监控与调优

#### 3.2.1 监控指标
- **数据库指标**：
  - 查询响应时间
  - 写入吞吐量
  - 连接池使用率
  - 分区访问频率
- **缓存指标**：
  - 缓存命中率
  - 缓存更新延迟
  - Redis内存使用率
  - Redis连接数

#### 3.2.2 调优策略
- **根据监控数据调整分区规则**：
  - 如果某个分区访问频繁，考虑拆分
  - 如果某个分区数据量过大，考虑增加分区数量
- **根据缓存命中率调整缓存策略**：
  - 提高热点数据的TTL
  - 增加缓存数据量
  - 调整缓存更新频率
- **根据查询模式优化索引**：
  - 添加复合索引
  - 优化现有索引
  - 删除无用索引

## 4. 分表分库与缓存的演进路线

### 4.1 阶段1：5000台车
- **分表**：实现时间分区
- **缓存**：单Redis实例，基本缓存功能
- **监控**：基础监控

### 4.2 阶段2：50000台车
- **分表**：实现车辆ID哈希分区
- **缓存**：Redis主从架构，完善缓存策略
- **监控**：全面监控，包括数据库和缓存

### 4.3 阶段3：200000台车
- **分库**：实现水平分库
- **缓存**：Redis集群，优化缓存一致性
- **监控**：分布式监控，性能分析

### 4.4 阶段4：1000000台车
- **分库**：动态分库，自动扩容
- **缓存**：多级缓存架构，智能缓存策略
- **监控**：智能监控，自动调优

## 5. 代码实现建议

### 5.1 数据库连接池优化
```rust
// src/database.rs
pub async fn init_db() -> Result<PgPool, sqlx::Error> {
    // ... 现有代码 ...
    
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(100) // 增加最大连接数
        .min_connections(10) // 增加最小连接数
        .idle_timeout(Some(std::time::Duration::from_secs(300)))
        .max_lifetime(Some(std::time::Duration::from_secs(1800)))
        .connect(&connection_string)
        .await?;
    
    // ... 现有代码 ...
}
```

### 5.2 分表分库路由实现
```rust
// src/sharding/route.rs
pub fn get_partition_table(table_name: &str, timestamp: DateTime<Utc>) -> String {
    let date_str = timestamp.format("%Y%m%d").to_string();
    format!("{}_{}", table_name, date_str)
}

pub fn get_shard_db(vehicle_id: i32, shard_count: u32) -> u32 {
    (vehicle_id as u32) % shard_count
}
```

### 5.3 缓存一致性保障
```rust
// src/cache/consistency.rs
pub async fn update_with_consistency<T, F>(
    key: &str,
    update_func: F,
    ttl: usize
) -> Result<T, redis::RedisError>
where
    F: FnOnce() -> Result<T, redis::RedisError>,
    T: serde::Serialize + serde::de::DeserializeOwned,
{
    // 1. 执行更新操作
    let result = update_func().await?;
    
    // 2. 更新缓存
    let manager = crate::redis::get_redis_manager().await;
    manager.json_set(key, ".", &result).await?;
    manager.expire(key, ttl as u64).await?;
    
    Ok(result)
}
```

## 6. 总结

本设计方案提供了详细的分表分库与缓存策略，包括：

1. **分表策略**：垂直分表和水平分表，其中水平分表包括时间分区和车辆ID哈希分区
2. **分库策略**：垂直分库和水平分库，基于一致性哈希算法实现
3. **Redis缓存策略**：缓存架构、键设计、更新策略和一致性保障
4. **集成方案**：数据访问层设计、Repository模式和缓存集成
5. **演进路线**：根据车辆规模动态调整分表分库和缓存策略

通过实施本方案，可以有效提高系统的性能和可扩展性，支持从5000台车到100万台的平滑演进，同时保证系统的可用性和数据一致性。

