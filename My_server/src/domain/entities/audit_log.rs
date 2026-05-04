use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Deserialize, Serialize, FromRow)]
pub struct AuditLog {
    pub id: i32,
    pub user_id: Option<i32>,
    pub action_type: String,
    pub resource_type: String,
    pub resource_id: Option<i32>,
    pub old_value: Option<serde_json::Value>,
    pub new_value: Option<serde_json::Value>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateAuditLog {
    pub user_id: Option<i32>,
    pub action_type: String,
    pub resource_type: String,
    pub resource_id: Option<i32>,
    pub old_value: Option<serde_json::Value>,
    pub new_value: Option<serde_json::Value>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}
