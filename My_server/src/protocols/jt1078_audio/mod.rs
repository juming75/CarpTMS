//! JT/T 1078 双向对讲模块
//!
//! 实现JT/T 1078-2016协议的音频双向对讲功能
//! 核心流程：
//! 1. 客户端发起对讲请求
//! 2. 平台下发0x9101实时音频对讲请求到终端
//! 3. 终端上传实时音频到流媒体服务器
//! 4. 流媒体服务器将音频转发给客户端
//! 5. 客户端通过WebSocket推送音频到服务器
//! 6. 服务器将音频转发给终端

pub mod audio_forwarder;
pub mod audio_queue;
pub mod commands;
pub mod websocket_audio;
