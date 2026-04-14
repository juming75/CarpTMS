//! / 测试旧服务器TCP连接
use std::env;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    // 从环境变量读取配置
    let host = env::var("LEGACY_SYNC_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("LEGACY_SYNC_PORT")
        .ok()
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(9808);
    let username = env::var("LEGACY_SYNC_USERNAME").unwrap_or_else(|_| "ED".to_string());
    let password = env::var("LEGACY_SYNC_PASSWORD").unwrap_or_else(|_| "888888".to_string());

    let addr = format!("{}:{}", host, port);
    info!("Attempting to connect to legacy server at {}...", addr);
    info!("Username: {}", username);

    match TcpStream::connect(&addr).await {
        Ok(mut stream) => {
            info!("Successfully connected to legacy server!");

            // 构建认证包 [magic(2) + version(1) + username_len(1) + username(n) + password_len(1) + password(n)]
            let username_bytes = username.as_bytes();
            let password_bytes = password.as_bytes();

            let mut auth_packet = vec![
                0x7E,
                0x7E,                       // Magic number
                0x01,                       // Version 1
                username_bytes.len() as u8, // Username length
            ];
            auth_packet.extend_from_slice(username_bytes);
            auth_packet.push(password_bytes.len() as u8); // Password length
            auth_packet.extend_from_slice(password_bytes);

            stream.write_all(&auth_packet).await?;
            info!(
                "Sent auth packet ({} bytes): {:?}",
                auth_packet.len(),
                auth_packet
            );

            // 读取响应
            let mut buf = vec![0u8; 1024];
            match tokio::time::timeout(std::time::Duration::from_secs(5), stream.read(&mut buf))
                .await
            {
                Ok(Ok(n)) => {
                    info!("Received {} bytes from legacy server", n);
                    if n > 0 {
                        info!("Response: {:?}", &buf[..n]);

                        // 尝试解析响应
                        if n >= 1 {
                            let status_code = buf[0];
                            info!("Status code: 0x{:02X}", status_code);
                            if status_code == 0x00 {
                                info!("✓ Authentication successful!");
                            } else {
                                info!(
                                    "✗ Authentication failed, status code: 0x{:02X}",
                                    status_code
                                );
                            }
                        }
                    } else {
                        info!("No response received (empty packet)");
                    }
                }
                Ok(Err(e)) => {
                    error!("Failed to read response: {}", e);
                }
                Err(_) => {
                    error!("Timeout waiting for response (5 seconds)");
                }
            }
        }
        Err(e) => {
            error!("Failed to connect to legacy server: {}", e);
            return Err(e.into());
        }
    }

    Ok(())
}
