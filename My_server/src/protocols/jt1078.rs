//! / JT1078-2019 视频协议实现
// 基于《道路运输车辆卫星定位系统 车载视频终端通讯协议》标准

use super::base::{Protocol, ProtocolData, ProtocolError};
use log::{debug, warn};
use serde::{Deserialize, Serialize};

/// JT1078协议版本
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Jt1078Version {
    /// JT1078-2016版本
    V2016,
    /// JT1078-2019版本
    V2019,
}

/// 视频数据类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VideoDataType {
    /// I帧(关键帧)
    IFrame,
    /// P帧(预测帧)
    PFrame,
    /// B帧(双向预测帧)
    BFrame,
    /// 音频帧
    AudioFrame,
    /// 未知类型
    Unknown,
}

impl From<u8> for VideoDataType {
    fn from(val: u8) -> Self {
        match val & 0x1F {
            0x01 => VideoDataType::IFrame,
            0x02 => VideoDataType::PFrame,
            0x03 => VideoDataType::BFrame,
            0x05 => VideoDataType::AudioFrame,
            _ => VideoDataType::Unknown,
        }
    }
}

/// JT1078协议帧头
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Jt1078Header {
    /// 起始标识符 (0x30 0x31 0x63 0x64)
    pub start_flag: u32,
    /// 数据类型
    pub data_type: u8,
    /// 逻辑通道号
    pub logic_channel: u8,
    /// 帧属性
    pub frame_property: u16,
    /// 数据时间戳
    pub timestamp: u32,
    /// 上次I帧或I帧之间的时间间隔
    pub last_i_frame_interval: u8,
    /// 上次I帧序号
    pub last_i_frame_no: u16,
    /// 当前帧序号
    pub current_frame_no: u16,
}

impl Jt1078Header {
    /// 从字节数组解析帧头
    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        if data.len() < 16 {
            return None;
        }

        // 检查起始标识符 "01cd" (0x30 0x31 0x63 0x64)
        let start_flag = u32::from_be_bytes([data[0], data[1], data[2], data[3]]);
        if start_flag != 0x30316364 {
            warn!("Invalid JT1078 start flag: 0x{:08x}", start_flag);
            return None;
        }

        Some(Self {
            start_flag,
            data_type: data[4],
            logic_channel: (data[5] & 0x1F),
            frame_property: u16::from_be_bytes([data[6], data[7]]),
            timestamp: u32::from_be_bytes([data[8], data[9], data[10], data[11]]),
            last_i_frame_interval: data[12],
            last_i_frame_no: u16::from_be_bytes([data[13], data[14]]),
            current_frame_no: u16::from_be_bytes([data[14], data[15]]),
        })
    }

    /// 转换为字节数组
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(16);

        bytes.extend_from_slice(&self.start_flag.to_be_bytes());
        bytes.push(self.data_type);
        bytes.push(self.logic_channel & 0x1F);
        bytes.extend_from_slice(&self.frame_property.to_be_bytes());
        bytes.extend_from_slice(&self.timestamp.to_be_bytes());
        bytes.push(self.last_i_frame_interval);
        bytes.extend_from_slice(&self.last_i_frame_no.to_be_bytes());
        bytes.extend_from_slice(&self.current_frame_no.to_be_bytes());

        bytes
    }

    /// 获取视频数据类型
    pub fn get_video_data_type(&self) -> VideoDataType {
        VideoDataType::from(self.data_type)
    }
}

/// JT1078协议帧
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Jt1078Frame {
    /// 帧头
    pub header: Jt1078Header,
    /// 负载数据
    pub payload: Vec<u8>,
}

impl Jt1078Frame {
    /// 从字节数组解析帧
    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        if data.len() < 16 {
            return None;
        }

        let header = Jt1078Header::from_bytes(data)?;
        let payload = data[16..].to_vec();

        Some(Self { header, payload })
    }

    /// 转换为字节数组
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = self.header.to_bytes();
        bytes.extend_from_slice(&self.payload);
        bytes
    }

    /// 计算帧大小
    pub fn size(&self) -> usize {
        16 + self.payload.len()
    }
}

/// JT1078命令ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Jt1078Command {
    /// 终端通用应答
    TerminalAck = 0x0001,
    /// 终端心跳
    TerminalHeartbeat = 0x0002,
    /// 终端注册
    TerminalRegister = 0x0100,
    /// 终端注销
    TerminalUnregister = 0x0003,
    /// 平台通用应答
    PlatformAck = 0x8001,
    /// 平台心跳
    PlatformHeartbeat = 0x8002,
    /// 补传分包请求
    RetransmitRequest = 0x9202,
    /// 摄像头立即拍摄命令
    CameraCapture = 0x9201,
    /// 查询资源列表
    QueryResourceList = 0x9205,
    /// 文件上传完成通知
    FileUploadComplete = 0x1205,
    /// 请求上传音视频数据
    RequestUploadAV = 0x9200,
    /// 实时音视频传输请求
    RealtimeAVRequest = 0x9206,
    /// 历史音视频上传请求
    HistoryAVRequest = 0x9207,
    /// 停止实时音视频传输
    StopRealtimeAV = 0x9203,
    /// 停止历史音视频传输
    StopHistoryAV = 0x9204,
    /// 报警附件上传
    AlarmAttachmentUpload = 0x1210,
}

impl TryFrom<u16> for Jt1078Command {
    type Error = String;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            0x0001 => Ok(Jt1078Command::TerminalAck),
            0x0002 => Ok(Jt1078Command::TerminalHeartbeat),
            0x0100 => Ok(Jt1078Command::TerminalRegister),
            0x0003 => Ok(Jt1078Command::TerminalUnregister),
            0x8001 => Ok(Jt1078Command::PlatformAck),
            0x8002 => Ok(Jt1078Command::PlatformHeartbeat),
            0x9202 => Ok(Jt1078Command::RetransmitRequest),
            0x9201 => Ok(Jt1078Command::CameraCapture),
            0x9205 => Ok(Jt1078Command::QueryResourceList),
            0x1205 => Ok(Jt1078Command::FileUploadComplete),
            0x9200 => Ok(Jt1078Command::RequestUploadAV),
            0x9206 => Ok(Jt1078Command::RealtimeAVRequest),
            0x9207 => Ok(Jt1078Command::HistoryAVRequest),
            0x9203 => Ok(Jt1078Command::StopRealtimeAV),
            0x9204 => Ok(Jt1078Command::StopHistoryAV),
            0x1210 => Ok(Jt1078Command::AlarmAttachmentUpload),
            _ => Err(format!("Unknown JT1078 command: 0x{:04x}", value)),
        }
    }
}

impl Jt1078Command {
    /// 转换为u16
    pub fn as_u16(self) -> u16 {
        self as u16
    }
}

/// JT1078协议
pub struct Jt1078Protocol {
    version: Jt1078Version,
    channel_count: u8,
}

impl Default for Jt1078Protocol {
    fn default() -> Self {
        Self::new()
    }
}

impl Jt1078Protocol {
    /// 创建新的JT1078协议实例
    pub fn new() -> Self {
        Self {
            version: Jt1078Version::V2019,
            channel_count: 4, // 默认4个逻辑通道
        }
    }

    /// 设置协议版本
    pub fn with_version(mut self, version: Jt1078Version) -> Self {
        self.version = version;
        self
    }

    /// 设置通道数量
    pub fn with_channel_count(mut self, count: u8) -> Self {
        self.channel_count = count.min(16); // 最多16个通道
        self
    }

    /// 解析JT1078视频帧
    pub fn parse_frame(&self, data: &[u8]) -> Option<Jt1078Frame> {
        Jt1078Frame::from_bytes(data)
    }

    /// 解析JT1078命令
    pub fn parse_command(&self, data: &[u8]) -> Result<ProtocolData, ProtocolError> {
        if data.len() < 12 {
            return Err(ProtocolError::ParsingError(
                "Invalid JT1078 command length".to_string(),
            ));
        }

        // 检查起始标识符 "01cd"
        if data[0] != 0x30 || data[1] != 0x31 || data[2] != 0x63 || data[3] != 0x64 {
            return Err(ProtocolError::ParsingError(
                "Invalid JT1078 start flag".to_string(),
            ));
        }

        // 解析命令ID (字节5-6)
        let command_id = u16::from_be_bytes([data[4], data[5]]);

        // 解析消息体属性 (字节7-8)
        let body_property = u16::from_be_bytes([data[6], data[7]]);

        // 解析终端ID (字节9-12)
        let terminal_id_bytes = &data[8..12];
        let terminal_id = format!(
            "{:02x}{:02x}{:02x}{:02x}",
            terminal_id_bytes[0], terminal_id_bytes[1], terminal_id_bytes[2], terminal_id_bytes[3]
        );

        // 解析消息流水号 (字节13-14)
        let sequence_no = u16::from_be_bytes([data[12], data[13]]);

        // 解析消息包封装项 (字节15)
        let package_flag = data[14];

        let mut protocol_data = ProtocolData::new(terminal_id, format!("0x{:04x}", command_id))
            .with_raw_data(data.to_vec());

        protocol_data
            .params
            .insert("command_id".to_string(), format!("{:04x}", command_id));
        protocol_data.params.insert(
            "body_property".to_string(),
            format!("{:04x}", body_property),
        );
        protocol_data
            .params
            .insert("sequence_no".to_string(), sequence_no.to_string());
        protocol_data
            .params
            .insert("package_flag".to_string(), format!("{:02x}", package_flag));

        // 解析消息体
        if data.len() > 15 {
            let body = &data[15..];
            protocol_data
                .params
                .insert("body_length".to_string(), body.len().to_string());
            debug!("JT1078 command body: {:02x?}", body);
        }

        Ok(protocol_data)
    }

    /// 构建JT1078命令
    pub fn build_command(
        &self,
        terminal_id: &str,
        command_id: Jt1078Command,
        body: Option<&[u8]>,
        sequence_no: u16,
    ) -> Result<Vec<u8>, ProtocolError> {
        let mut packet = Vec::new();

        // 起始标识符 "01cd"
        packet.extend_from_slice(&[0x30, 0x31, 0x63, 0x64]);

        // 命令ID
        packet.extend_from_slice(&command_id.as_u16().to_be_bytes());

        // 消息体属性
        let body_len = body.map(|b| b.len()).unwrap_or(0);
        let body_property = if body_len > 0 { body_len as u16 } else { 0 };
        packet.extend_from_slice(&body_property.to_be_bytes());

        // 终端ID (4字节)
        let terminal_id_bytes = hex::decode(terminal_id)
            .map_err(|_| ProtocolError::ParsingError("Invalid terminal ID".to_string()))?;
        let terminal_id_fixed: Vec<u8> = terminal_id_bytes
            .iter()
            .take(4)
            .cloned()
            .chain(std::iter::repeat(0))
            .take(4)
            .collect();
        packet.extend_from_slice(&terminal_id_fixed);

        // 消息流水号
        packet.extend_from_slice(&sequence_no.to_be_bytes());

        // 消息包封装项
        packet.push(0x00); // 分包标志

        // 消息体
        if let Some(body) = body {
            packet.extend_from_slice(body);
        }

        Ok(packet)
    }
}

impl Protocol for Jt1078Protocol {
    fn parse(&self, data: &[u8]) -> Result<ProtocolData, ProtocolError> {
        debug!("Parsing JT1078 protocol data");

        // 检查是否是视频帧 (起始标识符 "01cd")
        if data.len() >= 4
            && data[0] == 0x30
            && data[1] == 0x31
            && data[2] == 0x63
            && data[3] == 0x64
        {
            // 尝试解析为视频帧
            if data.len() >= 16 {
                if let Some(frame) = self.parse_frame(data) {
                    let data_type = frame.header.get_video_data_type();
                    let protocol_data = ProtocolData::new(
                        format!("channel_{}", frame.header.logic_channel),
                        format!("video_{:?}", data_type),
                    )
                    .with_raw_data(data.to_vec());

                    return Ok(protocol_data);
                }
            }

            // 尝试解析为命令
            return self.parse_command(data);
        }

        Err(ProtocolError::ParsingError(
            "Invalid JT1078 protocol data".to_string(),
        ))
    }

    fn generate(&self, data: &ProtocolData) -> Result<Vec<u8>, ProtocolError> {
        debug!(
            "Generating JT1078 protocol data for command: {}",
            data.command
        );

        // 根据命令类型生成响应
        if let Ok(command_id) = u16::from_str_radix(data.command.trim_start_matches("0x"), 16) {
            let _command = Jt1078Command::try_from(command_id);
            // 从params中提取终端ID和流水号
            let terminal_id = data
                .params
                .get("terminal_id")
                .cloned()
                .unwrap_or_else(|| "00000000".to_string());
            let sequence_no = data
                .params
                .get("sequence_no")
                .and_then(|s| s.parse::<u16>().ok())
                .unwrap_or(0);

            // 生成平台应答
            return self.build_command(&terminal_id, Jt1078Command::PlatformAck, None, sequence_no);
        }

        Err(ProtocolError::UnsupportedCommand(format!(
            "Unsupported JT1078 command: {}",
            data.command
        )))
    }

    fn name(&self) -> &str {
        "JT1078"
    }

    fn version(&self) -> &str {
        match self.version {
            Jt1078Version::V2016 => "2016",
            Jt1078Version::V2019 => "2019",
        }
    }

    fn validate(&self, data: &[u8]) -> bool {
        // 检查起始标识符
        if data.len() < 4 {
            return false;
        }

        data[0] == 0x30 && data[1] == 0x31 && data[2] == 0x63 && data[3] == 0x64
    }

    fn supported_commands(&self) -> Vec<&str> {
        vec![
            "terminal_register",
            "terminal_heartbeat",
            "realtime_av_request",
            "history_av_request",
            "camera_capture",
            "query_resource_list",
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_video_data_type() {
        assert_eq!(VideoDataType::from(0x01), VideoDataType::IFrame);
        assert_eq!(VideoDataType::from(0x02), VideoDataType::PFrame);
        assert_eq!(VideoDataType::from(0x03), VideoDataType::BFrame);
        assert_eq!(VideoDataType::from(0x05), VideoDataType::AudioFrame);
        assert_eq!(VideoDataType::from(0xFF), VideoDataType::Unknown);
    }

    #[test]
    fn test_jt1078_header_serialization() {
        let header = Jt1078Header {
            start_flag: 0x30316364,
            data_type: 0x01,
            logic_channel: 0,
            frame_property: 0x0000,
            timestamp: 0x00000000,
            last_i_frame_interval: 0,
            last_i_frame_no: 0,
            current_frame_no: 0,
        };

        let bytes = header.to_bytes();
        let parsed = Jt1078Header::from_bytes(&bytes);

        assert!(parsed.is_some());
        let parsed = parsed.unwrap();
        assert_eq!(parsed.start_flag, header.start_flag);
        assert_eq!(parsed.data_type, header.data_type);
    }

    #[test]
    fn test_jt1078_command_conversion() {
        assert_eq!(
            Jt1078Command::try_from(0x0001),
            Ok(Jt1078Command::TerminalAck)
        );
        assert_eq!(
            Jt1078Command::try_from(0x0100),
            Ok(Jt1078Command::TerminalRegister)
        );
        assert_eq!(Jt1078Command::TerminalRegister.as_u16(), 0x0100);
    }
}
