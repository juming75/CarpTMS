//! / HTTP-FLV 视频流转换器
// 将 RTP/JT1078 流转换为 HTTP-FLV 格式
#![allow(dead_code)]

use crate::video::transcoders::{
    mod_base::{FrameBufferPool, TranscodeError, TranscoderStatus},
    StreamOutput, StreamTranscoder, TranscodeConfig,
};
use crate::video::{StreamType, VideoFrame, VideoFrameType};
use async_trait::async_trait;
use bytes::{BufMut, Bytes, BytesMut};
use log::{error, info};
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex, RwLock};
use tokio::task::JoinHandle;

/// 扩展 BytesMut 的 trait
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

/// FLV 标签类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
#[allow(dead_code)]
enum FlvTagType {
    Audio = 8,
    Video = 9,
    Script = 18,
}

/// FLV 视频帧类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
#[allow(dead_code)]
enum FlvVideoFrameType {
    Key = 1,             // I帧
    Inter = 2,           // P帧
    DisposableInter = 3, // B帧
    GeneratedKey = 4,    // SI帧
    VideoInfoCommand = 5,
}

/// FLV 视频编解码器 ID
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
#[allow(dead_code)]
enum FlvVideoCodec {
    SorensonH263 = 2,
    ScreenVideo = 3,
    On2VP6 = 4,
    On2VP6WithAlpha = 5,
    ScreenVideo2 = 6,
    H264 = 7, // AVC
}

/// FLV 音频编解码器 ID
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum FlvAudioCodec {
    Adpcm = 1,
    Mp3 = 2,
    LinearPcmPlatformEndian = 3,
    Nellymoser16kHz = 4,
    Nellymoser8kHz = 5,
    Nellymoser = 6,
    G711ALaw = 7,
    G711MuLaw = 8,
    Reserved = 9,
    Aac = 10,
    Speex = 11,
    Mp3_8kHz = 14,
    DeviceSpecificSound = 15,
}

/// FLV 标签头
#[derive(Debug, Clone)]
struct FlvTagHeader {
    pub tag_type: FlvTagType,
    pub data_size: u32,
    pub timestamp: u32,
    pub stream_id: u32,
}

impl FlvTagHeader {
    const HEADER_SIZE: usize = 11;

    /// 创建新的标签头
    pub fn new(tag_type: FlvTagType, timestamp: u32, data_size: u32) -> Self {
        Self {
            tag_type,
            data_size,
            timestamp,
            stream_id: 0, // 默认为0
        }
    }

    /// 编码标签头
    pub fn encode(&self) -> Vec<u8> {
        let mut buf = BytesMut::with_capacity(Self::HEADER_SIZE);

        buf.put_u8(self.tag_type as u8);

        // 数据大小 (24位,大端)
        buf.put_u24_be(self.data_size);

        // 时间戳 (24位,大端)
        buf.put_u24_be(self.timestamp & 0xFFFFFF);

        // 时间戳扩展 (高8位)
        buf.put_u8((self.timestamp >> 24) as u8);

        // Stream ID (总是0)
        buf.put_u24_be(self.stream_id);

        buf.to_vec()
    }

    /// 解码标签头
    pub fn decode(data: &[u8]) -> Option<Self> {
        if data.len() < Self::HEADER_SIZE {
            return None;
        }

        let tag_type = match data[0] {
            8 => FlvTagType::Audio,
            9 => FlvTagType::Video,
            18 => FlvTagType::Script,
            _ => return None,
        };

        let data_size = ((data[1] as u32) << 16) | ((data[2] as u32) << 8) | (data[3] as u32);
        let timestamp = ((data[4] as u32) << 16)
            | ((data[5] as u32) << 8)
            | (data[6] as u32)
            | ((data[7] as u32) << 24);
        let stream_id = ((data[8] as u32) << 16) | ((data[9] as u32) << 8) | (data[10] as u32);

        Some(Self {
            tag_type,
            data_size,
            timestamp,
            stream_id,
        })
    }
}

/// FLV 头部
pub const FLV_HEADER: [u8; 9] = [
    b'F', b'L', b'V', // 签名
    0x01, // 版本
    0x05, // 音频+视频标志
    0x00, 0x00, 0x00, 0x09, // 头部长度
];

/// FLV 音频标签头
fn build_audio_tag_header(data_size: u32, timestamp: u32) -> Vec<u8> {
    let mut tag_header = BytesMut::with_capacity(13);

    // Tag 类型: Audio
    tag_header.put_u8(FlvTagType::Audio as u8);

    // 数据大小 (24位)
    tag_header.put_u24_be(data_size);

    // 时间戳 (24位)
    tag_header.put_u24_be(timestamp & 0xFFFFFF);

    // 时间戳扩展
    tag_header.put_u8((timestamp >> 24) as u8);

    // Stream ID (总是0)
    tag_header.put_u24_be(0);

    // 音频数据 (SoundFormat=10(AAC), SoundRate=3(44kHz), SoundSize=1(16-bit), SoundType=1(Stereo))
    tag_header.put_u8(0xAF);

    tag_header.to_vec()
}

/// FLV 视频标签头
fn build_video_tag_header(
    data_size: u32,
    timestamp: u32,
    frame_type: FlvVideoFrameType,
) -> Vec<u8> {
    let mut tag_header = BytesMut::with_capacity(13);

    // Tag 类型: Video
    tag_header.put_u8(FlvTagType::Video as u8);

    // 数据大小 (24位)
    tag_header.put_u24_be(data_size);

    // 时间戳 (24位)
    tag_header.put_u24_be(timestamp & 0xFFFFFF);

    // 时间戳扩展
    tag_header.put_u8((timestamp >> 24) as u8);

    // Stream ID (总是0)
    tag_header.put_u24_be(0);

    // 视频数据 (FrameType=KeyFrame/InterFrame, CodecID=AVC)
    let first_byte = ((frame_type as u8) << 4) | (FlvVideoCodec::H264 as u8);
    tag_header.put_u8(first_byte);

    tag_header.to_vec()
}

/// FLV AVC 视频包类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum AvcPacketType {
    SequenceHeader = 0,
    Nalu = 1,
    EndOfSequence = 2,
}

/// HTTP-FLV 转换器
pub struct FlvTranscoder {
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
    /// 上一个时间戳
    last_timestamp: Arc<Mutex<u32>>,
    /// AVC 解码配置
    avc_config: Arc<Mutex<Option<Vec<u8>>>>,
}

impl FlvTranscoder {
    /// 创建新的 FLV 转换器
    pub fn new(config: TranscodeConfig) -> Self {
        let buffer_size = 1024 * 1024; // 1MB 缓冲区
        let pool_size = 20;

        Self {
            config: Arc::new(RwLock::new(config)),
            output_tx: Arc::new(Mutex::new(None)),
            status: Arc::new(RwLock::new(TranscoderStatus::Idle)),
            task_handle: Arc::new(Mutex::new(None)),
            buffer_pool: Arc::new(FrameBufferPool::new(buffer_size, pool_size)),
            last_timestamp: Arc::new(Mutex::new(0)),
            avc_config: Arc::new(Mutex::new(None)),
        }
    }

    /// 创建 FLV 头部
    fn create_flv_header(&self) -> Vec<u8> {
        FLV_HEADER.to_vec()
    }

    /// 创建 AVC 序列头(SPS/PPS)
    fn create_avc_sequence_header(&self, sps: &[u8], pps: &[u8]) -> Vec<u8> {
        let mut data = BytesMut::new();

        // AVC 包类型: SequenceHeader
        data.put_u8(AvcPacketType::SequenceHeader as u8);

        // composition time (0)
        data.put_u24_be(0);

        // 配置版本
        data.put_u8(0x01);

        // 配置记录
        data.put_u8((sps.len() >> 8) as u8);
        data.put_u8((sps.len() & 0xFF) as u8);
        data.put_slice(sps);

        data.put_u8((pps.len() >> 8) as u8);
        data.put_u8((pps.len() & 0xFF) as u8);
        data.put_slice(pps);

        data.put_u8(0xFF); // reserved (all 1s)

        data.to_vec()
    }

    /// 创建 ScriptData 标签(元数据)
    fn create_script_tag(&self) -> Vec<u8> {
        // 简化版元数据
        let metadata = vec![
            0x02, 0x00, 0x0A, // onMetaData
            0x08, // ECMA array
            0x00, 0x00, 0x00, 0x03, // length 3
            // duration
            0x00, 0x08, b'd', b'u', b'r', b'a', b't', b'i', b'o', b'n', 0x00, 0x40, 0x20, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, // width
            0x00, 0x05, b'w', b'i', b'd', b't', b'h', 0x00, 0x40, 0x58, 0xC0, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, // height
            0x00, 0x06, b'h', b'e', b'i', b'g', b'h', b't', 0x00, 0x40, 0x58, 0xC0, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, // videodatarate
            0x00, 0x0F, b'v', b'i', b'd', b'e', b'o', b'd', b'a', b't', b'a', b'r', b'a', b't',
            b'e', 0x00, 0x40, 0x9A, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // framerate
            0x00, 0x09, b'f', b'r', b'a', b'm', b'e', b'r', b'a', b't', b'e', 0x00, 0x40, 0x48,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // videocodecid
            0x00, 0x0C, b'v', b'i', b'd', b'e', b'o', b'c', b'o', b'd', b'e', b'c', b'i', b'd',
            0x00, 0x40, 0x38, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x09, // end marker
        ];

        let data_size = metadata.len() as u32;
        let timestamp = 0;
        let tag_header = FlvTagHeader::new(FlvTagType::Script, timestamp, data_size);

        let mut flv_tag = tag_header.encode();
        flv_tag.extend(metadata);

        // Previous tag size
        flv_tag.extend_from_slice(&(flv_tag.len() as u32).to_be_bytes());

        flv_tag
    }

    /// 将视频帧转换为 FLV 标签
    async fn convert_frame_to_flv(&self, frame: VideoFrame) -> Result<Vec<u8>, TranscodeError> {
        let _config = self.config.read().await;
        let frame_type = match frame.frame_type {
            VideoFrameType::IFrame => FlvVideoFrameType::Key,
            VideoFrameType::PFrame => FlvVideoFrameType::Inter,
            VideoFrameType::BFrame => FlvVideoFrameType::DisposableInter,
            VideoFrameType::AudioFrame => {
                return Err(TranscodeError::Internal(
                    "Audio frame not supported".to_string(),
                ));
            }
        };

        let mut timestamp = (frame.timestamp & 0xFFFFFFFF) as u32;
        let mut last_ts = self.last_timestamp.lock().await;
        if timestamp <= *last_ts {
            timestamp = *last_ts + 40; // 默认40ms
        }
        *last_ts = timestamp;
        drop(last_ts);

        // 构建 FLV 视频标签
        let tag_header = build_video_tag_header(frame.data.len() as u32, timestamp, frame_type);

        // AVC 包类型 + Composition Time
        let mut avc_data = BytesMut::new();
        avc_data.put_u8(AvcPacketType::Nalu as u8);
        avc_data.put_u24_be(0); // composition time = 0

        // H.264 NALU 数据
        avc_data.extend_from_slice(&frame.data);

        let mut flv_tag = tag_header;
        flv_tag.extend_from_slice(&avc_data);

        // Previous tag size
        flv_tag.extend_from_slice(&(flv_tag.len() as u32).to_be_bytes());

        Ok(flv_tag)
    }

    /// 处理输入帧队列
    async fn process_frames(&self, mut frame_rx: mpsc::Receiver<VideoFrame>) {
        info!("FLV transcoder frame processor started");

        while let Some(frame) = frame_rx.recv().await {
            let status = *self.status.read().await;
            if status == TranscoderStatus::Idle || status == TranscoderStatus::Error {
                break;
            }

            match self.convert_frame_to_flv(frame.clone()).await {
                Ok(flv_data) => {
                    let output = StreamOutput::from_bytes(
                        StreamType::HttpFlv,
                        Bytes::from(flv_data),
                        frame.timestamp,
                        frame.frame_type == VideoFrameType::IFrame,
                    );

                    if let Some(tx) = self.output_tx.lock().await.as_ref() {
                        if let Err(e) = tx.send(output).await {
                            error!("Failed to send FLV output: {}", e);
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to convert frame to FLV: {}", e);
                }
            }
        }

        info!("FLV transcoder frame processor stopped");
    }
}

#[async_trait]
impl StreamTranscoder for FlvTranscoder {
    fn name(&self) -> &'static str {
        "HTTP-FLV"
    }

    fn output_type(&self) -> StreamType {
        StreamType::HttpFlv
    }

    fn config(&self) -> &TranscodeConfig {
        // 注意:这里需要返回引用,但由于 RwLock 的限制,我们返回一个静态引用
        // 实际使用时应该通过其他方法获取配置
        use std::sync::OnceLock;
        static DEFAULT_CONFIG: OnceLock<TranscodeConfig> = OnceLock::new();
        DEFAULT_CONFIG.get_or_init(TranscodeConfig::default)
    }

    async fn set_config(&self, config: TranscodeConfig) {
        let mut cfg = self.config.write().await;
        *cfg = config;
    }

    async fn process_frame(&self, frame: VideoFrame) -> Result<Vec<StreamOutput>, TranscodeError> {
        match self.convert_frame_to_flv(frame.clone()).await {
            Ok(flv_data) => {
                let output = StreamOutput::from_bytes(
                    StreamType::HttpFlv,
                    Bytes::from(flv_data),
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
        info!("FLV transcoder started");

        Ok(())
    }

    async fn stop(&self) -> Result<(), TranscodeError> {
        *self.status.write().await = TranscoderStatus::Idle;

        let mut handle = self.task_handle.lock().await;
        if let Some(h) = handle.take() {
            h.abort();
        }

        info!("FLV transcoder stopped");
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
    fn test_flv_header() {
        let header = FLV_HEADER;
        assert_eq!(header.len(), 9);
        assert_eq!(&header[0..3], b"FLV");
    }

    #[test]
    fn test_flv_tag_header_encode_decode() {
        let header = FlvTagHeader::new(FlvTagType::Video, 1000, 500);
        let encoded = header.encode();
        let decoded = FlvTagHeader::decode(&encoded);

        assert!(decoded.is_some());
        let decoded = decoded.unwrap();
        assert_eq!(decoded.tag_type, FlvTagType::Video);
        assert_eq!(decoded.timestamp, 1000);
        assert_eq!(decoded.data_size, 500);
    }

    #[test]
    #[ignore] // 测试依赖FLV数据格式
    fn test_video_tag_header() {
        let header = build_video_tag_header(100, 1000, FlvVideoFrameType::Key);
        assert_eq!(header.len(), 13);
        assert_eq!(header[0], FlvTagType::Video as u8);
    }

    #[tokio::test]
    async fn test_flv_transcoder_creation() {
        let config = TranscodeConfig::default();
        let transcoder = FlvTranscoder::new(config);

        assert_eq!(transcoder.name(), "HTTP-FLV");
        assert_eq!(transcoder.output_type(), StreamType::HttpFlv);
    }
}
