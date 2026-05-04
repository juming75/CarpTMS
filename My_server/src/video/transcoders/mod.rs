//! / 视频流转换器模块
// 将 RTP/JT1078/GB28181 流转换为标准 Web 流协议

pub mod flv;
pub mod hls;
pub mod mod_base;
pub mod rtmp;

pub use flv::FlvTranscoder;
pub use hls::HlsTranscoder;
pub use mod_base::{StreamOutput, StreamTranscoder, TranscodeConfig};
pub use rtmp::RtmpTranscoder;

use log::info;

/// 初始化流转换器模块
pub fn init_transcoders() {
    info!("Initializing video stream transcoders...");
}
