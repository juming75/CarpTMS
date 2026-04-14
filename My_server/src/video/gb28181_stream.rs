//! / GB28181视频流处理器

use super::rtp::RtpPacket;
use crate::protocols::gb28181::{GB28181Protocol, GB28181SipMessage, SipMethod};
use log::{debug, info};
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::sync::RwLock;

/// GB28181视频流处理器
pub struct Gb28181StreamHandler {
    #[allow(dead_code)]
    protocol: GB28181Protocol,
    /// 会话管理 (call_id -> session)
    sessions: Arc<RwLock<HashMap<String, StreamSession>>>,
    /// RTP序列号 (ssrc -> sequence)
    rtp_sequences: Arc<RwLock<HashMap<u32, u16>>>,
    /// RTP时间戳 (ssrc -> timestamp)
    #[allow(dead_code)]
    rtp_timestamps: Arc<RwLock<HashMap<u32, u32>>>,
}

/// 视频流会话
#[derive(Debug, Clone)]
struct StreamSession {
    /// 设备ID
    device_id: String,
    /// 通道ID
    channel_id: u8,
    /// SSRC
    ssrc: u32,
    /// 服务器IP
    server_ip: String,
    /// 服务器端口
    server_port: u16,
    /// 设备IP
    device_ip: Option<String>,
    /// 设备端口
    device_port: Option<u16>,
    /// 会话状态
    state: SessionState,
}

/// 会话状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum SessionState {
    /// 等待INVITE响应
    InviteSent,
    /// 正在建立
    Establishing,
    /// 已建立
    Established,
    /// 正在关闭
    Closing,
    /// 已关闭
    Closed,
}

impl Gb28181StreamHandler {
    /// 创建新的GB28181流处理器
    pub fn new(_sip_port: u16, _rtp_port_start: u16, _rtp_port_end: u16) -> Self {
        Self {
            protocol: GB28181Protocol::new(),
            sessions: Arc::new(RwLock::new(HashMap::new())),
            rtp_sequences: Arc::new(RwLock::new(HashMap::new())),
            rtp_timestamps: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 处理SIP消息
    pub async fn process_sip_message(&self, data: &[u8]) -> Result<SipMessage, StreamError> {
        let message_str = String::from_utf8_lossy(data);
        let sip_message = GB28181SipMessage::from_str(&message_str)
            .map_err(|e| StreamError::ParseError(e.to_string()))?;

        debug!("Processing SIP message: method={:?}", sip_message.method);

        match sip_message.method {
            SipMethod::Invite => {
                // 处理INVITE请求
                self.handle_invite(&sip_message).await?;
            }
            SipMethod::Ack => {
                // 处理ACK确认
                self.handle_ack(&sip_message).await?;
            }
            SipMethod::Bye => {
                // 处理BYE请求
                self.handle_bye(&sip_message).await?;
            }
            SipMethod::Register => {
                // 处理REGISTER请求
                info!("Received REGISTER request from {}", sip_message.from);
            }
            SipMethod::Message => {
                // 处理MESSAGE请求
                self.handle_message(&sip_message).await?;
            }
            _ => {
                debug!("Unhandled SIP method: {:?}", sip_message.method);
            }
        }

        Ok(SipMessage {
            method: sip_message.method,
            call_id: sip_message.call_id,
            from: sip_message.from,
            to: sip_message.to,
        })
    }

    /// 处理INVITE请求
    async fn handle_invite(&self, message: &GB28181SipMessage) -> Result<(), StreamError> {
        debug!("Handling INVITE request: call_id={}", message.call_id);

        // 提取设备ID
        let device_id = message
            .from
            .strip_prefix("<sip:")
            .and_then(|s| s.strip_suffix('>'))
            .and_then(|s| s.split('@').next())
            .ok_or_else(|| StreamError::InvalidData("Missing device ID".to_string()))?;

        // 提取SDP信息
        let sdp = message
            .sdp
            .as_ref()
            .ok_or_else(|| StreamError::InvalidData("Missing SDP".to_string()))?;

        // 解析SDP获取IP和端口
        let (device_ip, device_port) = self.parse_sdp_for_connection(sdp)?;

        // 创建新会话
        let ssrc = self.generate_ssrc(device_id);
        let session = StreamSession {
            device_id: device_id.to_string(),
            channel_id: 1, // 默认通道
            ssrc,
            server_ip: "0.0.0.0".to_string(),
            server_port: 50000, // 默认RTP端口
            device_ip: Some(device_ip),
            device_port: Some(device_port),
            state: SessionState::InviteSent,
        };

        let mut sessions = self.sessions.write().await;
        sessions.insert(message.call_id.clone(), session);

        info!("Created new session for device {}", device_id);

        Ok(())
    }

    /// 处理ACK确认
    async fn handle_ack(&self, message: &GB28181SipMessage) -> Result<(), StreamError> {
        debug!("Handling ACK: call_id={}", message.call_id);

        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(&message.call_id) {
            session.state = SessionState::Established;
            info!("Session {} established", message.call_id);
        }

        Ok(())
    }

    /// 处理BYE请求
    async fn handle_bye(&self, message: &GB28181SipMessage) -> Result<(), StreamError> {
        debug!("Handling BYE: call_id={}", message.call_id);

        let mut sessions = self.sessions.write().await;
        if let Some(mut session) = sessions.remove(&message.call_id) {
            session.state = SessionState::Closed;
            info!("Session {} closed", message.call_id);
        }

        Ok(())
    }

    /// 处理MESSAGE请求
    async fn handle_message(&self, message: &GB28181SipMessage) -> Result<(), StreamError> {
        debug!("Handling MESSAGE: call_id={}", message.call_id);

        // GB28181的MESSAGE通常包含设备状态、报警信息等
        // TODO: 解析消息内容

        Ok(())
    }

    /// 处理RTP包
    pub async fn process_rtp_packet(&self, data: &[u8]) -> Result<RtpPacket, StreamError> {
        let packet = RtpPacket::from_bytes(data)
            .ok_or_else(|| StreamError::InvalidData("Invalid RTP packet".to_string()))?;

        debug!(
            "Received RTP packet: ssrc={}, seq={}, size={}",
            packet.ssrc,
            packet.sequence,
            packet.payload.len()
        );

        Ok(packet)
    }

    /// 将H.264数据打包为RTP包
    pub async fn h264_to_rtp(&self, ssrc: u32, h264_data: &[u8], timestamp: u32) -> Vec<RtpPacket> {
        const MAX_PAYLOAD: usize = 1400;

        let mut rtp_packets = Vec::new();
        let total_packets = h264_data.len().div_ceil(MAX_PAYLOAD);

        // 获取或初始化序列号
        let mut sequences = self.rtp_sequences.write().await;
        let sequence = sequences.entry(ssrc).or_insert(0);

        for i in 0..total_packets {
            let offset = i * MAX_PAYLOAD;
            let is_last = i == total_packets - 1;
            let chunk_size = if is_last {
                h264_data.len() - offset
            } else {
                MAX_PAYLOAD
            };

            let chunk = &h264_data[offset..offset + chunk_size];

            // 创建RTP包
            let packet = RtpPacket::new(96, *sequence, timestamp, ssrc)
                .with_marker(is_last)
                .with_payload(chunk.to_vec());

            rtp_packets.push(packet);
            *sequence = sequence.wrapping_add(1);
        }

        rtp_packets
    }

    /// 发送RTP包
    pub async fn send_rtp(
        &self,
        call_id: &str,
        rtp_packets: Vec<RtpPacket>,
    ) -> Result<(), StreamError> {
        let sessions = self.sessions.read().await;
        let session = sessions
            .get(call_id)
            .ok_or_else(|| StreamError::SessionNotFound(call_id.to_string()))?;

        // 绑定UDP socket
        let socket = UdpSocket::bind((session.server_ip.as_str(), session.server_port))
            .await
            .map_err(|e| StreamError::NetworkError(e.to_string()))?;

        let device_ip = session
            .device_ip
            .as_ref()
            .ok_or_else(|| StreamError::InvalidData("Device IP not set".to_string()))?;
        let device_port = session
            .device_port
            .ok_or_else(|| StreamError::InvalidData("Device port not set".to_string()))?;

        // 发送所有RTP包
        for packet in rtp_packets {
            let data = packet.to_bytes();
            socket
                .send_to(&data, (device_ip.as_str(), device_port))
                .await
                .map_err(|e| StreamError::NetworkError(e.to_string()))?;
        }

        Ok(())
    }

    /// 获取会话信息
    pub async fn get_session(&self, call_id: &str) -> Option<StreamSessionInfo> {
        let sessions = self.sessions.read().await;
        sessions.get(call_id).map(|s| StreamSessionInfo {
            device_id: s.device_id.clone(),
            channel_id: s.channel_id,
            ssrc: s.ssrc,
            state: s.state,
        })
    }

    /// 获取所有会话
    pub async fn get_all_sessions(&self) -> Vec<StreamSessionInfo> {
        let sessions = self.sessions.read().await;
        sessions
            .values()
            .map(|s| StreamSessionInfo {
                device_id: s.device_id.clone(),
                channel_id: s.channel_id,
                ssrc: s.ssrc,
                state: s.state,
            })
            .collect()
    }

    /// 解析SDP获取连接信息
    fn parse_sdp_for_connection(&self, sdp: &str) -> Result<(String, u16), StreamError> {
        let mut ip = String::new();
        let mut port: Option<u16> = None;

        for line in sdp.lines() {
            if let Some(stripped) = line.strip_prefix("c=IN IP4 ") {
                ip = stripped.to_string();
            } else if line.starts_with("m=video ") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() > 1 {
                    port = parts[1].parse().ok();
                }
            }

            if !ip.is_empty() && port.is_some() {
                break;
            }
        }

        let port =
            port.ok_or_else(|| StreamError::InvalidData("Port not found in SDP".to_string()))?;
        Ok((ip, port))
    }

    /// 生成SSRC
    fn generate_ssrc(&self, device_id: &str) -> u32 {
        // 使用设备ID的哈希作为SSRC
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        device_id.hash(&mut hasher);
        hasher.finish() as u32
    }
}

impl Default for Gb28181StreamHandler {
    fn default() -> Self {
        Self::new(5060, 10000, 10100)
    }
}

/// SIP消息简化版本
#[derive(Debug, Clone)]
pub struct SipMessage {
    pub method: SipMethod,
    pub call_id: String,
    pub from: String,
    pub to: String,
}

/// 流会话信息
#[derive(Debug, Clone)]
pub struct StreamSessionInfo {
    pub device_id: String,
    pub channel_id: u8,
    pub ssrc: u32,
    pub state: SessionState,
}

/// 流处理错误
#[derive(Debug, thiserror::Error)]
pub enum StreamError {
    #[error("Parse error: {0}")]
    ParseError(String),
    #[error("Invalid data: {0}")]
    InvalidData(String),
    #[error("Network error: {0}")]
    NetworkError(String),
    #[error("Session not found: {0}")]
    SessionNotFound(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_process_sip_invite() {
        let handler = Gb28181StreamHandler::new(5060, 4000, 5000);

        let invite_message = r#"INVITE sip:1112345678@3402000000 SIP/2.0
Via: SIP/2.0/UDP 192.168.1.100:5060
From: <sip:1112345678@3402000000>
To: <sip:3402000000@3402000000>
Call-ID: 12345678@192.168.1.100
CSeq: 1 INVITE
Content-Type: application/sdp
Content-Length: 0

v=0
o=- 0 0 IN IP4 192.168.1.100
s=Play
c=IN IP4 192.168.1.100
t=0 0
m=video 50000 RTP/AVP 96
a=rtpmap:96 H264/90000"#;

        let result = handler.process_sip_message(invite_message.as_bytes()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_h264_to_rtp() {
        let handler = Gb28181StreamHandler::new(5060, 4000, 5000);

        let h264_data = vec![0u8; 1500];
        let rtp_packets = handler.h264_to_rtp(0x12345678, &h264_data, 0).await;

        assert!(!rtp_packets.is_empty());
        assert_eq!(rtp_packets.len(), 2);
    }
}
