//! / 视频流管理器
// 统一管理所有视频流的生命周期

use super::gb28181_stream::Gb28181StreamHandler;
use super::jt1078_stream::Jt1078StreamHandler;
use super::{StreamType, VideoFrame, VideoFrameType, VideoStreamInfo};
use crate::protocols::jt1078::VideoDataType;
use log::{info, warn};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;

/// 视频流管理器
pub struct VideoStreamManager {
    /// JT1078流处理器
    jt1078_handler: Arc<Jt1078StreamHandler>,
    /// GB28181流处理器
    gb28181_handler: Arc<Gb28181StreamHandler>,
    /// 流信息列表
    streams: Arc<RwLock<HashMap<String, VideoStreamInfo>>>,
    /// 客户端订阅 (stream_id -> client_ids)
    subscriptions: Arc<RwLock<HashMap<String, Vec<String>>>>,
    /// 客户端推送通道 (client_id -> frame sender)
    clients: Arc<RwLock<HashMap<String, tokio::sync::mpsc::UnboundedSender<VideoFrame>>>>,
}

/// 流控制命令
#[derive(Debug, Clone)]
pub enum StreamCommand {
    /// 开始播放
    Start,
    /// 暂停播放
    Pause,
    /// 停止播放
    Stop,
    /// 切换通道
    SwitchChannel(u8),
    /// 调整码率
    AdjustBitrate(u32),
    /// 截图
    Capture,
}

impl VideoStreamManager {
    /// 创建新的视频流管理器(使用默认配置)
    pub fn new() -> Self {
        Self {
            jt1078_handler: Arc::new(Jt1078StreamHandler::new(1000, 65536)),
            gb28181_handler: Arc::new(Gb28181StreamHandler::new(5060, 10000, 20000)),
            streams: Arc::new(RwLock::new(HashMap::new())),
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
            clients: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 创建新的视频流管理器(带配置)
    pub fn with_config(_max_streams: usize, _frame_buffer_size: usize) -> Self {
        Self::new()
    }

    /// 创建新的视频流
    pub async fn create_stream(
        &self,
        device_id: String,
        channel_id: u8,
        stream_type: StreamType,
    ) -> Result<String, StreamError> {
        let stream_id = format!("{}_ch{}", device_id, channel_id);

        let stream_info = VideoStreamInfo {
            stream_id: stream_id.clone(),
            device_id: device_id.clone(),
            channel_id,
            stream_type,
            video_codec: super::VideoCodec::H264,
            audio_codec: Some(super::AudioCodec::G711A),
            resolution: None,
            framerate: None,
            bitrate: None,
            online: true,
            client_count: 0,
        };

        let mut streams = self.streams.write().await;
        streams.insert(stream_id.clone(), stream_info);

        info!("Created stream: {} (type: {:?})", stream_id, stream_type);
        Ok(stream_id)
    }

    /// 删除视频流
    pub async fn remove_stream(&self, stream_id: &str) -> Result<(), StreamError> {
        let mut streams = self.streams.write().await;
        if streams.remove(stream_id).is_some() {
            info!("Removed stream: {}", stream_id);
            Ok(())
        } else {
            Err(StreamError::StreamNotFound(stream_id.to_string()))
        }
    }

    /// 获取流信息
    pub async fn get_stream(&self, stream_id: &str) -> Option<VideoStreamInfo> {
        let streams = self.streams.read().await;
        streams.get(stream_id).cloned()
    }

    /// 获取所有流
    pub async fn get_all_streams(&self) -> Vec<VideoStreamInfo> {
        let streams = self.streams.read().await;
        streams.values().cloned().collect()
    }

    /// 客户端订阅流(返回接收器)
    pub async fn subscribe_stream(
        &self,
        client_id: String,
        stream_id: String,
    ) -> Result<tokio::sync::mpsc::UnboundedReceiver<VideoFrame>, StreamError> {
        // 检查流是否存在
        let streams = self.streams.read().await;
        if !streams.contains_key(&stream_id) {
            return Err(StreamError::StreamNotFound(stream_id.clone()));
        }
        drop(streams);

        // 创建通道
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

        // 添加客户端
        let mut clients = self.clients.write().await;
        clients.insert(client_id.clone(), tx);

        // 添加订阅
        let mut subscriptions = self.subscriptions.write().await;
        subscriptions
            .entry(stream_id.clone())
            .or_insert_with(Vec::new)
            .push(client_id.clone());

        // 更新流的客户端计数
        let mut streams = self.streams.write().await;
        if let Some(stream) = streams.get_mut(&stream_id) {
            stream.client_count += 1;
        }

        info!("Client {} subscribed to stream {}", client_id, stream_id);
        Ok(rx)
    }

    /// 订阅流(简化版,用于WebSocket)
    pub async fn subscribe_stream_simple(
        &self,
        stream_id: &str,
        client_id: &str,
    ) -> Result<(), StreamError> {
        // 检查流是否存在
        let streams = self.streams.read().await;
        if !streams.contains_key(stream_id) {
            return Err(StreamError::StreamNotFound(stream_id.to_string()));
        }
        drop(streams);

        // 创建通道
        let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();

        // 添加客户端
        let mut clients = self.clients.write().await;
        clients.insert(client_id.to_string(), tx);

        // 添加订阅
        let mut subscriptions = self.subscriptions.write().await;
        subscriptions
            .entry(stream_id.to_string())
            .or_insert_with(Vec::new)
            .push(client_id.to_string());

        // 更新流的客户端计数
        let mut streams = self.streams.write().await;
        if let Some(stream) = streams.get_mut(stream_id) {
            stream.client_count += 1;
        }

        info!("Client {} subscribed to stream {}", client_id, stream_id);
        Ok(())
    }

    /// 客户端取消订阅
    pub async fn unsubscribe_stream(&self, client_id: &str, stream_id: &str) {
        // 从订阅列表中移除
        let mut subscriptions = self.subscriptions.write().await;
        if let Some(clients) = subscriptions.get_mut(stream_id) {
            clients.retain(|id| id != client_id);
            if clients.is_empty() {
                subscriptions.remove(stream_id);
            }
        }
        drop(subscriptions);

        // 移除客户端
        let mut clients = self.clients.write().await;
        clients.remove(client_id);
        drop(clients);

        // 更新流的客户端计数
        let mut streams = self.streams.write().await;
        if let Some(stream) = streams.get_mut(stream_id) {
            stream.client_count = stream.client_count.saturating_sub(1);
        }

        info!(
            "Client {} unsubscribed from stream {}",
            client_id, stream_id
        );
    }

    /// 处理JT1078视频帧
    pub async fn handle_jt1078_frame(&self, data: &[u8]) -> Result<(), StreamError> {
        // 解析通道ID
        if data.len() < 16 {
            return Err(StreamError::InvalidData("Frame too short".to_string()));
        }

        let channel_id = data[5] & 0x1F;
        let stream_id = format!("device_ch{}", channel_id); // 简化的设备ID

        // 处理帧
        if let Some(frame_data) = self.jt1078_handler.process_frame(data).await {
            // 创建视频帧
            let frame_type = if data.len() >= 16 {
                let data_type = VideoDataType::from(data[4]);
                match data_type {
                    VideoDataType::IFrame => VideoFrameType::IFrame,
                    VideoDataType::PFrame => VideoFrameType::PFrame,
                    VideoDataType::BFrame => VideoFrameType::BFrame,
                    VideoDataType::AudioFrame => VideoFrameType::AudioFrame,
                    _ => VideoFrameType::PFrame,
                }
            } else {
                VideoFrameType::PFrame
            };

            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system time is before UNIX epoch")
                .as_secs();

            let frame = VideoFrame {
                frame_type,
                timestamp: now,
                data: frame_data,
                sequence: 0,
            };

            // 推送给订阅的客户端
            self.push_frame_to_subscribers(&stream_id, frame).await;
        }

        Ok(())
    }

    /// 推送帧到订阅者
    async fn push_frame_to_subscribers(&self, stream_id: &str, frame: VideoFrame) {
        let subscriptions = self.subscriptions.read().await;
        if let Some(client_ids) = subscriptions.get(stream_id) {
            let clients = self.clients.read().await;
            for client_id in client_ids {
                if let Some(tx) = clients.get(client_id) {
                    if let Err(e) = tx.send(frame.clone()) {
                        warn!("Failed to send frame to client {}: {}", client_id, e);
                    }
                }
            }
        }
    }

    /// 获取JT1078处理器
    pub fn jt1078_handler(&self) -> &Jt1078StreamHandler {
        &self.jt1078_handler
    }

    /// 获取GB28181处理器
    pub fn gb28181_handler(&self) -> &Gb28181StreamHandler {
        &self.gb28181_handler
    }

    /// 控制流
    pub async fn control_stream(
        &self,
        stream_id: &str,
        command: StreamCommand,
    ) -> Result<(), StreamError> {
        let streams = self.streams.read().await;
        if !streams.contains_key(stream_id) {
            return Err(StreamError::StreamNotFound(stream_id.to_string()));
        }
        drop(streams);

        match command {
            StreamCommand::Start => {
                info!("Starting stream: {}", stream_id);
            }
            StreamCommand::Pause => {
                info!("Pausing stream: {}", stream_id);
            }
            StreamCommand::Stop => {
                info!("Stopping stream: {}", stream_id);
            }
            StreamCommand::SwitchChannel(channel) => {
                info!("Switching stream {} to channel {}", stream_id, channel);
            }
            StreamCommand::AdjustBitrate(bitrate) => {
                info!("Adjusting stream {} bitrate to {} kbps", stream_id, bitrate);
            }
            StreamCommand::Capture => {
                info!("Capturing frame from stream: {}", stream_id);
            }
        }

        Ok(())
    }

    /// 获取统计信息
    pub async fn get_statistics(&self) -> StreamStatistics {
        let streams = self.streams.read().await;
        let clients = self.clients.read().await;
        let subscriptions = self.subscriptions.read().await;

        let total_streams = streams.len();
        let active_streams = streams.values().filter(|s| s.online).count();
        let total_clients = clients.len();
        let total_subscriptions: usize = subscriptions.values().map(|v| v.len()).sum();

        StreamStatistics {
            total_streams,
            active_streams,
            total_clients,
            total_subscriptions,
        }
    }

    /// 检查流是否存在
    pub async fn stream_exists(&self, stream_id: &str) -> bool {
        let streams = self.streams.read().await;
        streams.contains_key(stream_id)
    }

    /// 停止所有流
    pub async fn stop_all_streams(&self) {
        let mut streams = self.streams.write().await;
        for stream in streams.values_mut() {
            stream.online = false;
            stream.client_count = 0;
        }

        // 清理所有订阅和客户端
        let mut subscriptions = self.subscriptions.write().await;
        subscriptions.clear();

        let mut clients = self.clients.write().await;
        clients.clear();

        info!("Stopped all video streams");
    }

    /// 清理不活动的流
    pub async fn cleanup_inactive_streams(&self) {
        let mut streams = self.streams.write().await;
        let inactive_streams: Vec<String> = streams
            .iter()
            .filter(|(_, stream)| !stream.online && stream.client_count == 0)
            .map(|(id, _)| id.clone())
            .collect();

        for stream_id in inactive_streams {
            if streams.remove(&stream_id).is_some() {
                info!("Cleaned up inactive stream: {}", stream_id);
            }
        }
    }

    /// 分发帧到流
    pub async fn distribute_frame(
        &self,
        stream_id: &str,
        frame: VideoFrame,
    ) -> Result<(), StreamError> {
        let streams = self.streams.read().await;
        if !streams.contains_key(stream_id) {
            return Err(StreamError::StreamNotFound(stream_id.to_string()));
        }
        drop(streams);

        self.push_frame_to_subscribers(stream_id, frame).await;
        Ok(())
    }
}

/// 流统计信息
#[derive(Debug, Clone)]
pub struct StreamStatistics {
    pub total_streams: usize,
    pub active_streams: usize,
    pub total_clients: usize,
    pub total_subscriptions: usize,
}

/// 流管理错误
#[derive(Debug, thiserror::Error)]
pub enum StreamError {
    #[error("Stream not found: {0}")]
    StreamNotFound(String),
    #[error("Invalid data: {0}")]
    InvalidData(String),
    #[error("Client not found: {0}")]
    ClientNotFound(String),
    #[error("Internal error: {0}")]
    InternalError(String),
}

impl Default for VideoStreamManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_and_get_stream() {
        let manager = VideoStreamManager::new();

        let stream_id = manager
            .create_stream("device001".to_string(), 1, StreamType::JT1078)
            .await
            .unwrap();

        let stream = manager.get_stream(&stream_id).await;
        assert!(stream.is_some());
        assert_eq!(stream.unwrap().channel_id, 1);
    }

    #[tokio::test]
    async fn test_subscribe_stream() {
        let manager = VideoStreamManager::new();

        let stream_id = manager
            .create_stream("device001".to_string(), 1, StreamType::JT1078)
            .await
            .unwrap();

        let _rx = manager
            .subscribe_stream("client001".to_string(), stream_id.clone())
            .await
            .unwrap();

        // 验证流信息更新
        let stream = manager.get_stream(&stream_id).await;
        assert_eq!(stream.unwrap().client_count, 1);

        // 取消订阅
        manager.unsubscribe_stream("client001", &stream_id).await;

        // 验证客户端计数减少
        let stream = manager.get_stream(&stream_id).await;
        assert_eq!(stream.unwrap().client_count, 0);
    }

    #[tokio::test]
    async fn test_get_statistics() {
        let manager = VideoStreamManager::new();

        let stats = manager.get_statistics().await;
        assert_eq!(stats.total_streams, 0);
        assert_eq!(stats.active_streams, 0);
        assert_eq!(stats.total_clients, 0);

        manager
            .create_stream("device001".to_string(), 1, StreamType::JT1078)
            .await
            .unwrap();

        let stats = manager.get_statistics().await;
        assert_eq!(stats.total_streams, 1);
        assert_eq!(stats.active_streams, 1);
    }
}
