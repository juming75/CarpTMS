//! /! 日志管理模块
//!
//! 提供日志聚合、分析和管理功能

use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::Error;
use chrono::{DateTime, Utc};
use futures_util::future::LocalBoxFuture;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 日志级别
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum LogLevel {
    #[serde(rename = "debug")]
    Debug,
    #[serde(rename = "info")]
    Info,
    #[serde(rename = "warn")]
    Warn,
    #[serde(rename = "error")]
    Error,
    #[serde(rename = "trace")]
    Trace,
}

/// 日志条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    /// 日志时间戳
    pub timestamp: DateTime<Utc>,
    /// 日志级别
    pub level: LogLevel,
    /// 日志消息
    pub message: String,
    /// 日志来源
    pub source: String,
    /// 日志标签
    pub tags: HashMap<String, String>,
    /// 日志上下文
    pub context: Option<serde_json::Value>,
}

/// 日志聚合器
pub struct LogAggregator {
    /// 日志缓冲区
    logs: Arc<RwLock<Vec<LogEntry>>>,
    /// 最大日志数量
    max_logs: usize,
}

impl LogAggregator {
    /// 创建新的日志聚合器
    pub fn new(max_logs: usize) -> Self {
        Self {
            logs: Arc::new(RwLock::new(Vec::new())),
            max_logs,
        }
    }

    /// 添加日志条目
    pub async fn add_log(&self, entry: LogEntry) {
        let mut logs = self.logs.write().await;

        // 添加新日志
        logs.push(entry);

        // 如果超过最大数量,删除 oldest logs
        if logs.len() > self.max_logs {
            let drain_count = logs.len() - self.max_logs;
            logs.drain(0..drain_count);
        }
    }

    /// 获取日志
    pub async fn get_logs(&self, limit: Option<usize>, level: Option<LogLevel>) -> Vec<LogEntry> {
        let logs = self.logs.read().await;

        let filtered_logs: Vec<LogEntry> = logs
            .iter()
            .filter(|log| match level {
                Some(lvl) => log.level == lvl,
                None => true,
            })
            .cloned()
            .collect();

        match limit {
            Some(lim) => filtered_logs.into_iter().take(lim).collect(),
            None => filtered_logs,
        }
    }

    /// 获取日志统计
    pub async fn get_log_stats(&self) -> LogStats {
        let logs = self.logs.read().await;

        let mut stats = LogStats {
            total: logs.len(),
            by_level: HashMap::new(),
            by_source: HashMap::new(),
        };

        for log in logs.iter() {
            // 按级别统计
            *stats.by_level.entry(log.level).or_insert(0) += 1;

            // 按来源统计
            *stats.by_source.entry(log.source.clone()).or_insert(0) += 1;
        }

        stats
    }

    /// 搜索日志
    pub async fn search_logs(&self, query: &str, limit: Option<usize>) -> Vec<LogEntry> {
        let logs = self.logs.read().await;

        let results: Vec<LogEntry> = logs
            .iter()
            .filter(|log| {
                log.message.contains(query)
                    || log.source.contains(query)
                    || log.tags.values().any(|v| v.contains(query))
            })
            .cloned()
            .collect();

        match limit {
            Some(lim) => results.into_iter().take(lim).collect(),
            None => results,
        }
    }
}

/// 日志统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogStats {
    /// 总日志数
    pub total: usize,
    /// 按级别统计
    pub by_level: HashMap<LogLevel, usize>,
    /// 按来源统计
    pub by_source: HashMap<String, usize>,
}

/// 日志管理器
pub struct LogManager {
    /// 日志聚合器
    aggregator: Arc<LogAggregator>,
}

impl LogManager {
    /// 创建新的日志管理器
    pub fn new(max_logs: usize) -> Self {
        Self {
            aggregator: Arc::new(LogAggregator::new(max_logs)),
        }
    }

    /// 获取日志聚合器
    pub fn aggregator(&self) -> Arc<LogAggregator> {
        self.aggregator.clone()
    }

    /// 记录调试日志
    pub async fn debug(
        &self,
        message: &str,
        source: &str,
        tags: Option<HashMap<String, String>>,
        context: Option<serde_json::Value>,
    ) {
        self.log(LogLevel::Debug, message, source, tags, context)
            .await;
    }

    /// 记录信息日志
    pub async fn info(
        &self,
        message: &str,
        source: &str,
        tags: Option<HashMap<String, String>>,
        context: Option<serde_json::Value>,
    ) {
        self.log(LogLevel::Info, message, source, tags, context)
            .await;
    }

    /// 记录警告日志
    pub async fn warn(
        &self,
        message: &str,
        source: &str,
        tags: Option<HashMap<String, String>>,
        context: Option<serde_json::Value>,
    ) {
        self.log(LogLevel::Warn, message, source, tags, context)
            .await;
    }

    /// 记录错误日志
    pub async fn error(
        &self,
        message: &str,
        source: &str,
        tags: Option<HashMap<String, String>>,
        context: Option<serde_json::Value>,
    ) {
        self.log(LogLevel::Error, message, source, tags, context)
            .await;
    }

    /// 记录追踪日志
    pub async fn trace(
        &self,
        message: &str,
        source: &str,
        tags: Option<HashMap<String, String>>,
        context: Option<serde_json::Value>,
    ) {
        self.log(LogLevel::Trace, message, source, tags, context)
            .await;
    }

    /// 记录日志
    async fn log(
        &self,
        level: LogLevel,
        message: &str,
        source: &str,
        tags: Option<HashMap<String, String>>,
        context: Option<serde_json::Value>,
    ) {
        let entry = LogEntry {
            timestamp: Utc::now(),
            level,
            message: message.to_string(),
            source: source.to_string(),
            tags: tags.unwrap_or_default(),
            context,
        };

        self.aggregator.add_log(entry).await;
    }
}

/// 全局日志管理器实例
pub static GLOBAL_LOG_MANAGER: once_cell::sync::OnceCell<Arc<LogManager>> =
    once_cell::sync::OnceCell::new();

/// 初始化日志管理器
pub fn init_log_manager(max_logs: usize) -> Result<(), String> {
    let log_manager = Arc::new(LogManager::new(max_logs));
    GLOBAL_LOG_MANAGER
        .set(log_manager)
        .map_err(|_| "Failed to set global log manager".to_string())?;
    Ok(())
}

/// 获取全局日志管理器
pub fn get_log_manager() -> Option<Arc<LogManager>> {
    GLOBAL_LOG_MANAGER.get().cloned()
}

/// 日志中间件
///
/// 用于在Actix Web中收集和聚合日志
pub struct LoggingMiddleware {
    log_manager: Arc<LogManager>,
}

impl LoggingMiddleware {
    /// 创建新的日志中间件
    pub fn new(log_manager: Arc<LogManager>) -> Self {
        Self { log_manager }
    }
}

impl<S, B> Transform<S, ServiceRequest> for LoggingMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = LoggingService<S>;
    type InitError = ();
    type Future = LocalBoxFuture<'static, Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        let log_manager = self.log_manager.clone();
        Box::pin(async move {
            Ok(LoggingService {
                service,
                log_manager,
            })
        })
    }
}

/// 日志服务
pub struct LoggingService<S> {
    service: S,
    log_manager: Arc<LogManager>,
}

impl<S, B> Service<ServiceRequest> for LoggingService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let start = std::time::Instant::now();
        let log_manager = self.log_manager.clone();
        let method = req.method().to_string();
        let path = req.path().to_string();
        let remote_addr = req
            .connection_info()
            .peer_addr()
            .unwrap_or("unknown")
            .to_string();

        let fut = self.service.call(req);

        Box::pin(async move {
            let result = fut.await;
            let duration = start.elapsed();

            match &result {
                Ok(res) => {
                    let status = res.status().as_u16();
                    let mut tags = HashMap::new();
                    tags.insert("method".to_string(), method.clone());
                    tags.insert("path".to_string(), path.clone());
                    tags.insert("status".to_string(), status.to_string());
                    tags.insert("remote_addr".to_string(), remote_addr.clone());
                    tags.insert("duration".to_string(), format!("{:?}", duration));

                    let context = serde_json::json!({
                        "method": method,
                        "path": path,
                        "status": status,
                        "duration": duration.as_millis(),
                        "remote_addr": remote_addr
                    });

                    let log_manager = log_manager.clone();
                    let message = format!("HTTP {} {} {}", status, method, path);
                    tokio::spawn(async move {
                        log_manager
                            .info(&message, "http", Some(tags), Some(context))
                            .await;
                    });
                }
                Err(err) => {
                    let mut tags = HashMap::new();
                    tags.insert("method".to_string(), method.clone());
                    tags.insert("path".to_string(), path.clone());
                    tags.insert("remote_addr".to_string(), remote_addr.clone());
                    tags.insert("duration".to_string(), format!("{:?}", duration));

                    let context = serde_json::json!({
                        "method": method,
                        "path": path,
                        "error": err.to_string(),
                        "duration": duration.as_millis(),
                        "remote_addr": remote_addr
                    });

                    let log_manager = log_manager.clone();
                    let message = format!("HTTP error {} {} {}: {:?}", 500, method, path, err);
                    tokio::spawn(async move {
                        log_manager
                            .error(&message, "http", Some(tags), Some(context))
                            .await;
                    });
                }
            }

            result
        })
    }
}
