//! 视频服务统一错误类型
//!
//! 为视频流相关模块提供标准化的错误处理方式
//!
//! # 设计原则
//! - 使用 thiserror 派生简化错误定义
//! - 支持错误链和上下文信息
//! - 与 AppError 完全兼容
//! - 支持 tracing 日志集成
//!
//! # 使用示例
//! ```ignore
//! use crate::video::errors::ServiceError;
//!
//! fn process_stream() -> Result<(), ServiceError> {
//!     Err(ServiceError::StreamNotFound {
//!         stream_id: "stream_001".to_string(),
//!     })
//! }
//! ```

use std::io;
use thiserror::Error;

/// 视频服务错误类型
#[derive(Debug, Error)]
pub enum ServiceError {
    // ============ 流相关错误 ============
    /// 流不存在
    #[error("Stream not found: {stream_id}")]
    StreamNotFound { stream_id: String },

    /// 流已存在
    #[error("Stream already exists: {stream_id}")]
    StreamAlreadyExists { stream_id: String },

    /// 流未激活
    #[error("Stream not active: {stream_id}")]
    StreamNotActive { stream_id: String },

    /// 流会话无效
    #[error("Invalid stream session: {session_id}")]
    InvalidSession { session_id: String },

    // ============ 编解码相关错误 ============
    /// 编解码器不支持
    #[error("Unsupported codec: {codec}")]
    UnsupportedCodec { codec: String },

    /// 编解码初始化失败
    #[error("Failed to initialize codec: {reason}")]
    CodecInitFailed { reason: String },

    /// 编解码错误
    #[error("Codec error: {message}")]
    CodecError { message: String },

    /// 转码失败
    #[error("Transcoding failed for stream {stream_id}: {reason}")]
    TranscodeFailed { stream_id: String, reason: String },

    // ============ 网络相关错误 ============
    /// 连接失败
    #[error("Connection failed to {host}:{port}: {reason}")]
    ConnectionFailed {
        host: String,
        port: u16,
        reason: String,
    },

    /// 连接超时
    #[error("Connection timeout to {host}:{port} after {timeout_ms}ms")]
    ConnectionTimeout {
        host: String,
        port: u16,
        timeout_ms: u64,
    },

    /// 连接断开
    #[error("Connection lost: {stream_id}")]
    ConnectionLost { stream_id: String },

    // ============ 数据相关错误 ============
    /// 数据格式错误
    #[error("Invalid data format: {format}")]
    InvalidFormat { format: String },

    /// 数据解析错误
    #[error("Failed to parse data: {context}")]
    ParseError { context: String },

    /// 数据包损坏
    #[error("Corrupted packet: sequence={sequence}")]
    CorruptedPacket { sequence: u32 },

    // ============ 存储相关错误 ============
    /// 录像存储失败
    #[error("Failed to store recording: {file_path}")]
    StorageError { file_path: String },

    /// 磁盘空间不足
    #[error("Disk space exhausted: {path} (available: {available_bytes} bytes)")]
    DiskSpaceExhausted { path: String, available_bytes: u64 },

    // ============ 协议相关错误 ============
    /// 协议错误
    #[error("Protocol error: {message}")]
    ProtocolError { message: String },

    /// 认证失败
    #[error("Authentication failed: {reason}")]
    AuthFailed { reason: String },

    /// 授权失败
    #[error("Authorization failed for stream: {stream_id}")]
    AuthzFailed { stream_id: String },

    // ============ 资源相关错误 ============
    /// 资源耗尽
    #[error("Resource exhausted: {resource}")]
    ResourceExhausted { resource: String },

    /// 连接数超限
    #[error("Too many connections: {current}/{max}")]
    TooManyConnections { current: usize, max: usize },

    // ============ 配置相关错误 ============
    /// 配置错误
    #[error("Configuration error: {message}")]
    ConfigError { message: String },

    // ============ 底层错误 ============
    /// IO 错误
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    /// 数据库错误
    #[error("Database error: {context}")]
    Database { context: String },

    /// Redis 错误
    #[error("Redis error: {context}")]
    Redis { context: String },

    // ============ 未知错误 ============
    /// 未知错误
    #[error("Unknown error: {message}")]
    Unknown { message: String },
}

impl ServiceError {
    /// 判断是否为可重试错误
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::ConnectionTimeout { .. }
                | Self::ConnectionFailed { .. }
                | Self::ConnectionLost { .. }
                | Self::Database { .. }
                | Self::Redis { .. }
                | Self::StorageError { .. }
                | Self::TranscodeFailed { .. }
        )
    }

    /// 获取错误级别
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            Self::DiskSpaceExhausted { .. }
            | Self::TooManyConnections { .. }
            | Self::ResourceExhausted { .. } => ErrorSeverity::Critical,

            Self::ConnectionLost { .. }
            | Self::ConnectionFailed { .. }
            | Self::ConnectionTimeout { .. }
            | Self::StorageError { .. }
            | Self::Database { .. }
            | Self::Redis { .. } => ErrorSeverity::Error,

            Self::CodecInitFailed { .. }
            | Self::TranscodeFailed { .. }
            | Self::ParseError { .. }
            | Self::CorruptedPacket { .. } => ErrorSeverity::Warning,

            Self::StreamNotFound { .. }
            | Self::StreamNotActive { .. }
            | Self::UnsupportedCodec { .. }
            | Self::InvalidFormat { .. }
            | Self::ProtocolError { .. }
            | Self::AuthFailed { .. }
            | Self::AuthzFailed { .. } => ErrorSeverity::Info,

            Self::StreamAlreadyExists { .. }
            | Self::InvalidSession { .. }
            | Self::CodecError { .. }
            | Self::ConfigError { .. }
            | Self::Unknown { .. } => ErrorSeverity::Info,
            Self::Io(_) => ErrorSeverity::Error,
        }
    }

    /// 记录错误日志
    pub fn log(&self) {
        let severity = self.severity();
        let message = self.to_string();

        match severity {
            ErrorSeverity::Critical => {
                tracing::error!(error = %self, "{}", message);
            }
            ErrorSeverity::Error => {
                tracing::error!(error = %self, "{}", message);
            }
            ErrorSeverity::Warning => {
                tracing::warn!(error = %self, "{}", message);
            }
            ErrorSeverity::Info => {
                tracing::info!(error = %self, "{}", message);
            }
        }
    }
}

/// 错误严重级别
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorSeverity {
    /// 严重错误 - 需要立即处理
    Critical,
    /// 错误 - 需要关注
    Error,
    /// 警告 - 需要关注但不紧急
    Warning,
    /// 信息 - 提示性信息
    Info,
}

impl std::fmt::Display for ErrorSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Critical => write!(f, "CRITICAL"),
            Self::Error => write!(f, "ERROR"),
            Self::Warning => write!(f, "WARNING"),
            Self::Info => write!(f, "INFO"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_is_retryable() {
        let errors = vec![
            (
                ServiceError::ConnectionTimeout {
                    host: "localhost".to_string(),
                    port: 8080,
                    timeout_ms: 5000,
                },
                true,
            ),
            (
                ServiceError::StreamNotFound {
                    stream_id: "test".to_string(),
                },
                false,
            ),
            (
                ServiceError::TranscodeFailed {
                    stream_id: "test".to_string(),
                    reason: "timeout".to_string(),
                },
                true,
            ),
        ];

        for (error, expected) in errors {
            assert_eq!(error.is_retryable(), expected);
        }
    }

    #[test]
    fn test_error_severity() {
        let disk_error = ServiceError::DiskSpaceExhausted {
            path: "/data".to_string(),
            available_bytes: 0,
        };
        assert_eq!(disk_error.severity(), ErrorSeverity::Critical);

        let stream_error = ServiceError::StreamNotFound {
            stream_id: "test".to_string(),
        };
        assert_eq!(stream_error.severity(), ErrorSeverity::Info);
    }
}
