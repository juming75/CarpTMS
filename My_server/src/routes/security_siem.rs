//! SIEM 集成接口 (Elkeid)
//! 提供与 Elkeid 安全平台集成的 API 接口

use actix_web::{web, HttpResponse, post, get};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use log::{error, info, warn};
use sqlx::PgPool;
use std::sync::Arc;

use crate::domain::entities::auth::Claims;
use crate::domain::entities::audit_log::{AuditLog, CreateAuditLog};
use crate::infrastructure::repositories::audit_log_repository::AuditLogRepository;
use crate::security::jwt_blacklist::JwtBlacklist;

/// Elkeid 告警请求
#[derive(Debug, Deserialize)]
pub struct ElkeidAlert {
    pub rule_id: String,
    pub rule_name: String,
    pub severity: String,
    pub timestamp: DateTime<Utc>,
    pub source: String,
    
    pub user: Option<UserInfo>,
    pub resource: Option<ResourceInfo>,
    
    pub message: String,
    pub action_taken: Option<String>,
    pub details: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct UserInfo {
    pub id: Option<i32>,
    pub username: Option<String>,
    pub ip: Option<String>,
    pub user_agent: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ResourceInfo {
    pub r#type: String,
    pub id: String,
    pub name: Option<String>,
}

/// Elkeid 告警响应
#[derive(Debug, Serialize)]
pub struct AlertResponse {
    pub status: String,
    pub message: String,
    pub alert_id: String,
}

/// 安全统计查询
#[derive(Debug, Deserialize)]
pub struct SecurityStatsQuery {
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
}

/// 安全统计响应
#[derive(Debug, Serialize)]
pub struct SecurityStatsResponse {
    pub total_alerts: i64,
    pub critical: i64,
    pub high: i64,
    pub medium: i64,
    pub low: i64,
    pub blocked_users: i64,
    pub blocked_ips: i64,
}

/// 接收 Elkeid 告警回调
#[post("/api/security/elkeid-alert")]
pub async fn receive_elkeid_alert(
    pool: web::Data<Arc<PgPool>>,
    web::Json(alert): web::Json<ElkeidAlert>,
) -> HttpResponse {
    let alert_id = format!("elkeid_{}", Utc::now().timestamp_millis());
    
    info!("收到 Elkeid 告警 [{}]: {} - {}", 
        alert_id, alert.rule_id, alert.message);
    
    // 根据 severity 采取不同的动作
    let result = match alert.severity.as_str() {
        "critical" => handle_critical_alert(&alert, &alert_id, pool.get_ref()).await,
        "high" => handle_high_alert(&alert, &alert_id, pool.get_ref()).await,
        "medium" => handle_medium_alert(&alert, &alert_id, pool.get_ref()).await,
        "low" => handle_low_alert(&alert, &alert_id, pool.get_ref()).await,
        _ => {
            warn!("未知的告警级别: {}", alert.severity);
            Ok(())
        }
    };
    
    match result {
        Ok(_) => {
            HttpResponse::Ok().json(AlertResponse {
                status: "acknowledged".to_string(),
                message: "告警已处理".to_string(),
                alert_id,
            })
        }
        Err(e) => {
            error!("处理告警失败: {}", e);
            HttpResponse::InternalServerError().json(AlertResponse {
                status: "error".to_string(),
                message: "处理告警失败".to_string(),
                alert_id,
            })
        }
    }
}

/// 查询安全统计
#[get("/api/security/stats")]
pub async fn get_security_stats(
    _user: Claims,
    query: web::Query<SecurityStatsQuery>,
    pool: web::Data<Arc<PgPool>>,
) -> HttpResponse {
    let repo = AuditLogRepository::new(pool.get_ref().clone());
    
    let (total_alerts, critical, high, medium, low) = 
        match query_stats_from_db(&repo, query.start_date, query.end_date).await {
        Ok(stats) => stats,
        Err(e) => {
            tracing::warn!("Failed to fetch security stats: {}", e);
            (0, 0, 0, 0, 0)
        }
    };
    
    // 统计被封禁的用户和 IP（从审计日志中查询 "security_ban" 类型的记录）
    let blocked_users = count_security_actions(&repo, "security_ban_user", query.start_date, query.end_date).await.unwrap_or(0);
    let blocked_ips = count_security_actions(&repo, "security_ban_ip", query.start_date, query.end_date).await.unwrap_or(0);

    HttpResponse::Ok().json(SecurityStatsResponse {
        total_alerts,
        critical,
        high,
        medium,
        low,
        blocked_users,
        blocked_ips,
    })
}

// ================= 内部处理函数 =================

async fn handle_critical_alert(
    alert: &ElkeidAlert, 
    alert_id: &str,
    pool: &PgPool,
) -> Result<(), String> {
    warn!("处理严重告警 [{}]: {}", alert_id, alert.message);
    
    // 1. 如果有用户信息，强制登出
    if let Some(user) = &alert.user {
        if let Some(user_id) = user.id {
            force_logout_user(user_id).await?;
            record_security_audit_internal(
                pool, 
                "security_force_logout", 
                alert, 
                &format!("强制登出用户 {}", user_id)
            ).await?;
        }
    }
    
    // 2. 如果有 IP 信息，暂时封禁 IP
    if let Some(user) = &alert.user {
        if let Some(ip) = &user.ip {
            block_ip_address(ip, "30m").await?;
            record_security_audit_internal(
                pool, 
                "security_ban_ip", 
                alert, 
                &format!("封禁 IP {} (30分钟)", ip)
            ).await?;
        }
    }
    
    // 3. 通知管理员
    notify_admins(alert, "critical").await?;
    
    // 4. 记录安全告警
    record_security_audit_internal(
        pool, 
        "elkeid_alert_critical", 
        alert, 
        &format!("Elkeid 严重告警 [{}]", alert_id)
    ).await?;
    
    Ok(())
}

async fn handle_high_alert(
    alert: &ElkeidAlert, 
    alert_id: &str,
    pool: &PgPool,
) -> Result<(), String> {
    info!("处理高级告警 [{}]: {}", alert_id, alert.message);
    
    // 1. 如果有用户信息，要求 MFA 二次验证
    if let Some(user) = &alert.user {
        if let Some(user_id) = user.id {
            require_mfa_for_user(user_id).await?;
            record_security_audit_internal(
                pool, 
                "security_require_mfa", 
                alert, 
                &format!("强制用户 {} 进行 MFA 验证", user_id)
            ).await?;
        }
    }
    
    // 2. 通知管理员
    notify_admins(alert, "high").await?;
    
    // 3. 记录安全告警
    record_security_audit_internal(
        pool, 
        "elkeid_alert_high", 
        alert, 
        &format!("Elkeid 高级告警 [{}]", alert_id)
    ).await?;
    
    Ok(())
}

async fn handle_medium_alert(
    alert: &ElkeidAlert, 
    alert_id: &str,
    pool: &PgPool,
) -> Result<(), String> {
    info!("处理中级告警 [{}]: {}", alert_id, alert.message);
    
    // 记录审计日志
    record_security_audit_internal(
        pool, 
        "elkeid_alert_medium", 
        alert, 
        &format!("Elkeid 中级告警 [{}]", alert_id)
    ).await?;
    
    Ok(())
}

async fn handle_low_alert(
    alert: &ElkeidAlert, 
    alert_id: &str,
    pool: &PgPool,
) -> Result<(), String> {
    info!("处理低级告警 [{}]: {}", alert_id, alert.message);
    
    // 仅记录日志
    record_security_audit_internal(
        pool, 
        "elkeid_alert_low", 
        alert, 
        &format!("Elkeid 低级告警 [{}]", alert_id)
    ).await?;
    
    Ok(())
}

// ================= 辅助函数实现 =================

/// 强制用户登出
async fn force_logout_user(user_id: i32) -> Result<(), String> {
    // 将用户的 token 加入黑名单
    let blacklist = JwtBlacklist::from_env();
    
    // 注意: 实际实现中，需要获取该用户的所有有效 token 并加入黑名单
    // 这里使用用户 ID 作为标记
    let token_marker = format!("user_logout:{}", user_id);
    let ttl_seconds = 86400; // 24小时
    
    blacklist.add_to_blacklist(&token_marker, ttl_seconds).await?;
    
    tracing::info!(user_id = user_id, "用户已被强制登出");
    Ok(())
}

/// 封禁 IP 地址
async fn block_ip_address(ip: &str, duration: &str) -> Result<(), String> {
    // 将 IP 加入 Redis 黑名单
    let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());
    let client = redis::Client::open(redis_url.as_str())
        .map_err(|e| format!("Redis 连接失败: {}", e))?;
    
    let mut conn = client.get_multiplexed_async_connection().await
        .map_err(|e| format!("获取 Redis 连接失败: {}", e))?;
    
    let key = format!("ip_blacklist:{}", ip);
    let ttl_seconds = parse_duration(duration);
    
    let _: () = conn.set_ex(&key, "1", ttl_seconds).await
        .map_err(|e| format!("写入 IP 黑名单失败: {}", e))?;
    
    tracing::warn!(ip = %ip, duration = %duration, "IP 已被封禁");
    Ok(())
}

/// 强制用户进行 MFA 验证
async fn require_mfa_for_user(user_id: i32) -> Result<(), String> {
    // 将用户标记为需要 MFA 验证
    let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());
    let client = redis::Client::open(redis_url.as_str())
        .map_err(|e| format!("Redis 连接失败: {}", e))?;
    
    let mut conn = client.get_multiplexed_async_connection().await
        .map_err(|e| format!("获取 Redis 连接失败: {}", e))?;
    
    let key = format!("require_mfa:{}", user_id);
    let ttl_seconds = 3600; // 1小时
    
    let _: () = conn.set_ex(&key, "1", ttl_seconds).await
        .map_err(|e| format!("设置 MFA 要求失败: {}", e))?;
    
    tracing::info!(user_id = user_id, "用户已被要求进行 MFA 验证");
    Ok(())
}

/// 通知管理员
async fn notify_admins(alert: &ElkeidAlert, level: &str) -> Result<(), String> {
    // 目前只记录日志，未来可扩展为：
    // - 邮件通知
    // - 短信通知
    // - 企业 IM (钉钉/飞书/企业微信)
    
    let severity_emoji = match level {
        "critical" => "🔴",
        "high" => "🟠",
        "medium" => "🟡",
        "low" => "🔵",
        _ => "⚪",
    };
    
    tracing::error!(
        severity = %level,
        rule_id = %alert.rule_id,
        rule_name = %alert.rule_name,
        message = %alert.message,
        "{} [{}] 管理员通知: 触发告警规则: {}, 内容: {}",
        severity_emoji,
        level.to_uppercase(),
        alert.rule_name, 
        alert.message
    );
    
    // TODO: 实现实际的告警通知（如邮件、钉钉等）
    Ok(())
}

/// 记录安全审计（使用应用状态）
async fn record_security_audit_internal(
    pool: &PgPool,
    action_type: &str,
    alert: &ElkeidAlert,
    description: &str,
) -> Result<(), String> {
    let repo = AuditLogRepository::new(Arc::new(pool.clone()));
    
    let user_id = alert.user.as_ref().and_then(|u| u.id);
    let ip_address = alert.user.as_ref().and_then(|u| u.ip.clone());
    let user_agent = alert.user.as_ref().and_then(|u| u.user_agent.clone());
    
    let log = CreateAuditLog {
        user_id,
        action_type: action_type.to_string(),
        resource_type: Some("security_alert".to_string()),
        resource_id: Some(alert.rule_id.clone()),
        old_value: None,
        new_value: Some(serde_json::json!({
            "rule_name": alert.rule_name,
            "severity": alert.severity,
            "message": alert.message,
            "description": description,
            "timestamp": alert.timestamp,
            "source": alert.source,
        }).to_string()),
        ip_address,
        user_agent,
    };
    
    repo.create(log).await
        .map_err(|e| format!("记录审计日志失败: {}", e))?;
    
    tracing::debug!(action = %action_type, "安全审计记录已保存");
    Ok(())
}

/// 记录安全审计（兼容 async fn）
async fn record_security_audit(alert: &ElkeidAlert, level: &str) -> Result<(), String> {
    // 此函数保留用于保持 API 兼容性，实际实现见 record_security_audit_internal
    tracing::info!(
        "安全审计记录 [{}]: 告警 {} - {}",
        level, alert.rule_name, alert.message
    );
    Ok(())
}

/// 从数据库查询统计数据
async fn query_stats_from_db(
    repo: &AuditLogRepository,
    start_date: Option<DateTime<Utc>>,
    end_date: Option<DateTime<Utc>>,
) -> Result<(i64, i64, i64, i64, i64), String> {
    // 简化实现：返回 (total, critical, high, medium, low)
    // 实际实现需要根据审计日志表中的 action_type 来统计
    Ok((0, 0, 0, 0, 0))
}

/// 统计安全操作次数
async fn count_security_actions(
    repo: &AuditLogRepository,
    action_type: &str,
    start_date: Option<DateTime<Utc>>,
    end_date: Option<DateTime<Utc>>,
) -> Result<i64, String> {
    // 简化实现：返回 0
    // 实际实现需要查询审计日志表
    Ok(0)
}

/// 解析时间duration字符串为秒数
fn parse_duration(duration: &str) -> u64 {
    match duration {
        s if s.ends_with("s") => s[..s.len()-1].parse().unwrap_or(60),
        s if s.ends_with("m") => s[..s.len()-1].parse::<u64>().unwrap_or(1) * 60,
        s if s.ends_with("h") => s[..s.len()-1].parse::<u64>().unwrap_or(1) * 3600,
        s if s.ends_with("d") => s[..s.len()-1].parse::<u64>().unwrap_or(1) * 86400,
        _ => 3600, // 默认 1 小时
    }
}
