//! / 会话管理
use chrono::{DateTime, Duration, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// 会话信息
#[derive(Debug, Clone)]
pub struct Session {
    pub session_id: String,
    pub user_id: String,
    pub connection_id: String,
    pub login_time: DateTime<Utc>,
    pub last_heartbeat: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub client_ip: String,
    pub client_version: String,
}

/// 会话管理器
pub struct SessionManager {
    sessions: Arc<RwLock<HashMap<String, Session>>>,
    connection_to_session: Arc<RwLock<HashMap<String, String>>>,
    user_to_sessions: Arc<RwLock<HashMap<String, Vec<String>>>>,
}

impl SessionManager {
    /// 创建新的会话管理器
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            connection_to_session: Arc::new(RwLock::new(HashMap::new())),
            user_to_sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 创建新会话
    pub async fn create_session(
        &self,
        user_id: String,
        connection_id: String,
        client_ip: String,
        client_version: String,
        session_timeout: i64, // 秒
    ) -> Result<Session, String> {
        let session_id = Uuid::new_v4().to_string();
        let now = Utc::now();
        let expires_at = now + Duration::seconds(session_timeout);

        let session = Session {
            session_id: session_id.clone(),
            user_id: user_id.clone(),
            connection_id: connection_id.clone(),
            login_time: now,
            last_heartbeat: now,
            expires_at,
            client_ip,
            client_version,
        };

        // 保存会话
        {
            let mut sessions = self.sessions.write().await;
            sessions.insert(session_id.clone(), session.clone());
        }

        // 建立连接与会话的映射
        {
            let mut conn_to_session = self.connection_to_session.write().await;
            conn_to_session.insert(connection_id, session_id.clone());
        }

        // 建立用户与会话的映射
        {
            let mut user_to_sessions = self.user_to_sessions.write().await;
            user_to_sessions
                .entry(user_id)
                .or_insert_with(Vec::new)
                .push(session_id.clone());
        }

        println!(
            "Session created: {} for user: {}",
            session_id, session.user_id
        );
        Ok(session)
    }

    /// 获取会话
    pub async fn get_session(&self, session_id: &str) -> Option<Session> {
        let sessions = self.sessions.read().await;
        sessions.get(session_id).cloned()
    }

    /// 通过连接ID获取会话
    pub async fn get_session_by_connection(&self, connection_id: &str) -> Option<Session> {
        let session_id = {
            let conn_to_session = self.connection_to_session.read().await;
            conn_to_session.get(connection_id).cloned()
        };
        if let Some(session_id) = session_id {
            return self.get_session(&session_id).await;
        }
        None
    }

    /// 更新心跳
    pub async fn update_heartbeat(&self, session_id: &str) -> Result<(), String> {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(session_id) {
            session.last_heartbeat = Utc::now();
            Ok(())
        } else {
            Err("Session not found".to_string())
        }
    }

    /// 删除会话
    pub async fn remove_session(&self, session_id: &str) -> Option<Session> {
        let mut sessions = self.sessions.write().await;
        let session = sessions.remove(session_id);

        if let Some(ref s) = session {
            // 删除连接与会话的映射
            let mut conn_to_session = self.connection_to_session.write().await;
            conn_to_session.remove(&s.connection_id);

            // 删除用户与会话的映射
            let mut user_to_sessions = self.user_to_sessions.write().await;
            if let Some(session_list) = user_to_sessions.get_mut(&s.user_id) {
                session_list.retain(|id| id != session_id);
                if session_list.is_empty() {
                    user_to_sessions.remove(&s.user_id);
                }
            }

            println!("Session removed: {}", session_id);
        }

        session
    }

    /// 清理过期会话
    pub async fn cleanup_expired_sessions(&self) -> Vec<Session> {
        let now = Utc::now();
        let mut expired_sessions = Vec::new();

        {
            let sessions = self.sessions.read().await;
            for (_, session) in sessions.iter() {
                if session.expires_at < now {
                    expired_sessions.push(session.clone());
                }
            }
        }

        for session in &expired_sessions {
            self.remove_session(&session.session_id).await;
            println!("Expired session removed: {}", session.session_id);
        }

        expired_sessions
    }

    /// 获取用户的所有会话
    pub async fn get_user_sessions(&self, user_id: &str) -> Vec<Session> {
        let user_to_sessions = self.user_to_sessions.read().await;
        if let Some(session_ids) = user_to_sessions.get(user_id) {
            let sessions = self.sessions.read().await;
            session_ids
                .iter()
                .filter_map(|id| sessions.get(id).cloned())
                .collect()
        } else {
            Vec::new()
        }
    }

    /// 检查会话是否有效
    pub async fn is_session_valid(&self, session_id: &str) -> bool {
        if let Some(session) = self.get_session(session_id).await {
            session.expires_at > Utc::now()
        } else {
            false
        }
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}
