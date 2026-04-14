use log::{debug, error, info};
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // 初始化日志
    env_logger::init();

    // 监听8988端口
    let listener = TcpListener::bind("0.0.0.0:8988").await?;
    info!("开始监测8988端口...");
    info!("监听地址: {}", listener.local_addr()?);

    loop {
        // 接受连接
        let (socket, addr) = listener.accept().await?;
        info!("接收到来自 {} 的连接", addr);

        // 处理连接
        tokio::spawn(async move {
            if let Err(e) = handle_connection(socket, addr).await {
                error!("处理连接时出错: {}", e);
            }
        });
    }
}

async fn handle_connection(
    mut socket: tokio::net::TcpStream,
    addr: SocketAddr,
) -> std::io::Result<()> {
    let mut buffer = vec![0; 4096];

    loop {
        // 读取数据
        let n = socket.read(&mut buffer).await?;

        if n == 0 {
            info!("连接 {} 已关闭", addr);
            break;
        }

        // 处理接收到的数据
        let data = &buffer[0..n];
        info!("从 {} 接收到 {} 字节数据", addr, n);

        // 打印原始数据(十六进制)
        debug!("原始数据 (十六进制): {:?}", data);

        // 尝试解析为JT808格式
        if let Some(jt808_info) = parse_jt808(data) {
            info!("检测到JT808格式数据: {}", jt808_info);
        } else {
            // 尝试解析为其他格式
            if let Ok(text) = String::from_utf8(data.to_vec()) {
                info!("文本数据: {}", text.trim());
            } else {
                info!("二进制数据,长度: {} 字节", n);
            }
        }

        // 发送响应
        let response = "收到数据\n";
        socket.write_all(response.as_bytes()).await?;
    }

    Ok(())
}

// 简单的JT808格式检测
fn parse_jt808(data: &[u8]) -> Option<String> {
    // JT808协议通常以0x7e开头和结尾
    if data.len() >= 2 && data[0] == 0x7e && data[data.len() - 1] == 0x7e {
        // 提取消息ID(通常在第5-6字节)
        if data.len() >= 6 {
            let msg_id = ((data[4] as u16) << 8) | (data[5] as u16);
            return Some(format!("消息ID: 0x{:04X}", msg_id));
        }
        return Some("JT808格式数据".to_string());
    }
    None
}
