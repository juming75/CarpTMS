//! / HLS (HTTP Live Streaming) 视频流转换器
// 将 RTP/JT1078 流转换为 HLS 格式(TS 分片 + M3U8 播放列表)
#![allow(dead_code)]

use crate::video::transcoders::{
    mod_base::{FrameBufferPool, TranscodeError, TranscoderStatus},
    StreamOutput, StreamTranscoder, TranscodeConfig,
};
use crate::video::{StreamType, VideoFrame, VideoFrameType};
use async_trait::async_trait;
use bytes::{BufMut, Bytes, BytesMut};
use log::{debug, error, info};
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex, RwLock};
use tokio::task::JoinHandle;
// chrono::Utc 未使用,移除导入

/// TS 分片信息
#[derive(Debug, Clone)]
struct TsSegment {
    /// 分片索引
    index: u32,
    /// 分片数据(零拷贝)
    data: Bytes,
    /// 起始时间戳
    start_timestamp: u64,
    /// 结束时间戳
    end_timestamp: u64,
    /// 时长(秒)
    duration: f64,
    /// 是否包含关键帧
    has_key_frame: bool,
}

impl TsSegment {
    /// 创建新的 TS 分片
    fn new(index: u32, data: Bytes, start_ts: u64, end_ts: u64, has_key: bool) -> Self {
        let duration = if end_ts > start_ts {
            (end_ts - start_ts) as f64 / 1000.0 // 转换为秒
        } else {
            5.0 // 默认5秒
        };

        Self {
            index,
            data,
            start_timestamp: start_ts,
            end_timestamp: end_ts,
            duration,
            has_key_frame: has_key,
        }
    }

    /// 获取分片时长(秒)
    fn duration(&self) -> f64 {
        self.duration
    }
}

/// M3U8 播放列表
#[derive(Debug, Clone)]
struct M3u8Playlist {
    /// 版本
    version: u32,
    /// 分片列表
    segments: VecDeque<TsSegment>,
    /// 最大分片数量
    max_segments: usize,
    /// 分片时长(秒)
    segment_duration: u32,
    /// 目标时长(秒)
    target_duration: f64,
    /// 是否为直播流
    is_live: bool,
    /// 列表序列号
    media_sequence: u32,
}

impl M3u8Playlist {
    /// 创建新的播放列表
    fn new(max_segments: usize, segment_duration: u32, is_live: bool) -> Self {
        Self {
            version: 3,
            segments: VecDeque::with_capacity(max_segments),
            max_segments,
            segment_duration,
            target_duration: segment_duration as f64,
            is_live,
            media_sequence: 0,
        }
    }

    /// 添加分片
    fn add_segment(&mut self, segment: TsSegment) {
        self.segments.push_back(segment);

        // 移除超出限制的分片
        while self.segments.len() > self.max_segments {
            self.segments.pop_front();
            self.media_sequence += 1;
        }
    }

    /// 生成 M3U8 内容
    fn generate(&self) -> String {
        let mut playlist = String::new();

        // 播放列表头部
        playlist.push_str("#EXTM3U\n");
        playlist.push_str(&format!("#EXT-X-VERSION:{}\n", self.version));
        playlist.push_str(&format!(
            "#EXT-X-TARGETDURATION:{:.2}\n",
            self.target_duration
        ));
        playlist.push_str(&format!("#EXT-X-MEDIA-SEQUENCE:{}\n", self.media_sequence));

        if self.is_live {
            playlist.push_str("#EXT-X-PLAYLIST-TYPE:EVENT\n");
        }

        // 分片信息
        for segment in &self.segments {
            playlist.push_str(&format!("#EXTINF:{:.3},\n", segment.duration()));
            playlist.push_str(&format!("segment_{}.ts\n", segment.index));
        }

        if self.is_live {
            playlist.push_str("#EXT-X-ENDLIST\n");
        }

        playlist
    }

    /// 获取分片数量
    fn segment_count(&self) -> usize {
        self.segments.len()
    }

    /// 获取最新的分片索引
    fn latest_segment_index(&self) -> Option<u32> {
        self.segments.back().map(|s| s.index)
    }
}

/// TS 包适配器
struct TsAdapter {
    /// PAT (Program Association Table)
    pat: Vec<u8>,
    /// PMT (Program Map Table)
    pmt: Vec<u8>,
    /// 自增计数器
    continuity_counter: u8,
}

impl TsAdapter {
    /// 创建新的 TS 适配器
    fn new() -> Self {
        Self {
            pat: Self::create_pat(),
            pmt: Self::create_pmt(),
            continuity_counter: 0,
        }
    }

    /// 创建 PAT 表
    fn create_pat() -> Vec<u8> {
        // 简化的 PAT,包含一个 program
        let mut pat = Vec::new();
        pat.extend_from_slice(&[0x47]); // sync byte

        // transport_scrambling_control (2bits) + payload_unit_start_indicator (1bit) +
        // transport_priority (1bit) + PID (13bits)
        pat.extend_from_slice(&[0x40, 0x00]); // PID = 0x0000 (PAT)

        // adaptation_field_control (2bits) + continuity_counter (4bits)
        pat.push(0x10); // no adaptation field, continuity_counter = 0

        // pointer_field (payload_unit_start_indicator=1 时需要)
        pat.push(0x00); // pointer = 0

        // table_id
        pat.push(0x00);

        // section_syntax_indicator + private_bit + reserved + section_length
        pat.extend_from_slice(&[0xB0, 0x0D]); // section_length = 13

        // transport_stream_id
        pat.extend_from_slice(&[0x00, 0x01]);

        // version + current_next_indicator + section_number + last_section_number
        pat.push(0xC1);

        // program_number
        pat.extend_from_slice(&[0x00, 0x01]);

        // reserved + PMT_PID
        pat.extend_from_slice(&[0xE0, 0x01, 0x00]); // PMT PID = 0x0100

        // CRC32 (简化版)
        pat.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);

        // 填充到 188 字节
        while pat.len() < 188 {
            pat.push(0xFF);
        }

        pat
    }

    /// 创建 PMT 表
    fn create_pmt() -> Vec<u8> {
        let mut pmt = Vec::new();
        pmt.extend_from_slice(&[0x47]); // sync byte

        // PID = 0x0100 (PMT)
        pmt.extend_from_slice(&[0x40, 0x01]);

        // no adaptation field, continuity_counter = 0
        pmt.push(0x10);

        // pointer_field = 0
        pmt.push(0x00);

        // table_id = 0x02 (PMT)
        pmt.push(0x02);

        // section_syntax_indicator + reserved + section_length
        pmt.extend_from_slice(&[0xB0, 0x12]); // section_length = 18

        // program_number
        pmt.extend_from_slice(&[0x00, 0x01]);

        // version + current_next_indicator + section_number + last_section_number
        pmt.push(0xC1);

        // PCR_PID = 0x0101
        pmt.extend_from_slice(&[0xE1, 0x01, 0x00]);

        // program_info_length = 0
        pmt.extend_from_slice(&[0xF0, 0x00]);

        // stream_type = 0x1B (H.264), reserved + elementary_PID = 0x0101
        pmt.extend_from_slice(&[0x1B, 0xE1, 0x01, 0xF0]);

        // CRC32 (简化版)
        pmt.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);

        // 填充到 188 字节
        while pmt.len() < 188 {
            pmt.push(0xFF);
        }

        pmt
    }

    /// 将数据封装为 TS 包
    fn encapsulate(&mut self, data: &[u8], pid: u16, is_key_frame: bool) -> Vec<u8> {
        const TS_PACKET_SIZE: usize = 188;
        const TS_PAYLOAD_SIZE: usize = 184;

        let mut packets = Vec::new();
        let data_offset = 0;

        for chunk in data.chunks(TS_PAYLOAD_SIZE) {
            let mut packet = BytesMut::with_capacity(TS_PACKET_SIZE);

            // sync byte
            packet.put_u8(0x47);

            // payload_unit_start_indicator (关键帧或第一个包)
            let pusi = if data_offset == 0 || is_key_frame {
                0x80
            } else {
                0x00
            };

            // PID (13 bits)
            let pid_high = ((pid >> 8) & 0x1F) as u8;
            let pid_low = (pid & 0xFF) as u8;

            packet.put_u8(pusi | pid_high);
            packet.put_u8(pid_low);

            // adaptation_field_control + continuity_counter
            packet.put_u8(0x10 | self.continuity_counter);

            // payload
            packet.put_slice(chunk);

            // padding
            while packet.len() < TS_PACKET_SIZE {
                packet.put_u8(0xFF);
            }

            packets.extend_from_slice(&packet);

            self.continuity_counter = (self.continuity_counter + 1) % 16;
        }

        packets
    }

    /// 创建完整的 TS 流(PAT + PMT + 数据)
    fn create_ts_stream(
        &mut self,
        video_data: &[u8],
        audio_data: Option<&[u8]>,
        is_key_frame: bool,
    ) -> Vec<u8> {
        let mut ts_stream = Vec::new();

        // PAT
        ts_stream.extend_from_slice(&self.pat);

        // PMT
        ts_stream.extend_from_slice(&self.pmt);

        // 视频流(PID = 0x0101)
        ts_stream.extend_from_slice(&self.encapsulate(video_data, 0x0101, is_key_frame));

        // 音频流(PID = 0x0102)
        if let Some(audio) = audio_data {
            ts_stream.extend_from_slice(&self.encapsulate(audio, 0x0102, false));
        }

        ts_stream
    }
}

impl Default for TsAdapter {
    fn default() -> Self {
        Self::new()
    }
}

/// HLS 转换器
pub struct HlsTranscoder {
    /// 转换器配置
    config: Arc<RwLock<TranscodeConfig>>,
    /// 输出通道
    output_tx: Arc<Mutex<Option<mpsc::Sender<StreamOutput>>>>,
    /// 运行状态
    status: Arc<RwLock<TranscoderStatus>>,
    /// 任务句柄
    task_handle: Arc<Mutex<Option<JoinHandle<()>>>>,
    /// 缓冲区池
    buffer_pool: Arc<FrameBufferPool>,
    /// TS 适配器
    ts_adapter: Arc<Mutex<TsAdapter>>,
    /// M3U8 播放列表
    playlist: Arc<RwLock<M3u8Playlist>>,
    /// 当前分片缓冲区
    segment_buffer: Arc<Mutex<BytesMut>>,
    /// 当前分片开始时间戳
    segment_start_time: Arc<Mutex<Option<u64>>>,
    /// 分片计数器
    segment_counter: Arc<Mutex<u32>>,
}

impl HlsTranscoder {
    /// 创建新的 HLS 转换器
    pub fn new(config: TranscodeConfig) -> Self {
        let playlist = M3u8Playlist::new(
            config.hls_segment_count as usize,
            config.hls_segment_duration,
            true, // 直播流
        );

        Self {
            config: Arc::new(RwLock::new(config)),
            output_tx: Arc::new(Mutex::new(None)),
            status: Arc::new(RwLock::new(TranscoderStatus::Idle)),
            task_handle: Arc::new(Mutex::new(None)),
            buffer_pool: Arc::new(FrameBufferPool::new(2 * 1024 * 1024, 10)), // 2MB 缓冲区
            ts_adapter: Arc::new(Mutex::new(TsAdapter::new())),
            playlist: Arc::new(RwLock::new(playlist)),
            segment_buffer: Arc::new(Mutex::new(BytesMut::with_capacity(5 * 1024 * 1024))), // 5MB 分片
            segment_start_time: Arc::new(Mutex::new(None)),
            segment_counter: Arc::new(Mutex::new(0)),
        }
    }

    /// 完成 TS 分片并添加到播放列表
    async fn finalize_segment(&self, timestamp: u64) -> Result<(), TranscodeError> {
        let mut buffer = self.segment_buffer.lock().await;
        let segment_data = buffer.clone().freeze();
        buffer.clear();

        let start_time = self.segment_start_time.lock().await.take();
        if let Some(start_ts) = start_time {
            let mut counter = self.segment_counter.lock().await;
            let index = *counter;
            *counter += 1;
            drop(counter);

            // 创建 TS 分片
            let ts_adapter = self.ts_adapter.lock().await;
            let mut ts_adapter = ts_adapter; // 释放锁
            let ts_stream = ts_adapter.create_ts_stream(&segment_data, None, false);

            let segment = TsSegment::new(index, Bytes::from(ts_stream), start_ts, timestamp, false);
            let duration = segment.duration();

            // 添加到播放列表
            let mut playlist = self.playlist.write().await;
            playlist.add_segment(segment);

            debug!("HLS segment {} finalized, duration {:.2}s", index, duration);
        }

        Ok(())
    }

    /// 生成 M3U8 播放列表
    pub async fn get_playlist(&self) -> Result<String, TranscodeError> {
        let playlist = self.playlist.read().await;
        Ok(playlist.generate())
    }

    /// 获取指定分片的数据
    pub async fn get_segment(&self, index: u32) -> Result<Option<Bytes>, TranscodeError> {
        let playlist = self.playlist.read().await;

        for segment in &playlist.segments {
            if segment.index == index {
                return Ok(Some(segment.data.clone()));
            }
        }

        Ok(None)
    }

    /// 处理输入帧队列
    async fn process_frames(&self, mut frame_rx: mpsc::Receiver<VideoFrame>) {
        info!("HLS transcoder frame processor started");

        while let Some(frame) = frame_rx.recv().await {
            let status = *self.status.read().await;
            if status == TranscoderStatus::Idle || status == TranscoderStatus::Error {
                break;
            }

            match self.process_frame_internal(frame).await {
                Ok(outputs) => {
                    for output in outputs {
                        if let Some(tx) = self.output_tx.lock().await.as_ref() {
                            if let Err(e) = tx.send(output).await {
                                error!("Failed to send HLS output: {}", e);
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to process frame for HLS: {}", e);
                }
            }
        }

        info!("HLS transcoder frame processor stopped");
    }

    /// 处理单个视频帧
    async fn process_frame_internal(
        &self,
        frame: VideoFrame,
    ) -> Result<Vec<StreamOutput>, TranscodeError> {
        let config = self.config.read().await;
        let segment_duration_ms = config.hls_segment_duration as u64 * 1000;
        let max_segment_size = 10 * 1024 * 1024; // 10MB

        // 检查是否需要开始新分片
        let mut segment_start = self.segment_start_time.lock().await;
        let should_finalize = segment_start.is_some()
            && (frame.timestamp - segment_start.unwrap_or(0) >= segment_duration_ms
                || frame.frame_type == VideoFrameType::IFrame);

        if should_finalize {
            self.finalize_segment(frame.timestamp).await?;
            *self.segment_start_time.lock().await = Some(frame.timestamp);
        } else if segment_start.is_none() {
            *segment_start = Some(frame.timestamp);
        }

        // 添加帧到当前分片
        let mut buffer = self.segment_buffer.lock().await;
        buffer.extend_from_slice(&frame.data);

        // 检查分片大小
        if buffer.len() >= max_segment_size {
            let _buffer_size = buffer.len();
            drop(buffer);
            self.finalize_segment(frame.timestamp).await?;
            let mut segment_buffer = self.segment_buffer.lock().await;
            segment_buffer.clear();
            drop(segment_buffer);
            *self.segment_start_time.lock().await = Some(frame.timestamp);
        }

        // 生成播放列表输出
        let playlist = self.playlist.read().await;
        let playlist_data = playlist.generate();
        let playlist_output = StreamOutput::from_bytes(
            StreamType::Hls,
            Bytes::from(playlist_data),
            frame.timestamp,
            frame.frame_type == VideoFrameType::IFrame,
        );

        Ok(vec![playlist_output])
    }
}

#[async_trait]
impl StreamTranscoder for HlsTranscoder {
    fn name(&self) -> &'static str {
        "HLS"
    }

    fn output_type(&self) -> StreamType {
        StreamType::Hls
    }

    fn config(&self) -> &TranscodeConfig {
        use std::sync::OnceLock;
        static DEFAULT_CONFIG: OnceLock<TranscodeConfig> = OnceLock::new();
        DEFAULT_CONFIG.get_or_init(TranscodeConfig::default)
    }

    async fn set_config(&self, config: TranscodeConfig) {
        let segment_count = config.hls_segment_count;
        let segment_duration = config.hls_segment_duration;

        let mut cfg = self.config.write().await;
        *cfg = config.clone();

        let mut playlist = self.playlist.write().await;
        playlist.max_segments = segment_count as usize;
        playlist.segment_duration = segment_duration;
        playlist.target_duration = segment_duration as f64;
    }

    async fn process_frame(&self, frame: VideoFrame) -> Result<Vec<StreamOutput>, TranscodeError> {
        self.process_frame_internal(frame).await
    }

    async fn get_output_channel(&self) -> Option<mpsc::Receiver<StreamOutput>> {
        let (tx, rx) = mpsc::channel(100);
        let mut output_tx = self.output_tx.lock().await;
        *output_tx = Some(tx);
        Some(rx)
    }

    async fn start(&self) -> Result<(), TranscodeError> {
        let status = *self.status.read().await;
        if status == TranscoderStatus::Running {
            return Err(TranscodeError::AlreadyRunning);
        }

        *self.status.write().await = TranscoderStatus::Running;
        info!("HLS transcoder started");

        Ok(())
    }

    async fn stop(&self) -> Result<(), TranscodeError> {
        *self.status.write().await = TranscoderStatus::Idle;

        let mut handle = self.task_handle.lock().await;
        if let Some(h) = handle.take() {
            h.abort();
        }

        info!("HLS transcoder stopped");
        Ok(())
    }

    async fn status(&self) -> TranscoderStatus {
        *self.status.read().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_m3u8_playlist_generation() {
        let mut playlist = M3u8Playlist::new(10, 5, true);
        let segment = TsSegment::new(0, Bytes::from(vec![1, 2, 3]), 0, 5000, true);
        playlist.add_segment(segment);

        let m3u8 = playlist.generate();
        assert!(m3u8.contains("#EXTM3U"));
        assert!(m3u8.contains("#EXT-X-VERSION:3"));
        assert!(m3u8.contains("segment_0.ts"));
    }

    #[test]
    fn test_ts_adapter() {
        let adapter = TsAdapter::new();
        assert!(!adapter.pat.is_empty());
        assert!(!adapter.pmt.is_empty());
    }

    #[tokio::test]
    async fn test_hls_transcoder_creation() {
        let config = TranscodeConfig::default();
        let transcoder = HlsTranscoder::new(config);

        assert_eq!(transcoder.name(), "HLS");
        assert_eq!(transcoder.output_type(), StreamType::Hls);
    }
}
