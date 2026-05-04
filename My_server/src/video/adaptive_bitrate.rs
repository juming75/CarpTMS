//! 动态码率调整模块
//!
//! 根据网络质量动态调整视频编码参数
//! 实现QoS自适应码率功能
//! 适用场景：4G/5G移动网络下优化视频传输质量

use log::{debug, info};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// 网络质量等级
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NetworkQuality {
    /// 优秀（>5Mbps，延迟<100ms）
    Excellent,
    /// 良好（2-5Mbps，延迟100-200ms）
    Good,
    /// 一般（1-2Mbps，延迟200-400ms）
    Fair,
    /// 较差（0.5-1Mbps，延迟400-800ms）
    Poor,
    /// 极差（<0.5Mbps，延迟>800ms）
    VeryPoor,
}

impl NetworkQuality {
    /// 获取推荐码率（kbps）
    pub fn recommended_bitrate(&self) -> u32 {
        match self {
            NetworkQuality::Excellent => 4096,
            NetworkQuality::Good => 2048,
            NetworkQuality::Fair => 1024,
            NetworkQuality::Poor => 512,
            NetworkQuality::VeryPoor => 256,
        }
    }

    /// 获取推荐帧率
    pub fn recommended_framerate(&self) -> u8 {
        match self {
            NetworkQuality::Excellent => 30,
            NetworkQuality::Good => 25,
            NetworkQuality::Fair => 20,
            NetworkQuality::Poor => 15,
            NetworkQuality::VeryPoor => 10,
        }
    }

    /// 获取推荐GOP（关键帧间隔，帧数）
    pub fn recommended_gop(&self) -> u16 {
        match self {
            NetworkQuality::Excellent => 60,
            NetworkQuality::Good => 50,
            NetworkQuality::Fair => 40,
            NetworkQuality::Poor => 30,
            NetworkQuality::VeryPoor => 20,
        }
    }

    /// 获取推荐缓冲区大小（毫秒）
    pub fn recommended_buffer_ms(&self) -> u32 {
        match self {
            NetworkQuality::Excellent => 1000,
            NetworkQuality::Good => 2000,
            NetworkQuality::Fair => 3000,
            NetworkQuality::Poor => 4000,
            NetworkQuality::VeryPoor => 5000,
        }
    }
}

/// 网络测量数据
#[derive(Debug, Clone)]
pub struct NetworkMeasurement {
    /// 测量时间
    pub timestamp: Instant,
    /// 带宽估计（Mbps）
    pub bandwidth_mbps: f64,
    /// 延迟（毫秒）
    pub latency_ms: f64,
    /// 丢包率（百分比）
    pub packet_loss_rate: f64,
    /// 抖动（毫秒）
    pub jitter_ms: f64,
}

/// 码率配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BitrateConfig {
    /// 当前码率（kbps）
    pub current_bitrate: u32,
    /// 最小码率（kbps）
    pub min_bitrate: u32,
    /// 最大码率（kbps）
    pub max_bitrate: u32,
    /// 当前帧率
    pub current_framerate: u8,
    /// 当前GOP
    pub current_gop: u16,
    /// 当前缓冲区大小（毫秒）
    pub buffer_size_ms: u32,
}

impl Default for BitrateConfig {
    fn default() -> Self {
        Self {
            current_bitrate: 2048,
            min_bitrate: 256,
            max_bitrate: 4096,
            current_framerate: 25,
            current_gop: 50,
            buffer_size_ms: 3000,
        }
    }
}

/// 码率调整事件
#[derive(Debug, Clone, Serialize)]
pub struct BitrateAdjustmentEvent {
    /// 事件时间
    pub timestamp: String,
    /// 调整原因
    pub reason: String,
    /// 旧码率
    pub old_bitrate: u32,
    /// 新码率
    pub new_bitrate: u32,
    /// 网络质量
    pub network_quality: NetworkQuality,
}

/// 动态码率调整器
/// 根据网络质量自动调整视频编码参数
pub struct AdaptiveBitrateController {
    /// 网络测量历史（保留最近20个样本）
    measurements: Arc<RwLock<VecDeque<NetworkMeasurement>>>,
    /// 当前码率配置
    config: Arc<RwLock<BitrateConfig>>,
    /// 调整历史
    adjustment_history: Arc<RwLock<Vec<BitrateAdjustmentEvent>>>,
    /// 最后调整时间
    last_adjustment: Arc<RwLock<Instant>>,
    /// 调整间隔（防止频繁调整）
    min_adjust_interval: Duration,
    /// 流ID
    stream_id: String,
}

impl AdaptiveBitrateController {
    /// 创建新的动态码率调整器
    pub fn new(stream_id: String, min_adjust_interval: Duration) -> Self {
        Self {
            measurements: Arc::new(RwLock::new(VecDeque::with_capacity(20))),
            config: Arc::new(RwLock::new(BitrateConfig::default())),
            adjustment_history: Arc::new(RwLock::new(Vec::new())),
            last_adjustment: Arc::new(RwLock::new(Instant::now())),
            min_adjust_interval,
            stream_id,
        }
    }

    /// 记录网络测量数据
    pub async fn record_measurement(&self, measurement: NetworkMeasurement) {
        let mut measurements = self.measurements.write().await;
        let bandwidth = measurement.bandwidth_mbps;
        let latency = measurement.latency_ms;
        measurements.push_back(measurement);

        // 保留最近20个样本
        if measurements.len() > 20 {
            measurements.pop_front();
        }

        debug!(
            "Recorded network measurement for stream {}: bandwidth={:.2}Mbps, latency={:.0}ms",
            self.stream_id, bandwidth, latency
        );

        // 尝试调整码率
        let _ = self.adjust_bitrate().await;
    }

    /// 评估网络质量
    async fn evaluate_network_quality(&self) -> NetworkQuality {
        let measurements = self.measurements.read().await;
        if measurements.is_empty() {
            return NetworkQuality::Good; // 默认良好
        }

        // 计算平均值
        let avg_bandwidth: f64 =
            measurements.iter().map(|m| m.bandwidth_mbps).sum::<f64>() / measurements.len() as f64;
        let avg_latency: f64 =
            measurements.iter().map(|m| m.latency_ms).sum::<f64>() / measurements.len() as f64;
        let avg_packet_loss: f64 = measurements.iter().map(|m| m.packet_loss_rate).sum::<f64>()
            / measurements.len() as f64;

        // 综合评估网络质量
        if avg_bandwidth > 5.0 && avg_latency < 100.0 && avg_packet_loss < 1.0 {
            NetworkQuality::Excellent
        } else if avg_bandwidth > 2.0 && avg_latency < 200.0 && avg_packet_loss < 3.0 {
            NetworkQuality::Good
        } else if avg_bandwidth > 1.0 && avg_latency < 400.0 && avg_packet_loss < 5.0 {
            NetworkQuality::Fair
        } else if avg_bandwidth > 0.5 && avg_latency < 800.0 && avg_packet_loss < 10.0 {
            NetworkQuality::Poor
        } else {
            NetworkQuality::VeryPoor
        }
    }

    /// 执行码率调整
    pub async fn adjust_bitrate(&self) -> Option<BitrateAdjustmentEvent> {
        let now = Instant::now();
        let last_adjustment = *self.last_adjustment.read().await;

        // 检查是否满足调整间隔要求
        if now.duration_since(last_adjustment) < self.min_adjust_interval {
            return None;
        }

        let quality = self.evaluate_network_quality().await;
        let mut config = self.config.write().await;

        let recommended_bitrate = quality.recommended_bitrate();
        let recommended_framerate = quality.recommended_framerate();
        let recommended_gop = quality.recommended_gop();
        let recommended_buffer = quality.recommended_buffer_ms();

        // 检查是否需要调整
        let needs_adjustment = (config.current_bitrate as i32 - recommended_bitrate as i32).abs()
            > 256
            || config.current_framerate != recommended_framerate
            || config.current_gop != recommended_gop;

        if needs_adjustment {
            let old_bitrate = config.current_bitrate;

            // 限制在最小和最大码率范围内
            config.current_bitrate = recommended_bitrate
                .max(config.min_bitrate)
                .min(config.max_bitrate);
            config.current_framerate = recommended_framerate;
            config.current_gop = recommended_gop;
            config.buffer_size_ms = recommended_buffer;

            let event = BitrateAdjustmentEvent {
                timestamp: chrono::Utc::now().to_rfc3339(),
                reason: format!("Network quality: {:?}", quality),
                old_bitrate,
                new_bitrate: config.current_bitrate,
                network_quality: quality,
            };

            // 记录调整历史
            let mut history = self.adjustment_history.write().await;
            history.push(event.clone());

            // 保留最近100条调整记录
            let len = history.len();
            if len > 100 {
                history.drain(0..len - 100);
            }

            let mut last_adjustment = self.last_adjustment.write().await;
            *last_adjustment = now;

            info!(
                "Bitrate adjusted for stream {}: {} -> {} kbps (quality: {:?})",
                self.stream_id, old_bitrate, config.current_bitrate, quality
            );

            Some(event)
        } else {
            None
        }
    }

    /// 获取当前码率配置
    pub async fn get_config(&self) -> BitrateConfig {
        self.config.read().await.clone()
    }

    /// 获取网络质量评估
    pub async fn get_network_quality(&self) -> NetworkQuality {
        self.evaluate_network_quality().await
    }

    /// 获取调整历史
    pub async fn get_adjustment_history(&self) -> Vec<BitrateAdjustmentEvent> {
        self.adjustment_history.read().await.clone()
    }

    /// 手动设置码率
    pub async fn set_bitrate(&self, bitrate: u32) {
        let mut config = self.config.write().await;
        config.current_bitrate = bitrate.max(config.min_bitrate).min(config.max_bitrate);

        info!(
            "Manual bitrate set for stream {}: {} kbps",
            self.stream_id, bitrate
        );
    }

    /// 获取统计信息
    pub async fn get_stats(&self) -> AdaptiveBitrateStats {
        let config = self.config.read().await;
        let history = self.adjustment_history.read().await;
        let quality = self.evaluate_network_quality().await;

        AdaptiveBitrateStats {
            stream_id: self.stream_id.clone(),
            current_bitrate: config.current_bitrate,
            min_bitrate: config.min_bitrate,
            max_bitrate: config.max_bitrate,
            current_framerate: config.current_framerate,
            current_gop: config.current_gop,
            network_quality: quality,
            total_adjustments: history.len(),
            last_adjustment_reason: history.last().map(|e| e.reason.clone()),
        }
    }
}

/// 动态码率统计信息
#[derive(Debug, Clone, Serialize)]
pub struct AdaptiveBitrateStats {
    /// 流ID
    pub stream_id: String,
    /// 当前码率
    pub current_bitrate: u32,
    /// 最小码率
    pub min_bitrate: u32,
    /// 最大码率
    pub max_bitrate: u32,
    /// 当前帧率
    pub current_framerate: u8,
    /// 当前GOP
    pub current_gop: u16,
    /// 网络质量
    pub network_quality: NetworkQuality,
    /// 总调整次数
    pub total_adjustments: usize,
    /// 最后调整原因
    pub last_adjustment_reason: Option<String>,
}

/// 创建动态码率调整器（便捷函数）
pub fn create_bitrate_controller(stream_id: String) -> Arc<AdaptiveBitrateController> {
    Arc::new(AdaptiveBitrateController::new(
        stream_id,
        Duration::from_secs(30), // 最小30秒调整一次
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_quality_bitrate() {
        assert_eq!(NetworkQuality::Excellent.recommended_bitrate(), 4096);
        assert_eq!(NetworkQuality::Good.recommended_bitrate(), 2048);
        assert_eq!(NetworkQuality::Fair.recommended_bitrate(), 1024);
        assert_eq!(NetworkQuality::Poor.recommended_bitrate(), 512);
        assert_eq!(NetworkQuality::VeryPoor.recommended_bitrate(), 256);
    }

    #[tokio::test]
    async fn test_bitrate_controller_creation() {
        let controller =
            AdaptiveBitrateController::new("test_stream".to_string(), Duration::from_secs(10));

        let config = controller.get_config().await;
        assert_eq!(config.current_bitrate, 2048);
    }
}
