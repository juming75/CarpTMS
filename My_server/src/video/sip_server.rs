//! / GB28181 SIP 服务器
//! 实现完整的 SIP 服务器功能

use crate::protocols::gb28181::{
    GB28181DeviceInfo, GB28181DeviceType, GB28181Protocol, GB28181SipMessage, GB28181Version,
    SipMethod,
};
use crate::video::config::Gb28181Config;
use log::{debug, error, info, warn};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::net::UdpSocket;
use tokio::sync::RwLock;
use tokio::time::interval;

/// GB28181 SIP 服务器状态
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SipServerState {
    /// 初始化中
    Initializing,
    /// 运行中
    Running,
    /// 停止中
    Stopping,
    /// 已停止
    Stopped,
}

/// GB28181 SIP Server
pub struct Gb28181SipServer {
    #[allow(dead_code)]
    protocol: Arc<GB28181Protocol>,
    /// 配置
    config: Gb28181Config,
    /// 服务器状态
    state: Arc<RwLock<SipServerState>>,
    /// 已注册设备 (device_id -> device_info)
    registered_devices: Arc<RwLock<HashMap<String, RegisteredDevice>>>,
    /// 活跃会话 (call_id -> session)
    sessions: Arc<RwLock<HashMap<String, SipSession>>>,
    /// 最后响应时间
    last_response_time: Arc<RwLock<Instant>>,
    /// 服务器启动时间
    started_at: Arc<RwLock<Option<Instant>>>,
}

/// 已注册设备信息
#[derive(Debug, Clone)]
pub struct RegisteredDevice {
    /// 设备信息
    pub info: GB28181DeviceInfo,
    /// 注册过期时间
    pub expires_at: Instant,
    /// IP地址
    pub ip_address: SocketAddr,
    /// 端口
    pub port: u16,
    /// 心跳超时计数
    pub heartbeat_failures: u8,
}

/// SIP 会话
#[derive(Debug, Clone)]
pub struct SipSession {
    /// 会话 ID
    pub call_id: String,
    /// 设备 ID
    pub device_id: String,
    /// 会话状态
    pub state: SipSessionState,
    /// 创建时间
    pub created_at: Instant,
    /// 最后活动时间
    pub last_activity: Instant,
    /// 媒体会话信息
    pub media_info: Option<MediaSessionInfo>,
}

/// SIP 会话状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SipSessionState {
    /// 等待 INVITE 响应
    InvitePending,
    /// 等待 ACK
    InviteSent,
    /// 已建立
    Established,
    /// 正在关闭
    Closing,
    /// 已关闭
    Closed,
}

/// 媒体会话信息
#[derive(Debug, Clone)]
pub struct MediaSessionInfo {
    /// SSRC
    pub ssrc: u32,
    /// RTP 地址
    pub rtp_addr: SocketAddr,
    /// RTP 端口
    pub rtp_port: u16,
    /// 设备 RTP 地址
    pub device_rtp_addr: SocketAddr,
    /// 设备 RTP 端口
    pub device_rtp_port: u16,
    /// 视频编码
    pub video_codec: String,
    /// 音频编码
    pub audio_codec: Option<String>,
    /// 分辨率
    pub resolution: Option<String>,
}

impl Gb28181SipServer {
    /// 创建新的 SIP 服务器
    pub fn new(config: Gb28181Config) -> Self {
        let protocol = GB28181Protocol::new()
            .with_version(GB28181Version::V2016)
            .with_sip_domain(config.server_domain.clone())
            .with_sip_port(config.sip_port);

        Self {
            protocol: Arc::new(protocol),
            config,
            state: Arc::new(RwLock::new(SipServerState::Initializing)),
            registered_devices: Arc::new(RwLock::new(HashMap::new())),
            sessions: Arc::new(RwLock::new(HashMap::new())),
            last_response_time: Arc::new(RwLock::new(Instant::now())),
            started_at: Arc::new(RwLock::new(None)),
        }
    }

    /// 启动 SIP 服务器
    pub async fn start(&self, bind_addr: &str) -> Result<(), SipServerError> {
        info!(
            "Starting GB28181 SIP server on {}:{}",
            bind_addr, self.config.sip_port
        );

        // 更新状态
        {
            let mut state = self.state.write().await;
            *state = SipServerState::Running;
        }

        // 记录启动时间
        {
            let mut started = self.started_at.write().await;
            *started = Some(Instant::now());
        }

        // 绑定 UDP socket
        let addr = format!("{}:{}", bind_addr, self.config.sip_port);
        let socket = UdpSocket::bind(&addr)
            .await
            .map_err(|e| SipServerError::BindFailed(e.to_string()))?;

        info!("GB28181 SIP server listening on {}", addr);

        // 启动后台任务 - 为每个任务克隆必要的 Arc
        let registered_devices = self.registered_devices.clone();
        let sessions = self.sessions.clone();
        let state_heartbeat = self.state.clone();
        let state_session = self.state.clone();

        // 启动心跳检查任务
        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_secs(30));
            loop {
                ticker.tick().await;

                let current_state = state_heartbeat.read().await;
                if *current_state != SipServerState::Running {
                    break;
                }
                drop(current_state);

                // 清理过期设备
                let now = Instant::now();
                let mut devices = registered_devices.write().await;
                devices.retain(|_, device| device.expires_at > now);

                // 检查设备心跳
                for (_, device) in devices.iter_mut() {
                    if device.heartbeat_failures >= 3 {
                        warn!(
                            "Device {} heartbeat timeout, removing",
                            device.info.device_id
                        );
                    }
                }
            }
            info!("GB28181 heartbeat checker stopped");
        });

        // 启动会话清理任务
        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_secs(60));
            loop {
                ticker.tick().await;

                let current_state = state_session.read().await;
                if *current_state != SipServerState::Running {
                    break;
                }
                drop(current_state);

                // 清理超时会话
                let now = Instant::now();
                let mut sessions_lock = sessions.write().await;
                sessions_lock.retain(|_, session| {
                    now.duration_since(session.last_activity) < Duration::from_secs(3600)
                });
            }
            info!("GB28181 session cleaner stopped");
        });

        // 主消息循环
        let mut buf = vec![0u8; 65536];
        loop {
            let current_state = self.state.read().await;
            if *current_state != SipServerState::Running {
                break;
            }
            drop(current_state);

            // 接收数据
            match socket.recv_from(&mut buf).await {
                Ok((len, remote_addr)) => {
                    let data = &buf[..len];

                    // 处理 SIP 消息
                    match self.process_sip_message(data, remote_addr).await {
                        Ok(response) => {
                            if let Some(response_data) = response {
                                if let Err(e) = socket.send_to(&response_data, remote_addr).await {
                                    warn!("Failed to send SIP response: {}", e);
                                }
                            }
                        }
                        Err(e) => {
                            warn!("Failed to process SIP message: {}", e);
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to receive SIP message: {}", e);
                }
            }
        }

        // 更新状态
        {
            let mut state = self.state.write().await;
            *state = SipServerState::Stopped;
        }

        info!("GB28181 SIP server stopped");
        Ok(())
    }

    /// 停止 SIP 服务器
    pub async fn stop(&self) {
        info!("Stopping GB28181 SIP server...");

        let mut state = self.state.write().await;
        *state = SipServerState::Stopping;

        // 关闭所有会话
        let mut sessions = self.sessions.write().await;
        for (_, session) in sessions.iter_mut() {
            session.state = SipSessionState::Closed;
        }
        sessions.clear();
        drop(sessions);

        // 清理设备
        let mut devices = self.registered_devices.write().await;
        devices.clear();
        drop(devices);

        *state = SipServerState::Stopped;
        info!("GB28181 SIP server stopped");
    }

    /// 处理 SIP 消息
    async fn process_sip_message(
        &self,
        data: &[u8],
        remote_addr: SocketAddr,
    ) -> Result<Option<Vec<u8>>, SipServerError> {
        // 解析 SIP 消息
        let message_str = String::from_utf8_lossy(data);
        let sip_message = GB28181SipMessage::from_str(&message_str)
            .map_err(|e| SipServerError::ParseError(e.to_string()))?;

        debug!(
            "Received SIP {:?} from {}: {:?}",
            sip_message.method, remote_addr, sip_message.call_id
        );

        // 更新最后响应时间
        {
            let mut last = self.last_response_time.write().await;
            *last = Instant::now();
        }

        // 处理不同类型的消息
        match sip_message.method {
            SipMethod::Register => self.handle_register(sip_message, remote_addr).await,
            SipMethod::Invite => self.handle_invite(sip_message).await,
            SipMethod::Ack => self.handle_ack(sip_message).await,
            SipMethod::Bye => self.handle_bye(sip_message).await,
            SipMethod::Message => self.handle_message(sip_message).await,
            SipMethod::Options => self.handle_options(sip_message).await,
            SipMethod::Notify => self.handle_notify(sip_message).await,
            _ => {
                debug!("Unhandled SIP method: {:?}", sip_message.method);
                Ok(None)
            }
        }
    }

    /// 处理 REGISTER 请求
    async fn handle_register(
        &self,
        message: GB28181SipMessage,
        remote_addr: SocketAddr,
    ) -> Result<Option<Vec<u8>>, SipServerError> {
        info!("Processing REGISTER from {}", remote_addr);

        // 提取设备 ID
        let device_id = extract_device_id(&message.from)
            .ok_or_else(|| SipServerError::InvalidMessage("Missing device ID".to_string()))?;

        // 提取过期时间
        let expires = message
            .headers
            .get("expires")
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(3600);

        // 提取认证信息
        let auth_password = message
            .headers
            .get("authorization")
            .and_then(|auth| extract_auth(auth))
            .map(|(user, _)| user)
            .unwrap_or_else(|| device_id.clone());

        // 验证密码（简化版本，实际应该验证摘要）
        if auth_password != self.config.auth_password && auth_password != device_id {
            info!("Authentication failed for device {}", device_id);
            return Ok(Some(self.build_response(
                &message,
                401,
                "Unauthorized",
                None,
                None,
            )));
        }

        // 处理注册
        let device_info = GB28181DeviceInfo {
            device_id: device_id.clone(),
            device_name: device_id.clone(),
            device_type: GB28181DeviceType::from_device_id(&device_id),
            manufacturer_id: "UNKNOWN".to_string(),
            model: "UNKNOWN".to_string(),
            firmware_version: "1.0".to_string(),
            ip_address: remote_addr.ip().to_string(),
            port: remote_addr.port(),
            online: true,
            last_heartbeat: Some(chrono::Utc::now().timestamp() as u64),
        };

        // 更新注册设备
        {
            let mut devices = self.registered_devices.write().await;
            devices.insert(
                device_id.clone(),
                RegisteredDevice {
                    info: device_info,
                    expires_at: Instant::now() + Duration::from_secs(expires),
                    ip_address: remote_addr,
                    port: remote_addr.port(),
                    heartbeat_failures: 0,
                },
            );
        }

        info!(
            "Device {} registered successfully, expires in {}s",
            device_id, expires
        );

        // 构建 200 OK 响应
        Ok(Some(self.build_response(
            &message,
            200,
            "OK",
            Some(&[
                (
                    "Contact",
                    format!(
                        "<sip:{}@{}:{}>;expires={}",
                        device_id,
                        remote_addr.ip(),
                        remote_addr.port(),
                        expires
                    ),
                ),
                ("Expires", expires.to_string()),
            ]),
            None,
        )))
    }

    /// 处理 INVITE 请求
    async fn handle_invite(
        &self,
        message: GB28181SipMessage,
    ) -> Result<Option<Vec<u8>>, SipServerError> {
        info!("Processing INVITE for call_id={}", message.call_id);

        // 提取设备 ID
        let device_id = extract_device_id(&message.from)
            .ok_or_else(|| SipServerError::InvalidMessage("Missing device ID".to_string()))?;

        // 验证设备是否已注册
        let devices = self.registered_devices.read().await;
        if !devices.contains_key(&device_id) {
            warn!("INVITE from unregistered device: {}", device_id);
            return Ok(Some(self.build_response(
                &message,
                403,
                "Forbidden",
                None,
                None,
            )));
        }

        // 解析 SDP
        let sdp = message.sdp.as_ref();

        // 提取媒体信息
        let (rtp_ip, rtp_port) = sdp
            .and_then(|s| parse_sdp_media(s))
            .unwrap_or(("0.0.0.0".to_string(), 0));

        // 创建会话
        let session = SipSession {
            call_id: message.call_id.clone(),
            device_id: device_id.clone(),
            state: SipSessionState::InvitePending,
            created_at: Instant::now(),
            last_activity: Instant::now(),
            media_info: Some(MediaSessionInfo {
                ssrc: rand::random(),
                // P4: 简化解析逻辑，使用 unwrap_or
                rtp_addr: "0.0.0.0:0".parse().unwrap_or_else(|e| {
                    tracing::warn!("Failed to parse default RTP addr: {}", e);
                    "127.0.0.1:0"
                        .parse()
                        .unwrap_or_else(|_| "127.0.0.1:0".parse().unwrap())
                }),

                rtp_port: self.config.rtp_port_start,
                device_rtp_addr: rtp_ip.parse().unwrap_or_else(|e| {
                    tracing::warn!("Failed to parse device RTP addr '{}': {}", rtp_ip, e);
                    "127.0.0.1:0"
                        .parse()
                        .unwrap_or_else(|_| "127.0.0.1:0".parse().unwrap())
                }),

                device_rtp_port: rtp_port,
                video_codec: "H264".to_string(),
                audio_codec: Some("G711A".to_string()),
                resolution: Some("1280x720".to_string()),
            }),
        };

        {
            let mut sessions = self.sessions.write().await;
            sessions.insert(message.call_id.clone(), session);
        }

        // 构建 200 OK 响应（带 SDP）
        let sdp_response = format!(
            r#"v=0
o=- {} {} IN IP4 {}
s=Play
c=IN IP4 {}
t=0 0
m=video {} RTP/AVP 96
a=rtpmap:96 H264/90000
a=sendonly"#,
            device_id,
            chrono::Utc::now().timestamp(),
            self.config.server_id,
            self.config.server_id,
            self.config.rtp_port_start
        );

        Ok(Some(self.build_response(
            &message,
            200,
            "OK",
            Some(&[
                ("Content-Type", "application/sdp".to_string()),
                ("Content-Length", sdp_response.len().to_string()),
            ]),
            Some(&sdp_response),
        )))
    }

    /// 处理 ACK 请求
    async fn handle_ack(
        &self,
        message: GB28181SipMessage,
    ) -> Result<Option<Vec<u8>>, SipServerError> {
        debug!("Processing ACK for call_id={}", message.call_id);

        // 更新会话状态
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(&message.call_id) {
            session.state = SipSessionState::Established;
            session.last_activity = Instant::now();
            info!("Session {} established", message.call_id);
        }

        Ok(None)
    }

    /// 处理 BYE 请求
    async fn handle_bye(
        &self,
        message: GB28181SipMessage,
    ) -> Result<Option<Vec<u8>>, SipServerError> {
        info!("Processing BYE for call_id={}", message.call_id);

        // 更新会话状态
        {
            let mut sessions = self.sessions.write().await;
            if let Some(session) = sessions.get_mut(&message.call_id) {
                session.state = SipSessionState::Closed;
            }
        }

        Ok(Some(self.build_response(&message, 200, "OK", None, None)))
    }

    /// 处理 MESSAGE 请求
    async fn handle_message(
        &self,
        message: GB28181SipMessage,
    ) -> Result<Option<Vec<u8>>, SipServerError> {
        debug!("Processing MESSAGE for call_id={}", message.call_id);

        // 解析消息内容
        if let Some(content) = &message.sdp {
            if content.contains("Keepalive") || content.contains("Heartbeat") {
                // 处理心跳
                let device_id =
                    extract_device_id(&message.from).unwrap_or_else(|| "unknown".to_string());

                let mut devices = self.registered_devices.write().await;
                if let Some(device) = devices.get_mut(&device_id) {
                    device.info.last_heartbeat = Some(chrono::Utc::now().timestamp() as u64);
                    device.heartbeat_failures = 0;
                    debug!("Heartbeat from device {}", device_id);
                }
            }
        }

        Ok(Some(self.build_response(&message, 200, "OK", None, None)))
    }

    /// 处理 OPTIONS 请求
    async fn handle_options(
        &self,
        message: GB28181SipMessage,
    ) -> Result<Option<Vec<u8>>, SipServerError> {
        debug!("Processing OPTIONS for call_id={}", message.call_id);

        Ok(Some(
            self.build_response(
                &message,
                200,
                "OK",
                Some(&[
                    (
                        "Allow",
                        "INVITE, ACK, CANCEL, BYE, REGISTER, OPTIONS, MESSAGE, INFO, NOTIFY"
                            .to_string(),
                    ),
                    ("Accept", "APPLICATION/SDP".to_string()),
                    ("User-Agent", "CarpTMS/2.0 GB28181 Server".to_string()),
                ]),
                None,
            ),
        ))
    }

    /// 处理 NOTIFY 请求
    async fn handle_notify(
        &self,
        message: GB28181SipMessage,
    ) -> Result<Option<Vec<u8>>, SipServerError> {
        debug!("Processing NOTIFY for call_id={}", message.call_id);

        Ok(Some(self.build_response(&message, 200, "OK", None, None)))
    }

    /// 构建 SIP 响应
    fn build_response(
        &self,
        request: &GB28181SipMessage,
        status_code: u16,
        reason: &str,
        headers: Option<&[(&str, String)]>,
        body: Option<&str>,
    ) -> Vec<u8> {
        let mut response = String::new();

        // 状态行
        response.push_str(&format!("SIP/2.0 {} {}\r\n", status_code, reason));

        // Via
        if let Some(via) = request.via.first() {
            response.push_str(&format!("Via: {}\r\n", via));
        }

        // From
        response.push_str(&format!("From: {}\r\n", request.from));

        // To
        response.push_str(&format!("To: {}\r\n", request.to));

        // Call-ID
        response.push_str(&format!("Call-ID: {}\r\n", request.call_id));

        // CSeq
        response.push_str(&format!(
            "CSeq: {} {}\r\n",
            request.cseq,
            request.method.as_str()
        ));

        // User-Agent
        response.push_str("User-Agent: CarpTMS/2.0 GB28181 Server\r\n");

        // 其他 headers
        if let Some(extra_headers) = headers {
            for (key, value) in extra_headers {
                response.push_str(&format!("{}: {}\r\n", key, value));
            }
        }

        // Content-Length
        let body_len = body.map(|b| b.len()).unwrap_or(0);
        response.push_str(&format!("Content-Length: {}\r\n", body_len));

        // 空行
        response.push_str("\r\n");

        // Body
        if let Some(body_content) = body {
            response.push_str(body_content);
        }

        response.into_bytes()
    }

    /// 获取服务器状态
    pub async fn get_state(&self) -> SipServerState {
        self.state.read().await.clone()
    }

    /// 获取已注册设备数量
    pub async fn get_registered_device_count(&self) -> usize {
        self.registered_devices.read().await.len()
    }

    /// 获取活跃会话数量
    pub async fn get_active_session_count(&self) -> usize {
        let sessions = self.sessions.read().await;
        sessions
            .values()
            .filter(|s| s.state == SipSessionState::Established)
            .count()
    }

    /// 获取所有已注册设备
    pub async fn get_registered_devices(&self) -> Vec<GB28181DeviceInfo> {
        let devices = self.registered_devices.read().await;
        devices.values().map(|d| d.info.clone()).collect()
    }

    /// 获取所有会话
    pub async fn get_all_sessions(&self) -> Vec<SipSession> {
        let sessions = self.sessions.read().await;
        sessions.values().cloned().collect()
    }
}

/// 从 URI 中提取设备 ID
fn extract_device_id(uri: &str) -> Option<String> {
    uri.strip_prefix("<sip:")
        .and_then(|s| s.strip_suffix('>'))
        .and_then(|s| s.split('@').next())
        .map(|s| s.to_string())
}

/// 从 Authorization header 中提取认证信息
fn extract_auth(auth: &str) -> Option<(String, String)> {
    // 简化实现，实际应该解析 WWW-Authenticate 头
    if auth.contains("Digest") {
        auth.split(',')
            .filter_map(|part| {
                let part = part.trim();
                if let Some(stripped) = part.strip_prefix("username=") {
                    Some(('u', stripped.trim_matches('"')))
                } else if let Some(stripped) = part.strip_prefix("response=") {
                    Some(('r', stripped.trim_matches('"')))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .first()
            .and_then(|(k, v)| {
                if *k == 'u' {
                    Some((v.to_string(), String::new()))
                } else {
                    None
                }
            })
    } else {
        None
    }
}

/// 解析 SDP 媒体信息
fn parse_sdp_media(sdp: &str) -> Option<(String, u16)> {
    let mut ip = None;
    let mut port = None;

    for line in sdp.lines() {
        if let Some(stripped) = line.strip_prefix("c=IN IP4 ") {
            ip = Some(stripped.to_string());
        } else if line.starts_with("m=video ") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                port = parts[1].parse().ok();
            }
        }
    }

    ip.zip(port)
}

/// SIP 服务器错误
#[derive(Debug, thiserror::Error)]
pub enum SipServerError {
    #[error("Bind failed: {0}")]
    BindFailed(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Invalid message: {0}")]
    InvalidMessage(String),

    #[error("Authentication failed")]
    AuthFailed,

    #[error("Session not found: {0}")]
    SessionNotFound(String),

    #[error("Device not found: {0}")]
    DeviceNotFound(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_device_id() {
        assert_eq!(
            extract_device_id("<sip:11123456789012345678@3402000000>"),
            Some("11123456789012345678".to_string())
        );

        assert_eq!(
            extract_device_id("sip:11123456789012345678@3402000000"),
            Some("11123456789012345678".to_string())
        );
    }

    #[test]
    fn test_parse_sdp_media() {
        let sdp = r#"v=0
o=- 0 0 IN IP4 192.168.1.100
s=Play
c=IN IP4 192.168.1.100
t=0 0
m=video 50000 RTP/AVP 96"#;

        let result = parse_sdp_media(sdp);
        let (ip, port) = result.expect("parse_sdp_media should return Some");
        assert_eq!(ip, "192.168.1.100");
        assert_eq!(port, 50000);
    }

    #[tokio::test]
    async fn test_sip_server_creation() {
        let config = Gb28181Config::default();
        let server = Gb28181SipServer::new(config);

        assert_eq!(server.get_state().await, SipServerState::Initializing);
    }

    #[tokio::test]
    async fn test_registered_device_count() {
        let config = Gb28181Config::default();
        let server = Gb28181SipServer::new(config);

        assert_eq!(server.get_registered_device_count().await, 0);
    }
}
