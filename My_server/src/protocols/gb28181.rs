//! / GB28181-2016 视频监控联网协议实现
// 基于《公共安全视频监控联网系统信息传输、交换、控制技术要求》标准

use super::base::{Protocol, ProtocolData, ProtocolError};
use log::{debug, info};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;

/// GB28181协议版本
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GB28181Version {
    /// GB28181-2011版本
    V2011,
    /// GB28181-2016版本
    V2016,
}

/// SIP方法类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SipMethod {
    /// INVITE - 邀请(建立会话)
    Invite,
    /// ACK - 确认
    Ack,
    /// BYE - 结束会话
    Bye,
    /// REGISTER - 注册
    Register,
    /// MESSAGE - 消息
    Message,
    /// OPTIONS - 查询能力
    Options,
    /// INFO - 信息
    Info,
    /// SUBSCRIBE - 订阅
    Subscribe,
    /// NOTIFY - 通知
    Notify,
    /// UPDATE - 更新
    Update,
    /// CANCEL - 取消
    Cancel,
}

impl std::str::FromStr for SipMethod {
    type Err = ();

    /// 从字符串解析
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "INVITE" => Ok(SipMethod::Invite),
            "ACK" => Ok(SipMethod::Ack),
            "BYE" => Ok(SipMethod::Bye),
            "REGISTER" => Ok(SipMethod::Register),
            "MESSAGE" => Ok(SipMethod::Message),
            "OPTIONS" => Ok(SipMethod::Options),
            "INFO" => Ok(SipMethod::Info),
            "SUBSCRIBE" => Ok(SipMethod::Subscribe),
            "NOTIFY" => Ok(SipMethod::Notify),
            "UPDATE" => Ok(SipMethod::Update),
            "CANCEL" => Ok(SipMethod::Cancel),
            _ => Err(()),
        }
    }
}

impl SipMethod {
    /// 转换为字符串
    pub fn as_str(&self) -> &'static str {
        match self {
            SipMethod::Invite => "INVITE",
            SipMethod::Ack => "ACK",
            SipMethod::Bye => "BYE",
            SipMethod::Register => "REGISTER",
            SipMethod::Message => "MESSAGE",
            SipMethod::Options => "OPTIONS",
            SipMethod::Info => "INFO",
            SipMethod::Subscribe => "SUBSCRIBE",
            SipMethod::Notify => "NOTIFY",
            SipMethod::Update => "UPDATE",
            SipMethod::Cancel => "CANCEL",
        }
    }
}

/// GB28181设备类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GB28181DeviceType {
    /// IPC - 网络摄像机
    IPC,
    /// NVR - 网络录像机
    NVR,
    /// DVR - 数字录像机
    DVR,
    /// 解码器
    Decoder,
    /// 中心服务器
    Server,
    /// 未知类型
    Unknown,
}

impl GB28181DeviceType {
    /// 从设备ID解析
    pub fn from_device_id(device_id: &str) -> Self {
        if device_id.len() >= 20 {
            // GB28181设备ID为20位
            match &device_id[0..2] {
                "11" => GB28181DeviceType::IPC,
                "12" => GB28181DeviceType::NVR,
                "13" => GB28181DeviceType::DVR,
                "14" => GB28181DeviceType::Decoder,
                "21" => GB28181DeviceType::Server,
                _ => GB28181DeviceType::Unknown,
            }
        } else {
            GB28181DeviceType::Unknown
        }
    }
}

/// GB28181设备信息
#[derive(Debug, Clone)]
pub struct GB28181DeviceInfo {
    /// 设备ID (20位)
    pub device_id: String,
    /// 设备名称
    pub device_name: String,
    /// 设备类型
    pub device_type: GB28181DeviceType,
    /// 制造商ID
    pub manufacturer_id: String,
    /// 型号
    pub model: String,
    /// 固件版本
    pub firmware_version: String,
    /// IP地址
    pub ip_address: String,
    /// 端口
    pub port: u16,
    /// 在线状态
    pub online: bool,
    /// 最后心跳时间
    pub last_heartbeat: Option<u64>,
}

/// GB28181 SIP消息
#[derive(Debug, Clone)]
pub struct GB28181SipMessage {
    /// SIP方法
    pub method: SipMethod,
    /// From URI
    pub from: String,
    /// To URI
    pub to: String,
    /// Call-ID
    pub call_id: String,
    /// CSeq
    pub cseq: u32,
    /// Via headers
    pub via: Vec<String>,
    /// Content-Type
    pub content_type: Option<String>,
    /// Content-Length
    pub content_length: usize,
    /// SDP内容 (用于INVITE)
    pub sdp: Option<String>,
    /// 其他headers
    pub headers: HashMap<String, String>,
    /// 原始消息
    pub raw: String,
}

impl std::str::FromStr for GB28181SipMessage {
    type Err = ProtocolError;

    /// 从字符串解析SIP消息
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lines: Vec<&str> = s.lines().collect();
        if lines.is_empty() {
            return Err(ProtocolError::ParsingError("Empty SIP message".to_string()));
        }

        // 解析请求行
        let request_line = lines[0];
        let parts: Vec<&str> = request_line.split_whitespace().collect();
        if parts.len() < 2 {
            return Err(ProtocolError::ParsingError(
                "Invalid SIP request line".to_string(),
            ));
        }

        let method = SipMethod::from_str(parts[0]).map_err(|_| {
            ProtocolError::ParsingError(format!("Unknown SIP method: {}", parts[0]))
        })?;

        // 解析headers
        let mut via = Vec::new();
        let mut from = String::new();
        let mut to = String::new();
        let mut call_id = String::new();
        let mut cseq = 0u32;
        let mut content_type: Option<String> = None;
        let mut content_length = 0usize;
        let mut headers = HashMap::new();

        let mut body_start = 0;

        for (i, line) in lines.iter().enumerate().skip(1) {
            if line.is_empty() {
                body_start = i + 1;
                break;
            }

            if let Some((key, value)) = line.split_once(':') {
                let key = key.trim();
                let value = value.trim();
                headers.insert(key.to_lowercase(), value.to_string());

                match key.to_lowercase().as_str() {
                    "via" => via.push(value.to_string()),
                    "from" => from = value.to_string(),
                    "to" => to = value.to_string(),
                    "call-id" => call_id = value.to_string(),
                    "cseq" => {
                        if let Some(seq) = value.split_whitespace().next() {
                            cseq = seq.parse().unwrap_or(0);
                        }
                    }
                    "content-type" => content_type = Some(value.to_string()),
                    "content-length" => {
                        content_length = value.parse().unwrap_or(0);
                    }
                    _ => {}
                }
            }
        }

        // 解析SDP内容
        let mut sdp = None;
        if body_start < lines.len() && content_type.as_deref() == Some("application/sdp") {
            sdp = Some(lines[body_start..].join("\r\n"));
        }

        Ok(Self {
            method,
            from,
            to,
            call_id,
            cseq,
            via,
            content_type,
            content_length,
            sdp,
            headers,
            raw: s.to_string(),
        })
    }
}

impl GB28181SipMessage {
    /// 构建SIP请求消息
    pub fn build_request(&self) -> String {
        let mut message = String::new();

        // 请求行
        message.push_str(&format!("{} SIP/2.0\r\n", self.method.as_str()));

        // Via
        for via in &self.via {
            message.push_str(&format!("Via: {}\r\n", via));
        }

        // From
        message.push_str(&format!("From: {}\r\n", self.from));

        // To
        message.push_str(&format!("To: {}\r\n", self.to));

        // Call-ID
        message.push_str(&format!("Call-ID: {}\r\n", self.call_id));

        // CSeq
        message.push_str(&format!("CSeq: {} {}\r\n", self.cseq, self.method.as_str()));

        // Content-Type
        if let Some(content_type) = &self.content_type {
            message.push_str(&format!("Content-Type: {}\r\n", content_type));
        }

        // Content-Length
        message.push_str(&format!("Content-Length: {}\r\n", self.content_length));

        // 其他headers
        for (key, value) in &self.headers {
            match key.to_lowercase().as_str() {
                "via" | "from" | "to" | "call-id" | "cseq" | "content-type" | "content-length" => {}
                _ => {
                    message.push_str(&format!("{}: {}\r\n", key, value));
                }
            }
        }

        // SDP内容
        if let Some(sdp) = &self.sdp {
            message.push_str("\r\n");
            message.push_str(sdp);
        }

        message
    }
}

/// GB28181控制命令类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GB28181ControlCommand {
    /// 云台控制
    PTZ,
    /// 预置位设置
    PresetSet,
    /// 预置位调用
    PresetCall,
    /// 报警布防
    AlarmSet,
    /// 报警撤防
    AlarmReset,
    /// 录像控制
    Record,
    /// 抓拍
    Capture,
    /// 重启设备
    Reboot,
    /// 设备配置
    Config,
}

/// GB28181协议
pub struct GB28181Protocol {
    version: GB28181Version,
    sip_domain: String,
    sip_port: u16,
    devices: HashMap<String, GB28181DeviceInfo>,
}

impl Default for GB28181Protocol {
    fn default() -> Self {
        Self::new()
    }
}

impl GB28181Protocol {
    /// 创建新的GB28181协议实例
    pub fn new() -> Self {
        Self {
            version: GB28181Version::V2016,
            sip_domain: "3402000000".to_string(), // 默认域编码
            sip_port: 5060,
            devices: HashMap::new(),
        }
    }

    /// 设置协议版本
    pub fn with_version(mut self, version: GB28181Version) -> Self {
        self.version = version;
        self
    }

    /// 设置SIP域
    pub fn with_sip_domain(mut self, domain: String) -> Self {
        self.sip_domain = domain;
        self
    }

    /// 设置SIP端口
    pub fn with_sip_port(mut self, port: u16) -> Self {
        self.sip_port = port;
        self
    }

    /// 添加设备
    pub fn add_device(&mut self, device: GB28181DeviceInfo) {
        self.devices.insert(device.device_id.clone(), device);
    }

    /// 获取设备信息
    pub fn get_device(&self, device_id: &str) -> Option<&GB28181DeviceInfo> {
        self.devices.get(device_id)
    }

    /// 移除设备
    pub fn remove_device(&mut self, device_id: &str) {
        self.devices.remove(device_id);
    }

    /// 获取所有设备
    pub fn get_all_devices(&self) -> Vec<&GB28181DeviceInfo> {
        self.devices.values().collect()
    }

    /// 构建设备注册请求
    pub fn build_register_request(
        &self,
        device_id: &str,
        sequence: u32,
        expires: u32,
    ) -> Result<String, ProtocolError> {
        let from = format!("<sip:{}@{}>", device_id, self.sip_domain);
        let to = format!("<sip:{}@{}>", device_id, self.sip_domain);
        let call_id = format!("{}@{}", device_id, self.sip_domain);

        let message = GB28181SipMessage {
            method: SipMethod::Register,
            from,
            to,
            call_id,
            cseq: sequence,
            via: vec![format!("SIP/2.0/UDP {}:{}", self.sip_domain, self.sip_port)],
            content_type: None,
            content_length: 0,
            sdp: None,
            headers: {
                let mut map = HashMap::new();
                map.insert("Expires".to_string(), expires.to_string());
                map.insert("User-Agent".to_string(), "TMS Server".to_string());
                map.insert(
                    "Contact".to_string(),
                    format!("<sip:{}@{}:{}>", device_id, self.sip_domain, self.sip_port),
                );
                map
            },
            raw: String::new(),
        };

        Ok(message.build_request())
    }

    /// 构建设备注销请求
    pub fn build_unregister_request(
        &self,
        device_id: &str,
        sequence: u32,
    ) -> Result<String, ProtocolError> {
        let mut register_request = self.build_register_request(device_id, sequence, 0)?;
        // 注销时Expires=0
        register_request = register_request.replace(&format!("Expires: {}", 0), "Expires: 0");
        Ok(register_request)
    }

    /// 构建实时视频邀请请求
    pub fn build_invite_request(
        &self,
        device_id: &str,
        _channel_id: u8,
        sequence: u32,
        sdp: &str,
    ) -> Result<String, ProtocolError> {
        let from = format!("<sip:{}@{}>", device_id, self.sip_domain);
        let to = format!("<sip:{}@{}>", device_id, self.sip_domain);
        let call_id = format!("{}@{}", device_id, self.sip_domain);

        let message = GB28181SipMessage {
            method: SipMethod::Invite,
            from,
            to,
            call_id,
            cseq: sequence,
            via: vec![format!("SIP/2.0/UDP {}:{}", self.sip_domain, self.sip_port)],
            content_type: Some("application/sdp".to_string()),
            content_length: sdp.len(),
            sdp: Some(sdp.to_string()),
            headers: {
                let mut map = HashMap::new();
                map.insert("User-Agent".to_string(), "TMS Server".to_string());
                map.insert(
                    "Contact".to_string(),
                    format!("<sip:{}@{}:{}>", device_id, self.sip_domain, self.sip_port),
                );
                map
            },
            raw: String::new(),
        };

        Ok(message.build_request())
    }

    /// 构建云台控制指令
    pub fn build_ptz_command(
        &self,
        device_id: &str,
        channel_id: u8,
        command: GB28181ControlCommand,
        params: &HashMap<String, String>,
    ) -> Result<Vec<u8>, ProtocolError> {
        // GB28181云台控制通过MANSCDP协议封装
        let mut data = Vec::new();

        // 设备ID (20字节)
        let device_id_bytes: Vec<u8> = device_id
            .bytes()
            .take(20)
            .chain(std::iter::repeat(0))
            .take(20)
            .collect();
        data.extend_from_slice(&device_id_bytes);

        // 命令类型
        data.push(match command {
            GB28181ControlCommand::PTZ => 0x01,
            GB28181ControlCommand::PresetSet => 0x02,
            GB28181ControlCommand::PresetCall => 0x03,
            GB28181ControlCommand::AlarmSet => 0x04,
            GB28181ControlCommand::AlarmReset => 0x05,
            GB28181ControlCommand::Record => 0x06,
            GB28181ControlCommand::Capture => 0x07,
            GB28181ControlCommand::Reboot => 0x08,
            GB28181ControlCommand::Config => 0x09,
        });

        // 通道号
        data.push(channel_id);

        // 参数
        if let Some(speed) = params.get("speed") {
            if let Ok(s) = speed.parse::<u8>() {
                data.push(s);
            }
        }

        Ok(data)
    }
}

impl Protocol for GB28181Protocol {
    fn parse(&self, data: &[u8]) -> Result<ProtocolData, ProtocolError> {
        debug!("Parsing GB28181 protocol data");

        // GB28181使用SIP协议,数据应该是文本格式
        if data.is_empty() {
            return Err(ProtocolError::ParsingError(
                "Empty GB28181 data".to_string(),
            ));
        }

        // 尝试解析为SIP消息
        let message_str = String::from_utf8_lossy(data);
        match GB28181SipMessage::from_str(&message_str) {
            Ok(sip_message) => {
                // 提取设备ID
                let device_id = sip_message
                    .from
                    .strip_prefix("<sip:")
                    .and_then(|s| s.strip_suffix('>'))
                    .and_then(|s| s.split('@').next())
                    .unwrap_or("unknown")
                    .to_string();

                let method_str = format!("{:?}", sip_message.method);
                let mut protocol_data = ProtocolData::new(device_id.to_string(), method_str)
                    .with_raw_data(data.to_vec());

                protocol_data
                    .params
                    .insert("call_id".to_string(), sip_message.call_id.clone());
                protocol_data
                    .params
                    .insert("cseq".to_string(), sip_message.cseq.to_string());

                // 如果是INVITE消息且包含SDP,解析SDP
                if sip_message.method == SipMethod::Invite {
                    if let Some(sdp) = &sip_message.sdp {
                        protocol_data.params.insert("sdp".to_string(), sdp.clone());
                        info!("GB28181 INVITE with SDP received");
                    }
                }

                Ok(protocol_data)
            }
            Err(e) => Err(ProtocolError::ParsingError(format!(
                "Failed to parse SIP message: {}",
                e
            ))),
        }
    }

    fn generate(&self, data: &ProtocolData) -> Result<Vec<u8>, ProtocolError> {
        debug!(
            "Generating GB28181 protocol data for command: {}",
            data.command
        );

        // 根据命令类型生成响应
        match data.command.to_lowercase().as_str() {
            "register" => {
                let device_id = data
                    .params
                    .get("device_id")
                    .cloned()
                    .unwrap_or_else(|| "00000000000000000000".to_string());
                let sequence = data
                    .params
                    .get("sequence")
                    .and_then(|s| s.parse::<u32>().ok())
                    .unwrap_or(1);
                let expires = data
                    .params
                    .get("expires")
                    .and_then(|s| s.parse::<u32>().ok())
                    .unwrap_or(3600);

                let message = self.build_register_request(&device_id, sequence, expires)?;
                Ok(message.into_bytes())
            }
            "invite" => {
                let device_id = data
                    .params
                    .get("device_id")
                    .cloned()
                    .unwrap_or_else(|| "00000000000000000000".to_string());
                let channel_id = data
                    .params
                    .get("channel_id")
                    .and_then(|s| s.parse::<u8>().ok())
                    .unwrap_or(1);
                let sequence = data
                    .params
                    .get("sequence")
                    .and_then(|s| s.parse::<u32>().ok())
                    .unwrap_or(1);
                let sdp = data.params.get("sdp").cloned().unwrap_or_else(String::new);

                let message = self.build_invite_request(&device_id, channel_id, sequence, &sdp)?;
                Ok(message.into_bytes())
            }
            _ => Err(ProtocolError::UnsupportedCommand(format!(
                "Unsupported GB28181 command: {}",
                data.command
            ))),
        }
    }

    fn name(&self) -> &str {
        "GB28181"
    }

    fn version(&self) -> &str {
        match self.version {
            GB28181Version::V2011 => "2011",
            GB28181Version::V2016 => "2016",
        }
    }

    fn validate(&self, data: &[u8]) -> bool {
        if data.is_empty() {
            return false;
        }

        // 检查是否以SIP方法开头
        let message_str = String::from_utf8_lossy(data);
        let first_line = message_str.lines().next().unwrap_or("");
        let methods = ["INVITE", "ACK", "BYE", "REGISTER", "MESSAGE", "OPTIONS"];
        methods.iter().any(|m| first_line.starts_with(m))
    }

    fn supported_commands(&self) -> Vec<&str> {
        vec![
            "register",
            "unregister",
            "invite",
            "bye",
            "message",
            "info",
            "subscribe",
            "notify",
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sip_method_from_str() {
        assert_eq!(SipMethod::from_str("INVITE"), Ok(SipMethod::Invite));
        assert_eq!(SipMethod::from_str("register"), Ok(SipMethod::Register));
        assert!(SipMethod::from_str("UNKNOWN").is_err());
    }

    #[test]
    fn test_device_type_from_id() {
        assert_eq!(
            GB28181DeviceType::from_device_id("11123456789012345678"),
            GB28181DeviceType::IPC
        );
        assert_eq!(
            GB28181DeviceType::from_device_id("12123456789012345678"),
            GB28181DeviceType::NVR
        );
        assert_eq!(
            GB28181DeviceType::from_device_id("21123456789012345678"),
            GB28181DeviceType::Server
        );
    }

    #[test]
    fn test_build_register_request() {
        let protocol = GB28181Protocol::new();
        let request = protocol.build_register_request("11123456789012345678", 1, 3600);

        assert!(request.is_ok());
        let request = request.unwrap();
        assert!(request.contains("REGISTER"));
        assert!(request.contains("11123456789012345678"));
        assert!(request.contains("Expires: 3600"));
    }
}
