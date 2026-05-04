//! / BSJ协议适配器
// 使用 BSJ Protocol (帧头: 0x2D 0x2D) 与旧服务器通讯

use crate::protocols::bsj::{BSJProtocol, BsjCommand};
use crate::sync::config::LegacyServerConfig;
use crate::sync::models::LegacyVehicle;
use anyhow::{anyhow, Result};
use log::{debug, error, info, warn};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufStream};
use tokio::net::TcpStream;

/// BSJ协议适配器
pub struct BsjAdapter {
    config: LegacyServerConfig,
    stream: Option<BufStream<TcpStream>>,
    connected: bool,
}

impl BsjAdapter {
    /// 创建新的适配器实例
    pub fn new(config: LegacyServerConfig) -> Self {
        Self {
            config,
            stream: None,
            connected: false,
        }
    }

    /// 连接到旧服务器
    pub async fn connect(&mut self) -> Result<()> {
        if self.connected {
            info!("Already connected to BSJ server");
            return Ok(());
        }

        let addr = format!("{}:{}", self.config.host, self.config.port);
        info!("Connecting to BSJ server at {}...", addr);

        match TcpStream::connect(&addr).await {
            Ok(stream) => {
                self.stream = Some(BufStream::new(stream));
                self.connected = true;

                info!("Connected to BSJ server: {}", addr);
                Ok(())
            }
            Err(e) => {
                error!("Failed to connect to BSJ server: {}", e);
                Err(anyhow!("Connection failed: {}", e))
            }
        }
    }

    /// 断开连接
    pub async fn disconnect(&mut self) {
        if let Some(mut stream) = self.stream.take() {
            let _ = stream.shutdown().await;
            info!("Disconnected from BSJ server");
        }
        self.connected = false;
    }

    /// 检查连接状态
    pub fn is_connected(&self) -> bool {
        self.connected
    }

    /// 发送认证
    pub async fn authenticate(&mut self) -> Result<bool> {
        if !self.connected {
            return Err(anyhow!("Not connected to server"));
        }

        info!("Authenticating to BSJ server...");

        // 构建认证包 (数据内容需要根据实际协议定义)
        let auth_data = vec![1u8]; // 1表示认证成功
        let protocol = BSJProtocol::new();
        let packet = protocol.build_packet(BsjCommand::Auth, &auth_data)?;

        self.send(&packet).await?;

        // 读取响应
        let response = self.receive().await?;

        // 解析响应
        if response.len() >= 3 {
            let header = &response[0..2];
            if header == crate::protocols::bsj::BSJ_FRAME_HEADER {
                // 检查命令ID
                let cmd_id = response[2];
                if cmd_id == BsjCommand::Auth.as_u8() {
                    // 提取认证结果
                    let data_start = 11; // 帧头(2) + CMD(1) + 长度(2) + IP(4) + 保留(2)
                    if data_start < response.len() {
                        let result = response[data_start];
                        info!("Authentication result: {}", result);
                        return Ok(result == 1);
                    }
                }
            }
        }

        Ok(false)
    }

    /// 查询车辆列表
    pub async fn fetch_vehicles(&mut self) -> Result<Vec<LegacyVehicle>> {
        self.ensure_connected().await?;

        info!("Fetching vehicles from BSJ server...");

        let protocol = BSJProtocol::new();
        let packet = protocol.build_packet(BsjCommand::VehicleQuery, &[])?;
        self.send(&packet).await?;

        // 读取响应
        let response_buf = self.receive_full_packet(Duration::from_secs(10)).await?;

        // 解析车辆列表
        self.parse_vehicle_response(&response_buf)
    }

    /// 查询用户列表
    pub async fn fetch_users(&mut self) -> Result<Vec<crate::sync::models::LegacyUser>> {
        self.ensure_connected().await?;

        info!("Fetching users from BSJ server...");

        let protocol = BSJProtocol::new();
        let packet = protocol.build_packet(BsjCommand::UserQuery, &[])?;
        self.send(&packet).await?;

        let response_buf = self.receive_full_packet(Duration::from_secs(10)).await?;

        self.parse_user_response(&response_buf)
    }

    /// 查询GPS历史数据
    pub async fn fetch_gps_history(
        &mut self,
        device_id: &str,
        start_time: i64,
        end_time: i64,
    ) -> Result<Vec<crate::sync::models::LegacyGpsData>> {
        self.ensure_connected().await?;

        info!(
            "Fetching GPS history for device {} from {} to {}",
            device_id, start_time, end_time
        );

        // 构建GPS历史查询请求
        let mut query_data = Vec::new();

        // 设备ID (简化格式,可能需要根据实际协议调整)
        let device_bytes = device_id.as_bytes();
        query_data.extend_from_slice(device_bytes);

        // 开始时间
        query_data.extend_from_slice(&start_time.to_be_bytes());

        // 结束时间
        query_data.extend_from_slice(&end_time.to_be_bytes());

        let protocol = BSJProtocol::new();
        let packet = protocol.build_packet(BsjCommand::GpsHistoryQuery, &query_data)?;
        self.send(&packet).await?;

        let response_buf = self.receive_full_packet(Duration::from_secs(15)).await?;

        self.parse_gps_response(&response_buf)
    }

    /// 发送心跳
    pub async fn send_heartbeat(&mut self) -> Result<()> {
        if !self.connected {
            return Err(anyhow!("Not connected to server"));
        }

        let protocol = BSJProtocol::new();
        let packet = protocol.build_packet(BsjCommand::RealtimePush, &[])?;
        self.send(&packet).await?;

        debug!("Heartbeat sent");
        Ok(())
    }

    /// 发送数据
    async fn send(&mut self, data: &[u8]) -> Result<()> {
        if let Some(stream) = self.stream.as_mut() {
            stream.write_all(data).await?;
            stream.flush().await?;
            debug!("Sent {} bytes: {:02X?}", data.len(), data);
            Ok(())
        } else {
            Err(anyhow!("Not connected to server"))
        }
    }

    /// 接收数据
    async fn receive(&mut self) -> Result<Vec<u8>> {
        let mut buf = vec![0u8; 8192];
        let stream = self
            .stream
            .as_mut()
            .ok_or_else(|| anyhow!("Not connected to server"))?;

        let n = stream.read(&mut buf).await?;
        if n == 0 {
            return Err(anyhow!("Connection closed by server"));
        }

        buf.truncate(n);
        debug!("Received {} bytes", n);
        Ok(buf)
    }

    /// 接收完整的数据包
    async fn receive_full_packet(&mut self, timeout: Duration) -> Result<Vec<u8>> {
        let mut response_buf = Vec::new();
        let mut temp_buf = vec![0u8; 8192];

        match tokio::time::timeout(timeout, async {
            while let Some(stream) = self.stream.as_mut() {
                let n = stream.read(&mut temp_buf).await?;

                if n == 0 {
                    break;
                }

                response_buf.extend_from_slice(&temp_buf[..n]);

                // 检查是否收到完整的BSJ数据包
                // 格式: 帧头(2B) + CMD(1B) + 长度(2B) + IP(4B) + 保留(2B) + 数据 + Xor(1B) + 帧尾(1B)
                if response_buf.len() >= 13 {
                    let length = u16::from_be_bytes([response_buf[3], response_buf[4]]) as usize;
                    let total_len = 11 + length + 2; // 头部(11) + 数据(length) + XOR(1) + 帧尾(1)

                    if response_buf.len() >= total_len {
                        break;
                    }
                }
            }

            Ok::<(), anyhow::Error>(())
        })
        .await
        {
            Ok(_) => {}
            Err(_) => {
                warn!("Receive packet timeout");
            }
        }

        debug!("Received full packet: {} bytes", response_buf.len());
        Ok(response_buf)
    }

    /// 确保连接状态
    async fn ensure_connected(&mut self) -> Result<()> {
        if !self.connected {
            self.connect().await?;
        }
        Ok(())
    }

    /// 解析车辆响应(简化实现,需要根据实际响应格式完善)
    #[allow(dead_code)]
    fn parse_vehicle_response(&self, data: &[u8]) -> Result<Vec<LegacyVehicle>> {
        debug!("Parsing BSJ vehicle response: {:02X?}", data);

        // TODO: 根据实际响应格式解析车辆列表
        // BSJ协议响应格式需要分析抓包数据
        Ok(Vec::new())
    }

    /// 解析用户响应(简化实现,需要根据实际响应格式完善)
    #[allow(dead_code)]
    fn parse_user_response(&self, data: &[u8]) -> Result<Vec<crate::sync::models::LegacyUser>> {
        debug!("Parsing BSJ user response: {:02X?}", data);

        // TODO: 根据实际响应格式解析用户列表
        Ok(Vec::new())
    }

    /// 解析GPS响应(简化实现,需要根据实际响应格式完善)
    #[allow(dead_code)]
    fn parse_gps_response(&self, data: &[u8]) -> Result<Vec<crate::sync::models::LegacyGpsData>> {
        debug!("Parsing BSJ GPS response: {:02X?}", data);

        // TODO: 根据实际响应格式解析GPS数据
        Ok(Vec::new())
    }
}
