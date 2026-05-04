//! JT1078音频对讲命令实现
//!
//! 实现0x9101实时音频对讲请求命令
//! 根据JT/T 1078-2016协议规范

use log::{debug, info};
use serde::{Deserialize, Serialize};

/// 0x9101实时音频控制命令
/// 用于平台向终端发起实时音频对讲请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeAudioControlCommand {
    /// 服务器标志（流媒体服务器ID）
    pub server_flag: u8,
    /// 音视频资源类型
    /// 0: 音视频, 1: 音频, 2: 视频, 3: 视频或音视频
    pub resource_type: u8,
    /// 音频参数
    pub audio_params: AudioControlParams,
    /// 逻辑通道号
    pub channel: u8,
}

/// 音频控制参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioControlParams {
    /// 音频格式
    /// 0: G.721, 1: G.722, 2: G.723, 3: G.728, 4: G.729
    /// 5: G.726(6位), 6: G.726(5位), 7: G.726(4位), 8: G.726(3位)
    /// 9: G.726(2位), 10: G.711_u, 11: G.711_a, 12: ADPCM, 13: MP3
    /// 14: 其他
    pub audio_format: u8,
    /// 音频采样率
    /// 0: 8KHz, 1: 22KHz, 2: 32KHz, 3: 44KHz, 4: 48KHz
    pub sample_rate: u8,
    /// 音频声道数
    /// 0: 单声道, 1: 立体声
    pub channels: u8,
    /// 音频位数
    /// 0: 8位, 1: 16位, 2: 32位
    pub bit_depth: u8,
}

/// 0x9102实时音视频传输状态应答
/// 终端收到0x9101后的应答
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeAVStatusResponse {
    /// 应答流水号（对应0x9101的流水号）
    pub response_serial: u16,
    /// 音视频资源类型
    pub resource_type: u8,
    /// 结果
    /// 0: 成功, 1: 失败, 2: 通道不支持
    pub result: u8,
}

/// 音频对讲命令构建器
/// 用于构建符合JT1078协议的音频对讲命令包
pub struct AudioCommandBuilder {
    /// 终端手机号（设备ID）
    pub terminal_phone: String,
    /// 消息流水号
    pub message_serial: u16,
    /// 音频控制参数
    pub audio_params: AudioControlParams,
    /// 逻辑通道号
    pub channel: u8,
    /// 服务器标识
    pub server_flag: u8,
}

impl AudioCommandBuilder {
    /// 创建新的音频命令构建器
    pub fn new(terminal_phone: String) -> Self {
        Self {
            terminal_phone,
            message_serial: 0,
            audio_params: AudioControlParams {
                audio_format: 10, // G.711_u (常用格式)
                sample_rate: 0,   // 8KHz
                channels: 0,      // 单声道
                bit_depth: 1,     // 16位
            },
            channel: 0,
            server_flag: 0,
        }
    }

    /// 设置消息流水号
    pub fn with_serial(mut self, serial: u16) -> Self {
        self.message_serial = serial;
        self
    }

    /// 设置音频格式
    pub fn with_audio_format(mut self, format: u8) -> Self {
        self.audio_params.audio_format = format;
        self
    }

    /// 设置采样率
    pub fn with_sample_rate(mut self, rate: u8) -> Self {
        self.audio_params.sample_rate = rate;
        self
    }

    /// 设置声道数
    pub fn with_channels(mut self, channels: u8) -> Self {
        self.audio_params.channels = channels;
        self
    }

    /// 设置位深度
    pub fn with_bit_depth(mut self, depth: u8) -> Self {
        self.audio_params.bit_depth = depth;
        self
    }

    /// 设置通道号
    pub fn with_channel(mut self, channel: u8) -> Self {
        self.channel = channel;
        self
    }

    /// 设置服务器标识
    pub fn with_server_flag(mut self, flag: u8) -> Self {
        self.server_flag = flag;
        self
    }

    /// 构建0x9101实时音频控制命令
    /// 命令格式（共8字节）：
    /// - 字节1: 服务器标识
    /// - 字节2: 音视频资源类型 (0表示仅音频)
    /// - 字节3: 音频格式
    /// - 字节4: 音频采样率
    /// - 字节5: 音频声道数
    /// - 字节6: 音频位数
    /// - 字节7-8: 保留
    pub fn build_9101_command(&self) -> Vec<u8> {
        let body = vec![
            self.server_flag,
            1, // 音视频资源类型 (0=仅音频)
            self.audio_params.audio_format,
            self.audio_params.sample_rate,
            self.audio_params.channels,
            self.audio_params.bit_depth,
            0, // 保留字节
            0, // 保留字节
        ];

        debug!(
            "Built 0x9101 command for terminal {}, body: {:02x?}",
            self.terminal_phone, body
        );

        body
    }

    /// 构建完整的JT808协议包（包含消息头）
    /// JT808消息头格式（12字节）：
    /// - 字节0-1: 消息ID (0x9101)
    /// - 字节2-3: 消息体属性 (长度+分包信息)
    /// - 字节4-9: 终端手机号 (6字节BCD码)
    /// - 字节10-11: 消息流水号
    pub fn build_complete_packet(&self) -> Vec<u8> {
        let body = self.build_9101_command();
        let body_len = body.len();

        let mut packet = Vec::with_capacity(12 + body_len + 2); // 头+体+校验+标识

        // 起始标识符 0x7e
        packet.push(0x7e);

        // 消息ID (0x9101)
        packet.push(0x91);
        packet.push(0x01);

        // 消息体属性 (高10位为长度)
        let body_property = body_len as u16;
        packet.push((body_property >> 8) as u8);
        packet.push(body_property as u8);

        // 终端手机号 (6字节BCD)
        let phone_bytes = self.encode_phone_bcd(&self.terminal_phone);
        packet.extend_from_slice(&phone_bytes);

        // 消息流水号
        packet.push((self.message_serial >> 8) as u8);
        packet.push(self.message_serial as u8);

        // 消息体
        packet.extend_from_slice(&body);

        // 计算校验码 (异或校验)
        let checksum = Self::calculate_checksum(&packet[1..]);
        packet.push(checksum);

        // 结束标识符 0x7e
        packet.push(0x7e);

        info!(
            "Built complete JT808 packet for 0x9101 command, total size: {} bytes",
            packet.len()
        );

        packet
    }

    /// 构建0x9203停止实时音视频传输命令
    pub fn build_9203_stop_command(&self) -> Vec<u8> {
        let body = vec![1, self.channel];

        self.build_packet_with_body(0x9203, &body)
    }

    /// 构建通用JT808命令包
    fn build_packet_with_body(&self, msg_id: u16, body: &[u8]) -> Vec<u8> {
        let mut packet = Vec::with_capacity(12 + body.len() + 2);

        // 起始标识符
        packet.push(0x7e);

        // 消息ID
        packet.push((msg_id >> 8) as u8);
        packet.push(msg_id as u8);

        // 消息体属性
        let body_property = body.len() as u16;
        packet.push((body_property >> 8) as u8);
        packet.push(body_property as u8);

        // 终端手机号
        let phone_bytes = self.encode_phone_bcd(&self.terminal_phone);
        packet.extend_from_slice(&phone_bytes);

        // 消息流水号
        packet.push((self.message_serial >> 8) as u8);
        packet.push(self.message_serial as u8);

        // 消息体
        packet.extend_from_slice(body);

        // 校验码
        let checksum = Self::calculate_checksum(&packet[1..]);
        packet.push(checksum);

        // 结束标识符
        packet.push(0x7e);

        packet
    }

    /// 编码手机号为BCD格式（6字节）
    fn encode_phone_bcd(&self, phone: &str) -> [u8; 6] {
        let mut result = [0x00; 6];
        let clean_phone = phone.replace(|c: char| !c.is_ascii_digit(), "");

        // 补齐到12位（6字节BCD）
        let padded = format!("{:0>12}", &clean_phone[..clean_phone.len().min(12)]);

        for (i, item) in result.iter_mut().enumerate() {
            let high = padded
                .chars()
                .nth(i * 2)
                .unwrap_or('0')
                .to_digit(10)
                .unwrap_or(0) as u8;
            let low = padded
                .chars()
                .nth(i * 2 + 1)
                .unwrap_or('0')
                .to_digit(10)
                .unwrap_or(0) as u8;
            *item = (high << 4) | low;
        }

        result
    }

    /// 计算异或校验码
    fn calculate_checksum(data: &[u8]) -> u8 {
        data.iter().fold(0, |acc, &b| acc ^ b)
    }
}

/// JT1078音频协议解析器
/// 用于解析终端返回的音频数据和应答
pub struct AudioProtocolParser;

impl AudioProtocolParser {
    /// 解析0x0001终端通用应答
    /// 检查终端是否成功接收0x9101命令
    pub fn parse_terminal_ack(data: &[u8]) -> Option<(u16, u16, u8)> {
        if data.len() < 5 {
            return None;
        }

        // 应答流水号
        let serial = u16::from_be_bytes([data[0], data[1]]);
        // 应答ID
        let ack_id = u16::from_be_bytes([data[2], data[3]]);
        // 结果
        let result = data[4];

        Some((serial, ack_id, result))
    }

    /// 解析JT1078音频帧
    /// 音频帧格式：0x30 0x31 0x63 0x64 + 帧头(12字节) + 音频数据
    pub fn parse_audio_frame(data: &[u8]) -> Option<(u8, Vec<u8>)> {
        if data.len() < 16 {
            return None;
        }

        // 检查起始标识符
        if data[0..4] != [0x30, 0x31, 0x63, 0x64] {
            return None;
        }

        // 数据类型 (字节4) - 0x05表示音频帧
        let data_type = data[4];
        if data_type & 0x1F != 0x05 {
            return None;
        }

        // 逻辑通道号 (字节5)
        let channel = data[5] & 0x1F;

        // 音频数据从第16字节开始
        let audio_data = data[16..].to_vec();

        Some((channel, audio_data))
    }

    /// 验证JT1078协议包
    pub fn validate_jt1078_packet(data: &[u8]) -> bool {
        if data.len() < 4 {
            return false;
        }

        // 检查JT808包
        if data[0] == 0x7e {
            return true;
        }

        // 检查JT1078视频/音频帧
        if data[0..4] == [0x30, 0x31, 0x63, 0x64] {
            return true;
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_9101_command() {
        let builder = AudioCommandBuilder::new("123456789012".to_string())
            .with_serial(1)
            .with_channel(1);

        let body = builder.build_9101_command();
        assert_eq!(body.len(), 8);
        assert_eq!(body[1], 1); // 音频资源类型
    }

    #[test]
    fn test_build_complete_packet() {
        let builder = AudioCommandBuilder::new("123456789012".to_string())
            .with_serial(1)
            .with_channel(1);

        let packet = builder.build_complete_packet();
        assert!(packet.len() > 12);
        assert_eq!(packet[0], 0x7e); // 起始标识
        assert_eq!(packet[packet.len() - 1], 0x7e); // 结束标识
    }

    #[test]
    fn test_calculate_checksum() {
        let data = vec![0x01, 0x02, 0x03];
        let checksum = AudioCommandBuilder::calculate_checksum(&data);
        assert_eq!(checksum, 0x01 ^ 0x02 ^ 0x03);
    }

    #[test]
    fn test_parse_audio_frame() {
        // 构造一个模拟的JT1078音频帧
        let mut frame = vec![0x30, 0x31, 0x63, 0x64]; // 起始标识
        frame.push(0x05); // 音频数据类型
        frame.push(0x01); // 通道1
        frame.extend_from_slice(&[0u8; 10]); // 帧头剩余部分
        frame.extend_from_slice(&[0x00, 0x01, 0x02, 0x03]); // 音频数据

        let result = AudioProtocolParser::parse_audio_frame(&frame);
        assert!(result.is_some());
        let (channel, audio_data) = result.unwrap();
        assert_eq!(channel, 1);
        assert_eq!(audio_data.len(), 4);
    }
}
