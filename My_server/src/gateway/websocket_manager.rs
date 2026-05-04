//! /! WebSocket 连接管理器
//!
//! 提供 WebSocket 连接的管理、监控和优化功能

use actix::Addr;
use log::{error, info, warn};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

use crate::gateway::websocket_server::{UnifiedMessage, WebSocketSession};

/// 连接映射类型
pub type ConnectionMap = Arc<RwLock<HashMap<String, (Addr<WebSocketSession>, ConnectionInfo)>>>;

/// WebSocket 连接管理器配置
#[derive(Debug, Clone)]
pub struct WebSocketManagerConfig {
    /// 最大连接数
    pub max_connections: usize,
    /// 心跳超时时间
    pub heartbeat_timeout: Duration,
    /// 清理间隔
    pub cleanup_interval: Duration,
    /// 最大消息大小(字节)
    pub max_message_size: usize,
    /// 每个客户端最大连接数
    pub max_connections_per_client: usize,
    /// 连接池大小
    pub connection_pool_size: usize,
    /// 消息队列大小
    pub message_queue_size: usize,
}

impl Default for WebSocketManagerConfig {
    fn default() -> Self {
        Self {
            max_connections: 1000,
            heartbeat_timeout: Duration::from_secs(60),
            cleanup_interval: Duration::from_secs(30),
            max_message_size: 1024 * 1024, // 1MB
            max_connections_per_client: 5, // 每个客户端最大5个连接
            connection_pool_size: 100,     // 连接池大小
            message_queue_size: 1000,      // 消息队列大小
        }
    }
}

/// WebSocket 连接信息
#[derive(Debug, Clone)]
pub struct ConnectionInfo {
    pub client_id: String,
    pub connected_at: Instant,
    pub last_heartbeat: Instant,
    pub user_id: Option<i32>,
    pub role: Option<String>,
    pub topics: HashSet<String>,
}

/// WebSocket 连接管理器
pub struct WebSocketManager {
    config: WebSocketManagerConfig,
    connections: ConnectionMap,
    is_running: Arc<Mutex<bool>>,
    connection_count: Arc<Mutex<usize>>,
    client_connections: Arc<RwLock<HashMap<String, usize>>>, // 客户端连接计数
}

impl WebSocketManager {
    /// 创建新的 WebSocket 连接管理器
    pub fn new(config: WebSocketManagerConfig) -> Self {
        Self {
            config,
            connections: Arc::new(RwLock::new(HashMap::new())),
            is_running: Arc::new(Mutex::new(false)),
            connection_count: Arc::new(Mutex::new(0)),
            client_connections: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 注册新连接
    pub async fn register_connection(
        &self,
        client_id: &str,
        addr: Addr<WebSocketSession>,
        user_id: Option<i32>,
        role: Option<String>,
    ) -> Result<(), String> {
        // 检查连接数限制
        let current_count = *self.connection_count.lock().unwrap_or_else(|p| {
            log::error!("WebSocket manager mutex was poisoned, recovering: {}", p);
            p.into_inner()
        });
        if current_count >= self.config.max_connections {
            return Err(format!(
                "Connection limit reached: {}/{}",
                current_count, self.config.max_connections
            ));
        }

        // 检查每个客户端的连接数限制
        let mut client_connections = self.client_connections.write().await;
        let client_count = client_connections.get(client_id).unwrap_or(&0);
        if *client_count >= self.config.max_connections_per_client {
            return Err(format!(
                "Connection limit per client reached: {}/{}",
                client_count, self.config.max_connections_per_client
            ));
        }

        let connection_info = ConnectionInfo {
            client_id: client_id.to_string(),
            connected_at: Instant::now(),
            last_heartbeat: Instant::now(),
            user_id,
            role,
            topics: HashSet::new(),
        };

        let mut connections = self.connections.write().await;
        connections.insert(client_id.to_string(), (addr, connection_info));
        {
            let mut guard = self.connection_count.lock().unwrap_or_else(|p| {
                log::error!("WebSocket manager mutex was poisoned, recovering: {}", p);
                p.into_inner()
            });
            *guard += 1;
        }

        // 更新客户端连接计数
        *client_connections.entry(client_id.to_string()).or_insert(0) += 1;

        info!(
            "WebSocket connection registered: {}, total connections: {}",
            client_id,
            *self.connection_count.lock().unwrap_or_else(|p| {
                log::error!("WebSocket manager mutex was poisoned, recovering: {}", p);
                p.into_inner()
            })
        );

        Ok(())
    }

    /// 注销连接
    pub async fn unregister_connection(&self, client_id: &str) {
        let mut connections = self.connections.write().await;
        if connections.remove(client_id).is_some() {
            {
                let mut guard = self.connection_count.lock().unwrap_or_else(|p| {
                    log::error!("WebSocket manager mutex was poisoned, recovering: {}", p);
                    p.into_inner()
                });
                *guard -= 1;
            }

            // 更新客户端连接计数
            let mut client_connections = self.client_connections.write().await;
            if let Some(count) = client_connections.get_mut(client_id) {
                *count -= 1;
                if *count == 0 {
                    client_connections.remove(client_id);
                }
            }

            info!(
                "WebSocket connection unregistered: {}, total connections: {}",
                client_id,
                *self.connection_count.lock().unwrap_or_else(|p| {
                    log::error!("WebSocket manager mutex was poisoned, recovering: {}", p);
                    p.into_inner()
                })
            );
        }
    }

    /// 更新心跳时间
    pub async fn update_heartbeat(&self, client_id: &str) {
        let mut connections = self.connections.write().await;
        if let Some((_, info)) = connections.get_mut(client_id) {
            info.last_heartbeat = Instant::now();
        }
    }

    /// 添加订阅
    pub async fn add_subscription(&self, client_id: &str, topic: &str) {
        let mut connections = self.connections.write().await;
        if let Some((_, info)) = connections.get_mut(client_id) {
            info.topics.insert(topic.to_string());
        }
    }

    /// 移除订阅
    pub async fn remove_subscription(&self, client_id: &str, topic: &str) {
        let mut connections = self.connections.write().await;
        if let Some((_, info)) = connections.get_mut(client_id) {
            info.topics.remove(topic);
        }
    }

    /// 清理超时连接
    pub async fn cleanup_timeout_connections(&self) {
        let now = Instant::now();
        let mut connections = self.connections.write().await;
        let timeout = self.config.heartbeat_timeout;

        let mut to_remove: Vec<String> = Vec::new();

        for (client_id, (_, info)) in connections.iter() {
            if now.duration_since(info.last_heartbeat) > timeout {
                warn!(
                    "WebSocket connection timeout: {}, last heartbeat: {:?}",
                    client_id, info.last_heartbeat
                );
                to_remove.push(client_id.clone());
            }
        }

        for client_id in to_remove {
            connections.remove(&client_id);
            {
                let mut guard = self.connection_count.lock().unwrap_or_else(|p| {
                    log::error!("WebSocket manager mutex was poisoned, recovering: {}", p);
                    p.into_inner()
                });
                *guard -= 1;
            }

            // 更新客户端连接计数
            let mut client_connections = self.client_connections.write().await;
            if let Some(count) = client_connections.get_mut(&client_id) {
                *count -= 1;
                if *count == 0 {
                    client_connections.remove(&client_id);
                }
            }

            info!(
                "Cleaned up timeout connection: {}, total connections: {}",
                client_id,
                *self.connection_count.lock().unwrap_or_else(|p| {
                    log::error!("WebSocket manager mutex was poisoned, recovering: {}", p);
                    p.into_inner()
                })
            );
        }
    }

    /// 启动连接管理器
    pub fn start(&self) {
        let mut running = self.is_running.lock().unwrap_or_else(|p| {
            log::error!("WebSocket manager mutex was poisoned, recovering: {}", p);
            p.into_inner()
        });
        if *running {
            warn!("WebSocket manager is already running");
            return;
        }
        *running = true;
        drop(running);

        let connections = self.connections.clone();
        let config = self.config.clone();
        let is_running = self.is_running.clone();
        let connection_count = self.connection_count.clone();
        let client_connections = self.client_connections.clone();

        tokio::spawn(async move {
            loop {
                if !*is_running.lock().unwrap_or_else(|p| {
                    log::error!("WebSocket manager mutex was poisoned, recovering: {}", p);
                    p.into_inner()
                }) {
                    break;
                }

                // 清理超时连接
                let now = Instant::now();
                let mut conns = connections.write().await;
                let timeout = config.heartbeat_timeout;

                let mut to_remove: Vec<String> = Vec::new();

                for (client_id, (_, info)) in conns.iter() {
                    if now.duration_since(info.last_heartbeat) > timeout {
                        warn!(
                            "WebSocket connection timeout: {}, last heartbeat: {:?}",
                            client_id, info.last_heartbeat
                        );
                        to_remove.push(client_id.clone());
                    }
                }

                for client_id in to_remove {
                    conns.remove(&client_id);
                    {
                        let mut guard = connection_count.lock().unwrap_or_else(|p| {
                            log::error!("WebSocket manager mutex was poisoned, recovering: {}", p);
                            p.into_inner()
                        });
                        *guard -= 1;
                    }

                    // 更新客户端连接计数
                    let mut client_conns = client_connections.write().await;
                    if let Some(count) = client_conns.get_mut(&client_id) {
                        *count -= 1;
                        if *count == 0 {
                            client_conns.remove(&client_id);
                        }
                    }

                    info!(
                        "Cleaned up timeout connection: {}, total connections: {}",
                        client_id,
                        *connection_count.lock().unwrap_or_else(|p| {
                            log::error!("WebSocket manager mutex was poisoned, recovering: {}", p);
                            p.into_inner()
                        })
                    );
                }

                // 等待下一次清理
                tokio::time::sleep(config.cleanup_interval).await;
            }
        });

        info!("WebSocket manager started with config: {:?}", self.config);
    }

    /// 停止连接管理器
    pub fn stop(&self) {
        let mut running = self.is_running.lock().unwrap_or_else(|p| {
            log::error!("WebSocket manager mutex was poisoned, recovering: {}", p);
            p.into_inner()
        });
        *running = false;
        drop(running);
        info!("WebSocket manager stopped");
    }

    /// 获取当前连接数
    pub async fn get_connection_count(&self) -> usize {
        *self.connection_count.lock().unwrap_or_else(|p| {
            log::error!("WebSocket manager mutex was poisoned, recovering: {}", p);
            p.into_inner()
        })
    }

    /// 获取连接信息
    pub async fn get_connection_info(&self, client_id: &str) -> Option<ConnectionInfo> {
        let connections = self.connections.read().await;
        connections.get(client_id).map(|(_, info)| info.clone())
    }

    /// 获取所有连接信息
    pub async fn get_all_connections(&self) -> Vec<ConnectionInfo> {
        let connections = self.connections.read().await;
        connections.values().map(|(_, info)| info.clone()).collect()
    }

    /// 向所有连接广播消息
    pub async fn broadcast_message(&self, message: &UnifiedMessage) {
        let connections = self.connections.read().await;
        let msg_json = match message.to_json() {
            Ok(json) => json,
            Err(e) => {
                error!("Failed to serialize message: {}", e);
                return;
            }
        };

        for (_, (addr, _)) in connections.iter() {
            let _ = addr.try_send(crate::gateway::websocket_server::ClientMessage(
                msg_json.clone(),
            ));
        }

        info!("Broadcasted message to {} connections", connections.len());
    }
}

/// WebSocket 连接限制中间件
pub struct WebSocketConnectionLimitMiddleware {
    manager: Arc<WebSocketManager>,
}

impl WebSocketConnectionLimitMiddleware {
    pub fn new(manager: Arc<WebSocketManager>) -> Self {
        Self { manager }
    }

    /// 检查连接是否允许
    pub async fn check_connection(&self) -> Result<(), String> {
        let current_count = self.manager.get_connection_count().await;
        if current_count >= self.manager.config.max_connections {
            return Err(format!(
                "Connection limit reached: {}/{}",
                current_count, self.manager.config.max_connections
            ));
        }
        Ok(())
    }
}
