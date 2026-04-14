//! /! 会话管理模块
//! 负责管理设备连接和会话状态

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

/// 会话状态
#[derive(Debug, Clone)]
pub struct Session {
    /// 设备ID
    pub device_id: String,
    /// 协议类型
    pub protocol_type: super::protocol::ProtocolType,
    /// 连接时间
    pub connect_time: Instant,
    /// 最后活动时间
    pub last_activity: Instant,
    /// 会话数据
    pub data: HashMap<String, serde_json::Value>,
}

impl Session {
    /// 创建新会话
    pub fn new(device_id: String, protocol_type: super::protocol::ProtocolType) -> Self {
        Self {
            device_id,
            protocol_type,
            connect_time: Instant::now(),
            last_activity: Instant::now(),
            data: HashMap::new(),
        }
    }

    /// 更新活动时间
    pub fn update_activity(&mut self) {
        self.last_activity = Instant::now();
    }

    /// 检查会话是否过期
    pub fn is_expired(&self, timeout: Duration) -> bool {
        Instant::now().duration_since(self.last_activity) > timeout
    }
}

/// 会话管理器
#[derive(Debug)]
pub struct SessionManager {
    /// 会话存储
    sessions: Arc<RwLock<HashMap<String, Session>>>,
    /// 会话超时时间
    session_timeout: Duration,
}

impl SessionManager {
    /// 创建新的会话管理器
    pub fn new(session_timeout: Duration) -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            session_timeout,
        }
    }

    /// 创建默认的会话管理器
    pub fn with_default_timeout() -> Self {
        Self::new(Duration::from_secs(3600)) // 默认1小时超时
    }

    /// 创建会话
    pub fn create_session(
        &self,
        device_id: String,
        protocol_type: super::protocol::ProtocolType,
    ) -> Session {
        let session = Session::new(device_id.clone(), protocol_type);
        if let Ok(mut sessions) = self.sessions.write() {
            sessions.insert(device_id, session.clone());
        }
        session
    }

    /// 获取会话
    pub fn get_session(&self, device_id: &str) -> Option<Session> {
        self.sessions.read().ok()?.get(device_id).cloned()
    }

    /// 更新会话
    pub fn update_session(&self, device_id: &str) -> Option<Session> {
        if let Ok(mut sessions) = self.sessions.write() {
            if let Some(session) = sessions.get_mut(device_id) {
                session.update_activity();
                return Some(session.clone());
            }
        }
        None
    }

    /// 删除会话
    pub fn remove_session(&self, device_id: &str) -> Option<Session> {
        self.sessions.write().ok()?.remove(device_id)
    }

    /// 清理过期会话
    pub fn cleanup_expired(&self) -> usize {
        if let Ok(mut sessions) = self.sessions.write() {
            let expired_count = sessions
                .iter()
                .filter(|(_, session)| session.is_expired(self.session_timeout))
                .count();
            sessions.retain(|_, session| !session.is_expired(self.session_timeout));
            expired_count
        } else {
            0
        }
    }

    /// 获取会话数量
    pub fn session_count(&self) -> usize {
        self.sessions.read().ok().map(|s| s.len()).unwrap_or(0)
    }

    /// 获取所有会话
    pub fn get_all_sessions(&self) -> Vec<Session> {
        self.sessions.read().ok()
            .map(|s| s.values().cloned().collect())
            .unwrap_or_default()
    }
}

impl Clone for SessionManager {
    fn clone(&self) -> Self {
        Self {
            sessions: self.sessions.clone(),
            session_timeout: self.session_timeout,
        }
    }
}
