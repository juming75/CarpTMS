//! / 流协议路由器
// 统一接口,自动选择转换器,格式转换性能优化

use crate::video::{VideoFrame, VideoCodec, StreamType};
use crate::video::transcoders::{
    StreamTranscoder, TranscodeConfig, StreamOutput, TranscodeError,
    FlvTranscoder, HlsTranscoder, RtmpTranscoder,
    FrameBufferPool,
};
use async_trait::async_trait;
use log::{debug, info, warn, error};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock, Mutex};
use tokio::task::JoinHandle;

/// 路由器配置
#[derive(Debug, Clone)]
pub struct RouterConfig {
    /// 默认输出格式
    pub default_output: StreamType,
    /// 是否启用多格式输出
    pub enable_multi_output: bool,
    /// 性能优化模式
    pub performance_mode: PerformanceMode,
}

impl Default for RouterConfig {
    fn default() -> Self {
        Self {
            default_output: StreamType::Hls,
            enable_multi_output: false,
            performance_mode: PerformanceMode::Balanced,
        }
    }
}

/// 性能模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PerformanceMode {
    /// 低延迟优先
    LowLatency,
    /// 高吞吐量优先
    HighThroughput,
    /// 均衡模式
    Balanced,
    /// 低功耗
    LowPower,
}

/// 转换器实例
struct TranscoderInstance {
    /// 转换器
    transcoder: Box<dyn StreamTranscoder>,
    /// 是否启用
    enabled: bool,
    /// 输出通道接收端(如果启动)
    output_rx: Option<mpsc::Receiver<StreamOutput>>,
    /// 任务句柄
    task_handle: Option<JoinHandle<()>>,
}

impl TranscoderInstance {
    fn new(transcoder: Box<dyn StreamTranscoder>) -> Self {
        Self {
            transcoder,
            enabled: false,
            output_rx: None,
            task_handle: None,
        }
    }
}

/// 流协议路由器
pub struct StreamRouter {
    /// 路由器配置
    config: Arc<RwLock<RouterConfig>>,
    /// 转换器实例
    transcoders: Arc<RwLock<HashMap<StreamType, TranscoderInstance>>>,
    /// 全局输入通道
    input_tx: Arc<Mutex<Option<mpsc::Sender<VideoFrame>>>>,
    /// 运行状态
    running: Arc<RwLock<bool>>,
    /// 任务句柄
    router_task: Arc<Mutex<Option<JoinHandle<()>>>>,
    /// 缓冲区池(共享)
    buffer_pool: Arc<FrameBufferPool>,
}

impl StreamRouter {
    /// 创建新的流协议路由器
    pub fn new(config: RouterConfig) -> Self {
        // 创建缓冲区池
        let buffer_pool = Arc::new(FrameBufferPool::new(2 * 1024 * 1024, 50)); // 2MB, 50 个缓冲区

        // 初始化转换器
        let mut transcoders_map = HashMap::new();

        // FLV 转换器
        let flv_config = TranscodeConfig::default();
        let flv_transcoder = Box::new(FlvTranscoder::new(flv_config));
        transcoders_map.insert(StreamType::HttpFlv, TranscoderInstance::new(flv_transcoder));

        // HLS 转换器
        let hls_config = TranscodeConfig::default();
        let hls_transcoder = Box::new(HlsTranscoder::new(hls_config));
        transcoders_map.insert(StreamType::Hls, TranscoderInstance::new(hls_transcoder));

        // RTMP 转换器(简化版,暂不实现完整功能)
        // let rtmp_config = TranscodeConfig::default();
        // let rtmp_transcoder = Box::new(RtmpTranscoder::new(rtmp_config));
        // transcoders_map.insert(StreamType::Rtmp, TranscoderInstance::new(rtmp_transcoder));

        Self {
            config: Arc::new(RwLock::new(config)),
            transcoders: Arc::new(RwLock::new(transcoders_map)),
            input_tx: Arc::new(Mutex::new(None)),
            running: Arc::new(RwLock::new(false)),
            router_task: Arc::new(Mutex::new(None)),
            buffer_pool,
        }
    }

    /// 添加转换器
    pub async fn add_transcoder(&self, stream_type: StreamType, transcoder: Box<dyn StreamTranscoder>) {
        let mut transcoders = self.transcoders.write().await;
        transcoders.insert(stream_type, TranscoderInstance::new(transcoder));
        info!("Added transcoder for stream type: {:?}", stream_type);
    }

    /// 移除转换器
    pub async fn remove_transcoder(&self, stream_type: StreamType) {
        let mut transcoders = self.transcoders.write().await;
        if let Some(instance) = transcoders.remove(&stream_type) {
            // 停止转换器
            let _ = instance.transcoder.stop().await;
            info!("Removed transcoder for stream type: {:?}", stream_type);
        }
    }

    /// 启动指定转换器
    pub async fn start_transcoder(&self, stream_type: StreamType) -> Result<(), TranscodeError> {
        let mut transcoders = self.transcoders.write().await;

        if let Some(instance) = transcoders.get_mut(&stream_type) {
            if !instance.enabled {
                // 获取输出通道
                let rx = instance.transcoder.get_output_channel().await
                    .ok_or_else(|| TranscodeError::Internal("Failed to get output channel".to_string()))?;

                instance.output_rx = Some(rx);

                // 启动转换器
                instance.transcoder.start().await?;
                instance.enabled = true;

                info!("Started transcoder: {:?}", stream_type);
            }
            Ok(())
        } else {
            Err(TranscodeError::Internal(format!("No transcoder found for {:?}", stream_type)))
        }
    }

    /// 停止指定转换器
    pub async fn stop_transcoder(&self, stream_type: StreamType) -> Result<(), TranscodeError> {
        let mut transcoders = self.transcoders.write().await;

        if let Some(instance) = transcoders.get_mut(&stream_type) {
            if instance.enabled {
                instance.transcoder.stop().await?;
                instance.enabled = false;
                instance.output_rx = None;

                info!("Stopped transcoder: {:?}", stream_type);
            }
            Ok(())
        } else {
            Err(TranscodeError::Internal(format!("No transcoder found for {:?}", stream_type)))
        }
    }

    /// 根据客户端请求自动选择最佳转换器
    pub async fn select_transcoder(&self, request_format: Option<StreamType>) -> Result<StreamType, TranscodeError> {
        let config = self.config.read().await;

        // 如果请求了特定格式,检查是否可用
        if let Some(req_format) = request_format {
            let transcoders = self.transcoders.read().await;
            if transcoders.contains_key(&req_format) {
                return Ok(req_format);
            }
        }

        // 否则根据性能模式选择默认格式
        match config.performance_mode {
            PerformanceMode::LowLatency => Ok(StreamType::HttpFlv), // FLV 延迟最低
            PerformanceMode::HighThroughput => Ok(StreamType::Hls), // HLS 兼容性最好
            PerformanceMode::Balanced => Ok(config.default_output),
            PerformanceMode::LowPower => Ok(StreamType::Hls),
        }
    }

    /// 启动路由器
    pub async fn start(&self) -> Result<(), TranscodeError> {
        let running = *self.running.read().await;
        if running {
            return Err(TranscodeError::AlreadyRunning);
        }

        // 创建输入通道
        let (input_tx, mut input_rx) = mpsc::channel(1000);
        *self.input_tx.lock().await = Some(input_tx);

        // 转发任务
        let transcoders = self.transcoders.clone();
        let config = self.config.clone();
        let running_flag = self.running.clone();

        let handle = tokio::spawn(async move {
            info!("Stream router forwarding task started");

            while let Some(frame) = input_rx.recv().await {
                // 检查是否正在运行
                if !*running_flag.read().await {
                    break;
                }

                // 读取配置
                let cfg = config.read().await;
                let enable_multi = cfg.enable_multi_output;
                let default_output = cfg.default_output;
                drop(cfg);

                let transcoders_map = transcoders.read().await;

                if enable_multi {
                    // 多格式输出:发送给所有启用的转换器
                    for (stream_type, instance) in transcoders_map.iter() {
                        if instance.enabled {
                            match instance.transcoder.process_frame(frame.clone()).await {
                                Ok(_outputs) => {
                                    // 输出已由转换器内部处理
                                }
                                Err(e) => {
                                    error!("Transcoder {:?} error: {}", stream_type, e);
                                }
                            }
                        }
                    }
                } else {
                    // 单格式输出:只发送给默认转换器
                    if let Some(instance) = transcoders_map.get(&default_output) {
                        if instance.enabled {
                            match instance.transcoder.process_frame(frame.clone()).await {
                                Ok(_outputs) => {
                                    // 输出已由转换器内部处理
                                }
                                Err(e) => {
                                    error!("Transcoder {:?} error: {}", default_output, e);
                                }
                            }
                        }
                    }
                }
            }

            info!("Stream router forwarding task stopped");
        });

        *self.router_task.lock().await = Some(handle);
        *self.running.write().await = true;

        // 启动默认转换器
        self.start_transcoder(config.read().await.default_output).await?;

        info!("Stream router started");
        Ok(())
    }

    /// 停止路由器
    pub async fn stop(&self) -> Result<(), TranscodeError> {
        *self.running.write().await = false;

        // 停止所有转换器
        let transcoders = self.transcoders.read().await;
        for (stream_type, instance) in transcoders.iter() {
            if instance.enabled {
                if let Err(e) = instance.transcoder.stop().await {
                    error!("Failed to stop transcoder {:?}: {}", stream_type, e);
                }
            }
        }

        // 取消路由任务
        let mut handle = self.router_task.lock().await;
        if let Some(h) = handle.take() {
            h.abort();
        }

        info!("Stream router stopped");
        Ok(())
    }

    /// 推送视频帧到路由器
    pub async fn push_frame(&self, frame: VideoFrame) -> Result<(), TranscodeError> {
        let tx = self.input_tx.lock().await;
        if let Some(ref tx) = *tx {
            tx.send(frame)
                .await
                .map_err(|e| TranscodeError::Internal(e.to_string()))?;
            Ok(())
        } else {
            Err(TranscodeError::NotInitialized)
        }
    }

    /// 获取指定格式的输出通道
    pub async fn get_output(&self, stream_type: StreamType) -> Option<mpsc::Receiver<StreamOutput>> {
        let transcoders = self.transcoders.read().await;
        transcoders.get(&stream_type)?.output_rx.clone()
    }

    /// 获取路由器状态
    pub async fn status(&self) -> RouterStatus {
        let config = self.config.read().await;
        let transcoders = self.transcoders.read().await;
        let running = *self.running.read().await;

        let mut transcoder_statuses = HashMap::new();
        for (stream_type, instance) in transcoders.iter() {
            transcoder_statuses.insert(
                *stream_type,
                TranscoderState {
                    enabled: instance.enabled,
                    status: instance.transcoder.status().await,
                    output_available: instance.output_rx.is_some(),
                }
            );
        }

        RouterStatus {
            running,
            default_output: config.default_output,
            enable_multi_output: config.enable_multi_output,
            performance_mode: config.performance_mode,
            transcoders: transcoder_statuses,
        }
    }

    /// 设置路由器配置
    pub async fn set_config(&self, config: RouterConfig) {
        *self.config.write().await = config;
    }
}

/// 转换器状态
#[derive(Debug, Clone)]
pub struct TranscoderState {
    pub enabled: bool,
    pub status: crate::video::transcoders::TranscoderStatus,
    pub output_available: bool,
}

/// 路由器状态
#[derive(Debug, Clone)]
pub struct RouterStatus {
    pub running: bool,
    pub default_output: StreamType,
    pub enable_multi_output: bool,
    pub performance_mode: PerformanceMode,
    pub transcoders: HashMap<StreamType, TranscoderState>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_router_creation() {
        let config = RouterConfig::default();
        let router = StreamRouter::new(config);

        let status = router.status().await;
        assert!(!status.running);
        assert_eq!(status.default_output, StreamType::Hls);
    }

    #[tokio::test]
    async fn test_router_start_stop() {
        let config = RouterConfig::default();
        let router = StreamRouter::new(config);

        router.start().await.unwrap();
        let status = router.status().await;
        assert!(status.running);

        router.stop().await.unwrap();
        let status = router.status().await;
        assert!(!status.running);
    }

    #[tokio::test]
    async fn test_transcoder_selection() {
        let config = RouterConfig::default();
        let router = StreamRouter::new(config);

        // 测试选择
        let selected = router.select_transcoder(None).await.unwrap();
        assert_eq!(selected, StreamType::Hls);

        let selected = router.select_transcoder(Some(StreamType::HttpFlv)).await.unwrap();
        assert_eq!(selected, StreamType::HttpFlv);
    }
}






