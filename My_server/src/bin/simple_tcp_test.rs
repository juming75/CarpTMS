//! / 简单的TCP连接测试(独立于BFF模块)
use std::env;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 从环境变量读取配置
    let host = env::var("LEGACY_SYNC_HOST").unwrap_or_else(|_| "203.170.59.153".to_string());
    let port = env::var("LEGACY_SYNC_PORT")
        .ok()
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(9808);
    let username = env::var("LEGACY_SYNC_USERNAME").unwrap_or_else(|_| "ED".to_string());
    let password = env::var("LEGACY_SYNC_PASSWORD").unwrap_or_else(|_| "888888".to_string());

    println!("========================================");
    println!("旧服务器TCP连接测试");
    println!("========================================");
    println!("服务器地址: {}:{}", host, port);
    println!("用户名: {}", username);
    println!("密码: {}", password);
    println!("========================================\n");

    let addr = format!("{}:{}", host, port);
    println!("正在连接到 {}...\n", addr);

    match TcpStream::connect(&addr).await {
        Ok(mut stream) => {
            println!("✓ 成功连接到旧服务器!\n");

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

            println!("发送认证包 ({} 字节):", auth_packet.len());
            println!("Hex: {:?}", auth_packet);
            println!("ASCII: {:?}", String::from_utf8_lossy(&auth_packet));
            println!();

            stream.write_all(&auth_packet).await?;
            println!("✓ 认证包已发送\n");

            // 读取响应
            println!("等待服务器响应...");
            let mut buf = vec![0u8; 1024];

            match tokio::time::timeout(std::time::Duration::from_secs(10), stream.read(&mut buf))
                .await
            {
                Ok(Ok(n)) => {
                    println!("✓ 收到 {} 字节响应\n", n);
                    if n > 0 {
                        let response = &buf[..n];
                        println!("响应数据 (Hex): {:?}", response);
                        println!("响应数据 (ASCII): {:?}", String::from_utf8_lossy(response));
                        println!();

                        // 尝试解析响应
                        if n >= 1 {
                            let status_code = response[0];
                            println!("状态码: 0x{:02X} ({})", status_code, status_code);

                            match status_code {
                                0x00 => println!("\n✓✓✓ 认证成功! ✓✓✓"),
                                0x01 => println!("\n✗ 认证失败: 用户名或密码错误"),
                                0x02 => println!("\n✗ 认证失败: 连接超时"),
                                0x03 => println!("\n✗ 认证失败: 服务器内部错误"),
                                _ => println!("\n⚠ 未知状态码: 0x{:02X}", status_code),
                            }

                            // 尝试打印完整响应内容
                            if n > 1 {
                                println!("\n完整响应详情:");
                                for (i, byte) in response.iter().enumerate() {
                                    print!("{:02X} ", byte);
                                    if (i + 1) % 16 == 0 {
                                        println!();
                                    }
                                }
                                println!();
                            }
                        }
                    } else {
                        println!("⚠ 收到空响应 (0 字节)");
                    }
                }
                Ok(Err(e)) => {
                    println!("✗ 读取响应失败: {}", e);
                }
                Err(_) => {
                    println!("✗ 超时等待响应 (10 秒)");
                }
            }
        }
        Err(e) => {
            println!("✗ 连接失败: {}", e);
            println!("\n可能的原因:");
            println!("1. 旧服务器未运行");
            println!("2. 网络不通 (防火墙、网络策略)");
            println!("3. 端口错误");
            println!("4. 主机地址错误");
            println!("\n建议:");
            println!("- 使用 telnet 检查连接: telnet {} {}", host, port);
            println!("- 检查防火墙设置");
            println!("- 确认旧服务器状态");
        }
    }

    println!("\n========================================");
    println!("测试完成");
    println!("========================================");

    Ok(())
}
