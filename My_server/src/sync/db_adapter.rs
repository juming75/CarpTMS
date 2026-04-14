//! / 数据库协议适配器
// 使用 DB_Protocol (魔数: 0x12 0x34 0x56 0x78) 与 GVLP 前置机通讯

use crate::protocols::db_protocol::DbCommand;
use crate::sync::config::LegacyServerConfig;
use crate::sync::models::LegacyVehicle;
use anyhow::{anyhow, Result};
use log::{debug, error, info, warn};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufStream};
use tokio::net::TcpStream;

/// 数据库协议适配器
pub struct DbAdapter {
    config: LegacyServerConfig,
    stream: Option<BufStream<TcpStream>>,
    connected: bool,
    user_id: Option<i32>,
}

impl DbAdapter {
    /// 创建新的适配器实例
    pub fn new(config: LegacyServerConfig) -> Self {
        Self {
            config,
            stream: None,
            connected: false,
            user_id: None,
        }
    }

    /// 连接到旧服务器
    pub async fn connect(&mut self) -> Result<()> {
        if self.connected {
            info!("Already connected to DB server");
            return Ok(());
        }

        let addr = format!("{}:{}", self.config.host, self.config.port);
        info!("Connecting to DB server at {}...", addr);

        match TcpStream::connect(&addr).await {
            Ok(stream) => {
                self.stream = Some(BufStream::new(stream));
                self.connected = true;

                info!("Connected to DB server: {}", addr);

                // 发送登录认证
                if let Err(e) = self.login().await {
                    error!("Failed to login: {}", e);
                    self.disconnect().await;
                    return Err(e);
                }

                info!("DB server login successful");
                Ok(())
            }
            Err(e) => {
                error!("Failed to connect to DB server: {}", e);
                Err(anyhow!("Connection failed: {}", e))
            }
        }
    }

    /// 断开连接
    pub async fn disconnect(&mut self) {
        if let Some(mut stream) = self.stream.take() {
            let _ = stream.shutdown().await;
            info!("Disconnected from DB server");
        }
        self.connected = false;
        self.user_id = None;
    }

    /// 检查连接状态
    pub fn is_connected(&self) -> bool {
        self.connected
    }

    /// 登录到服务器
    async fn login(&mut self) -> Result<()> {
        // 构建登录请求包
        let mut login_data = Vec::new();

        // 用户名长度(2) + 用户名
        let username = self
            .config
            .username
            .as_ref()
            .map(|s| s.as_bytes())
            .unwrap_or(b"");
        login_data.extend_from_slice(&(username.len() as u16).to_be_bytes());
        login_data.extend_from_slice(username);

        // 密码长度(2) + 密码
        let password = self
            .config
            .password
            .as_ref()
            .map(|s| s.as_bytes())
            .unwrap_or(b"");
        login_data.extend_from_slice(&(password.len() as u16).to_be_bytes());
        login_data.extend_from_slice(password);

        debug!("Building login packet for user: {:?}", self.config.username);

        // 构建完整的数据包
        let packet = self.build_packet(DbCommand::ApplyLogin, &login_data)?;

        // 发送登录请求
        self.send(&packet).await?;

        // 等待登录响应
        let response = self.receive().await?;
        debug!("Received login response: {:02X?}", response);

        // 解析响应
        if response.len() >= 11 {
            // 验证魔数
            if response[0..4] != crate::protocols::db_protocol::DB_PROTOCOL_MAGIC {
                return Err(anyhow!("Invalid login response magic"));
            }

            // 检查命令ID
            let cmd_id = u16::from_be_bytes([response[8], response[9]]);
            if cmd_id != DbCommand::AnswerLogin.as_u16() {
                return Err(anyhow!("Unexpected response command: 0x{:04X}", cmd_id));
            }

            // 解析响应数据
            // 数据格式: 标志位(1) + 用户ID(4)
            if response.len() >= 15 {
                let flag = response[10];
                let user_id =
                    i32::from_be_bytes([response[11], response[12], response[13], response[14]]);

                if flag == 0 {
                    return Err(anyhow!("Login failed: invalid credentials"));
                }

                self.user_id = Some(user_id);
                info!("Logged in successfully, user_id: {}", user_id);
                return Ok(());
            }
        }

        Err(anyhow!("Invalid login response format"))
    }

    /// 发送心跳
    pub async fn send_heartbeat(&mut self) -> Result<()> {
        if !self.connected {
            return Err(anyhow!("Not connected to server"));
        }

        let packet = self.build_packet(DbCommand::Heartbeat, &[])?;
        self.send(&packet).await?;

        // 等待心跳响应
        let response = self.receive().await?;
        if response.len() >= 10 {
            let cmd_id = u16::from_be_bytes([response[8], response[9]]);
            if cmd_id != DbCommand::HeartbeatResponse.as_u16() {
                warn!("Unexpected heartbeat response: 0x{:04X}", cmd_id);
            }
        }

        debug!("Heartbeat sent and received");
        Ok(())
    }

    /// 查询车辆列表
    pub async fn fetch_vehicles(&mut self) -> Result<Vec<LegacyVehicle>> {
        self.ensure_connected().await?;

        info!("Fetching vehicles from DB server...");

        // 构建车辆查询请求
        let packet = self.build_packet(DbCommand::VehicleQuery, &[])?;
        self.send(&packet).await?;

        // 读取响应(可能分多次读取)
        let mut response_buf = Vec::new();
        let mut temp_buf = vec![0u8; 8192];

        // 设置读取超时
        match tokio::time::timeout(Duration::from_secs(10), async {
            while let Some(stream) = self.stream.as_mut() {
                let n = stream.read(&mut temp_buf).await?;

                if n == 0 {
                    break;
                }

                response_buf.extend_from_slice(&temp_buf[..n]);

                // 检查是否收到完整的数据包
                if response_buf.len() >= 11 {
                    let length = u32::from_be_bytes([
                        response_buf[4],
                        response_buf[5],
                        response_buf[6],
                        response_buf[7],
                    ]) as usize;

                    let total_len = 10 + length + 1; // 魔数(4) + 长度(4) + 命令(2) + 数据(length) + 校验(1)

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
                warn!("Fetch vehicles timeout");
                return Ok(Vec::new());
            }
        }

        // 解析车辆列表
        self.parse_vehicle_response(&response_buf)
    }

    /// 查询用户列表
    pub async fn fetch_users(&mut self) -> Result<Vec<crate::sync::models::LegacyUser>> {
        self.ensure_connected().await?;

        info!("Fetching users from DB server...");

        let packet = self.build_packet(DbCommand::UserQuery, &[])?;
        self.send(&packet).await?;

        let mut response_buf = Vec::new();
        let mut temp_buf = vec![0u8; 8192];

        match tokio::time::timeout(Duration::from_secs(10), async {
            while let Some(stream) = self.stream.as_mut() {
                let n = stream.read(&mut temp_buf).await?;

                if n == 0 {
                    break;
                }

                response_buf.extend_from_slice(&temp_buf[..n]);

                if response_buf.len() >= 11 {
                    let length = u32::from_be_bytes([
                        response_buf[4],
                        response_buf[5],
                        response_buf[6],
                        response_buf[7],
                    ]) as usize;

                    let total_len = 10 + length + 1;

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
                warn!("Fetch users timeout");
                return Ok(Vec::new());
            }
        }

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

        // 设备ID长度(2) + 设备ID
        let device_bytes = device_id.as_bytes();
        query_data.extend_from_slice(&(device_bytes.len() as u16).to_be_bytes());
        query_data.extend_from_slice(device_bytes);

        // 开始时间 (4字节, 大端序)
        query_data.extend_from_slice(&start_time.to_be_bytes());

        // 结束时间 (4字节, 大端序)
        query_data.extend_from_slice(&end_time.to_be_bytes());

        let packet = self.build_packet(DbCommand::GpsHistoryQuery, &query_data)?;
        self.send(&packet).await?;

        // 读取响应
        let mut response_buf = Vec::new();
        let mut temp_buf = vec![0u8; 16384]; // GPS数据可能较大

        match tokio::time::timeout(Duration::from_secs(15), async {
            while let Some(stream) = self.stream.as_mut() {
                let n = stream.read(&mut temp_buf).await?;

                if n == 0 {
                    break;
                }

                response_buf.extend_from_slice(&temp_buf[..n]);

                if response_buf.len() >= 11 {
                    let length = u32::from_be_bytes([
                        response_buf[4],
                        response_buf[5],
                        response_buf[6],
                        response_buf[7],
                    ]) as usize;

                    let total_len = 10 + length + 1;

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
                warn!("Fetch GPS history timeout");
                return Ok(Vec::new());
            }
        }

        self.parse_gps_response(&response_buf)
    }

    /// 构建数据包
    fn build_packet(&self, command: DbCommand, data: &[u8]) -> Result<Vec<u8>> {
        let mut packet = Vec::new();

        // 魔数
        packet.extend_from_slice(&crate::protocols::db_protocol::DB_PROTOCOL_MAGIC);

        // 数据长度 (包含命令+数据+校验)
        let length = 2 + data.len() + 1;

        // 长度字段 (4字节,大端序)
        packet.extend_from_slice(&(length as u32).to_be_bytes());

        // 命令ID (2字节,大端序)
        packet.extend_from_slice(&command.as_u16().to_be_bytes());

        // 数据
        packet.extend_from_slice(data);

        // 计算校验和 (从魔数到数据末尾)
        let mut checksum: u8 = 0;
        for byte in &packet {
            checksum ^= byte;
        }
        packet.push(checksum);

        debug!(
            "Built packet: command={:?}, length={}, checksum=0x{:02X}",
            command, length, checksum
        );

        Ok(packet)
    }

    /// 发送数据
    async fn send(&mut self, data: &[u8]) -> Result<()> {
        if let Some(stream) = self.stream.as_mut() {
            stream.write_all(data).await?;
            stream.flush().await?;
            debug!("Sent {} bytes", data.len());
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
        debug!("Received {} bytes: {:02X?}", n, &buf);
        Ok(buf)
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
        debug!("Parsing vehicle response: {:02X?}", data);

        // TODO: 根据实际响应格式解析车辆列表
        // 这里返回空列表,实际需要解析响应数据
        Ok(Vec::new())
    }

    /// 解析用户响应(简化实现,需要根据实际响应格式完善)
    #[allow(dead_code)]
    fn parse_user_response(&self, data: &[u8]) -> Result<Vec<crate::sync::models::LegacyUser>> {
        debug!("Parsing user response: {:02X?}", data);

        // TODO: 根据实际响应格式解析用户列表
        // 这里返回空列表,实际需要解析响应数据
        Ok(Vec::new())
    }

    /// 解析GPS响应(简化实现,需要根据实际响应格式完善)
    #[allow(dead_code)]
    fn parse_gps_response(&self, data: &[u8]) -> Result<Vec<crate::sync::models::LegacyGpsData>> {
        debug!("Parsing GPS response: {:02X?}", data);

        // TODO: 根据实际响应格式解析GPS数据
        // 这里返回空列表,实际需要解析响应数据
        Ok(Vec::new())
    }
}
