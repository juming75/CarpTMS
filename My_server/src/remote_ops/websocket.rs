//! WebSocket 终端服务模块
//!
//! 提供基于 WebSocket 的实时 SSH 终端功能，支持 xterm.js 集成

use actix::{Actor, ActorContext, AsyncContext, Running, StreamHandler};
use actix_web::{web, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::errors::AppResult;
use crate::remote_ops::ssh::SshConnectionManager;

/// WebSocket SSH 终端会话 Actor
pub struct TerminalSession {
    pub session_id: Uuid,
    pub server_id: Uuid,
    pub ssh_tx: Option<mpsc::UnboundedSender<String>>,
    pub ssh_rx: Option<mpsc::UnboundedReceiver<String>>,
    pub hb: Instant,
}

impl TerminalSession {
    pub fn new(server_id: Uuid) -> (Self, mpsc::UnboundedSender<String>) {
        let (tx, rx) = mpsc::unbounded_channel();
        (
            Self {
                session_id: Uuid::new_v4(),
                server_id,
                ssh_tx: Some(tx.clone()),
                ssh_rx: Some(rx),
                hb: Instant::now(),
            },
            tx,
        )
    }

    fn hb_interval(&self, ctx: &mut <Self as Actor>::Context) {
        ctx.run_interval(Duration::from_secs(5), |act, ctx| {
            if Instant::now().duration_since(act.hb) > Duration::from_secs(30) {
                ctx.stop();
                return;
            }
            ctx.ping(b"");
        });
    }
}

impl Actor for TerminalSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.hb_interval(ctx);
    }

    fn stopping(&mut self, _ctx: &mut Self::Context) -> Running {
        Running::Stop
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for TerminalSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            Ok(ws::Message::Pong(_)) => {
                self.hb = Instant::now();
            }
            Ok(ws::Message::Text(text)) => {
                // 转发用户输入到 SSH
                if let Some(tx) = &self.ssh_tx {
                    let _ = tx.send(text.to_string());
                }
            }
            Ok(ws::Message::Binary(bin)) => {
                if let Some(tx) = &self.ssh_tx {
                    let _ = tx.send(format!("{:?}", bin));
                }
            }
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            _ => ctx.stop(),
        }
    }
}

/// 会话管理器
pub struct WsSessionManager {
    sessions: Arc<Mutex<HashMap<String, TerminalSession>>>,
    ssh_manager: SshConnectionManager,
}

impl WsSessionManager {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            ssh_manager: SshConnectionManager::new(),
        }
    }
}

impl Default for WsSessionManager {
    fn default() -> Self {
        Self::new()
    }
}

/// WebSocket 终端连接入口
pub async fn terminal_ws(
    req: HttpRequest,
    stream: web::Payload,
    path: web::Path<String>,
) -> Result<HttpResponse, actix_web::Error> {
    let server_id = path.into_inner();
    let server_uuid = Uuid::parse_str(&server_id).unwrap_or_default();
    let (session, _tx) = TerminalSession::new(server_uuid);
    ws::start(session, &req, stream)
}
