//! / 心跳保活模块
use chrono::{DateTime, Duration, Utc};

/// 心跳间隔:60秒
pub const HEARTBEAT_INTERVAL: Duration = Duration::seconds(60);

/// 心跳超时:3次未响应
pub const HEARTBEAT_TIMEOUT: Duration = Duration::seconds(180);

/// 心跳处理器
pub struct HeartbeatHandler;

impl HeartbeatHandler {
    /// 创建新的心跳处理器
    pub fn new() -> Self {
        Self
    }

    /// 检查心跳是否超时
    pub fn is_timeout(&self, last_heartbeat: DateTime<Utc>) -> bool {
        let now = Utc::now();
        now - last_heartbeat > HEARTBEAT_TIMEOUT
    }

    /// 获取下一次心跳时间
    pub fn next_heartbeat(&self, last_heartbeat: DateTime<Utc>) -> DateTime<Utc> {
        last_heartbeat + HEARTBEAT_INTERVAL
    }
}

impl Default for HeartbeatHandler {
    fn default() -> Self {
        Self::new()
    }
}
