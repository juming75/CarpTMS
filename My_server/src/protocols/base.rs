//! / 协议基础模块
// 定义所有协议都要实现的基本特性

use std::collections::HashMap;
use std::error::Error;
use std::fmt;

// 协议数据
#[derive(Clone)]
pub struct ProtocolData {
    pub device_id: String,
    pub command: String,
    pub params: HashMap<String, String>,
    pub raw_data: Vec<u8>,
    pub timestamp: std::time::SystemTime,
}

impl ProtocolData {
    pub fn new(device_id: String, command: String) -> Self {
        Self {
            device_id,
            command,
            params: HashMap::new(),
            raw_data: Vec::new(),
            timestamp: std::time::SystemTime::now(),
        }
    }

    pub fn with_raw_data(mut self, raw_data: Vec<u8>) -> Self {
        self.raw_data = raw_data;
        self
    }

    pub fn with_param(mut self, key: &str, value: &str) -> Self {
        self.params.insert(key.to_string(), value.to_string());
        self
    }
}

// 协议错误
#[derive(Debug, Clone)]
pub enum ProtocolError {
    ParsingError(String),
    ValidationError(String),
    UnknownProtocol(String),
    ProtocolDetectionFailed,
    UnsupportedCommand(String),
    EncodingError(String),
}

impl fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProtocolError::ParsingError(msg) => write!(f, "Parsing error: {}", msg),
            ProtocolError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            ProtocolError::UnknownProtocol(protocol) => write!(f, "Unknown protocol: {}", protocol),
            ProtocolError::ProtocolDetectionFailed => write!(f, "Failed to detect protocol"),
            ProtocolError::UnsupportedCommand(cmd) => write!(f, "Unsupported command: {}", cmd),
            ProtocolError::EncodingError(msg) => write!(f, "Encoding error: {}", msg),
        }
    }
}

impl Error for ProtocolError {}

// 协议特性
pub trait Protocol: Send + Sync + 'static {
    // 解析协议数据
    fn parse(&self, data: &[u8]) -> Result<ProtocolData, ProtocolError>;

    // 生成协议数据
    fn generate(&self, data: &ProtocolData) -> Result<Vec<u8>, ProtocolError>;

    // 获取协议名称
    fn name(&self) -> &str;

    // 获取协议版本
    fn version(&self) -> &str;

    // 验证协议数据
    fn validate(&self, data: &[u8]) -> bool;

    // 获取支持的命令列表
    fn supported_commands(&self) -> Vec<&str>;

    // 检查命令是否支持
    fn supports_command(&self, command: &str) -> bool {
        self.supported_commands().contains(&command)
    }
}

// 协议事件处理器
pub trait ProtocolEventHandler: Send + Sync + 'static {
    // 处理设备连接事件
    fn handle_device_connected(
        &self,
        device_id: &str,
        protocol: &str,
    ) -> Result<(), Box<dyn Error>>;

    // 处理设备断开连接事件
    fn handle_device_disconnected(
        &self,
        device_id: &str,
        protocol: &str,
        reason: &str,
    ) -> Result<(), Box<dyn Error>>;

    // 处理设备数据事件
    fn handle_device_data(&self, data: &ProtocolData) -> Result<(), Box<dyn Error>>;

    // 处理命令响应事件
    fn handle_command_response(
        &self,
        device_id: &str,
        command: &str,
        success: bool,
        result: &str,
    ) -> Result<(), Box<dyn Error>>;
}
