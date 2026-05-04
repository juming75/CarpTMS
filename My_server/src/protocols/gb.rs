//! / 国标协议实现
// 基于GB/T 19056协议规范

use super::base::{Protocol, ProtocolData, ProtocolError};
use log::debug;

pub struct GBProtocol;

impl Default for GBProtocol {
    fn default() -> Self {
        Self::new()
    }
}

impl GBProtocol {
    pub fn new() -> Self {
        Self
    }

    // 解析GB协议帧头
    fn parse_header(&self, data: &[u8]) -> Result<(u8, u16, u16), ProtocolError> {
        // 简单实现:假设帧头格式为 起始符(1) + 命令(1) + 长度(2) + 设备ID(6) + ...
        if data.len() < 10 {
            return Err(ProtocolError::ParsingError(
                "Invalid GB protocol data length".to_string(),
            ));
        }

        // 检查起始符
        if data[0] != 0x7e {
            return Err(ProtocolError::ParsingError(
                "Invalid GB protocol start flag".to_string(),
            ));
        }

        let command = data[1];
        let length = ((data[2] as u16) << 8) | (data[3] as u16);
        let device_id = ((data[4] as u16) << 8) | (data[5] as u16);

        Ok((command, length, device_id))
    }

    // 解析GB协议命令
    fn parse_command(&self, command: u8, data: &[u8]) -> Result<ProtocolData, ProtocolError> {
        match command {
            0x01 => self.parse_heartbeat(data),
            0x02 => self.parse_location(data),
            0x03 => self.parse_status(data),
            _ => Err(ProtocolError::UnsupportedCommand(format!(
                "Unknown GB command: {}",
                command
            ))),
        }
    }

    // 解析心跳包
    fn parse_heartbeat(&self, data: &[u8]) -> Result<ProtocolData, ProtocolError> {
        // 简单实现:假设心跳包格式为 起始符 + 命令(0x01) + 长度 + 设备ID + 时间 + ...
        if data.len() < 16 {
            return Err(ProtocolError::ParsingError(
                "Invalid GB heartbeat packet length".to_string(),
            ));
        }

        // 设备ID为6字节,这里简化为使用前2字节
        let device_id = format!("{:04x}", ((data[4] as u16) << 8) | (data[5] as u16));

        Ok(ProtocolData::new(device_id, "heartbeat".to_string()).with_raw_data(data.to_vec()))
    }

    // 解析位置信息包
    fn parse_location(&self, data: &[u8]) -> Result<ProtocolData, ProtocolError> {
        // 简单实现:假设位置包格式为 起始符 + 命令(0x02) + 长度 + 设备ID + 经度 + 纬度 + ...
        if data.len() < 24 {
            return Err(ProtocolError::ParsingError(
                "Invalid GB location packet length".to_string(),
            ));
        }

        let device_id = format!("{:04x}", ((data[4] as u16) << 8) | (data[5] as u16));

        // 解析经纬度(假设为4字节整数,单位为1/1000000度)
        let longitude = ((data[6] as i32) << 24)
            | ((data[7] as i32) << 16)
            | ((data[8] as i32) << 8)
            | (data[9] as i32);
        let latitude = ((data[10] as i32) << 24)
            | ((data[11] as i32) << 16)
            | ((data[12] as i32) << 8)
            | (data[13] as i32);

        let mut protocol_data =
            ProtocolData::new(device_id, "location".to_string()).with_raw_data(data.to_vec());

        protocol_data.params.insert(
            "longitude".to_string(),
            format!("{}", longitude as f64 / 1000000.0),
        );
        protocol_data.params.insert(
            "latitude".to_string(),
            format!("{}", latitude as f64 / 1000000.0),
        );

        Ok(protocol_data)
    }

    // 解析状态信息包
    fn parse_status(&self, data: &[u8]) -> Result<ProtocolData, ProtocolError> {
        // 简单实现:假设状态包格式为 起始符 + 命令(0x03) + 长度 + 设备ID + 状态 + ...
        if data.len() < 18 {
            return Err(ProtocolError::ParsingError(
                "Invalid GB status packet length".to_string(),
            ));
        }

        let device_id = format!("{:04x}", ((data[4] as u16) << 8) | (data[5] as u16));
        let status = data[6];

        let mut protocol_data =
            ProtocolData::new(device_id, "status".to_string()).with_raw_data(data.to_vec());

        protocol_data
            .params
            .insert("status".to_string(), format!("{:02x}", status));

        Ok(protocol_data)
    }
}

impl Protocol for GBProtocol {
    fn parse(&self, data: &[u8]) -> Result<ProtocolData, ProtocolError> {
        debug!("Parsing GB protocol data: {:?}", data);

        // 验证数据
        if !self.validate(data) {
            return Err(ProtocolError::ValidationError(
                "Invalid GB protocol data".to_string(),
            ));
        }

        // 解析帧头
        let (command, _length, _device_id) = self.parse_header(data)?;

        // 解析命令
        self.parse_command(command, data)
    }

    fn generate(&self, data: &ProtocolData) -> Result<Vec<u8>, ProtocolError> {
        debug!("Generating GB protocol data for command: {}", data.command);

        match data.command.as_str() {
            "heartbeat" => {
                // 生成心跳响应
                Ok(vec![0x7e, 0x81, 0x00, 0x04, 0x00, 0x01, 0x00, 0x00, 0x7e])
            }
            "location" => {
                // 生成位置响应
                Ok(vec![0x7e, 0x82, 0x00, 0x04, 0x00, 0x01, 0x00, 0x00, 0x7e])
            }
            "status" => {
                // 生成状态响应
                Ok(vec![0x7e, 0x83, 0x00, 0x04, 0x00, 0x01, 0x00, 0x00, 0x7e])
            }
            _ => Err(ProtocolError::UnsupportedCommand(format!(
                "Unsupported GB command: {}",
                data.command
            ))),
        }
    }

    fn name(&self) -> &str {
        "GB/T 19056"
    }

    fn version(&self) -> &str {
        "2012"
    }

    fn validate(&self, data: &[u8]) -> bool {
        // 简单验证:检查起始符和结束符
        if data.len() < 2 {
            return false;
        }

        // 检查起始符
        if data[0] != 0x7e {
            return false;
        }

        // 检查结束符
        if data[data.len() - 1] != 0x7e {
            return false;
        }

        true
    }

    fn supported_commands(&self) -> Vec<&str> {
        vec!["heartbeat", "location", "status"]
    }
}
