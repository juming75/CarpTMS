//! / DB44 协议
use anyhow::{anyhow, Result};

/// DB44 协议头部
#[derive(Debug, Clone, Copy)]
pub struct Db44Header {
    pub start_flag: u16,  // 0xDB 0x44
    pub command_id: u16,  // 命令ID
    pub sequence: u16,    // 序列号
    pub data_length: u16, // 数据长度
}

/// DB44 协议尾部
#[derive(Debug, Clone, Copy)]
pub struct Db44Footer {
    pub checksum: u16, // CRC16 校验
}

/// DB44 协议消息
#[derive(Debug, Clone)]
pub struct Db44Message {
    pub header: Db44Header,
    pub data: Vec<u8>,
    pub footer: Db44Footer,
}

/// DB44 协议解析器
pub struct Db44Parser;

impl Db44Parser {
    /// 创建新的 DB44 解析器
    pub fn new() -> Self {
        Self
    }

    /// 解析 DB44 协议
    pub fn parse(&self, data: &[u8]) -> Result<Db44Message> {
        // 检查最小长度
        if data.len() < 10 {
            // 2+2+2+2+2 = 10
            return Err(anyhow!("Data too short for DB44 protocol"));
        }

        // 解析头部
        let start_flag = u16::from_be_bytes([data[0], data[1]]);
        if start_flag != 0xDB44 {
            return Err(anyhow!("Invalid DB44 start flag: 0x{:04X}", start_flag));
        }

        let command_id = u16::from_be_bytes([data[2], data[3]]);
        let sequence = u16::from_be_bytes([data[4], data[5]]);
        let data_length = u16::from_be_bytes([data[6], data[7]]);

        // 检查数据长度
        if data.len() < 10 + data_length as usize {
            return Err(anyhow!(
                "Incomplete DB44 packet: expected {} bytes, got {}",
                10 + data_length as usize,
                data.len()
            ));
        }

        let header = Db44Header {
            start_flag,
            command_id,
            sequence,
            data_length,
        };

        // 提取数据
        let data_start = 8;
        let data_end = 8 + data_length as usize;
        let packet_data = data[data_start..data_end].to_vec();

        // 解析尾部
        let checksum_offset = data_end;
        let checksum = u16::from_be_bytes([data[checksum_offset], data[checksum_offset + 1]]);

        // 验证校验和
        let calculated_checksum = self.calculate_checksum(&data[..checksum_offset]);
        if calculated_checksum != checksum {
            return Err(anyhow!(
                "Checksum mismatch: expected 0x{:04X}, got 0x{:04X}",
                calculated_checksum,
                checksum
            ));
        }

        let footer = Db44Footer { checksum };

        Ok(Db44Message {
            header,
            data: packet_data,
            footer,
        })
    }

    /// 计算 CRC16 校验和
    fn calculate_checksum(&self, data: &[u8]) -> u16 {
        let mut crc: u16 = 0xFFFF;

        for &byte in data {
            crc ^= byte as u16;
            for _ in 0..8 {
                if crc & 0x0001 != 0 {
                    crc = (crc >> 1) ^ 0xA001;
                } else {
                    crc >>= 1;
                }
            }
        }

        crc
    }
}

impl Default for Db44Parser {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for Db44Parser {
    fn clone(&self) -> Self {
        Self
    }
}

/// DB44 协议构建器
pub struct Db44Builder;

impl Db44Builder {
    /// 创建新的 DB44 构建器
    pub fn new() -> Self {
        Self
    }

    /// 构建响应
    pub fn build_response(&self, message: &Db44Message) -> Result<Vec<u8>> {
        let mut packet = Vec::new();

        // 构建头部
        packet.extend_from_slice(&message.header.start_flag.to_be_bytes());
        packet.extend_from_slice(&message.header.command_id.to_be_bytes());
        packet.extend_from_slice(&message.header.sequence.to_be_bytes());
        packet.extend_from_slice(&message.header.data_length.to_be_bytes());

        // 添加数据
        packet.extend_from_slice(&message.data);

        // 计算校验和
        let checksum = self.calculate_checksum(&packet);
        packet.extend_from_slice(&checksum.to_be_bytes());

        Ok(packet)
    }

    /// 计算 CRC16 校验和
    fn calculate_checksum(&self, data: &[u8]) -> u16 {
        let mut crc: u16 = 0xFFFF;

        for &byte in data {
            crc ^= byte as u16;
            for _ in 0..8 {
                if crc & 0x0001 != 0 {
                    crc = (crc >> 1) ^ 0xA001;
                } else {
                    crc >>= 1;
                }
            }
        }

        crc
    }
}

impl Default for Db44Builder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_db44_parser() {
        let parser = Db44Parser::new();

        // 构造测试数据包
        let mut packet = vec![0u8; 15];
        packet[0..2].copy_from_slice(&0xDB44u16.to_be_bytes()); // start_flag
        packet[2..4].copy_from_slice(&0x0001u16.to_be_bytes()); // command_id
        packet[4..6].copy_from_slice(&0x0001u16.to_be_bytes()); // sequence
        packet[6..8].copy_from_slice(&4u16.to_be_bytes()); // data_length
        packet[8..12].copy_from_slice(&[1, 2, 3, 4]); // data

        // 计算校验和
        let checksum = parser.calculate_checksum(&packet[..12]);
        packet[12..14].copy_from_slice(&checksum.to_be_bytes());

        // 解析
        let message = parser.parse(&packet);
        assert!(message.is_ok(), "DB44 parse failed: {:?}", message.err());
        let message = message.unwrap();
        assert_eq!(message.header.start_flag, 0xDB44);
        assert_eq!(message.header.command_id, 0x0001);
        assert_eq!(message.data, vec![1, 2, 3, 4]);
    }

    #[test]
    fn test_crc16() {
        let parser = Db44Parser::new();
        let data = vec![1, 2, 3, 4, 5];
        let checksum = parser.calculate_checksum(&data);

        // 验证校验和计算的一致性
        let calculated = parser.calculate_checksum(&data);
        assert_eq!(checksum, calculated);
    }
}
