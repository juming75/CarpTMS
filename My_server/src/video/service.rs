//! / 视频服务启动模块
// 负责初始化和启动所有视频相关服务

use super::{
    config::VideoConfig, gb28181_stream::Gb28181StreamHandler, jt1078_stream::Jt1078StreamHandler,
    recording::RecordingManager, video_manager::VideoStreamManager, StreamType, VideoFrame,
    VideoFrameType,
};
use crate::protocols::jt1078::VideoDataType;
use log::{error, info, warn};
use std::sync::Arc;
use tokio::sync::RwLock;

/// 视频服务实例
///
/// 统一管理所有视频相关功能,包括:
/// - JT1078协议视频流处理
/// - GB28181协议视频流处理
/// - 视频流转码(HLS、FLV、RTMP等)
/// - 视频录制和回放
/// - WebSocket实时视频推送
///
/// # 功能特性
/// - 支持多种视频协议(JT1078、GB28181、RTMP、RTSP)
/// - 自动视频流转码
/// - 多格式输出(HLS、HTTP-FLV、RTMP)
/// - 实时WebSocket推送
/// - 云端录制和回放
/// - 高并发支持
///
/// # 性能优化
/// - 异步非阻塞设计
/// - 连接池管理
/// - 视频帧批处理
/// - 内存高效管理
///
/// # 示例
/// ```ignore
/// let service = VideoService::from_env()?;
/// service.start().await?;
///
/// // 创建视频流
/// let stream_id = service.stream_manager.create_stream("device001", 1, StreamType::JT1078).await?;
///
/// // 停止服务
/// service.stop().await?;
/// ```
#[derive(Clone)]
pub struct VideoService {
    /// 配置
    pub config: VideoConfig,
    /// 视频流管理器
    pub stream_manager: Arc<VideoStreamManager>,
    /// JT1078流处理器
    pub jt1078_handler: Arc<Jt1078StreamHandler>,
    /// GB28181流处理器
    pub gb28181_handler: Arc<Gb28181StreamHandler>,
    /// 录像管理器
    pub recording_manager: Arc<RecordingManager>,
    /// 服务运行状态
    running: Arc<RwLock<bool>>,
}

impl VideoService {
    /// 创建新的视频服务
    ///
    /// # 参数
    /// - `config`: 视频服务配置,包含所有必要的参数
    ///
    /// # 返回
    /// 成功返回初始化好的`VideoService`实例
    ///
    /// # 错误
    /// - 配置验证失败
    /// - 组件初始化失败
    ///
    /// # 示例
    /// ```ignore
    /// let config = VideoConfig {
    ///     jt1078_max_connections: 1000,
    ///     storage_path: "./recordings".to_string(),
    ///     ..Default::default()
    /// };
    /// let service = VideoService::new(config)?;
    /// ```
    pub fn new(config: VideoConfig) -> Result<Self, String> {
        // 验证配置
        config.validate()?;

        // 创建视频流管理器
        let stream_manager = Arc::new(VideoStreamManager::new());

        // 创建JT1078处理器
        let jt1078_handler = Arc::new(Jt1078StreamHandler::new(
            config.jt1078.max_connections,
            config.jt1078.buffer_size,
        ));

        // 创建GB28181处理器
        let gb28181_handler = Arc::new(Gb28181StreamHandler::new(
            config.gb28181.sip_port,
            config.gb28181.rtp_port_start,
            config.gb28181.rtp_port_end,
        ));

        // 创建录像管理器
        let recording_config = super::recording::RecordingConfig {
            storage_root: config.storage_path.clone(),
            ..Default::default()
        };
        let recording_manager = Arc::new(RecordingManager::new(recording_config));

        Ok(Self {
            config,
            stream_manager,
            jt1078_handler,
            gb28181_handler,
            recording_manager,
            running: Arc::new(RwLock::new(false)),
        })
    }

    /// 从环境变量创建视频服务
    ///
    /// 从以下环境变量读取配置:
    /// - `JT1078_MAX_CONNECTIONS`: 最大JT1078连接数
    /// - `JT1078_BUFFER_SIZE`: JT1078缓冲区大小
    /// - `GB28181_SIP_PORT`: GB28181 SIP端口
    /// - `VIDEO_STORAGE_PATH`: 视频存储路径
    ///
    /// # 返回
    /// 成功返回初始化好的`VideoService`实例
    ///
    /// # 错误
    /// - 环境变量读取失败
    /// - 配置验证失败
    pub fn from_env() -> Result<Self, String> {
        let config = VideoConfig::from_env();
        Self::new(config)
    }

    /// 启动视频服务
    ///
    /// 启动所有视频相关服务,包括:
    /// - JT1078监听器
    /// - GB28181 SIP服务器
    /// - 录像管理器
    /// - 视频流转码器
    ///
    /// # 返回
    /// 成功返回`Ok(())`,服务开始运行
    ///
    /// # 错误
    /// - 服务已经在运行
    /// - 端口绑定失败
    /// - 组件启动失败
    ///
    /// # 示例
    /// ```ignore
    /// service.start().await?;
    /// log::info!("Video service started successfully");
    /// ```
    pub async fn start(&self) -> Result<(), String> {
        let mut running = self.running.write().await;
        if *running {
            return Err("Video service is already running".to_string());
        }

        info!("Starting video streaming service...");

        // 启动JT1078监听器
        if self.config.jt1078.enabled {
            self.start_jt1078_listener().await?;
        } else {
            info!("JT1078 is disabled");
        }

        // 启动GB28181 SIP服务器
        if self.config.gb28181.enabled {
            self.start_gb28181_server().await?;
        } else {
            info!("GB28181 is disabled");
        }

        // 启动定时任务
        self.start_background_tasks().await;

        *running = true;
        info!("Video service started successfully");

        Ok(())
    }

    /// 停止视频服务
    pub async fn stop(&self) -> Result<(), String> {
        let mut running = self.running.write().await;
        if !*running {
            return Err("Video service is not running".to_string());
        }

        info!("Stopping video streaming service...");

        // 停止所有视频流
        self.stream_manager.stop_all_streams().await;

        *running = false;
        info!("Video service stopped");

        Ok(())
    }

    /// 检查服务是否在运行
    pub async fn is_running(&self) -> bool {
        *self.running.read().await
    }

    /// 启动JT1078 UDP监听器
    async fn start_jt1078_listener(&self) -> Result<(), String> {
        let port = self.config.jt1078.listen_port;
        info!("Starting JT1078 UDP listener on port {}", port);

        let handler = self.jt1078_handler.clone();
        let stream_manager = self.stream_manager.clone();
        let max_connections = self.config.jt1078.max_connections;
        let buffer_size = self.config.jt1078.buffer_size;

        tokio::spawn(async move {
            use tokio::net::UdpSocket;

            // 绑定UDP socket
            let socket = match UdpSocket::bind(format!("0.0.0.0:{}", port)).await {
                Ok(s) => {
                    info!("JT1078 UDP socket bound successfully on port {}", port);
                    s
                }
                Err(e) => {
                    error!("Failed to bind JT1078 UDP socket: {}", e);
                    return;
                }
            };

            let mut buf = vec![0u8; buffer_size];
            let mut connection_count = 0;

            loop {
                match socket.recv_from(&mut buf).await {
                    Ok((n, src)) => {
                        let data = buf[..n].to_vec();

                        // 处理JT1078数据包
                        if let Some(frame) = handler.process_frame(&data).await {
                            // 解析通道ID和数据类型
                            let channel_id = if data.len() >= 6 { data[5] & 0x1F } else { 0 };
                            let data_type = if data.len() >= 5 {
                                VideoDataType::from(data[4])
                            } else {
                                VideoDataType::Unknown
                            };

                            // 创建或获取流
                            let device_id = if data.len() >= 12 {
                                format!(
                                    "{:02X}{:02X}{:02X}{:02X}",
                                    data[8], data[9], data[10], data[11]
                                )
                            } else {
                                "unknown".to_string()
                            };
                            let stream_id = format!("jt1078_{}_ch{}", device_id, channel_id);

                            // 如果流不存在,创建新流
                            if !stream_manager.stream_exists(&stream_id).await {
                                if connection_count < max_connections {
                                    if let Err(e) = stream_manager
                                        .create_stream(
                                            device_id.clone(),
                                            channel_id,
                                            StreamType::JT1078,
                                        )
                                        .await
                                    {
                                        warn!("Failed to create stream {}: {}", stream_id, e);
                                    } else {
                                        connection_count += 1;
                                        info!(
                                            "Created new JT1078 stream: {} (total: {})",
                                            stream_id, connection_count
                                        );
                                    }
                                } else {
                                    warn!("Max JT1078 connections reached, ignoring packet");
                                }
                            }

                            // 将帧转发到流
                            let frame_data = VideoFrame {
                                frame_type: match data_type {
                                    VideoDataType::IFrame => VideoFrameType::IFrame,
                                    VideoDataType::PFrame => VideoFrameType::PFrame,
                                    VideoDataType::BFrame => VideoFrameType::BFrame,
                                    VideoDataType::AudioFrame => VideoFrameType::AudioFrame,
                                    _ => VideoFrameType::PFrame,
                                },
                                timestamp: std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .map(|d| d.as_secs())
                                    .unwrap_or(0), // P4: 处理系统时间异常
                                data: bytes::Bytes::from(frame),
                                sequence: 0,
                            };

                            if let Err(e) = stream_manager
                                .distribute_frame(&stream_id, frame_data)
                                .await
                            {
                                warn!("Failed to distribute frame to {}: {}", stream_id, e);
                            }
                        } else {
                            warn!("Failed to parse JT1078 frame from {}", src);
                        }
                    }
                    Err(e) => {
                        error!("JT1078 UDP receive error: {}", e);
                    }
                }
            }
        });

        Ok(())
    }

    /// 启动GB28181 SIP服务器
    async fn start_gb28181_server(&self) -> Result<(), String> {
        let port = self.config.gb28181.sip_port;
        info!("Starting GB28181 SIP server on port {}", port);

        // TODO: 实现GB28181 SIP服务器
        // 这里需要实现完整的SIP协议栈
        // 包括REGISTER、INVITE、ACK、BYE等消息处理

        warn!("GB28181 SIP server not yet implemented (requires full SIP stack)");

        Ok(())
    }

    /// 启动后台任务
    async fn start_background_tasks(&self) {
        let stream_manager = self.stream_manager.clone();
        let interval = tokio::time::Duration::from_secs(60);

        // 清理过期流
        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(interval);
            loop {
                ticker.tick().await;
                // TODO: 清理长时间不活动的流
                stream_manager.cleanup_inactive_streams().await;
            }
        });

        // 输出统计信息
        let stream_manager2 = self.stream_manager.clone();
        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(tokio::time::Duration::from_secs(300));
            loop {
                ticker.tick().await;
                let stats = stream_manager2.get_statistics().await;
                info!("Video service statistics: {:?}", stats);
            }
        });
    }
}

/// 创建并启动视频服务(便捷函数)
pub async fn create_and_start_video_service() -> Result<Arc<VideoService>, String> {
    let service = Arc::new(VideoService::from_env()?);
    service.start().await?;
    Ok(service)
}
