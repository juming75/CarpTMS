//! / 协议模块
// 负责定义和实现多种协议的解析和生成

pub mod base;
pub mod bsj;
pub mod coap;
pub mod db44;
pub mod db_protocol;
pub mod gb;
pub mod gb28181;
pub mod http;
pub mod jt1078;
pub mod jt808;
pub mod mqtt;

use log::{error, info};
use std::collections::HashMap;

// 协议类型枚举
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ProtocolType {
    GB,
    BSJ,
    DB44,
    DB,
    HTTP,
    MQTT,
    CoAP,
    GB28181,
    JT1078,
    JT808,
    Unknown(String),
}

impl From<&str> for ProtocolType {
    fn from(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "GB" => ProtocolType::GB,
            "BSJ" => ProtocolType::BSJ,
            "DB44" => ProtocolType::DB44,
            "DB" => ProtocolType::DB,
            "HTTP" => ProtocolType::HTTP,
            "MQTT" => ProtocolType::MQTT,
            "COAP" => ProtocolType::CoAP,
            "GB28181" => ProtocolType::GB28181,
            "JT1078" => ProtocolType::JT1078,
            "JT808" => ProtocolType::JT808,
            _ => ProtocolType::Unknown(s.to_string()),
        }
    }
}

impl std::fmt::Display for ProtocolType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProtocolType::GB => write!(f, "GB"),
            ProtocolType::BSJ => write!(f, "BSJ"),
            ProtocolType::DB44 => write!(f, "DB44"),
            ProtocolType::DB => write!(f, "DB"),
            ProtocolType::HTTP => write!(f, "HTTP"),
            ProtocolType::MQTT => write!(f, "MQTT"),
            ProtocolType::CoAP => write!(f, "CoAP"),
            ProtocolType::GB28181 => write!(f, "GB28181"),
            ProtocolType::JT1078 => write!(f, "JT1078"),
            ProtocolType::JT808 => write!(f, "JT808"),
            ProtocolType::Unknown(s) => write!(f, "{}", s),
        }
    }
}

// 协议工厂
pub struct ProtocolFactory {
    protocols: HashMap<ProtocolType, Box<dyn base::Protocol>>,
}

impl Default for ProtocolFactory {
    fn default() -> Self {
        Self::new()
    }
}

impl ProtocolFactory {
    pub fn new() -> Self {
        let mut protocols: HashMap<ProtocolType, Box<dyn base::Protocol>> = HashMap::new();

        // 注册各种协议
        protocols.insert(
            ProtocolType::GB,
            Box::new(gb::GBProtocol::new()) as Box<dyn base::Protocol>,
        );
        protocols.insert(
            ProtocolType::BSJ,
            Box::new(bsj::BSJProtocol::new()) as Box<dyn base::Protocol>,
        );
        protocols.insert(
            ProtocolType::DB44,
            Box::new(db44::DB44Protocol::new()) as Box<dyn base::Protocol>,
        );
        protocols.insert(
            ProtocolType::DB,
            Box::new(db_protocol::DbProtocol::new()) as Box<dyn base::Protocol>,
        );
        protocols.insert(
            ProtocolType::HTTP,
            Box::new(http::HttpProtocol::new()) as Box<dyn base::Protocol>,
        );
        protocols.insert(
            ProtocolType::MQTT,
            Box::new(mqtt::MqttProtocol::new()) as Box<dyn base::Protocol>,
        );
        protocols.insert(
            ProtocolType::CoAP,
            Box::new(coap::CoapProtocol::new()) as Box<dyn base::Protocol>,
        );
        protocols.insert(
            ProtocolType::GB28181,
            Box::new(gb28181::GB28181Protocol::new()) as Box<dyn base::Protocol>,
        );
        protocols.insert(
            ProtocolType::JT1078,
            Box::new(jt1078::Jt1078Protocol::new()) as Box<dyn base::Protocol>,
        );

        Self { protocols }
    }

    // 获取协议实例
    pub fn get_protocol(&self, protocol_type: &ProtocolType) -> Option<&dyn base::Protocol> {
        self.protocols.get(protocol_type).map(|v| &**v)
    }

    // 解析数据
    pub fn parse_data(
        &self,
        protocol_type: &ProtocolType,
        data: &[u8],
    ) -> Result<base::ProtocolData, base::ProtocolError> {
        if let Some(protocol) = self.get_protocol(protocol_type) {
            protocol.parse(data)
        } else {
            Err(base::ProtocolError::UnknownProtocol(
                protocol_type.to_string(),
            ))
        }
    }

    // 生成数据
    pub fn generate_data(
        &self,
        protocol_type: &ProtocolType,
        data: &base::ProtocolData,
    ) -> Result<Vec<u8>, base::ProtocolError> {
        if let Some(protocol) = self.get_protocol(protocol_type) {
            protocol.generate(data)
        } else {
            Err(base::ProtocolError::UnknownProtocol(
                protocol_type.to_string(),
            ))
        }
    }

    // 自动识别协议并解析
    pub fn auto_parse(
        &self,
        data: &[u8],
    ) -> Result<(ProtocolType, base::ProtocolData), base::ProtocolError> {
        // 尝试所有协议解析,返回第一个成功的
        for (protocol_type, protocol) in &self.protocols {
            match protocol.parse(data) {
                Ok(protocol_data) => {
                    info!("Auto-detected protocol: {}", protocol_type);
                    return Ok((protocol_type.clone(), protocol_data));
                }
                Err(_) => {
                    // 解析失败,继续尝试下一个协议
                }
            }
        }

        error!("Failed to auto-detect protocol for data: {:?}", data);
        Err(base::ProtocolError::ProtocolDetectionFailed)
    }
}

// 为ProtocolFactory添加Clone实现
impl Clone for ProtocolFactory {
    fn clone(&self) -> Self {
        // 由于Box<dyn Protocol>不支持Clone,我们需要重新创建协议实例
        let mut protocols: HashMap<ProtocolType, Box<dyn base::Protocol>> = HashMap::new();

        // 重新注册各种协议
        protocols.insert(
            ProtocolType::GB,
            Box::new(gb::GBProtocol::new()) as Box<dyn base::Protocol>,
        );
        protocols.insert(
            ProtocolType::BSJ,
            Box::new(bsj::BSJProtocol::new()) as Box<dyn base::Protocol>,
        );
        protocols.insert(
            ProtocolType::DB44,
            Box::new(db44::DB44Protocol::new()) as Box<dyn base::Protocol>,
        );
        protocols.insert(
            ProtocolType::DB,
            Box::new(db_protocol::DbProtocol::new()) as Box<dyn base::Protocol>,
        );
        protocols.insert(
            ProtocolType::HTTP,
            Box::new(http::HttpProtocol::new()) as Box<dyn base::Protocol>,
        );
        protocols.insert(
            ProtocolType::MQTT,
            Box::new(mqtt::MqttProtocol::new()) as Box<dyn base::Protocol>,
        );
        protocols.insert(
            ProtocolType::CoAP,
            Box::new(coap::CoapProtocol::new()) as Box<dyn base::Protocol>,
        );
        protocols.insert(
            ProtocolType::GB28181,
            Box::new(gb28181::GB28181Protocol::new()) as Box<dyn base::Protocol>,
        );
        protocols.insert(
            ProtocolType::JT1078,
            Box::new(jt1078::Jt1078Protocol::new()) as Box<dyn base::Protocol>,
        );

        Self { protocols }
    }
}
