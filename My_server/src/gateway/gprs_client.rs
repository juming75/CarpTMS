//! / GPRS客户端模块
// 负责与GPRS设备建立连接并处理数据

use super::{DeviceConnected, DeviceData, DeviceDisconnected};
use actix::prelude::*;
use log::{debug, error, info};
use std::net::SocketAddr;
use std::time::{Duration, Instant};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

// GPRS客户端状态
#[derive(Debug, Clone, PartialEq)]
pub enum GprsClientState {
    Connecting,
    Connected,
    Authenticated,
    Disconnected,
}

// GPRS信号质量
#[derive(Debug, Clone, PartialEq)]
pub enum SignalQuality {
    Excellent,
    Good,
    Fair,
    Poor,
    None,
}

// GPRS客户端
pub struct GprsClient {
    device_id: String,
    addr: SocketAddr,
    state: GprsClientState,
    stream: Option<TcpStream>,
    buffer: Vec<u8>,
    last_heartbeat: Instant,
    heartbeat_interval: Duration,
    protocol: String,
    reconnection_attempts: u32,
    max_reconnection_attempts: u32,
    reconnection_delay: Duration,
    signal_quality: SignalQuality,
    protocol_analyzer: Addr<super::gprs_server::ProtocolAnalyzer>,
}

impl GprsClient {
    pub fn new(
        device_id: String,
        addr: SocketAddr,
        protocol: String,
        protocol_analyzer: Addr<super::gprs_server::ProtocolAnalyzer>,
    ) -> Self {
        Self {
            device_id,
            addr,
            state: GprsClientState::Connecting,
            stream: None,
            buffer: Vec::with_capacity(4096),
            last_heartbeat: Instant::now(),
            heartbeat_interval: Duration::from_secs(30),
            protocol,
            reconnection_attempts: 0,
            max_reconnection_attempts: 5,
            reconnection_delay: Duration::from_secs(5),
            signal_quality: SignalQuality::None,
            protocol_analyzer,
        }
    }

    // 连接到GPRS服务器
    pub async fn connect(&mut self) -> Result<(), std::io::Error> {
        info!(
            "Connecting to GPRS server at {} for device {}",
            self.addr, self.device_id
        );

        let stream = TcpStream::connect(self.addr).await?;
        self.stream = Some(stream);
        self.state = GprsClientState::Connected;
        self.reconnection_attempts = 0;

        info!("Connected to GPRS server for device {}", self.device_id);
        Ok(())
    }

    // 断开连接
    pub async fn disconnect(&mut self) -> Result<(), std::io::Error> {
        if let Some(mut stream) = self.stream.take() {
            stream.shutdown().await?;
        }
        self.state = GprsClientState::Disconnected;
        info!(
            "Disconnected from GPRS server for device {}",
            self.device_id
        );
        Ok(())
    }

    // 重连机制
    pub async fn reconnect(&mut self) -> Result<(), std::io::Error> {
        if self.reconnection_attempts >= self.max_reconnection_attempts {
            error!(
                "Max reconnection attempts reached for device {}",
                self.device_id
            );
            self.state = GprsClientState::Disconnected;
            return Err(std::io::Error::other("Max reconnection attempts reached"));
        }

        self.reconnection_attempts += 1;
        info!(
            "Attempting to reconnect to GPRS server for device {} (attempt {}/{})",
            self.device_id, self.reconnection_attempts, self.max_reconnection_attempts
        );

        // 指数退避算法
        let delay = self.reconnection_delay * (2u32.pow(self.reconnection_attempts - 1));
        tokio::time::sleep(delay).await;

        self.connect().await
    }

    // 发送数据到GPRS设备
    pub async fn send_data(&mut self, data: &[u8]) -> Result<(), std::io::Error> {
        if let Some(stream) = &mut self.stream {
            stream.write_all(data).await?;
            debug!("Sent data to device {}: {:?}", self.device_id, data);
            Ok(())
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                "Not connected to device",
            ))
        }
    }

    // 接收数据
    pub async fn recv_data(&mut self) -> Result<Option<Vec<u8>>, std::io::Error> {
        if let Some(stream) = &mut self.stream {
            let mut read_buf = [0u8; 4096];
            let n = stream.read(&mut read_buf).await?;

            if n == 0 {
                // 连接关闭
                return Ok(None);
            }

            self.buffer.extend_from_slice(&read_buf[..n]);
            debug!(
                "Received {} bytes from device {}: {:?}",
                n,
                self.device_id,
                &read_buf[..n]
            );

            // 简单的协议帧解析(根据协议类型可能需要更复杂的解析)
            // 这里假设每个完整的数据包以0x7e开头和结尾
            if self.buffer.starts_with(&[0x7e]) && self.buffer.ends_with(&[0x7e]) {
                let packet = self.buffer.clone();
                self.buffer.clear();
                Ok(Some(packet))
            } else {
                // 数据不完整,继续接收
                Ok(None)
            }
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                "Not connected to device",
            ))
        }
    }

    // 发送心跳包
    pub async fn send_heartbeat(&mut self) -> Result<(), std::io::Error> {
        // 先生成心跳包,避免可变借用冲突
        let heartbeat_packet = self.generate_heartbeat_packet();

        if let Some(stream) = &mut self.stream {
            stream.write_all(&heartbeat_packet).await?;
            self.last_heartbeat = Instant::now();
            debug!("Sent heartbeat to device {}", self.device_id);
            Ok(())
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                "Not connected to device",
            ))
        }
    }

    // 生成心跳包
    fn generate_heartbeat_packet(&self) -> Vec<u8> {
        // 根据不同协议生成不同的心跳包
        match self.protocol.as_str() {
            "GB" => self.generate_gb_heartbeat(),
            "BSJ" => self.generate_bsj_heartbeat(),
            "DB44" => self.generate_db44_heartbeat(),
            _ => vec![0x7e, 0x01, 0x00, 0x01, 0x7e], // 默认心跳包
        }
    }

    // 生成GB协议心跳包
    fn generate_gb_heartbeat(&self) -> Vec<u8> {
        // GB/T 19056协议心跳包示例
        vec![0x7e, 0x01, 0x00, 0x01, 0x7e]
    }

    // 生成BSJ协议心跳包
    fn generate_bsj_heartbeat(&self) -> Vec<u8> {
        // BSJ协议心跳包示例
        vec![0x7e, 0x02, 0x00, 0x01, 0x7e]
    }

    // 生成DB44协议心跳包
    fn generate_db44_heartbeat(&self) -> Vec<u8> {
        // DB44协议心跳包示例
        vec![0x7e, 0x03, 0x00, 0x01, 0x7e]
    }

    // 检查是否需要发送心跳包
    pub fn need_heartbeat(&self) -> bool {
        Instant::now().duration_since(self.last_heartbeat) > self.heartbeat_interval
    }

    // 监测信号质量
    pub fn update_signal_quality(&mut self, rssi: i32, ber: u32) {
        // 根据RSSI和BER值更新信号质量
        self.signal_quality = match rssi {
            -50..=0 => SignalQuality::Excellent,
            -70..=-51 => SignalQuality::Good,
            -85..=-71 => SignalQuality::Fair,
            -100..=-86 => SignalQuality::Poor,
            _ => SignalQuality::None,
        };

        info!(
            "Signal quality for device {}: {:?} (RSSI: {} dBm, BER: {})
",
            self.device_id, self.signal_quality, rssi, ber
        );
    }

    // 主处理循环
    pub async fn run(&mut self) -> Result<(), std::io::Error> {
        // 连接到设备
        if let Err(e) = self.connect().await {
            error!("Failed to connect to GPRS device {}: {}", self.device_id, e);
            return self.reconnect().await;
        }

        // 发送设备连接消息
        self.protocol_analyzer.do_send(DeviceConnected {
            device_id: self.device_id.clone(),
            addr: self.addr,
            protocol: self.protocol.clone(),
        });

        // 主循环
        loop {
            // 复制心跳间隔,避免借用冲突
            let heartbeat_interval = self.heartbeat_interval;

            tokio::select! {
                // 接收数据
                result = self.recv_data() => {
                    match result {
                        Ok(Some(data)) => {
                            // 处理接收到的数据
                            self.protocol_analyzer.do_send(DeviceData {
                                device_id: self.device_id.clone(),
                                data: data.clone(),
                                protocol: self.protocol.clone(),
                            });

                            // 解析信号质量(如果数据中包含)
                            // 这里简化处理,实际应该根据协议解析
                            self.update_signal_quality(-60, 0);
                        },
                        Ok(None) => {
                            // 连接关闭,尝试重连
                            error!("Connection closed by GPRS device {}", self.device_id);
                            if let Err(e) = self.reconnect().await {
                                error!("Failed to reconnect to GPRS device {}: {}", self.device_id, e);
                                break;
                            }
                        },
                        Err(e) => {
                            error!("Error receiving data from GPRS device {}: {}", self.device_id, e);
                            if let Err(reconnect_err) = self.reconnect().await {
                                error!("Failed to reconnect to GPRS device {}: {}", self.device_id, reconnect_err);
                                break;
                            }
                        }
                    }
                },
                // 发送心跳包
                _ = tokio::time::sleep(heartbeat_interval) => {
                    if self.need_heartbeat() {
                        if let Err(e) = self.send_heartbeat().await {
                            error!("Failed to send heartbeat to GPRS device {}: {}", self.device_id, e);
                            if let Err(reconnect_err) = self.reconnect().await {
                                error!("Failed to reconnect to GPRS device {}: {}", self.device_id, reconnect_err);
                                break;
                            }
                        }
                    }
                }
            }
        }

        // 断开连接并发送断开消息
        self.disconnect().await?;
        self.protocol_analyzer.do_send(DeviceDisconnected {
            device_id: self.device_id.clone(),
            reason: "Connection lost".to_string(),
        });

        Ok(())
    }
}

impl Actor for GprsClient {
    type Context = Context<Self>;

    // 启动时连接到GPRS服务器
    fn started(&mut self, _ctx: &mut Self::Context) {
        info!("GPRS client started for device {}", self.device_id);

        // 启动主处理循环
        let device_id = self.device_id.clone();
        let addr = self.addr;
        let protocol = self.protocol.clone();
        let protocol_analyzer = self.protocol_analyzer.clone();

        tokio::spawn(async move {
            let mut client = GprsClient::new(device_id, addr, protocol, protocol_analyzer);
            if let Err(e) = client.run().await {
                error!("GPRS client main loop exited with error: {}", e);
            }
        });
    }
}

// 发送数据消息
#[derive(Message)]
#[rtype(result = "Result<(), std::io::Error>")]
pub struct SendData {
    pub device_id: String,
    pub data: Vec<u8>,
}

// 实现消息处理
impl Handler<SendData> for GprsClient {
    type Result = ResponseFuture<Result<(), std::io::Error>>;

    fn handle(&mut self, msg: SendData, _ctx: &mut Self::Context) -> Self::Result {
        // 处理发送数据请求
        let stream = self.stream.take();
        let device_id = self.device_id.clone();
        let data = msg.data;

        Box::pin(async move {
            if let Some(mut stream) = stream {
                stream.write_all(&data).await?;
                debug!("Sent data to GPRS device {}: {:?}", device_id, data);
                Ok(())
            } else {
                error!(
                    "Failed to send data to GPRS device {}: Not connected",
                    device_id
                );
                Err(std::io::Error::new(
                    std::io::ErrorKind::NotConnected,
                    "Not connected to device",
                ))
            }
        })
    }
}
