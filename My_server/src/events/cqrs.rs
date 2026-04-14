//! /! CQRS架构实现
//!
//! 实现命令查询职责分离(Command Query Responsibility Segregation)

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// 命令trait
pub trait Command: Debug + Serialize + for<'de> Deserialize<'de> + Send + Sync {
    /// 命令类型
    fn command_type() -> &'static str;

    /// 命令ID
    fn command_id(&self) -> &str;
}

/// 命令处理器trait
#[async_trait]
pub trait CommandHandler<C>: Send + Sync
where
    C: Command,
{
    /// 处理命令
    async fn handle(&self, command: C) -> Result<(), CommandError>;

    /// 获取处理器名称
    fn name(&self) -> &str;
}

/// 命令错误
#[derive(Debug, thiserror::Error)]
pub enum CommandError {
    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Execution error: {0}")]
    Execution(String),

    #[error("Concurrency error: {0}")]
    Concurrency(String),

    #[error("Other error: {0}")]
    Other(String),
}

/// 查询trait
pub trait Query<R>: Debug + Serialize + for<'de> Deserialize<'de> + Send + Sync
where
    R: Debug + Serialize + Send + Sync + for<'de> Deserialize<'de>,
{
    /// 查询类型
    fn query_type() -> &'static str;

    /// 查询ID
    fn query_id(&self) -> &str;
}

/// 查询处理器trait
#[async_trait]
pub trait QueryHandler<Q, R>: Send + Sync
where
    Q: Query<R>,
    R: Debug + Serialize + Send + Sync + for<'de> Deserialize<'de>,
{
    /// 处理查询
    async fn handle(&self, query: Q) -> Result<R, QueryError>;

    /// 获取处理器名称
    fn name(&self) -> &str;
}

/// 查询错误
#[derive(Debug, thiserror::Error)]
pub enum QueryError {
    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Execution error: {0}")]
    Execution(String),

    #[error("NotFound error: {0}")]
    NotFound(String),

    #[error("Other error: {0}")]
    Other(String),
}

/// 命令处理器注册表
pub struct CommandHandlerRegistry {
    handlers: std::sync::Arc<
        tokio::sync::RwLock<
            std::collections::HashMap<String, std::sync::Arc<dyn CommandHandlerAny>>,
        >,
    >,
}

/// 类型擦除的命令处理器
#[async_trait]
pub trait CommandHandlerAny: Send + Sync {
    async fn handle(&self, command_json: Vec<u8>) -> Result<(), CommandError>;
    fn name(&self) -> &str;
}

/// 命令处理器包装器
struct AnyCommandHandlerWrapper<C, H>
where
    C: Command,
    H: CommandHandler<C>,
{
    handler: H,
    _phantom: std::marker::PhantomData<C>,
}

impl<C, H> AnyCommandHandlerWrapper<C, H>
where
    C: Command,
    H: CommandHandler<C>,
{
    fn new(handler: H) -> Self {
        Self {
            handler,
            _phantom: std::marker::PhantomData,
        }
    }
}

#[async_trait]
impl<C, H> CommandHandlerAny for AnyCommandHandlerWrapper<C, H>
where
    C: Command,
    H: CommandHandler<C>,
{
    async fn handle(&self, command_json: Vec<u8>) -> Result<(), CommandError> {
        let command: C = serde_json::from_slice(&command_json).map_err(|e| {
            CommandError::Validation(format!("Failed to deserialize command: {}", e))
        })?;
        self.handler.handle(command).await
    }

    fn name(&self) -> &str {
        self.handler.name()
    }
}

/// 命令处理器注册表实现
impl CommandHandlerRegistry {
    pub fn new() -> Self {
        Self {
            handlers: std::sync::Arc::new(tokio::sync::RwLock::new(
                std::collections::HashMap::new(),
            )),
        }
    }

    /// 注册命令处理器
    pub async fn register<C, H>(&self, handler: H)
    where
        C: Command + 'static,
        H: CommandHandler<C> + 'static,
    {
        let command_type = C::command_type();
        let any_handler = std::sync::Arc::new(AnyCommandHandlerWrapper::new(handler));

        self.handlers
            .write()
            .await
            .insert(command_type.to_string(), any_handler);

        log::info!("Registered command handler for command '{}'", command_type);
    }

    /// 获取命令处理器
    pub async fn get_handler(
        &self,
        command_type: &str,
    ) -> Option<std::sync::Arc<dyn CommandHandlerAny>> {
        self.handlers.read().await.get(command_type).cloned()
    }
}

impl Default for CommandHandlerRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// 查询处理器注册表
pub struct QueryHandlerRegistry {
    handlers: std::sync::Arc<
        tokio::sync::RwLock<std::collections::HashMap<String, std::sync::Arc<dyn QueryHandlerAny>>>,
    >,
}

/// 类型擦除的查询处理器
#[async_trait]
pub trait QueryHandlerAny: Send + Sync {
    async fn handle(&self, query_json: Vec<u8>) -> Result<Vec<u8>, QueryError>;
    fn name(&self) -> &str;
}

/// 查询处理器包装器
struct AnyQueryHandlerWrapper<Q, R, H>
where
    Q: Query<R>,
    R: Debug + Serialize + Send + Sync + for<'de> Deserialize<'de>,
    H: QueryHandler<Q, R>,
{
    handler: H,
    _phantom: std::marker::PhantomData<(Q, R)>,
}

impl<Q, R, H> AnyQueryHandlerWrapper<Q, R, H>
where
    Q: Query<R>,
    R: Debug + Serialize + Send + Sync + for<'de> Deserialize<'de>,
    H: QueryHandler<Q, R>,
{
    fn new(handler: H) -> Self {
        Self {
            handler,
            _phantom: std::marker::PhantomData,
        }
    }
}

#[async_trait]
impl<Q, R, H> QueryHandlerAny for AnyQueryHandlerWrapper<Q, R, H>
where
    Q: Query<R>,
    R: Debug + Serialize + Send + Sync + for<'de> Deserialize<'de>,
    H: QueryHandler<Q, R>,
{
    async fn handle(&self, query_json: Vec<u8>) -> Result<Vec<u8>, QueryError> {
        let query: Q = serde_json::from_slice(&query_json)
            .map_err(|e| QueryError::Validation(format!("Failed to deserialize query: {}", e)))?;

        let result = self.handler.handle(query).await?;
        let result_json = serde_json::to_vec(&result)
            .map_err(|e| QueryError::Execution(format!("Failed to serialize result: {}", e)))?;

        Ok(result_json)
    }

    fn name(&self) -> &str {
        self.handler.name()
    }
}

/// 查询处理器注册表实现
impl QueryHandlerRegistry {
    pub fn new() -> Self {
        Self {
            handlers: std::sync::Arc::new(tokio::sync::RwLock::new(
                std::collections::HashMap::new(),
            )),
        }
    }

    /// 注册查询处理器
    pub async fn register<Q, R, H>(&self, handler: H)
    where
        Q: Query<R> + 'static,
        R: Debug + Serialize + Send + Sync + for<'de> Deserialize<'de> + 'static,
        H: QueryHandler<Q, R> + 'static,
    {
        let query_type = Q::query_type();
        let any_handler = std::sync::Arc::new(AnyQueryHandlerWrapper::new(handler));

        self.handlers
            .write()
            .await
            .insert(query_type.to_string(), any_handler);

        log::info!("Registered query handler for query '{}'", query_type);
    }

    /// 获取查询处理器
    pub async fn get_handler(
        &self,
        query_type: &str,
    ) -> Option<std::sync::Arc<dyn QueryHandlerAny>> {
        self.handlers.read().await.get(query_type).cloned()
    }
}

impl Default for QueryHandlerRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// CQRS应用服务
pub struct CqrsApplication {
    command_handler_registry: CommandHandlerRegistry,
    query_handler_registry: QueryHandlerRegistry,
}

impl CqrsApplication {
    pub fn new() -> Self {
        Self {
            command_handler_registry: CommandHandlerRegistry::new(),
            query_handler_registry: QueryHandlerRegistry::new(),
        }
    }

    /// 注册命令处理器
    pub async fn register_command_handler<C, H>(&self, handler: H)
    where
        C: Command + 'static,
        H: CommandHandler<C> + 'static,
    {
        self.command_handler_registry.register(handler).await;
    }

    /// 注册查询处理器
    pub async fn register_query_handler<Q, R, H>(&self, handler: H)
    where
        Q: Query<R> + 'static,
        R: Debug + Serialize + Send + Sync + for<'de> Deserialize<'de> + 'static,
        H: QueryHandler<Q, R> + 'static,
    {
        self.query_handler_registry.register(handler).await;
    }

    /// 处理命令
    pub async fn handle_command<C>(&self, command: C) -> Result<(), CommandError>
    where
        C: Command,
    {
        let command_type = C::command_type();
        log::debug!("Handling command: {}", command_type);

        let handler = self
            .command_handler_registry
            .get_handler(command_type)
            .await
            .ok_or_else(|| {
                CommandError::Other(format!("No handler found for command: {}", command_type))
            })?;

        let command_json = serde_json::to_vec(&command)
            .map_err(|e| CommandError::Validation(format!("Failed to serialize command: {}", e)))?;

        handler.handle(command_json).await
    }

    /// 处理查询
    pub async fn handle_query<Q, R>(&self, query: Q) -> Result<R, QueryError>
    where
        Q: Query<R>,
        R: Debug + Serialize + Send + Sync + for<'de> Deserialize<'de>,
    {
        let query_type = Q::query_type();
        log::debug!("Handling query: {}", query_type);

        let handler = self
            .query_handler_registry
            .get_handler(query_type)
            .await
            .ok_or_else(|| {
                QueryError::Other(format!("No handler found for query: {}", query_type))
            })?;

        let query_json = serde_json::to_vec(&query)
            .map_err(|e| QueryError::Validation(format!("Failed to serialize query: {}", e)))?;

        let result_json = handler.handle(query_json).await?;
        let result: R = serde_json::from_slice(&result_json)
            .map_err(|e| QueryError::Execution(format!("Failed to deserialize result: {}", e)))?;

        Ok(result)
    }
}

impl Default for CqrsApplication {
    fn default() -> Self {
        Self::new()
    }
}

/// 全局CQRS应用
static GLOBAL_CQRS_APPLICATION: tokio::sync::OnceCell<CqrsApplication> =
    tokio::sync::OnceCell::const_new();

/// 初始化全局CQRS应用
pub async fn init_global_cqrs_application() {
    GLOBAL_CQRS_APPLICATION
        .get_or_init(|| async { CqrsApplication::new() })
        .await;
}

/// 获取全局CQRS应用
pub fn global_cqrs_application() -> &'static CqrsApplication {
    GLOBAL_CQRS_APPLICATION
        .get()
        .expect("CQRS application not initialized")
}

/// 便捷函数:处理命令
pub async fn handle_command<C>(command: C) -> Result<(), CommandError>
where
    C: Command,
{
    global_cqrs_application().handle_command(command).await
}

/// 便捷函数:处理查询
pub async fn handle_query<Q, R>(query: Q) -> Result<R, QueryError>
where
    Q: Query<R>,
    R: Debug + Serialize + Send + Sync + for<'de> Deserialize<'de>,
{
    global_cqrs_application().handle_query(query).await
}
