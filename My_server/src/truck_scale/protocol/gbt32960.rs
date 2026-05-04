//! / GB/T 32960 协议(国标)
use anyhow::{anyhow, Result};

/// GB/T 32960 协议头部
#[derive(Debug, Clone, Copy)]
pub struct Gbt32960Header {
    pub start_flag: u8,   // 0x23
    pub command_id: u8,   // 命令ID
    pub vin: [u8; 17],    // 车辆识别码
    pub encrypt_type: u8, // 加密方式
    pub data_length: u16, // 数据长度
}

/// GB/T 32960 协议尾部
#[derive(Debug, Clone, Copy)]
pub struct Gbt32960Footer {
    pub checksum: u8, // BCC 校验
}

/// GB/T 32960 协议消息
#[derive(Debug, Clone)]
pub struct Gbt32960Message {
    pub header: Gbt32960Header,
    pub data: Vec<u8>,
    pub footer: Gbt32960Footer,
}

/// GB/T 32960 协议解析器
pub struct Gbt32960Parser;

impl Gbt32960Parser {
    /// 创建新的 GB/T 32960 解析器
    pub fn new() -> Self {
        Self
    }

    /// 解析 GB/T 32960 协议
    pub fn parse(&self, data: &[u8]) -> Result<Gbt32960Message> {
        // 检查最小长度
        if data.len() < 25 {
            // 1+1+17+1+2+1+2 = 25
            return Err(anyhow!("Data too short for GB/T 32960 protocol"));
        }

        // 解析头部
        let start_flag = data[0];
        if start_flag != 0x23 {
            return Err(anyhow!(
                "Invalid GB/T 32960 start flag: 0x{:02X}",
                start_flag
            ));
        }

        let command_id = data[1];
        let mut vin = [0u8; 17];
        vin.copy_from_slice(&data[2..19]);
        let encrypt_type = data[19];
        let data_length = u16::from_be_bytes([data[20], data[21]]);

        // 检查数据长度
        if data.len() < 25 + data_length as usize {
            return Err(anyhow!(
                "Incomplete GB/T 32960 packet: expected {} bytes, got {}",
                25 + data_length as usize,
                data.len()
            ));
        }

        let header = Gbt32960Header {
            start_flag,
            command_id,
            vin,
            encrypt_type,
            data_length,
        };

        // 提取数据
        let data_start = 22;
        let data_end = 22 + data_length as usize;
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

        let footer = Gbt32960Footer { checksum };

        Ok(Gbt32960Message {
            header,
            data: packet_data,
            footer,
        })
    }

    /// 计算 BCC 校验和
    fn calculate_checksum(&self, data: &[u8]) -> u8 {
        data.iter().fold(0u8, |acc, &byte| acc.wrapping_add(byte))
    }
}

impl Default for Gbt32960Parser {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for Gbt32960Parser {
    fn clone(&self) -> Self {
        Self
    }
}

/// GB/T 32960 协议构建器
pub struct Gbt32960Builder;

impl Gbt32960Builder {
    /// 创建新的 GB/T 32960 构建器
    pub fn new() -> Self {
        Self
    }

    /// 构建响应
    pub fn build_response(&self, message: &Gbt32960Message) -> Result<Vec<u8>> {
        let mut packet = Vec::new();

        // 构建头部
        packet.push(message.header.start_flag);
        packet.push(message.header.command_id);
        packet.extend_from_slice(&message.header.vin);
        packet.push(message.header.encrypt_type);
        packet.extend_from_slice(&message.header.data_length.to_be_bytes());

        // 添加数据
        packet.extend_from_slice(&message.data);

        // 计算校验和
        let checksum = self.calculate_checksum(&packet);
        packet.push(checksum);

        Ok(packet)
    }

    /// 计算 BCC 校验和
    fn calculate_checksum(&self, data: &[u8]) -> u8 {
        data.iter().fold(0u8, |acc, &byte| acc.wrapping_add(byte))
    }
}

impl Default for Gbt32960Builder {
    fn default() -> Self {
        Self::new()
    }
}
