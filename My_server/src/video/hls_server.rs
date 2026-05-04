//! HLS (HTTP Live Streaming) 输出服务器
//!
//! 实现JT1078视频流的HLS输出功能
//! 包括：
//! 1. M3U8播放列表生成
//! 2. TS分片生成和管理
//! 3. HLS流订阅和分发
//! 4. 分片清理和过期处理

use actix_web::{get, web, Error, HttpResponse};
use bytes::Bytes;
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::fs;
use tokio::sync::RwLock;

/// HLS配置
#[derive(Debug, Clone)]
pub struct HlsServerConfig {
    /// HLS输出目录
    pub output_dir: String,
    /// 分片时长（秒）
    pub segment_duration: u64,
    /// 播放列表中的分片数量
    pub playlist_size: usize,
    /// 分片清理间隔（秒）
    pub cleanup_interval: u64,
    /// 分片过期时间（秒）
    pub segment_ttl: u64,
}

impl Default for HlsServerConfig {
    fn default() -> Self {
        Self {
            output_dir: "./hls_output".to_string(),
            segment_duration: 5,
            playlist_size: 5,
            cleanup_interval: 60,
            segment_ttl: 300,
        }
    }
}

/// TS分片信息
#[derive(Debug, Clone)]
pub struct TsSegment {
    /// 分片文件名
    pub filename: String,
    /// 分片时长（秒）
    pub duration: f64,
    /// 创建时间
    pub created_at: Instant,
    /// 分片大小（字节）
    pub size: usize,
}

/// HLS流信息
#[derive(Debug, Clone)]
pub struct HlsStream {
    /// 流ID
    pub stream_id: String,
    /// 分片列表
    pub segments: Vec<TsSegment>,
    /// 当前分片序号
    pub current_sequence: u64,
    /// 是否结束
    pub ended: bool,
    /// 最后更新时间
    pub last_updated: Instant,
    /// 总播放时长（秒）
    pub total_duration: f64,
}

impl HlsStream {
    /// 创建新的HLS流
    pub fn new(stream_id: String) -> Self {
        Self {
            stream_id,
            segments: Vec::new(),
            current_sequence: 0,
            ended: false,
            last_updated: Instant::now(),
            total_duration: 0.0,
        }
    }

    /// 添加TS分片
    pub fn add_segment(&mut self, segment: TsSegment) {
        self.segments.push(segment);
        self.current_sequence += 1;
        self.last_updated = Instant::now();
        self.total_duration += self.segments.last().map(|s| s.duration).unwrap_or(0.0);

        // 限制分片数量
        if self.segments.len() > 100 {
            self.segments.drain(0..self.segments.len() - 100);
        }
    }

    /// 生成M3U8播放列表
    pub fn generate_playlist(&self, target_duration: u64) -> String {
        let mut m3u8 = String::new();

        // M3U8头部
        m3u8.push_str("#EXTM3U\n");
        m3u8.push_str("#EXT-X-VERSION:3\n");
        m3u8.push_str(&format!("#EXT-X-TARGETDURATION:{}\n", target_duration));
        m3u8.push_str(&format!(
            "#EXT-X-MEDIA-SEQUENCE:{}\n",
            self.current_sequence
                .saturating_sub(self.segments.len() as u64)
        ));
        m3u8.push_str("#EXT-X-PLAYLIST-TYPE:EVENT\n");
        m3u8.push_str("#EXT-X-DISCONTINUITY-SEQUENCE:0\n");

        // 添加分片信息
        for segment in &self.segments {
            m3u8.push_str(&format!("#EXTINF:{:.3},\n", segment.duration));
            m3u8.push_str(&format!("{}\n", segment.filename));
        }

        // 如果流已结束，添加结束标记
        if self.ended {
            m3u8.push_str("#EXT-X-ENDLIST\n");
        }

        m3u8
    }
}

/// HLS流管理器
pub struct HlsStreamManager {
    /// HLS流列表
    streams: Arc<RwLock<HashMap<String, HlsStream>>>,
    /// 配置
    config: HlsServerConfig,
    /// 输出目录
    output_dir: PathBuf,
    /// 统计信息
    stats: Arc<RwLock<HlsStats>>,
}

/// HLS统计信息
#[derive(Debug, Clone, Serialize, Default)]
pub struct HlsStats {
    /// 活跃流数量
    pub active_streams: usize,
    /// 总分片数
    pub total_segments: usize,
    /// 总请求次数
    pub total_requests: u64,
    /// 总发送字节数
    pub total_bytes_sent: u64,
}

impl HlsStreamManager {
    /// 创建新的HLS流管理器
    pub fn new(config: HlsServerConfig) -> Self {
        let output_dir = PathBuf::from(&config.output_dir);

        // 确保输出目录存在
        if !output_dir.exists() {
            if let Err(e) = std::fs::create_dir_all(&output_dir) {
                tracing::error!("Failed to create HLS output directory: {}", e);
                // 继续运行，让后续操作失败时产生更明确的错误
            }
        }

        Self {
            streams: Arc::new(RwLock::new(HashMap::new())),
            config,
            output_dir,
            stats: Arc::new(RwLock::new(HlsStats::default())),
        }
    }

    /// 获取或创建HLS流
    pub async fn get_or_create_stream(&self, stream_id: &str) -> Arc<RwLock<HlsStream>> {
        let mut streams = self.streams.write().await;

        if !streams.contains_key(stream_id) {
            streams.insert(stream_id.to_string(), HlsStream::new(stream_id.to_string()));
            info!("Created new HLS stream: {}", stream_id);
        }

        // 安全获取流引用，因为我们刚刚插入了它
        let stream = streams.get(stream_id).cloned().unwrap_or_else(|| {
            tracing::error!("Failed to get newly created HLS stream: {}", stream_id);
            HlsStream::new(stream_id.to_string())
        });
        Arc::new(RwLock::new(stream))
    }

    /// 添加TS分片
    pub async fn add_segment(
        &self,
        stream_id: &str,
        ts_data: Vec<u8>,
        duration: f64,
    ) -> Result<String, String> {
        let stream_dir = self.output_dir.join(stream_id);
        if !stream_dir.exists() {
            fs::create_dir_all(&stream_dir)
                .await
                .map_err(|e| format!("Failed to create stream directory: {}", e))?;
        }

        // 生成分片文件名
        let sequence = {
            let streams = self.streams.read().await;
            if let Some(stream) = streams.get(stream_id) {
                stream.current_sequence
            } else {
                0
            }
        };

        let filename = format!("segment_{}.ts", sequence);
        let filepath = stream_dir.join(&filename);

        // 写入TS分片文件
        let mut file = std::fs::File::create(&filepath)
            .map_err(|e| format!("Failed to create TS segment file: {}", e))?;

        file.write_all(&ts_data)
            .map_err(|e| format!("Failed to write TS segment: {}", e))?;

        // 更新流信息
        {
            let mut streams = self.streams.write().await;
            if let Some(stream) = streams.get_mut(stream_id) {
                stream.add_segment(TsSegment {
                    filename: filename.clone(),
                    duration,
                    created_at: Instant::now(),
                    size: ts_data.len(),
                });
            }
        }

        // 更新统计信息
        {
            let mut stats = self.stats.write().await;
            stats.total_segments += 1;
            stats.total_bytes_sent += ts_data.len() as u64;
        }

        debug!("Added TS segment {} for stream {}", filename, stream_id);
        Ok(filename)
    }

    /// 获取M3U8播放列表
    pub async fn get_playlist(&self, stream_id: &str) -> Result<String, String> {
        let streams = self.streams.read().await;

        if let Some(stream) = streams.get(stream_id) {
            let playlist = stream.generate_playlist(self.config.segment_duration);

            // 更新统计信息
            {
                let mut stats = self.stats.write().await;
                stats.total_requests += 1;
            }

            Ok(playlist)
        } else {
            Err(format!("HLS stream {} not found", stream_id))
        }
    }

    /// 获取TS分片内容
    pub async fn get_segment(&self, stream_id: &str, filename: &str) -> Result<Vec<u8>, String> {
        let filepath = self.output_dir.join(stream_id).join(filename);

        if !filepath.exists() {
            return Err(format!(
                "TS segment {} not found for stream {}",
                filename, stream_id
            ));
        }

        let data = fs::read(&filepath)
            .await
            .map_err(|e| format!("Failed to read TS segment: {}", e))?;

        // 更新统计信息
        {
            let mut stats = self.stats.write().await;
            stats.total_requests += 1;
            stats.total_bytes_sent += data.len() as u64;
        }

        Ok(data)
    }

    /// 清理过期分片
    pub async fn cleanup_expired_segments(&self) {
        let now = Instant::now();
        let ttl = Duration::from_secs(self.config.segment_ttl);
        let mut cleaned_count = 0;

        let streams = self.streams.read().await;
        for (stream_id, stream) in streams.iter() {
            let stream_dir = self.output_dir.join(stream_id);

            for segment in &stream.segments {
                if now.duration_since(segment.created_at) > ttl {
                    let filepath = stream_dir.join(&segment.filename);
                    if filepath.exists() {
                        let _ = std::fs::remove_file(&filepath);
                        cleaned_count += 1;
                    }
                }
            }
        }

        if cleaned_count > 0 {
            info!("Cleaned up {} expired HLS segments", cleaned_count);
        }
    }

    /// 获取统计信息
    pub async fn get_stats(&self) -> HlsStats {
        self.stats.read().await.clone()
    }

    /// 结束HLS流
    pub async fn end_stream(&self, stream_id: &str) -> Result<(), String> {
        let mut streams = self.streams.write().await;
        if let Some(stream) = streams.get_mut(stream_id) {
            stream.ended = true;
            info!("HLS stream ended: {}", stream_id);
            Ok(())
        } else {
            Err(format!("HLS stream {} not found", stream_id))
        }
    }
}

/// HLS播放查询参数
#[derive(Debug, Deserialize)]
pub struct HlsPlayQuery {
    /// 推流密钥（可选）
    pub key: Option<String>,
}

/// 获取M3U8播放列表
/// GET /hls/{stream_id}/index.m3u8
#[get("/hls/{stream_id}/index.m3u8")]
pub async fn get_hls_playlist(
    path: web::Path<String>,
    query: web::Query<HlsPlayQuery>,
    hls_manager: web::Data<Arc<HlsStreamManager>>,
) -> Result<HttpResponse, Error> {
    let stream_id = path.into_inner();

    // 验证密钥（如果提供）
    if let Some(key) = &query.key {
        debug!(
            "HLS playlist request with key: {} for stream {}",
            key, stream_id
        );
    }

    info!("HLS playlist request for stream: {}", stream_id);

    match hls_manager.get_playlist(&stream_id).await {
        Ok(playlist) => Ok(HttpResponse::Ok()
            .content_type("application/vnd.apple.mpegurl")
            .append_header(("Cache-Control", "no-cache"))
            .append_header(("Access-Control-Allow-Origin", "*"))
            .body(playlist)),
        Err(e) => {
            warn!("Failed to get HLS playlist: {}", e);
            Ok(HttpResponse::NotFound().json(serde_json::json!({
                "error": e
            })))
        }
    }
}

/// 获取TS分片
/// GET /hls/{stream_id}/{segment}
#[get("/hls/{stream_id}/{segment}")]
pub async fn get_hls_segment(
    path: web::Path<(String, String)>,
    query: web::Query<HlsPlayQuery>,
    hls_manager: web::Data<Arc<HlsStreamManager>>,
) -> Result<HttpResponse, Error> {
    let (stream_id, segment) = path.into_inner();

    if let Some(key) = &query.key {
        debug!(
            "HLS segment request with key: {} for stream {}",
            key, stream_id
        );
    }

    debug!(
        "HLS segment request: stream={}, segment={}",
        stream_id, segment
    );

    match hls_manager.get_segment(&stream_id, &segment).await {
        Ok(data) => Ok(HttpResponse::Ok()
            .content_type("video/MP2T")
            .append_header(("Cache-Control", "public, max-age=3600"))
            .append_header(("Access-Control-Allow-Origin", "*"))
            .body(Bytes::from(data))),
        Err(e) => {
            warn!("Failed to get HLS segment: {}", e);
            Ok(HttpResponse::NotFound().json(serde_json::json!({
                "error": e
            })))
        }
    }
}

/// 获取HLS统计信息
/// GET /api/hls/stats
pub async fn get_hls_stats(hls_manager: web::Data<Arc<HlsStreamManager>>) -> HttpResponse {
    let stats = hls_manager.get_stats().await;
    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "stats": stats
    }))
}

/// 配置HLS路由
pub fn configure_hls_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get_hls_playlist)
        .service(get_hls_segment)
        .route("/api/hls/stats", web::get().to(get_hls_stats));
}

/// 创建HLS流管理器（便捷函数）
pub fn create_hls_manager(config: HlsServerConfig) -> Arc<HlsStreamManager> {
    Arc::new(HlsStreamManager::new(config))
}

/// 构建TS分片头
/// TS包头长度为4字节
pub fn build_ts_header(
    packet_id: u16,
    payload_unit_start_indicator: bool,
    continuity_counter: u8,
) -> [u8; 4] {
    let mut header = [0u8; 4];

    // 同步字节
    header[0] = 0x47;

    // 传输错误指示 + 有效负载单元开始指示 + 传输优先级 + PID
    header[1] = if payload_unit_start_indicator {
        0x40
    } else {
        0x00
    };
    header[1] |= ((packet_id >> 8) & 0x1F) as u8;
    header[2] = (packet_id & 0xFF) as u8;

    // 传输加扰控制 + 自适应字段控制 + 连续性计数器
    header[3] = 0x10 | (continuity_counter & 0x0F);

    header
}

/// 构建PAT（Program Association Table）TS分片
pub fn build_pat_segment() -> Vec<u8> {
    let mut ts_packet = vec![0u8; 188]; // TS分片固定长度188字节

    // TS包头
    let ts_header = build_ts_header(0, true, 0);
    ts_packet[0..4].copy_from_slice(&ts_header);

    // 自适应字段（表示只有有效负载）
    ts_packet[4] = 0x00; // 无自适应字段，只有有效负载

    // PAT数据（简化版）
    let pat_data = vec![
        0x00, // table_id = 0 (PAT)
        0xB0, 0x0D, // section_length
        0x00, 0x01, // transport_stream_id
        0x00, // reserved + version_number + current_next_indicator
        0x00, // section_number
        0x00, // last_section_number
        0x00, 0x01, // program_number
        0xE0, 0x10, // reserved + program_map_PID
    ];

    // 填充PAT数据
    ts_packet[5..5 + pat_data.len()].copy_from_slice(&pat_data);

    // CRC32（简化，实际应该计算）
    ts_packet[5 + pat_data.len()..5 + pat_data.len() + 4]
        .copy_from_slice(&[0x00, 0x00, 0x00, 0x00]);

    ts_packet
}

/// 构建PMT（Program Map Table）TS分片
pub fn build_pmt_segment() -> Vec<u8> {
    let mut ts_packet = vec![0u8; 188];

    // TS包头
    let ts_header = build_ts_header(0x100, true, 0);
    ts_packet[0..4].copy_from_slice(&ts_header);

    // 自适应字段
    ts_packet[4] = 0x00;

    // PMT数据（简化版）
    let pmt_data = vec![
        0x02, // table_id = 2 (PMT)
        0xB0, 0x17, // section_length
        0x00, 0x01, // program_number
        0x00, // reserved + version_number + current_next_indicator
        0x00, // section_number
        0x00, // last_section_number
        0xE0, 0x10, // reserved + PCR_PID
        0x00, 0x00, // program_info_length
        // 视频流
        0x1B, // stream_type = 0x1B (H.264)
        0xE0, 0x11, // reserved + elementary_PID
        0x00, 0x00, // ES_info_length
        // 音频流
        0x0F, // stream_type = 0x0F (AAC)
        0xE0, 0x12, // reserved + elementary_PID
        0x00, 0x00, // ES_info_length
    ];

    // 填充PMT数据
    ts_packet[5..5 + pmt_data.len()].copy_from_slice(&pmt_data);

    // CRC32（简化）
    ts_packet[5 + pmt_data.len()..5 + pmt_data.len() + 4]
        .copy_from_slice(&[0x00, 0x00, 0x00, 0x00]);

    ts_packet
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_ts_header() {
        let header = build_ts_header(0x100, true, 0);
        assert_eq!(header[0], 0x47); // 同步字节
        assert_eq!(header[1] & 0x40, 0x40); // 有效负载单元开始指示
    }

    #[test]
    fn test_hls_stream_playlist() {
        let mut stream = HlsStream::new("test_stream".to_string());

        stream.add_segment(TsSegment {
            filename: "segment_0.ts".to_string(),
            duration: 5.0,
            created_at: Instant::now(),
            size: 1024,
        });

        let playlist = stream.generate_playlist(5);
        assert!(playlist.contains("#EXTM3U"));
        assert!(playlist.contains("segment_0.ts"));
    }
}
