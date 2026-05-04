//! / WebSocket视频推送模块
// 通过WebSocket实时推送视频帧到客户端
#![allow(dead_code)]

use actix::{Actor, ActorContext, AsyncContext, StreamHandler};
use actix_web::{web, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::{video_manager::StreamError, VideoFrame, VideoFrameType, VideoStreamManager};
use log::error;

/// WebSocket消息类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum WsMessage {
    /// 订阅视频流
    #[serde(rename = "subscribe")]
    Subscribe { stream_id: String },
    /// 取消订阅
    #[serde(rename = "unsubscribe")]
    Unsubscribe { stream_id: String },
    /// 视频帧数据(二进制)
    #[serde(rename = "frame")]
    VideoFrame {
        stream_id: String,
        frame_type: VideoFrameType,
        timestamp: u64,
        sequence: u32,
    },
    /// 心跳
    #[serde(rename = "ping")]
    Ping,
    /// 错误消息
    #[serde(rename = "error")]
    Error { message: String },
}

/// WebSocket会话信息
#[derive(Debug, Clone)]
struct WsSessionInfo {
    /// 客户端ID
    client_id: String,
    /// 订阅的流ID列表
    subscribed_streams: Vec<String>,
    /// 最后活跃时间
    last_active: std::time::Instant,
}

/// WebSocket视频推送处理器
pub struct VideoWsHandler {
    /// 视频流管理器
    stream_manager: Arc<VideoStreamManager>,
    /// 客户端ID
    client_id: String,
    /// 会话信息
    session: WsSessionInfo,
}

impl VideoWsHandler {
    /// 创建新的WebSocket处理器
    pub fn new(stream_manager: Arc<VideoStreamManager>) -> Self {
        let client_id = format!("client_{}", uuid::Uuid::new_v4());
        let session = WsSessionInfo {
            client_id: client_id.clone(),
            subscribed_streams: Vec::new(),
            last_active: std::time::Instant::now(),
        };

        Self {
            stream_manager,
            client_id,
            session,
        }
    }

    /// 处理订阅请求
    async fn handle_subscribe(&mut self, stream_id: &str) -> Result<(), StreamError> {
        // 检查流是否存在
        if !self.stream_manager.stream_exists(stream_id).await {
            return Err(StreamError::StreamNotFound(stream_id.to_string()));
        }

        // 添加到订阅列表
        if !self
            .session
            .subscribed_streams
            .contains(&stream_id.to_string())
        {
            self.session.subscribed_streams.push(stream_id.to_string());

            // 订阅流
            let _rx = self
                .stream_manager
                .subscribe_stream(self.client_id.clone(), stream_id.to_string())
                .await?;
        }

        Ok(())
    }

    /// 处理取消订阅请求
    async fn handle_unsubscribe(&mut self, stream_id: &str) -> Result<(), StreamError> {
        // 从订阅列表中移除
        self.session.subscribed_streams.retain(|s| s != stream_id);

        // 取消订阅流
        self.stream_manager
            .unsubscribe_stream(&self.client_id, stream_id)
            .await;

        Ok(())
    }

    /// 发送错误消息
    fn send_error(&mut self, ctx: &mut <Self as Actor>::Context, message: String) {
        let msg = WsMessage::Error { message };
        if let Ok(json) = serde_json::to_string(&msg) {
            ctx.text(json);
        }
    }
}

impl Actor for VideoWsHandler {
    type Context = ws::WebsocketContext<Self>;
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for VideoWsHandler {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                ctx.pong(&msg);
            }
            Ok(ws::Message::Pong(_)) => {
                // 心跳响应
            }
            Ok(ws::Message::Text(text)) => {
                // 解析文本消息
                if let Ok(ws_msg) = serde_json::from_str::<WsMessage>(&text) {
                    match ws_msg {
                        WsMessage::Subscribe { stream_id } => {
                            let stream_manager = self.stream_manager.clone();
                            let stream_id_clone = stream_id.clone();
                            let client_id = self.client_id.clone();
                            let _ctx_addr = ctx.address();

                            tokio::spawn(async move {
                                match stream_manager
                                    .subscribe_stream(client_id.clone(), stream_id_clone)
                                    .await
                                {
                                    Ok(_) => {
                                        // 订阅成功
                                    }
                                    Err(e) => {
                                        // 发送错误消息
                                        let _error_msg = format!("订阅失败: {}", e);
                                    }
                                }
                            });
                        }
                        WsMessage::Unsubscribe { stream_id } => {
                            let stream_manager = self.stream_manager.clone();
                            let stream_id_clone = stream_id.clone();

                            tokio::spawn(async move {
                                let _ = stream_manager
                                    .unsubscribe_stream(&stream_id_clone, &stream_id_clone)
                                    .await;
                            });

                            // 从本地订阅列表中移除
                            self.session.subscribed_streams.retain(|s| s != &stream_id);
                        }
                        WsMessage::Ping => {
                            // 响应心跳
                            match serde_json::to_string(&WsMessage::Ping) {
                                Ok(msg) => ctx.text(msg),
                                Err(e) => {
                                    error!("Failed to serialize ping message: {}", e);
                                    ctx.text(r#"{"type":"ping"}"#); // 使用固定回退
                                }
                            }
                        }
                        _ => {
                            self.send_error(ctx, "不支持的消息类型".to_string());
                        }
                    }
                } else {
                    self.send_error(ctx, "无效的JSON格式".to_string());
                }

                // 更新活跃时间
                self.session.last_active = std::time::Instant::now();
            }
            Ok(ws::Message::Binary(_data)) => {
                // 不接受客户端二进制数据
                self.send_error(ctx, "不支持发送二进制数据".to_string());
            }
            Ok(ws::Message::Close(reason)) => {
                // 清理所有订阅
                let stream_manager = self.stream_manager.clone();
                let client_id = self.client_id.clone();
                let streams = self.session.subscribed_streams.clone();

                tokio::spawn(async move {
                    for stream_id in streams {
                        let _ = stream_manager
                            .unsubscribe_stream(&stream_id, &client_id)
                            .await;
                    }
                });

                ctx.close(reason);
                ctx.stop();
            }
            _ => {}
        }
    }
}

/// 视频帧分发器
#[derive(Clone)]
pub struct VideoFrameDistributor {
    /// 订阅信息: stream_id -> client_id集合
    subscriptions: Arc<RwLock<HashMap<String, Vec<String>>>>,
    /// 客户端连接: client_id -> WebSocket地址
    clients: Arc<RwLock<HashMap<String, actix::Addr<VideoWsHandler>>>>,
}

impl VideoFrameDistributor {
    /// 创建新的分发器
    pub fn new() -> Self {
        Self {
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
            clients: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 注册客户端
    pub async fn register_client(&self, client_id: String, addr: actix::Addr<VideoWsHandler>) {
        let mut clients = self.clients.write().await;
        clients.insert(client_id, addr);
    }

    /// 取消注册客户端
    pub async fn unregister_client(&self, client_id: &str) {
        let mut clients = self.clients.write().await;
        clients.remove(client_id);

        // 从所有订阅中移除该客户端
        let mut subscriptions = self.subscriptions.write().await;
        for streams in subscriptions.values_mut() {
            streams.retain(|id| id != client_id);
        }
    }

    /// 添加订阅
    pub async fn add_subscription(&self, stream_id: String, client_id: String) {
        let mut subscriptions = self.subscriptions.write().await;
        subscriptions.entry(stream_id).or_default().push(client_id);
    }

    /// 移除订阅
    pub async fn remove_subscription(&self, stream_id: &str, client_id: &str) {
        let mut subscriptions = self.subscriptions.write().await;
        if let Some(streams) = subscriptions.get_mut(stream_id) {
            streams.retain(|id| id != client_id);
        }
    }

    /// 分发视频帧
    pub async fn distribute_frame(&self, stream_id: &str, frame: VideoFrame) {
        let subscriptions = self.subscriptions.read().await;
        if let Some(client_ids) = subscriptions.get(stream_id) {
            let clients = self.clients.read().await;

            for client_id in client_ids {
                if let Some(addr) = clients.get(client_id) {
                    // 发送帧元数据
                    let meta_msg = WsMessage::VideoFrame {
                        stream_id: stream_id.to_string(),
                        frame_type: frame.frame_type,
                        timestamp: frame.timestamp,
                        sequence: frame.sequence,
                    };

                    if let Ok(json) = serde_json::to_string(&meta_msg) {
                        addr.do_send(WsTextMessage(json));
                    }

                    // 发送帧数据(二进制)
                    addr.do_send(WsBinaryMessage(frame.data.clone()));
                }
            }
        }
    }
}

impl Default for VideoFrameDistributor {
    fn default() -> Self {
        Self::new()
    }
}

/// WebSocket文本消息
pub struct WsTextMessage(pub String);

impl actix::Message for WsTextMessage {
    type Result = ();
}

impl actix::Handler<WsTextMessage> for VideoWsHandler {
    type Result = ();

    fn handle(&mut self, msg: WsTextMessage, ctx: &mut Self::Context) -> Self::Result {
        ctx.text(msg.0);
    }
}

/// WebSocket二进制消息
pub struct WsBinaryMessage(pub Bytes);

impl actix::Message for WsBinaryMessage {
    type Result = ();
}

impl actix::Handler<WsBinaryMessage> for VideoWsHandler {
    type Result = ();

    fn handle(&mut self, msg: WsBinaryMessage, ctx: &mut Self::Context) -> Self::Result {
        ctx.binary(msg.0);
    }
}

/// 配置WebSocket路由
pub fn configure_video_ws_routes(cfg: &mut web::ServiceConfig) {
    cfg.route("/ws/video", web::get().to(video_ws_index));
}

/// WebSocket视频流处理端点
pub async fn video_ws_index(
    req: HttpRequest,
    stream: web::Payload,
    stream_manager: web::Data<Arc<VideoStreamManager>>,
) -> Result<HttpResponse, actix_web::Error> {
    let ws_handler = VideoWsHandler::new(stream_manager.get_ref().clone());
    ws::start(ws_handler, &req, stream)
}

/// 心跳检查任务
pub async fn heartbeat_task(distributor: Arc<VideoFrameDistributor>, interval_secs: u64) {
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(interval_secs));

    loop {
        interval.tick().await;

        // 向所有活跃客户端发送心跳
        let clients = distributor.clients.read().await;
        for addr in clients.values() {
            addr.do_send(WsTextMessage(
                serde_json::to_string(&WsMessage::Ping).unwrap_or_default(),
            ));
        }
    }
}

/// 创建视频帧分发器
pub fn create_frame_distributor() -> Arc<VideoFrameDistributor> {
    Arc::new(VideoFrameDistributor::new())
}
