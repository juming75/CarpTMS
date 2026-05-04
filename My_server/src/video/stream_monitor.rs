//! JT1078流媒体平台监控看板
//!
//! 提供流媒体平台的实时监控指标，包括：
//! - 设备在线率（≥99.5%）
//! - 端到端延迟（P95≤3秒）
//! - 推流成功率（≥99.9%）
//! - 服务器负载（CPU≤70%）
//! - 带宽使用情况
//! - 流媒体质量指标

use log::debug;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// 监控指标集合
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringMetrics {
    /// 设备在线率（百分比）
    pub device_online_rate: f64,
    /// 端到端延迟P50（毫秒）
    pub latency_p50: f64,
    /// 端到端延迟P95（毫秒）
    pub latency_p95: f64,
    /// 端到端延迟P99（毫秒）
    pub latency_p99: f64,
    /// 推流成功率（百分比）
    pub stream_success_rate: f64,
    /// 服务器CPU使用率（百分比）
    pub cpu_usage: f64,
    /// 服务器内存使用率（百分比）
    pub memory_usage: f64,
    /// 网络带宽使用（Mbps）
    pub bandwidth_usage: f64,
    /// 活跃流数量
    pub active_streams: u64,
    /// 总设备数
    pub total_devices: u64,
    /// 在线设备数
    pub online_devices: u64,
    /// 总帧率（fps）
    pub total_framerate: u64,
    /// 错误率（百分比）
    pub error_rate: f64,
}

impl Default for MonitoringMetrics {
    fn default() -> Self {
        Self {
            device_online_rate: 0.0,
            latency_p50: 0.0,
            latency_p95: 0.0,
            latency_p99: 0.0,
            stream_success_rate: 100.0,
            cpu_usage: 0.0,
            memory_usage: 0.0,
            bandwidth_usage: 0.0,
            active_streams: 0,
            total_devices: 0,
            online_devices: 0,
            total_framerate: 0,
            error_rate: 0.0,
        }
    }
}

/// 延迟采样点
#[derive(Debug, Clone)]
pub struct LatencySample {
    /// 时间戳
    pub timestamp: Instant,
    /// 延迟值（毫秒）
    pub latency_ms: f64,
    /// 流ID
    pub stream_id: String,
}

/// 推流统计
#[derive(Debug, Clone)]
pub struct StreamStats {
    /// 推流开始时间
    pub start_time: Instant,
    /// 成功帧数
    pub success_frames: u64,
    /// 失败帧数
    pub failed_frames: u64,
    /// 最后活跃时间
    pub last_active: Instant,
}

/// 流媒体监控器
/// 收集和分析流媒体平台的各项监控指标
#[allow(dead_code)]
pub struct StreamMediaMonitor {
    /// 延迟采样数据
    latency_samples: Arc<RwLock<Vec<LatencySample>>>,
    /// 推流统计
    stream_stats: Arc<RwLock<HashMap<String, StreamStats>>>,
    /// 设备统计
    device_stats: Arc<RwLock<DeviceStats>>,
    /// 服务器资源统计
    server_stats: Arc<RwLock<ServerResourceStats>>,
    /// 监控数据更新间隔
    update_interval: Duration,
    /// 上次更新监控指标的时间
    last_update: Arc<RwLock<Instant>>,
    /// 缓存的监控指标
    cached_metrics: Arc<RwLock<MonitoringMetrics>>,
    /// 视频流管理器引用 (P1: 用于获取帧率)
    video_manager: Option<Arc<super::VideoStreamManager>>,
}

/// 设备统计
#[derive(Debug, Clone)]
pub struct DeviceStats {
    /// 总设备数
    pub total_devices: u64,
    /// 在线设备数
    pub online_devices: u64,
    /// 离线设备数
    pub offline_devices: u64,
    /// 最后更新时间
    pub last_update: Instant,
}

impl Default for DeviceStats {
    fn default() -> Self {
        Self {
            total_devices: 0,
            online_devices: 0,
            offline_devices: 0,
            last_update: Instant::now(),
        }
    }
}

/// 服务器资源统计
#[derive(Debug, Clone)]
pub struct ServerResourceStats {
    /// CPU使用率（百分比）
    pub cpu_usage: f64,
    /// 内存使用率（百分比）
    pub memory_usage: f64,
    /// 网络入站带宽（Mbps）
    pub inbound_bandwidth: f64,
    /// 网络出站带宽（Mbps）
    pub outbound_bandwidth: f64,
    /// 最后更新时间
    pub last_update: Instant,
}

impl Default for ServerResourceStats {
    fn default() -> Self {
        Self {
            cpu_usage: 0.0,
            memory_usage: 0.0,
            inbound_bandwidth: 0.0,
            outbound_bandwidth: 0.0,
            last_update: Instant::now(),
        }
    }
}

impl StreamMediaMonitor {
    /// 创建新的流媒体监控器
    pub fn new(update_interval: Duration) -> Self {
        let now = Instant::now();
        Self {
            latency_samples: Arc::new(RwLock::new(Vec::new())),
            stream_stats: Arc::new(RwLock::new(HashMap::new())),
            device_stats: Arc::new(RwLock::new(DeviceStats::default())),
            server_stats: Arc::new(RwLock::new(ServerResourceStats::default())),
            update_interval,
            last_update: Arc::new(RwLock::new(now)),
            cached_metrics: Arc::new(RwLock::new(MonitoringMetrics::default())),
            video_manager: None, // P1: 默认无视频流管理器
        }
    }

    /// 创建流媒体监控器并绑定视频流管理器 (P1: 用于获取帧率)
    pub fn with_video_manager(
        update_interval: Duration,
        video_manager: Arc<super::VideoStreamManager>,
    ) -> Self {
        let now = Instant::now();
        Self {
            latency_samples: Arc::new(RwLock::new(Vec::new())),
            stream_stats: Arc::new(RwLock::new(HashMap::new())),
            device_stats: Arc::new(RwLock::new(DeviceStats::default())),
            server_stats: Arc::new(RwLock::new(ServerResourceStats::default())),
            update_interval,
            last_update: Arc::new(RwLock::new(now)),
            cached_metrics: Arc::new(RwLock::new(MonitoringMetrics::default())),
            video_manager: Some(video_manager),
        }
    }

    /// 设置视频流管理器 (P1: 用于获取帧率)
    pub fn set_video_manager(&mut self, video_manager: Arc<super::VideoStreamManager>) {
        self.video_manager = Some(video_manager);
    }

    /// 记录延迟采样
    pub async fn record_latency(&self, stream_id: &str, latency_ms: f64) {
        let sample = LatencySample {
            timestamp: Instant::now(),
            latency_ms,
            stream_id: stream_id.to_string(),
        };

        let mut samples = self.latency_samples.write().await;
        samples.push(sample);

        // 保留最近5分钟的采样数据
        let cutoff = Instant::now() - Duration::from_secs(300);
        samples.retain(|s| s.timestamp > cutoff);

        debug!(
            "Recorded latency for stream {}: {}ms",
            stream_id, latency_ms
        );
    }

    /// 记录推流成功
    pub async fn record_stream_success(&self, stream_id: &str) {
        let mut stats = self.stream_stats.write().await;
        let entry = stats
            .entry(stream_id.to_string())
            .or_insert_with(|| StreamStats {
                start_time: Instant::now(),
                success_frames: 0,
                failed_frames: 0,
                last_active: Instant::now(),
            });
        entry.success_frames += 1;
        entry.last_active = Instant::now();
    }

    /// 记录推流失败
    pub async fn record_stream_failure(&self, stream_id: &str) {
        let mut stats = self.stream_stats.write().await;
        let entry = stats
            .entry(stream_id.to_string())
            .or_insert_with(|| StreamStats {
                start_time: Instant::now(),
                success_frames: 0,
                failed_frames: 0,
                last_active: Instant::now(),
            });
        entry.failed_frames += 1;
        entry.last_active = Instant::now();
    }

    /// 更新设备统计
    pub async fn update_device_stats(&self, total: u64, online: u64) {
        let mut stats = self.device_stats.write().await;
        stats.total_devices = total;
        stats.online_devices = online;
        stats.offline_devices = total.saturating_sub(online);
        stats.last_update = Instant::now();
    }

    /// 更新服务器资源统计
    pub async fn update_server_stats(
        &self,
        cpu_usage: f64,
        memory_usage: f64,
        inbound_bandwidth: f64,
        outbound_bandwidth: f64,
    ) {
        let mut stats = self.server_stats.write().await;
        stats.cpu_usage = cpu_usage;
        stats.memory_usage = memory_usage;
        stats.inbound_bandwidth = inbound_bandwidth;
        stats.outbound_bandwidth = outbound_bandwidth;
        stats.last_update = Instant::now();
    }

    /// 计算延迟百分位数
    async fn calculate_latency_percentile(&self, percentile: f64) -> f64 {
        let samples = self.latency_samples.read().await;
        if samples.is_empty() {
            return 0.0;
        }

        let mut latencies: Vec<f64> = samples.iter().map(|s| s.latency_ms).collect();
        latencies.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let index = ((percentile / 100.0) * latencies.len() as f64) as usize;
        let index = index.min(latencies.len() - 1);
        latencies[index]
    }

    /// 计算推流成功率
    async fn calculate_stream_success_rate(&self) -> f64 {
        let stats = self.stream_stats.read().await;
        let mut total_success = 0u64;
        let mut total_failed = 0u64;

        for stream_stat in stats.values() {
            total_success += stream_stat.success_frames;
            total_failed += stream_stat.failed_frames;
        }

        let total = total_success + total_failed;
        if total == 0 {
            return 100.0;
        }

        (total_success as f64 / total as f64) * 100.0
    }

    /// 计算活跃流数量
    async fn calculate_active_streams(&self) -> u64 {
        let stats = self.stream_stats.read().await;
        let cutoff = Instant::now() - Duration::from_secs(60);
        stats.values().filter(|s| s.last_active > cutoff).count() as u64
    }

    /// 更新监控指标
    pub async fn update_metrics(&self) {
        let device_stats = self.device_stats.read().await;
        let server_stats = self.server_stats.read().await;

        let device_online_rate = if device_stats.total_devices > 0 {
            (device_stats.online_devices as f64 / device_stats.total_devices as f64) * 100.0
        } else {
            0.0
        };

        let latency_p50 = self.calculate_latency_percentile(50.0).await;
        let latency_p95 = self.calculate_latency_percentile(95.0).await;
        let latency_p99 = self.calculate_latency_percentile(99.0).await;
        let stream_success_rate = self.calculate_stream_success_rate().await;
        let active_streams = self.calculate_active_streams().await;

        let bandwidth_usage = server_stats.inbound_bandwidth + server_stats.outbound_bandwidth;

        // P1: 从视频流管理器获取总帧率
        let total_framerate = if let Some(ref manager) = self.video_manager {
            manager.get_total_framerate().await
        } else {
            0
        };

        let metrics = MonitoringMetrics {
            device_online_rate,
            latency_p50,
            latency_p95,
            latency_p99,
            stream_success_rate,
            cpu_usage: server_stats.cpu_usage,
            memory_usage: server_stats.memory_usage,
            bandwidth_usage,
            active_streams,
            total_devices: device_stats.total_devices,
            online_devices: device_stats.online_devices,
            total_framerate, // P1: 从视频流管理器获取
            error_rate: 100.0 - stream_success_rate,
        };

        let mut cached = self.cached_metrics.write().await;
        *cached = metrics;

        let mut last_update = self.last_update.write().await;
        *last_update = Instant::now();

        debug!("Monitoring metrics updated");
    }

    /// 获取当前监控指标
    pub async fn get_metrics(&self) -> MonitoringMetrics {
        self.cached_metrics.read().await.clone()
    }

    /// 获取告警信息
    pub async fn get_alerts(&self) -> Vec<Alert> {
        let metrics = self.get_metrics().await;
        let mut alerts = Vec::new();

        // 检查设备在线率
        if metrics.device_online_rate < 99.5 {
            alerts.push(Alert {
                level: AlertLevel::Warning,
                metric: "device_online_rate".to_string(),
                message: format!(
                    "设备在线率过低: {:.2}% (阈值: 99.5%)",
                    metrics.device_online_rate
                ),
                value: metrics.device_online_rate,
                threshold: 99.5,
            });
        }

        // 检查延迟
        if metrics.latency_p95 > 3000.0 {
            alerts.push(Alert {
                level: AlertLevel::Critical,
                metric: "latency_p95".to_string(),
                message: format!("P95延迟过高: {:.0}ms (阈值: 3000ms)", metrics.latency_p95),
                value: metrics.latency_p95,
                threshold: 3000.0,
            });
        }

        // 检查推流成功率
        if metrics.stream_success_rate < 99.9 {
            alerts.push(Alert {
                level: AlertLevel::Warning,
                metric: "stream_success_rate".to_string(),
                message: format!(
                    "推流成功率过低: {:.2}% (阈值: 99.9%)",
                    metrics.stream_success_rate
                ),
                value: metrics.stream_success_rate,
                threshold: 99.9,
            });
        }

        // 检查CPU使用率
        if metrics.cpu_usage > 70.0 {
            alerts.push(Alert {
                level: AlertLevel::Warning,
                metric: "cpu_usage".to_string(),
                message: format!("CPU使用率过高: {:.2}% (阈值: 70%)", metrics.cpu_usage),
                value: metrics.cpu_usage,
                threshold: 70.0,
            });
        }

        alerts
    }

    /// 清理过期数据
    pub async fn cleanup_expired_data(&self) {
        let cutoff = Instant::now() - Duration::from_secs(300);

        // 清理过期延迟采样
        {
            let mut samples = self.latency_samples.write().await;
            samples.retain(|s| s.timestamp > cutoff);
        }

        // 清理过期的推流统计
        {
            let mut stats = self.stream_stats.write().await;
            stats.retain(|_, s| s.last_active > cutoff);
        }

        debug!("Cleaned up expired monitoring data");
    }
}

/// 告警信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    /// 告警级别
    pub level: AlertLevel,
    /// 指标名称
    pub metric: String,
    /// 告警消息
    pub message: String,
    /// 当前值
    pub value: f64,
    /// 阈值
    pub threshold: f64,
}

/// 告警级别
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertLevel {
    /// 信息
    Info,
    /// 警告
    Warning,
    /// 严重
    Critical,
}

/// 创建流媒体监控器（便捷函数）
pub fn create_stream_monitor() -> Arc<StreamMediaMonitor> {
    Arc::new(StreamMediaMonitor::new(Duration::from_secs(10)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_monitor_creation() {
        let monitor = StreamMediaMonitor::new(Duration::from_secs(10));
        assert!(monitor.get_metrics().await.device_online_rate == 0.0);
    }

    #[tokio::test]
    async fn test_latency_recording() {
        let monitor = StreamMediaMonitor::new(Duration::from_secs(10));
        monitor.record_latency("stream1", 100.0).await;
        monitor.record_latency("stream1", 200.0).await;
        monitor.record_latency("stream2", 150.0).await;

        let p50 = monitor.calculate_latency_percentile(50.0).await;
        assert!(p50 > 0.0);
    }
}
