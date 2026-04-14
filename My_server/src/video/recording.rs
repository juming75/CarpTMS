//! / 视频录像管理模块
// 支持视频流录像和历史视频回放

use super::{video_manager::StreamError, VideoFrame, VideoFrameType};
use chrono::{DateTime, Duration, Utc};
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

/// 录像状态
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RecordingState {
    /// 空闲
    Idle,
    /// 录像中
    Recording,
    /// 暂停
    Paused,
    /// 错误
    Error(String),
}

/// 录像配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingConfig {
    /// 录像存储根目录
    pub storage_root: String,
    /// 单个录像文件最大大小(MB)
    pub max_file_size_mb: u64,
    /// 磁盘使用阈值(%)
    pub disk_threshold_percent: u8,
    /// 自动循环录像
    pub auto_circular: bool,
    /// 录像保留天数
    pub retention_days: u32,
    /// 默认录像格式
    pub default_format: RecordingFormat,
}

impl Default for RecordingConfig {
    fn default() -> Self {
        Self {
            storage_root: "./recordings".to_string(),
            max_file_size_mb: 500,
            disk_threshold_percent: 80,
            auto_circular: true,
            retention_days: 7,
            default_format: RecordingFormat::Mp4,
        }
    }
}

/// 录像格式
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum RecordingFormat {
    /// MP4容器
    Mp4,
    /// FLV容器
    Flv,
    /// TS容器
    Ts,
    /// 原始H.264流
    H264Raw,
}

/// 录像元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingMetadata {
    /// 录像ID
    pub id: String,
    /// 流ID
    pub stream_id: String,
    /// 设备ID
    pub device_id: String,
    /// 通道ID
    pub channel_id: u8,
    /// 开始时间
    pub start_time: DateTime<Utc>,
    /// 结束时间
    pub end_time: Option<DateTime<Utc>>,
    /// 录像时长(秒)
    pub duration_seconds: u64,
    /// 文件大小(字节)
    pub file_size_bytes: u64,
    /// 文件路径
    pub file_path: String,
    /// 录像格式
    pub format: RecordingFormat,
    /// 总帧数
    pub total_frames: u64,
    /// 关键帧数
    pub key_frames: u64,
    /// 视频分辨率
    pub resolution: Option<String>,
}

/// 录像任务
#[derive(Debug)]
struct RecordingTask {
    /// 任务ID
    #[allow(dead_code)]
    id: String,
    /// 流ID
    stream_id: String,
    /// 录像元数据
    metadata: RecordingMetadata,
    /// 文件写入器
    writer: Option<BufWriter<fs::File>>,
    /// 当前文件大小(字节)
    current_size: u64,
    /// 当前帧数
    frame_count: u64,
    /// 关键帧数
    key_frame_count: u64,
}

/// 录像管理器
pub struct RecordingManager {
    /// 配置
    config: RecordingConfig,
    /// 活跃的录像任务
    active_recordings: Arc<Mutex<HashMap<String, RecordingTask>>>,
    /// 录像索引(所有录像的元数据)
    recording_index: Arc<RwLock<HashMap<String, RecordingMetadata>>>,
    /// 流到录像的映射
    stream_recordings: Arc<RwLock<HashMap<String, String>>>,
}

impl RecordingManager {
    /// 创建新的录像管理器
    pub fn new(config: RecordingConfig) -> Self {
        // 确保存储目录存在
        if let Err(e) = fs::create_dir_all(&config.storage_root) {
            error!("Failed to create recording storage directory: {}", e);
        }

        Self {
            config,
            active_recordings: Arc::new(Mutex::new(HashMap::new())),
            recording_index: Arc::new(RwLock::new(HashMap::new())),
            stream_recordings: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 创建默认配置的录像管理器
    pub fn with_default_config() -> Self {
        Self::new(RecordingConfig::default())
    }

    /// 开始录像
    pub async fn start_recording(
        &self,
        stream_id: String,
        device_id: String,
        channel_id: u8,
        format: Option<RecordingFormat>,
    ) -> Result<String, StreamError> {
        // 检查流是否已经在录像
        let stream_recordings = self.stream_recordings.read().await;
        if stream_recordings.contains_key(&stream_id) {
            drop(stream_recordings);
            return Err(StreamError::InternalError(format!(
                "Stream {} is already being recorded",
                stream_id
            )));
        }
        drop(stream_recordings);

        // 检查磁盘空间
        if !self.check_disk_space().await {
            return Err(StreamError::InternalError(
                "Insufficient disk space".to_string(),
            ));
        }

        // 生成录像ID和文件路径
        let recording_id = format!(
            "rec_{}_{}",
            Utc::now().timestamp(),
            &uuid::Uuid::new_v4().to_string()[..8]
        );

        let format = format.unwrap_or(self.config.default_format);
        let file_ext = match format {
            RecordingFormat::Mp4 => "mp4",
            RecordingFormat::Flv => "flv",
            RecordingFormat::Ts => "ts",
            RecordingFormat::H264Raw => "h264",
        };

        // 创建设备目录
        let device_dir = PathBuf::from(&self.config.storage_root).join(&device_id);
        if let Err(e) = fs::create_dir_all(&device_dir) {
            error!("Failed to create device directory: {}", e);
            return Err(StreamError::InternalError(e.to_string()));
        }

        // 创建日期目录
        let date_str = Utc::now().format("%Y-%m-%d").to_string();
        let date_dir = device_dir.join(&date_str);
        if let Err(e) = fs::create_dir_all(&date_dir) {
            error!("Failed to create date directory: {}", e);
            return Err(StreamError::InternalError(e.to_string()));
        }

        let file_path = date_dir.join(format!("{}.{}", recording_id, file_ext));

        // 创建文件
        let file = fs::File::create(&file_path).map_err(|e| {
            StreamError::InternalError(format!("Failed to create recording file: {}", e))
        })?;

        let writer = BufWriter::new(file);

        // 创建元数据
        let metadata = RecordingMetadata {
            id: recording_id.clone(),
            stream_id: stream_id.clone(),
            device_id: device_id.clone(),
            channel_id,
            start_time: Utc::now(),
            end_time: None,
            duration_seconds: 0,
            file_size_bytes: 0,
            file_path: file_path.to_string_lossy().to_string(),
            format,
            total_frames: 0,
            key_frames: 0,
            resolution: None,
        };

        // 创建录像任务
        let task = RecordingTask {
            id: recording_id.clone(),
            stream_id: stream_id.clone(),
            metadata: metadata.clone(),
            writer: Some(writer),
            current_size: 0,
            frame_count: 0,
            key_frame_count: 0,
        };

        // 添加到活跃录像
        let mut active_recordings = self.active_recordings.lock().await;
        active_recordings.insert(recording_id.clone(), task);
        drop(active_recordings);

        // 更新流到录像的映射
        let mut stream_recordings = self.stream_recordings.write().await;
        stream_recordings.insert(stream_id.clone(), recording_id.clone());
        drop(stream_recordings);

        // 添加到索引
        let mut recording_index = self.recording_index.write().await;
        recording_index.insert(recording_id.clone(), metadata);

        info!(
            "Started recording {} for stream {}",
            recording_id, stream_id
        );
        Ok(recording_id)
    }

    /// 停止录像
    pub async fn stop_recording(&self, recording_id: &str) -> Result<(), StreamError> {
        let mut active_recordings = self.active_recordings.lock().await;

        if let Some(mut task) = active_recordings.remove(recording_id) {
            // 关闭文件
            if let Some(mut writer) = task.writer.take() {
                if let Err(e) = writer.flush() {
                    warn!("Failed to flush recording {}: {}", recording_id, e);
                }
            }

            // 更新元数据
            task.metadata.end_time = Some(Utc::now());
            if let Some(end_time) = task.metadata.end_time {
                task.metadata.duration_seconds = task
                    .metadata
                    .start_time
                    .signed_duration_since(end_time)
                    .num_seconds()
                    .unsigned_abs();
            }
            task.metadata.file_size_bytes = task.current_size;
            task.metadata.total_frames = task.frame_count;
            task.metadata.key_frames = task.key_frame_count;

            // 更新索引
            let mut recording_index = self.recording_index.write().await;
            if let Some(meta) = recording_index.get_mut(recording_id) {
                *meta = task.metadata.clone();
            }
            drop(recording_index);

            // 从流映射中移除
            let mut stream_recordings = self.stream_recordings.write().await;
            stream_recordings.remove(&task.stream_id);

            info!("Stopped recording {}", recording_id);
            Ok(())
        } else {
            Err(StreamError::InternalError(format!(
                "Recording {} not found",
                recording_id
            )))
        }
    }

    /// 写入视频帧
    pub async fn write_frame(
        &self,
        stream_id: &str,
        frame: &VideoFrame,
    ) -> Result<(), StreamError> {
        // 查找对应的录像任务
        let recording_id = {
            let stream_recordings = self.stream_recordings.read().await;
            stream_recordings.get(stream_id).cloned()
        };

        let recording_id = match recording_id {
            Some(id) => id,
            None => return Ok(()), // 流没有在录像
        };

        let mut active_recordings = self.active_recordings.lock().await;
        if let Some(task) = active_recordings.get_mut(&recording_id) {
            // 检查文件大小限制
            if task.current_size >= self.config.max_file_size_mb * 1024 * 1024 {
                warn!("Recording {} reached max file size, stopping", recording_id);
                drop(active_recordings);
                // 需要停止当前录像并开始新的(暂时简化为停止)
                return self.stop_recording(&recording_id).await;
            }

            // 写入帧数据
            if let Some(writer) = task.writer.as_mut() {
                // 简化的写入逻辑:直接写入H.264 NALU
                // 在实际实现中,这里应该根据format选择合适的封装格式
                if let Err(e) = writer.write_all(&frame.data) {
                    error!("Failed to write frame to recording {}: {}", recording_id, e);
                    return Err(StreamError::InternalError(e.to_string()));
                }

                task.current_size += frame.data.len() as u64;
                task.frame_count += 1;

                if frame.frame_type == VideoFrameType::IFrame {
                    task.key_frame_count += 1;
                }
            }
        }

        Ok(())
    }

    /// 获取录像列表
    pub async fn get_recordings(
        &self,
        stream_id: Option<String>,
        start_time: Option<DateTime<Utc>>,
        end_time: Option<DateTime<Utc>>,
        limit: Option<usize>,
    ) -> Vec<RecordingMetadata> {
        let recording_index = self.recording_index.read().await;

        let mut recordings: Vec<RecordingMetadata> = recording_index
            .values()
            .filter(|meta| {
                // 按流ID过滤
                if let Some(ref stream) = stream_id {
                    if &meta.stream_id != stream {
                        return false;
                    }
                }

                // 按开始时间过滤
                if let Some(start) = start_time {
                    if meta.start_time < start {
                        return false;
                    }
                }

                // 按结束时间过滤
                if let Some(end) = end_time {
                    if let Some(end_time) = meta.end_time {
                        if end_time > end {
                            return false;
                        }
                    }
                }

                true
            })
            .cloned()
            .collect();

        // 按开始时间降序排序
        recordings.sort_by(|a, b| b.start_time.cmp(&a.start_time));

        // 应用限制
        if let Some(limit) = limit {
            recordings.truncate(limit);
        }

        recordings
    }

    /// 获取录像详情
    pub async fn get_recording(&self, recording_id: &str) -> Option<RecordingMetadata> {
        let recording_index = self.recording_index.read().await;
        recording_index.get(recording_id).cloned()
    }

    /// 删除录像
    pub async fn delete_recording(&self, recording_id: &str) -> Result<(), StreamError> {
        // 获取元数据
        let metadata = {
            let recording_index = self.recording_index.read().await;
            recording_index.get(recording_id).cloned()
        };

        let metadata = match metadata {
            Some(meta) => meta,
            None => {
                return Err(StreamError::InternalError(format!(
                    "Recording {} not found",
                    recording_id
                )))
            }
        };

        // 删除文件
        if let Err(e) = fs::remove_file(&metadata.file_path) {
            warn!(
                "Failed to delete recording file {}: {}",
                metadata.file_path, e
            );
        }

        // 从索引中移除
        let mut recording_index = self.recording_index.write().await;
        recording_index.remove(recording_id);

        info!("Deleted recording {}", recording_id);
        Ok(())
    }

    /// 获取录像状态
    pub async fn get_recording_status(&self, recording_id: &str) -> RecordingState {
        let active_recordings = self.active_recordings.lock().await;

        if active_recordings.contains_key(recording_id) {
            RecordingState::Recording
        } else {
            let recording_index = self.recording_index.read().await;
            if recording_index.contains_key(recording_id) {
                RecordingState::Idle
            } else {
                RecordingState::Error("Recording not found".to_string())
            }
        }
    }

    /// 检查磁盘空间
    async fn check_disk_space(&self) -> bool {
        // 简化实现:始终返回true
        // 在实际实现中,应该检查磁盘使用情况
        true
    }

    /// 清理过期录像
    pub async fn cleanup_old_recordings(&self) -> Result<usize, StreamError> {
        let retention_duration = Duration::days(self.config.retention_days as i64);
        let cutoff_time = Utc::now() - retention_duration;

        let recording_index = self.recording_index.read().await;
        let old_recordings: Vec<String> = recording_index
            .iter()
            .filter(|(_, meta)| meta.end_time.map(|end| end < cutoff_time).unwrap_or(false))
            .map(|(id, _)| id.clone())
            .collect();
        drop(recording_index);

        let mut deleted_count = 0;
        for recording_id in old_recordings {
            if self.delete_recording(&recording_id).await.is_ok() {
                deleted_count += 1;
            }
        }

        info!("Cleaned up {} old recordings", deleted_count);
        Ok(deleted_count)
    }

    /// 获取统计信息
    pub async fn get_statistics(&self) -> RecordingStatistics {
        let active_recordings = self.active_recordings.lock().await;
        let recording_index = self.recording_index.read().await;

        let total_recordings = recording_index.len();
        let active_recordings_count = active_recordings.len();
        let total_size_bytes: u64 = recording_index.values().map(|m| m.file_size_bytes).sum();
        let total_duration_seconds: u64 =
            recording_index.values().map(|m| m.duration_seconds).sum();

        RecordingStatistics {
            total_recordings,
            active_recordings_count,
            total_size_bytes,
            total_duration_seconds,
        }
    }
}

/// 录像统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingStatistics {
    /// 总录像数
    pub total_recordings: usize,
    /// 活跃录像数
    pub active_recordings_count: usize,
    /// 总大小(字节)
    pub total_size_bytes: u64,
    /// 总时长(秒)
    pub total_duration_seconds: u64,
}

/// 回放请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybackRequest {
    /// 录像ID
    pub recording_id: String,
    /// 开始时间(相对于录像开始的秒数)
    pub start_offset: u64,
    /// 结束时间(相对于录像开始的秒数)
    pub end_offset: Option<u64>,
    /// 播放速度(1.0 = 正常速度)
    pub speed: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_recording_manager() {
        let config = RecordingConfig::default();
        let manager = RecordingManager::new(config);

        let stats = manager.get_statistics().await;
        assert_eq!(stats.total_recordings, 0);
    }

    #[tokio::test]
    async fn test_start_stop_recording() {
        let manager = RecordingManager::with_default_config();

        let recording_id: String = manager
            .start_recording(
                "test_stream".to_string(),
                "device001".to_string(),
                1,
                Some(RecordingFormat::Mp4),
            )
            .await
            .expect("Failed to start recording");

        let status = manager.get_recording_status(&recording_id).await;
        assert_eq!(status, RecordingState::Recording);

        manager
            .stop_recording(&recording_id)
            .await
            .expect("Failed to stop recording");

        let status = manager.get_recording_status(&recording_id).await;
        assert_eq!(status, RecordingState::Idle);
    }
}
