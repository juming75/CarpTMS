//! /! 协议解析模块
//! 负责识别和解析各种车联网协议

use anyhow::Result;

/// 协议类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProtocolType {
    /// JT808协议
    JT808,
    /// Truck Scale协议 - BSJ
    TruckScaleBsj,
    /// Truck Scale协议 - YW
    TruckScaleYw,
    /// Truck Scale协议 - GB/T 32960
    TruckScaleGbt32960,
    /// Truck Scale协议 - DB44
    TruckScaleDb44,
    /// Truck Scale协议 - TF_CarManager
    TruckScaleCarManager,
    /// GPRS协议
    GPRS,
    /// 未知协议
    Unknown,
}

/// 解析后的消息
#[derive(Debug, Clone)]
pub enum ParsedMessage {
    /// JT808协议消息
    JT808(crate::protocols::jt808::models::JT808Frame),
    /// Truck Scale协议消息
    TruckScale(Box<crate::truck_scale::protocol::message_protocol::UnifiedMessage>),
    /// GPRS协议消息
    GPRS(Vec<u8>),
    /// 未知协议消息
    Unknown(Vec<u8>),
}

/// 协议解析器
pub struct ProtocolParser {
    jt808_parser: crate::protocols::jt808::parser::JT808Parser,
    truck_scale_parser: crate::truck_scale::protocol::parser::ProtocolParser,
}

// 手动实现Debug trait
impl std::fmt::Debug for ProtocolParser {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProtocolParser").finish()
    }
}

impl ProtocolParser {
    /// 创建新的协议解析器
    pub fn new() -> Self {
        Self {
            jt808_parser: crate::protocols::jt808::parser::JT808Parser,
            truck_scale_parser: crate::truck_scale::protocol::parser::ProtocolParser::new(),
        }
    }

    /// 检测协议类型
    pub fn detect_protocol(data: &[u8]) -> ProtocolType {
        if data.is_empty() {
            return ProtocolType::Unknown;
        }

        // 检测JT808协议 (0x7E)
        if data[0] == 0x7E && data.last() == Some(&0x7E) {
            return ProtocolType::JT808;
        }

        // 检测Truck Scale协议
        if let Some(ts_type) = Self::detect_truck_scale_protocol(data) {
            return ts_type;
        }

        // 检测GPRS协议(简单检测,实际需要根据具体GPRS协议格式调整)
        // 这里暂时假设GPRS协议有特定的格式特征
        if data.len() > 4 && data[0] == 0x01 && data[1] == 0x02 {
            return ProtocolType::GPRS;
        }

        ProtocolType::Unknown
    }

    /// 检测Truck Scale协议类型
    fn detect_truck_scale_protocol(data: &[u8]) -> Option<ProtocolType> {
        if data.len() < 2 {
            return None;
        }

        // 检测 BSJ 协议 (0x2D 0x2D)
        if data[0] == 0x2D && data[1] == 0x2D {
            return Some(ProtocolType::TruckScaleBsj);
        }

        // 检测 YW 协议 (0xAA)
        if data[0] == 0xAA {
            return Some(ProtocolType::TruckScaleYw);
        }

        // 检测 GB/T 32960 协议 (0x23)
        if data[0] == 0x23 {
            return Some(ProtocolType::TruckScaleGbt32960);
        }

        // 检测 TF_CarManager 协议 (0x78 0x56 0x34 0x12)
        if data.len() >= 4
            && data[0] == 0x78
            && data[1] == 0x56
            && data[2] == 0x34
            && data[3] == 0x12
        {
            return Some(ProtocolType::TruckScaleCarManager);
        }

        // 检测 DB44 协议 (0xDB 0x44)
        if data.len() >= 2 && data[0] == 0xDB && data[1] == 0x44 {
            return Some(ProtocolType::TruckScaleDb44);
        }

        None
    }

    /// 解析数据
    pub fn parse(&self, data: &[u8]) -> Result<ParsedMessage> {
        let protocol_type = Self::detect_protocol(data);

        match protocol_type {
            ProtocolType::JT808 => {
                let frame = crate::protocols::jt808::parser::JT808Parser::parse_frame(data)?;
                Ok(ParsedMessage::JT808(frame))
            }
            ProtocolType::TruckScaleBsj
            | ProtocolType::TruckScaleYw
            | ProtocolType::TruckScaleGbt32960
            | ProtocolType::TruckScaleDb44
            | ProtocolType::TruckScaleCarManager => {
                let message = self.truck_scale_parser.parse(data)?;
                // 转换为统一消息格式
                Ok(ParsedMessage::TruckScale(Box::new(
                    self.convert_to_unified_message(message)?,
                )))
            }
            ProtocolType::GPRS => Ok(ParsedMessage::GPRS(data.to_vec())),
            ProtocolType::Unknown => Ok(ParsedMessage::Unknown(data.to_vec())),
        }
    }

    /// 将Truck Scale消息转换为统一消息格式
    fn convert_to_unified_message(
        &self,
        _message: crate::truck_scale::protocol::parser::ParsedMessage,
    ) -> Result<crate::truck_scale::protocol::message_protocol::UnifiedMessage> {
        // 这里需要根据实际的Truck Scale消息格式进行转换
        // 暂时返回一个默认的统一消息
        Ok(
            crate::truck_scale::protocol::message_protocol::UnifiedMessage::new(
                crate::truck_scale::protocol::message_protocol::MessageType::Notification,
                crate::truck_scale::protocol::message_protocol::MessageBody::Notification(
                    crate::truck_scale::protocol::message_protocol::NotificationMessage {
                        notification_type: "truck_scale".to_string(),
                        title: "Truck Scale Message".to_string(),
                        content: "Truck Scale data received".to_string(),
                        data: None,
                    },
                ),
            ),
        )
    }
}

impl Default for ProtocolParser {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for ProtocolParser {
    fn clone(&self) -> Self {
        Self {
            jt808_parser: self.jt808_parser.clone(),
            truck_scale_parser: self.truck_scale_parser.clone(),
        }
    }
}
