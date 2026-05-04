//! / 会话管理器
use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

/// 会话信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub session_id: String,
    pub user_id: String,
    pub connection_id: String,
    pub login_time: DateTime<Utc>,
    pub last_heartbeat: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub client_ip: Option<String>,
    pub client_version: Option<String>,
}

/// 会话管理器
pub struct SessionManager {
    pool: PgPool,
    session_timeout: Duration,
}

impl SessionManager {
    /// 创建新的会话管理器
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            session_timeout: Duration::hours(24),
        }
    }

    /// 创建会话
    pub async fn create_session(
        &self,
        user_id: &str,
        connection_id: &str,
        client_ip: Option<&str>,
        client_version: Option<&str>,
    ) -> Result<Session> {
        let session_id = Uuid::new_v4().to_string();
        let now = Utc::now();
        let expires_at = now + self.session_timeout;

        sqlx::query(
            "INSERT INTO truck_scale_sessions 
                (session_id, user_id, connection_id, login_time, last_heartbeat, expires_at, client_ip, client_version)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"
        )
        .bind(&session_id)
        .bind(user_id)
        .bind(connection_id)
        .bind(now)
        .bind(now)
        .bind(expires_at)
        .bind(client_ip)
        .bind(client_version)
        .execute(&self.pool)
        .await?;

        Ok(Session {
            session_id: session_id.clone(),
            user_id: user_id.to_string(),
            connection_id: connection_id.to_string(),
            login_time: now,
            last_heartbeat: now,
            expires_at,
            client_ip: client_ip.map(|s| s.to_string()),
            client_version: client_version.map(|s| s.to_string()),
        })
    }

    /// 获取会话
    pub async fn get_session(&self, session_id: &str) -> Result<Option<Session>> {
        let session = sqlx::query_as::<_, (String, String, String, DateTime<Utc>, DateTime<Utc>, DateTime<Utc>, Option<String>, Option<String>)>(
            "SELECT session_id, user_id, connection_id, login_time, last_heartbeat, expires_at, client_ip, client_version
             FROM truck_scale_sessions 
             WHERE session_id = $1 AND status = 0"
        )
        .bind(session_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(session.map(|s| Session {
            session_id: s.0,
            user_id: s.1,
            connection_id: s.2,
            login_time: s.3,
            last_heartbeat: s.4,
            expires_at: s.5,
            client_ip: s.6,
            client_version: s.7,
        }))
    }

    /// 更新会话心跳时间
    pub async fn update_heartbeat(&self, session_id: &str) -> Result<()> {
        sqlx::query(
            "UPDATE truck_scale_sessions 
             SET last_heartbeat = CURRENT_TIMESTAMP 
             WHERE session_id = $1 AND status = 0",
        )
        .bind(session_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// 检查会话是否有效
    pub async fn is_session_valid(&self, session_id: &str) -> Result<bool> {
        let now = Utc::now();

        let count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) 
             FROM truck_scale_sessions 
             WHERE session_id = $1 
               AND status = 0 
               AND expires_at > $2",
        )
        .bind(session_id)
        .bind(now)
        .fetch_one(&self.pool)
        .await?;

        Ok(count > 0)
    }

    /// 注销会话
    pub async fn logout_session(&self, session_id: &str) -> Result<()> {
        sqlx::query(
            "UPDATE truck_scale_sessions 
             SET status = 1, logout_time = CURRENT_TIMESTAMP 
             WHERE session_id = $1",
        )
        .bind(session_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// 获取用户的所有会话
    pub async fn get_user_sessions(&self, user_id: &str) -> Result<Vec<Session>> {
        let sessions = sqlx::query_as::<_, (String, String, String, DateTime<Utc>, DateTime<Utc>, DateTime<Utc>, Option<String>, Option<String>)>(
            "SELECT session_id, user_id, connection_id, login_time, last_heartbeat, expires_at, client_ip, client_version
             FROM truck_scale_sessions 
             WHERE user_id = $1 AND status = 0"
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(sessions
            .into_iter()
            .map(|s| Session {
                session_id: s.0,
                user_id: s.1,
                connection_id: s.2,
                login_time: s.3,
                last_heartbeat: s.4,
                expires_at: s.5,
                client_ip: s.6,
                client_version: s.7,
            })
            .collect())
    }

    /// 清理过期会话
    pub async fn cleanup_expired_sessions(&self) -> Result<u64> {
        let now = Utc::now();

        let result = sqlx::query(
            "UPDATE truck_scale_sessions 
             SET status = 2, logout_time = CURRENT_TIMESTAMP 
             WHERE expires_at < $1 AND status = 0",
        )
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }

    /// 获取所有活动会话
    pub async fn get_active_sessions(&self) -> Result<Vec<Session>> {
        let sessions = sqlx::query_as::<_, (String, String, String, DateTime<Utc>, DateTime<Utc>, DateTime<Utc>, Option<String>, Option<String>)>(
            "SELECT session_id, user_id, connection_id, login_time, last_heartbeat, expires_at, client_ip, client_version
             FROM truck_scale_sessions 
             WHERE status = 0"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(sessions
            .into_iter()
            .map(|s| Session {
                session_id: s.0,
                user_id: s.1,
                connection_id: s.2,
                login_time: s.3,
                last_heartbeat: s.4,
                expires_at: s.5,
                client_ip: s.6,
                client_version: s.7,
            })
            .collect())
    }

    /// 获取会话统计信息
    pub async fn get_session_stats(&self) -> Result<SessionStats> {
        let active_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM truck_scale_sessions WHERE status = 0")
                .fetch_one(&self.pool)
                .await?;

        let expired_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM truck_scale_sessions WHERE status = 2")
                .fetch_one(&self.pool)
                .await?;

        let logged_out_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM truck_scale_sessions WHERE status = 1")
                .fetch_one(&self.pool)
                .await?;

        Ok(SessionStats {
            active_count,
            expired_count,
            logged_out_count,
            total_count: active_count + expired_count + logged_out_count,
        })
    }
}

/// 会话统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStats {
    pub active_count: i64,
    pub expired_count: i64,
    pub logged_out_count: i64,
    pub total_count: i64,
}
