// 客户端服务端通讯服务器模块
// 负责处理来自客户端的TCP连接和数据

use log::{debug, error, info};  
use std::net::SocketAddr;  
use std::sync::Arc;  
use std::time::{Duration, Instant};  
use tokio::io::{AsyncReadExt, AsyncWriteExt};  
use tokio::net::{TcpListener, TcpStream};  
use tokio::sync::{Mutex, RwLock};  

// 客户端状态 - 使用分离的读写半部
pub struct ClientState {
    pub client_id: String,  
    pub addr: SocketAddr,  
    pub last_activity: Mutex<Instant>, // 使用Mutex保护可变字段
    pub write_half: Arc<Mutex<tokio::net::tcp::OwnedWriteHalf>>, // 可共享的写入半部
    pub buffer: Mutex<Vec<u8>>,  
    pub connection_time: Instant,  
    pub total_data_received: Mutex<u64>, // 使用Mutex保护可变字段
    pub total_data_sent: Mutex<u64>,     // 使用Mutex保护可变字段
}

// 客户端服务端通讯服务器
pub struct ClientServer {
    addr: SocketAddr,  
    listener: Option<TcpListener>,  
    clients: Arc<RwLock<Vec<Arc<ClientState>>>>, // 使用Vec存储所有客户端
    max_connections: usize,  
    cleanup_interval: Duration,  
    client_timeout: Duration,  
}

impl ClientServer {
    pub fn new(addr: SocketAddr, max_connections: usize) -> Self {
        Self {
            addr,
            listener: None,
            clients: Arc::new(RwLock::new(Vec::new())),
            max_connections,
            cleanup_interval: Duration::from_secs(60),
            client_timeout: Duration::from_secs(300),
        }
    }

    // 启动服务器
    pub async fn start(&mut self) -> Result<(), std::io::Error> {
        info!("Starting Client-Server Communication Server on {}", self.addr);

        let listener = TcpListener::bind(self.addr).await?;
        self.listener = Some(listener);

        info!("Client-Server Communication Server started successfully on {}", self.addr);
        Ok(())
    }

    // 运行服务器
    pub async fn run(&self) -> Result<(), std::io::Error> {
        let listener = self.listener.as_ref().expect("Client server not started");
        let clients = self.clients.clone();
        let max_connections = self.max_connections;
        let cleanup_interval = self.cleanup_interval;
        let client_timeout = self.client_timeout;

        // 启动连接清理任务
        tokio::spawn({
            let clients = clients.clone();
            async move {
                loop {
                    tokio::time::sleep(cleanup_interval).await;
                    Self::cleanup_stale_connections(clients.clone(), client_timeout).await;
                }
            }
        });

        loop {
            match listener.accept().await {
                Ok((mut stream, addr)) => {
                    info!("New client connection from {}", addr);

                    // 检查连接数是否超过上限
                    let current_connections = clients.read().await.len();
                    if current_connections >= max_connections {
                        error!(
                            "Maximum client connections reached, rejecting connection from {}",
                            addr
                        );
                        // 发送拒绝连接消息并关闭连接
                        if let Err(e) = stream.shutdown().await {
                            error!("Failed to shutdown client connection from {}: {}", addr, e);
                        }
                        continue;
                    }

                    // 处理新连接
                    let clients_clone = clients.clone();

                    tokio::spawn(async move {
                        if let Err(e) = Self::handle_connection(
                            stream,
                            addr,
                            clients_clone,
                        )
                        .await
                        {
                            error!("Failed to handle client connection from {}: {}", addr, e);
                        }
                    });
                }
                Err(e) => {
                    error!("Client server accept error: {}", e);
                    break;
                }
            }
        }

        Ok(())
    }

    // 处理单个客户端连接
    async fn handle_connection(
        stream: TcpStream,
        addr: SocketAddr,
        clients: Arc<RwLock<Vec<Arc<ClientState>>>>,
    ) -> Result<(), std::io::Error> {
        // 分离读写半部
        let (mut read_half, write_half) = stream.into_split();

        // 生成客户端ID
        let client_id = format!("client_{}_{}", addr.ip(), addr.port());

        info!("Client {} connected from {}", client_id, addr);

        // 创建客户端状态 - 将写入半部包装在 Arc<Mutex<>> 中以支持共享
        let client_state = Arc::new(ClientState {
            client_id: client_id.clone(),
            addr,
            last_activity: Mutex::new(Instant::now()),
            write_half: Arc::new(Mutex::new(write_half)), // 保存可共享的写入半部
            buffer: Mutex::new(Vec::with_capacity(4096)),
            connection_time: Instant::now(),
            total_data_received: Mutex::new(0),
            total_data_sent: Mutex::new(0),
        });

        // 添加客户端到列表
        {
            let mut clients_write = clients.write().await;
            clients_write.push(client_state.clone());
        }

        // 读取数据循环 - 使用 read_half
        let mut read_buf = [0u8; 4096];

        loop {
            let n = match read_half.read(&mut read_buf).await {
                Ok(n) => n,
                Err(e) => {
                    error!("Client read error from {}: {}", client_id, e);
                    break;
                }
            };

            if n == 0 {
                // 连接关闭
                break;
            }

            debug!(
                "Client received {} bytes from {}: {:?}",
                n,
                client_id,
                &read_buf[..n]
            );

            // 更新客户端状态
            {
                *client_state.last_activity.lock().await = Instant::now();
                let mut buffer_lock = client_state.buffer.lock().await;
                buffer_lock.extend_from_slice(&read_buf[..n]);
                *client_state.total_data_received.lock().await += n as u64;
            }

            // 处理接收到的数据
            let data = &read_buf[..n];
            Self::handle_client_data(&client_id, data, clients.clone()).await;
        }

        // 清理客户端资源
        {
            let mut clients_write = clients.write().await;
            clients_write.retain(|client| client.client_id != client_id);
        }

        info!("Client {} disconnected from {}", client_id, addr);
        Ok(())
    }

    // 处理客户端数据
    async fn handle_client_data(
        client_id: &str,
        data: &[u8],
        clients: Arc<RwLock<Vec<Arc<ClientState>>>>,
    ) {
        // 尝试将数据转换为字符串
        if let Ok(message) = String::from_utf8(data.to_vec()) {
            info!("Received message from client {}: {}", client_id, message.trim());

            // 处理不同类型的消息
            match message.trim() {
                "ping" => {
                    // 回复pong
                    Self::send_to_client(client_id, "pong".as_bytes(), clients.clone()).await;
                }
                "status" => {
                    // 发送服务器状态
                    let status = Self::get_server_status(clients.clone()).await;
                    Self::send_to_client(client_id, status.as_bytes(), clients.clone()).await;
                }
                _ => {
                    // 广播消息给所有客户端
                    Self::broadcast_message(client_id, message.trim(), clients.clone()).await;
                }
            }
        } else {
            debug!("Received binary data from client {}: {:?}", client_id, data);
            // 处理二进制数据
        }
    }

    // 发送数据到指定客户端
    async fn send_to_client(
        client_id: &str,
        data: &[u8],
        clients: Arc<RwLock<Vec<Arc<ClientState>>>>,
    ) {
        let clients_read = clients.read().await;
        for client in clients_read.iter() {
            if client.client_id == client_id {
                // 获取客户端写入半部并发送数据
                let mut write_half_lock = client.write_half.lock().await;
                if let Err(e) = write_half_lock.write_all(data).await {
                    error!("Failed to send data to client {}: {}", client_id, e);
                } else {
                    debug!("Sent {} bytes to client {}", data.len(), client_id);
                    *client.total_data_sent.lock().await += data.len() as u64;
                }
                break;
            }
        }
    }

    // 广播消息给所有客户端
    async fn broadcast_message(
        sender_id: &str,
        message: &str,
        clients: Arc<RwLock<Vec<Arc<ClientState>>>>,
    ) {
        let broadcast_msg = format!("[{}]: {}\n", sender_id, message);
        let clients_read = clients.read().await;
        
        for client in clients_read.iter() {
            // 不要发送给发送者自己
            if client.client_id != sender_id {
                let mut write_half_lock = client.write_half.lock().await;
                if let Err(e) = write_half_lock.write_all(broadcast_msg.as_bytes()).await {
                    error!("Failed to broadcast to client {}: {}", client.client_id, e);
                } else {
                    debug!("Broadcast to client {}", client.client_id);
                    *client.total_data_sent.lock().await += broadcast_msg.len() as u64;
                }
            }
        }
    }

    // 获取服务器状态
    async fn get_server_status(clients: Arc<RwLock<Vec<Arc<ClientState>>>>) -> String {
        let clients_read = clients.read().await;
        let active_connections = clients_read.len();
        
        let mut status = "Server Status:\n".to_string();
        status.push_str(&format!("Active connections: {}\n", active_connections));
        
        for client in clients_read.iter() {
            status.push_str(&format!("Client {}: {}\n", client.client_id, client.addr));
        }
        
        status
    }

    // 清理过期连接
    async fn cleanup_stale_connections(
        clients: Arc<RwLock<Vec<Arc<ClientState>>>>,
        timeout: Duration,
    ) {
        let now = Instant::now();
        let mut stale_client_ids = Vec::new();

        // 找出所有过期的连接
        {
            let clients_read = clients.read().await;
            for client in clients_read.iter() {
                if now.duration_since(*client.last_activity.lock().await) > timeout {
                    stale_client_ids.push(client.client_id.clone());
                }
            }
        }

        // 清理过期连接
        if !stale_client_ids.is_empty() {
            let mut clients_write = clients.write().await;
            clients_write.retain(|client| !stale_client_ids.contains(&client.client_id));
            
            for client_id in stale_client_ids {
                info!("Cleaned up stale client connection: {}", client_id);
            }
        }
    }
}

// 启动客户端服务端通讯服务器的函数
pub async fn start_client_server(addr: SocketAddr) {
    let mut server = ClientServer::new(addr, 100);
    
    if let Err(e) = server.start().await {
        error!("Failed to start client server: {}", e);
        return;
    }
    
    tokio::spawn(async move {
        if let Err(e) = server.run().await {
            error!("Client server error: {}", e);
        }
    });
}
