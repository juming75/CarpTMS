//! /! 服务器初始化模块

use crate::central::config::CentralConfig;
use crate::gateway::client_api_server::ClientApiServer;
use crate::vehicle_comm::server::VehicleCommServer;
use actix::Actor;
use log::{error, info};
use sqlx::postgres::PgPool;
use std::net::SocketAddr;

/// 启动客户端服务端通讯服务器(复用 ClientApiServer)
///
/// 等价替换已删除的 client_server::start_client_server,
/// 使用同一 ClientApiServer 实现(9808端口)。
pub async fn start_client_server_service() {
    info!("Initializing Client-Server Communication Server...");

    let addr: SocketAddr = "0.0.0.0:9808".parse().expect("hardcoded addr always valid");
    let server = ClientApiServer::new(addr, 1000);

    // 启动服务器
    actix::spawn(async move {
        server.run().await;
        info!("Client-Server Communication Server exited");
    });

    info!("Client-Server Communication Server started successfully on port 9808");
}

/// 启动车联网通讯服务器
///
/// # 参数
/// - `pool` - 数据库连接池
/// - `central_config` - 中心服务配置
pub async fn start_vehicle_comm_server(pool: &PgPool, _central_config: &CentralConfig) {
    info!("Initializing Vehicle Communication Server...");

    // 车联网通讯服务器配置
    let server_addr: SocketAddr = "0.0.0.0:8988".parse().expect("hardcoded addr always valid");
    let max_connections = 1000;
    let pool_clone = pool.clone();

    // 使用actix::spawn而不是tokio::spawn
    actix::spawn(async move {
        let vehicle_comm_server =
            VehicleCommServer::new(server_addr, max_connections, Some(pool_clone));

        // 启动Actor
        let server_addr = vehicle_comm_server.start();
        info!("Starting Vehicle Communication Server on port 8988...");

        // 发送StartServer消息
        if let Err(e) = server_addr
            .send(crate::vehicle_comm::server::StartServer)
            .await
        {
            error!("Vehicle Communication Server error: {}", e);
        } else {
            info!("Vehicle Communication Server started successfully");
        }
    });
}

/// 启动UDP服务器
///
/// # 参数
/// - `central_config` - 中心服务配置
pub async fn start_udp_server(_central_config: &CentralConfig) {
    info!("UDP server initialization skipped for now");
    // 暂时禁用UDP服务器启动,避免Tokio运行时问题
    // 后续需要修复UDP服务器的Tokio运行时上下文问题
    //
    // info!("Initializing UDP server...");
    //
    // // 初始化中心服务
    // let central_service = crate::central::service::CentralService::new();
    // let central_addr = central_service.start();
    //
    // // 启动中心服务
    // central_addr.do_send(crate::central::StartCentralService {
    //     config: central_config.clone(),
    // });
    //
    // // 初始化协议分析器
    // let protocol_analyzer = crate::gateway::gprs_server::ProtocolAnalyzer::new();
    // let protocol_analyzer_addr = protocol_analyzer.start();
    //
    // // 初始化并启动UDP服务器
    // let udp_server = crate::gateway::udp_server::UdpServer::new(
    //     central_config.gateway_config.udp_server_addr,
    //     protocol_analyzer_addr.clone(),
    //     Some(central_addr.clone()),
    // );
    //
    // let _udp_server_addr = udp_server.start();
    // info!(
    //     "UDP server started successfully on {}",
    //     central_config.gateway_config.udp_server_addr
    // );
}

/// 启动负载均衡相关任务
pub async fn start_load_balancing_tasks() {
    info!("Starting load balancing tasks...");

    // 启动服务注册中心心跳清理任务
    actix::spawn(async move {
        crate::load_balancing::SERVICE_REGISTRY
            .start_heartbeat_cleanup()
            .await;
    });

    // 启动服务注册中心健康检查任务
    actix::spawn(async move {
        crate::load_balancing::SERVICE_REGISTRY
            .start_health_checks()
            .await;
    });

    // 启动扩缩容监控任务
    actix::spawn(async move {
        crate::load_balancing::SCALING_MANAGER
            .start_scaling_monitor()
            .await;
    });

    info!("Load balancing tasks started successfully");
}
