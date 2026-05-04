//! /! 消息路由模块
//! 负责分发和处理不同协议的消息

use anyhow::Result;
use sqlx::PgPool;

/// 消息路由器
#[derive(Debug)]
pub struct MessageRouter {
    /// 数据库连接池
    pool: Option<PgPool>,
}

impl MessageRouter {
    /// 创建新的消息路由器
    pub fn new(pool: Option<PgPool>) -> Self {
        Self { pool }
    }

    /// 创建默认的消息路由器
    pub fn with_no_pool() -> Self {
        Self { pool: None }
    }

    /// 处理解析后的消息
    pub async fn handle_message(&self, message: super::protocol::ParsedMessage) -> Result<()> {
        match message {
            super::protocol::ParsedMessage::JT808(frame) => self.handle_jt808_message(frame).await,
            super::protocol::ParsedMessage::TruckScale(message) => {
                self.handle_truck_scale_message(*message).await
            }
            super::protocol::ParsedMessage::GPRS(data) => self.handle_gprs_message(data).await,
            super::protocol::ParsedMessage::Unknown(data) => {
                self.handle_unknown_message(data).await
            }
        }
    }

    /// 处理JT808消息
    async fn handle_jt808_message(
        &self,
        frame: crate::protocols::jt808::models::JT808Frame,
    ) -> Result<()> {
        // 处理JT808消息
        // 这里可以调用现有的JT808处理逻辑
        tracing::debug!(msg_id = ?frame.msg_id, "处理JT808消息");
        Ok(())
    }

    /// 处理Truck Scale消息
    async fn handle_truck_scale_message(
        &self,
        message: crate::truck_scale::protocol::message_protocol::UnifiedMessage,
    ) -> Result<()> {
        // 处理Truck Scale消息
        // 这里可以调用现有的Truck Scale处理逻辑
        tracing::debug!(message_type = ?message.header.message_type, "处理Truck Scale消息");
        Ok(())
    }

    /// 处理GPRS消息
    async fn handle_gprs_message(&self, data: Vec<u8>) -> Result<()> {
        // 处理GPRS消息
        // 这里可以调用现有的GPRS处理逻辑
        tracing::debug!(length = data.len(), "处理GPRS消息");
        Ok(())
    }

    /// 处理未知消息
    async fn handle_unknown_message(&self, data: Vec<u8>) -> Result<()> {
        // 处理未知消息
        tracing::warn!(length = data.len(), "处理未知消息");
        Ok(())
    }
}

impl Clone for MessageRouter {
    fn clone(&self) -> Self {
        Self {
            pool: self.pool.clone(),
        }
    }
}
