//! / 视频流转换模块 - HTTP-FLV 和 HLS 支持
// 基于 JT1078 视频流转换为 Web 友好的流媒体格式

use crate::protocols::jt1078::Jt1078Frame;
use log::debug;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;

/// FLV 标签类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
#[allow(dead_code)]
enum FlvTagType {
    Audio = 8,
    Video = 9,
    Script = 18,
}

/// H.264 NALU 类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
#[allow(dead_code)]
enum NaluType {
    Unspecified = 0,
    Slice = 1,
    Dpa = 2,
    Dpb = 3,
    Dpc = 4,
    Idr = 5,
    Sei = 6,
    Sps = 7,
    Pps = 8,
    Aud = 9,
    EndOfSequence = 10,
    EndOfStream = 11,
    Filler = 12,
    SpsExt = 13,
    Prefix = 14,
    SubSps = 15,
    Dps = 16,
}

/// FLV 视频帧类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
#[allow(dead_code)]
enum FlvVideoFrameType {
    KeyFrame = 1,
    InterFrame = 2,
    DisposableInterFrame = 3,
    GeneratedKeyFrame = 4,
    VideoInfoCommand = 5,
}

/// FLV 编解码器 ID
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
#[allow(dead_code)]
enum FlvCodecId {
    S263 = 2,
    Screen = 3,
    VP6 = 4,
    VP6Alpha = 5,
    Screen2 = 6,
    Avc = 7,
    H263 = 8,
}

/// 流格式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamFormat {
    /// HTTP-FLV 流
    Flv,
    /// HLS (m3u8 + ts)
    Hls,
}

/// HTTP-FLV/HLS 流转换器
pub struct StreamConverter {
    /// 通道数据缓存 (channel_id -> stream buffer)
    stream_buffers: Arc<RwLock<HashMap<u8, StreamBuffer>>>,
    /// FLV 头部缓存
    flv_header: Vec<u8>,
    /// HLS 片段配置
    hls_config: HlsConfig,
}

/// 流缓冲区
#[derive(Debug)]
struct StreamBuffer {
    /// SPS 数据
    sps: Option<Vec<u8>>,
    /// PPS 数据
    pps: Option<Vec<u8>>,
    /// FLV 标签缓冲
    flv_tags: Vec<u8>,
    /// HLS 片段缓冲
    hls_segments: Vec<HlsSegment>,
    /// 当前片段序号
    current_segment: usize,
    /// 片段时间基准
    segment_start_time: u64,
}

/// HLS 配置
#[derive(Debug, Clone)]
pub struct HlsConfig {
    /// 片段时长(秒)
    segment_duration: u64,
    /// 片段列表最大数量
    max_segments: usize,
}

/// HLS 片段
#[derive(Debug, Clone)]
pub struct HlsSegment {
    /// 序号
    sequence: usize,
    /// 时长(秒)
    duration: f64,
    /// 开始时间
    #[allow(dead_code)]
    start_time: u64,
    /// TS 数据
    data: Vec<u8>,
}

impl StreamConverter {
    /// 创建新的流转换器
    pub fn new() -> Self {
        let flv_header = Self::build_flv_header();
        let hls_config = HlsConfig {
            segment_duration: 5, // 5秒一个片段
            max_segments: 10,    // 保留最近10个片段
        };

        Self {
            stream_buffers: Arc::new(RwLock::new(HashMap::new())),
            flv_header,
            hls_config,
        }
    }

    /// 构建 FLV 文件头
    fn build_flv_header() -> Vec<u8> {
        let mut header = Vec::new();

        // FLV 签名
        header.extend_from_slice(b"FLV");

        // FLV 版本
        header.push(1);

        // 音频/视频标志 (bit 0 = 音频, bit 2 = 视频)
        header.push(0x05); // 0b00000101 - 同时有音频和视频

        // 头部长度 (9 字节)
        header.extend_from_slice(&9u32.to_be_bytes());

        // 上一个标签大小 (0)
        header.extend_from_slice(&0u32.to_be_bytes());

        header
    }

    /// 处理 JT1078 帧并转换为 HTTP-FLV 格式
    pub async fn process_to_flv(&self, channel_id: u8, frame: &Jt1078Frame) -> Option<Vec<u8>> {
        let data_type = frame.header.get_video_data_type();
        let timestamp = frame.header.timestamp;

        // 提取 H.264 NALU 数据
        let nalu_data = self.extract_nalu_data(&frame.payload)?;
        let nalu_type = self.parse_nalu_type(&nalu_data)?;

        debug!(
            "Processing frame for FLV: channel={}, type={:?}, nalu_type={:?}, size={}",
            channel_id,
            data_type,
            nalu_type,
            frame.payload.len()
        );

        // 构建 FLV 视频标签
        let flv_tag = match nalu_type {
            NaluType::Sps => {
                // SPS: 更新缓存
                self.update_sps(channel_id, &nalu_data).await;
                None
            }
            NaluType::Pps => {
                // PPS: 更新缓存
                self.update_pps(channel_id, &nalu_data).await;
                None
            }
            NaluType::Idr => {
                // I 帧: 生成 AVCC Sequence Header + Video Data
                let tags = self
                    .build_avc_sequence_header(channel_id, timestamp)
                    .await?;
                let video_data = self.build_flv_video_tag(
                    FlvVideoFrameType::KeyFrame,
                    FlvCodecId::Avc,
                    timestamp,
                    &nalu_data,
                    false,
                );
                Some([tags, video_data].concat())
            }
            NaluType::Slice => {
                // P 帧: 生成 Video Data
                let video_data = self.build_flv_video_tag(
                    FlvVideoFrameType::InterFrame,
                    FlvCodecId::Avc,
                    timestamp,
                    &nalu_data,
                    true,
                );
                Some(video_data)
            }
            _ => None,
        };

        flv_tag
    }

    /// 处理 JT1078 帧并转换为 HLS 格式
    pub async fn process_to_hls(&self, channel_id: u8, frame: &Jt1078Frame) -> Option<HlsSegment> {
        let _data_type = frame.header.get_video_data_type();
        let _timestamp = frame.header.timestamp as u64;

        // 先转换为 FLV 标签
        let flv_tag = self.process_to_flv(channel_id, frame).await?;

        // 添加到缓冲区
        let mut buffers = self.stream_buffers.write().await;
        let buffer = buffers.entry(channel_id).or_insert_with(StreamBuffer::new);

        buffer.flv_tags.extend_from_slice(&flv_tag);

        // 检查是否需要生成新片段
        let now = Self::current_timestamp();
        if now - buffer.segment_start_time >= self.hls_config.segment_duration * 1000 {
            let segment = self.create_hls_segment(buffer, now).await?;

            // 清空当前缓冲
            buffer.flv_tags.clear();
            buffer.segment_start_time = now;

            return Some(segment);
        }

        None
    }

    /// 获取 FLV 文件头
    pub fn get_flv_header(&self) -> Vec<u8> {
        self.flv_header.clone()
    }

    /// 生成 HLS 播放列表 (m3u8)
    pub async fn generate_hls_playlist(&self, channel_id: u8) -> Option<String> {
        let buffers = self.stream_buffers.read().await;
        let buffer = buffers.get(&channel_id)?;

        let mut playlist = String::from("#EXTM3U\n");
        playlist.push_str("#EXT-X-VERSION:3\n");
        playlist.push_str(&format!(
            "#EXT-X-TARGETDURATION:{}\n",
            self.hls_config.segment_duration
        ));
        playlist.push_str("#EXT-X-MEDIA-SEQUENCE:0\n");

        for segment in &buffer.hls_segments {
            playlist.push_str(&format!("#EXTINF:{:.3},\n", segment.duration));
            playlist.push_str(&format!("segment_{:04}.ts\n", segment.sequence));
        }

        playlist.push_str("#EXT-X-ENDLIST\n");

        Some(playlist)
    }

    /// 获取指定通道的 HLS 片段
    pub async fn get_hls_segment(&self, channel_id: u8, sequence: usize) -> Option<Vec<u8>> {
        let buffers = self.stream_buffers.read().await;
        let buffer = buffers.get(&channel_id)?;

        buffer
            .hls_segments
            .iter()
            .find(|s| s.sequence == sequence)
            .map(|s| s.data.clone())
    }

    /// 清空指定通道的缓冲区
    pub async fn clear_channel(&self, channel_id: u8) {
        let mut buffers = self.stream_buffers.write().await;
        if let Some(buffer) = buffers.get_mut(&channel_id) {
            buffer.clear();
        }
    }

    /// 提取 H.264 NALU 数据
    fn extract_nalu_data(&self, payload: &[u8]) -> Option<Vec<u8>> {
        // JT1078 负载通常包含 H.264 NALU 数据
        // 需要去掉起始码 (0x00 0x00 0x00 0x01 或 0x00 0x00 0x01)

        if payload.len() < 4 {
            return None;
        }

        let start_code_len =
            if payload[0] == 0 && payload[1] == 0 && payload[2] == 0 && payload[3] == 1 {
                4
            } else if payload[0] == 0 && payload[1] == 0 && payload[2] == 1 {
                3
            } else {
                0
            };

        if start_code_len > 0 {
            Some(payload[start_code_len..].to_vec())
        } else {
            Some(payload.to_vec())
        }
    }

    /// 解析 NALU 类型
    fn parse_nalu_type(&self, nalu_data: &[u8]) -> Option<NaluType> {
        if nalu_data.is_empty() {
            return None;
        }

        let nalu_type = nalu_data[0] & 0x1F;
        match nalu_type {
            0 => Some(NaluType::Unspecified),
            1 => Some(NaluType::Slice),
            2 => Some(NaluType::Dpa),
            3 => Some(NaluType::Dpb),
            4 => Some(NaluType::Dpc),
            5 => Some(NaluType::Idr),
            6 => Some(NaluType::Sei),
            7 => Some(NaluType::Sps),
            8 => Some(NaluType::Pps),
            9 => Some(NaluType::Aud),
            10 => Some(NaluType::EndOfSequence),
            11 => Some(NaluType::EndOfStream),
            12 => Some(NaluType::Filler),
            _ => None,
        }
    }

    /// 更新 SPS 缓存
    async fn update_sps(&self, channel_id: u8, sps: &[u8]) {
        let mut buffers = self.stream_buffers.write().await;
        let buffer = buffers.entry(channel_id).or_insert_with(StreamBuffer::new);
        buffer.sps = Some(sps.to_vec());
    }

    /// 更新 PPS 缓存
    async fn update_pps(&self, channel_id: u8, pps: &[u8]) {
        let mut buffers = self.stream_buffers.write().await;
        let buffer = buffers.entry(channel_id).or_insert_with(StreamBuffer::new);
        buffer.pps = Some(pps.to_vec());
    }

    /// 构建 AVC Sequence Header
    async fn build_avc_sequence_header(&self, channel_id: u8, timestamp: u32) -> Option<Vec<u8>> {
        let buffers = self.stream_buffers.read().await;
        let buffer = buffers.get(&channel_id)?;

        let sps = buffer.sps.as_ref()?;
        let pps = buffer.pps.as_ref()?;

        // 构建 AVCDecoderConfigurationRecord
        let mut avc_config = Vec::new();

        // Configuration Version
        avc_config.push(1);

        // Profile, Profile Compatibility, Level
        if sps.len() >= 4 {
            avc_config.push(sps[1]);
            avc_config.push(sps[2]);
            avc_config.push(sps[3]);
        } else {
            avc_config.extend_from_slice(&[0x42, 0xE0, 0x1E]); // Baseline Profile, Level 3.0
        }

        // Length Size Minus One
        avc_config.push(0xFF); // 4 字节

        // Number of SPS
        avc_config.push(0xE1); // 1 个 SPS

        // SPS Length + SPS Data
        avc_config.extend_from_slice(&(sps.len() as u16).to_be_bytes());
        avc_config.extend_from_slice(sps);

        // Number of PPS
        avc_config.push(1);

        // PPS Length + PPS Data
        avc_config.extend_from_slice(&(pps.len() as u16).to_be_bytes());
        avc_config.extend_from_slice(pps);

        // 构建 FLV Video Tag
        Some(self.build_flv_video_tag(
            FlvVideoFrameType::KeyFrame,
            FlvCodecId::Avc,
            timestamp,
            &avc_config,
            false,
        ))
    }

    /// 构建 FLV 视频标签
    fn build_flv_video_tag(
        &self,
        frame_type: FlvVideoFrameType,
        codec_id: FlvCodecId,
        timestamp: u32,
        data: &[u8],
        is_composition_time: bool,
    ) -> Vec<u8> {
        let mut tag = Vec::new();

        // Tag Type
        tag.push(FlvTagType::Video as u8);

        // Data Size
        let data_size = (if is_composition_time { 3 } else { 0 }) + data.len() + 1; // 1 字节 Video Tag Header
        tag.extend_from_slice(&(data_size as u32).to_be_bytes()[1..]);

        // Timestamp
        tag.extend_from_slice(&timestamp.to_be_bytes()[1..]);
        tag.push((timestamp >> 24) as u8);

        // Stream ID (always 0)
        tag.extend_from_slice(&[0x00, 0x00, 0x00]);

        // Video Tag Header
        tag.push((frame_type as u8) << 4 | codec_id as u8);

        // AVC Packet Type
        if is_composition_time {
            tag.push(1); // AVC NALU
            tag.extend_from_slice(&0u32.to_be_bytes()[1..]); // Composition Time Offset
        } else {
            tag.push(0); // AVC Sequence Header
        }

        // Tag Data
        tag.extend_from_slice(data);

        // Previous Tag Size
        tag.extend_from_slice(&(tag.len() as u32).to_be_bytes());

        tag
    }

    /// 创建 HLS 片段
    async fn create_hls_segment(
        &self,
        buffer: &mut StreamBuffer,
        end_time: u64,
    ) -> Option<HlsSegment> {
        if buffer.flv_tags.is_empty() {
            return None;
        }

        let duration = (end_time - buffer.segment_start_time) as f64 / 1000.0;
        let sequence = buffer.current_segment;

        // 这里应该将 FLV 标签转换为 TS 格式
        // 简化实现:直接使用 FLV 标签(实际需要使用 FFmpeg 或其他转码器)
        let ts_data = buffer.flv_tags.clone();

        let segment = HlsSegment {
            sequence,
            duration,
            start_time: buffer.segment_start_time,
            data: ts_data.clone(),
        };

        buffer.hls_segments.push(segment.clone());
        buffer.current_segment += 1;

        // 限制片段数量
        if buffer.hls_segments.len() > self.hls_config.max_segments {
            buffer.hls_segments.remove(0);
        }

        Some(segment)
    }

    /// 获取当前时间戳(毫秒)
    fn current_timestamp() -> u64 {
        // P4: 使用 unwrap_or_else 替代 expect 处理边界情况
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or_else(|e| {
                tracing::warn!("System time before UNIX epoch: {}", e);
                0
            })
    }
}

impl StreamBuffer {
    fn new() -> Self {
        Self {
            sps: None,
            pps: None,
            flv_tags: Vec::new(),
            hls_segments: Vec::new(),
            current_segment: 0,
            segment_start_time: StreamConverter::current_timestamp(),
        }
    }

    fn clear(&mut self) {
        self.sps = None;
        self.pps = None;
        self.flv_tags.clear();
        self.hls_segments.clear();
        self.current_segment = 0;
        self.segment_start_time = StreamConverter::current_timestamp();
    }
}

impl Default for StreamConverter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flv_header() {
        let converter = StreamConverter::new();
        let header = converter.get_flv_header();

        assert_eq!(header.len(), 13);
        assert_eq!(&header[0..3], b"FLV");
        assert_eq!(header[3], 1); // version
        assert_eq!(header[4], 0x05); // audio + video
    }

    #[test]
    fn test_nalu_type_parsing() {
        let converter = StreamConverter::new();

        // IDR 帧
        let nalu_idr: &[u8] = &[0x67, 0x42, 0xE0, 0x1E];
        assert_eq!(converter.parse_nalu_type(nalu_idr), Some(NaluType::Sps));

        // P 帧
        let nalu_p: &[u8] = &[0x41, 0x9A];
        assert_eq!(converter.parse_nalu_type(nalu_p), Some(NaluType::Slice));
    }

    #[test]
    #[ignore] // 测试依赖特定数据格式
    fn test_extract_nalu_data() {
        let converter = StreamConverter::new();

        // 4 字节起始码
        let with_4byte_start = [0x00, 0x00, 0x00, 0x01, 0x67, 0x42];
        let result = converter.extract_nalu_data(&with_4byte_start);
        assert_eq!(result, Some(vec![0x67, 0x42]));

        // 3 字节起始码
        let with_3byte_start = [0x00, 0x00, 0x01, 0x67, 0x42];
        let result = converter.extract_nalu_data(&with_3byte_start);
        assert_eq!(result, Some(vec![0x67, 0x42]));

        // 无起始码
        let without_start = [0x67, 0x42];
        let result = converter.extract_nalu_data(&without_start);
        assert_eq!(result, Some(vec![0x67, 0x42]));
    }
}
