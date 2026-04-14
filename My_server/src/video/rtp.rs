//! / RTP协议实现
// 用于GB28181和JT1078视频流传输

use log::warn;
use std::mem;

/// RTP固定头部大小
pub const RTP_HEADER_SIZE: usize = 12;

/// RTP包
#[derive(Debug, Clone)]
pub struct RtpPacket {
    /// RTP版本 (2 bits)
    pub version: u8,
    /// 填充标志 (1 bit)
    pub padding: bool,
    /// 扩展标志 (1 bit)
    pub extension: bool,
    /// CSRC计数器 (4 bits)
    pub csrc_count: u8,
    /// 标记 (1 bit)
    pub marker: bool,
    /// 载荷类型 (7 bits)
    pub payload_type: u8,
    /// 序列号
    pub sequence: u16,
    /// 时间戳
    pub timestamp: u32,
    /// 同步源标识符
    pub ssrc: u32,
    /// CSRC列表
    pub csrc: Vec<u32>,
    /// 扩展头
    pub extension_header: Option<RtpExtension>,
    /// 载荷数据
    pub payload: Vec<u8>,
}

/// RTP扩展头
#[derive(Debug, Clone)]
pub struct RtpExtension {
    /// 扩展类型
    pub profile: u16,
    /// 扩展数据
    pub data: Vec<u8>,
}

impl RtpPacket {
    /// 从字节数组解析RTP包
    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        if data.len() < RTP_HEADER_SIZE {
            warn!("RTP packet too short: {} bytes", data.len());
            return None;
        }

        let first_byte = data[0];
        let version = (first_byte >> 6) & 0x03;
        let padding = (first_byte & 0x20) != 0;
        let extension = (first_byte & 0x10) != 0;
        let csrc_count = first_byte & 0x0F;

        let second_byte = data[1];
        let marker = (second_byte & 0x80) != 0;
        let payload_type = second_byte & 0x7F;

        let sequence = u16::from_be_bytes([data[2], data[3]]);
        let timestamp = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);
        let ssrc = u32::from_be_bytes([data[8], data[9], data[10], data[11]]);

        let mut offset = RTP_HEADER_SIZE;
        let mut csrc = Vec::new();

        // 解析CSRC列表
        for _ in 0..csrc_count {
            if offset + 4 > data.len() {
                warn!("Invalid CSRC in RTP packet");
                return None;
            }
            let csrc_item = u32::from_be_bytes([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
            ]);
            csrc.push(csrc_item);
            offset += 4;
        }

        // 解析扩展头
        let extension_header = if extension {
            if offset + 4 > data.len() {
                warn!("Invalid extension header in RTP packet");
                return None;
            }
            let profile = u16::from_be_bytes([data[offset], data[offset + 1]]);
            let length = u16::from_be_bytes([data[offset + 2], data[offset + 3]]) as usize;
            offset += 4;

            if offset + length * 4 > data.len() {
                warn!("Invalid extension data in RTP packet");
                return None;
            }

            let ext_data = data[offset..offset + length * 4].to_vec();
            offset += length * 4;

            Some(RtpExtension {
                profile,
                data: ext_data,
            })
        } else {
            None
        };

        // 剩余数据为载荷
        let payload = data[offset..].to_vec();

        Some(Self {
            version,
            padding,
            extension,
            csrc_count,
            marker,
            payload_type,
            sequence,
            timestamp,
            ssrc,
            csrc,
            extension_header,
            payload,
        })
    }

    /// 转换为字节数组
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        // 第一个字节: V(2) P(1) X(1) CC(4)
        let first_byte = (self.version << 6)
            | ((self.padding as u8) << 5)
            | ((self.extension as u8) << 4)
            | (self.csrc_count & 0x0F);
        bytes.push(first_byte);

        // 第二个字节: M(1) PT(7)
        let second_byte = ((self.marker as u8) << 7) | (self.payload_type & 0x7F);
        bytes.push(second_byte);

        // 序列号
        bytes.extend_from_slice(&self.sequence.to_be_bytes());

        // 时间戳
        bytes.extend_from_slice(&self.timestamp.to_be_bytes());

        // SSRC
        bytes.extend_from_slice(&self.ssrc.to_be_bytes());

        // CSRC列表
        for csrc in &self.csrc {
            bytes.extend_from_slice(&csrc.to_be_bytes());
        }

        // 扩展头
        if let Some(ext) = &self.extension_header {
            bytes.extend_from_slice(&ext.profile.to_be_bytes());
            let length = (ext.data.len() / 4) as u16;
            bytes.extend_from_slice(&length.to_be_bytes());
            bytes.extend_from_slice(&ext.data);
        }

        // 载荷数据
        bytes.extend_from_slice(&self.payload);

        bytes
    }

    /// 创建新的RTP包
    pub fn new(payload_type: u8, sequence: u16, timestamp: u32, ssrc: u32) -> Self {
        Self {
            version: 2,
            padding: false,
            extension: false,
            csrc_count: 0,
            marker: false,
            payload_type,
            sequence,
            timestamp,
            ssrc,
            csrc: Vec::new(),
            extension_header: None,
            payload: Vec::new(),
        }
    }

    /// 设置载荷数据
    pub fn with_payload(mut self, payload: Vec<u8>) -> Self {
        self.payload = payload;
        self
    }

    /// 设置标记位
    pub fn with_marker(mut self, marker: bool) -> Self {
        self.marker = marker;
        self
    }

    /// 计算包大小
    pub fn size(&self) -> usize {
        let mut size = RTP_HEADER_SIZE;
        size += self.csrc.len() * mem::size_of::<u32>();
        if let Some(ext) = &self.extension_header {
            size += 4 + ext.data.len();
        }
        size += self.payload.len();
        size
    }

    /// 是否为关键帧(通常由marker位指示)
    pub fn is_key_frame(&self) -> bool {
        self.marker
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rtp_packet_serialization() {
        let packet = RtpPacket::new(96, 1000, 0, 0x12345678)
            .with_marker(true)
            .with_payload(vec![0x00, 0x01, 0x02, 0x03]);

        let bytes = packet.to_bytes();
        let parsed = RtpPacket::from_bytes(&bytes);

        assert!(parsed.is_some());
        let parsed = parsed.unwrap();
        assert_eq!(parsed.version, 2);
        assert_eq!(parsed.marker, true);
        assert_eq!(parsed.payload_type, 96);
        assert_eq!(parsed.sequence, 1000);
        assert_eq!(parsed.ssrc, 0x12345678);
    }

    #[test]
    fn test_rtp_packet_min_size() {
        let bytes = vec![0u8; 11]; // 小于最小RTP头部大小
        let packet = RtpPacket::from_bytes(&bytes);
        assert!(packet.is_none());
    }
}
