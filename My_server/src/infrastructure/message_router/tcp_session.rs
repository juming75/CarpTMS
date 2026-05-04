//! / TCP 会话管理器
// 管理所有 TCP 设备连接,支持向指定设备下发指令

use actix::prelude::*;
use log::{debug, info, warn};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// TCP 会话
#[derive(Debug, Clone)]
pub struct TcpSession {
    /// 设备ID
    pub device_id: String,
    /// Socket 地址
    pub addr: SocketAddr,
    /// 协议类型 (JT808/GB/BSJ/DB44)
    pub protocol: String,
    /// 连接时间
    pub connected_at: Instant,
    /// 最后心跳时间
    pub last_heartbeat: Instant,
    /// 流水号计数器
    pub flow_no: u16,
    /// 认证状态
    pub authenticated: bool,
    /// 终端手机号
    pub phone: Option<String>,
    /// 会话数据 (存储自定义数据)
    pub data: HashMap<String, String>,
}

impl TcpSession {
    pub fn new(device_id: String, addr: SocketAddr, protocol: String) -> Self {
        let now = Instant::now();
        Self {
            device_id,
            addr,
            protocol,
            connected_at: now,
            last_heartbeat: now,
            flow_no: 0,
            authenticated: false,
            phone: None,
            data: HashMap::new(),
        }
    }

    /// 更新心跳时间
    pub fn update_heartbeat(&mut self) {
        self.last_heartbeat = Instant::now();
    }

    /// 检查心跳是否超时
    pub fn is_heartbeat_timeout(&self, timeout: Duration) -> bool {
        self.last_heartbeat.elapsed() > timeout
    }

    /// 获取下一个流水号
    pub fn next_flow_no(&mut self) -> u16 {
        self.flow_no = self.flow_no.wrapping_add(1);
        self.flow_no
    }

    /// 设置认证状态
    pub fn set_authenticated(&mut self, authenticated: bool) {
        self.authenticated = authenticated;
    }

    /// 设置终端手机号
    pub fn set_phone(&mut self, phone: String) {
        self.phone = Some(phone);
    }
}

/// TCP 会话管理器
pub struct TcpSessionManager {
    /// 会话列表: key=设备ID, value=TcpSession
    sessions: Arc<RwLock<HashMap<String, TcpSession>>>,
    /// 设备ID映射: key=电话号码, value=设备ID
    phone_to_device: Arc<RwLock<HashMap<String, String>>>,
    /// 心跳超时时间
    heartbeat_timeout: Duration,
    /// 是否启用心跳监控
    heartbeat_monitor_enabled: bool,
}

impl TcpSessionManager {
    pub fn new(heartbeat_timeout: Duration) -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            phone_to_device: Arc::new(RwLock::new(HashMap::new())),
            heartbeat_timeout,
            heartbeat_monitor_enabled: true,
        }
    }

    /// 创建默认会话管理器 (心跳超时 5 分钟)
    pub fn with_default_timeout() -> Self {
        Self::new(Duration::from_secs(300))
    }

    /// 添加会话
    pub async fn add_session(&self, session: TcpSession) -> Result<(), String> {
        let device_id = session.device_id.clone();
        let phone = session.phone.clone();
        let protocol = session.protocol.clone();
        let addr = session.addr;

        let mut sessions = self.sessions.write().await;
        sessions.insert(device_id.clone(), session.clone());

        // 更新电话号码映射
        if let Some(phone) = phone {
            let mut phone_map = self.phone_to_device.write().await;
            phone_map.insert(phone, device_id.clone());
        }

        info!(
            "TCP session added: device_id={}, protocol={}, addr={}",
            device_id, protocol, addr
        );

        Ok(())
    }

    /// 移除会话
    pub async fn remove_session(&self, device_id: &str) -> Option<TcpSession> {
        let mut sessions = self.sessions.write().await;
        let session = sessions.remove(device_id);

        if let Some(ref session) = session {
            // 清理电话号码映射
            if let Some(phone) = &session.phone {
                let mut phone_map = self.phone_to_device.write().await;
                phone_map.remove(phone);
            }

            info!("TCP session removed: device_id={}", device_id);
        }

        session
    }

    /// 获取会话
    pub async fn get_session(&self, device_id: &str) -> Option<TcpSession> {
        let sessions = self.sessions.read().await;
        sessions.get(device_id).cloned()
    }

    /// 根据电话号码获取会话
    pub async fn get_session_by_phone(&self, phone: &str) -> Option<TcpSession> {
        let phone_map = self.phone_to_device.read().await;
        let device_id_opt = phone_map.get(phone).cloned();
        drop(phone_map);

        if let Some(device_id) = device_id_opt {
            self.get_session(&device_id).await
        } else {
            None
        }
    }

    /// 更新会话心跳
    pub async fn update_heartbeat(&self, device_id: &str) -> Result<(), String> {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(device_id) {
            session.update_heartbeat();
            debug!("Heartbeat updated for device: {}", device_id);
            Ok(())
        } else {
            Err(format!("Session not found: {}", device_id))
        }
    }

    /// 检查并清理超时会话
    pub async fn cleanup_timeout_sessions(&self) -> usize {
        let mut sessions = self.sessions.write().await;
        let mut phone_map = self.phone_to_device.write().await;

        let mut to_remove = Vec::new();

        for (device_id, session) in sessions.iter() {
            if session.is_heartbeat_timeout(self.heartbeat_timeout) {
                to_remove.push(device_id.clone());
            }
        }

        for device_id in &to_remove {
            if let Some(session) = sessions.remove(device_id) {
                if let Some(phone) = session.phone {
                    phone_map.remove(&phone);
                }
                info!(
                    "TCP session timeout removed: device_id={}, addr={}",
                    device_id, session.addr
                );
            }
        }

        debug!("Cleaned up {} timeout sessions", to_remove.len());
        to_remove.len()
    }

    /// 获取所有会话
    pub async fn get_all_sessions(&self) -> Vec<TcpSession> {
        let sessions = self.sessions.read().await;
        sessions.values().cloned().collect()
    }

    /// 获取会话数量
    pub async fn session_count(&self) -> usize {
        self.sessions.read().await.len()
    }

    /// 检查设备是否在线
    pub async fn is_online(&self, device_id: &str) -> bool {
        self.sessions.read().await.contains_key(device_id)
    }

    /// 根据协议类型获取会话
    pub async fn get_sessions_by_protocol(&self, protocol: &str) -> Vec<TcpSession> {
        let sessions = self.sessions.read().await;
        sessions
            .values()
            .filter(|s| s.protocol == protocol)
            .cloned()
            .collect()
    }

    /// 发送指令到设备 (返回流水号)
    pub async fn send_command(
        &self,
        device_id: &str,
        command_data: Vec<u8>,
    ) -> Result<(u16, Option<SocketAddr>), String> {
        let mut sessions = self.sessions.write().await;

        if let Some(session) = sessions.get_mut(device_id) {
            let flow_no = session.next_flow_no();
            let addr = session.addr;

            // 注意:这里只返回流水号和地址,实际发送需要在连接层完成
            debug!(
                "Command prepared for device: {}, flow_no={}, data_len={}",
                device_id,
                flow_no,
                command_data.len()
            );

            Ok((flow_no, Some(addr)))
        } else {
            Err(format!("Session not found: {}", device_id))
        }
    }

    /// 启动心跳监控
    pub async fn start_heartbeat_monitor(&self) {
        if !self.heartbeat_monitor_enabled {
            return;
        }

        let sessions = self.sessions.clone();
        let heartbeat_timeout = self.heartbeat_timeout;

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));

            loop {
                interval.tick().await;

                let mut sessions_write = sessions.write().await;
                let mut to_remove = Vec::new();

                for (device_id, session) in sessions_write.iter_mut() {
                    if session.is_heartbeat_timeout(heartbeat_timeout) {
                        to_remove.push(device_id.clone());
                        warn!(
                            "Device heartbeat timeout: device_id={}, last_heartbeat={:?}",
                            device_id,
                            session.last_heartbeat.elapsed()
                        );
                    }
                }

                for device_id in &to_remove {
                    sessions_write.remove(device_id);
                    info!("Removed timeout session: {}", device_id);
                }

                if !to_remove.is_empty() {
                    info!("Heartbeat monitor: cleaned up {} sessions", to_remove.len());
                }
            }
        });

        info!("Heartbeat monitor started");
    }

    /// 设置心跳监控开关
    pub fn set_heartbeat_monitor_enabled(&mut self, enabled: bool) {
        self.heartbeat_monitor_enabled = enabled;
    }

    /// 获取会话统计信息
    pub async fn get_statistics(&self) -> TcpSessionStatistics {
        let sessions = self.sessions.read().await;

        let total = sessions.len();
        let mut authenticated = 0;
        let mut unauthenticated = 0;
        let mut by_protocol: HashMap<String, usize> = HashMap::new();

        for session in sessions.values() {
            if session.authenticated {
                authenticated += 1;
            } else {
                unauthenticated += 1;
            }

            *by_protocol.entry(session.protocol.clone()).or_insert(0) += 1;
        }

        TcpSessionStatistics {
            total,
            authenticated,
            unauthenticated,
            by_protocol,
        }
    }
}

impl Default for TcpSessionManager {
    fn default() -> Self {
        Self::new(Duration::from_secs(300))
    }
}

/// TCP 会话统计信息
#[derive(Debug, Clone, serde::Serialize)]
pub struct TcpSessionStatistics {
    /// 总会话数
    pub total: usize,
    /// 已认证会话数
    pub authenticated: usize,
    /// 未认证会话数
    pub unauthenticated: usize,
    /// 按协议类型统计
    pub by_protocol: HashMap<String, usize>,
}

/// 发送指令到 TCP 设备的消息
#[derive(Message)]
#[rtype(result = "Result<(u16, Option<SocketAddr>), String>")]
pub struct SendTcpCommand {
    pub device_id: String,
    pub command_data: Vec<u8>,
}

/// Actor 实现
impl Actor for TcpSessionManager {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        info!("TCP session manager started");

        // 启动心跳监控
        let sessions = self.sessions.clone();
        let heartbeat_timeout = self.heartbeat_timeout;

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));

            loop {
                interval.tick().await;

                let mut sessions_write = sessions.write().await;
                let mut to_remove = Vec::new();

                for (device_id, session) in sessions_write.iter_mut() {
                    if session.is_heartbeat_timeout(heartbeat_timeout) {
                        to_remove.push(device_id.clone());
                        warn!(
                            "Device heartbeat timeout: device_id={}, last_heartbeat={:?}",
                            device_id,
                            session.last_heartbeat.elapsed()
                        );
                    }
                }

                for device_id in &to_remove {
                    sessions_write.remove(device_id);
                    info!("Removed timeout session: {}", device_id);
                }

                if !to_remove.is_empty() {
                    info!("Heartbeat monitor: cleaned up {} sessions", to_remove.len());
                }
            }
        });

        info!("Heartbeat monitor started");
    }
}

impl Handler<SendTcpCommand> for TcpSessionManager {
    type Result = Result<(u16, Option<SocketAddr>), String>;

    fn handle(&mut self, msg: SendTcpCommand, _ctx: &mut Self::Context) -> Self::Result {
        let device_id = msg.device_id;

        // 转换为异步操作
        let sessions = self.sessions.clone();
        let device_id_clone = device_id.clone();
        let command_data = msg.command_data;

        tokio::spawn(async move {
            let mut sessions_write = sessions.write().await;

            if let Some(session) = sessions_write.get_mut(&device_id_clone) {
                let flow_no = session.next_flow_no();
                let addr = session.addr;

                debug!(
                    "Command prepared for device: {}, flow_no={}, data_len={}",
                    device_id_clone,
                    flow_no,
                    command_data.len()
                );

                Ok((flow_no, Some(addr)))
            } else {
                Err(format!("Session not found: {}", device_id_clone))
            }
        });

        // 返回占位符,实际结果需要通过其他方式返回
        Err("Command queued".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};

    #[test]
    fn test_tcp_session_creation() {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
        let session = TcpSession::new("test_device".to_string(), addr, "JT808".to_string());

        assert_eq!(session.device_id, "test_device");
        assert_eq!(session.protocol, "JT808");
        assert!(!session.authenticated);
        assert_eq!(session.flow_no, 0);
    }

    #[test]
    fn test_flow_no_wrapping() {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
        let mut session = TcpSession::new("test_device".to_string(), addr, "JT808".to_string());

        session.flow_no = 0xFFFF;

        assert_eq!(session.next_flow_no(), 0);
        assert_eq!(session.flow_no, 0);
    }
}
