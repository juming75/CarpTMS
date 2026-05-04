//! / TF_CarManager 协议
use crate::truck_scale::protocol::compression;
use anyhow::{anyhow, Result};

/// TF_CarManager 数据包头部(16字节)
#[derive(Debug, Clone, Copy)]
pub struct CarManagerHeader {
    pub start_flag: u32,   // 0x12345678
    pub length: u32,       // 数据长度
    pub command_id: u32,   // 命令ID
    pub record_count: u32, // 记录数
    pub field_count: u32,  // 字段数
}

/// TF_CarManager 数据包尾部
#[derive(Debug, Clone, Copy)]
pub struct CarManagerFooter {
    pub checksum: u8,  // XOR 校验
    pub version: u32,  // 版本号
    pub end_flag: u32, // 0x87654321
}

/// TF_CarManager 协议消息
#[derive(Debug, Clone)]
pub struct CarManagerMessage {
    pub header: CarManagerHeader,
    pub field_types: Vec<u8>,  // 字段类型数组
    pub field_sizes: Vec<u32>, // 字段大小数组
    pub data: Vec<u8>,         // Deflate 压缩的数据
    pub footer: CarManagerFooter,
}

/// TF_CarManager 协议解析器
pub struct CarManagerParser {
    /// 是否解压缩数据
    decompress: bool,
}

impl CarManagerParser {
    /// 创建新的 TF_CarManager 解析器
    pub fn new() -> Self {
        Self { decompress: true }
    }

    /// 设置是否解压缩数据
    pub fn set_decompress(&mut self, decompress: bool) {
        self.decompress = decompress;
    }

    /// 解析 TF_CarManager 协议
    pub fn parse(&self, data: &[u8]) -> Result<CarManagerMessage> {
        // 检查最小长度
        if data.len() < 24 {
            // 16字节头部 + 4字节校验 + 4字节版本
            return Err(anyhow!("Data too short for TF_CarManager protocol"));
        }

        // 解析头部
        let start_flag = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        if start_flag != 0x12345678 {
            return Err(anyhow!(
                "Invalid TF_CarManager start flag: 0x{:08X}",
                start_flag
            ));
        }

        let length = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
        let command_id = u32::from_le_bytes([data[8], data[9], data[10], data[11]]);
        let record_count = u32::from_le_bytes([data[12], data[13], data[14], data[15]]);
        let field_count = u32::from_le_bytes([data[16], data[17], data[18], data[19]]);

        let header = CarManagerHeader {
            start_flag,
            length,
            command_id,
            record_count,
            field_count,
        };

        // 解析字段类型数组
        let offset = 20;
        let field_types_end = offset + field_count as usize;
        if data.len() < field_types_end {
            return Err(anyhow!("Incomplete field types array"));
        }
        let field_types = data[offset..field_types_end].to_vec();

        // 解析字段大小数组
        let field_sizes_start = field_types_end;
        let field_sizes_end = field_sizes_start + (field_count as usize * 4);
        if data.len() < field_sizes_end {
            return Err(anyhow!("Incomplete field sizes array"));
        }

        let mut field_sizes = Vec::with_capacity(field_count as usize);
        for i in 0..field_count as usize {
            let start = field_sizes_start + i * 4;
            let _end = start + 4;
            let size = u32::from_le_bytes([
                data[start],
                data[start + 1],
                data[start + 2],
                data[start + 3],
            ]);
            field_sizes.push(size);
        }

        // 解析压缩数据
        let compressed_data_start = field_sizes_end;
        let footer_offset = data.len() - 8;
        let compressed_data_end = footer_offset;
        let compressed_data = data[compressed_data_start..compressed_data_end].to_vec();

        // 解压缩数据(如果启用)
        let packet_data = if self.decompress {
            compression::decompress(&compressed_data)?
        } else {
            compressed_data
        };

        // 解析尾部
        let checksum = data[footer_offset];
        let version = u32::from_le_bytes([
            data[footer_offset + 1],
            data[footer_offset + 2],
            data[footer_offset + 3],
            data[footer_offset + 4],
        ]);
        let end_flag = u32::from_le_bytes([
            data[footer_offset + 5],
            data[footer_offset + 6],
            data[footer_offset + 7],
            data[footer_offset + 8],
        ]);

        if end_flag != 0x87654321 {
            return Err(anyhow!(
                "Invalid TF_CarManager end flag: 0x{:08X}",
                end_flag
            ));
        }

        let footer = CarManagerFooter {
            checksum,
            version,
            end_flag,
        };

        Ok(CarManagerMessage {
            header,
            field_types,
            field_sizes,
            data: packet_data,
            footer,
        })
    }
}

impl Default for CarManagerParser {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for CarManagerParser {
    fn clone(&self) -> Self {
        Self {
            decompress: self.decompress,
        }
    }
}

/// TF_CarManager 协议构建器
pub struct CarManagerBuilder {
    /// 是否压缩数据
    compress: bool,
}

impl CarManagerBuilder {
    /// 创建新的 TF_CarManager 构建器
    pub fn new() -> Self {
        Self { compress: true }
    }

    /// 设置是否压缩数据
    pub fn set_compress(&mut self, compress: bool) {
        self.compress = compress;
    }

    /// 构建响应
    pub fn build_response(&self, message: &CarManagerMessage) -> Result<Vec<u8>> {
        let mut packet = Vec::new();

        // 压缩数据(如果启用)
        let data = if self.compress {
            compression::compress(&message.data)?
        } else {
            message.data.clone()
        };

        // 构建头部
        packet.extend_from_slice(&message.header.start_flag.to_le_bytes());
        packet.extend_from_slice(&message.header.length.to_le_bytes());
        packet.extend_from_slice(&message.header.command_id.to_le_bytes());
        packet.extend_from_slice(&message.header.record_count.to_le_bytes());
        packet.extend_from_slice(&message.header.field_count.to_le_bytes());

        // 添加字段类型数组
        packet.extend_from_slice(&message.field_types);

        // 添加字段大小数组
        for size in &message.field_sizes {
            packet.extend_from_slice(&size.to_le_bytes());
        }

        // 添加压缩数据
        packet.extend_from_slice(&data);

        // 添加尾部
        packet.push(message.footer.checksum);
        packet.extend_from_slice(&message.footer.version.to_le_bytes());
        packet.extend_from_slice(&message.footer.end_flag.to_le_bytes());

        Ok(packet)
    }
}

impl Default for CarManagerBuilder {
    fn default() -> Self {
        Self::new()
    }
}
