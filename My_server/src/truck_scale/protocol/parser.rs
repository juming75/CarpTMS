//! / 协议解析器
use crate::truck_scale::protocol::bsj::BsjParser;
use crate::truck_scale::protocol::car_manager::CarManagerParser;
use crate::truck_scale::protocol::db44::Db44Parser;
use crate::truck_scale::protocol::gbt32960::Gbt32960Parser;
use crate::truck_scale::protocol::yw::YwParser;
use anyhow::Result;

/// 解析后的消息
#[derive(Debug, Clone)]
pub enum ParsedMessage {
    /// BSJ 协议消息
    Bsj(crate::truck_scale::protocol::bsj::BsjMessage),

    /// YW 协议消息
    Yw(crate::truck_scale::protocol::yw::YwMessage),

    /// GB/T 32960 协议消息
    Gbt32960(crate::truck_scale::protocol::gbt32960::Gbt32960Message),

    /// DB44 协议消息
    Db44(crate::truck_scale::protocol::db44::Db44Message),

    /// TF_CarManager 协议消息
    CarManager(crate::truck_scale::protocol::car_manager::CarManagerMessage),

    /// 未知协议
    Unknown(Vec<u8>),
}

/// 协议类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProtocolType {
    Bsj,
    Yw,
    Gbt32960,
    Db44,
    CarManager,
    Unknown,
}

/// 协议路由器
pub struct ProtocolParser {
    bsj_parser: BsjParser,
    yw_parser: YwParser,
    gbt_parser: Gbt32960Parser,
    db44_parser: Db44Parser,
    car_manager_parser: CarManagerParser,
}

impl ProtocolParser {
    /// 创建新的协议解析器
    pub fn new() -> Self {
        Self {
            bsj_parser: BsjParser::new(),
            yw_parser: YwParser::new(),
            gbt_parser: Gbt32960Parser::new(),
            db44_parser: Db44Parser::new(),
            car_manager_parser: CarManagerParser::new(),
        }
    }

    /// 解析协议类型
    fn detect_protocol_type(data: &[u8]) -> ProtocolType {
        if data.len() < 2 {
            return ProtocolType::Unknown;
        }

        // 检测 BSJ 协议 (0x2D 0x2D)
        if data[0] == 0x2D && data[1] == 0x2D {
            return ProtocolType::Bsj;
        }

        // 检测 YW 协议 (0xAA)
        if data[0] == 0xAA {
            return ProtocolType::Yw;
        }

        // 检测 GB/T 32960 协议 (0x23)
        if data[0] == 0x23 {
            return ProtocolType::Gbt32960;
        }

        // 检测 TF_CarManager 协议 (0x78 0x56 0x34 0x12)
        if data.len() >= 4
            && data[0] == 0x78
            && data[1] == 0x56
            && data[2] == 0x34
            && data[3] == 0x12
        {
            return ProtocolType::CarManager;
        }

        // 检测 DB44 协议 (0xDB 0x44)
        if data.len() >= 2 && data[0] == 0xDB && data[1] == 0x44 {
            return ProtocolType::Db44;
        }

        ProtocolType::Unknown
    }

    /// 解析数据
    pub fn parse(&self, data: &[u8]) -> Result<ParsedMessage> {
        let protocol_type = Self::detect_protocol_type(data);

        match protocol_type {
            ProtocolType::Bsj => {
                let message = self.bsj_parser.parse(data)?;
                Ok(ParsedMessage::Bsj(message))
            }
            ProtocolType::Yw => {
                let message = self.yw_parser.parse(data)?;
                Ok(ParsedMessage::Yw(message))
            }
            ProtocolType::Gbt32960 => {
                let message = self.gbt_parser.parse(data)?;
                Ok(ParsedMessage::Gbt32960(message))
            }
            ProtocolType::Db44 => {
                let message = self.db44_parser.parse(data)?;
                Ok(ParsedMessage::Db44(message))
            }
            ProtocolType::CarManager => {
                let message = self.car_manager_parser.parse(data)?;
                Ok(ParsedMessage::CarManager(message))
            }
            ProtocolType::Unknown => Ok(ParsedMessage::Unknown(data.to_vec())),
        }
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
            bsj_parser: self.bsj_parser.clone(),
            yw_parser: self.yw_parser.clone(),
            gbt_parser: self.gbt_parser.clone(),
            db44_parser: self.db44_parser.clone(),
            car_manager_parser: self.car_manager_parser.clone(),
        }
    }
}
