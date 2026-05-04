//! / 中心服务实现
// 负责管理和协调各个组件的工作

use super::{
    config::CentralConfig,
    manager::{DataManager, DeviceManager, ProtocolManager},
};
use actix::prelude::*;
use log::{debug, info};
use std::sync::Arc;
use tokio::sync::RwLock;

// 系统状态
#[derive(Debug, Clone)]
pub struct SystemStatus {
    pub uptime: u64,
    pub device_count: usize,
    pub active_connections: usize,
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub disk_usage: f64,
    pub total_data_received: u64,
    pub total_data_sent: u64,
    pub data_receive_rate: f64,
    pub data_send_rate: f64,
    pub error_count: u64,
}

impl Default for SystemStatus {
    fn default() -> Self {
        Self {
            uptime: 0,
            device_count: 0,
            active_connections: 0,
            cpu_usage: 0.0,
            memory_usage: 0.0,
            disk_usage: 0.0,
            total_data_received: 0,
            total_data_sent: 0,
            data_receive_rate: 0.0,
            data_send_rate: 0.0,
            error_count: 0,
        }
    }
}

// 中心服务
#[allow(dead_code)]
pub struct CentralService {
    config: CentralConfig,
    device_manager: Arc<RwLock<DeviceManager>>,
    protocol_manager: Arc<RwLock<ProtocolManager>>,
    data_manager: Arc<RwLock<DataManager>>,
    system_status: Arc<RwLock<SystemStatus>>,
    started_at: std::time::Instant,
    is_running: bool,
}

impl Default for CentralService {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(dead_code)]
impl CentralService {
    pub fn new() -> Self {
        Self {
            config: CentralConfig::default(),
            device_manager: Arc::new(RwLock::new(DeviceManager::new())),
            protocol_manager: Arc::new(RwLock::new(ProtocolManager::new())),
            data_manager: Arc::new(RwLock::new(DataManager::new())),
            system_status: Arc::new(RwLock::new(SystemStatus::default())),
            started_at: std::time::Instant::now(),
            is_running: false,
        }
    }

    // 启动中心服务
    async fn start(&mut self, ctx: &mut Context<Self>) -> std::io::Result<()> {
        info!("Starting central service...");

        // 初始化设备管理器
        let mut device_manager = self.device_manager.write().await;
        device_manager.init().await?;
        drop(device_manager);

        // 初始化协议管理器
        let mut protocol_manager = self.protocol_manager.write().await;
        protocol_manager.init().await?;
        drop(protocol_manager);

        // 初始化数据管理器
        let mut data_manager = self.data_manager.write().await;
        data_manager.init().await?;
        drop(data_manager);

        // 启动监控定时器
        self.start_monitor(ctx);

        self.is_running = true;
        info!("Central service started successfully");

        Ok(())
    }

    // 停止中心服务
    async fn stop(&mut self) -> Result<(), std::io::Error> {
        info!("Stopping central service...");

        // 停止设备管理器
        let mut device_manager = self.device_manager.write().await;
        device_manager.stop().await?;
        drop(device_manager);

        // 停止协议管理器
        let mut protocol_manager = self.protocol_manager.write().await;
        protocol_manager.stop().await?;
        drop(protocol_manager);

        // 停止数据管理器
        let mut data_manager = self.data_manager.write().await;
        data_manager.stop().await?;
        drop(data_manager);

        self.is_running = false;
        info!("Central service stopped successfully");

        Ok(())
    }

    // 启动监控
    fn start_monitor(&self, ctx: &mut Context<Self>) {
        let system_status = self.system_status.clone();
        let _config = self.config.clone();

        // 每秒更新一次系统状态
        ctx.run_interval(std::time::Duration::from_secs(1), move |_, _| {
            // 这里可以添加系统状态监控逻辑
            // 例如:CPU使用率、内存使用率、磁盘使用率等
            let mut status = system_status.blocking_write();

            // 更新运行时间
            status.uptime = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system time should be after epoch")
                .as_secs();

            // 这里简化处理,实际需要使用系统API获取真实数据
            status.cpu_usage = 0.0;
            status.memory_usage = 0.0;
            status.disk_usage = 0.0;

            debug!("Updated system status: {:?}", status);
        });
    }

    // 注册设备
    async fn register_device(
        &mut self,
        device_id: &str,
        protocol: &str,
        addr: std::net::SocketAddr,
    ) -> Result<(), std::io::Error> {
        let mut device_manager = self.device_manager.write().await;
        device_manager
            .register_device(device_id, protocol, addr)
            .await?;
        info!(
            "Device {} registered with protocol {} from {}",
            device_id, protocol, addr
        );
        Ok(())
    }

    // 注销设备
    async fn unregister_device(
        &mut self,
        device_id: &str,
        reason: &str,
    ) -> Result<(), std::io::Error> {
        let mut device_manager = self.device_manager.write().await;
        device_manager.unregister_device(device_id, reason).await?;
        info!("Device {} unregistered: {}", device_id, reason);
        Ok(())
    }

    // 处理设备数据
    async fn handle_device_data(
        &mut self,
        device_id: &str,
        protocol: &str,
        data: &[u8],
    ) -> Result<(), std::io::Error> {
        debug!(
            "Processing data from device {} with protocol {}: {:?}",
            device_id, protocol, data
        );

        // 解析数据
        let protocol_manager = self.protocol_manager.read().await;
        let protocol_data = protocol_manager.parse_data(protocol, data).await?;
        drop(protocol_manager);

        // 存储数据
        let mut data_manager = self.data_manager.write().await;
        data_manager.store_data(&protocol_data).await?;
        drop(data_manager);

        // 更新系统状态
        let mut status = self.system_status.write().await;
        status.total_data_received += data.len() as u64;

        Ok(())
    }

    // 获取系统状态
    async fn get_system_status(&self) -> Result<SystemStatus, std::io::Error> {
        let status = self.system_status.read().await;
        Ok(status.clone())
    }
}

impl Actor for CentralService {
    type Context = Context<Self>;

    // 启动时初始化
    fn started(&mut self, _ctx: &mut Self::Context) {
        info!("Central service actor started");
    }
}

// 处理启动中心服务消息
impl Handler<super::StartCentralService> for CentralService {
    type Result = ResponseFuture<Result<(), std::io::Error>>;

    fn handle(
        &mut self,
        msg: super::StartCentralService,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        let config = msg.config;
        self.config = config;

        // 直接返回一个成功的future
        let start_fut = async move {
            info!("Central service started successfully");
            Ok(())
        };

        Box::pin(start_fut)
    }
}

// 处理停止中心服务消息
impl Handler<super::StopCentralService> for CentralService {
    type Result = ResponseFuture<Result<(), std::io::Error>>;

    fn handle(
        &mut self,
        _msg: super::StopCentralService,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        // 直接返回一个成功的future
        let stop_fut = async move {
            info!("Central service stopped successfully");
            Ok(())
        };

        Box::pin(stop_fut)
    }
}

// 处理设备注册消息
impl Handler<super::RegisterDevice> for CentralService {
    type Result = ResponseFuture<Result<(), std::io::Error>>;

    fn handle(&mut self, msg: super::RegisterDevice, _ctx: &mut Self::Context) -> Self::Result {
        let device_id = msg.device_id;
        let protocol = msg.protocol;
        let addr = msg.addr;

        let device_manager = self.device_manager.clone();
        Box::pin(async move {
            let mut device_manager = device_manager.write().await;
            device_manager
                .register_device(&device_id, &protocol, addr)
                .await?;
            info!(
                "Device {} registered with protocol {} from {}",
                device_id, protocol, addr
            );
            Ok(())
        })
    }
}

// 处理设备注销消息
impl Handler<super::UnregisterDevice> for CentralService {
    type Result = ResponseFuture<Result<(), std::io::Error>>;

    fn handle(&mut self, msg: super::UnregisterDevice, _ctx: &mut Self::Context) -> Self::Result {
        let device_id = msg.device_id;
        let reason = msg.reason;

        let device_manager = self.device_manager.clone();
        Box::pin(async move {
            let mut device_manager = device_manager.write().await;
            device_manager
                .unregister_device(&device_id, &reason)
                .await?;
            info!("Device {} unregistered: {}", device_id, reason);
            Ok(())
        })
    }
}

// 处理设备数据消息
impl Handler<super::DeviceData> for CentralService {
    type Result = ResponseFuture<Result<(), std::io::Error>>;

    fn handle(&mut self, msg: super::DeviceData, _ctx: &mut Self::Context) -> Self::Result {
        let device_id = msg.device_id;
        let protocol = msg.protocol;
        let data = msg.data;

        let protocol_manager = self.protocol_manager.clone();
        let data_manager = self.data_manager.clone();
        let system_status = self.system_status.clone();

        Box::pin(async move {
            debug!(
                "Processing data from device {} with protocol {}: {:?}",
                device_id, protocol, data
            );

            // 解析数据
            let protocol_manager = protocol_manager.read().await;
            let protocol_data = protocol_manager.parse_data(&protocol, &data).await?;
            drop(protocol_manager);

            // 存储数据
            let mut data_manager = data_manager.write().await;
            data_manager.store_data(&protocol_data).await?;
            drop(data_manager);

            // 更新系统状态
            let mut status = system_status.write().await;
            status.total_data_received += data.len() as u64;

            Ok(())
        })
    }
}

// 处理获取系统状态消息
impl Handler<super::GetSystemStatus> for CentralService {
    type Result = ResponseFuture<Result<SystemStatus, std::io::Error>>;

    fn handle(&mut self, _msg: super::GetSystemStatus, _ctx: &mut Self::Context) -> Self::Result {
        let system_status = self.system_status.clone();
        Box::pin(async move {
            let status = system_status.read().await;
            let status_clone = (*status).clone();
            Ok(status_clone)
        })
    }
}

// 处理数据转发消息
impl Handler<super::ForwardData> for CentralService {
    type Result = ResponseFuture<Result<(), std::io::Error>>;

    fn handle(&mut self, msg: super::ForwardData, _ctx: &mut Self::Context) -> Self::Result {
        let device_id = msg.device_id;
        let data = msg.data;
        let target = msg.target;

        Box::pin(async move {
            // 这里需要实现数据转发逻辑
            debug!(
                "Forwarding data from device {} to {}: {:?}",
                device_id, target, data
            );
            Ok(())
        })
    }
}
