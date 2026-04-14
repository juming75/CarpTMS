//! / YW 协议
use anyhow::{anyhow, Result};

/// YW 协议头部(6字节)
#[derive(Debug, Clone, Copy)]
pub struct YwHeader {
    pub start_flag: u8,   // 0xAA
    pub command_id: u16,  // 命令ID
    pub sequence: u16,    // 序列号
    pub data_length: u16, // 数据长度
}

/// YW 协议尾部
#[derive(Debug, Clone, Copy)]
pub struct YwFooter {
    pub checksum: u8, // XOR 校验
}

/// YW 协议消息
#[derive(Debug, Clone)]
pub struct YwMessage {
    pub header: YwHeader,
    pub data: Vec<u8>,
    pub footer: YwFooter,
}

/// YW 协议解析器
pub struct YwParser;

impl YwParser {
    /// 创建新的 YW 解析器
    pub fn new() -> Self {
        Self
    }

    /// 解析 YW 协议
    pub fn parse(&self, data: &[u8]) -> Result<YwMessage> {
        // 检查最小长度
        if data.len() < 7 {
            // 6字节头部 + 1字节校验
            return Err(anyhow!("Data too short for YW protocol"));
        }

        // 解析头部
        let start_flag = data[0];
        if start_flag != 0xAA {
            return Err(anyhow!("Invalid YW start flag: 0x{:02X}", start_flag));
        }

        let command_id = u16::from_be_bytes([data[1], data[2]]);
        let sequence = u16::from_be_bytes([data[3], data[4]]);
        let data_length = u16::from_be_bytes([data[5], data[6]]);

        // 检查数据长度
        if data.len() < 7 + data_length as usize {
            return Err(anyhow!(
                "Incomplete YW packet: expected {} bytes, got {}",
                7 + data_length as usize,
                data.len()
            ));
        }

        let header = YwHeader {
            start_flag,
            command_id,
            sequence,
            data_length,
        };

        // 提取数据
        let data_start = 7;
        let data_end = 7 + data_length as usize;
        let packet_data = data[data_start..data_end].to_vec();

        // 解析尾部
        let checksum_offset = data_end;
        let checksum = data[checksum_offset];

        // 验证校验和
        let calculated_checksum = self.calculate_checksum(&data[..checksum_offset]);
        if calculated_checksum != checksum {
            return Err(anyhow!(
                "Checksum mismatch: expected 0x{:02X}, got 0x{:02X}",
                calculated_checksum,
                checksum
            ));
        }

        let footer = YwFooter { checksum };

        Ok(YwMessage {
            header,
            data: packet_data,
            footer,
        })
    }

    /// 计算 XOR 校验和
    fn calculate_checksum(&self, data: &[u8]) -> u8 {
        data.iter().fold(0u8, |acc, &byte| acc ^ byte)
    }
}

impl Default for YwParser {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for YwParser {
    fn clone(&self) -> Self {
        Self
    }
}

/// YW 协议构建器
pub struct YwBuilder;

impl YwBuilder {
    /// 创建新的 YW 构建器
    pub fn new() -> Self {
        Self
    }

    /// 构建响应
    pub fn build_response(&self, message: &YwMessage) -> Result<Vec<u8>> {
        let mut packet = Vec::new();

        // 构建头部
        packet.push(message.header.start_flag);
        packet.extend_from_slice(&message.header.command_id.to_be_bytes());
        packet.extend_from_slice(&message.header.sequence.to_be_bytes());
        packet.extend_from_slice(&message.header.data_length.to_be_bytes());

        // 添加数据
        packet.extend_from_slice(&message.data);

        // 计算校验和
        let checksum = self.calculate_checksum(&packet);
        packet.push(checksum);

        Ok(packet)
    }

    /// 计算 XOR 校验和
    fn calculate_checksum(&self, data: &[u8]) -> u8 {
        data.iter().fold(0u8, |acc, &byte| acc ^ byte)
    }
}

impl Default for YwBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_yw_parser() {
        let parser = YwParser::new();

        // 构造测试数据包
        let mut packet = vec![0u8; 15];
        packet[0] = 0xAA; // start_flag
        packet[1..3].copy_from_slice(&0x0001u16.to_be_bytes()); // command_id
        packet[3..5].copy_from_slice(&0x0001u16.to_be_bytes()); // sequence
        packet[5..7].copy_from_slice(&4u16.to_be_bytes()); // data_length
        packet[7..11].copy_from_slice(&[1, 2, 3, 4]); // data

        // 计算校验和
        let checksum = parser.calculate_checksum(&packet[..11]);
        packet[11] = checksum;

        // 解析
        let message = parser.parse(&packet);
        assert!(message.is_ok(), "YW parse failed: {:?}", message.err());
        let message = message.unwrap();
        assert_eq!(message.header.start_flag, 0xAA);
        assert_eq!(message.header.command_id, 0x0001);
        assert_eq!(message.data, vec![1, 2, 3, 4]);
    }
}
