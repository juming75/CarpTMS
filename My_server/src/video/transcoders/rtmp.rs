//! / RTMP (Real-Time Messaging Protocol) 视频流转换器
// 将 RTP/JT1078 流转换为 RTMP 格式
#![allow(dead_code)]

use bytes::{BufMut, Bytes, BytesMut};

/// 鎵╁睍 BytesMut 鐨?trait
trait BytesMutExt {
    fn put_u24_be(&mut self, val: u32);
}

impl BytesMutExt for BytesMut {
    fn put_u24_be(&mut self, val: u32) {
        self.put_u8(((val >> 16) & 0xFF) as u8);
        self.put_u8(((val >> 8) & 0xFF) as u8);
        self.put_u8((val & 0xFF) as u8);
    }
}

use crate::video::transcoders::{
    mod_base::{FrameBufferPool, TranscodeError, TranscoderStatus},
    StreamOutput, StreamTranscoder, TranscodeConfig,
};
use crate::video::{StreamType, VideoFrame, VideoFrameType};
use async_trait::async_trait;
use log::{debug, error, info};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::{mpsc, Mutex, RwLock};
use tokio::task::JoinHandle;

/// RTMP 消息类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum RtmpMessageType {
    SetChunkSize = 1,
    Abort = 2,
    Acknowledgement = 3,
    UserControl = 4,
    WindowAckSize = 5,
    SetPeerBandwidth = 6,
    Audio = 8,
    Video = 9,
    DataAMF3 = 18,
    SharedObject = 19,
    Command = 20,
}

/// RTMP 头类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum RtmpHeaderType {
    Type0 = 0, // 11 字节头
    Type1 = 1, // 7 字节头
    Type2 = 2, // 3 字节头
    Type3 = 3, // 1 字节头
}

/// RTMP 块头
struct RtmpChunkHeader {
    /// 块流 ID
    pub cs_id: u32,
    /// 时间戳
    pub timestamp: u32,
    /// 消息长度
    pub msg_len: u32,
    /// 消息类型
    pub msg_type: RtmpMessageType,
    /// 流 ID
    pub stream_id: u32,
}

impl RtmpChunkHeader {
    /// 编码块头
    pub fn encode(&self, header_type: RtmpHeaderType) -> Vec<u8> {
        let mut buf = BytesMut::new();

        match header_type {
            RtmpHeaderType::Type0 => {
                // 格式: [fmt(2bits) cs_id(6bits)][timestamp(3bytes)][msg_len(3bytes)][msg_type(1byte)][stream_id(4bytes)]
                let fmt_cs_id = (self.cs_id & 0x3F) as u8;
                buf.put_u8(fmt_cs_id);

                buf.put_u24_be(self.timestamp);
                buf.put_u24_be(self.msg_len);
                buf.put_u8(self.msg_type as u8);
                buf.put_u32_le(self.stream_id);
            }
            RtmpHeaderType::Type1 => {
                // 格式: [fmt(2bits) cs_id(6bits)][timestamp_delta(3bytes)][msg_len(3bytes)][msg_type(1byte)]
                let fmt_cs_id = (1 << 6) | ((self.cs_id & 0x3F) as u8);
                buf.put_u8(fmt_cs_id);

                buf.put_u24_be(self.timestamp);
                buf.put_u24_be(self.msg_len);
                buf.put_u8(self.msg_type as u8);
            }
            RtmpHeaderType::Type2 => {
                // 格式: [fmt(2bits) cs_id(6bits)][timestamp_delta(3bytes)]
                let fmt_cs_id = (2 << 6) | ((self.cs_id & 0x3F) as u8);
                buf.put_u8(fmt_cs_id);

                buf.put_u24_be(self.timestamp);
            }
            RtmpHeaderType::Type3 => {
                // 格式: [fmt(2bits) cs_id(6bits)]
                let fmt_cs_id = (3 << 6) | ((self.cs_id & 0x3F) as u8);
                buf.put_u8(fmt_cs_id);
            }
        }

        buf.to_vec()
    }
}

/// RTMP 转换器配置
#[derive(Debug, Clone)]
pub struct RtmpConfig {
    /// RTMP 端口
    pub port: u16,
    /// 应用名称
    pub app: String,
    /// 流名称
    pub stream_name: String,
    /// 是否为推流模式
    pub is_push_mode: bool,
}

impl Default for RtmpConfig {
    fn default() -> Self {
        Self {
            port: 1935,
            app: "live".to_string(),
            stream_name: "stream".to_string(),
            is_push_mode: false,
        }
    }
}

/// RTMP 转换器
pub struct RtmpTranscoder {
    /// 转换器配置
    config: Arc<RwLock<TranscodeConfig>>,
    /// RTMP 配置
    rtmp_config: Arc<RwLock<RtmpConfig>>,
    /// 输出通道
    output_tx: Arc<Mutex<Option<mpsc::Sender<StreamOutput>>>>,
    /// 运行状态
    status: Arc<RwLock<TranscoderStatus>>,
    /// 任务句柄
    task_handle: Arc<Mutex<Option<JoinHandle<()>>>>,
    /// 缓冲区池
    buffer_pool: Arc<FrameBufferPool>,
    /// 时间戳计数器
    timestamp_counter: Arc<Mutex<u32>>,
    /// 块大小
    chunk_size: Arc<Mutex<u32>>,
}

impl RtmpTranscoder {
    /// 创建新的 RTMP 转换器
    pub fn new(config: TranscodeConfig, rtmp_config: RtmpConfig) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
            rtmp_config: Arc::new(RwLock::new(rtmp_config)),
            output_tx: Arc::new(Mutex::new(None)),
            status: Arc::new(RwLock::new(TranscoderStatus::Idle)),
            task_handle: Arc::new(Mutex::new(None)),
            buffer_pool: Arc::new(FrameBufferPool::new(1024 * 1024, 20)), // 1MB 缓冲区
            timestamp_counter: Arc::new(Mutex::new(0)),
            chunk_size: Arc::new(Mutex::new(4096)), // 默认 4KB 块
        }
    }

    /// 设置块大小
    pub async fn set_chunk_size(&self, size: u32) {
        let mut cs = self.chunk_size.lock().await;
        *cs = size;
    }

    /// 将视频帧转换为 RTMP 消息
    async fn convert_frame_to_rtmp(&self, frame: VideoFrame) -> Result<Vec<u8>, TranscodeError> {
        let msg_type = match frame.frame_type {
            VideoFrameType::AudioFrame => RtmpMessageType::Audio,
            _ => RtmpMessageType::Video,
        };

        let mut timestamp = self.timestamp_counter.lock().await;
        let ts = *timestamp;
        *timestamp = timestamp.wrapping_add(40); // 40ms
        drop(timestamp);

        // 创建 RTMP 头
        let header = RtmpChunkHeader {
            cs_id: 4, // 视频流 ID
            timestamp: ts,
            msg_len: frame.data.len() as u32,
            msg_type,
            stream_id: 1,
        };

        let chunk_header = header.encode(RtmpHeaderType::Type1);

        // 将数据分块
        let chunk_size = *self.chunk_size.lock().await;
        let mut rtmp_data = BytesMut::new();

        for chunk in frame.data.chunks(chunk_size as usize) {
            rtmp_data.extend_from_slice(&chunk_header);
            rtmp_data.extend_from_slice(chunk);
        }

        Ok(rtmp_data.to_vec())
    }

    /// 启动 RTMP 服务器(简化版)
    async fn start_rtmp_server(&self) -> Result<(), TranscodeError> {
        let rtmp_config = self.rtmp_config.read().await;
        let addr = format!("0.0.0.0:{}", rtmp_config.port);

        let listener = TcpListener::bind(&addr).await.map_err(TranscodeError::Io)?;

        info!("RTMP server listening on {}", addr);

        // 简化版:仅绑定端口,实际握手逻辑需要更多代码
        tokio::spawn(async move {
            while let Ok((_socket, addr)) = listener.accept().await {
                debug!("RTMP connection from {}", addr);
                // TODO: 实现 RTMP 握手逻辑
            }
        });

        Ok(())
    }

    /// 处理输入帧队列
    async fn process_frames(&self, mut frame_rx: mpsc::Receiver<VideoFrame>) {
        info!("RTMP transcoder frame processor started");

        while let Some(frame) = frame_rx.recv().await {
            let status = *self.status.read().await;
            if status == TranscoderStatus::Idle || status == TranscoderStatus::Error {
                break;
            }

            match self.convert_frame_to_rtmp(frame.clone()).await {
                Ok(rtmp_data) => {
                    let output = StreamOutput::from_bytes(
                        StreamType::Rtmp,
                        Bytes::from(rtmp_data),
                        frame.timestamp,
                        frame.frame_type == VideoFrameType::IFrame,
                    );

                    // 先获取发送器，然后释放锁
                    let tx = self.output_tx.lock().await.clone();
                    if let Some(tx) = tx {
                        if let Err(e) = tx.send(output).await {
                            error!("Failed to send RTMP output: {}", e);
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to convert frame to RTMP: {}", e);
                }
            }
        }

        info!("RTMP transcoder frame processor stopped");
    }
}

#[async_trait]
impl StreamTranscoder for RtmpTranscoder {
    fn name(&self) -> &'static str {
        "RTMP"
    }

    fn output_type(&self) -> StreamType {
        StreamType::Rtmp
    }

    fn config(&self) -> &TranscodeConfig {
        use std::sync::OnceLock;
        static DEFAULT_CONFIG: OnceLock<TranscodeConfig> = OnceLock::new();
        DEFAULT_CONFIG.get_or_init(TranscodeConfig::default)
    }

    async fn set_config(&self, config: TranscodeConfig) {
        let mut cfg = self.config.write().await;
        *cfg = config;
    }

    async fn process_frame(&self, frame: VideoFrame) -> Result<Vec<StreamOutput>, TranscodeError> {
        match self.convert_frame_to_rtmp(frame.clone()).await {
            Ok(rtmp_data) => {
                let output = StreamOutput::from_bytes(
                    StreamType::Rtmp,
                    Bytes::from(rtmp_data),
                    frame.timestamp,
                    frame.frame_type == VideoFrameType::IFrame,
                );
                Ok(vec![output])
            }
            Err(e) => Err(e),
        }
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

        // 启动 RTMP 服务器
        self.start_rtmp_server().await?;

        info!("RTMP transcoder started");
        Ok(())
    }

    async fn stop(&self) -> Result<(), TranscodeError> {
        *self.status.write().await = TranscoderStatus::Idle;

        let mut handle = self.task_handle.lock().await;
        if let Some(h) = handle.take() {
            h.abort();
        }

        info!("RTMP transcoder stopped");
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
    fn test_rtmp_chunk_header() {
        let header = RtmpChunkHeader {
            cs_id: 4,
            timestamp: 1000,
            msg_len: 500,
            msg_type: RtmpMessageType::Video,
            stream_id: 1,
        };

        let encoded = header.encode(RtmpHeaderType::Type1);
        assert!(!encoded.is_empty());
    }

    #[tokio::test]
    async fn test_rtmp_transcoder_creation() {
        let config = TranscodeConfig::default();
        let rtmp_config = RtmpConfig::default();
        let transcoder = RtmpTranscoder::new(config, rtmp_config);

        assert_eq!(transcoder.name(), "RTMP");
        assert_eq!(transcoder.output_type(), StreamType::Rtmp);
    }

    #[test]
    fn test_rtmp_config_default() {
        let config = RtmpConfig::default();
        assert_eq!(config.port, 1935);
        assert_eq!(config.app, "live");
    }
}
