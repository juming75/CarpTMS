#[cfg(test)]
mod tests {
    use crate::video::{StreamType, VideoCodec, AudioCodec, VideoFrameType, VideoFrame, VideoStreamInfo};

    #[test]
    fn test_stream_type_values() {
        // 测试所有流类型都能正常创建
        let types = [
            StreamType::JT1078,
            StreamType::GB28181,
            StreamType::GB28181Playback,
            StreamType::HttpFlv,
            StreamType::Hls,
            StreamType::Rtmp,
            StreamType::WebRtc,
        ];

        assert_eq!(types.len(), 7);
    }

    #[test]
    fn test_video_codec_values() {
        // 测试所有视频编解码格式
        let codecs = [
            VideoCodec::H264,
            VideoCodec::H265,
            VideoCodec::Mpeg4,
            VideoCodec::Mjpeg,
            VideoCodec::Vp8,
            VideoCodec::Vp9,
            VideoCodec::Av1,
        ];

        assert_eq!(codecs.len(), 7);
    }

    #[test]
    fn test_audio_codec_values() {
        // 测试所有音频编解码格式
        let codecs = [
            AudioCodec::Aac,
            AudioCodec::G711A,
            AudioCodec::G711U,
            AudioCodec::G726,
            AudioCodec::Opus,
            AudioCodec::Pcmu,
            AudioCodec::Pcma,
        ];

        assert_eq!(codecs.len(), 7);
    }

    #[test]
    fn test_video_frame_type_values() {
        // 测试所有帧类型
        let types = [
            VideoFrameType::IFrame,
            VideoFrameType::PFrame,
            VideoFrameType::BFrame,
            VideoFrameType::AudioFrame,
        ];

        assert_eq!(types.len(), 4);
    }

    #[test]
    fn test_video_frame_creation() {
        use bytes::Bytes;
        // 测试视频帧创建
        let frame = VideoFrame {
            frame_type: VideoFrameType::IFrame,
            timestamp: 12345,
            data: Bytes::from(vec![1, 2, 3, 4, 5]),
            sequence: 100,
        };

        assert_eq!(frame.frame_type, VideoFrameType::IFrame);
        assert_eq!(frame.timestamp, 12345);
        assert_eq!(frame.data, Bytes::from(vec![1, 2, 3, 4, 5]));
        assert_eq!(frame.sequence, 100);
    }

    #[test]
    fn test_video_stream_info_creation() {
        // 测试视频流信息创建
        let info = VideoStreamInfo {
            stream_id: "stream_001".to_string(),
            device_id: "device_001".to_string(),
            channel_id: 1,
            stream_type: StreamType::JT1078,
            video_codec: VideoCodec::H264,
            audio_codec: Some(AudioCodec::Aac),
            resolution: Some((1920, 1080)),
            framerate: Some(30.0),
            bitrate: Some(4000),
            online: true,
            client_count: 5,
        };

        assert_eq!(info.stream_id, "stream_001");
        assert_eq!(info.stream_type, StreamType::JT1078);
        assert_eq!(info.video_codec, VideoCodec::H264);
        assert_eq!(info.online, true);
        assert_eq!(info.client_count, 5);
    }

    #[test]
    fn test_stream_type_serialization() {
        // 测试流类型序列化
        let stream_type = StreamType::JT1078;
        let serialized = serde_json::to_string(&stream_type).unwrap();
        let deserialized: StreamType = serde_json::from_str(&serialized).unwrap();

        assert_eq!(stream_type, deserialized);
    }

    #[test]
    fn test_video_stream_info_serialization() {
        // 测试视频流信息序列化
        let info = VideoStreamInfo {
            stream_id: "stream_001".to_string(),
            device_id: "device_001".to_string(),
            channel_id: 1,
            stream_type: StreamType::JT1078,
            video_codec: VideoCodec::H264,
            audio_codec: Some(AudioCodec::Aac),
            resolution: Some((1920, 1080)),
            framerate: Some(30.0),
            bitrate: Some(4000),
            online: true,
            client_count: 5,
        };

        let serialized = serde_json::to_string(&info).unwrap();
        let deserialized: VideoStreamInfo = serde_json::from_str(&serialized).unwrap();

        assert_eq!(info.stream_id, deserialized.stream_id);
        assert_eq!(info.stream_type, deserialized.stream_type);
    }

    #[test]
    fn test_video_frame_type_serialization() {
        // 测试视频帧类型序列化
        let frame_type = VideoFrameType::IFrame;
        let serialized = serde_json::to_string(&frame_type).unwrap();
        let deserialized: VideoFrameType = serde_json::from_str(&serialized).unwrap();

        assert_eq!(frame_type, deserialized);
    }
}





