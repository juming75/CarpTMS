//! /! 网关服务初始化模块
//!
//! 负责启动TCP、UDP、WebSocket、GPRS等网关服务

use actix::Actor;
use log::info;
use carptms::{
    gateway, central, truck_scale, infrastructure::message_router,
};

/// 网关服务句柄
pub struct GatewayHandles {
    pub gprs_server: Option<actix::Addr<gateway::gprs_server::GprsServer>>,
    pub tcp_server: Option<actix::Addr<gateway::tcp_server::TcpServer>>,
    pub tcp_server2: Option<actix::Addr<gateway::tcp_server::TcpServer>>,
    pub udp_server: Option<actix::Addr<gateway::udp_server::UdpServer>>,
}

/// 初始化并启动所有网关服务
pub async fn init_gateways(
    central_config: &central::config::CentralConfig,
    pool: &sqlx::PgPool,
    central_addr: actix::Addr<central::service::CentralService>,
) -> Result<GatewayHandles, Box<dyn std::error::Error>> {
    // 初始化协议分析器
    let protocol_analyzer = gateway::gprs_server::ProtocolAnalyzer::new();
    let protocol_analyzer_addr = protocol_analyzer.start();

    // 初始化 TCP 会话管理器
    let tcp_session_manager = infrastructure::message_router::tcp_session::TcpSessionManager::default();
    let tcp_session_manager_addr = tcp_session_manager.start();

    // 初始化消息路由器
    info!("Initializing message router...");
    let message_router = infrastructure::message_router::router::MessageRouter::new(tcp_session_manager_addr);
    let message_router_addr = message_router.start();

    // 初始化 JT808 会话管理器
    info!("Initializing JT808 session manager...");
    let jt808_session_manager = protocols::jt808::session::Jt808SessionManager::new();
    let jt808_session_manager_addr = jt808_session_manager.start();

    // 初始化并启动GPRS服务器
    let gprs_server = gateway::gprs_server::GprsServer::new(
        central_config.gateway_config.gprs_server_addr,
        central_config.gateway_config.max_connections,
        protocol_analyzer_addr.clone(),
        Some(central_addr.clone()),
    );
    let gprs_server_addr = gprs_server.start();

    // 初始化并启动Truck Scale服务器
    info!("Initializing Truck Scale protocol adapter...");
    let truck_scale_config = truck_scale::TruckScaleConfig::from_env().unwrap_or_default();
    let truck_scale_pool = pool.clone();
    tokio::spawn(async move {
        let mut truck_scale_server = truck_scale::TruckScaleServer::new(truck_scale_config, truck_scale_pool);
        info!("Starting Truck Scale server on port 9808...");
        if let Err(e) = truck_scale_server.start().await {
            log::error!("Truck Scale server error: {}", e);
        }
    });

    // 初始化并启动TCP服务器(端口8988)
    let tcp_server = gateway::tcp_server::TcpServer::new(
        central_config.gateway_config.tcp_server_addr,
        central_config.gateway_config.max_connections,
        protocol_analyzer_addr.clone(),
        Some(central_addr.clone()),
        Some(message_router_addr.clone()),
        Some(jt808_session_manager_addr.clone()),
    )
    .with_message_router(message_router_addr.clone())
    .with_session_manager(jt808_session_manager_addr.clone());
    let tcp_server_addr = tcp_server.start();

    // 初始化并启动第二个TCP服务器(端口8085,用于车载终端/无人机/物联网终端)
    let tcp_server2 = gateway::tcp_server::TcpServer::new(
        central_config.gateway_config.tcp_server_addr2,
        central_config.gateway_config.max_connections,
        protocol_analyzer_addr.clone(),
        Some(central_addr.clone()),
        Some(message_router_addr.clone()),
        Some(jt808_session_manager_addr.clone()),
    )
    .with_message_router(message_router_addr.clone())
    .with_session_manager(jt808_session_manager_addr.clone());
    let tcp_server_addr2 = tcp_server2.start();

    // 初始化并启动UDP服务器
    let udp_server = gateway::udp_server::UdpServer::new(
        central_config.gateway_config.udp_server_addr,
        protocol_analyzer_addr.clone(),
        Some(central_addr.clone()),
    );
    let udp_server_addr = udp_server.start();

    info!("GPRS gateway services started successfully");
    info!(
        "GPRS server: {}",
        central_config.gateway_config.gprs_server_addr
    );
    info!(
        "TCP server: {}",
        central_config.gateway_config.tcp_server_addr
    );
    info!(
        "UDP server: {}",
        central_config.gateway_config.udp_server_addr
    );
    info!(
        "TCP server 2: {}",
        central_config.gateway_config.tcp_server_addr2
    );

    Ok(GatewayHandles {
        gprs_server: Some(gprs_server_addr),
        tcp_server: Some(tcp_server_addr),
        tcp_server2: Some(tcp_server_addr2),
        udp_server: Some(udp_server_addr),
    })
}

/// 启动服务注册相关任务
pub fn start_service_registry_tasks() {
    // 启动服务注册中心心跳清理任务
    tokio::spawn(async move {
        tms_server::load_balancing::SERVICE_REGISTRY
            .start_heartbeat_cleanup()
            .await;
    });

    // 启动服务注册中心健康检查任务
    tokio::spawn(async move {
        tms_server::load_balancing::SERVICE_REGISTRY
            .start_health_checks()
            .await;
    });

    // 启动扩缩容监控任务
    tokio::spawn(async move {
        tms_server::load_balancing::SCALING_MANAGER
            .start_scaling_monitor()
            .await;
    });
}







