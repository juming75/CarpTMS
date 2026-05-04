//! / BSJ协议实现 (帧头: 0x2D 0x2D)
// 用于中心站与数据库服务器通讯

use super::base::{Protocol, ProtocolData, ProtocolError};
use log::{debug, warn};

/// BSJ协议帧头
pub const BSJ_FRAME_HEADER: [u8; 2] = [0x2D, 0x2D];

/// BSJ协议帧尾
pub const BSJ_FRAME_TAIL: u8 = 0x0D;

/// BSJ协议命令ID
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum BsjCommand {
    /// 认证
    Auth = 0x01,
    /// 车辆查询
    VehicleQuery = 0x02,
    /// 用户查询
    UserQuery = 0x03,
    /// GPS历史查询
    GpsHistoryQuery = 0x04,
    /// 实时数据推送
    RealtimePush = 0x05,
}

impl BsjCommand {
    pub fn from_u8(val: u8) -> Option<Self> {
        match val {
            0x01 => Some(Self::Auth),
            0x02 => Some(Self::VehicleQuery),
            0x03 => Some(Self::UserQuery),
            0x04 => Some(Self::GpsHistoryQuery),
            0x05 => Some(Self::RealtimePush),
            _ => None,
        }
    }

    pub fn as_u8(self) -> u8 {
        self as u8
    }
}

/// BSJ协议实现
pub struct BSJProtocol;

impl Default for BSJProtocol {
    fn default() -> Self {
        Self::new()
    }
}

impl BSJProtocol {
    pub fn new() -> Self {
        Self
    }

    /// 解析BSJ协议头部
    fn parse_header(&self, data: &[u8]) -> Result<(BsjCommand, u16, u32), ProtocolError> {
        // 格式: 帧头(2B) + CMD(1B) + 长度(2B) + IP(4B) + 保留(2B) + 数据 + Xor(1B) + 帧尾(1B)
        if data.len() < 11 {
            return Err(ProtocolError::ParsingError(
                "Invalid BSJ protocol data length".to_string(),
            ));
        }

        // 检查帧头
        if data[0..2] != BSJ_FRAME_HEADER {
            warn!("Invalid BSJ protocol frame header: {:02X?}", &data[0..2]);
            return Err(ProtocolError::ParsingError(
                "Invalid BSJ protocol frame header".to_string(),
            ));
        }

        // 解析命令ID
        let command = BsjCommand::from_u8(data[2]).ok_or_else(|| {
            ProtocolError::UnsupportedCommand(format!("Unknown BSJ command: 0x{:02X}", data[2]))
        })?;

        // 解析长度(大端序,不包含帧头和帧尾)
        let length = u16::from_be_bytes([data[3], data[4]]);

        // 解析IP地址(通常全为0)
        let ip = u32::from_be_bytes([data[5], data[6], data[7], data[8]]);

        debug!(
            "BSJ header: command={:?}, length={}, ip={}",
            command, length, ip
        );

        Ok((command, length, ip))
    }

    /// 计算XOR校验
    fn calculate_xor_checksum(&self, data: &[u8], length: usize) -> u8 {
        let mut checksum: u8 = 0;
        for &byte in data.iter().take(length) {
            checksum ^= byte;
        }
        checksum
    }

    /// 解析心跳包
    fn parse_heartbeat(&self, _data: &[u8]) -> Result<ProtocolData, ProtocolError> {
        // 简单实现:心跳包可能只包含设备ID或为空
        let protocol_data = ProtocolData::new("bsj_heartbeat".to_string(), "heartbeat".to_string());
        Ok(protocol_data)
    }

    /// 解析位置信息包
    fn parse_location(&self, data: &[u8]) -> Result<ProtocolData, ProtocolError> {
        // 假设格式:设备ID(4B) + 经度(4B) + 纬度(4B) + 速度(1B) + 方向(1B)
        if data.len() < 14 {
            return Err(ProtocolError::ParsingError(
                "Invalid BSJ location packet length".to_string(),
            ));
        }

        let device_id = u32::from_be_bytes([data[0], data[1], data[2], data[3]]);
        let longitude = ((data[4] as i32) << 24)
            | ((data[5] as i32) << 16)
            | ((data[6] as i32) << 8)
            | (data[7] as i32);
        let latitude = ((data[8] as i32) << 24)
            | ((data[9] as i32) << 16)
            | ((data[10] as i32) << 8)
            | (data[11] as i32);
        let speed = data[12];
        let direction = data[13];

        let mut protocol_data = ProtocolData::new(device_id.to_string(), "location".to_string())
            .with_raw_data(data.to_vec());

        protocol_data.params.insert(
            "longitude".to_string(),
            format!("{}", longitude as f64 / 1000000.0),
        );
        protocol_data.params.insert(
            "latitude".to_string(),
            format!("{}", latitude as f64 / 1000000.0),
        );
        protocol_data
            .params
            .insert("speed".to_string(), speed.to_string());
        protocol_data
            .params
            .insert("direction".to_string(), direction.to_string());

        Ok(protocol_data)
    }

    /// 解析状态信息包
    fn parse_status(&self, data: &[u8]) -> Result<ProtocolData, ProtocolError> {
        // 假设格式:设备ID(4B) + 状态(1B)
        if data.len() < 5 {
            return Err(ProtocolError::ParsingError(
                "Invalid BSJ status packet length".to_string(),
            ));
        }

        let device_id = u32::from_be_bytes([data[0], data[1], data[2], data[3]]);
        let status = data[4];

        let mut protocol_data = ProtocolData::new(device_id.to_string(), "status".to_string())
            .with_raw_data(data.to_vec());

        protocol_data
            .params
            .insert("status".to_string(), format!("{:02X}", status));

        Ok(protocol_data)
    }

    /// 构建BSJ数据包
    pub fn build_packet(&self, command: BsjCommand, data: &[u8]) -> Result<Vec<u8>, ProtocolError> {
        let mut packet = Vec::new();

        // 帧头
        packet.extend_from_slice(&BSJ_FRAME_HEADER);

        // 命令ID
        packet.push(command.as_u8());

        // 长度(不包括帧头、命令、长度字段本身,即:IP(4) + 保留(2) + 数据长度)
        let data_len = 6 + data.len();
        packet.extend_from_slice(&(data_len as u16).to_be_bytes());

        // IP地址(通常全为0)
        packet.extend_from_slice(&[0, 0, 0, 0]);

        // 保留字段(2字节)
        packet.extend_from_slice(&[0, 0]);

        // 数据
        packet.extend_from_slice(data);

        // XOR校验(计算从帧头到数据末尾的所有字节)
        let xor_checksum = self.calculate_xor_checksum(&packet, packet.len());
        packet.push(xor_checksum);

        // 帧尾
        packet.push(BSJ_FRAME_TAIL);

        debug!(
            "Built BSJ packet: command={:?}, data_len={}, checksum=0x{:02X}",
            command, data_len, xor_checksum
        );

        Ok(packet)
    }
}

impl Protocol for BSJProtocol {
    fn parse(&self, data: &[u8]) -> Result<ProtocolData, ProtocolError> {
        debug!("Parsing BSJ protocol data: {:02X?}", data);

        // 验证数据
        if !self.validate(data) {
            return Err(ProtocolError::ValidationError(
                "Invalid BSJ protocol data".to_string(),
            ));
        }

        // 解析头部
        let (command, length, _ip) = self.parse_header(data)?;

        // 计算数据段的起始和结束位置
        // 格式: 帧头(2) + CMD(1) + 长度(2) + IP(4) + 保留(2) + 数据(len) + Xor(1) + 帧尾(1)
        let data_start = 11; // 2 + 1 + 2 + 4 + 2
        let data_end = data_start + length as usize;

        if data_end > data.len() - 2 {
            // 减去 XOR 和 帧尾
            return Err(ProtocolError::ParsingError(
                "BSJ packet data length mismatch".to_string(),
            ));
        }

        let payload = &data[data_start..data_end];

        // 根据命令解析数据
        let protocol_data = match command {
            BsjCommand::Auth => {
                // 认证包
                ProtocolData::new("bsj_auth".to_string(), "auth".to_string())
            }
            BsjCommand::VehicleQuery => {
                // 车辆查询
                ProtocolData::new("bsj_vehicle".to_string(), "vehicle_query".to_string())
            }
            BsjCommand::UserQuery => {
                // 用户查询
                ProtocolData::new("bsj_user".to_string(), "user_query".to_string())
            }
            BsjCommand::GpsHistoryQuery => {
                // GPS历史查询
                ProtocolData::new("bsj_gps".to_string(), "gps_history_query".to_string())
            }
            BsjCommand::RealtimePush => {
                // 实时数据推送,根据数据内容进一步解析
                if payload.len() >= 14 {
                    // 尝试解析为位置数据
                    self.parse_location(payload)?
                } else if payload.len() >= 5 {
                    // 尝试解析为状态数据
                    self.parse_status(payload)?
                } else {
                    // 心跳
                    self.parse_heartbeat(payload)?
                }
            }
        };

        Ok(protocol_data)
    }

    fn generate(&self, data: &ProtocolData) -> Result<Vec<u8>, ProtocolError> {
        debug!("Generating BSJ protocol data for command: {}", data.command);

        match data.command.as_str() {
            "auth_response" => {
                self.build_packet(BsjCommand::Auth, &[1]) // 1=成功
            }
            "vehicle_response" => self.build_packet(BsjCommand::VehicleQuery, &[]),
            "user_response" => self.build_packet(BsjCommand::UserQuery, &[]),
            "gps_response" => self.build_packet(BsjCommand::GpsHistoryQuery, &[]),
            "heartbeat" => self.build_packet(BsjCommand::RealtimePush, &[]),
            "location" => {
                // 构建位置数据包
                let mut packet_data = Vec::new();
                let device_id: u32 = data.device_id.parse().unwrap_or(0);
                packet_data.extend_from_slice(&device_id.to_be_bytes());
                packet_data.extend_from_slice(&[0; 12]); // 其他数据
                self.build_packet(BsjCommand::RealtimePush, &packet_data)
            }
            _ => Err(ProtocolError::UnsupportedCommand(format!(
                "Unsupported BSJ command: {}",
                data.command
            ))),
        }
    }

    fn name(&self) -> &str {
        "BSJ"
    }

    fn version(&self) -> &str {
        "1.0"
    }

    fn validate(&self, data: &[u8]) -> bool {
        if data.len() < 13 {
            warn!("BSJ protocol data too short: {} bytes", data.len());
            return false;
        }

        // 检查帧头
        if data[0..2] != BSJ_FRAME_HEADER {
            warn!("BSJ protocol invalid frame header: {:02X?}", &data[0..2]);
            return false;
        }

        // 检查帧尾
        if data[data.len() - 1] != BSJ_FRAME_TAIL {
            warn!(
                "BSJ protocol invalid frame tail: 0x{:02X}",
                data[data.len() - 1]
            );
            return false;
        }

        true
    }

    fn supported_commands(&self) -> Vec<&str> {
        vec![
            "auth",
            "vehicle_query",
            "user_query",
            "gps_history_query",
            "heartbeat",
            "location",
            "status",
        ]
    }
}
