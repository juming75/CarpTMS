//! / JT808 设备会话管理
// 管理车载终端的会话状态、认证、心跳等

use actix::prelude::*;
use actix::ResponseActFuture;
use chrono::{DateTime, Utc};
use log::{debug, info, warn};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

/// JT808 设备会话
#[derive(Debug, Clone)]
pub struct Jt808DeviceSession {
    /// 设备ID(手机号)
    pub device_id: String,
    /// 手机号(BCD编码)
    pub phone: String,
    /// 认证状态
    pub auth_status: AuthStatus,
    /// 最后活动时间
    pub last_activity: DateTime<Utc>,
    /// 心跳时间
    pub heartbeat_time: DateTime<Utc>,
    /// 序列号管理
    pub flow_no: u16,
    /// 版本号
    pub version: Option<String>,
    /// 制造商ID
    pub manufacturer_id: Option<String>,
    /// 终端型号
    pub model: Option<String>,
    /// 终端ID
    pub terminal_id: Option<String>,
    /// SIM卡ICCID
    pub iccid: Option<String>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
}

/// 认证状态
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthStatus {
    /// 未认证
    Unauthenticated,
    /// 认证中
    Authenticating,
    /// 已认证
    Authenticated,
    /// 认证失败
    Failed,
}

/// 设备连接消息
#[derive(Message)]
#[rtype(result = "Result<(), SessionError>")]
pub struct DeviceConnect {
    pub device_id: String,
    pub phone: String,
}

/// 设备断开消息
#[derive(Message)]
#[rtype(result = "()")]
pub struct DeviceDisconnect {
    pub device_id: String,
    pub reason: String,
}

/// 设备心跳消息
#[derive(Message)]
#[rtype(result = "()")]
pub struct DeviceHeartbeat {
    pub device_id: String,
    pub timestamp: DateTime<Utc>,
}

/// 更新会话信息
#[derive(Message)]
#[rtype(result = "Result<(), SessionError>")]
pub struct UpdateSession {
    pub device_id: String,
    pub flow_no: Option<u16>,
    pub timestamp: DateTime<Utc>,
}

/// 查询会话
#[derive(Message)]
#[rtype(result = "Option<Jt808DeviceSession>")]
pub struct QuerySession {
    pub device_id: String,
}

/// 获取所有会话
#[derive(Message)]
#[rtype(result = "Vec<Jt808DeviceSession>")]
pub struct GetAllSessions;

/// 清理过期会话
#[derive(Message)]
#[rtype(result = "usize")]
pub struct CleanupExpiredSessions {
    pub timeout_seconds: u64,
}

/// 会话错误
#[derive(Debug, thiserror::Error)]
pub enum SessionError {
    #[error("Device not found: {0}")]
    DeviceNotFound(String),

    #[error("Device already authenticated: {0}")]
    AlreadyAuthenticated(String),

    #[error("Invalid session state")]
    InvalidState,

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// JT808 会话管理器 Actor
pub struct Jt808SessionManager {
    /// 会话存储:device_id -> Session
    sessions: Arc<RwLock<HashMap<String, Jt808DeviceSession>>>,
    /// 会话超时时间(秒)
    session_timeout: Duration,
    /// 心跳间隔(秒)
    heartbeat_interval: Duration,
}

impl Jt808SessionManager {
    /// 创建新的会话管理器
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            session_timeout: Duration::from_secs(600), // 10分钟超时
            heartbeat_interval: Duration::from_secs(60), // 60秒心跳
        }
    }

    /// 配置会话管理器
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.session_timeout = timeout;
        self
    }

    pub fn with_heartbeat_interval(mut self, interval: Duration) -> Self {
        self.heartbeat_interval = interval;
        self
    }
}

impl Default for Jt808SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Actor for Jt808SessionManager {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        info!("JT808 session manager started");

        // 启动定时清理任务
        let timeout_seconds = self.session_timeout.as_secs();
        let addr = _ctx.address();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));
            loop {
                interval.tick().await;
                if let Ok(count) = addr.send(CleanupExpiredSessions { timeout_seconds }).await {
                    if count > 0 {
                        debug!("Cleaned up {} expired sessions", count);
                    }
                }
            }
        });
    }
}

/// 处理设备连接
impl Handler<DeviceConnect> for Jt808SessionManager {
    type Result = ResponseActFuture<Self, Result<(), SessionError>>;

    fn handle(&mut self, msg: DeviceConnect, _ctx: &mut Self::Context) -> Self::Result {
        let sessions: Arc<RwLock<HashMap<String, Jt808DeviceSession>>> = self.sessions.clone();
        let device_id: String = msg.device_id.clone();
        let phone: String = msg.phone;

        let fut = async move {
            let mut sessions = sessions.write().await;

            // 检查设备是否已存在
            if sessions.contains_key(&device_id) {
                warn!("Device {} already connected, updating session", device_id);
                if let Some(session) = sessions.get_mut(&device_id) {
                    session.last_activity = Utc::now();
                    session.heartbeat_time = Utc::now();
                }
                return Ok(());
            }

            // 创建新会话
            let session = Jt808DeviceSession {
                device_id: device_id.clone(),
                phone,
                auth_status: AuthStatus::Unauthenticated,
                last_activity: Utc::now(),
                heartbeat_time: Utc::now(),
                flow_no: 0,
                version: None,
                manufacturer_id: None,
                model: None,
                terminal_id: None,
                iccid: None,
                created_at: Utc::now(),
            };

            sessions.insert(device_id.clone(), session);
            info!("Device {} connected and session created", device_id);

            Ok(())
        };

        Box::pin(actix::fut::wrap_future(fut))
    }
}

/// 处理设备断开
impl Handler<DeviceDisconnect> for Jt808SessionManager {
    type Result = ResponseActFuture<Self, ()>;

    fn handle(&mut self, msg: DeviceDisconnect, _ctx: &mut Self::Context) -> Self::Result {
        let sessions: Arc<RwLock<HashMap<String, Jt808DeviceSession>>> = self.sessions.clone();
        let device_id: String = msg.device_id.clone();
        let reason: String = msg.reason;

        let fut = async move {
            let mut sessions = sessions.write().await;

            if let Some(session) = sessions.remove(&device_id) {
                info!(
                    "Device {} disconnected: {}, session duration: {}s",
                    device_id,
                    reason,
                    (Utc::now() - session.created_at).num_seconds()
                );
            } else {
                debug!("Device {} not found in sessions", device_id);
            }
        };

        Box::pin(actix::fut::wrap_future(fut))
    }
}

/// 处理设备心跳
impl Handler<DeviceHeartbeat> for Jt808SessionManager {
    type Result = ResponseActFuture<Self, ()>;

    fn handle(&mut self, msg: DeviceHeartbeat, _ctx: &mut Self::Context) -> Self::Result {
        let sessions: Arc<RwLock<HashMap<String, Jt808DeviceSession>>> = self.sessions.clone();
        let device_id: String = msg.device_id.clone();
        let timestamp: DateTime<Utc> = msg.timestamp;

        let fut = async move {
            let mut sessions = sessions.write().await;

            if let Some(session) = sessions.get_mut(&device_id) {
                session.last_activity = timestamp;
                session.heartbeat_time = timestamp;
                debug!("Device {} heartbeat updated", device_id);
            } else {
                warn!("Heartbeat from unknown device: {}", device_id);
            }
        };

        Box::pin(actix::fut::wrap_future(fut))
    }
}

/// 处理会话更新
impl Handler<UpdateSession> for Jt808SessionManager {
    type Result = ResponseActFuture<Self, Result<(), SessionError>>;

    fn handle(&mut self, msg: UpdateSession, _ctx: &mut Self::Context) -> Self::Result {
        let sessions: Arc<RwLock<HashMap<String, Jt808DeviceSession>>> = self.sessions.clone();
        let device_id: String = msg.device_id.clone();
        let flow_no: Option<u16> = msg.flow_no;
        let timestamp: DateTime<Utc> = msg.timestamp;

        let fut = async move {
            let mut sessions = sessions.write().await;

            if let Some(session) = sessions.get_mut(&device_id) {
                session.last_activity = timestamp;
                if let Some(flow_no) = flow_no {
                    session.flow_no = flow_no;
                }
                debug!(
                    "Session {} updated, flow_no: {}",
                    device_id, session.flow_no
                );
                Ok(())
            } else {
                Err(SessionError::DeviceNotFound(device_id))
            }
        };

        Box::pin(actix::fut::wrap_future(fut))
    }
}

/// 处理会话查询
impl Handler<QuerySession> for Jt808SessionManager {
    type Result = ResponseActFuture<Self, Option<Jt808DeviceSession>>;

    fn handle(&mut self, msg: QuerySession, _ctx: &mut Self::Context) -> Self::Result {
        let sessions: Arc<RwLock<HashMap<String, Jt808DeviceSession>>> = self.sessions.clone();
        let device_id: String = msg.device_id.clone();

        let fut = async move {
            let sessions = sessions.read().await;
            sessions.get(&device_id).cloned()
        };

        Box::pin(actix::fut::wrap_future(fut))
    }
}

/// 处理获取所有会话
impl Handler<GetAllSessions> for Jt808SessionManager {
    type Result = ResponseActFuture<Self, Vec<Jt808DeviceSession>>;

    fn handle(&mut self, _msg: GetAllSessions, _ctx: &mut Self::Context) -> Self::Result {
        let sessions: Arc<RwLock<HashMap<String, Jt808DeviceSession>>> = self.sessions.clone();

        let fut = async move {
            let sessions = sessions.read().await;
            sessions.values().cloned().collect()
        };

        Box::pin(actix::fut::wrap_future(fut))
    }
}

/// 处理清理过期会话
impl Handler<CleanupExpiredSessions> for Jt808SessionManager {
    type Result = ResponseActFuture<Self, usize>;

    fn handle(&mut self, msg: CleanupExpiredSessions, _ctx: &mut Self::Context) -> Self::Result {
        let sessions: Arc<RwLock<HashMap<String, Jt808DeviceSession>>> = self.sessions.clone();
        let timeout_seconds: u64 = msg.timeout_seconds;

        let fut = async move {
            let mut sessions = sessions.write().await;
            let now: DateTime<Utc> = Utc::now();
            let mut expired_devices: Vec<String> = Vec::new();

            // 查找过期会话
            for (device_id, session) in sessions.iter() {
                let elapsed = now.signed_duration_since(session.last_activity);
                if elapsed.num_seconds() as u64 > timeout_seconds {
                    expired_devices.push(device_id.clone());
                }
            }

            // 移除过期会话
            let count: usize = expired_devices.len();
            for device_id in &expired_devices {
                if let Some(session) = sessions.remove(device_id) {
                    info!(
                        "Expired session removed: {}, duration: {}s",
                        device_id,
                        (now - session.created_at).num_seconds()
                    );
                }
            }

            count
        };

        Box::pin(actix::fut::wrap_future(fut))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_creation() {
        let session = Jt808DeviceSession {
            device_id: "123456".to_string(),
            phone: "12345678901".to_string(),
            auth_status: AuthStatus::Unauthenticated,
            last_activity: Utc::now(),
            heartbeat_time: Utc::now(),
            flow_no: 0,
            version: None,
            manufacturer_id: None,
            model: None,
            terminal_id: None,
            iccid: None,
            created_at: Utc::now(),
        };

        assert_eq!(session.device_id, "123456");
        assert_eq!(session.auth_status, AuthStatus::Unauthenticated);
    }

    #[test]
    fn test_session_manager() {
        let manager = Jt808SessionManager::new();
        assert_eq!(manager.session_timeout.as_secs(), 600);
    }
}
