# CarpTMS DDD 架构指南

## 概述

本文档说明 CarpTMS 系统的 DDD（领域驱动设计）架构实现，包括：
- 当前架构状态
- 理想架构目标
- 改造路径

## 当前架构状态

### 已实现的组件

✅ **Domain Layer (领域层)**
- `src/domain/entities/` - 领域实体
- `src/domain/use_cases/` - 业务用例（核心业务逻辑）
- `src/domain/value_objects/` - 值对象
- `src/domain/events/` - 领域事件
- `src/domain/repositories/` - 仓储接口

✅ **Infrastructure Layer (基础设施层)**
- `src/infrastructure/repositories/` - 仓储实现（PostgreSQL）
- `src/infrastructure/persistence/` - 持久化
- `src/infrastructure/cache/` - 缓存实现

✅ **Application Layer (应用层)**
- `src/application/services/` - 应用服务（部分）

⚠️ **Interface Layer (接口层)**
- `src/routes/` - HTTP 路由（部分使用直接 SQL）

### 当前调用链路

**现状（混合模式）：**
```
HTTP 请求 
  ↓
routes/vehicles.rs (混合模式)
  ├─→ 直接使用 sqlx + PostgreSQL (部分接口)
  └─→ 使用 application/services/vehicle_service.rs (部分接口)
       ↓
     repository
       ↓
     PostgreSQL
```

## 理想架构目标

### 完整的 DDD 分层架构

```
┌─────────────────────────────────────────────────────┐
│               Interface Layer (接口层)                │
│  src/routes/                                        │
│  - HTTP 请求处理                                    │
│  - 参数提取、序列化、反序列化                       │
│  - 响应构建                                         │
│  - 不包含任何业务逻辑！                             │
└──────────────────┬──────────────────────────────────┘
                   │
                   ↓
┌─────────────────────────────────────────────────────┐
│             Application Layer (应用层)               │
│  src/domain/use_cases/                              │
│  - 业务用例编排                                     │
│  - 业务逻辑验证                                     │
│  - 事务管理                                         │
│  - 缓存策略                                         │
│  - 调用 Domain Services                              │
└──────────────────┬──────────────────────────────────┘
                   │
                   ↓
┌─────────────────────────────────────────────────────┐
│              Domain Layer (领域层)                   │
│  src/domain/entities/                               │
│  - 领域实体（聚合根、实体、值对象）                 │
│  - 领域事件                                         │
│  - 领域服务                                         │
│  - 仓储接口（Repository Trait）                     │
└──────────────────┬──────────────────────────────────┘
                   │
                   ↓
┌─────────────────────────────────────────────────────┐
│          Infrastructure Layer (基础设施层)          │
│  src/infrastructure/repositories/                   │
│  - 仓储实现（PostgreSQL）                           │
│  - 外部服务调用                                     │
│  - 缓存实现                                         │
└──────────────────┬──────────────────────────────────┘
                   │
                   ↓
              PostgreSQL / Redis
```

### 理想调用链路

```
HTTP 请求
  ↓
routes/vehicles.rs (只做 HTTP 适配)
  ├─ 提取 HTTP 参数
  ├─ 反序列化为领域对象
  └─ 调用 use_cases
       ↓
     domain/use_cases/vehicle.rs (核心业务逻辑)
       ├─ 业务规则验证
       ├─ 缓存处理
       └─ 调用 Repository
            ↓
          infrastructure/repositories/vehicle_repository.rs
               ↓
             PostgreSQL
```

## 各层职责详解

### 1. Interface Layer (接口层) - src/routes/

**职责：**
- ✅ 接收 HTTP 请求
- ✅ 提取和验证 HTTP 参数
- ✅ 反序列化请求体为领域对象
- ✅ 调用 use_cases 执行业务逻辑
- ✅ 序列化领域对象为 HTTP 响应
- ✅ 处理 HTTP 状态码
- ❌ **不包含任何业务逻辑**
- ❌ **不直接调用数据库**
- ❌ **不包含缓存逻辑**

**示例 - routes/vehicles.rs (理想状态)：**

```rust
// routes/vehicles.rs - 只做 HTTP 适配

use actix_web::{web, HttpResponse};
use crate::domain::use_cases::vehicle::VehicleUseCases;
use crate::domain::entities::vehicle::{VehicleQuery, VehicleCreate, VehicleUpdate};
use crate::schemas::VehicleResponse;

/// 获取车辆列表
pub async fn get_vehicles(
    use_cases: web::Data<VehicleUseCases>,
    query: web::Query<VehicleQuery>,
) -> AppResult<HttpResponse> {
    // 1. 提取 HTTP 参数
    let vehicle_query = query.into_inner();
    
    // 2. 调用 use_cases 执行业务逻辑
    let (vehicles, total) = use_cases.get_vehicles(vehicle_query).await?;
    
    // 3. 转换为 HTTP 响应
    let responses: Vec<VehicleResponse> = vehicles
        .into_iter()
        .map(VehicleResponse::from)
        .collect();
    
    // 4. 返回 HTTP 响应
    Ok(success_response(responses, total))
}

/// 创建车辆
pub async fn create_vehicle(
    use_cases: web::Data<VehicleUseCases>,
    body: web::Json<VehicleCreate>,
) -> AppResult<HttpResponse> {
    // 1. 反序列化为领域对象
    let create_data = body.into_inner();
    
    // 2. 调用 use_cases
    let vehicle = use_cases.create_vehicle(create_data).await?;
    
    // 3. 转换为响应
    let response = VehicleResponse::from(vehicle);
    
    // 4. 返回
    Ok(created_response(response))
}
```

### 2. Application Layer (应用层) - src/domain/use_cases/

**职责：**
- ✅ 实现业务用例（Use Cases）
- ✅ 业务规则验证
- ✅ 缓存策略实现
- ✅ 事务管理
- ✅ 调用多个 Repository 协调操作
- ✅ 发布领域事件

**示例 - domain/use_cases/vehicle.rs：**

```rust
// domain/use_cases/vehicle.rs - 核心业务逻辑

use crate::domain::entities::vehicle::{Vehicle, VehicleCreate, VehicleQuery, VehicleUpdate};
use crate::domain::repositories::vehicle::VehicleRepository;
use crate::redis::{get_cache, set_cache, del_cache_pattern};

#[derive(Clone)]
pub struct VehicleUseCases {
    repository: Arc<dyn VehicleRepository>,
}

impl VehicleUseCases {
    /// 获取车辆列表 - 包含缓存逻辑
    pub async fn get_vehicles(
        &self,
        query: VehicleQuery,
    ) -> Result<(Vec<Vehicle>, i64), AppError> {
        // 1. 构建缓存键
        let cache_key = format!("vehicles:{:?}", query);
        
        // 2. 尝试从缓存获取
        if let Some(cached) = get_cache(&cache_key).await? {
            return Ok(cached);
        }
        
        // 3. 从数据库获取
        let result = self.repository.get_vehicles(query).await?;
        
        // 4. 写入缓存
        set_cache(&cache_key, &result, 1800).await?;
        
        Ok(result)
    }
    
    /// 创建车辆 - 包含业务验证
    pub async fn create_vehicle(
        &self,
        data: VehicleCreate,
    ) -> Result<Vehicle, AppError> {
        // 1. 业务规则验证
        self.validate_vehicle_create(&data)?;
        
        // 2. 调用仓储
        let vehicle = self.repository.create_vehicle(data).await?;
        
        // 3. 清除相关缓存
        del_cache_pattern("vehicles:*").await?;
        
        // 4. 发布领域事件
        self.publish_vehicle_created_event(&vehicle).await?;
        
        Ok(vehicle)
    }
    
    /// 业务验证逻辑
    fn validate_vehicle_create(&self, data: &VehicleCreate) -> Result<(), AppError> {
        if data.vehicle_name.is_empty() {
            return Err(AppError::Validation("车辆名称不能为空".into()));
        }
        if data.license_plate.is_empty() {
            return Err(AppError::Validation("车牌号不能为空".into()));
        }
        if data.inspection_date < data.register_date {
            return Err(AppError::Validation("年检日期不能早于注册日期".into()));
        }
        Ok(())
    }
}
```

### 3. Domain Layer (领域层) - src/domain/

**职责：**
- ✅ 领域实体定义
- ✅ 值对象定义
- ✅ 仓储接口（Trait）
- ✅ 领域事件
- ✅ 领域服务

**示例 - domain/entities/vehicle.rs：**

```rust
// domain/entities/vehicle.rs - 领域实体

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vehicle {
    pub vehicle_id: i32,
    pub vehicle_name: String,
    pub license_plate: String,
    // ... 其他字段
}

impl Vehicle {
    /// 领域逻辑 - 验证车辆状态
    pub fn is_operational(&self) -> bool {
        self.status == 1 && 
        self.inspection_date > chrono::Utc::now().naive_utc()
    }
    
    /// 领域逻辑 - 计算车辆使用年限
    pub fn age_years(&self) -> i32 {
        let now = chrono::Utc::now().naive_utc();
        let duration = now.signed_duration_since(self.register_date);
        duration.num_days() / 365
    }
}

/// 仓储接口 - 定义在领域层
#[async_trait]
pub trait VehicleRepository: Send + Sync {
    async fn get_vehicles(&self, query: VehicleQuery) -> Result<(Vec<Vehicle>, i64), AppError>;
    async fn get_vehicle(&self, id: i32) -> Result<Option<Vehicle>, AppError>;
    async fn create_vehicle(&self, data: VehicleCreate) -> Result<Vehicle, AppError>;
    async fn update_vehicle(&self, id: i32, data: VehicleUpdate) -> Result<Option<Vehicle>, AppError>;
    async fn delete_vehicle(&self, id: i32) -> Result<bool, AppError>;
}
```

### 4. Infrastructure Layer (基础设施层) - src/infrastructure/

**职责：**
- ✅ 实现仓储接口（PostgreSQL）
- ✅ 数据库访问
- ✅ 外部服务调用
- ✅ 缓存实现

**示例 - infrastructure/repositories/vehicle_repository.rs：**

```rust
// infrastructure/repositories/vehicle_repository.rs - 仓储实现

use sqlx::PgPool;
use crate::domain::entities::vehicle::{Vehicle, VehicleQuery, VehicleCreate, VehicleUpdate};
use crate::domain::use_cases::vehicle::VehicleRepository;

pub struct PgVehicleRepository {
    pool: Arc<PgPool>,
}

#[async_trait]
impl VehicleRepository for PgVehicleRepository {
    async fn get_vehicles(&self, query: VehicleQuery) -> Result<(Vec<Vehicle>, i64), AppError> {
        // 纯数据库操作，不包含业务逻辑
        let page = query.page.unwrap_or(1);
        let page_size = query.page_size.unwrap_or(20);
        let offset = (page - 1) * page_size;
        
        // 构建 SQL 查询
        let vehicles = sqlx::query_as!(Vehicle, "...")
            .fetch_all(&*self.pool)
            .await?;
            
        let count = sqlx::query_scalar!("SELECT COUNT(*) FROM vehicles ...")
            .fetch_one(&*self.pool)
            .await?;
            
        Ok((vehicles, count))
    }
    
    // ... 其他仓储方法
}
```

## 改造路径

### 第一阶段：现有代码确认 (已完成 ✅)

- ✅ 确认 use_cases 层已存在
- ✅ 确认 repository 层已存在
- ✅ 确认 entities 层已存在
- ✅ 代码编译通过
- ✅ Clippy 检查通过

### 第二阶段：单个模块改造 (示例：vehicles)

1. **修改 routes/vehicles.rs**
   - 移除直接 SQL 调用
   - 移除缓存逻辑
   - 只保留 HTTP 适配
   - 调用 use_cases

2. **扩展 use_cases/vehicle.rs**
   - 添加缓存逻辑（从 routes 移过来）
   - 添加更完整的业务验证
   - 添加事务管理

3. **测试验证**
   - 单元测试
   - 集成测试
   - 手动测试

### 第三阶段：其他模块改造

按照相同模式改造：
- drivers.rs
- vehicle_groups.rs
- weighing.rs
- finance.rs
- statistics.rs
- 等等

### 第四阶段：优化和完善

- 添加更多 use_cases 测试
- 性能优化
- 文档完善

## 关键要点

### Routes 层的职责

**Routes 层保留的：**
- HTTP 参数提取
- 请求/响应序列化
- HTTP 状态码处理
- 调用 use_cases

**Routes 层移除的：**
- 直接 SQL 调用
- 业务逻辑验证
- 缓存逻辑
- 事务管理

### Use Cases 层的职责

**Use Cases 层包含的：**
- 业务用例编排
- 业务规则验证
- 缓存策略
- 事务管理
- 领域事件发布

## 文件清单

### 已存在的关键文件

```
src/
├── domain/
│   ├── entities/
│   │   ├── vehicle.rs          ✅ 领域实体
│   │   ├── driver.rs
│   │   └── ...
│   ├── use_cases/
│   │   ├── vehicle.rs          ✅ 业务用例
│   │   ├── user.rs
│   │   └── ...
│   └── repositories/
│       └── vehicle_repository.rs  ✅ 仓储接口
├── infrastructure/
│   └── repositories/
│       ├── vehicle_repository.rs  ✅ 仓储实现
│       └── ...
└── routes/
    ├── vehicles.rs             ⚠️ 需要改造
    └── ...
```

## 总结

### 当前状态
- ✅ DDD 架构的各层已基本搭建完成
- ✅ use_cases、repository、entities 都已实现
- ⚠️ routes 层还在使用混合模式

### 下一步行动
1. 选择一个模块（如 vehicles）进行完整改造
2. 验证改造后的功能
3. 推广到其他模块
4. 完善测试和文档
