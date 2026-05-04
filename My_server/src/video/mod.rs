//! / 视频流管理和转发服务

pub mod adaptive_bitrate;
pub mod config;
pub mod errors;
pub mod flv_server; // P3: 启用 FLV 服务器
pub mod gb28181_stream;
pub mod hls_server;
pub mod jt1078_stream;
pub mod performance;
pub mod recording;
pub mod rtp;
pub mod security;
pub mod service;
pub mod sip_server;
pub mod stream_auth; // P3: 启用流鉴权
pub mod stream_converter;
pub mod stream_monitor;
pub mod stream_optimizer;
pub mod tcp_stream_server;
pub mod timeout;
pub mod transcoders;
pub mod video_cache;
pub mod video_encryption;
pub mod video_manager;
pub mod ws_handler; // P3: 启用 TCP 流服务器

pub use adaptive_bitrate::{
    create_bitrate_controller, AdaptiveBitrateController, BitrateConfig, NetworkQuality,
};
pub use config::{Gb28181Config, Jt1078Config, ServerConfig, StreamConfig, VideoConfig};
pub use errors::{ErrorSeverity, ServiceError};
pub use flv_server::{configure_flv_routes, create_flv_manager, FlvStreamManager}; // P3: 导出 FLV 模块
pub use gb28181_stream::Gb28181StreamHandler;
pub use hls_server::{configure_hls_routes, create_hls_manager, HlsServerConfig, HlsStreamManager};
pub use jt1078_stream::Jt1078StreamHandler;
pub use performance::{
    FrameError, FrameTask, GpuAcceleratedTranscoder, GpuType, MultiThreadFrameProcessor,
    PerformanceMonitor, PerformanceStats, PoolStats, TranscodeError, TranscoderStatus,
    ZeroCopyFramePool,
};
pub use recording::{
    PlaybackRequest, RecordingConfig, RecordingFormat, RecordingManager, RecordingMetadata,
    RecordingState,
};
pub use rtp::RtpPacket;
pub use security::{
    detect_sql_injection, escape_sql_string, mask_address, mask_bank_card, mask_email,
    mask_id_card, mask_password, mask_phone, mask_plate, AuditAction, AuditLogEntry, SensitiveData,
    SensitiveDataType,
};
pub use service::{create_and_start_video_service, VideoService};
pub use sip_server::{
    Gb28181SipServer, SipServerError, SipServerState, SipSession, SipSessionState,
};
pub use stream_auth::{create_token_manager, StreamAuthMiddleware, TokenManager}; // P3: 导出流鉴权模块
pub use stream_converter::{HlsConfig, HlsSegment, StreamConverter, StreamFormat};
pub use stream_monitor::{create_stream_monitor, MonitoringMetrics, StreamMediaMonitor};
pub use stream_optimizer::{
    create_video_stream_optimizer, EdgeCacheConfig, NetworkQuality as VideoNetworkQuality,
    OptimizerStats, VideoParams, VideoStreamOptimizer,
};
pub use tcp_stream_server::{
    configure_tcp_stream_routes, create_tcp_stream_server, TcpStreamServer, TcpStreamServerConfig,
}; // P3: 导出 TCP 流服务器
pub use timeout::{
    retry_with_backoff, retry_with_timeout, with_fallback, with_timeout, with_timeout_ignore_error,
    ConnectTimeout, RequestTimeout, RetryConfig, TimeoutConfig,
};
pub use transcoders::{
    FlvTranscoder, HlsTranscoder, RtmpTranscoder, StreamOutput, StreamTranscoder, TranscodeConfig,
};
pub use video_cache::{
    create_video_cache, create_video_cache_with_config, CacheStats, VideoCacheConfig,
    VideoStreamCache,
};
pub use video_encryption::{create_video_encryptor, EncryptionAlgorithm, VideoEncryptor};
pub use video_manager::VideoStreamManager;
pub use ws_handler::{
    configure_video_ws_routes, create_frame_distributor, VideoFrameDistributor, VideoWsHandler,
};

use log::info;

/// 视频流类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum StreamType {
    /// JT1078实时视频流
    JT1078,
    /// GB28181实时视频流
    GB28181,
    /// GB28181历史视频回放
    GB28181Playback,
    /// HTTP-FLV流
    HttpFlv,
    /// HLS流
    Hls,
    /// RTMP流
    Rtmp,
    /// WebRTC流
    WebRtc,
}

/// 视频编解码格式
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum VideoCodec {
    /// H.264/AVC
    H264,
    /// H.265/HEVC
    H265,
    /// MPEG4
    Mpeg4,
    /// MJPEG
    Mjpeg,
    /// VP8
    Vp8,
    /// VP9
    Vp9,
    /// AV1
    Av1,
}

/// 音频编解码格式
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum AudioCodec {
    /// AAC
    Aac,
    /// G.711 A-law
    G711A,
    /// G.711 μ-law
    G711U,
    /// G.726
    G726,
    /// Opus
    Opus,
    /// PCMU
    Pcmu,
    /// PCMA
    Pcma,
}

/// 视频流信息
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VideoStreamInfo {
    /// 流ID
    pub stream_id: String,
    /// 设备ID
    pub device_id: String,
    /// 通道ID
    pub channel_id: u8,
    /// 流类型
    pub stream_type: StreamType,
    /// 视频编解码格式
    pub video_codec: VideoCodec,
    /// 音频编解码格式
    pub audio_codec: Option<AudioCodec>,
    /// 视频分辨率
    pub resolution: Option<(u16, u16)>,
    /// 帧率
    pub framerate: Option<f32>,
    /// 比特率 (kbps)
    pub bitrate: Option<u32>,
    /// 在线状态
    pub online: bool,
    /// 连接的客户端数量
    pub client_count: usize,
}

/// 视频帧
/// P1-1优化: 使用Bytes实现零拷贝，减少内存复制开销
#[derive(Debug, Clone)]
pub struct VideoFrame {
    /// 帧类型
    pub frame_type: VideoFrameType,
    /// 时间戳
    pub timestamp: u64,
    /// 数据 - P1-1优化: 使用Bytes实现零拷贝
    pub data: bytes::Bytes,
    /// 帧序号
    pub sequence: u32,
}

impl VideoFrame {
    /// 从Vec创建VideoFrame（兼容旧代码）
    pub fn from_vec(
        frame_type: VideoFrameType,
        timestamp: u64,
        data: Vec<u8>,
        sequence: u32,
    ) -> Self {
        Self {
            frame_type,
            timestamp,
            data: bytes::Bytes::from(data),
            sequence,
        }
    }

    /// 获取数据切片引用
    pub fn as_slice(&self) -> &[u8] {
        &self.data
    }
}

/// 视频帧类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum VideoFrameType {
    /// I帧(关键帧)
    IFrame,
    /// P帧(预测帧)
    PFrame,
    /// B帧(双向预测帧)
    BFrame,
    /// 音频帧
    AudioFrame,
}

/// 初始化视频服务
pub fn init_video_service() {
    info!("Initializing video streaming service...");
    // 初始化视频流管理器
    // 注意:实际的视频流管理器需要在有数据库连接的情况下初始化
    // 这里只做基础初始化,完整的初始化逻辑在 VideoService 中
}

#[cfg(test)]
mod tests {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/video/mod_test.rs"
    ));
}
