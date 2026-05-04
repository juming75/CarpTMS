//! /! DDD(领域驱动设计)核心组件
//!
//! 实现DDD的核心概念:实体、值对象、聚合根、仓储、领域服务等

use crate::errors::AppResult;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Row};
use std::fmt::Debug;
use std::hash::Hash;

/// 实体trait - 具有唯一标识的领域对象
pub trait Entity: Debug + Send + Sync {
    /// 获取实体ID
    fn id(&self) -> &impl EntityId;
}

/// 实体ID trait
pub trait EntityId:
    Debug
    + Send
    + Sync
    + PartialEq
    + Eq
    + Hash
    + Clone
    + Serialize
    + for<'de> Deserialize<'de>
    + std::fmt::Display
{
    /// ID的类型标识
    fn type_name(&self) -> &'static str;
}

/// 值对象trait - 没有标识,通过属性值判断相等
pub trait ValueObject:
    Debug + Send + Sync + PartialEq + Eq + Clone + Serialize + for<'de> Deserialize<'de>
{
}

/// 聚合根trait - 领域模型的核心,管理内部一致性
pub trait AggregateRoot: Entity {
    /// 获取聚合根的版本号(用于乐观锁)
    fn version(&self) -> u64;

    /// 获取领域事件
    fn events(&self) -> &[DomainEvent];

    /// 清除领域事件
    fn clear_events(&mut self);
}

/// 领域事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainEvent {
    pub id: String,
    pub aggregate_type: String,
    pub aggregate_id: String,
    pub event_type: String,
    pub event_data: serde_json::Value,
    pub occurred_at: chrono::DateTime<chrono::Utc>,
    pub version: i32,
}

impl DomainEvent {
    pub fn new(
        aggregate_type: &str,
        aggregate_id: &str,
        event_type: &str,
        event_data: serde_json::Value,
        version: i32,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            aggregate_type: aggregate_type.to_string(),
            aggregate_id: aggregate_id.to_string(),
            event_type: event_type.to_string(),
            event_data,
            occurred_at: chrono::Utc::now(),
            version,
        }
    }
}

// 为DomainEvent实现FromRow trait,以便sqlx::query_as可以使用它
impl<'r> FromRow<'r, sqlx::postgres::PgRow> for DomainEvent {
    fn from_row(row: &'r sqlx::postgres::PgRow) -> Result<Self, sqlx::Error> {
        Ok(Self {
            id: row.try_get("id")?,
            aggregate_type: row.try_get("aggregate_type")?,
            aggregate_id: row.try_get("aggregate_id")?,
            event_type: row.try_get("event_type")?,
            event_data: row.try_get("event_data")?,
            occurred_at: row.try_get("occurred_at")?,
            version: row.try_get("version")?,
        })
    }
}

/// 仓储trait - 负责聚合的持久化
#[async_trait]
pub trait Repository<T, ID>: Send + Sync
where
    T: AggregateRoot + Send + Sync,
    ID: EntityId + Send + Sync,
{
    /// 根据ID查找聚合
    async fn find_by_id(&self, id: &ID) -> AppResult<Option<T>>;

    /// 保存聚合
    async fn save(&self, aggregate: &mut T) -> AppResult<()>;

    /// 删除聚合
    async fn delete(&self, id: &ID) -> AppResult<()>;

    /// 查找所有聚合
    async fn find_all(&self) -> AppResult<Vec<T>>;

    /// 根据条件查找聚合
    async fn find_where(
        &self,
        predicate: Box<dyn Fn(&T) -> bool + Send + Sync>,
    ) -> AppResult<Vec<T>>;
}

/// 领域服务trait - 协调多个聚合的业务逻辑
#[async_trait]
pub trait DomainService: Send + Sync {
    /// 服务名称
    fn name(&self) -> &str;

    /// 执行领域逻辑
    async fn execute(&self) -> AppResult<()>;
}

/// 应用服务trait - 应用层服务,协调领域服务和仓储
#[async_trait]
pub trait ApplicationService: Send + Sync {
    /// 服务名称
    fn name(&self) -> &str;

    /// 初始化服务
    async fn initialize(&self) -> AppResult<()>;

    /// 执行应用逻辑
    async fn execute(&self, command: Command) -> AppResult<CommandResult>;
}

/// 命令 - 表示对系统的意图
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Command {
    pub command_type: String,
    pub aggregate_id: Option<String>,
    pub data: serde_json::Value,
}

impl Command {
    pub fn new(command_type: &str, data: serde_json::Value) -> Self {
        Self {
            command_type: command_type.to_string(),
            aggregate_id: None,
            data,
        }
    }

    pub fn with_aggregate_id(mut self, aggregate_id: String) -> Self {
        self.aggregate_id = Some(aggregate_id);
        self
    }
}

/// 命令处理结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandResult {
    pub success: bool,
    pub events: Vec<DomainEvent>,
    pub data: Option<serde_json::Value>,
    pub error: Option<String>,
}

impl CommandResult {
    pub fn success(events: Vec<DomainEvent>) -> Self {
        Self {
            success: true,
            events,
            data: None,
            error: None,
        }
    }

    pub fn success_with_data(events: Vec<DomainEvent>, data: serde_json::Value) -> Self {
        Self {
            success: true,
            events,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(error: String) -> Self {
        Self {
            success: false,
            events: vec![],
            data: None,
            error: Some(error),
        }
    }
}

/// 查询 - 表示对数据的请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Query {
    pub query_type: String,
    pub parameters: serde_json::Value,
}

impl Query {
    pub fn new(query_type: &str, parameters: serde_json::Value) -> Self {
        Self {
            query_type: query_type.to_string(),
            parameters,
        }
    }
}

/// 查询结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> QueryResult<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(error: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error),
        }
    }
}

/// 规范模式 - 定义业务规则
pub trait Specification<T>: Send + Sync {
    /// 测试对象是否满足规范
    fn is_satisfied_by(&self, candidate: &T) -> bool;

    /// 与其他规范组合(AND)
    fn and(self, other: impl Specification<T> + 'static) -> AndSpecification<T>
    where
        Self: Sized + 'static,
    {
        AndSpecification::new(Box::new(self), Box::new(other))
    }

    /// 与其他规范组合(OR)
    fn or(self, other: impl Specification<T> + 'static) -> OrSpecification<T>
    where
        Self: Sized + 'static,
    {
        OrSpecification::new(Box::new(self), Box::new(other))
    }

    /// 规范取反
    fn not(self) -> NotSpecification<T>
    where
        Self: Sized + 'static,
    {
        NotSpecification::new(Box::new(self))
    }
}

/// AND组合规范
pub struct AndSpecification<T> {
    specifications: Vec<Box<dyn Specification<T>>>,
}

impl<T> AndSpecification<T> {
    pub fn new(spec1: Box<dyn Specification<T>>, spec2: Box<dyn Specification<T>>) -> Self {
        Self {
            specifications: vec![spec1, spec2],
        }
    }
}

impl<T> Specification<T> for AndSpecification<T> {
    fn is_satisfied_by(&self, candidate: &T) -> bool {
        self.specifications
            .iter()
            .all(|spec| spec.is_satisfied_by(candidate))
    }
}

/// OR组合规范
pub struct OrSpecification<T> {
    specifications: Vec<Box<dyn Specification<T>>>,
}

impl<T> OrSpecification<T> {
    pub fn new(spec1: Box<dyn Specification<T>>, spec2: Box<dyn Specification<T>>) -> Self {
        Self {
            specifications: vec![spec1, spec2],
        }
    }
}

impl<T> Specification<T> for OrSpecification<T> {
    fn is_satisfied_by(&self, candidate: &T) -> bool {
        self.specifications
            .iter()
            .any(|spec| spec.is_satisfied_by(candidate))
    }
}

/// NOT规范
pub struct NotSpecification<T> {
    specification: Box<dyn Specification<T>>,
}

impl<T> NotSpecification<T> {
    pub fn new(specification: Box<dyn Specification<T>>) -> Self {
        Self { specification }
    }
}

impl<T> Specification<T> for NotSpecification<T> {
    fn is_satisfied_by(&self, candidate: &T) -> bool {
        !self.specification.is_satisfied_by(candidate)
    }
}

/// 事件溯源支持
pub trait EventSourcedAggregate: AggregateRoot {
    /// 从事件重建聚合状态
    fn rebuild_from_events(&mut self, events: &[DomainEvent]) -> AppResult<()>;

    /// 获取所有变更事件(用于持久化)
    fn get_uncommitted_events(&self) -> &[DomainEvent];

    /// 标记事件已提交
    fn mark_events_committed(&mut self);
}

/// 工作单元 - 管理事务边界
pub struct UnitOfWork<T, ID, R>
where
    T: AggregateRoot,
    ID: EntityId,
    R: Repository<T, ID>,
{
    repository: R,
    aggregates: std::collections::HashMap<String, T>,
    _phantom: std::marker::PhantomData<ID>,
}

impl<T, ID, R> UnitOfWork<T, ID, R>
where
    T: AggregateRoot,
    ID: EntityId,
    R: Repository<T, ID>,
{
    pub fn new(repository: R) -> Self {
        Self {
            repository,
            aggregates: std::collections::HashMap::new(),
            _phantom: std::marker::PhantomData,
        }
    }

    /// 注册聚合到工作单元
    pub fn register(&mut self, aggregate: T) {
        let id = aggregate.id().to_string();
        self.aggregates.insert(id, aggregate);
    }

    /// 获取注册的聚合
    pub fn get(&self, id: &str) -> Option<&T> {
        self.aggregates.get(id)
    }

    /// 获取可修改的聚合
    pub fn get_mut(&mut self, id: &str) -> Option<&mut T> {
        self.aggregates.get_mut(id)
    }

    /// 提交所有变更
    pub async fn commit(&mut self) -> AppResult<Vec<DomainEvent>> {
        let mut all_events = Vec::new();

        for (_, aggregate) in self.aggregates.iter_mut() {
            self.repository.save(aggregate).await?;
            all_events.extend_from_slice(aggregate.events());
        }

        Ok(all_events)
    }

    /// 回滚所有变更
    pub fn rollback(&mut self) {
        self.aggregates.clear();
    }
}
