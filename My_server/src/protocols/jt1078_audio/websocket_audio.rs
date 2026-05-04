//! WebSocket音频流处理
//!
//! 提供WebSocket接口供客户端推送和接收音频流
//! 实现双向音频传输的WebSocket端点

use actix::{Actor, ActorContext, StreamHandler};
use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use super::audio_forwarder::AudioForwarder;

/// WebSocket音频连接请求参数
#[derive(Debug, Deserialize)]
pub struct AudioWebSocketQuery {
    /// 设备ID（终端标识）
    pub device_id: String,
    /// 客户端ID（可选，用于识别对讲客户端）
    pub client_id: Option<String>,
    /// 音频通道号
    pub channel: Option<u8>,
    /// 操作类型: "intercom"（对讲）, "listen"（监听）
    pub action: Option<String>,
}

/// WebSocket音频消息类型
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WebSocketAudioMessage {
    /// 音频数据帧
    #[serde(rename = "audio_data")]
    AudioData {
        /// Base64编码的音频数据
        data: String,
        /// 时间戳
        timestamp: u64,
    },
    /// 开始对讲
    #[serde(rename = "start_intercom")]
    StartIntercom { device_id: String, channel: u8 },
    /// 停止对讲
    #[serde(rename = "stop_intercom")]
    StopIntercom,
    /// 错误消息
    #[serde(rename = "error")]
    Error { message: String },
    /// 状态消息
    #[serde(rename = "status")]
    Status { message: String, status: String },
}

/// 音频WebSocket Actor
#[allow(dead_code)]
pub struct AudioWebSocketActor {
    /// 音频转发器
    forwarder: Arc<AudioForwarder>,
    /// 客户端ID
    client_id: String,
    /// 设备ID
    device_id: String,
    /// 音频通道号
    channel: u8,
}

impl AudioWebSocketActor {
    /// 创建新的音频WebSocket Actor
    pub fn new(
        forwarder: Arc<AudioForwarder>,
        client_id: String,
        device_id: String,
        channel: u8,
    ) -> Self {
        Self {
            forwarder,
            client_id,
            device_id,
            channel,
        }
    }
}

impl Actor for AudioWebSocketActor {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        info!(
            "Audio WebSocket session started: client={}, device={}",
            self.client_id, self.device_id
        );
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        info!("Audio WebSocket session stopped: client={}", self.client_id);
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for AudioWebSocketActor {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Text(text)) => {
                // 解析文本消息（控制命令）
                if let Ok(ws_msg) = serde_json::from_str::<WebSocketAudioMessage>(&text) {
                    match ws_msg {
                        WebSocketAudioMessage::StartIntercom {
                            device_id,
                            channel: _,
                        } => {
                            let forwarder = self.forwarder.clone();
                            let client_id = self.client_id.clone();

                            actix_web::rt::spawn(async move {
                                if let Err(e) = forwarder
                                    .start_intercom_session(&device_id, &client_id)
                                    .await
                                {
                                    error!("Failed to start intercom: {}", e);
                                } else {
                                    info!(
                                        "Intercom started: device={}, client={}",
                                        device_id, client_id
                                    );
                                }
                            });

                            let status_msg = WebSocketAudioMessage::Status {
                                message: "Intercom session started".to_string(),
                                status: "intercom_active".to_string(),
                            };
                            ctx.text(serde_json::to_string(&status_msg).unwrap());
                        }
                        WebSocketAudioMessage::StopIntercom => {
                            let forwarder = self.forwarder.clone();
                            let device_id = self.device_id.clone();

                            actix_web::rt::spawn(async move {
                                if let Err(e) = forwarder.stop_intercom_session(&device_id).await {
                                    error!("Failed to stop intercom: {}", e);
                                }
                            });

                            let status_msg = WebSocketAudioMessage::Status {
                                message: "Intercom session stopped".to_string(),
                                status: "stopped".to_string(),
                            };
                            ctx.text(serde_json::to_string(&status_msg).unwrap());
                        }
                        _ => {
                            debug!("Received control message: {:?}", ws_msg);
                        }
                    }
                }
            }
            Ok(ws::Message::Binary(bin)) => {
                // 处理二进制音频数据
                debug!("Received audio data from client: {} bytes", bin.len());

                let forwarder = self.forwarder.clone();
                let client_id = self.client_id.clone();

                actix_web::rt::spawn(async move {
                    if let Err(e) = forwarder
                        .handle_client_audio(&client_id, bin.to_vec())
                        .await
                    {
                        warn!("Failed to handle client audio: {}", e);
                    }
                });
            }
            Ok(ws::Message::Ping(msg)) => {
                ctx.pong(&msg);
            }
            Ok(ws::Message::Pong(_)) => {
                // 心跳响应，无需处理
            }
            Ok(ws::Message::Close(reason)) => {
                info!("Audio WebSocket connection closed: {:?}", reason);
                ctx.stop();
            }
            _ => {}
        }
    }
}

/// 音频WebSocket处理
/// 处理WebSocket连接
pub async fn handle_audio_websocket(
    req: HttpRequest,
    payload: web::Payload,
    forwarder: web::Data<Arc<AudioForwarder>>,
) -> Result<HttpResponse, Error> {
    // 解析连接参数
    let query = web::Query::<AudioWebSocketQuery>::from_query(req.query_string()).map_err(|e| {
        error!("Failed to parse WebSocket query: {}", e);
        actix_web::error::ErrorBadRequest("Invalid query parameters")
    })?;

    // 生成客户端ID
    let client_id = format!("ws_{}", uuid::Uuid::new_v4());
    let device_id = query.device_id.clone();
    let channel = query.channel.unwrap_or(0);

    info!(
        "WebSocket audio connection established: client={}, device={}",
        client_id, device_id
    );

    // 创建Actor并启动WebSocket会话
    let actor =
        AudioWebSocketActor::new(forwarder.get_ref().clone(), client_id, device_id, channel);

    let resp = ws::start(actor, &req, payload)?;
    Ok(resp)
}

/// 创建对讲控制API路由
pub fn configure_audio_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/api/audio/ws").route(web::get().to(audio_websocket_endpoint)))
        .service(
            web::resource("/api/audio/intercom/start")
                .route(web::post().to(start_intercom_endpoint)),
        )
        .service(
            web::resource("/api/audio/intercom/stop/{device_id}")
                .route(web::post().to(stop_intercom_endpoint)),
        )
        .service(web::resource("/api/audio/stats").route(web::get().to(get_audio_stats_endpoint)));
}

/// WebSocket音频端点
async fn audio_websocket_endpoint(
    req: HttpRequest,
    payload: web::Payload,
    forwarder: web::Data<Arc<AudioForwarder>>,
) -> Result<HttpResponse, Error> {
    handle_audio_websocket(req, payload, forwarder).await
}

/// 开始对讲请求
#[derive(Debug, Deserialize)]
pub struct StartIntercomRequest {
    /// 设备ID
    pub device_id: String,
    /// 客户端ID（可选）
    pub client_id: Option<String>,
    /// 音频通道号
    pub channel: u8,
    /// 服务器标识
    pub server_flag: Option<u8>,
}

/// 对讲控制API响应
#[derive(Debug, Serialize)]
pub struct IntercomResponse {
    /// 是否成功
    pub success: bool,
    /// 消息
    pub message: String,
    /// 命令数据（Base64）
    pub command_data: Option<String>,
}

/// 开始对讲端点
async fn start_intercom_endpoint(
    forwarder: web::Data<Arc<AudioForwarder>>,
    body: web::Json<StartIntercomRequest>,
) -> HttpResponse {
    use super::commands::AudioCommandBuilder;

    // 检查设备是否在线
    if !forwarder.has_active_intercom(&body.device_id).await {
        // 构建0x9101命令
        let builder = AudioCommandBuilder::new(body.device_id.clone())
            .with_channel(body.channel)
            .with_server_flag(body.server_flag.unwrap_or(0));

        let command_data = builder.build_complete_packet();
        let command_base64 =
            base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &command_data);

        // 开始对讲会话
        let client_id = body.client_id.clone().unwrap_or_default();
        if let Err(e) = forwarder
            .start_intercom_session(&body.device_id, &client_id)
            .await
        {
            return HttpResponse::InternalServerError().json(IntercomResponse {
                success: false,
                message: format!("Failed to start intercom: {}", e),
                command_data: None,
            });
        }

        HttpResponse::Ok().json(IntercomResponse {
            success: true,
            message: "Intercom session started, 0x9101 command generated".to_string(),
            command_data: Some(command_base64),
        })
    } else {
        HttpResponse::BadRequest().json(IntercomResponse {
            success: false,
            message: "Device already in intercom session".to_string(),
            command_data: None,
        })
    }
}

/// 停止对讲端点
async fn stop_intercom_endpoint(
    forwarder: web::Data<Arc<AudioForwarder>>,
    path: web::Path<String>,
) -> HttpResponse {
    let device_id = path.into_inner();

    if let Err(e) = forwarder.stop_intercom_session(&device_id).await {
        return HttpResponse::InternalServerError().json(IntercomResponse {
            success: false,
            message: format!("Failed to stop intercom: {}", e),
            command_data: None,
        });
    }

    HttpResponse::Ok().json(IntercomResponse {
        success: true,
        message: "Intercom session stopped".to_string(),
        command_data: None,
    })
}

/// 获取音频统计信息端点
async fn get_audio_stats_endpoint(forwarder: web::Data<Arc<AudioForwarder>>) -> HttpResponse {
    let stats = forwarder.get_stats().await;
    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "data": {
            "client_to_terminal_bytes": stats.client_to_terminal_bytes,
            "terminal_to_client_bytes": stats.terminal_to_client_bytes,
            "forward_errors": stats.forward_errors,
            "active_sessions": stats.active_sessions,
        }
    }))
}
