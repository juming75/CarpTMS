//! / 会话数据库操作
use sqlx::PgPool;
use anyhow::Result;
use chrono::{DateTime, Utc};
use uuid::Uuid;

impl crate::truck_scale::db::TruckScaleDb {
    // ==================== 会话管理 ====================

    /// 创建会话
    pub async fn create_session(
        &self,
        session_id: &str,
        user_id: &str,
        connection_id: &str,
        client_ip: Option<&str>,
        client_version: Option<&str>,
    ) -> Result<()> {
        let now = Utc::now();
        let expires_at = now + chrono::Duration::hours(24);

        sqlx::query(
            "INSERT INTO truck_scale_sessions 
                (session_id, user_id, connection_id, login_time, last_heartbeat, expires_at, client_ip, client_version)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"
        )
        .bind(session_id)
        .bind(user_id)
        .bind(connection_id)
        .bind(now)
        .bind(now)
        .bind(expires_at)
        .bind(client_ip)
        .bind(client_version)
        .execute(self.pool())
        .await?;

        Ok(())
    }

    /// 查询会话
    pub async fn query_session(&self, session_id: &str) -> Result<Option<serde_json::Value>> {
        let session = sqlx::query_as::<_, (
            String, String, String, DateTime<Utc>, DateTime<Utc>, 
            DateTime<Utc>, Option<String>, Option<String>
        )>(
            "SELECT session_id, user_id, connection_id, login_time, last_heartbeat, 
                    expires_at, client_ip, client_version
             FROM truck_scale_sessions 
             WHERE session_id = $1 AND status = 0"
        )
        .bind(session_id)
        .fetch_optional(self.pool())
        .await?;

        Ok(session.map(|s| {
            serde_json::json!({
                "session_id": s.0,
                "user_id": s.1,
                "connection_id": s.2,
                "login_time": s.3,
                "last_heartbeat": s.4,
                "expires_at": s.5,
                "client_ip": s.6,
                "client_version": s.7,
            })
        }))
    }

    /// 更新会话心跳
    pub async fn update_session_heartbeat(&self, session_id: &str) -> Result<()> {
        sqlx::query(
            "UPDATE truck_scale_sessions 
             SET last_heartbeat = CURRENT_TIMESTAMP 
             WHERE session_id = $1 AND status = 0"
        )
        .bind(session_id)
        .execute(self.pool())
        .await?;

        Ok(())
    }

    /// 注销会话
    pub async fn logout_session(&self, session_id: &str) -> Result<()> {
        sqlx::query(
            "UPDATE truck_scale_sessions 
             SET status = 1, logout_time = CURRENT_TIMESTAMP 
             WHERE session_id = $1"
        )
        .bind(session_id)
        .execute(self.pool())
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
               AND expires_at > $2"
        )
        .bind(session_id)
        .bind(now)
        .fetch_one(self.pool())
        .await?;

        Ok(count > 0)
    }

    /// 清理过期会话
    pub async fn cleanup_expired_sessions(&self) -> Result<u64> {
        let now = Utc::now();
        
        let result = sqlx::query(
            "UPDATE truck_scale_sessions 
             SET status = 2, logout_time = CURRENT_TIMESTAMP 
             WHERE expires_at < $1 AND status = 0"
        )
        .bind(now)
        .execute(self.pool())
        .await?;

        Ok(result.rows_affected())
    }
}






