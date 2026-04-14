//! / 辅助函数:独立构建数据包(不借用self)

use crate::sync::config::LegacyServerConfig;
use anyhow::Result;
use log::debug;

/// 构建认证包(基于旧服务器TCP协议分析)
///
/// 协议格式(0x7E 0x7E 认证协议):
/// 起始标识 (2字节) | 长度 (2字节) | 消息类型 (1字节) | 用户名 (N字节) | 密码 (M字节) | 校验和 (1字节)
/// 0x7E 0x7E       | 大端序        | 0x01 = 认证      | ASCII编码      | ASCII编码      | 累加校验
///
/// 示例(用户名:ED, 密码:888888):
/// 7E 7E 00 0B 01 45 44 06 38 38 38 38 38 E2
pub fn build_auth_packet(config: &LegacyServerConfig) -> Result<Vec<u8>> {
    let mut packet = Vec::new();

    // 包头标识 (0x7E 0x7E)
    packet.extend_from_slice(&[0x7E, 0x7E]);

    // 构建数据部分
    let mut data = Vec::new();

    // 消息类型: 0x01 (认证)
    data.push(0x01);

    // 添加用户名(ASCII编码)
    if let Some(username) = &config.username {
        data.extend_from_slice(username.as_bytes());
    }

    // 添加密码(ASCII编码)
    if let Some(password) = &config.password {
        data.extend_from_slice(password.as_bytes());
    }

    // 计算校验和 (累加所有数据字节,取低8位)
    let checksum: u8 = data.iter().fold(0u8, |acc, &b| acc.wrapping_add(b));

    // 添加数据长度 (2字节,大端序)
    let data_len = data.len() as u16;
    packet.extend_from_slice(&data_len.to_be_bytes());

    // 添加数据
    packet.extend_from_slice(&data);

    // 添加校验和
    packet.push(checksum);

    debug!("Auth packet: {:?}", packet);
    log::info!(
        "Auth packet built: {} bytes, username: {:?}, password: {:?}",
        packet.len(),
        config.username.as_ref().unwrap_or(&String::new()),
        if config.password.is_some() {
            "***"
        } else {
            "None"
        }
    );

    Ok(packet)
}

/// 构建请求包(基于旧服务器协议)
pub fn build_request_packet(_config: &LegacyServerConfig, data: &[u8]) -> Result<Vec<u8>> {
    let mut packet = Vec::new();

    // 包头标识 (0x7E 0x7E)
    packet.extend_from_slice(&[0x7E, 0x7E]);

    // 添加数据长度 (2字节,大端序)
    let data_len = data.len() as u16;
    packet.extend_from_slice(&data_len.to_be_bytes());

    // 添加数据
    packet.extend_from_slice(data);

    // 计算校验和
    let checksum: u8 = data.iter().fold(0u8, |acc, &b| acc.wrapping_add(b));
    packet.push(checksum);

    debug!("Request packet: {:?}", packet);
    Ok(packet)
}

/// 构建命令包(基于旧服务器协议)
pub fn build_command_packet(
    _config: &LegacyServerConfig,
    device_id: i32,
    command: &[u8],
) -> Result<Vec<u8>> {
    let mut packet = Vec::new();

    // 包头标识 (0x7E 0x7E)
    packet.extend_from_slice(&[0x7E, 0x7E]);

    // 构建数据部分
    let mut data = Vec::new();

    // 添加设备ID (4字节,大端序)
    data.extend_from_slice(&device_id.to_be_bytes());

    // 添加命令数据
    data.extend_from_slice(command);

    // 添加数据长度 (2字节,大端序)
    let data_len = data.len() as u16;
    packet.extend_from_slice(&data_len.to_be_bytes());

    // 添加数据
    packet.extend_from_slice(&data);

    // 计算校验和
    let checksum: u8 = data.iter().fold(0u8, |acc, &b| acc.wrapping_add(b));
    packet.push(checksum);

    debug!("Command packet: {:?}", packet);
    Ok(packet)
}
