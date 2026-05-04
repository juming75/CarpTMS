//! / 报警通知器
// 通过多种渠道推送报警消息

use actix::prelude::*;
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};

use super::service::AlarmRecord;
use crate::infrastructure::message_router::{MessageRouter, UnifiedMessage};

/// 通知渠道
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum NotificationChannel {
    /// WebSocket 推送
    WebSocket,
    /// 短信
    SMS,
    /// 邮件
    Email,
    /// App 推送
    AppPush,
}

/// 报警通知器
pub struct AlarmNotifier {
    message_router: Option<Addr<MessageRouter>>,
    enabled_channels: Vec<NotificationChannel>,
}

impl AlarmNotifier {
    pub fn new() -> Self {
        info!("Creating alarm notifier");

        Self {
            message_router: None,
            enabled_channels: vec![NotificationChannel::WebSocket],
        }
    }

    /// 设置消息路由器
    pub fn with_message_router(mut self, router: Addr<MessageRouter>) -> Self {
        self.message_router = Some(router);
        self
    }

    /// 发送报警通知
    pub async fn notify(&self, alarm: &AlarmRecord) -> Result<(), String> {
        info!("Sending alarm notification for device {}: {}", 
              alarm.device_id, alarm.alarm_type);

        // 遍历所有启用的通知渠道
        for channel in &self.enabled_channels {
            if let Err(e) = self.send_via_channel(channel, alarm).await {
                error!("Failed to send notification via {:?}: {}", channel, e);
            }
        }

        Ok(())
    }

    /// 通过指定渠道发送通知
    async fn send_via_channel(&self, channel: &NotificationChannel, alarm: &AlarmRecord) -> Result<(), String> {
        match channel {
            NotificationChannel::WebSocket => {
                self.send_websocket_notification(alarm).await
            }
            NotificationChannel::SMS => {
                self.send_sms_notification(alarm).await
            }
            NotificationChannel::Email => {
                self.send_email_notification(alarm).await
            }
            NotificationChannel::AppPush => {
                self.send_app_push(alarm).await
            }
        }
    }

    /// WebSocket 推送通知
    async fn send_websocket_notification(&self, alarm: &AlarmRecord) -> Result<(), String> {
        if let Some(router) = &self.message_router {
            let message = UnifiedMessage {
                device_id: alarm.device_id.clone(),
                message_type: "alarm".to_string(),
                timestamp: chrono::Utc::now(),
                data: serde_json::json!({
                    "alarm_id": alarm.id,
                    "alarm_type": alarm.alarm_type,
                    "alarm_level": alarm.alarm_level,
                    "alarm_time": alarm.alarm_time,
                    "location": alarm.location,
                    "description": alarm.description,
                    "status": alarm.status,
                }),
            };

            // TODO: 发送到消息路由器
            debug!("WebSocket notification sent for alarm {}", alarm.id.unwrap_or(0));
        }

        Ok(())
    }

    /// 短信通知(待实现)
    async fn send_sms_notification(&self, _alarm: &AlarmRecord) -> Result<(), String> {
        debug!("SMS notification - to be implemented");
        Ok(())
    }

    /// 邮件通知(待实现)
    async fn send_email_notification(&self, _alarm: &AlarmRecord) -> Result<(), String> {
        debug!("Email notification - to be implemented");
        Ok(())
    }

    /// App 推送通知(待实现)
    async fn send_app_push(&self, _alarm: &AlarmRecord) -> Result<(), String> {
        debug!("App push notification - to be implemented");
        Ok(())
    }
}

impl Clone for AlarmNotifier {
    fn clone(&self) -> Self {
        Self {
            message_router: self.message_router.clone(),
            enabled_channels: self.enabled_channels.clone(),
        }
    }
}






