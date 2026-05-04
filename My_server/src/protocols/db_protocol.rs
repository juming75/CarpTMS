//! / 数据库协议实现 (魔数: 0x12 0x34 0x56 0x78)
// 用于 GVLP 客户端与前置机通讯

use super::base::{Protocol, ProtocolData, ProtocolError};
use log::{debug, warn};

/// 数据库协议魔数
pub const DB_PROTOCOL_MAGIC: [u8; 4] = [0x12, 0x34, 0x56, 0x78];

/// 数据库协议命令ID
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u16)]
pub enum DbCommand {
    /// 申请登录
    ApplyLogin = 0x0001,
    /// 应答登录
    AnswerLogin = 0x0002,
    /// 心跳
    Heartbeat = 0x0003,
    /// 心跳应答
    HeartbeatResponse = 0x0004,
    /// 数据查询
    DataQuery = 0x0005,
    /// 数据应答
    DataResponse = 0x0006,
    /// 车辆查询
    VehicleQuery = 0x0010,
    /// 车辆应答
    VehicleResponse = 0x0011,
    /// 用户查询
    UserQuery = 0x0012,
    /// 用户应答
    UserResponse = 0x0013,
    /// GPS历史查询
    GpsHistoryQuery = 0x0020,
    /// GPS历史应答
    GpsHistoryResponse = 0x0021,
}

impl DbCommand {
    pub fn from_u16(val: u16) -> Option<Self> {
        match val {
            0x0001 => Some(Self::ApplyLogin),
            0x0002 => Some(Self::AnswerLogin),
            0x0003 => Some(Self::Heartbeat),
            0x0004 => Some(Self::HeartbeatResponse),
            0x0005 => Some(Self::DataQuery),
            0x0006 => Some(Self::DataResponse),
            0x0010 => Some(Self::VehicleQuery),
            0x0011 => Some(Self::VehicleResponse),
            0x0012 => Some(Self::UserQuery),
            0x0013 => Some(Self::UserResponse),
            0x0020 => Some(Self::GpsHistoryQuery),
            0x0021 => Some(Self::GpsHistoryResponse),
            _ => None,
        }
    }

    pub fn as_u16(self) -> u16 {
        self as u16
    }
}

/// 数据库协议头部
#[derive(Debug, Clone)]
pub struct DbHeader {
    pub magic: [u8; 4],
    pub length: u32,
    pub command: DbCommand,
}

/// 数据库协议实现
pub struct DbProtocol;

impl Default for DbProtocol {
    fn default() -> Self {
        Self::new()
    }
}

impl DbProtocol {
    pub fn new() -> Self {
        Self
    }

    /// 解析协议头部
    fn parse_header(&self, data: &[u8]) -> Result<DbHeader, ProtocolError> {
        if data.len() < 10 {
            return Err(ProtocolError::ParsingError(
                "Invalid DbProtocol data length, at least 10 bytes required".to_string(),
            ));
        }

        // 检查魔数
        let magic = [data[0], data[1], data[2], data[3]];
        if magic != DB_PROTOCOL_MAGIC {
            warn!("Invalid DbProtocol magic: {:02X?}", magic);
            return Err(ProtocolError::ValidationError(format!(
                "Invalid DbProtocol magic: expected {:02X?}, got {:02X?}",
                DB_PROTOCOL_MAGIC, magic
            )));
        }

        // 解析长度(大端序)
        let length = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);

        // 解析命令ID(大端序)
        let command_id = u16::from_be_bytes([data[8], data[9]]);
        let command = DbCommand::from_u16(command_id).ok_or_else(|| {
            ProtocolError::UnsupportedCommand(format!(
                "Unknown DbProtocol command: 0x{:04X}",
                command_id
            ))
        })?;

        debug!(
            "DbProtocol header: magic={:02X?}, length={}, command={:?}",
            magic, length, command
        );

        Ok(DbHeader {
            magic,
            length,
            command,
        })
    }

    /// 计算校验和
    fn calculate_checksum(&self, data: &[u8]) -> u8 {
        let mut checksum: u8 = 0;
        for byte in data {
            checksum ^= byte;
        }
        checksum
    }

    /// 解析登录请求
    fn parse_login_request(&self, data: &[u8]) -> Result<ProtocolData, ProtocolError> {
        // 格式: 用户名长度(2) + 用户名(n) + 密码长度(2) + 密码(n)
        if data.len() < 4 {
            return Err(ProtocolError::ParsingError(
                "Invalid login request length".to_string(),
            ));
        }

        let username_len = u16::from_be_bytes([data[0], data[1]]) as usize;
        if data.len() < 2 + username_len + 2 {
            return Err(ProtocolError::ParsingError(
                "Invalid login request data".to_string(),
            ));
        }

        let username = &data[2..2 + username_len];
        let offset = 2 + username_len;
        let password_len = u16::from_be_bytes([data[offset], data[offset + 1]]) as usize;

        if data.len() < offset + 2 + password_len {
            return Err(ProtocolError::ParsingError(
                "Invalid login request data".to_string(),
            ));
        }

        let password = &data[offset + 2..offset + 2 + password_len];
        let username_str = String::from_utf8_lossy(username).to_string();
        let password_str = String::from_utf8_lossy(password).to_string();

        debug!("Login request: username={}, password=***", username_str);

        let mut protocol_data =
            ProtocolData::new("db_login".to_string(), "apply_login".to_string());
        protocol_data
            .params
            .insert("username".to_string(), username_str);
        protocol_data
            .params
            .insert("password".to_string(), password_str);

        Ok(protocol_data)
    }

    /// 解析心跳包
    fn parse_heartbeat(&self, _data: &[u8]) -> Result<ProtocolData, ProtocolError> {
        let protocol_data = ProtocolData::new("db_heartbeat".to_string(), "heartbeat".to_string());
        Ok(protocol_data)
    }

    /// 解析车辆查询请求
    fn parse_vehicle_query(&self, _data: &[u8]) -> Result<ProtocolData, ProtocolError> {
        let protocol_data =
            ProtocolData::new("db_vehicle".to_string(), "vehicle_query".to_string());
        Ok(protocol_data)
    }

    /// 解析用户查询请求
    fn parse_user_query(&self, _data: &[u8]) -> Result<ProtocolData, ProtocolError> {
        let protocol_data = ProtocolData::new("db_user".to_string(), "user_query".to_string());
        Ok(protocol_data)
    }

    /// 解析GPS历史查询请求
    fn parse_gps_history_query(&self, data: &[u8]) -> Result<ProtocolData, ProtocolError> {
        // 格式: 设备ID长度(2) + 设备ID(n) + 开始时间(4) + 结束时间(4)
        if data.len() < 10 {
            return Err(ProtocolError::ParsingError(
                "Invalid GPS history query length".to_string(),
            ));
        }

        let device_id_len = u16::from_be_bytes([data[0], data[1]]) as usize;
        if data.len() < 2 + device_id_len + 8 {
            return Err(ProtocolError::ParsingError(
                "Invalid GPS history query data".to_string(),
            ));
        }

        let device_id = &data[2..2 + device_id_len];
        let offset = 2 + device_id_len;

        let start_time = u32::from_be_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ]);

        let end_time = u32::from_be_bytes([
            data[offset + 4],
            data[offset + 5],
            data[offset + 6],
            data[offset + 7],
        ]);

        let device_id_str = String::from_utf8_lossy(device_id).to_string();

        debug!(
            "GPS history query: device_id={}, start_time={}, end_time={}",
            device_id_str, start_time, end_time
        );

        let mut protocol_data =
            ProtocolData::new("db_gps".to_string(), "gps_history_query".to_string());
        protocol_data
            .params
            .insert("device_id".to_string(), device_id_str);
        protocol_data
            .params
            .insert("start_time".to_string(), start_time.to_string());
        protocol_data
            .params
            .insert("end_time".to_string(), end_time.to_string());

        Ok(protocol_data)
    }

    /// 构建登录响应
    fn build_login_response(&self, success: bool, user_id: i32) -> Result<Vec<u8>, ProtocolError> {
        let mut data = Vec::new();

        // 标志位 (0=失败, 1=成功)
        data.push(if success { 1 } else { 0 });

        // 用户ID (4字节)
        data.extend_from_slice(&user_id.to_be_bytes());

        // 构建完整数据包
        self.build_packet(DbCommand::AnswerLogin, &data)
    }

    /// 构建心跳响应
    fn build_heartbeat_response(&self) -> Result<Vec<u8>, ProtocolError> {
        self.build_packet(DbCommand::HeartbeatResponse, &[])
    }

    /// 构建数据包
    fn build_packet(&self, command: DbCommand, data: &[u8]) -> Result<Vec<u8>, ProtocolError> {
        let mut packet = Vec::new();

        // 魔数
        packet.extend_from_slice(&DB_PROTOCOL_MAGIC);

        // 数据长度 (包含命令+数据+校验)
        let length = 2 + data.len() + 1;

        // 长度字段 (4字节,大端序)
        packet.extend_from_slice(&(length as u32).to_be_bytes());

        // 命令ID (2字节,大端序)
        packet.extend_from_slice(&command.as_u16().to_be_bytes());

        // 数据
        packet.extend_from_slice(data);

        // 计算校验和 (从魔数到数据末尾)
        let checksum = self.calculate_checksum(&packet);
        packet.push(checksum);

        debug!(
            "Built DbProtocol packet: command={:?}, length={}, checksum=0x{:02X}",
            command, length, checksum
        );

        Ok(packet)
    }
}

impl Protocol for DbProtocol {
    fn parse(&self, data: &[u8]) -> Result<ProtocolData, ProtocolError> {
        debug!("Parsing DbProtocol data: {:02X?}", data);

        // 验证数据
        if !self.validate(data) {
            return Err(ProtocolError::ValidationError(
                "Invalid DbProtocol data".to_string(),
            ));
        }

        // 解析头部
        let header = self.parse_header(data)?;

        // 提取数据段(跳过头部10字节和尾部1字节校验和)
        let data_start = 10;
        let data_end = data.len() - 1;
        let payload = if data_end > data_start {
            &data[data_start..data_end]
        } else {
            &[]
        };

        // 解析命令
        let protocol_data = match header.command {
            DbCommand::ApplyLogin => self.parse_login_request(payload)?,
            DbCommand::Heartbeat => self.parse_heartbeat(payload)?,
            DbCommand::VehicleQuery => self.parse_vehicle_query(payload)?,
            DbCommand::UserQuery => self.parse_user_query(payload)?,
            DbCommand::GpsHistoryQuery => self.parse_gps_history_query(payload)?,
            _ => ProtocolData::new(
                "db_unknown".to_string(),
                format!("{:?}", header.command).to_lowercase(),
            )
            .with_raw_data(payload.to_vec()),
        };

        Ok(protocol_data)
    }

    fn generate(&self, data: &ProtocolData) -> Result<Vec<u8>, ProtocolError> {
        debug!("Generating DbProtocol data for command: {}", data.command);

        match data.command.as_str() {
            "answer_login" => {
                let success = data
                    .params
                    .get("success")
                    .and_then(|s| s.parse::<bool>().ok())
                    .unwrap_or(false);
                let user_id = data
                    .params
                    .get("user_id")
                    .and_then(|s| s.parse::<i32>().ok())
                    .unwrap_or(0);
                self.build_login_response(success, user_id)
            }
            "heartbeat_response" => self.build_heartbeat_response(),
            _ => Err(ProtocolError::UnsupportedCommand(format!(
                "Unsupported DbProtocol command: {}",
                data.command
            ))),
        }
    }

    fn name(&self) -> &str {
        "DB_Protocol"
    }

    fn version(&self) -> &str {
        "1.0"
    }

    fn validate(&self, data: &[u8]) -> bool {
        if data.len() < 11 {
            warn!("DbProtocol data too short: {} bytes", data.len());
            return false;
        }

        // 检查魔数
        let magic = [data[0], data[1], data[2], data[3]];
        if magic != DB_PROTOCOL_MAGIC {
            warn!("DbProtocol invalid magic: {:02X?}", magic);
            return false;
        }

        // 验证校验和
        let calculated_checksum = self.calculate_checksum(&data[..data.len() - 1]);
        let actual_checksum = data[data.len() - 1];

        if calculated_checksum != actual_checksum {
            warn!(
                "DbProtocol checksum mismatch: calculated=0x{:02X}, actual=0x{:02X}",
                calculated_checksum, actual_checksum
            );
            return false;
        }

        true
    }

    fn supported_commands(&self) -> Vec<&str> {
        vec![
            "apply_login",
            "answer_login",
            "heartbeat",
            "heartbeat_response",
            "vehicle_query",
            "user_query",
            "gps_history_query",
        ]
    }
}
