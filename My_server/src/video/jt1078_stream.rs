//! / JT1078视频流处理器

use super::rtp::RtpPacket;
use crate::protocols::jt1078::{Jt1078Protocol, VideoDataType};
use log::{debug, warn};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// JT1078视频流处理器
pub struct Jt1078StreamHandler {
    protocol: Jt1078Protocol,
    /// 通道数据缓存 (channel_id -> frame buffer)
    frame_buffers: Arc<RwLock<HashMap<u8, FrameBuffer>>>,
    /// RTP包序列号
    rtp_sequence: Arc<RwLock<HashMap<u8, u16>>>,
    /// 时间戳
    timestamp: Arc<RwLock<HashMap<u8, u32>>>,
}

/// 帧缓冲区
#[derive(Debug, Clone)]
struct FrameBuffer {
    /// I帧
    i_frame: Option<Vec<u8>>,
    /// P帧列表
    p_frames: Vec<Vec<u8>>,
    /// B帧列表
    b_frames: Vec<Vec<u8>>,
    /// 当前帧序号
    current_frame_no: u16,
    /// 缓冲区大小限制
    max_size: usize,
}

impl FrameBuffer {
    fn new() -> Self {
        Self {
            i_frame: None,
            p_frames: Vec::new(),
            b_frames: Vec::new(),
            current_frame_no: 0,
            max_size: 1024 * 1024, // 1MB限制
        }
    }

    /// 添加帧到缓冲区
    fn add_frame(
        &mut self,
        frame_no: u16,
        data_type: VideoDataType,
        data: Vec<u8>,
    ) -> Option<Vec<u8>> {
        let current_size = self.current_size();
        if current_size + data.len() > self.max_size {
            warn!("Frame buffer size limit reached, clearing old frames");
            self.clear();
        }

        match data_type {
            VideoDataType::IFrame => {
                // 收到I帧,清空之前的P帧和B帧
                self.i_frame = Some(data.clone());
                self.p_frames.clear();
                self.b_frames.clear();
                self.current_frame_no = frame_no;
                Some(data) // 返回完整的I帧
            }
            VideoDataType::PFrame => {
                self.p_frames.push(data.clone());
                self.current_frame_no = frame_no;
                None
            }
            VideoDataType::BFrame => {
                self.b_frames.push(data);
                self.current_frame_no = frame_no;
                None
            }
            VideoDataType::AudioFrame => {
                Some(data) // 音频帧直接返回
            }
            VideoDataType::Unknown => {
                warn!("Unknown video data type, skipping frame");
                None
            }
        }
    }

    /// 获取当前缓冲区大小
    fn current_size(&self) -> usize {
        let i_size = self.i_frame.as_ref().map(|d| d.len()).unwrap_or(0);
        let p_size: usize = self.p_frames.iter().map(|d| d.len()).sum();
        let b_size: usize = self.b_frames.iter().map(|d| d.len()).sum();
        i_size + p_size + b_size
    }

    /// 清空缓冲区
    fn clear(&mut self) {
        self.i_frame = None;
        self.p_frames.clear();
        self.b_frames.clear();
    }
}

impl Jt1078StreamHandler {
    /// 创建新的JT1078流处理器
    pub fn new(_max_connections: usize, _buffer_size: usize) -> Self {
        Self {
            protocol: Jt1078Protocol::new(),
            frame_buffers: Arc::new(RwLock::new(HashMap::new())),
            rtp_sequence: Arc::new(RwLock::new(HashMap::new())),
            timestamp: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 处理JT1078视频帧
    pub async fn process_frame(&self, data: &[u8]) -> Option<Vec<u8>> {
        // 解析JT1078帧
        let frame = match self.protocol.parse_frame(data) {
            Some(f) => f,
            None => {
                warn!("Failed to parse JT1078 frame");
                return None;
            }
        };

        debug!(
            "Received JT1078 frame: channel={}, data_type={:?}, size={}",
            frame.header.logic_channel,
            frame.header.get_video_data_type(),
            frame.payload.len()
        );

        let channel_id = frame.header.logic_channel;
        let data_type = frame.header.get_video_data_type();
        let frame_no = frame.header.current_frame_no;

        // 获取或创建帧缓冲区
        let mut buffers = self.frame_buffers.write().await;
        let buffer = buffers.entry(channel_id).or_insert_with(FrameBuffer::new);

        // 添加帧到缓冲区
        buffer.add_frame(frame_no, data_type, frame.payload)
    }

    /// 将JT1078帧转换为RTP包
    pub async fn frame_to_rtp(
        &self,
        channel_id: u8,
        frame: Vec<u8>,
        marker: bool,
    ) -> Vec<RtpPacket> {
        const RTP_PAYLOAD_SIZE: usize = 1400; // RTP标准负载大小

        let mut rtp_packets = Vec::new();
        let total_packets = frame.len().div_ceil(RTP_PAYLOAD_SIZE);

        // 获取或初始化序列号
        let mut sequences = self.rtp_sequence.write().await;
        let sequence = sequences.entry(channel_id).or_insert(0);

        // 获取或初始化时间戳
        let mut timestamps = self.timestamp.write().await;
        let timestamp = timestamps.entry(channel_id).or_insert(0);

        for i in 0..total_packets {
            let offset = i * RTP_PAYLOAD_SIZE;
            let is_last = i == total_packets - 1;
            let chunk_size = if is_last {
                frame.len() - offset
            } else {
                RTP_PAYLOAD_SIZE
            };

            let chunk = &frame[offset..offset + chunk_size];

            let packet = RtpPacket::new(96, *sequence, *timestamp, channel_id as u32)
                .with_marker(marker && is_last)
                .with_payload(chunk.to_vec());

            rtp_packets.push(packet);

            *sequence = sequence.wrapping_add(1);
        }

        // 更新时间戳 (假设90kHz时钟)
        *timestamp += 3600; // 每帧40ms = 90kHz * 0.04s = 3600

        rtp_packets
    }

    /// 处理完整的JT1078帧并转换为RTP包流
    pub async fn process_frame_to_rtp(&self, data: &[u8]) -> Vec<RtpPacket> {
        if let Some(frame) = self.process_frame(data).await {
            // 解析通道ID
            if data.len() >= 16 {
                let channel_id = data[5] & 0x1F;
                let data_type = VideoDataType::from(data[4]);
                let marker = matches!(data_type, VideoDataType::IFrame | VideoDataType::AudioFrame);

                self.frame_to_rtp(channel_id, frame, marker).await
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        }
    }

    /// 获取指定通道的帧缓冲区状态
    pub async fn get_buffer_status(&self, channel_id: u8) -> Option<(usize, usize, usize)> {
        let buffers = self.frame_buffers.read().await;
        buffers.get(&channel_id).map(|b| {
            let i_size = b.i_frame.as_ref().map(|d| d.len()).unwrap_or(0);
            let p_count = b.p_frames.len();
            let b_count = b.b_frames.len();
            (i_size, p_count, b_count)
        })
    }

    /// 清空指定通道的帧缓冲区
    pub async fn clear_buffer(&self, channel_id: u8) {
        let mut buffers = self.frame_buffers.write().await;
        if let Some(buffer) = buffers.get_mut(&channel_id) {
            buffer.clear();
        }
    }

    /// 清空所有帧缓冲区
    pub async fn clear_all_buffers(&self) {
        let mut buffers = self.frame_buffers.write().await;
        for buffer in buffers.values_mut() {
            buffer.clear();
        }
    }
}

impl Default for Jt1078StreamHandler {
    fn default() -> Self {
        Self::new(0, 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_process_frame() {
        let handler = Jt1078StreamHandler::new(10, 1024);

        // 创建一个模拟的JT1078 I帧
        let mut frame_data = vec![0x30, 0x31, 0x63, 0x64]; // 起始标识符
        frame_data.push(0x01); // 数据类型: I帧
        frame_data.push(0x00); // 逻辑通道号: 0
        frame_data.extend_from_slice(&[0u8; 11]); // 帧属性和其他字段
        frame_data.extend_from_slice(b"test_frame_payload"); // 负载数据

        let result = handler.process_frame(&frame_data).await;
        assert!(result.is_some());
    }

    #[tokio::test]
    async fn test_frame_to_rtp() {
        let handler = Jt1078StreamHandler::new(10, 1024);

        let frame_data = vec![0u8; 1500]; // 大于RTP负载大小
        let rtp_packets = handler.frame_to_rtp(0, frame_data, true).await;

        assert!(!rtp_packets.is_empty());
        assert_eq!(rtp_packets.len(), 2); // 应该分成2个RTP包
        assert!(rtp_packets.last().unwrap().marker); // 最后一个包应该有marker
    }
}
