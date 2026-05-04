//! / CoAP协议实现
// 基于CoAP RFC 7252协议规范

use super::base::{Protocol, ProtocolData, ProtocolError};
use log::debug;
use std::collections::HashMap;

// CoAP消息类型
#[derive(Debug)]
enum CoapMessageType {
    Confirmable = 0,
    NonConfirmable = 1,
    Acknowledgement = 2,
    Reset = 3,
}

// CoAP方法代码
#[allow(dead_code)]
enum CoapMethod {
    Get = 1,
    Post = 2,
    Put = 3,
    Delete = 4,
}

// CoAP响应码
#[allow(dead_code)]
enum CoapResponseCode {
    Success = 200,
    Created = 201,
    Deleted = 202,
    Valid = 203,
    Changed = 204,
    Content = 205,
    BadRequest = 400,
    Unauthorized = 401,
    BadOption = 402,
    Forbidden = 403,
    NotFound = 404,
    MethodNotAllowed = 405,
    NotAcceptable = 406,
    PreconditionFailed = 412,
    RequestEntityTooLarge = 413,
    UnsupportedContentFormat = 415,
    InternalServerError = 500,
    NotImplemented = 501,
    BadGateway = 502,
    ServiceUnavailable = 503,
    GatewayTimeout = 504,
    ProxyingNotSupported = 505,
}

// CoAP选项编号
#[allow(dead_code)]
enum CoapOption {
    IfMatch = 1,
    UriHost = 3,
    ETag = 4,
    IfNoneMatch = 5,
    Observe = 6,
    UriPort = 7,
    LocationPath = 8,
    UriPath = 11,
    ContentFormat = 12,
    MaxAge = 14,
    UriQuery = 15,
    Accept = 17,
    LocationQuery = 20,
    Block2 = 23,
    Block1 = 27,
    Size2 = 28,
    Size1 = 60,
}

pub struct CoapProtocol;

impl Default for CoapProtocol {
    fn default() -> Self {
        Self::new()
    }
}

impl CoapProtocol {
    pub fn new() -> Self {
        Self
    }

    // 解析CoAP消息类型
    fn parse_message_type(&self, first_byte: u8) -> Result<CoapMessageType, ProtocolError> {
        let message_type = (first_byte >> 4) & 0x03;
        match message_type {
            0 => Ok(CoapMessageType::Confirmable),
            1 => Ok(CoapMessageType::NonConfirmable),
            2 => Ok(CoapMessageType::Acknowledgement),
            3 => Ok(CoapMessageType::Reset),
            _ => Err(ProtocolError::ParsingError(format!(
                "Unknown CoAP message type: {}",
                message_type
            ))),
        }
    }

    // 解析CoAP选项
    fn parse_options(
        &self,
        data: &[u8],
        offset: usize,
        end: usize,
    ) -> Result<HashMap<String, String>, ProtocolError> {
        let mut options = HashMap::new();
        let mut current_option = 0;
        let mut offset = offset;

        while offset < end {
            let delta = (data[offset] >> 4) & 0x0F;
            let length = data[offset] & 0x0F;
            offset += 1;

            // 处理扩展delta
            let extended_delta = if delta == 13 {
                if offset >= end {
                    return Err(ProtocolError::ParsingError(
                        "Invalid CoAP option delta".to_string(),
                    ));
                }
                let delta = data[offset] as u16;
                offset += 1;
                delta
            } else if delta == 14 {
                if offset + 1 >= end {
                    return Err(ProtocolError::ParsingError(
                        "Invalid CoAP option delta".to_string(),
                    ));
                }
                let delta = ((data[offset] as u16) << 8) | (data[offset + 1] as u16);
                offset += 2;
                delta
            } else if delta == 15 {
                return Err(ProtocolError::ParsingError(
                    "Invalid CoAP option delta".to_string(),
                ));
            } else {
                delta as u16
            };

            // 处理扩展length
            let extended_length = if length == 13 {
                if offset >= end {
                    return Err(ProtocolError::ParsingError(
                        "Invalid CoAP option length".to_string(),
                    ));
                }
                let len = data[offset] as usize;
                offset += 1;
                len
            } else if length == 14 {
                if offset + 1 >= end {
                    return Err(ProtocolError::ParsingError(
                        "Invalid CoAP option length".to_string(),
                    ));
                }
                let len = ((data[offset] as usize) << 8) | (data[offset + 1] as usize);
                offset += 2;
                len
            } else if length == 15 {
                return Err(ProtocolError::ParsingError(
                    "Invalid CoAP option length".to_string(),
                ));
            } else {
                length as usize
            };

            // 计算选项编号
            current_option += extended_delta;

            // 解析选项值
            if offset + extended_length > end {
                return Err(ProtocolError::ParsingError(
                    "Invalid CoAP option value length".to_string(),
                ));
            }

            let option_value = data[offset..offset + extended_length].to_vec();
            let option_value_str = String::from_utf8_lossy(&option_value).to_string();

            // 将选项添加到HashMap
            options.insert(current_option.to_string(), option_value_str);

            offset += extended_length;
        }

        Ok(options)
    }

    // 解析CoAP数据包
    fn parse_coap_packet(&self, data: &[u8]) -> Result<ProtocolData, ProtocolError> {
        if data.len() < 4 {
            return Err(ProtocolError::ParsingError(
                "Invalid CoAP packet length".to_string(),
            ));
        }

        // 解析版本、类型、TKL(Token Length)
        let version = (data[0] >> 6) & 0x03;
        let message_type = self.parse_message_type(data[0])?;
        let token_length = data[0] & 0x0F;

        if version != 1 {
            return Err(ProtocolError::ParsingError(format!(
                "Unsupported CoAP version: {}",
                version
            )));
        }

        // 解析代码
        let code = data[1];
        let class = (code >> 5) & 0x07;
        let detail = code & 0x1F;

        // 解析消息ID
        let message_id = ((data[2] as u16) << 8) | (data[3] as u16);

        // 解析Token
        let token_end = 4 + token_length as usize;
        if token_end > data.len() {
            return Err(ProtocolError::ParsingError(
                "Invalid CoAP token length".to_string(),
            ));
        }
        let token = data[4..token_end].to_vec();

        // 解析选项和负载
        let payload_marker_pos = data[token_end..].iter().position(|&b| b == 0xFF);
        let (options, payload) = match payload_marker_pos {
            Some(pos) => {
                let options_end = token_end + pos;
                let payload_start = options_end + 1;
                let options = self.parse_options(data, token_end, options_end)?;
                let payload = if payload_start < data.len() {
                    data[payload_start..].to_vec()
                } else {
                    Vec::new()
                };
                (options, payload)
            }
            None => {
                let options = self.parse_options(data, token_end, data.len())?;
                (options, Vec::new())
            }
        };

        // 构建协议数据
        let mut protocol_data =
            ProtocolData::new("coap_device".to_string(), "coap_request".to_string())
                .with_raw_data(data.to_vec());

        // 添加基本信息
        protocol_data
            .params
            .insert("version".to_string(), version.to_string());
        protocol_data.params.insert(
            "message_type".to_string(),
            format!("{:?}", message_type).to_lowercase(),
        );
        protocol_data
            .params
            .insert("message_id".to_string(), message_id.to_string());
        protocol_data
            .params
            .insert("token".to_string(), hex::encode(token));

        // 根据代码类型设置命令
        let command = match class {
            0 => {
                if detail == 0 {
                    "coap_empty".to_string()
                } else {
                    "coap_request".to_string()
                }
            }
            2 => "coap_response".to_string(),
            _ => "coap_other".to_string(),
        };
        protocol_data.params.insert("command".to_string(), command);

        // 解析方法或响应码
        if class == 0 && detail > 0 {
            // 请求方法
            let method = match detail {
                1 => "GET",
                2 => "POST",
                3 => "PUT",
                4 => "DELETE",
                _ => "UNKNOWN",
            };
            protocol_data
                .params
                .insert("method".to_string(), method.to_string());
        } else if class > 0 {
            // 响应码
            let status_code = (class * 32) + detail;
            protocol_data
                .params
                .insert("status_code".to_string(), status_code.to_string());
        }

        // 添加选项
        for (key, value) in options {
            protocol_data
                .params
                .insert(format!("option_{}", key), value);
        }

        // 添加负载
        if !payload.is_empty() {
            let payload_str = String::from_utf8_lossy(&payload).to_string();
            protocol_data
                .params
                .insert("payload".to_string(), payload_str);
        }

        Ok(protocol_data)
    }
}

impl Protocol for CoapProtocol {
    fn parse(&self, data: &[u8]) -> Result<ProtocolData, ProtocolError> {
        debug!("Parsing CoAP protocol data: {:?}", data);

        // 验证数据
        if !self.validate(data) {
            return Err(ProtocolError::ValidationError(
                "Invalid CoAP protocol data".to_string(),
            ));
        }

        self.parse_coap_packet(data)
    }

    fn generate(&self, data: &ProtocolData) -> Result<Vec<u8>, ProtocolError> {
        debug!(
            "Generating CoAP protocol data for command: {}",
            data.command
        );

        // 简单实现:生成ACK响应
        let version = 1;
        let message_type = 2; // Acknowledgement
        let token_length = 0;
        let code = 0; // Empty ACK

        // 获取消息ID
        let message_id = data
            .params
            .get("message_id")
            .unwrap_or(&"0".to_string())
            .parse::<u16>()
            .unwrap_or(0);

        // 构建CoAP头
        let coap_packet = vec![
            ((version << 6) | (message_type << 4) | token_length) as u8,
            code,
            (message_id >> 8) as u8,
            (message_id & 0xFF) as u8,
        ];

        Ok(coap_packet)
    }

    fn name(&self) -> &str {
        "CoAP"
    }

    fn version(&self) -> &str {
        "1.0"
    }

    fn validate(&self, data: &[u8]) -> bool {
        // 简单验证:检查数据长度和版本
        if data.len() < 4 {
            return false;
        }

        let version = (data[0] >> 6) & 0x03;
        version == 1
    }

    fn supported_commands(&self) -> Vec<&str> {
        vec!["coap_request", "coap_response", "coap_empty"]
    }
}
