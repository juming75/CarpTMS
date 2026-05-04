//! HTTP-FLV 流式输出服务器
//!
//! 实现JT1078视频流的HTTP-FLV输出功能
//! 客户端可通过HTTP GET请求获取FLV视频流
//! 对应流程：终端推流 → 服务器转码 → HTTP-FLV输出 → Web端播放

use actix_web::{get, web, Error, HttpResponse};
use bytes::Bytes;
use futures_util::Stream;
use log::{debug, info};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tokio::sync::{mpsc, RwLock};

use crate::video::{VideoFrame, VideoFrameType};

/// FLV标签头类型
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
enum FlvTagType {
    Audio = 8,
    Video = 9,
    ScriptData = 18,
}

/// HTTP-FLV流管理器
/// 管理所有活跃的FLV流订阅者
pub struct FlvStreamManager {
    /// 流订阅者 (stream_id -> 订阅者列表)
    subscribers: Arc<RwLock<HashMap<String, Vec<mpsc::UnboundedSender<FlvData>>>>>,
    /// 流统计信息
    stats: Arc<RwLock<FlvStreamStats>>,
}

/// FLV数据
#[derive(Debug, Clone)]
pub struct FlvData {
    /// FLV标签数据（包含标签头）
    pub tag: Vec<u8>,
    /// 是否为视频关键帧
    pub is_keyframe: bool,
}

/// FLV流统计信息
#[derive(Debug, Clone, Serialize, Default)]
pub struct FlvStreamStats {
    /// 活跃流数量
    pub active_streams: usize,
    /// 总订阅者数量
    pub total_subscribers: usize,
    /// 总发送字节数
    pub total_bytes_sent: u64,
    /// 总帧数
    pub total_frames_sent: u64,
}

impl FlvStreamManager {
    /// 创建新的FLV流管理器
    pub fn new() -> Self {
        Self {
            subscribers: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(FlvStreamStats::default())),
        }
    }

    /// 订阅FLV流
    /// 返回接收器用于接收FLV数据
    pub async fn subscribe(&self, stream_id: &str) -> mpsc::UnboundedReceiver<FlvData> {
        let (tx, rx) = mpsc::unbounded_channel();

        let mut subs = self.subscribers.write().await;
        subs.entry(stream_id.to_string())
            .or_insert_with(Vec::new)
            .push(tx);

        {
            let mut stats = self.stats.write().await;
            stats.active_streams = subs.len();
            stats.total_subscribers = subs.values().map(|v| v.len()).sum();
        }

        info!(
            "New FLV subscriber for stream {}, total: {}",
            stream_id,
            subs.get(stream_id).map(|v| v.len()).unwrap_or(0)
        );

        rx
    }

    /// 取消订阅
    pub async fn unsubscribe(&self, stream_id: &str, sender: &mpsc::UnboundedSender<FlvData>) {
        let mut subs = self.subscribers.write().await;
        if let Some(subscribers) = subs.get_mut(stream_id) {
            // 使用指针比较来找到要移除的 sender
            subscribers.retain(|tx| !std::ptr::eq(tx as *const _, sender as *const _));

            if subscribers.is_empty() {
                subs.remove(stream_id);
            }
        }

        {
            let mut stats = self.stats.write().await;
            stats.active_streams = subs.len();
            stats.total_subscribers = subs.values().map(|v| v.len()).sum();
        }
    }

    /// 推送FLV数据到订阅者
    pub async fn push_frame(&self, stream_id: &str, frame: &VideoFrame) -> Result<(), String> {
        let subs = self.subscribers.read().await;
        if let Some(subscribers) = subs.get(stream_id) {
            // 构建FLV标签
            let flv_tag = self.build_flv_tag(frame)?;
            let is_keyframe = frame.frame_type == VideoFrameType::IFrame;

            let flv_data = FlvData {
                tag: flv_tag,
                is_keyframe,
            };

            let mut removed = Vec::new();
            for (i, subscriber) in subscribers.iter().enumerate() {
                if subscriber.send(flv_data.clone()).is_err() {
                    removed.push(i);
                }
            }

            // 清理失效的订阅者
            if !removed.is_empty() {
                drop(subs);
                let mut subs = self.subscribers.write().await;
                if let Some(subscribers) = subs.get_mut(stream_id) {
                    for i in removed.into_iter().rev() {
                        subscribers.remove(i);
                    }
                    if subscribers.is_empty() {
                        subs.remove(stream_id);
                    }
                }
            }

            {
                let mut stats = self.stats.write().await;
                stats.total_frames_sent += 1;
                stats.total_bytes_sent += frame.data.len() as u64;
            }

            Ok(())
        } else {
            Err(format!("No subscribers for stream {}", stream_id))
        }
    }

    /// 构建FLV标签
    fn build_flv_tag(&self, frame: &VideoFrame) -> Result<Vec<u8>, String> {
        let mut tag = Vec::new();

        // FLV标签头（11字节）
        // 字节0: 标签类型
        match frame.frame_type {
            VideoFrameType::AudioFrame => {
                tag.push(FlvTagType::Audio as u8);
            }
            _ => {
                tag.push(FlvTagType::Video as u8);
            }
        }

        // 字节1-3: 数据长度（24位）
        let data_len = frame.data.len() + 5; // 5字节视频标签头
        tag.push(((data_len >> 16) & 0xFF) as u8);
        tag.push(((data_len >> 8) & 0xFF) as u8);
        tag.push((data_len & 0xFF) as u8);

        // 字节4-6: 时间戳（24位）
        let timestamp = frame.timestamp as u32;
        tag.push(((timestamp >> 16) & 0xFF) as u8);
        tag.push(((timestamp >> 8) & 0xFF) as u8);
        tag.push((timestamp & 0xFF) as u8);

        // 字节7: 时间戳扩展（8位）
        tag.push(((timestamp >> 24) & 0xFF) as u8);

        // 字节8-10: StreamID（始终为0）
        tag.push(0);
        tag.push(0);
        tag.push(0);

        // 视频标签头（5字节）
        if frame.frame_type != VideoFrameType::AudioFrame {
            // 字节0: 帧类型 + 编码ID
            // 帧类型：1=关键帧, 2=内部帧
            let frame_type = match frame.frame_type {
                VideoFrameType::IFrame => 1,
                _ => 2,
            };
            let codec_id = 7; // 7 = AVC (H.264)
            tag.push((frame_type << 4) | codec_id);

            // 字节1: AVC包类型
            // 0 = AVC sequence header, 1 = AVC NALU
            tag.push(1); // AVC NALU

            // 字节2-4: Composition Time Offset (24位)
            tag.push(0);
            tag.push(0);
            tag.push(0);
        }

        // 附加视频数据
        tag.extend_from_slice(&frame.data);

        // PreviousTagSize（4字节）
        let previous_tag_size = tag.len();
        tag.extend_from_slice(&(previous_tag_size as u32).to_be_bytes());

        Ok(tag)
    }

    /// 获取统计信息
    pub async fn get_stats(&self) -> FlvStreamStats {
        self.stats.read().await.clone()
    }
}

impl Default for FlvStreamManager {
    fn default() -> Self {
        Self::new()
    }
}

/// FLV流响应
struct FlvStreamResponse {
    receiver: mpsc::UnboundedReceiver<FlvData>,
    first_frame_sent: bool,
}

impl Stream for FlvStreamResponse {
    type Item = Result<Bytes, Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match Pin::new(&mut self.receiver).poll_recv(cx) {
            Poll::Ready(Some(flv_data)) => {
                self.first_frame_sent = true;
                Poll::Ready(Some(Ok(Bytes::from(flv_data.tag))))
            }
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

/// HTTP-FLV播放端点
/// GET /live/{stream_id}.flv
#[get("/live/{stream_id}.flv")]
pub async fn play_flv(
    path: web::Path<String>,
    query: web::Query<FlvPlayQuery>,
    flv_manager: web::Data<Arc<FlvStreamManager>>,
) -> Result<HttpResponse, Error> {
    let stream_id = path.into_inner();

    // 验证推流密钥（如果提供了key参数）
    if let Some(key) = &query.key {
        // TODO: 与DeviceManager集成，验证密钥
        debug!(
            "FLV play request with key: {} for stream {}",
            key, stream_id
        );
    }

    info!("FLV play request for stream: {}", stream_id);

    // 订阅FLV流
    let receiver = flv_manager.subscribe(&stream_id).await;

    // 构建FLV文件头（9字节）已废弃，不再使用文件头

    // 创建响应流
    let flv_stream = FlvStreamResponse {
        receiver,
        first_frame_sent: false,
    };

    // 返回FLV流响应
    Ok(HttpResponse::Ok()
        .append_header(("Content-Type", "video/x-flv"))
        .append_header(("Cache-Control", "no-cache"))
        .append_header(("Connection", "keep-alive"))
        .append_header(("Access-Control-Allow-Origin", "*"))
        .streaming(flv_stream))
}

/// FLV播放查询参数
#[derive(Debug, Deserialize)]
pub struct FlvPlayQuery {
    /// 推流密钥（可选）
    pub key: Option<String>,
}

/// 构建FLV文件头
/// 格式：FLV + 版本号(1) + 标志(5) + 头部长度(4)
#[allow(dead_code)]
fn build_flv_header() -> Vec<u8> {
    let mut header = vec![
        b'F',
        b'L',
        b'V',
        1,
        0x05,
    ];

    // 头部长度（9字节，4字节大端）
    header.extend_from_slice(&9u32.to_be_bytes());

    header
}

/// 构建FLV脚本标签（onMetaData）
#[allow(dead_code)]
fn build_script_tag() -> Vec<u8> {
    let mut tag = vec![
        18, // 标签类型
        0, 0, 0, // 数据长度
        0, 0, 0, 0, // 时间戳
        0, 0, 0, // StreamID
        0x02, 0x00, 0x0A, // onMetaData脚本数据
    ];

    tag.extend_from_slice(b"onMetaData");

    // PreviousTagSize
    tag.extend_from_slice(&(tag.len() as u32).to_be_bytes());

    tag
}

/// 获取FLV统计信息
/// GET /api/flv/stats
pub async fn get_flv_stats(flv_manager: web::Data<Arc<FlvStreamManager>>) -> HttpResponse {
    let stats = flv_manager.get_stats().await;
    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "stats": stats
    }))
}

/// 配置FLV路由
pub fn configure_flv_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(play_flv)
        .route("/api/flv/stats", web::get().to(get_flv_stats));
}

/// 创建FLV流管理器（便捷函数）
pub fn create_flv_manager() -> Arc<FlvStreamManager> {
    Arc::new(FlvStreamManager::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flv_header() {
        let header = build_flv_header();
        assert_eq!(header.len(), 9);
        assert_eq!(&header[0..3], b"FLV");
        assert_eq!(header[3], 1); // 版本号
        assert_eq!(header[4], 0x05); // 有音频和视频
    }
}
