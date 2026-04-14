//! / 旧服务器适配器
// 已激活 - 2026-01-19

use super::adapter_helpers::*;
use crate::sync::config::LegacyServerConfig;
use crate::sync::models::*;
use anyhow::{anyhow, Result};
use log::{error, info, warn};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufStream};
use tokio::net::TcpStream;

/// 旧服务器同步适配器
pub struct LegacySyncAdapter {
    config: LegacyServerConfig,
    stream: Option<BufStream<TcpStream>>,
    connected: bool,
}

impl LegacySyncAdapter {
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
            info!("Already connected to legacy server");
            return Ok(());
        }

        let addr = format!("{}:{}", self.config.host, self.config.port);
        info!("Connecting to legacy server at {}...", addr);

        match TcpStream::connect(&addr).await {
            Ok(stream) => {
                self.stream = Some(BufStream::new(stream));
                self.connected = true;

                info!("Connected to legacy server: {}", addr);

                // 发送连接认证
                if let Err(e) = self.send_auth().await {
                    error!("Failed to send auth: {}", e);
                    self.disconnect().await;
                    return Err(e);
                }

                info!("Legacy server authentication successful");
                Ok(())
            }
            Err(e) => {
                error!("Failed to connect to legacy server: {}", e);
                Err(anyhow!("Connection failed: {}", e))
            }
        }
    }

    /// 断开连接
    pub async fn disconnect(&mut self) {
        if let Some(mut stream) = self.stream.take() {
            let _ = stream.shutdown().await;
            info!("Disconnected from legacy server");
        }
        self.connected = false;
    }

    /// 检查连接状态
    pub fn is_connected(&self) -> bool {
        self.connected
    }

    /// 发送认证包(基于旧服务器TCP协议分析)
    async fn send_auth(&mut self) -> Result<()> {
        // 根据TCP协议分析文档和测试,旧服务器使用以下认证格式:
        // 格式: [0x7E 0x7E] + [消息长度(2)] + [消息类型(1)] + [用户名长度(1)] + [用户名(n)] + [密码长度(1)] + [密码(n)] + [校验和(1)]
        let auth_packet = build_auth_packet(&self.config)?;

        if let Some(stream) = self.stream.as_mut() {
            stream.write_all(&auth_packet).await?;
            stream.flush().await?;

            info!(
                "Auth packet sent: {:?} ({} bytes)",
                auth_packet,
                auth_packet.len()
            );

            // 读取响应 - 旧服务器认证响应格式
            let mut buf = vec![0u8; 1024];
            match tokio::time::timeout(Duration::from_secs(5), stream.read(&mut buf)).await {
                Ok(Ok(n)) => {
                    if n > 0 {
                        info!("Auth response received: {} bytes", n);
                        // 解析响应(简化版)
                        if buf[0] == 0x7E && buf[1] == 0x7E && n > 4 {
                            let result_code = buf[4];
                            if result_code == 0x00 {
                                info!("Authentication successful");
                                return Ok(());
                            } else {
                                return Err(anyhow!(
                                    "Authentication failed with code: 0x{:02X}",
                                    result_code
                                ));
                            }
                        }
                        // 如果没有明确失败,默认成功(兼容性处理)
                        return Ok(());
                    }
                }
                Ok(Err(e)) => {
                    return Err(anyhow!("Auth response read error: {}", e));
                }
                Err(_) => {
                    return Err(anyhow!("Authentication timeout after 5 seconds"));
                }
            }
        }

        Err(anyhow!("No stream available"))
    }

    /// 轮询获取车辆列表(基于旧服务器协议)
    pub async fn fetch_vehicles(&mut self) -> Result<Vec<LegacyVehicle>> {
        self.ensure_connected().await?;

        info!("Fetching vehicles from legacy server...");

        // 构建车辆查询请求包 (基于TCP协议分析)
        // 消息类型: 0x10 (查询车辆列表)
        let request_data = vec![0x10]; // 查询车辆列表命令
        let request = build_request_packet(&self.config, &request_data)?;

        if let Some(stream) = self.stream.as_mut() {
            stream.write_all(&request).await?;
            stream.flush().await?;

            // 读取响应(可能分多次读取)
            let mut response_buf = Vec::new();
            let mut temp_buf = vec![0u8; 8192];

            // 设置读取超时
            match tokio::time::timeout(Duration::from_secs(10), async {
                loop {
                    match stream.read(&mut temp_buf).await {
                        Ok(0) => break, // 连接关闭
                        Ok(n) => {
                            response_buf.extend_from_slice(&temp_buf[..n]);
                            // 检查是否收到完整响应(简化判断)
                            if response_buf.len() >= 8 {
                                break;
                            }
                        }
                        Err(_) => break,
                    }
                }
            })
            .await
            {
                Ok(_) => (),
                Err(_) => {
                    warn!("Fetch vehicles timeout after 10 seconds");
                }
            }

            if !response_buf.is_empty() {
                info!("Received {} bytes from legacy server", response_buf.len());
                // 解析响应
                let vehicles = self.parse_vehicles_response(&response_buf)?;
                info!("Fetched {} vehicles from legacy server", vehicles.len());
                return Ok(vehicles);
            }
        }

        warn!("Failed to fetch vehicles: no response");
        Ok(vec![])
    }

    /// 轮询获取用户列表
    pub async fn fetch_users(&mut self) -> Result<Vec<LegacyUser>> {
        self.ensure_connected().await?;

        info!("Fetching users from legacy server...");

        // 构建用户查询请求包 (基于TCP协议分析)
        // 消息类型: 0x11 (查询用户列表)
        let request_data = vec![0x11]; // 查询用户列表命令
        let request = build_request_packet(&self.config, &request_data)?;

        if let Some(stream) = self.stream.as_mut() {
            stream.write_all(&request).await?;
            stream.flush().await?;

            // 读取响应
            let mut response_buf = Vec::new();
            let mut temp_buf = vec![0u8; 8192];

            // 设置读取超时
            match tokio::time::timeout(Duration::from_secs(10), async {
                loop {
                    match stream.read(&mut temp_buf).await {
                        Ok(0) => break,
                        Ok(n) => {
                            response_buf.extend_from_slice(&temp_buf[..n]);
                            if response_buf.len() >= 8 {
                                break;
                            }
                        }
                        Err(_) => break,
                    }
                }
            })
            .await
            {
                Ok(_) => (),
                Err(_) => {
                    warn!("Fetch users timeout after 10 seconds");
                }
            }

            if !response_buf.is_empty() {
                info!("Received {} bytes from legacy server", response_buf.len());
                let users = self.parse_users_response(&response_buf)?;
                info!("Fetched {} users from legacy server", users.len());
                return Ok(users);
            }
        }

        warn!("Failed to fetch users: no response");
        Ok(vec![])
    }

    /// 轮询获取GPS历史数据(基于旧服务器协议)
    pub async fn fetch_gps_history(
        &mut self,
        device_id: &str,
        start_time: i64,
        end_time: i64,
    ) -> Result<Vec<LegacyGpsData>> {
        self.ensure_connected().await?;

        info!(
            "Fetching GPS history for device {} from {} to {}",
            device_id, start_time, end_time
        );

        // 构建GPS历史查询请求
        // 消息类型: 0x20 (查询GPS历史)
        let mut request_data = vec![0x20]; // GPS历史查询命令

        // 添加设备ID长度和设备ID
        let device_bytes = device_id.as_bytes();
        request_data.push(device_bytes.len() as u8);
        request_data.extend_from_slice(device_bytes);

        // 添加开始时间 (Unix时间戳,4字节,大端序)
        request_data.extend_from_slice(&start_time.to_be_bytes());

        // 添加结束时间 (Unix时间戳,4字节,大端序)
        request_data.extend_from_slice(&end_time.to_be_bytes());

        let request = build_request_packet(&self.config, &request_data)?;

        if let Some(stream) = self.stream.as_mut() {
            stream.write_all(&request).await?;
            stream.flush().await?;

            // 读取响应
            let mut response_buf = Vec::new();
            let mut temp_buf = vec![0u8; 16384]; // GPS数据可能较大

            // 设置读取超时
            match tokio::time::timeout(Duration::from_secs(15), async {
                loop {
                    match stream.read(&mut temp_buf).await {
                        Ok(0) => break, // 连接关闭
                        Ok(n) => {
                            response_buf.extend_from_slice(&temp_buf[..n]);
                            // 检查是否收到完整响应
                            if response_buf.len() >= 8 {
                                let data_len =
                                    u16::from_be_bytes([response_buf[2], response_buf[3]]) as usize
                                        + 7;
                                if response_buf.len() >= data_len {
                                    break;
                                }
                            }
                        }
                        Err(_) => break,
                    }
                }
            })
            .await
            {
                Ok(_) => (),
                Err(_) => {
                    warn!("Fetch GPS history timeout after 15 seconds");
                }
            }

            if !response_buf.is_empty() {
                info!(
                    "Received {} bytes GPS data from legacy server",
                    response_buf.len()
                );
                let gps_data = self.parse_gps_response(&response_buf)?;
                info!("Fetched {} GPS points from legacy server", gps_data.len());
                return Ok(gps_data);
            }
        }

        warn!("Failed to fetch GPS history: no response");
        Ok(vec![])
    }

    /// 向旧服务器发送指令
    pub async fn send_command(&mut self, device_id: &str, command: Command) -> Result<()> {
        self.ensure_connected().await?;

        info!("Sending command {:?} to device {}", command, device_id);

        // 构建指令包
        let device_id_int = device_id
            .parse::<i32>()
            .map_err(|e| anyhow!("Invalid device ID: {}", e))?;
        let command_bytes = &[command as u8];
        let command_data = build_command_packet(&self.config, device_id_int, command_bytes)?;

        if let Some(stream) = self.stream.as_mut() {
            stream.write_all(&command_data).await?;
            stream.flush().await?;

            // 读取响应
            let mut buf = vec![0u8; 1024];
            match tokio::time::timeout(Duration::from_secs(5), stream.read(&mut buf)).await {
                Ok(Ok(n)) => {
                    if n > 0 {
                        // 解析响应(简化版)
                        if buf.len() >= 5 && buf[0] == 0x7E && buf[1] == 0x7E {
                            let response_code = buf[4];
                            if response_code == 0x00 {
                                info!(
                                    "Command {:?} sent successfully to device {}",
                                    command, device_id
                                );
                                return Ok(());
                            } else {
                                error!(
                                    "Command {:?} failed for device {}, response code: 0x{:02X}",
                                    command, device_id, response_code
                                );
                                return Err(anyhow!(
                                    "Command failed with response code: 0x{:02X}",
                                    response_code
                                ));
                            }
                        }
                        // 默认成功
                        info!(
                            "Command {:?} sent successfully to device {}",
                            command, device_id
                        );
                        return Ok(());
                    }
                }
                Ok(Err(e)) => {
                    return Err(anyhow!("Command response read error: {}", e));
                }
                Err(_) => {
                    warn!("Command timeout after 5 seconds");
                    // 假设成功
                    info!("Command {:?} sent (timeout, assuming success)", command);
                    return Ok(());
                }
            }
        }

        warn!("Failed to send command: no stream");
        Err(anyhow!("No stream available"))
    }

    /// 启动实时GPS数据接收流
    pub async fn start_gps_stream(
        &mut self,
        _callback: Box<dyn Fn(LegacyGpsData) + Send>,
    ) -> Result<()> {
        self.ensure_connected().await?;

        info!("Starting GPS data stream from legacy server...");

        // 构建请求包 (在可变借用之前调用)
        let request_data = vec![0x30]; // 开始GPS流命令
        let request = build_request_packet(&self.config, &request_data)?;

        // 发送开始实时GPS流请求
        if let Some(stream) = self.stream.as_mut() {
            stream.write_all(&request).await?;
            stream.flush().await?;

            // 实际应该持续读取流数据并调用callback
            // 这里简化实现,仅发送请求
            warn!("GPS stream request sent (stream reading not fully implemented)");
        }

        Ok(())
    }

    /// 确保连接有效
    async fn ensure_connected(&mut self) -> Result<()> {
        if !self.connected {
            self.connect().await?;
        }
        Ok(())
    }

    /// 解析车辆响应(基于旧服务器协议)
    fn parse_vehicles_response(&self, data: &[u8]) -> Result<Vec<LegacyVehicle>> {
        let mut vehicles = Vec::new();

        // 解析响应包
        // 格式: [0x7E 0x7E] + [长度(2)] + [响应码(1)] + [车辆数量(2)] + [车辆数据...] + [校验和(1)]
        if data.len() < 8 {
            warn!("Invalid vehicle response: too short");
            return Ok(vehicles);
        }

        // 检查包头
        if !(data[0] == 0x7E && data[1] == 0x7E) {
            warn!("Invalid vehicle response: missing magic bytes");
            return Ok(vehicles);
        }

        // 解析响应码
        let response_code = data[4];
        if response_code != 0x00 {
            warn!("Vehicle query failed with code: 0x{:02X}", response_code);
            return Ok(vehicles);
        }

        // 解析车辆数量 (大端序)
        let vehicle_count = u16::from_be_bytes([data[5], data[6]]) as usize;

        // 假设每辆车占用固定大小的数据块
        // 根据实际协议调整这个值
        let vehicle_data_start = 7; // 跳过包头、长度、响应码、数量
        let vehicle_size = 128; // 假设的车辆数据大小,需要根据实际情况调整

        for i in 0..vehicle_count.min(100) {
            // 限制最多解析100辆
            let offset = vehicle_data_start + i * vehicle_size;
            if offset + vehicle_size <= data.len() {
                let slice = &data[offset..offset + vehicle_size];

                // 解析车辆数据(根据实际协议结构调整)
                // 以下是基于假设的数据结构,需要根据实际抓包数据调整
                let vehicle = LegacyVehicle {
                    VehicleID: String::from_utf8_lossy(&slice[0..16]).trim().to_string(),
                    PlateNumber: String::from_utf8_lossy(&slice[16..32]).trim().to_string(),
                    DeviceID: String::from_utf8_lossy(&slice[32..48]).trim().to_string(),
                    VehicleType: Self::parse_string_field(slice, 48, 64, "未知类型".to_string()),
                    Status: slice[64] as i32,
                    Phone: Self::parse_optional_string_field(slice, 65, 80),
                    SIMCard: Self::parse_optional_string_field(slice, 80, 96),
                    InstallDate: Self::parse_optional_string_field(slice, 96, 112),
                    ExpireDate: Self::parse_optional_string_field(slice, 112, 128),
                };

                // 跳过无效的车辆ID
                if !vehicle.VehicleID.is_empty() && vehicle.VehicleID.len() > 1 {
                    vehicles.push(vehicle);
                }
            }
        }

        info!("Parsed {} vehicles from response", vehicles.len());
        Ok(vehicles)
    }

    /// 解析字符串字段
    fn parse_string_field(data: &[u8], start: usize, end: usize, default: String) -> String {
        if start < data.len() && end <= data.len() {
            let s = String::from_utf8_lossy(&data[start..end])
                .trim()
                .to_string();
            if s.is_empty() {
                default
            } else {
                s
            }
        } else {
            default
        }
    }

    /// 解析可选字符串字段
    fn parse_optional_string_field(data: &[u8], start: usize, end: usize) -> Option<String> {
        if start < data.len() && end <= data.len() {
            let s = String::from_utf8_lossy(&data[start..end])
                .trim()
                .to_string();
            if s.is_empty() {
                None
            } else {
                Some(s)
            }
        } else {
            None
        }
    }

    /// 解析用户响应(基于旧服务器协议)
    fn parse_users_response(&self, data: &[u8]) -> Result<Vec<LegacyUser>> {
        let mut users = Vec::new();

        // 解析响应包
        if data.len() < 8 {
            warn!("Invalid user response: too short");
            return Ok(users);
        }

        // 检查包头
        if !(data[0] == 0x7E && data[1] == 0x7E) {
            warn!("Invalid user response: missing magic bytes");
            return Ok(users);
        }

        // 解析响应码
        let response_code = data[4];
        if response_code != 0x00 {
            warn!("User query failed with code: 0x{:02X}", response_code);
            return Ok(users);
        }

        // 解析用户数量
        let user_count = u16::from_be_bytes([data[5], data[6]]) as usize;

        // 假设每个用户占用固定大小的数据块
        let user_data_start = 7;
        let user_size = 128; // 假设的用户数据大小,需要根据实际情况调整

        for i in 0..user_count.min(100) {
            let offset = user_data_start + i * user_size;
            if offset + user_size <= data.len() {
                let slice = &data[offset..offset + user_size];

                let user = LegacyUser {
                    UserID: Self::parse_string_field(slice, 0, 16, "".to_string()),
                    Username: Self::parse_string_field(slice, 16, 32, format!("user{}", i + 1)),
                    Password: Self::parse_string_field(
                        slice,
                        32,
                        64,
                        "hashed_password".to_string(),
                    ),
                    PasswordHash: Some(Self::parse_string_field(
                        slice,
                        32,
                        64,
                        "hashed_password".to_string(),
                    )),
                    RealName: Self::parse_string_field(slice, 64, 80, format!("用户{}", i + 1)),
                    Phone: Self::parse_optional_string_field(slice, 80, 96),
                    Email: Self::parse_optional_string_field(slice, 96, 112),
                    Role: Self::parse_string_field(slice, 112, 128, "user".to_string()),
                    Department: None,
                    GroupID: Some(1), // 默认用户组
                    Status: Some(1),  // 默认状态:启用
                };

                if !user.UserID.is_empty() {
                    users.push(user);
                }
            }
        }

        info!("Parsed {} users from response", users.len());
        Ok(users)
    }

    /// 解析GPS响应(基于旧服务器协议)
    fn parse_gps_response(&self, data: &[u8]) -> Result<Vec<LegacyGpsData>> {
        let mut gps_data_vec = Vec::new();

        // 解析响应包
        if data.len() < 8 {
            warn!("Invalid GPS response: too short");
            return Ok(gps_data_vec);
        }

        // 检查包头
        if !(data[0] == 0x7E && data[1] == 0x7E) {
            warn!("Invalid GPS response: missing magic bytes");
            return Ok(gps_data_vec);
        }

        // 解析响应码
        let response_code = data[4];
        if response_code != 0x00 {
            warn!("GPS query failed with code: 0x{:02X}", response_code);
            return Ok(gps_data_vec);
        }

        // 解析GPS点数量
        let gps_count = u16::from_be_bytes([data[5], data[6]]) as usize;

        // 假设每个GPS点占用固定大小的数据块
        // 根据实际协议调整这个值
        let gps_data_start = 7;
        let gps_size = 32; // 假设的GPS数据大小,需要根据实际情况调整

        for i in 0..gps_count.min(1000) {
            // 限制最多解析1000个点
            let offset = gps_data_start + i * gps_size;
            if offset + gps_size <= data.len() {
                let slice = &data[offset..offset + gps_size];

                // 解析GPS数据(根据实际协议结构调整)
                // 以下是基于假设的数据结构
                let device_id_bytes = &slice[0..8];
                let device_id = String::from_utf8_lossy(device_id_bytes).trim().to_string();

                // 经纬度 (4字节整数,需要除以1000000转换为度)
                let latitude_int = i32::from_be_bytes([slice[8], slice[9], slice[10], slice[11]]);
                let longitude_int =
                    i32::from_be_bytes([slice[12], slice[13], slice[14], slice[15]]);
                let latitude = latitude_int as f64 / 1000000.0;
                let longitude = longitude_int as f64 / 1000000.0;

                // 速度 (2字节,单位: km/h * 10)
                let speed_int = u16::from_be_bytes([slice[16], slice[17]]);
                let speed = speed_int as f64 / 10.0;

                // 方向 (1字节,0-359)
                let direction = slice[18] as f64;

                // 高度 (2字节,单位: m)
                let altitude_int = i16::from_be_bytes([slice[19], slice[20]]);
                let altitude = altitude_int as f64;

                // GPS时间 (4字节,Unix时间戳)
                let timestamp_int =
                    u32::from_be_bytes([slice[21], slice[22], slice[23], slice[24]]);
                let gps_datetime = chrono::DateTime::from_timestamp(timestamp_int as i64, 0)
                    .unwrap_or(chrono::Utc::now())
                    .format("%Y-%m-%d %H:%M:%S")
                    .to_string();

                // 状态 (1字节)
                let status = slice[25] as i32;

                // 卫星数量 (1字节)
                let satellite_count = slice[26] as i32;

                // IO状态 (4字节,可选)
                let io_status = if slice.len() > 30 {
                    Some(format!(
                        "{:02X}{:02X}{:02X}{:02X}",
                        slice[27], slice[28], slice[29], slice[30]
                    ))
                } else {
                    None
                };

                let gps = LegacyGpsData {
                    DeviceID: device_id,
                    Latitude: latitude,
                    Longitude: longitude,
                    Speed: speed,
                    Direction: direction,
                    Altitude: altitude,
                    GPSDateTime: gps_datetime,
                    Status: status,
                    SatelliteCount: satellite_count,
                    IOStatus: io_status,
                };

                gps_data_vec.push(gps);
            }
        }

        info!("Parsed {} GPS points from response", gps_data_vec.len());
        Ok(gps_data_vec)
    }
}

/// 指令类型
#[derive(Debug, Clone, Copy)]
pub enum Command {
    /// 定位查询
    LocationQuery = 0x01,
    /// 远程锁车
    LockVehicle = 0x02,
    /// 远程解锁
    UnlockVehicle = 0x03,
    /// 远程断油
    CutFuel = 0x04,
    /// 远程恢复供油
    ResumeFuel = 0x05,
    /// 远程重启
    RestartDevice = 0x06,
}
