//! / HTTP协议实现
// 基于HTTP/1.1协议规范

use super::base::{Protocol, ProtocolData, ProtocolError};
use log::debug;
use std::collections::HashMap;

pub struct HttpProtocol;

impl Default for HttpProtocol {
    fn default() -> Self {
        Self::new()
    }
}

impl HttpProtocol {
    pub fn new() -> Self {
        Self
    }

    // 解析HTTP请求行
    fn parse_request_line(
        &self,
        request_line: &str,
    ) -> Result<(String, String, String), ProtocolError> {
        let parts: Vec<&str> = request_line.split_whitespace().collect();
        if parts.len() != 3 {
            return Err(ProtocolError::ParsingError(format!(
                "Invalid HTTP request line: {}",
                request_line
            )));
        }

        Ok((
            parts[0].to_string(),
            parts[1].to_string(),
            parts[2].to_string(),
        ))
    }

    // 解析HTTP头部
    fn parse_headers(&self, headers: &str) -> Result<HashMap<String, String>, ProtocolError> {
        let mut parsed_headers = HashMap::new();

        for line in headers.lines() {
            if line.is_empty() {
                continue;
            }

            if let Some(colon_pos) = line.find(':') {
                let key = line[..colon_pos].trim().to_lowercase();
                let value = line[colon_pos + 1..].trim().to_string();
                parsed_headers.insert(key, value);
            } else {
                return Err(ProtocolError::ParsingError(format!(
                    "Invalid HTTP header line: {}",
                    line
                )));
            }
        }

        Ok(parsed_headers)
    }

    // 生成HTTP响应
    #[allow(dead_code)]
    fn generate_response(
        &self,
        status_code: u16,
        headers: HashMap<String, String>,
        body: &[u8],
    ) -> Vec<u8> {
        let mut response = Vec::new();

        // 响应行
        let status_text = match status_code {
            200 => "OK",
            400 => "Bad Request",
            404 => "Not Found",
            500 => "Internal Server Error",
            _ => "Unknown Status",
        };
        response
            .extend_from_slice(format!("HTTP/1.1 {} {}\r\n", status_code, status_text).as_bytes());

        // 响应头部
        for (key, value) in headers {
            response.extend_from_slice(format!("{}: {}\r\n", key, value).as_bytes());
        }
        response.extend_from_slice(format!("Content-Length: {}\r\n", body.len()).as_bytes());
        response.extend_from_slice(b"\r\n");

        // 响应体
        response.extend_from_slice(body);

        response
    }
}

impl Protocol for HttpProtocol {
    fn parse(&self, data: &[u8]) -> Result<ProtocolData, ProtocolError> {
        debug!("Parsing HTTP protocol data: {:?}", data);

        // 验证数据
        if !self.validate(data) {
            return Err(ProtocolError::ValidationError(
                "Invalid HTTP protocol data".to_string(),
            ));
        }

        // 将字节转换为字符串
        let data_str = String::from_utf8(data.to_vec()).map_err(|e| {
            ProtocolError::ParsingError(format!("Invalid UTF-8 in HTTP data: {}", e))
        })?;

        // 分割请求行、头部和正文
        let parts: Vec<&str> = data_str.splitn(2, "\r\n\r\n").collect();
        if parts.is_empty() {
            return Err(ProtocolError::ParsingError(
                "Invalid HTTP data format".to_string(),
            ));
        }

        let headers_part = parts[0];
        let body_part = if parts.len() > 1 { parts[1] } else { "" };

        // 分割请求行和头部
        let header_lines: Vec<&str> = headers_part.splitn(2, "\r\n").collect();
        if header_lines.is_empty() {
            return Err(ProtocolError::ParsingError(
                "Invalid HTTP header format".to_string(),
            ));
        }

        // 解析请求行
        let (method, path, version) = self.parse_request_line(header_lines[0])?;

        // 解析头部
        let headers = if header_lines.len() > 1 {
            self.parse_headers(header_lines[1])?
        } else {
            HashMap::new()
        };

        // 从头部获取设备ID
        let device_id = headers
            .get("x-device-id")
            .unwrap_or(&"unknown".to_string())
            .clone();

        // 创建协议数据
        let mut protocol_data =
            ProtocolData::new(device_id, format!("http_{}", method.to_lowercase()))
                .with_raw_data(data.to_vec());

        // 添加参数
        protocol_data.params.insert("path".to_string(), path);
        protocol_data.params.insert("version".to_string(), version);
        protocol_data.params.insert("method".to_string(), method);
        protocol_data
            .params
            .insert("body".to_string(), body_part.to_string());

        // 添加所有头部作为参数
        for (key, value) in headers {
            protocol_data
                .params
                .insert(format!("header_{}", key), value);
        }

        Ok(protocol_data)
    }

    fn generate(&self, data: &ProtocolData) -> Result<Vec<u8>, ProtocolError> {
        debug!(
            "Generating HTTP protocol data for command: {}",
            data.command
        );

        // 解析命令,提取HTTP方法
        let method = if data.command.starts_with("http_") {
            data.command[5..].to_uppercase()
        } else {
            "GET".to_string()
        };

        // 获取路径和版本
        let path = data.params.get("path").unwrap_or(&"/".to_string()).clone();
        let version = data
            .params
            .get("version")
            .unwrap_or(&"HTTP/1.1".to_string())
            .clone();

        // 构建请求行
        let request_line = format!("{} {} {}\r\n", method, path, version);

        // 构建头部
        let mut headers = String::new();
        let empty_body = "".to_string();
        let body = data.params.get("body").unwrap_or(&empty_body).as_bytes();

        // 提取所有以"header_"开头的参数作为HTTP头部
        for (key, value) in &data.params {
            if let Some(stripped) = key.strip_prefix("header_") {
                let header_name = stripped.to_string();
                headers.push_str(&format!("{}: {}\r\n", header_name, value));
            }
        }

        // 添加设备ID头部
        headers.push_str(&format!("X-Device-ID: {}\r\n", data.device_id));
        headers.push_str(&format!("Content-Length: {}\r\n", body.len()));
        headers.push_str("\r\n");

        // 构建完整请求
        let mut request = Vec::new();
        request.extend_from_slice(request_line.as_bytes());
        request.extend_from_slice(headers.as_bytes());
        request.extend_from_slice(body);

        Ok(request)
    }

    fn name(&self) -> &str {
        "HTTP"
    }

    fn version(&self) -> &str {
        "1.1"
    }

    fn validate(&self, data: &[u8]) -> bool {
        // 简单验证:检查数据是否以HTTP方法开头
        if data.len() < 4 {
            return false;
        }

        // 检查是否是有效的HTTP请求
        let methods = [
            "GET ", "POST ", "PUT ", "DELETE ", "HEAD ", "OPTIONS ", "PATCH ",
        ];
        let data_str = String::from_utf8_lossy(data);

        for method in methods {
            if data_str.starts_with(method) {
                return true;
            }
        }

        // 检查是否是有效的HTTP响应
        data_str.starts_with("HTTP/")
    }

    fn supported_commands(&self) -> Vec<&str> {
        vec![
            "http_get",
            "http_post",
            "http_put",
            "http_delete",
            "http_head",
            "http_options",
            "http_patch",
        ]
    }
}
