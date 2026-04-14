//! / 协议构建器
use crate::truck_scale::protocol::bsj::BsjBuilder;
use crate::truck_scale::protocol::car_manager::CarManagerBuilder;
use crate::truck_scale::protocol::db44::Db44Builder;
use crate::truck_scale::protocol::gbt32960::Gbt32960Builder;
use crate::truck_scale::protocol::parser::ParsedMessage;
use crate::truck_scale::protocol::yw::YwBuilder;
use anyhow::Result;

/// 协议构建器
pub struct ProtocolBuilder {
    bsj_builder: BsjBuilder,
    yw_builder: YwBuilder,
    gbt_builder: Gbt32960Builder,
    db44_builder: Db44Builder,
    car_manager_builder: CarManagerBuilder,
}

impl ProtocolBuilder {
    /// 创建新的协议构建器
    pub fn new() -> Self {
        Self {
            bsj_builder: BsjBuilder::new(),
            yw_builder: YwBuilder::new(),
            gbt_builder: Gbt32960Builder::new(),
            db44_builder: Db44Builder::new(),
            car_manager_builder: CarManagerBuilder::new(),
        }
    }

    /// 构建响应
    pub fn build_response(&self, message: &ParsedMessage) -> Result<Vec<u8>> {
        match message {
            ParsedMessage::Bsj(msg) => self.bsj_builder.build_response(msg),
            ParsedMessage::Yw(msg) => self.yw_builder.build_response(msg),
            ParsedMessage::Gbt32960(msg) => self.gbt_builder.build_response(msg),
            ParsedMessage::Db44(msg) => self.db44_builder.build_response(msg),
            ParsedMessage::CarManager(msg) => self.car_manager_builder.build_response(msg),
            ParsedMessage::Unknown(data) => Ok(data.clone()),
        }
    }
}

impl Default for ProtocolBuilder {
    fn default() -> Self {
        Self::new()
    }
}
