//! / BFF实时推送优化模块

use crate::bff::models::*;
use actix::{Actor, ActorContext, StreamHandler};
use actix_web::{web, HttpRequest, HttpResponse, Result as ActixResult};
use actix_web_actors::ws;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// WebSocket会话
pub struct BffWebSocketSession {
    /// 车辆ID列表
    vehicle_ids: Vec<i32>,
    /// 客户端ID
    client_id: String,
}

impl BffWebSocketSession {
    pub fn new(client_id: String) -> Self {
        Self {
            vehicle_ids: vec![],
            client_id,
        }
    }
}

impl Actor for BffWebSocketSession {
    type Context = ws::WebsocketContext<Self>;
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for BffWebSocketSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Pong(_)) => {}
            Ok(ws::Message::Text(text)) => {
                // 处理客户端消息
                if let Ok(cmd) = serde_json::from_str::<ClientCommand>(&text) {
                    self.handle_command(cmd, ctx);
                }
            }
            Ok(ws::Message::Close(reason)) => {
                log::info!("WebSocket client {} disconnected", self.client_id);
                ctx.close(reason);
                ctx.stop();
            }
            _ => {}
        }
    }
}

impl BffWebSocketSession {
    /// 处理客户端命令
    fn handle_command(
        &mut self,
        cmd: ClientCommand,
        ctx: &mut <BffWebSocketSession as actix::Actor>::Context,
    ) {
        match cmd.action.as_str() {
            "subscribe" => {
                // 订阅车辆实时数据
                if let Some(vehicle_ids) = cmd.vehicle_ids {
                    self.vehicle_ids = vehicle_ids;
                    log::info!(
                        "Client {} subscribed to vehicles: {:?}",
                        self.client_id,
                        self.vehicle_ids
                    );

                    // 发送确认消息
                    let response = ServerMessage {
                        message_type: "subscription_confirmed".to_string(),
                        data: Some(serde_json::json!({
                            "vehicle_ids": self.vehicle_ids,
                            "client_id": self.client_id
                        })),
                        timestamp: chrono::Utc::now(),
                    };

                    ctx.text(serde_json::to_string(&response).unwrap_or_default());
                }
            }
            "unsubscribe" => {
                // 取消订阅
                self.vehicle_ids.clear();
                log::info!("Client {} unsubscribed", self.client_id);
            }
            _ => {
                log::warn!("Unknown command: {}", cmd.action);
            }
        }
    }
}

/// 客户端命令
#[derive(Debug, Deserialize)]
pub struct ClientCommand {
    pub action: String,
    pub vehicle_ids: Option<Vec<i32>>,
}

/// 服务器消息
#[derive(Debug, Serialize)]
pub struct ServerMessage {
    pub message_type: String,
    pub data: Option<serde_json::Value>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// 实时推送消息
#[derive(Debug, Clone, Serialize)]
pub struct RealtimePushMessage {
    pub vehicle_id: i32,
    pub message_type: String, // "gps", "sensor", "alarm"
    pub data: serde_json::Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// 实时推送消息处理器
pub struct RealtimePushHandler {
    /// 订阅的客户端
    subscribers: Arc<Mutex<HashMap<String, Vec<i32>>>>, // client_id -> vehicle_ids
}

impl RealtimePushHandler {
    pub fn new() -> Self {
        Self {
            subscribers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// 添加订阅
    pub fn subscribe(&self, client_id: String, vehicle_ids: Vec<i32>) {
        if let Ok(mut subs) = self.subscribers.lock() {
            subs.insert(client_id, vehicle_ids);
        }
    }

    /// 取消订阅
    pub fn unsubscribe(&self, client_id: String) {
        if let Ok(mut subs) = self.subscribers.lock() {
            subs.remove(&client_id);
        }
    }

    /// 推送消息到订阅的客户端
    pub fn push_message(&self, vehicle_id: i32, _message: RealtimePushMessage) {
        if let Ok(subs) = self.subscribers.lock() {
            for (client_id, subscribed_vehicles) in subs.iter() {
                if subscribed_vehicles.contains(&vehicle_id) {
                    // TODO: 发送消息到WebSocket会话
                    log::debug!(
                        "Pushing message to client {} for vehicle {}",
                        client_id,
                        vehicle_id
                    );
                }
            }
        }
    }
}

/// 推送GPS实时数据
impl RealtimePushHandler {
    pub fn push_gps_update(&self, vehicle_id: i32, gps: GpsData) {
        let message = RealtimePushMessage {
            vehicle_id,
            message_type: "gps".to_string(),
            data: serde_json::to_value(&gps).unwrap_or_default(),
            timestamp: chrono::Utc::now(),
        };

        self.push_message(vehicle_id, message);
    }

    pub fn push_sensor_update(&self, vehicle_id: i32, sensor: SensorData) {
        let message = RealtimePushMessage {
            vehicle_id,
            message_type: "sensor".to_string(),
            data: serde_json::to_value(&sensor).unwrap_or_default(),
            timestamp: chrono::Utc::now(),
        };

        self.push_message(vehicle_id, message);
    }
}

impl Default for RealtimePushHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// WebSocket路由处理器
pub async fn bff_websocket_route(
    req: HttpRequest,
    stream: web::Payload,
    query: web::Query<HashMap<String, String>>,
) -> ActixResult<HttpResponse> {
    let client_id = query
        .get("client_id")
        .unwrap_or(&uuid::Uuid::new_v4().to_string())
        .clone();

    log::info!("New WebSocket connection from client: {}", client_id);

    let session = BffWebSocketSession::new(client_id);

    ws::start(session, &req, stream)
}

/// 订阅请求
#[derive(Debug, Deserialize)]
pub struct SubscribeRequest {
    pub client_id: String,
    pub vehicle_ids: Vec<i32>,
}

/// 订阅车辆实时数据(HTTP接口)
#[utoipa::path(
    post,
    path = "/bff/realtime/subscribe",
    request_body = SubscribeRequest,
    responses(
        (status = 200, description = "订阅成功"),
        (status = 400, description = "请求参数错误"),
        (status = 500, description = "服务器错误")
    ),
    tag = "BFF实时推送"
)]
pub async fn subscribe_vehicles(
    req: web::Json<SubscribeRequest>,
    handler: web::Data<RealtimePushHandler>,
) -> ActixResult<HttpResponse> {
    let client_id = req.client_id.clone();
    let vehicle_ids = req.vehicle_ids.clone();
    let vehicle_count = req.vehicle_ids.len();

    handler.subscribe(client_id.clone(), vehicle_ids);

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "code": 200,
        "message": "Subscribed successfully",
        "data": {
            "client_id": req.client_id,
            "vehicle_count": vehicle_count
        }
    })))
}
