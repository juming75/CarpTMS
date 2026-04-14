//! / 视频服务配置模块
// 支持JT1078和GB28181视频协议的配置

use serde::{Deserialize, Serialize};
use std::env;

/// 视频服务器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoConfig {
    /// 服务器是否启用
    pub enabled: bool,

    /// 存储路径(录像文件存储位置)
    pub storage_path: String,

    /// JT1078配置
    pub jt1078: Jt1078Config,

    /// GB28181配置
    pub gb28181: Gb28181Config,

    /// 流配置
    pub stream: StreamConfig,

    /// 服务器配置
    pub server: ServerConfig,
}

/// JT1078协议配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Jt1078Config {
    /// 是否启用JT1078
    pub enabled: bool,

    /// UDP监听端口
    pub listen_port: u16,

    /// 最大并发连接数
    pub max_connections: usize,

    /// 接收缓冲区大小(字节)
    pub buffer_size: usize,

    /// 会话超时时间(秒)
    pub session_timeout: u64,

    /// 心跳间隔(秒)
    pub heartbeat_interval: u64,

    /// 最大丢包重传次数
    pub max_retries: u32,
}

/// GB28181协议配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gb28181Config {
    /// 是否启用GB28181
    pub enabled: bool,

    /// SIP服务器监听端口
    pub sip_port: u16,

    /// SIP服务器ID
    pub server_id: String,

    /// SIP服务器域
    pub server_domain: String,

    /// RTP端口范围
    pub rtp_port_start: u16,

    pub rtp_port_end: u16,

    /// 设备认证密码
    pub auth_password: String,

    /// 会话超时时间(秒)
    pub session_timeout: u64,

    /// 心跳间隔(秒)
    pub heartbeat_interval: u64,
}

/// 流配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamConfig {
    /// 默认视频编码
    pub default_video_codec: String,

    /// 默认音频编码
    pub default_audio_codec: String,

    /// 默认分辨率
    pub default_resolution: String,

    /// 默认帧率
    pub default_framerate: u8,

    /// 默认码率 (kbps)
    pub default_bitrate: u32,

    /// 最大流数量
    pub max_streams: usize,

    /// 每个流的最大客户端数
    pub max_clients_per_stream: usize,

    /// 流缓冲区大小(帧数)
    pub frame_buffer_size: usize,
}

/// 服务器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// HTTP流服务器端口(HTTP-FLV)
    pub http_flv_port: u16,

    /// HLS服务器端口
    pub hls_port: u16,

    /// RTMP服务器端口(可选)
    pub rtmp_port: Option<u16>,

    /// WebSocket服务器端口
    pub ws_port: u16,

    /// 工作线程数
    pub worker_threads: Option<usize>,

    /// 最大并发连接
    pub max_connections: usize,

    /// 读取超时(秒)
    pub read_timeout: u64,

    /// 写入超时(秒)
    pub write_timeout: u64,
}

impl Default for VideoConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            storage_path: "./recordings".to_string(),
            jt1078: Jt1078Config::default(),
            gb28181: Gb28181Config::default(),
            stream: StreamConfig::default(),
            server: ServerConfig::default(),
        }
    }
}

impl Default for Jt1078Config {
    fn default() -> Self {
        Self {
            enabled: true,
            listen_port: 9788,
            max_connections: 1000,
            buffer_size: 65536,
            session_timeout: 300,
            heartbeat_interval: 60,
            max_retries: 3,
        }
    }
}

impl Default for Gb28181Config {
    fn default() -> Self {
        Self {
            enabled: true,
            sip_port: 5060,
            server_id: "34020000002000000001".to_string(),
            server_domain: "3402000000".to_string(),
            rtp_port_start: 10000,
            rtp_port_end: 20000,
            auth_password: "admin123".to_string(),
            session_timeout: 3600,
            heartbeat_interval: 60,
        }
    }
}

impl Default for StreamConfig {
    fn default() -> Self {
        Self {
            default_video_codec: "H264".to_string(),
            default_audio_codec: "G711A".to_string(),
            default_resolution: "1280x720".to_string(),
            default_framerate: 25,
            default_bitrate: 2048,
            max_streams: 1000,
            max_clients_per_stream: 100,
            frame_buffer_size: 100,
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            http_flv_port: 8081,
            hls_port: 8084,
            rtmp_port: None,
            ws_port: 8083,
            worker_threads: None,
            max_connections: 5000,
            read_timeout: 30,
            write_timeout: 30,
        }
    }
}

impl VideoConfig {
    /// 从环境变量加载配置
    pub fn from_env() -> Self {
        let mut config = Self {
            enabled: env::var("VIDEO_ENABLED")
                .map(|v| v.parse::<bool>().unwrap_or(true))
                .unwrap_or(true),
            ..Self::default()
        };

        // 存储路径
        if let Ok(path) = env::var("VIDEO_STORAGE_PATH") {
            config.storage_path = path;
        }

        // JT1078配置
        if let Ok(port) = env::var("JT1078_PORT") {
            config.jt1078.listen_port = port.parse().unwrap_or(9788);
        }
        if let Ok(max_conn) = env::var("JT1078_MAX_CONNECTIONS") {
            config.jt1078.max_connections = max_conn.parse().unwrap_or(1000);
        }
        config.jt1078.enabled = env::var("JT1078_ENABLED")
            .map(|v| v.parse::<bool>().unwrap_or(true))
            .unwrap_or(true);

        // GB28181配置
        if let Ok(port) = env::var("GB28181_SIP_PORT") {
            config.gb28181.sip_port = port.parse().unwrap_or(5060);
        }
        if let Ok(server_id) = env::var("GB28181_SERVER_ID") {
            config.gb28181.server_id = server_id;
        }
        if let Ok(domain) = env::var("GB28181_DOMAIN") {
            config.gb28181.server_domain = domain;
        }
        if let Ok(port_start) = env::var("GB28181_RTP_PORT_START") {
            config.gb28181.rtp_port_start = port_start.parse().unwrap_or(10000);
        }
        if let Ok(port_end) = env::var("GB28181_RTP_PORT_END") {
            config.gb28181.rtp_port_end = port_end.parse().unwrap_or(20000);
        }
        config.gb28181.enabled = env::var("GB28181_ENABLED")
            .map(|v| v.parse::<bool>().unwrap_or(true))
            .unwrap_or(true);

        // 流配置
        if let Ok(codec) = env::var("VIDEO_DEFAULT_CODEC") {
            config.stream.default_video_codec = codec;
        }
        if let Ok(bitrate) = env::var("VIDEO_DEFAULT_BITRATE") {
            config.stream.default_bitrate = bitrate.parse().unwrap_or(2048);
        }
        if let Ok(framerate) = env::var("VIDEO_DEFAULT_FRAMERATE") {
            config.stream.default_framerate = framerate.parse().unwrap_or(25);
        }
        if let Ok(resolution) = env::var("VIDEO_DEFAULT_RESOLUTION") {
            config.stream.default_resolution = resolution;
        }
        if let Ok(max_streams) = env::var("VIDEO_MAX_STREAMS") {
            config.stream.max_streams = max_streams.parse().unwrap_or(1000);
        }

        // 服务器配置
        if let Ok(flv_port) = env::var("VIDEO_HTTP_FLV_PORT") {
            config.server.http_flv_port = flv_port.parse().unwrap_or(8081);
        }
        if let Ok(hls_port) = env::var("VIDEO_HLS_PORT") {
            config.server.hls_port = hls_port.parse().unwrap_or(8082);
        }
        if let Ok(ws_port) = env::var("VIDEO_WS_PORT") {
            config.server.ws_port = ws_port.parse().unwrap_or(8083);
        }
        if let Ok(max_conn) = env::var("VIDEO_MAX_CONNECTIONS") {
            config.server.max_connections = max_conn.parse().unwrap_or(5000);
        }

        config
    }

    /// 验证配置
    pub fn validate(&self) -> Result<(), String> {
        if !self.enabled {
            return Ok(());
        }

        // 验证JT1078配置
        if self.jt1078.enabled {
            if self.jt1078.listen_port == 0 {
                return Err("JT1078 listen port cannot be 0".to_string());
            }
            if self.jt1078.max_connections == 0 {
                return Err("JT1078 max connections cannot be 0".to_string());
            }
        }

        // 验证GB28181配置
        if self.gb28181.enabled {
            if self.gb28181.sip_port == 0 {
                return Err("GB28181 SIP port cannot be 0".to_string());
            }
            if self.gb28181.rtp_port_start >= self.gb28181.rtp_port_end {
                return Err("GB28181 RTP port range invalid".to_string());
            }
            if self.gb28181.server_id.is_empty() {
                return Err("GB28181 server ID cannot be empty".to_string());
            }
            if self.gb28181.server_domain.is_empty() {
                return Err("GB28181 server domain cannot be empty".to_string());
            }
        }

        // 验证流配置
        if self.stream.max_streams == 0 {
            return Err("Max streams cannot be 0".to_string());
        }
        if self.stream.frame_buffer_size == 0 {
            return Err("Frame buffer size cannot be 0".to_string());
        }

        // 验证服务器配置
        if self.server.http_flv_port == 0 {
            return Err("HTTP-FLV port cannot be 0".to_string());
        }
        if self.server.hls_port == 0 {
            return Err("HLS port cannot be 0".to_string());
        }
        if self.server.ws_port == 0 {
            return Err("WebSocket port cannot be 0".to_string());
        }
        if self.server.max_connections == 0 {
            return Err("Max connections cannot be 0".to_string());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = VideoConfig::default();
        assert!(config.enabled);
        assert!(config.jt1078.enabled);
        assert!(config.gb28181.enabled);
    }

    #[test]
    fn test_validate_valid_config() {
        let config = VideoConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_invalid_jt1078_port() {
        let mut config = VideoConfig::default();
        config.jt1078.listen_port = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_invalid_rtp_port_range() {
        let mut config = VideoConfig::default();
        config.gb28181.rtp_port_start = 20000;
        config.gb28181.rtp_port_end = 10000;
        assert!(config.validate().is_err());
    }
}
