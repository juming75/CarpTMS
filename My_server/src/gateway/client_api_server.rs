//! / Client API 服务器模块
// 负责处理来自客户端(桌面、移动APP)的TCP连接和数据

use actix::prelude::*;
use log::{debug, error, info};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{Mutex, RwLock};

use crate::infrastructure::message_router::router::MessageRouter;
use crate::infrastructure::message_router::types::TcpDeviceMessage;
use crate::utils::jwt;

// 客户端状态 - 使用分离的读写半部
pub struct ClientApiClientState {
    pub client_id: String,
    pub addr: SocketAddr,
    pub token: String,
    pub user_id: Option<String>,
    pub last_activity: Mutex<Instant>,
    pub write_half: Arc<Mutex<tokio::net::tcp::OwnedWriteHalf>>,
    pub buffer: Mutex<Vec<u8>>,
    pub connection_time: Instant,
    pub total_data_received: Mutex<u64>,
    pub total_data_sent: Mutex<u64>,
    pub subscribed_topics: Mutex<Vec<String>>,
}

// 客户端API响应
#[derive(serde::Serialize)]
enum ClientApiResponse {
    #[serde(rename = "auth")]
    Auth {
        success: bool,
        user_id: Option<String>,
    },
    #[serde(rename = "subscribe")]
    Subscribe { success: bool, topics: Vec<String> },
    #[serde(rename = "unsubscribe")]
    Unsubscribe { success: bool, topics: Vec<String> },
    #[serde(rename = "publish")]
    Publish { success: bool, topic: String },
    #[serde(rename = "response")]
    Response {
        id: String,
        result: serde_json::Value,
    },
    #[serde(rename = "error")]
    Error { code: u32, message: String },
    #[serde(rename = "heartbeat")]
    Heartbeat { timestamp: u64 },
}

// Client API 服务器
pub struct ClientApiServer {
    addr: SocketAddr,
    listener: Option<TcpListener>,
    clients: Arc<RwLock<HashMap<String, Arc<ClientApiClientState>>>>,
    max_connections: usize,
    message_router: Option<Addr<MessageRouter>>,
    cleanup_interval: Duration,
    client_timeout: Duration,
    heartbeat_interval: Duration,
}

impl ClientApiServer {
    pub fn new(addr: SocketAddr, max_connections: usize) -> Self {
        Self {
            addr,
            listener: None,
            clients: Arc::new(RwLock::new(HashMap::new())),
            max_connections,
            message_router: None,
            cleanup_interval: Duration::from_secs(60),
            client_timeout: Duration::from_secs(300),
            heartbeat_interval: Duration::from_secs(30),
        }
    }

    // 设置消息路由器
    pub fn with_message_router(mut self, message_router: Addr<MessageRouter>) -> Self {
        self.message_router = Some(message_router);
        self
    }

    // 启动Client API服务器
    pub async fn start(&mut self) -> Result<(), std::io::Error> {
        info!("Starting Client API server on {}", self.addr);

        let listener = TcpListener::bind(self.addr).await?;
        self.listener = Some(listener);

        info!("Client API server started successfully on {}", self.addr);
        Ok(())
    }

    // 运行Client API服务器
    pub async fn run(&self) {
        let listener = self
            .listener
            .as_ref()
            .expect("Client API server not started");
        let clients = self.clients.clone();
        let max_connections = self.max_connections;
        let message_router = self.message_router.clone();
        let cleanup_interval = self.cleanup_interval;
        let client_timeout = self.client_timeout;
        let heartbeat_interval = self.heartbeat_interval;

        // 启动连接清理任务
        tokio::spawn({
            let clients = clients.clone();
            async move {
                loop {
                    tokio::time::sleep(cleanup_interval).await;
                    Self::cleanup_stale_connections(clients.clone(), client_timeout).await;
                }
            }
        });

        // 主监听循环
        loop {
            match listener.accept().await {
                Ok((stream, addr)) => {
                    info!("New Client API connection from {}", addr);

                    // 检查连接数是否超过上限
                    let current_connections = clients.read().await.len();
                    if current_connections >= max_connections {
                        error!(
                            "Maximum Client API connections reached, rejecting connection from {}",
                            addr
                        );
                        // 发送拒绝连接消息并关闭连接
                        let mut stream_clone = stream;
                        if let Err(e) = stream_clone.shutdown().await {
                            error!(
                                "Failed to shutdown Client API connection from {}: {}",
                                addr, e
                            );
                        }
                        continue;
                    }

                    // 处理新连接
                    let message_router_clone = message_router.clone();
                    let clients_clone = clients.clone();
                    let heartbeat_interval_clone = heartbeat_interval;

                    tokio::spawn(async move {
                        if let Err(e) = Self::handle_connection(
                            stream,
                            addr,
                            clients_clone,
                            message_router_clone,
                            heartbeat_interval_clone,
                        )
                        .await
                        {
                            error!(
                                "Failed to handle Client API connection from {}: {}",
                                addr, e
                            );
                        }
                    });
                }
                Err(e) => {
                    error!("Client API accept error: {}", e);
                    // 短暂休眠后继续尝试
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    continue;
                }
            }
        }
    }

    // 处理单个Client API连接
    async fn handle_connection(
        stream: TcpStream,
        addr: SocketAddr,
        clients: Arc<RwLock<HashMap<String, Arc<ClientApiClientState>>>>,
        message_router: Option<Addr<MessageRouter>>,
        heartbeat_interval: Duration,
    ) -> Result<(), std::io::Error> {
        // 分离读写半部
        let (mut read_half, write_half) = stream.into_split();

        // 生成客户端ID
        let client_id = format!("client_{}_{}", addr, chrono::Utc::now().timestamp_millis());

        // 创建客户端状态
        let client_state = Arc::new(ClientApiClientState {
            client_id: client_id.clone(),
            addr,
            token: String::new(),
            user_id: None,
            last_activity: Mutex::new(Instant::now()),
            write_half: Arc::new(Mutex::new(write_half)),
            buffer: Mutex::new(Vec::with_capacity(4096)),
            connection_time: Instant::now(),
            total_data_received: Mutex::new(0),
            total_data_sent: Mutex::new(0),
            subscribed_topics: Mutex::new(Vec::new()),
        });

        // 添加客户端到列表
        {
            let mut clients_write = clients.write().await;
            clients_write.insert(client_id.clone(), client_state.clone());
        }

        // 启动心跳任务
        let client_state_clone = client_state.clone();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(heartbeat_interval).await;

                let mut write_half_lock = client_state_clone.write_half.lock().await;
                let response = ClientApiResponse::Heartbeat {
                    timestamp: chrono::Utc::now().timestamp_millis() as u64,
                };

                let response_json = serde_json::to_string(&response).unwrap();
                if let Err(e) = write_half_lock
                    .write_all(format!("{}\n", response_json).as_bytes())
                    .await
                {
                    error!(
                        "Failed to send heartbeat to client {}: {}",
                        client_state_clone.client_id, e
                    );
                    break;
                }
            }
        });

        // 读取数据循环
        let mut read_buf = [0u8; 4096];
        let mut buffer = Vec::with_capacity(4096);

        loop {
            let n = match read_half.read(&mut read_buf).await {
                Ok(n) => n,
                Err(e) => {
                    error!("Client API read error from client {}: {}", client_id, e);
                    break;
                }
            };

            if n == 0 {
                // 连接关闭
                break;
            }

            buffer.extend_from_slice(&read_buf[..n]);
            debug!(
                "Client API received {} bytes from client {}: {:?}",
                n,
                client_id,
                &read_buf[..n]
            );

            // 更新客户端状态
            {
                let clients_read = clients.read().await;
                if let Some(client) = clients_read.get(&client_id) {
                    // 更新最后活动时间
                    *client.last_activity.lock().await = Instant::now();
                    // 更新数据接收统计
                    let mut buffer_lock = client.buffer.lock().await;
                    buffer_lock.extend_from_slice(&read_buf[..n]);
                    *client.total_data_received.lock().await += n as u64;
                }
            }

            // 处理完整的消息
            while let Some(message_str) = Self::read_message(&mut buffer) {
                match serde_json::from_str::<serde_json::Value>(&message_str) {
                    Ok(json) => {
                        Self::process_message(&client_id, &json, &clients, message_router.clone())
                            .await;
                    }
                    Err(e) => {
                        error!("Failed to parse Client API message: {}", e);
                        // 发送错误响应
                        let response = ClientApiResponse::Error {
                            code: 400,
                            message: format!("Invalid JSON: {}", e),
                        };
                        Self::send_response(&client_id, &clients, response).await;
                    }
                }
            }
        }

        // 清理客户端资源
        {
            let mut clients_write = clients.write().await;
            clients_write.remove(&client_id);
        }

        info!("Client API client {} disconnected from {}", client_id, addr);
        Ok(())
    }

    // 读取完整的消息(以换行符分隔)
    fn read_message(buffer: &mut Vec<u8>) -> Option<String> {
        if let Some(newline_pos) = buffer.iter().position(|&b| b == b'\n') {
            let message = String::from_utf8_lossy(&buffer[..newline_pos]).to_string();
            buffer.drain(0..=newline_pos);
            Some(message)
        } else {
            None
        }
    }

    // 处理客户端消息
    async fn process_message(
        client_id: &str,
        json: &serde_json::Value,
        clients: &Arc<RwLock<HashMap<String, Arc<ClientApiClientState>>>>,
        message_router: Option<Addr<MessageRouter>>,
    ) {
        let clients_read = clients.read().await;
        let client = match clients_read.get(client_id) {
            Some(client) => client,
            None => {
                error!("Client {} not found", client_id);
                return;
            }
        };

        if let Some(message_type) = json.get("type").and_then(|v| v.as_str()) {
            match message_type {
                "auth" => {
                    if let Some(token) = json.get("token").and_then(|v| v.as_str()) {
                        let result = jwt::verify_token(token);
                        match result {
                            Ok(claims) => {
                                let user_id = claims.sub;

                                // 更新客户端状态
                                let mut clients_write = clients.write().await;
                                if let Some(client) = clients_write.get_mut(client_id) {
                                    if let Some(c) = Arc::get_mut(client) {
                                        c.token = token.to_string();
                                        c.user_id = Some(user_id.clone());
                                    } else {
                                        log::warn!(
                                            "Could not get exclusive access to client Arc for {}",
                                            client_id
                                        );
                                    }
                                }

                                // 发送认证成功响应
                                let response = ClientApiResponse::Auth {
                                    success: true,
                                    user_id: Some(user_id),
                                };
                                Self::send_response(client_id, clients, response).await;
                            }
                            Err(_e) => {
                                // 发送认证失败响应
                                let response = ClientApiResponse::Auth {
                                    success: false,
                                    user_id: None,
                                };
                                Self::send_response(client_id, clients, response).await;
                            }
                        }
                    }
                }

                "subscribe" => {
                    if let Some(topics) = json.get("topics").and_then(|v| v.as_array()) {
                        let topics: Vec<String> = topics
                            .iter()
                            .filter_map(|v| v.as_str())
                            .map(|s| s.to_string())
                            .collect();

                        // 更新订阅主题
                        let mut subscribed_topics = client.subscribed_topics.lock().await;
                        subscribed_topics.extend(topics.clone());

                        // 发送订阅成功响应
                        let response = ClientApiResponse::Subscribe {
                            success: true,
                            topics,
                        };
                        Self::send_response(client_id, clients, response).await;
                    }
                }

                "unsubscribe" => {
                    if let Some(topics) = json.get("topics").and_then(|v| v.as_array()) {
                        let topics: Vec<String> = topics
                            .iter()
                            .filter_map(|v| v.as_str())
                            .map(|s| s.to_string())
                            .collect();

                        // 移除订阅主题
                        let mut subscribed_topics = client.subscribed_topics.lock().await;
                        for topic in &topics {
                            subscribed_topics.retain(|t| t != topic);
                        }

                        // 发送取消订阅成功响应
                        let response = ClientApiResponse::Unsubscribe {
                            success: true,
                            topics,
                        };
                        Self::send_response(client_id, clients, response).await;
                    }
                }

                "publish" => {
                    if let (Some(topic), Some(data)) =
                        (json.get("topic").and_then(|v| v.as_str()), json.get("data"))
                    {
                        // 发送消息到消息路由器
                        if let Some(router) = message_router {
                            // 创建统一消息
                            let unified_msg = crate::infrastructure::message_router::types::UnifiedMessage::new(
                                crate::infrastructure::message_router::types::MessageType::Data,
                                crate::infrastructure::message_router::types::MessageSource::Tcp,
                                Some(client_id.to_string()),
                                Some("client_publish".to_string()),
                                data.clone(),
                            );

                            router.do_send(TcpDeviceMessage {
                                device_id: client_id.to_string(),
                                protocol: "client_api".to_string(),
                                raw_data: topic.as_bytes().to_vec(),
                                unified_msg: Some(unified_msg),
                                received_at: chrono::Utc::now(),
                            });
                        }

                        // 发送发布成功响应
                        let response = ClientApiResponse::Publish {
                            success: true,
                            topic: topic.to_string(),
                        };
                        Self::send_response(client_id, clients, response).await;
                    }
                }

                "request" => {
                    if let (Some(id), Some(method), Some(_params)) = (
                        json.get("id").and_then(|v| v.as_str()),
                        json.get("method").and_then(|v| v.as_str()),
                        json.get("params"),
                    ) {
                        // 处理请求(这里是示例实现)
                        let result = match method {
                            "get_vehicles" => {
                                serde_json::json!({
                                    "vehicles": []
                                })
                            }
                            "get_users" => {
                                serde_json::json!({
                                    "users": []
                                })
                            }
                            _ => {
                                serde_json::json!({
                                    "error": "Method not found"
                                })
                            }
                        };

                        // 发送响应
                        let response = ClientApiResponse::Response {
                            id: id.to_string(),
                            result,
                        };
                        Self::send_response(client_id, clients, response).await;
                    }
                }

                "heartbeat" => {
                    // 发送心跳响应
                    let response = ClientApiResponse::Heartbeat {
                        timestamp: chrono::Utc::now().timestamp_millis() as u64,
                    };
                    Self::send_response(client_id, clients, response).await;
                }

                _ => {
                    // 发送错误响应
                    let response = ClientApiResponse::Error {
                        code: 400,
                        message: "Invalid message type".to_string(),
                    };
                    Self::send_response(client_id, clients, response).await;
                }
            }
        } else {
            // 发送错误响应
            let response = ClientApiResponse::Error {
                code: 400,
                message: "Missing message type".to_string(),
            };
            Self::send_response(client_id, clients, response).await;
        }
    }

    // 发送响应到客户端
    async fn send_response(
        client_id: &str,
        clients: &Arc<RwLock<HashMap<String, Arc<ClientApiClientState>>>>,
        response: ClientApiResponse,
    ) {
        let clients_read = clients.read().await;
        if let Some(client) = clients_read.get(client_id) {
            let response_json = serde_json::to_string(&response).unwrap();

            let mut write_half_lock = client.write_half.lock().await;
            if let Err(e) = write_half_lock
                .write_all(format!("{}\n", response_json).as_bytes())
                .await
            {
                error!("Failed to send response to client {}: {}", client_id, e);
            }
        }
    }

    // 清理过期连接
    async fn cleanup_stale_connections(
        clients: Arc<RwLock<HashMap<String, Arc<ClientApiClientState>>>>,
        timeout: Duration,
    ) {
        let now = Instant::now();
        let mut stale_client_ids = Vec::new();

        // 找出所有过期的连接
        {
            let clients_read = clients.read().await;
            for (client_id, client) in clients_read.iter() {
                if now.duration_since(*client.last_activity.lock().await) > timeout {
                    stale_client_ids.push(client_id.clone());
                }
            }
        }

        // 清理过期连接
        if !stale_client_ids.is_empty() {
            let mut clients_write = clients.write().await;
            for client_id in stale_client_ids {
                if let Some(client) = clients_write.remove(&client_id) {
                    info!(
                        "Cleaning up stale Client API connection for client {}",
                        client_id
                    );

                    // 关闭写入半部
                    let mut write_half_lock = client.write_half.lock().await;
                    if let Err(e) = write_half_lock.shutdown().await {
                        error!(
                            "Failed to shutdown stale Client API connection for client {}: {}",
                            client_id, e
                        );
                    }
                }
            }
        }
    }

    // 发送数据到指定客户端
    pub async fn send_data(&self, client_id: &str, data: &[u8]) -> Result<(), std::io::Error> {
        let clients_read = self.clients.read().await;
        if let Some(client) = clients_read.get(client_id) {
            // 获取客户端写入半部并发送数据
            let mut write_half_lock = client.write_half.lock().await;
            write_half_lock.write_all(data).await?;

            // 更新发送统计
            debug!(
                "Client API sent {} bytes to client {}",
                data.len(),
                client_id
            );
            Ok(())
        } else {
            error!("Client {} not found in Client API clients", client_id);
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Client not found",
            ))
        }
    }

    // 发布消息到订阅的客户端
    pub async fn publish_message(&self, topic: &str, data: &serde_json::Value) {
        let clients_read = self.clients.read().await;

        for (client_id, client) in clients_read.iter() {
            let subscribed_topics = client.subscribed_topics.lock().await;
            if subscribed_topics.contains(&topic.to_string()) {
                let message = serde_json::json!({
                    "type": "message",
                    "topic": topic,
                    "data": data,
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                });

                let message_str = message.to_string();
                let mut write_half_lock = client.write_half.lock().await;
                if let Err(e) = write_half_lock
                    .write_all(format!("{}\n", message_str).as_bytes())
                    .await
                {
                    error!("Failed to publish message to client {}: {}", client_id, e);
                }
            }
        }
    }
}

impl Actor for ClientApiServer {
    type Context = Context<Self>;

    // 启动时运行Client API服务器
    fn started(&mut self, _ctx: &mut Self::Context) {
        info!("Client API server actor started");

        // 直接在当前actor中启动Client API服务器
        let addr = self.addr;
        let clients = self.clients.clone();
        let max_connections = self.max_connections;
        let message_router = self.message_router.clone();
        let cleanup_interval = self.cleanup_interval;
        let client_timeout = self.client_timeout;
        let heartbeat_interval = self.heartbeat_interval;

        tokio::spawn(async move {
            // 启动TCP监听器
            let listener = match TcpListener::bind(addr).await {
                Ok(listener) => {
                    info!("Client API server started successfully on {}", addr);
                    listener
                }
                Err(e) => {
                    error!("Failed to bind Client API server on {}: {}", addr, e);
                    return;
                }
            };

            // 启动连接清理任务
            tokio::spawn({
                let clients = clients.clone();
                async move {
                    loop {
                        tokio::time::sleep(cleanup_interval).await;
                        ClientApiServer::cleanup_stale_connections(clients.clone(), client_timeout)
                            .await;
                    }
                }
            });

            // 主监听循环
            loop {
                match listener.accept().await {
                    Ok((stream, addr)) => {
                        info!("New Client API connection from {}", addr);

                        // 检查连接数是否超过上限
                        let current_connections = clients.read().await.len();
                        if current_connections >= max_connections {
                            error!(
                                "Maximum Client API connections reached, rejecting connection from {}",
                                addr
                            );
                            // 发送拒绝连接消息并关闭连接
                            let mut stream_clone = stream;
                            if let Err(e) = stream_clone.shutdown().await {
                                error!(
                                    "Failed to shutdown Client API connection from {}: {}",
                                    addr, e
                                );
                            }
                            continue;
                        }

                        // 处理新连接
                        let message_router_clone = message_router.clone();
                        let clients_clone = clients.clone();
                        let heartbeat_interval_clone = heartbeat_interval;

                        tokio::spawn(async move {
                            if let Err(e) = ClientApiServer::handle_connection(
                                stream,
                                addr,
                                clients_clone,
                                message_router_clone,
                                heartbeat_interval_clone,
                            )
                            .await
                            {
                                error!(
                                    "Failed to handle Client API connection from {}: {}",
                                    addr, e
                                );
                            }
                        });
                    }
                    Err(e) => {
                        error!("Client API accept error: {}", e);
                        // 短暂休眠后继续尝试
                        tokio::time::sleep(Duration::from_secs(1)).await;
                        continue;
                    }
                }
            }
        });
    }
}

// 发送Client API数据消息
#[derive(Message)]
#[rtype(result = "Result<(), std::io::Error>")]
pub struct SendClientApiData {
    pub client_id: String,
    pub data: Vec<u8>,
}

// 发布Client API消息
#[derive(Message)]
#[rtype(result = "()")]
pub struct PublishClientApiMessage {
    pub topic: String,
    pub data: serde_json::Value,
}

// 发送Client API数据消息处理
impl Handler<SendClientApiData> for ClientApiServer {
    type Result = ResponseFuture<Result<(), std::io::Error>>;

    fn handle(&mut self, msg: SendClientApiData, _ctx: &mut Self::Context) -> Self::Result {
        let client_id = msg.client_id;
        let data = msg.data;
        let clients = self.clients.clone();

        Box::pin(async move {
            let clients_read = clients.read().await;
            if let Some(client) = clients_read.get(&client_id) {
                // 获取客户端写入半部并发送数据
                let mut write_half_lock = client.write_half.lock().await;
                write_half_lock.write_all(&data).await?;
                debug!(
                    "Client API sent {} bytes to client {}: {:?}",
                    data.len(),
                    client_id,
                    data
                );
                Ok(())
            } else {
                error!("Client {} not found in Client API clients", client_id);
                Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Client not found",
                ))
            }
        })
    }
}

// 发布Client API消息处理
impl Handler<PublishClientApiMessage> for ClientApiServer {
    type Result = ResponseFuture<()>;

    fn handle(&mut self, msg: PublishClientApiMessage, _ctx: &mut Self::Context) -> Self::Result {
        let topic = msg.topic;
        let data = msg.data;
        let clients = self.clients.clone();

        Box::pin(async move {
            let clients_read = clients.read().await;

            for (client_id, client) in clients_read.iter() {
                let subscribed_topics = client.subscribed_topics.lock().await;
                if subscribed_topics.contains(&topic) {
                    let message = serde_json::json!({
                        "type": "message",
                        "topic": topic,
                        "data": data,
                        "timestamp": chrono::Utc::now().to_rfc3339(),
                    });

                    let message_str = message.to_string();
                    let mut write_half_lock = client.write_half.lock().await;
                    if let Err(e) = write_half_lock
                        .write_all(format!("{}\n", message_str).as_bytes())
                        .await
                    {
                        error!("Failed to publish message to client {}: {}", client_id, e);
                    }
                }
            }
        })
    }
}
