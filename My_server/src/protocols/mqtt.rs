//! / MQTT协议实现
// 基于MQTT 3.1.1协议规范

use super::base::{Protocol, ProtocolData, ProtocolError};
use log::debug;

// MQTT控制包类型
#[derive(Debug)]
enum MqttPacketType {
    Connect = 1,
    ConnAck = 2,
    Publish = 3,
    PubAck = 4,
    PubRec = 5,
    PubRel = 6,
    PubComp = 7,
    Subscribe = 8,
    SubAck = 9,
    Unsubscribe = 10,
    UnsubAck = 11,
    PingReq = 12,
    PingResp = 13,
    Disconnect = 14,
}

pub struct MqttProtocol;

impl Default for MqttProtocol {
    fn default() -> Self {
        Self::new()
    }
}

impl MqttProtocol {
    pub fn new() -> Self {
        Self
    }

    // 解析MQTT控制包类型
    fn parse_packet_type(&self, first_byte: u8) -> Result<MqttPacketType, ProtocolError> {
        let packet_type = (first_byte >> 4) & 0x0F;
        match packet_type {
            1 => Ok(MqttPacketType::Connect),
            2 => Ok(MqttPacketType::ConnAck),
            3 => Ok(MqttPacketType::Publish),
            4 => Ok(MqttPacketType::PubAck),
            5 => Ok(MqttPacketType::PubRec),
            6 => Ok(MqttPacketType::PubRel),
            7 => Ok(MqttPacketType::PubComp),
            8 => Ok(MqttPacketType::Subscribe),
            9 => Ok(MqttPacketType::SubAck),
            10 => Ok(MqttPacketType::Unsubscribe),
            11 => Ok(MqttPacketType::UnsubAck),
            12 => Ok(MqttPacketType::PingReq),
            13 => Ok(MqttPacketType::PingResp),
            14 => Ok(MqttPacketType::Disconnect),
            _ => Err(ProtocolError::ParsingError(format!(
                "Unknown MQTT packet type: {}",
                packet_type
            ))),
        }
    }

    // 解析MQTT剩余长度
    fn parse_remaining_length(&self, data: &[u8]) -> Result<(usize, usize), ProtocolError> {
        let mut multiplier = 1;
        let mut value = 0;
        let mut index = 1;

        loop {
            if index >= data.len() {
                return Err(ProtocolError::ParsingError(
                    "Invalid MQTT remaining length".to_string(),
                ));
            }

            let byte = data[index];
            value += ((byte & 0x7F) as usize) * multiplier;

            if (byte & 0x80) == 0 {
                break;
            }

            multiplier *= 128;
            if multiplier > 128 * 128 * 128 {
                return Err(ProtocolError::ParsingError(
                    "Invalid MQTT remaining length".to_string(),
                ));
            }

            index += 1;
        }

        Ok((value, index + 1))
    }

    // 解析MQTT字符串
    fn parse_mqtt_string(
        &self,
        data: &[u8],
        offset: usize,
    ) -> Result<(String, usize), ProtocolError> {
        if offset + 2 > data.len() {
            return Err(ProtocolError::ParsingError(
                "Invalid MQTT string".to_string(),
            ));
        }

        let length = ((data[offset] as u16) << 8) | (data[offset + 1] as u16);
        let end = offset + 2 + length as usize;

        if end > data.len() {
            return Err(ProtocolError::ParsingError(
                "Invalid MQTT string length".to_string(),
            ));
        }

        let s = String::from_utf8(data[offset + 2..end].to_vec()).map_err(|e| {
            ProtocolError::ParsingError(format!("Invalid UTF-8 in MQTT string: {}", e))
        })?;

        Ok((s, end))
    }

    // 解析CONNECT数据包
    fn parse_connect(&self, data: &[u8], offset: usize) -> Result<ProtocolData, ProtocolError> {
        // 解析协议名称
        let (protocol_name, offset) = self.parse_mqtt_string(data, offset)?;

        // 解析协议级别
        if offset >= data.len() {
            return Err(ProtocolError::ParsingError(
                "Invalid MQTT CONNECT packet".to_string(),
            ));
        }
        let protocol_level = data[offset];

        // 解析连接标志
        let offset = offset + 1;
        if offset >= data.len() {
            return Err(ProtocolError::ParsingError(
                "Invalid MQTT CONNECT packet".to_string(),
            ));
        }
        let connect_flags = data[offset];

        // 解析保持连接
        let offset = offset + 1;
        if offset + 1 >= data.len() {
            return Err(ProtocolError::ParsingError(
                "Invalid MQTT CONNECT packet".to_string(),
            ));
        }
        let keep_alive = ((data[offset] as u16) << 8) | (data[offset + 1] as u16);

        // 解析客户端ID
        let (client_id, offset) = self.parse_mqtt_string(data, offset + 2)?;

        // 解析用户名和密码(如果存在)
        let has_username = (connect_flags & 0x80) != 0;
        let has_password = (connect_flags & 0x40) != 0;
        let has_will = (connect_flags & 0x04) != 0;

        let mut username = None;
        let mut password = None;

        // 解析Will Topic和Will Message(如果存在)
        if has_will {
            let (_will_topic, _new_offset) = self.parse_mqtt_string(data, offset)?;

            let (_will_message, _new_offset) = self.parse_mqtt_string(data, offset)?;
        }

        // 解析用户名(如果存在)
        if has_username {
            let (user, _new_offset) = self.parse_mqtt_string(data, offset)?;
            username = Some(user);
        }

        // 解析密码(如果存在)
        if has_password {
            let (pass, _new_offset) = self.parse_mqtt_string(data, offset)?;
            password = Some(pass);
        }

        // 创建协议数据
        let mut protocol_data = ProtocolData::new(client_id.clone(), "mqtt_connect".to_string())
            .with_raw_data(data.to_vec());

        // 添加参数
        protocol_data
            .params
            .insert("protocol_name".to_string(), protocol_name);
        protocol_data
            .params
            .insert("protocol_level".to_string(), protocol_level.to_string());
        protocol_data
            .params
            .insert("keep_alive".to_string(), keep_alive.to_string());

        if let Some(user) = username {
            protocol_data.params.insert("username".to_string(), user);
        }

        if let Some(pass) = password {
            protocol_data.params.insert("password".to_string(), pass);
        }

        Ok(protocol_data)
    }

    // 解析PUBLISH数据包
    fn parse_publish(&self, data: &[u8], offset: usize) -> Result<ProtocolData, ProtocolError> {
        // 解析主题名
        let (topic, offset) = self.parse_mqtt_string(data, offset)?;

        // 解析Packet ID(如果QoS > 0)
        let first_byte = data[0];
        let qos = (first_byte >> 1) & 0x03;

        let mut packet_id = None;
        let mut offset = offset;

        if qos > 0 {
            if offset + 1 >= data.len() {
                return Err(ProtocolError::ParsingError(
                    "Invalid MQTT PUBLISH packet".to_string(),
                ));
            }
            packet_id = Some(((data[offset] as u16) << 8) | (data[offset + 1] as u16));
            offset += 2;
        }

        // 解析负载
        let payload = data[offset..].to_vec();
        let payload_str = String::from_utf8_lossy(&payload).to_string();

        // 创建协议数据
        let protocol_data =
            ProtocolData::new("mqtt_device".to_string(), "mqtt_publish".to_string())
                .with_raw_data(data.to_vec());

        let mut protocol_data = protocol_data;

        // 添加参数
        protocol_data.params.insert("topic".to_string(), topic);
        protocol_data
            .params
            .insert("qos".to_string(), qos.to_string());
        protocol_data
            .params
            .insert("payload".to_string(), payload_str);

        if let Some(id) = packet_id {
            protocol_data
                .params
                .insert("packet_id".to_string(), id.to_string());
        }

        Ok(protocol_data)
    }

    // 解析PINGREQ数据包
    fn parse_pingreq(&self, data: &[u8]) -> Result<ProtocolData, ProtocolError> {
        let protocol_data =
            ProtocolData::new("mqtt_device".to_string(), "mqtt_pingreq".to_string())
                .with_raw_data(data.to_vec());

        Ok(protocol_data)
    }
}

impl Protocol for MqttProtocol {
    fn parse(&self, data: &[u8]) -> Result<ProtocolData, ProtocolError> {
        debug!("Parsing MQTT protocol data: {:?}", data);

        // 验证数据
        if !self.validate(data) {
            return Err(ProtocolError::ValidationError(
                "Invalid MQTT protocol data".to_string(),
            ));
        }

        // 解析数据包类型
        let packet_type = self.parse_packet_type(data[0])?;

        // 解析剩余长度
        let (_remaining_length, offset) = self.parse_remaining_length(data)?;

        // 根据数据包类型进行解析
        match packet_type {
            MqttPacketType::Connect => self.parse_connect(data, offset),
            MqttPacketType::Publish => self.parse_publish(data, offset),
            MqttPacketType::PingReq => self.parse_pingreq(data),
            _ => {
                // 其他数据包类型的简单实现
                let protocol_data = ProtocolData::new(
                    "mqtt_device".to_string(),
                    format!("mqtt_{:?}", packet_type).to_lowercase(),
                )
                .with_raw_data(data.to_vec());
                Ok(protocol_data)
            }
        }
    }

    fn generate(&self, data: &ProtocolData) -> Result<Vec<u8>, ProtocolError> {
        debug!(
            "Generating MQTT protocol data for command: {}",
            data.command
        );

        // 目前只实现简单的PINGRESP响应
        if data.command == "mqtt_pingreq" {
            // PINGRESP数据包:控制包类型(13) + 剩余长度(0)
            Ok(vec![0xD0, 0x00])
        } else if data.command == "mqtt_connect" {
            // CONNACK数据包:控制包类型(2) + 剩余长度(2) + 连接确认标志(0) + 连接返回码(0)
            Ok(vec![0x20, 0x02, 0x00, 0x00])
        } else {
            // 默认返回空数据
            Ok(vec![])
        }
    }

    fn name(&self) -> &str {
        "MQTT"
    }

    fn version(&self) -> &str {
        "3.1.1"
    }

    fn validate(&self, data: &[u8]) -> bool {
        // 简单验证:检查数据长度
        if data.len() < 2 {
            return false;
        }

        // 解析剩余长度以验证数据包完整性
        match self.parse_remaining_length(data) {
            Ok((remaining_length, offset)) => offset + remaining_length <= data.len(),
            Err(_) => false,
        }
    }

    fn supported_commands(&self) -> Vec<&str> {
        vec![
            "mqtt_connect",
            "mqtt_publish",
            "mqtt_pingreq",
            "mqtt_pingresp",
            "mqtt_disconnect",
        ]
    }
}
