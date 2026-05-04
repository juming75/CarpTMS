//! / WebSocket 服务器模块
// 负责处理来自前端客户端的WebSocket连接和实时消息推送

use actix::{Actor, Addr, AsyncContext, Handler, Message, StreamHandler};
use actix_web::{web, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

// 导入 JWT 验证模块
use crate::utils::jwt::verify_token;

// 统一消息结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedMessage {
    pub msg_type: String,           // 消息类型:ping/pong/command/data/notification
    pub msg_id: Option<String>,     // 消息ID(用于请求响应匹配)
    pub device_id: Option<String>,  // 设备ID(如果是设备消息)
    pub command: Option<String>,    // 命令类型
    pub payload: serde_json::Value, // 消息内容
    pub timestamp: i64,             // 时间戳
}

impl UnifiedMessage {
    /// 将消息转换为JSON字符串
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// 从JSON字符串创建消息
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// 创建ping消息
    pub fn ping() -> Self {
        Self {
            msg_type: "ping".to_string(),
            msg_id: None,
            device_id: None,
            command: None,
            payload: serde_json::Value::Null,
            timestamp: chrono::Utc::now().timestamp(),
        }
    }

    /// 创建pong消息
    pub fn pong() -> Self {
        Self {
            msg_type: "pong".to_string(),
            msg_id: None,
            device_id: None,
            command: None,
            payload: serde_json::Value::Null,
            timestamp: chrono::Utc::now().timestamp(),
        }
    }
}

// Topic 订阅管理器
#[derive(Debug)]
pub struct TopicManager {
    // key: topic名称, value: 订阅该topic的client_id集合
    subscriptions: Arc<RwLock<HashMap<String, HashSet<String>>>>,
}

impl TopicManager {
    pub fn new() -> Self {
        Self {
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    // 订阅一个 topic
    pub async fn subscribe(&self, client_id: &str, topic: &str) {
        let mut subs = self.subscriptions.write().await;
        subs.entry(topic.to_string())
            .or_insert_with(HashSet::new)
            .insert(client_id.to_string());
        info!("Client {} subscribed to topic: {}", client_id, topic);
    }

    // 取消订阅一个 topic
    pub async fn unsubscribe(&self, client_id: &str, topic: &str) {
        let mut subs = self.subscriptions.write().await;
        if let Some(clients) = subs.get_mut(topic) {
            clients.remove(client_id);
            info!("Client {} unsubscribed from topic: {}", client_id, topic);

            // 如果没有客户端订阅该topic,删除该topic
            if clients.is_empty() {
                subs.remove(topic);
            }
        }
    }

    // 获取订阅某个 topic 的所有客户端
    pub async fn get_subscribers(&self, topic: &str) -> Vec<String> {
        let subs = self.subscriptions.read().await;
        if let Some(clients) = subs.get(topic) {
            clients.iter().cloned().collect()
        } else {
            Vec::new()
        }
    }

    // 检查客户端是否订阅了某个 topic
    pub async fn is_subscribed(&self, client_id: &str, topic: &str) -> bool {
        let subs = self.subscriptions.read().await;
        if let Some(clients) = subs.get(topic) {
            clients.contains(client_id)
        } else {
            false
        }
    }

    // 获取客户端订阅的所有 topic
    pub async fn get_client_topics(&self, client_id: &str) -> Vec<String> {
        let subs = self.subscriptions.read().await;
        subs.iter()
            .filter(|(_, clients)| clients.contains(client_id))
            .map(|(topic, _)| topic.clone())
            .collect()
    }

    // 客户端断开连接时,清理所有订阅
    pub async fn cleanup_client(&self, client_id: &str) {
        let mut subs = self.subscriptions.write().await;
        let topics: Vec<String> = subs
            .iter()
            .filter(|(_, clients)| clients.contains(client_id))
            .map(|(topic, _)| topic.clone())
            .collect();

        let topics_count = topics.len();

        for topic in &topics {
            if let Some(clients) = subs.get_mut(topic) {
                clients.remove(client_id);
                if clients.is_empty() {
                    subs.remove(topic);
                }
            }
        }
        info!(
            "Cleaned up {} topics for disconnected client: {}",
            topics_count, client_id
        );
    }

    // 获取所有 topic 和订阅数量
    pub async fn get_all_topics(&self) -> Vec<(String, usize)> {
        let subs = self.subscriptions.read().await;
        subs.iter()
            .map(|(topic, clients)| (topic.clone(), clients.len()))
            .collect()
    }
}

impl Default for TopicManager {
    fn default() -> Self {
        Self::new()
    }
}

// WebSocket 应用状态
pub struct WsAppState {
    pub topic_manager: Arc<TopicManager>,
    pub session_registry: Arc<WsSessionRegistry>,
}

impl WsAppState {
    pub fn new() -> Self {
        Self {
            topic_manager: Arc::new(TopicManager::new()),
            session_registry: Arc::new(WsSessionRegistry::new()),
        }
    }
}

impl Default for WsAppState {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for WsAppState {
    fn clone(&self) -> Self {
        Self {
            topic_manager: self.topic_manager.clone(),
            session_registry: self.session_registry.clone(),
        }
    }
}

// WebSocket 会话注册表
#[derive(Debug)]
pub struct WsSessionRegistry {
    // key: client_id, value: WebSocket Session Actor 地址
    sessions: Arc<RwLock<HashMap<String, Addr<WebSocketSession>>>>,
}

impl WsSessionRegistry {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    // 注册会话
    pub async fn register(&self, client_id: &str, addr: Addr<WebSocketSession>) {
        let mut sessions = self.sessions.write().await;
        sessions.insert(client_id.to_string(), addr);
        debug!("Registered WebSocket session: {}", client_id);
    }

    // 注销会话
    pub async fn unregister(&self, client_id: &str) {
        let mut sessions = self.sessions.write().await;
        sessions.remove(client_id);
        debug!("Unregistered WebSocket session: {}", client_id);
    }

    // 向指定客户端发送消息
    pub async fn send_to_client(&self, client_id: &str, message: &UnifiedMessage) -> bool {
        let sessions = self.sessions.read().await;
        if let Some(addr) = sessions.get(client_id) {
            let msg_json = match message.to_json() {
                Ok(json) => json,
                Err(e) => {
                    error!("Failed to serialize message: {}", e);
                    return false;
                }
            };

            addr.do_send(ClientMessage(msg_json));
            true
        } else {
            debug!("WebSocket session not found: {}", client_id);
            false
        }
    }

    // 广播消息到订阅指定 topic 的所有客户端
    pub async fn broadcast_to_topic(
        &self,
        topic_manager: Arc<TopicManager>,
        topic: &str,
        message: &UnifiedMessage,
    ) -> usize {
        let subscribers = topic_manager.get_subscribers(topic).await;

        if subscribers.is_empty() {
            debug!("No subscribers for topic: {}", topic);
            return 0;
        }

        let sessions = self.sessions.read().await;
        let msg_json = match message.to_json() {
            Ok(json) => json,
            Err(e) => {
                error!("Failed to serialize message: {}", e);
                return 0;
            }
        };

        let mut success_count = 0;
        for client_id in &subscribers {
            if let Some(addr) = sessions.get(client_id) {
                addr.do_send(ClientMessage(msg_json.clone()));
                success_count += 1;
            }
        }

        debug!(
            "Broadcasted message to topic: {}, sent to {} clients",
            topic, success_count
        );

        success_count
    }

    // 获取会话数量
    pub async fn session_count(&self) -> usize {
        self.sessions.read().await.len()
    }
}

impl Default for WsSessionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// 客户端消息(用于向 WebSocket 会话发送消息)
#[derive(Message)]
#[rtype(result = "()")]
pub struct ClientMessage(pub String);

// 辅助函数:发布消息到指定 topic
pub async fn publish_to_topic(
    registry: Arc<WsSessionRegistry>,
    topic_manager: Arc<TopicManager>,
    topic: &str,
    message: UnifiedMessage,
) {
    registry
        .broadcast_to_topic(topic_manager, topic, &message)
        .await;
}

// WebSocket 会话
pub struct WebSocketSession {
    pub client_id: String,
    pub hb: Instant,                              // 心跳时间
    pub user_id: Option<i32>,                     // 用户ID
    pub group_id: Option<i32>,                    // 组ID
    pub role: Option<String>,                     // 用户角色
    pub topics: HashSet<String>,                  // 订阅的 topic 列表
    pub topic_manager: Arc<TopicManager>,         // Topic 管理器引用
    pub session_registry: Arc<WsSessionRegistry>, // 会话注册表引用
    pub heartbeat_failure_count: u32,             // 心跳失败计数器
    pub max_heartbeat_failures: u32,              // 最大心跳失败次数
    pub ping_latency: Option<Duration>,           // 上次ping延迟
    pub connection_quality: f64,                  // 连接质量 (0.0-1.0)
    pub total_pings: u32,                         // 总ping次数
    pub successful_pongs: u32,                    // 成功pong次数
}

// 定义心跳检查间隔消息
struct HeartbeatCheck;

impl actix::Message for HeartbeatCheck {
    type Result = ();
}

impl WebSocketSession {
    pub fn new(topic_manager: Arc<TopicManager>, session_registry: Arc<WsSessionRegistry>) -> Self {
        Self {
            client_id: uuid::Uuid::new_v4().to_string(),
            hb: Instant::now(),
            user_id: None,
            group_id: None,
            role: None,
            topics: HashSet::new(),
            topic_manager,
            session_registry,
            heartbeat_failure_count: 0,
            max_heartbeat_failures: 3,
            ping_latency: None,
            connection_quality: 1.0,
            total_pings: 0,
            successful_pongs: 0,
        }
    }

    // 发送消息到客户端
    fn send_message(&self, ctx: &mut ws::WebsocketContext<Self>, message: UnifiedMessage) {
        if let Ok(json) = serde_json::to_string(&message) {
            ctx.text(json);
        }
    }

    // 发送 pong 响应
    fn send_pong(&self, ctx: &mut ws::WebsocketContext<Self>) {
        let pong_msg = UnifiedMessage {
            msg_type: "pong".to_string(),
            msg_id: None,
            device_id: None,
            command: None,
            payload: serde_json::json!({"timestamp": chrono::Utc::now().timestamp()}),
            timestamp: chrono::Utc::now().timestamp(),
        };
        self.send_message(ctx, pong_msg);
    }

    // 心跳检查方法 - 与客户端心跳配置保持一致
    fn heartbeat_check(&self, ctx: &mut ws::WebsocketContext<Self>) {
        // 如果超过60秒没有收到任何消息，主动发送协议级ping
        // 客户端每45秒发送应用级ping，服务端在1.33倍间隔后检查更合理
        if Instant::now().duration_since(self.hb) > Duration::from_secs(60) {
            info!(
                "Sending heartbeat ping to client {} (last activity: {}s ago)",
                self.client_id,
                Instant::now().duration_since(self.hb).as_secs()
            );
            ctx.ping(b"");
        }
    }
}

impl Actor for WebSocketSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        info!(
            "[WS] ✅ 客户端连接成功 - ClientID: {}, 时间: {}",
            self.client_id,
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
        );

        // 注册到会话注册表
        let addr = ctx.address();
        let client_id = self.client_id.clone();
        let session_registry = self.session_registry.clone();

        tokio::spawn(async move {
            session_registry.register(&client_id, addr).await;
            info!("[WS] 📝 会话注册成功 - ClientID: {}", client_id);
        });

        // 启动定时心跳检查 - 每45秒检查一次（与客户端pingInterval同步）
        ctx.run_interval(Duration::from_secs(45), |act, ctx| {
            act.heartbeat_check(ctx);
        });
        
        info!("[WS] ❤️ 心跳监控已启动 (间隔: 45s)");
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        let client_id = self.client_id.clone();
        warn!(
            "[WS] ⚠️ 客户端断开连接 - ClientID: {}, 运行时长统计: pings={}, pongs={}, quality={:.1}%",
            client_id,
            self.total_pings,
            self.successful_pongs,
            if self.total_pings > 0 {
                (self.successful_pongs as f64 / self.total_pings as f64) * 100.0
            } else {
                100.0
            }
        );

        // 清理该客户端的所有订阅
        let topic_manager = self.topic_manager.clone();
        let client_id_1 = self.client_id.clone();

        tokio::spawn(async move {
            topic_manager.cleanup_client(&client_id_1).await;
        });

        // 从会话注册表中移除
        let session_registry = self.session_registry.clone();
        let client_id_2 = self.client_id.clone();

        tokio::spawn(async move {
            session_registry.unregister(&client_id_2).await;
        });
    }
}

// 处理 WebSocket 消息
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WebSocketSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                // 响应 ping
                ctx.pong(&msg);
                self.hb = Instant::now();
            }
            Ok(ws::Message::Pong(_)) => {
                // 收到 pong,更新心跳时间
                self.hb = Instant::now();
                debug!("WebSocket client {} pong received", self.client_id);
            }
            Ok(ws::Message::Text(text)) => {
                // 处理文本消息
                self.hb = Instant::now();

                if let Ok(unified_msg) = serde_json::from_str::<UnifiedMessage>(&text) {
                    debug!(
                        "WebSocket client {} received message: {:?}",
                        self.client_id, unified_msg
                    );

                    match unified_msg.msg_type.as_str() {
                        "ping" => {
                            // 响应 ping
                            self.send_pong(ctx);
                        }
                        "pong" => {
                            // 更新心跳时间
                            self.hb = Instant::now();
                        }
                        "auth" => {
                            // 认证消息
                            self.handle_auth(&unified_msg, ctx);
                        }
                        "subscribe" => {
                            // 订阅消息
                            self.handle_subscribe(&unified_msg, ctx);
                        }
                        "unsubscribe" => {
                            // 取消订阅
                            self.handle_unsubscribe(&unified_msg, ctx);
                        }
                        "command" => {
                            // 命令消息
                            self.handle_command(&unified_msg, ctx);
                        }
                        _ => {
                            debug!("Unknown message type: {}", unified_msg.msg_type);
                        }
                    }
                } else {
                    error!("Failed to parse WebSocket message: {}", text);
                }
            }
            Ok(ws::Message::Close(reason)) => {
                info!("WebSocket client {} closing: {:?}", self.client_id, reason);
                ctx.close(reason);
            }
            Err(e) => {
                error!("WebSocket protocol error: {}", e);
                ctx.close(None);
            }
            _ => {}
        }
    }
}

impl Handler<ClientMessage> for WebSocketSession {
    type Result = ();

    fn handle(&mut self, msg: ClientMessage, ctx: &mut Self::Context) {
        debug!(
            "Sending message to WebSocket client {}: {}",
            self.client_id, msg.0
        );
        ctx.text(msg.0);
    }
}

impl Handler<HeartbeatCheck> for WebSocketSession {
    type Result = ();

    fn handle(&mut self, _msg: HeartbeatCheck, ctx: &mut Self::Context) {
        self.heartbeat_check(ctx);
    }
}

impl WebSocketSession {
    // 处理认证消息
    fn handle_auth(&mut self, msg: &UnifiedMessage, ctx: &mut ws::WebsocketContext<Self>) {
        // 从 payload 中提取 token
        if let Some(token) = msg.payload.get("token") {
            if let Some(token_str) = token.as_str() {
                debug!("Auth request with token: {}", token_str);

                // 验证 JWT token
                match verify_token(token_str) {
                    Ok(claims) => {
                        // 认证成功,保存用户信息
                        // 将 claims.sub 解析为 user_id
                        self.user_id = claims.sub.parse::<i32>().ok();
                        self.role = Some(claims.role);
                        self.group_id = Some(claims.group_id);

                        info!(
                            "WebSocket client {} authenticated successfully: user_id={}, role={}, group_id={}",
                            self.client_id,
                            self.user_id.unwrap_or(0),
                            self.role.as_ref().unwrap_or(&"unknown".to_string()),
                            self.group_id.unwrap_or(0)
                        );

                        // 发送认证成功响应
                        let response = UnifiedMessage {
                            msg_type: "auth_response".to_string(),
                            msg_id: msg.msg_id.clone(),
                            device_id: None,
                            command: None,
                            payload: serde_json::json!({
                                "success": true,
                                "message": "Authentication successful",
                                "user_id": self.user_id,
                                "role": self.role,
                                "group_id": self.group_id
                            }),
                            timestamp: chrono::Utc::now().timestamp(),
                        };
                        self.send_message(ctx, response);
                    }
                    Err(e) => {
                        // 认证失败
                        error!("JWT verification failed: {}", e);

                        let response = UnifiedMessage {
                            msg_type: "auth_response".to_string(),
                            msg_id: msg.msg_id.clone(),
                            device_id: None,
                            command: None,
                            payload: serde_json::json!({
                                "success": false,
                                "message": format!("Authentication failed: {}", e)
                            }),
                            timestamp: chrono::Utc::now().timestamp(),
                        };
                        self.send_message(ctx, response);
                    }
                }
            }
        }
    }

    // 处理订阅消息
    fn handle_subscribe(&mut self, msg: &UnifiedMessage, ctx: &mut ws::WebsocketContext<Self>) {
        // 从 payload 中提取订阅类型
        if let Some(topic) = msg.payload.get("topic") {
            if let Some(topic_str) = topic.as_str() {
                debug!(
                    "Client {} subscribing to topic: {}",
                    self.client_id, topic_str
                );

                // 添加到本地订阅列表
                self.topics.insert(topic_str.to_string());

                // 更新 TopicManager(异步操作,使用 spawn)
                let topic_manager = self.topic_manager.clone();
                let client_id = self.client_id.clone();
                let topic_owned = topic_str.to_string();

                tokio::spawn(async move {
                    topic_manager.subscribe(&client_id, &topic_owned).await;
                });

                // 发送响应
                let response = UnifiedMessage {
                    msg_type: "subscribe_response".to_string(),
                    msg_id: msg.msg_id.clone(),
                    device_id: None,
                    command: None,
                    payload: serde_json::json!({
                        "success": true,
                        "topic": topic_str,
                        "message": format!("Successfully subscribed to {}", topic_str)
                    }),
                    timestamp: chrono::Utc::now().timestamp(),
                };
                self.send_message(ctx, response);
            } else {
                // topic 参数格式错误
                let response = UnifiedMessage {
                    msg_type: "subscribe_response".to_string(),
                    msg_id: msg.msg_id.clone(),
                    device_id: None,
                    command: None,
                    payload: serde_json::json!({
                        "success": false,
                        "message": "Invalid topic format"
                    }),
                    timestamp: chrono::Utc::now().timestamp(),
                };
                self.send_message(ctx, response);
            }
        } else {
            // 缺少 topic 参数
            let response = UnifiedMessage {
                msg_type: "subscribe_response".to_string(),
                msg_id: msg.msg_id.clone(),
                device_id: None,
                command: None,
                payload: serde_json::json!({
                    "success": false,
                    "message": "Missing 'topic' parameter"
                }),
                timestamp: chrono::Utc::now().timestamp(),
            };
            self.send_message(ctx, response);
        }
    }

    // 处理取消订阅消息
    fn handle_unsubscribe(&mut self, msg: &UnifiedMessage, ctx: &mut ws::WebsocketContext<Self>) {
        if let Some(topic) = msg.payload.get("topic") {
            if let Some(topic_str) = topic.as_str() {
                debug!(
                    "Client {} unsubscribing from topic: {}",
                    self.client_id, topic_str
                );

                // 从本地订阅列表移除
                self.topics.remove(topic_str);

                // 更新 TopicManager(异步操作,使用 spawn)
                let topic_manager = self.topic_manager.clone();
                let client_id = self.client_id.clone();
                let topic_owned = topic_str.to_string();

                tokio::spawn(async move {
                    topic_manager.unsubscribe(&client_id, &topic_owned).await;
                });

                // 发送响应
                let response = UnifiedMessage {
                    msg_type: "unsubscribe_response".to_string(),
                    msg_id: msg.msg_id.clone(),
                    device_id: None,
                    command: None,
                    payload: serde_json::json!({
                        "success": true,
                        "topic": topic_str,
                        "message": format!("Successfully unsubscribed from {}", topic_str)
                    }),
                    timestamp: chrono::Utc::now().timestamp(),
                };
                self.send_message(ctx, response);
            } else {
                // topic 参数格式错误
                let response = UnifiedMessage {
                    msg_type: "unsubscribe_response".to_string(),
                    msg_id: msg.msg_id.clone(),
                    device_id: None,
                    command: None,
                    payload: serde_json::json!({
                        "success": false,
                        "message": "Invalid topic format"
                    }),
                    timestamp: chrono::Utc::now().timestamp(),
                };
                self.send_message(ctx, response);
            }
        } else {
            // 缺少 topic 参数
            let response = UnifiedMessage {
                msg_type: "unsubscribe_response".to_string(),
                msg_id: msg.msg_id.clone(),
                device_id: None,
                command: None,
                payload: serde_json::json!({
                    "success": false,
                    "message": "Missing 'topic' parameter"
                }),
                timestamp: chrono::Utc::now().timestamp(),
            };
            self.send_message(ctx, response);
        }
    }

    // 处理命令消息
    fn handle_command(&mut self, msg: &UnifiedMessage, ctx: &mut ws::WebsocketContext<Self>) {
        debug!("Handle command: {:?}", msg.command);

        let response = UnifiedMessage {
            msg_type: "command_response".to_string(),
            msg_id: msg.msg_id.clone(),
            device_id: None,
            command: msg.command.clone(),
            payload: serde_json::json!({
                "success": true,
                "message": "Command received"
            }),
            timestamp: chrono::Utc::now().timestamp(),
        };
        self.send_message(ctx, response);
    }
}

// HTTP 路由处理器(增强版 - 详细日志和错误处理)
pub async fn websocket_index_route(
    req: HttpRequest,
    stream: web::Payload,
    data: web::Data<Arc<WsAppState>>,
) -> Result<HttpResponse, actix_web::Error> {
    // 记录详细的连接信息
    let peer_addr = req.peer_addr().map(|a| a.to_string()).unwrap_or_else(|| "unknown".to_string());
    let user_agent = req.headers()
        .get("User-Agent")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown");
    let origin = req.headers()
        .get("Origin")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("none");
    
    info!(
        "[WS] 📥 新连接请求 - IP: {}, Origin: {}, UserAgent: {}",
        peer_addr, origin, user_agent
    );

    // 检查当前连接数（用于监控）
    let current_connections = data.get_ref().session_registry.session_count().await;
    info!("[WS] 📊 当前活跃连接数: {}", current_connections);

    // 尝试创建会话并启动 WebSocket 连接
    let session = WebSocketSession::new(
        data.get_ref().topic_manager.clone(),
        data.get_ref().session_registry.clone(),
    );

    let client_id_for_log = session.client_id.clone();

    info!(
        "[WS] 🔧 创建新会话 - ClientID: {}",
        client_id_for_log
    );

    // 启动 WebSocket 连接，添加详细错误处理
    match ws::start(session, &req, stream) {
        Ok(resp) => {
            info!(
                "[WS] ✅ 连接建立成功 - ClientID: {}, IP: {}",
                client_id_for_log,
                peer_addr
            );
            Ok(resp)
        }
        Err(e) => {
            error!(
                "[WS] ❌ 连接建立失败 - IP: {}, Error: {}",
                peer_addr, e
            );
            
            // 返回适当的错误响应
            Err(actix_web::error::ErrorInternalServerError(format!(
                "WebSocket connection failed: {}", e
            )))
        }
    }
}
