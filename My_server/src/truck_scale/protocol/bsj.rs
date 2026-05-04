//! / BSJ 协议
use anyhow::{anyhow, Result};

/// BSJ 协议头部(13字节)
#[derive(Debug, Clone, Copy)]
pub struct BsjHeader {
    pub start_flag: u16,  // 0x2D 0x2D
    pub version: u8,      // 协议版本
    pub command_id: u16,  // 命令ID
    pub sequence: u16,    // 序列号
    pub data_length: u32, // 数据长度
}

/// BSJ 协议尾部
#[derive(Debug, Clone, Copy)]
pub struct BsjFooter {
    pub checksum: u8, // XOR 校验
    pub end_flag: u8, // 0x0D
}

/// BSJ 协议消息
#[derive(Debug, Clone)]
pub struct BsjMessage {
    pub header: BsjHeader,
    pub data: Vec<u8>,
    pub footer: BsjFooter,
}

/// BSJ 协议变体
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BsjVariant {
    Standard, // 标准变体
    Extended, // 扩展变体
    Video,    // 视频变体
    Scale,    // 称重变体
}

/// BSJ 协议解析器
pub struct BsjParser {
    variant: BsjVariant,
}

impl BsjParser {
    /// 创建新的 BSJ 解析器
    pub fn new() -> Self {
        Self {
            variant: BsjVariant::Standard,
        }
    }

    /// 设置协议变体
    pub fn set_variant(&mut self, variant: BsjVariant) {
        self.variant = variant;
    }

    /// 解析 BSJ 协议
    pub fn parse(&self, data: &[u8]) -> Result<BsjMessage> {
        // 检查最小长度
        if data.len() < 15 {
            // 13字节头部 + 1字节校验 + 1字节尾部
            return Err(anyhow!("Data too short for BSJ protocol"));
        }

        // 解析头部
        let start_flag = u16::from_be_bytes([data[0], data[1]]);
        if start_flag != 0x2D2D {
            return Err(anyhow!("Invalid BSJ start flag: 0x{:04X}", start_flag));
        }

        let version = data[2];
        let command_id = u16::from_be_bytes([data[3], data[4]]);
        let sequence = u16::from_be_bytes([data[5], data[6]]);
        let data_length = u32::from_be_bytes([data[7], data[8], data[9], data[10]]);

        // 检查数据长度
        if data.len() < 13 + data_length as usize {
            return Err(anyhow!(
                "Incomplete BSJ packet: expected {} bytes, got {}",
                13 + data_length as usize,
                data.len()
            ));
        }

        let header = BsjHeader {
            start_flag,
            version,
            command_id,
            sequence,
            data_length,
        };

        // 提取数据
        let data_start = 11;
        let data_end = 11 + data_length as usize;
        let packet_data = data[data_start..data_end].to_vec();

        // 解析尾部
        let checksum_offset = data_end;
        let end_flag_offset = data_end + 1;

        let checksum = data[checksum_offset];
        let end_flag = data[end_flag_offset];

        if end_flag != 0x0D {
            return Err(anyhow!("Invalid BSJ end flag: 0x{:02X}", end_flag));
        }

        // 验证校验和
        let calculated_checksum = self.calculate_checksum(&data[..checksum_offset]);
        if calculated_checksum != checksum {
            return Err(anyhow!(
                "Checksum mismatch: expected 0x{:02X}, got 0x{:02X}",
                calculated_checksum,
                checksum
            ));
        }

        let footer = BsjFooter { checksum, end_flag };

        Ok(BsjMessage {
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

impl Default for BsjParser {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for BsjParser {
    fn clone(&self) -> Self {
        Self {
            variant: self.variant,
        }
    }
}

/// BSJ 协议构建器
pub struct BsjBuilder {
    variant: BsjVariant,
}

impl BsjBuilder {
    /// 创建新的 BSJ 构建器
    pub fn new() -> Self {
        Self {
            variant: BsjVariant::Standard,
        }
    }

    /// 设置协议变体
    pub fn set_variant(&mut self, variant: BsjVariant) {
        self.variant = variant;
    }

    /// 构建响应
    pub fn build_response(&self, message: &BsjMessage) -> Result<Vec<u8>> {
        let mut packet = Vec::new();

        // 构建头部
        packet.extend_from_slice(&message.header.start_flag.to_be_bytes());
        packet.push(message.header.version);
        packet.extend_from_slice(&message.header.command_id.to_be_bytes());
        packet.extend_from_slice(&message.header.sequence.to_be_bytes());
        packet.extend_from_slice(&message.header.data_length.to_be_bytes());

        // 添加数据
        packet.extend_from_slice(&message.data);

        // 计算校验和
        let checksum = self.calculate_checksum(&packet);
        packet.push(checksum);

        // 添加尾部
        packet.push(message.footer.end_flag);

        Ok(packet)
    }

    /// 构建 BSJ 消息
    pub fn build_message(&self, command_id: u16, sequence: u16, data: Vec<u8>) -> Result<Vec<u8>> {
        let header = BsjHeader {
            start_flag: 0x2D2D,
            version: 1,
            command_id,
            sequence,
            data_length: data.len() as u32,
        };

        let footer = BsjFooter {
            checksum: 0, // 稍后计算
            end_flag: 0x0D,
        };

        let message = BsjMessage {
            header,
            data,
            footer,
        };

        self.build_response(&message)
    }

    /// 计算 XOR 校验和
    fn calculate_checksum(&self, data: &[u8]) -> u8 {
        data.iter().fold(0u8, |acc, &byte| acc ^ byte)
    }
}

impl Default for BsjBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bsj_parser() {
        let parser = BsjParser::new();

        // 构造测试数据包
        let mut packet = vec![0u8; 20];
        packet[0..2].copy_from_slice(&0x2D2Du16.to_be_bytes());
        packet[2] = 1; // version
        packet[3..5].copy_from_slice(&0x0001u16.to_be_bytes()); // command_id
        packet[5..7].copy_from_slice(&0x0001u16.to_be_bytes()); // sequence
        packet[7..11].copy_from_slice(&4u32.to_be_bytes()); // data_length
        packet[11..15].copy_from_slice(&[1, 2, 3, 4]); // data

        // 计算校验和
        let checksum = parser.calculate_checksum(&packet[..15]);
        packet[15] = checksum;
        packet[16] = 0x0D; // end_flag

        // 解析
        let message = parser
            .parse(&packet)
            .map_err(|e| {
                log::warn!("BSJ parse failed for packet: {}", e);
                format!("Invalid BSJ packet format: {}", e)
            })
            .expect("Failed to parse BSJ packet");
        assert_eq!(message.header.start_flag, 0x2D2D);
        assert_eq!(message.header.command_id, 0x0001);
        assert_eq!(message.data, vec![1, 2, 3, 4]);
    }

    #[test]
    fn test_bsj_builder() {
        let builder = BsjBuilder::new();
        let packet = builder
            .build_message(0x0001, 0x0001, vec![1, 2, 3, 4])
            .unwrap();

        let parser = BsjParser::new();
        let message = parser
            .parse(&packet)
            .map_err(|e| {
                log::warn!("BSJ parse failed for packet: {}", e);
                format!("Invalid BSJ packet format: {}", e)
            })
            .expect("Failed to parse BSJ packet");

        assert_eq!(message.header.command_id, 0x0001);
        assert_eq!(message.data, vec![1, 2, 3, 4]);
    }
}
