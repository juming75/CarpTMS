//! / 视频流转换器基础接口和通用类型

use crate::video::{AudioCodec, StreamType, VideoCodec, VideoFrame};
use async_trait::async_trait;
use bytes::Bytes;
use std::sync::Arc;
use tokio::sync::mpsc;

/// 流转换器配置
#[derive(Debug, Clone)]
pub struct TranscodeConfig {
    /// 视频编解码格式
    pub video_codec: VideoCodec,
    /// 音频编解码格式
    pub audio_codec: Option<AudioCodec>,
    /// 视频比特率 (kbps)
    pub video_bitrate: u32,
    /// 音频比特率 (kbps)
    pub audio_bitrate: u32,
    /// 帧率 (fps)
    pub framerate: f32,
    /// 分辨率 (宽, 高)
    pub resolution: (u16, u16),
    /// 是否启用硬件加速
    pub hardware_acceleration: bool,
    /// GOP 大小(关键帧间隔)
    pub gop_size: u32,
    /// HLS TS 分段时长(秒)
    pub hls_segment_duration: u32,
    /// HLS 分段数量
    pub hls_segment_count: u32,
}

impl Default for TranscodeConfig {
    fn default() -> Self {
        Self {
            video_codec: VideoCodec::H264,
            audio_codec: Some(AudioCodec::Aac),
            video_bitrate: 2000, // 2Mbps
            audio_bitrate: 128,  // 128kbps
            framerate: 25.0,
            resolution: (1920, 1080),
            hardware_acceleration: false,
            gop_size: 30, // 每秒1个关键帧
            hls_segment_duration: 5,
            hls_segment_count: 10,
        }
    }
}

/// 流输出数据
#[derive(Debug, Clone)]
pub struct StreamOutput {
    /// 输出类型
    pub output_type: StreamType,
    /// 数据块(零拷贝)
    pub data: Bytes,
    /// 时间戳
    pub timestamp: u64,
    /// 是否为关键帧
    pub is_key_frame: bool,
    /// 分片索引(HLS用)
    pub segment_index: Option<u32>,
    /// 总分片数(HLS用)
    pub total_segments: Option<u32>,
}

impl StreamOutput {
    /// 创建新的输出数据
    pub fn new(output_type: StreamType, data: Vec<u8>, timestamp: u64, is_key_frame: bool) -> Self {
        Self {
            output_type,
            data: Bytes::from(data),
            timestamp,
            is_key_frame,
            segment_index: None,
            total_segments: None,
        }
    }

    /// 使用零拷贝 Bytes 创建输出数据
    pub fn from_bytes(
        output_type: StreamType,
        data: Bytes,
        timestamp: u64,
        is_key_frame: bool,
    ) -> Self {
        Self {
            output_type,
            data,
            timestamp,
            is_key_frame,
            segment_index: None,
            total_segments: None,
        }
    }

    /// 设置分片信息(HLS用)
    pub fn with_segments(mut self, index: u32, total: u32) -> Self {
        self.segment_index = Some(index);
        self.total_segments = Some(total);
        self
    }
}

/// 流转换器 trait
#[async_trait]
pub trait StreamTranscoder: Send + Sync {
    /// 获取转换器名称
    fn name(&self) -> &'static str;

    /// 获取输出流类型
    fn output_type(&self) -> StreamType;

    /// 获取转换器配置
    fn config(&self) -> &TranscodeConfig;

    /// 设置转换器配置
    async fn set_config(&self, config: TranscodeConfig);

    /// 处理视频帧
    async fn process_frame(&self, frame: VideoFrame) -> Result<Vec<StreamOutput>, TranscodeError>;

    /// 获取流输出通道
    async fn get_output_channel(&self) -> Option<mpsc::Receiver<StreamOutput>>;

    /// 启动转换器
    async fn start(&self) -> Result<(), TranscodeError>;

    /// 停止转换器
    async fn stop(&self) -> Result<(), TranscodeError>;

    /// 获取转换器状态
    async fn status(&self) -> TranscoderStatus;
}

/// 转换器状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TranscoderStatus {
    /// 未启动
    Idle,
    /// 运行中
    Running,
    /// 暂停
    Paused,
    /// 错误
    Error,
}

/// 转换错误
#[derive(Debug, thiserror::Error)]
pub enum TranscodeError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Unsupported codec: {0:?}")]
    UnsupportedCodec(VideoCodec),

    #[error("Invalid frame data")]
    InvalidFrameData,

    #[error("Transcoder not initialized")]
    NotInitialized,

    #[error("Transcoder already running")]
    AlreadyRunning,

    #[error("Internal error: {0}")]
    Internal(String),
}

/// 帧缓冲区管理器(零拷贝)
pub struct FrameBufferPool {
    /// 缓冲区池
    pool: Arc<tokio::sync::RwLock<Vec<Bytes>>>,
    /// 最大缓冲区大小
    max_size: usize,
    /// 缓冲区大小
    buffer_size: usize,
}

impl FrameBufferPool {
    /// 创建新的缓冲区池
    pub fn new(buffer_size: usize, pool_size: usize) -> Self {
        let mut pool = Vec::with_capacity(pool_size);
        for _ in 0..pool_size {
            pool.push(Bytes::from(vec![0u8; buffer_size]));
        }

        Self {
            pool: Arc::new(tokio::sync::RwLock::new(pool)),
            max_size: pool_size,
            buffer_size,
        }
    }

    /// 从池中获取缓冲区
    pub async fn acquire(&self, size: usize) -> Bytes {
        let mut pool = self.pool.write().await;
        if let Some(buffer) = pool.pop() {
            // 检查缓冲区是否足够大
            if buffer.len() >= size {
                return buffer.slice(0..size);
            } else {
                // 缓冲区太小,丢弃并重新分配
                return Bytes::from(vec![0u8; size]);
            }
        }
        // 池为空,分配新缓冲区
        Bytes::from(vec![0u8; size])
    }

    /// 归还缓冲区到池中
    pub async fn release(&self, buffer: Bytes) {
        let mut pool = self.pool.write().await;
        if pool.len() < self.max_size {
            // 重置缓冲区并放回池中
            if buffer.len() == self.buffer_size {
                pool.push(buffer);
            }
        }
    }

    /// 获取池状态
    pub async fn status(&self) -> (usize, usize) {
        let pool = self.pool.read().await;
        (pool.len(), self.max_size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transcode_config_default() {
        let config = TranscodeConfig::default();
        assert_eq!(config.video_codec, VideoCodec::H264);
        assert_eq!(config.video_bitrate, 2000);
        assert_eq!(config.framerate, 25.0);
    }

    #[test]
    fn test_stream_output_creation() {
        let data = vec![1, 2, 3, 4, 5];
        let output = StreamOutput::new(StreamType::HttpFlv, data, 1000, true);

        assert_eq!(output.output_type, StreamType::HttpFlv);
        assert_eq!(output.timestamp, 1000);
        assert!(output.is_key_frame);
        assert!(output.data.len() == 5);
    }

    #[tokio::test]
    #[ignore] // 测试需要异步运行时
    async fn test_frame_buffer_pool() {
        let pool = FrameBufferPool::new(1024, 10);

        // 获取缓冲区
        let buffer = pool.acquire(512).await;
        assert_eq!(buffer.len(), 512);

        // 归还缓冲区
        pool.release(buffer).await;

        // 检查状态
        let (available, max) = pool.status().await;
        assert_eq!(available, 1);
        assert_eq!(max, 10);
    }
}
