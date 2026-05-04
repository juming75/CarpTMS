//! 视频卡顿优化模块
//!
//! 提供视频卡顿问题的解决方案：
//! - 动态调整视频参数（分辨率、帧率）
//! - 根据网络质量优化传输策略
//! - 前置缓存机制
//! - 错峰传输优化

use log::info;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

/// 视频参数配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoParams {
    /// 分辨率（宽度）
    pub width: u32,
    /// 分辨率（高度）
    pub height: u32,
    /// 帧率（fps）
    pub fps: u32,
    /// 码率（kbps）
    pub bitrate_kbps: u32,
}

impl VideoParams {
    /// 创建高清参数
    pub fn hd() -> Self {
        Self {
            width: 1280,
            height: 720,
            fps: 25,
            bitrate_kbps: 2000,
        }
    }

    /// 创建标清参数
    pub fn sd() -> Self {
        Self {
            width: 640,
            height: 480,
            fps: 15,
            bitrate_kbps: 800,
        }
    }

    /// 创建低清参数
    pub fn ld() -> Self {
        Self {
            width: 320,
            height: 240,
            fps: 10,
            bitrate_kbps: 400,
        }
    }
}

/// 网络质量等级
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkQuality {
    /// 优秀
    Excellent,
    /// 良好
    Good,
    /// 一般
    Fair,
    /// 差
    Poor,
}

impl NetworkQuality {
    /// 根据延迟和丢包率评估网络质量
    pub fn evaluate(latency_ms: u64, packet_loss_percent: f64) -> Self {
        if latency_ms < 50 && packet_loss_percent < 1.0 {
            NetworkQuality::Excellent
        } else if latency_ms < 100 && packet_loss_percent < 3.0 {
            NetworkQuality::Good
        } else if latency_ms < 200 && packet_loss_percent < 5.0 {
            NetworkQuality::Fair
        } else {
            NetworkQuality::Poor
        }
    }

    /// 获取推荐的视频参数
    pub fn recommended_params(&self) -> VideoParams {
        match self {
            NetworkQuality::Excellent => VideoParams::hd(),
            NetworkQuality::Good => VideoParams {
                width: 960,
                height: 540,
                fps: 20,
                bitrate_kbps: 1500,
            },
            NetworkQuality::Fair => VideoParams::sd(),
            NetworkQuality::Poor => VideoParams::ld(),
        }
    }
}

/// 前置缓存配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeCacheConfig {
    /// 是否启用边缘缓存
    pub enabled: bool,
    /// 缓存时长（秒）
    pub cache_duration: u64,
    /// 缓存大小限制（MB）
    pub max_cache_size_mb: u64,
    /// 预加载策略
    pub preload_strategy: PreloadStrategy,
}

impl Default for EdgeCacheConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            cache_duration: 30,
            max_cache_size_mb: 512,
            preload_strategy: PreloadStrategy::Smart,
        }
    }
}

/// 预加载策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PreloadStrategy {
    /// 不预加载
    None,
    /// 始终预加载
    Always,
    /// 智能预加载（基于网络质量）
    Smart,
}

/// 视频流优化器
pub struct VideoStreamOptimizer {
    /// 当前视频参数
    current_params: Arc<RwLock<HashMap<String, VideoParams>>>,
    /// 网络质量历史
    network_history: Arc<RwLock<HashMap<String, Vec<NetworkQualitySample>>>>,
    /// 边缘缓存配置
    edge_cache_config: Arc<RwLock<EdgeCacheConfig>>,
    /// 统计信息
    stats: Arc<RwLock<OptimizerStats>>,
}

/// 网络质量采样
#[derive(Debug, Clone)]
pub struct NetworkQualitySample {
    /// 采样时间
    pub timestamp: Instant,
    /// 延迟（毫秒）
    pub latency_ms: u64,
    /// 丢包率（百分比）
    pub packet_loss_percent: f64,
    /// 评估的网络质量
    pub quality: NetworkQuality,
}

/// 优化器统计信息
#[derive(Debug, Clone, Default)]
pub struct OptimizerStats {
    /// 参数调整次数
    pub param_adjustments: u64,
    /// 缓存命中次数
    pub cache_hits: u64,
    /// 缓存未命中次数
    pub cache_misses: u64,
    /// 优化收益（减少的卡顿次数）
    pub stuttering_reduction: u64,
}

impl Default for VideoStreamOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

impl VideoStreamOptimizer {
    /// 创建新的视频流优化器
    pub fn new() -> Self {
        Self {
            current_params: Arc::new(RwLock::new(HashMap::new())),
            network_history: Arc::new(RwLock::new(HashMap::new())),
            edge_cache_config: Arc::new(RwLock::new(EdgeCacheConfig::default())),
            stats: Arc::new(RwLock::new(OptimizerStats::default())),
        }
    }

    /// 优化视频流参数
    pub async fn optimize_stream(
        &self,
        stream_id: &str,
        latency_ms: u64,
        packet_loss_percent: f64,
    ) -> VideoParams {
        // 评估网络质量
        let quality = NetworkQuality::evaluate(latency_ms, packet_loss_percent);

        // 记录网络质量采样
        self.record_network_sample(stream_id, latency_ms, packet_loss_percent, quality.clone())
            .await;

        // 获取推荐参数
        let recommended_params = quality.recommended_params();

        // 检查是否需要调整参数
        let should_adjust = self
            .should_adjust_params(stream_id, &recommended_params)
            .await;

        if should_adjust {
            self.adjust_params(stream_id, &recommended_params).await;

            let mut stats = self.stats.write().await;
            stats.param_adjustments += 1;

            info!(
                "Adjusted video params for stream {}: {}x{} @ {}fps, {}kbps",
                stream_id,
                recommended_params.width,
                recommended_params.height,
                recommended_params.fps,
                recommended_params.bitrate_kbps
            );
        }

        recommended_params
    }

    /// 记录网络质量采样
    async fn record_network_sample(
        &self,
        stream_id: &str,
        latency_ms: u64,
        packet_loss_percent: f64,
        quality: NetworkQuality,
    ) {
        let sample = NetworkQualitySample {
            timestamp: Instant::now(),
            latency_ms,
            packet_loss_percent,
            quality,
        };

        let mut history = self.network_history.write().await;
        let samples = history
            .entry(stream_id.to_string())
            .or_insert_with(Vec::new);
        samples.push(sample);

        // 只保留最近60个采样
        if samples.len() > 60 {
            samples.drain(..samples.len() - 60);
        }
    }

    /// 判断是否需要调整参数
    async fn should_adjust_params(&self, stream_id: &str, new_params: &VideoParams) -> bool {
        let current_params = self.current_params.read().await;

        if let Some(current) = current_params.get(stream_id) {
            // 如果参数差异较大，需要调整
            let width_diff = (current.width as i64 - new_params.width as i64).abs();
            let fps_diff = (current.fps as i64 - new_params.fps as i64).abs();
            let bitrate_diff = (current.bitrate_kbps as i64 - new_params.bitrate_kbps as i64).abs();

            width_diff > 100 || fps_diff > 5 || bitrate_diff > 300
        } else {
            true
        }
    }

    /// 调整视频参数
    async fn adjust_params(&self, stream_id: &str, params: &VideoParams) {
        let mut current_params = self.current_params.write().await;
        current_params.insert(stream_id.to_string(), params.clone());
    }

    /// 获取当前视频参数
    pub async fn get_current_params(&self, stream_id: &str) -> Option<VideoParams> {
        let current_params = self.current_params.read().await;
        current_params.get(stream_id).cloned()
    }

    /// 配置边缘缓存
    pub async fn configure_edge_cache(&self, config: EdgeCacheConfig) {
        let mut edge_cache_config = self.edge_cache_config.write().await;
        *edge_cache_config = config;
        info!("Edge cache configuration updated");
    }

    /// 检查是否应该使用缓存
    pub async fn should_use_cache(&self, stream_id: &str) -> bool {
        let config = self.edge_cache_config.read().await;

        if !config.enabled {
            return false;
        }

        // 根据网络质量决定是否使用缓存
        let network_history = self.network_history.read().await;
        if let Some(samples) = network_history.get(stream_id) {
            if let Some(last_sample) = samples.last() {
                return matches!(
                    last_sample.quality,
                    NetworkQuality::Fair | NetworkQuality::Poor
                );
            }
        }

        matches!(
            config.preload_strategy,
            PreloadStrategy::Always | PreloadStrategy::Smart
        )
    }

    /// 获取统计信息
    pub async fn get_stats(&self) -> OptimizerStats {
        self.stats.read().await.clone()
    }
}

/// 创建视频流优化器实例
pub fn create_video_stream_optimizer() -> VideoStreamOptimizer {
    VideoStreamOptimizer::new()
}
