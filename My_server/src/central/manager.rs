//! / 中心服务管理器模块
// 负责具体的业务逻辑管理

use log::{debug, error, info};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;

// 设备管理器
pub struct DeviceManager {
    devices: HashMap<String, DeviceInfo>,
    connections: HashMap<String, ConnectionInfo>,
    max_connections: usize,
}

// 设备信息
#[allow(dead_code)]
pub struct DeviceInfo {
    device_id: String,
    protocol: String,
    addr: SocketAddr,
    status: DeviceStatus,
    last_activity: std::time::Instant,
    total_data_received: u64,
    total_data_sent: u64,
}

// 设备状态
#[derive(Debug, Clone, PartialEq)]
pub enum DeviceStatus {
    Online,
    Offline,
    Connecting,
    Disconnecting,
    Error,
}

// 连接信息
#[allow(dead_code)]
pub struct ConnectionInfo {
    conn_id: String,
    device_id: String,
    addr: SocketAddr,
    start_time: std::time::Instant,
    last_activity: std::time::Instant,
    total_data_received: u64,
    total_data_sent: u64,
}

impl Default for DeviceManager {
    fn default() -> Self {
        Self::new()
    }
}

impl DeviceManager {
    pub fn new() -> Self {
        Self {
            devices: HashMap::new(),
            connections: HashMap::new(),
            max_connections: 1000,
        }
    }

    // 初始化
    pub async fn init(&mut self) -> Result<(), std::io::Error> {
        info!("Initializing device manager...");
        // 这里可以添加初始化逻辑,例如从数据库加载设备信息
        Ok(())
    }

    // 停止
    pub async fn stop(&mut self) -> Result<(), std::io::Error> {
        info!("Stopping device manager...");
        // 这里可以添加停止逻辑,例如关闭所有连接
        Ok(())
    }

    // 注册设备
    pub async fn register_device(
        &mut self,
        device_id: &str,
        protocol: &str,
        addr: SocketAddr,
    ) -> Result<(), std::io::Error> {
        // 检查连接数是否超过上限
        if self.connections.len() >= self.max_connections {
            return Err(std::io::Error::new(
                std::io::ErrorKind::ConnectionRefused,
                "Maximum connections reached",
            ));
        }

        // 创建设备信息
        let device_info = DeviceInfo {
            device_id: device_id.to_string(),
            protocol: protocol.to_string(),
            addr,
            status: DeviceStatus::Online,
            last_activity: std::time::Instant::now(),
            total_data_received: 0,
            total_data_sent: 0,
        };

        // 添加设备
        self.devices.insert(device_id.to_string(), device_info);

        // 创建连接信息
        let conn_id = format!("conn-{}-{:?}", device_id, std::time::Instant::now());
        let connection_info = ConnectionInfo {
            conn_id: conn_id.clone(),
            device_id: device_id.to_string(),
            addr,
            start_time: std::time::Instant::now(),
            last_activity: std::time::Instant::now(),
            total_data_received: 0,
            total_data_sent: 0,
        };

        // 添加连接
        self.connections.insert(conn_id, connection_info);

        Ok(())
    }

    // 注销设备
    pub async fn unregister_device(
        &mut self,
        device_id: &str,
        _reason: &str,
    ) -> Result<(), std::io::Error> {
        // 更新设备状态
        if let Some(device) = self.devices.get_mut(device_id) {
            device.status = DeviceStatus::Offline;
            device.last_activity = std::time::Instant::now();
        }

        // 移除相关连接
        self.connections
            .retain(|_, conn| conn.device_id != device_id);

        Ok(())
    }

    // 获取设备信息
    pub async fn get_device(&self, device_id: &str) -> Option<&DeviceInfo> {
        self.devices.get(device_id)
    }

    // 获取所有设备
    pub async fn get_all_devices(&self) -> Vec<&DeviceInfo> {
        self.devices.values().collect()
    }

    // 获取设备数量
    pub async fn get_device_count(&self) -> usize {
        self.devices.len()
    }

    // 获取连接数量
    pub async fn get_connection_count(&self) -> usize {
        self.connections.len()
    }

    // 更新设备活动时间
    pub async fn update_device_activity(&mut self, device_id: &str) -> Result<(), std::io::Error> {
        if let Some(device) = self.devices.get_mut(device_id) {
            device.last_activity = std::time::Instant::now();
        }
        Ok(())
    }
}

// 协议管理器
pub struct ProtocolManager {
    // 暂时注释掉protocols字段,因为ProtocolHandler trait已注释
    // protocols: HashMap<String, Arc<dyn ProtocolHandler>>,
    protocol_factory: Option<Arc<crate::protocols::ProtocolFactory>>,
}

// 暂时注释掉ProtocolHandler,因为async trait方法不支持dyn对象
/*
pub trait ProtocolHandler: Send + Sync + 'static {
    // 解析数据
    async fn parse(&self, data: &[u8]) -> Result<crate::protocols::base::ProtocolData, crate::protocols::base::ProtocolError>;

    // 生成数据
    async fn generate(&self, data: &crate::protocols::base::ProtocolData) -> Result<Vec<u8>, crate::protocols::base::ProtocolError>;

    // 获取协议名称
    fn name(&self) -> &str;

    // 获取协议版本
    fn version(&self) -> &str;
}
*/

impl Default for ProtocolManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ProtocolManager {
    pub fn new() -> Self {
        Self {
            // protocols: HashMap::new(),
            protocol_factory: None,
        }
    }

    // 初始化
    pub async fn init(&mut self) -> Result<(), std::io::Error> {
        info!("Initializing protocol manager...");

        // 创建协议工厂
        let factory = Arc::new(crate::protocols::ProtocolFactory::new());
        self.protocol_factory = Some(factory);

        // 这里可以添加初始化逻辑,例如注册协议处理器
        Ok(())
    }

    // 停止
    pub async fn stop(&mut self) -> Result<(), std::io::Error> {
        info!("Stopping protocol manager...");
        // 这里可以添加停止逻辑
        Ok(())
    }

    // 解析数据
    pub async fn parse_data(
        &self,
        protocol: &str,
        data: &[u8],
    ) -> Result<crate::protocols::base::ProtocolData, std::io::Error> {
        if let Some(factory) = &self.protocol_factory {
            let protocol_type = crate::protocols::ProtocolType::from(protocol);
            match factory.parse_data(&protocol_type, data) {
                Ok(protocol_data) => Ok(protocol_data),
                Err(e) => Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Protocol parse error: {}", e),
                )),
            }
        } else {
            Err(std::io::Error::other("Protocol factory not initialized"))
        }
    }

    // 生成数据
    pub async fn generate_data(
        &self,
        protocol: &str,
        data: &crate::protocols::base::ProtocolData,
    ) -> Result<Vec<u8>, std::io::Error> {
        if let Some(factory) = &self.protocol_factory {
            let protocol_type = crate::protocols::ProtocolType::from(protocol);
            match factory.generate_data(&protocol_type, data) {
                Ok(generated_data) => Ok(generated_data),
                Err(e) => Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Protocol generate error: {}", e),
                )),
            }
        } else {
            Err(std::io::Error::other("Protocol factory not initialized"))
        }
    }

    // 注册协议处理器 - 暂时注释掉,因为ProtocolHandler trait已注释
    /*
    pub async fn register_protocol(&mut self, protocol: &str, handler: Arc<dyn ProtocolHandler>) -> Result<(), std::io::Error> {
        self.protocols.insert(protocol.to_string(), handler);
        Ok(())
    }

    // 获取协议处理器 - 暂时注释掉,因为ProtocolHandler trait已注释
    pub async fn get_protocol(&self, protocol: &str) -> Option<Arc<dyn ProtocolHandler>> {
        self.protocols.get(protocol).cloned()
    }
    */
}

// 数据管理器
pub struct DataManager {
    db_pool: Option<sqlx::PgPool>,
    data_buffer: Arc<RwLock<DataBuffer>>,
    batch_size: usize,
    flush_interval: std::time::Duration,
    is_running: bool,
    total_data_stored: u64,
    total_flushes: u64,
}

// 数据缓冲区
pub struct DataBuffer {
    data: Vec<crate::protocols::base::ProtocolData>,
}

impl Default for DataManager {
    fn default() -> Self {
        Self::new()
    }
}

impl DataManager {
    pub fn new() -> Self {
        Self {
            db_pool: None,
            data_buffer: Arc::new(RwLock::new(DataBuffer { data: Vec::new() })),
            batch_size: 100,
            flush_interval: std::time::Duration::from_secs(5),
            is_running: false,
            total_data_stored: 0,
            total_flushes: 0,
        }
    }

    // 初始化
    pub async fn init(&mut self) -> Result<(), std::io::Error> {
        info!("Initializing data manager...");

        // 初始化数据库连接
        let database_url = std::env::var("DATABASE_URL")
            .map_err(|e| std::io::Error::other(format!("DATABASE_URL not set: {}", e)))?;
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(10)
            .connect(&database_url)
            .await
            .map_err(|e| std::io::Error::other(format!("Failed to initialize database: {}", e)))?;
        self.db_pool = Some(pool);
        info!("Database connection established successfully");

        self.is_running = true;

        // 启动定期刷新任务
        let data_buffer = self.data_buffer.clone();
        let _batch_size = self.batch_size;
        let flush_interval = self.flush_interval;
        let db_pool = self.db_pool.clone();

        tokio::spawn(async move {
            loop {
                tokio::time::sleep(flush_interval).await;

                let mut buffer = data_buffer.write().await;
                if !buffer.data.is_empty() {
                    let data_to_flush = buffer.data.drain(..).collect::<Vec<_>>();
                    drop(buffer);

                    if let Some(pool) = db_pool.clone() {
                        match DataManager::write_to_database(&pool, &data_to_flush).await {
                            Ok(()) => {
                                info!("Flushed {} data items to database", data_to_flush.len());
                            }
                            Err(e) => {
                                error!("Failed to write data to database: {}", e);
                            }
                        }
                    }
                }
            }
        });

        Ok(())
    }

    // 停止
    pub async fn stop(&mut self) -> Result<(), std::io::Error> {
        info!("Stopping data manager...");
        self.is_running = false;

        // 刷新剩余数据到数据库
        self.flush_buffer().await?;

        // 关闭数据库连接
        if let Some(pool) = self.db_pool.take() {
            drop(pool);
            info!("Database connection closed");
        }

        Ok(())
    }

    // 存储数据
    pub async fn store_data(
        &mut self,
        data: &crate::protocols::base::ProtocolData,
    ) -> Result<(), std::io::Error> {
        // 添加数据到缓冲区
        {
            let mut buffer = self.data_buffer.write().await;
            buffer.data.push(data.clone());
        }

        // 如果缓冲区达到阈值,刷新到数据库
        {
            let buffer = self.data_buffer.read().await;
            if buffer.data.len() >= self.batch_size {
                // 释放读锁,然后调用flush_buffer(它会获取写锁)
                drop(buffer);
                self.flush_buffer().await?;
            }
        }

        Ok(())
    }

    // 刷新缓冲区
    async fn flush_buffer(&mut self) -> Result<(), std::io::Error> {
        let mut buffer = self.data_buffer.write().await;
        if buffer.data.is_empty() {
            return Ok(());
        }

        let data_to_flush = buffer.data.drain(..).collect::<Vec<_>>();
        drop(buffer);

        let data_count = data_to_flush.len();
        info!("Flushing {} data items to database", data_count);

        // 写入数据库
        if let Some(pool) = self.db_pool.as_ref() {
            DataManager::write_to_database(pool, &data_to_flush).await?;
        }

        // 更新统计信息
        self.total_data_stored += data_count as u64;
        self.total_flushes += 1;

        Ok(())
    }

    // 将数据写入数据库
    async fn write_to_database(
        pool: &sqlx::PgPool,
        data: &[crate::protocols::base::ProtocolData],
    ) -> Result<(), std::io::Error> {
        if data.is_empty() {
            return Ok(());
        }

        // 执行批量插入
        let mut transaction = pool
            .begin()
            .await
            .map_err(|e| std::io::Error::other(format!("Failed to begin transaction: {}", e)))?;

        let result = async {
            for item in data {
                let device_id = item.device_id.clone();
                let command = item.command.clone();
                let params = serde_json::to_string(&item.params).unwrap_or_default();
                let raw_data = item.raw_data.clone();
                let timestamp = chrono::DateTime::<chrono::Utc>::from(item.timestamp);

                // 1. 写入设备数据主表
                let sql = "INSERT INTO device_data (device_id, command, params, raw_data, timestamp) VALUES ($1, $2, $3, $4, $5)";
                sqlx::query(sql)
                    .bind(&device_id)
                    .bind(&command)
                    .bind(params)
                    .bind(raw_data)
                    .bind(timestamp)
                    .execute(transaction.as_mut())
                    .await?;

                // 2. 如果是位置数据,更新车辆实时位置
                if command == "location" {
                    // 解析经纬度
                    if let (Some(longitude_str), Some(latitude_str)) = (
                        item.params.get("longitude"),
                        item.params.get("latitude")
                    ) {
                        if let (Ok(longitude), Ok(latitude)) = (
                            longitude_str.parse::<f64>(),
                            latitude_str.parse::<f64>()
                        ) {
                            if let Ok(vehicle_id) = device_id.parse::<i32>() {
                        // 更新车辆实时位置
                        let sql_update_location = r#" 
                            INSERT INTO vehicle_realtime_locations ( 
                                vehicle_id, longitude, latitude, speed, direction, altitude, accuracy, status, update_time 
                            ) VALUES ( 
                                $1, $2, $3, $4, $5, $6, $7, $8, $9 
                            ) ON CONFLICT (vehicle_id) DO UPDATE SET 
                                longitude = EXCLUDED.longitude, 
                                latitude = EXCLUDED.latitude, 
                                speed = EXCLUDED.speed, 
                                direction = EXCLUDED.direction, 
                                altitude = EXCLUDED.altitude, 
                                accuracy = EXCLUDED.accuracy, 
                                status = EXCLUDED.status, 
                                update_time = EXCLUDED.update_time 
                        "#;

                        // 解析其他可选参数
                        let speed = item.params.get("speed").and_then(|s| s.parse::<f64>().ok());
                        let direction = item.params.get("direction").and_then(|d| d.parse::<i16>().ok());
                        let altitude = item.params.get("altitude").and_then(|a| a.parse::<f64>().ok());
                        let accuracy = item.params.get("accuracy").and_then(|a| a.parse::<f64>().ok());

                        sqlx::query(sql_update_location)
                            .bind(vehicle_id)
                            .bind(longitude)
                            .bind(latitude)
                            .bind(speed)
                            .bind(direction)
                            .bind(altitude)
                            .bind(accuracy)
                            .bind(1) // 默认状态为1(在线)
                            .bind(chrono::Utc::now().naive_utc())
                            .execute(transaction.as_mut())
                            .await?;

                        // 3. 更新Redis缓存
                        let location = crate::models::VehicleRealtimeLocation {
                            id: 0,
                            vehicle_id,
                            latitude,
                            longitude,
                            speed: speed.unwrap_or(0.0),
                            direction: direction.unwrap_or(0) as i32,
                            altitude: altitude.unwrap_or(0.0),
                            accuracy: Some(accuracy.unwrap_or(0.0)),
                            status: 1,
                            timestamp: chrono::Utc::now(),
                            update_time: chrono::Utc::now(),
                            created_at: chrono::Utc::now(),
                        };

                        // 忽略Redis缓存更新失败,不影响主流程
                        let _ = crate::cache::VehicleCache::default()
                            .set_vehicle_realtime_location(vehicle_id, &location).await;
                            }
                        }
                    }
                }
            }

            transaction.commit().await
        }
        .await;

        match result {
            Ok(_) => {
                debug!("Successfully inserted {} records into database", data.len());
                Ok(())
            }
            Err(e) => {
                error!("Failed to execute batch insert: {}", e);
                Ok(())
            }
        }
    }

    // 查询数据
    pub async fn query_data(
        &self,
        query: &DataQuery,
    ) -> Result<Vec<crate::protocols::base::ProtocolData>, std::io::Error> {
        // 如果有数据库连接,从数据库查询
        if let Some(pool) = self.db_pool.as_ref() {
            match self.query_from_database(pool, query).await {
                Ok(db_result) => {
                    let mut result = db_result;
                    // 如果数据库查询结果不足,从内存缓冲区补充
                    if result.len() < query.limit {
                        let remaining = query.limit - result.len();
                        let buffer_result = self.query_from_buffer(query, remaining).await;
                        result.extend(buffer_result);
                    }
                    return Ok(result);
                }
                Err(e) => {
                    error!("Failed to query data from database: {}", e);
                    // 数据库查询失败,回退到从内存缓冲区查询
                }
            }
        }

        // 从内存缓冲区中查询(用于测试和开发或数据库查询失败时)
        let result = self.query_from_buffer(query, query.limit).await;

        Ok(result)
    }

    // 从数据库查询数据
    async fn query_from_database(
        &self,
        pool: &sqlx::PgPool,
        query: &DataQuery,
    ) -> Result<Vec<crate::protocols::base::ProtocolData>, std::io::Error> {
        let mut result = Vec::new();

        // 使用sqlx的查询宏并指定类型
        let sql = "SELECT device_id, command, params, raw_data, timestamp FROM device_data 
        WHERE (device_id = $1 OR $1 IS NULL) 
        AND (command = $2 OR $2 IS NULL) 
        AND (timestamp >= $3 OR $3 IS NULL) 
        AND (timestamp <= $4 OR $4 IS NULL) 
        ORDER BY timestamp DESC 
        LIMIT $5 OFFSET $6";

        // 转换SystemTime为chrono::DateTime
        let start_time = query.start_time.map(chrono::DateTime::<chrono::Utc>::from);
        let end_time = query.end_time.map(chrono::DateTime::<chrono::Utc>::from);

        let rows = sqlx::query_as::<
            _,
            (
                String,
                String,
                serde_json::Value,
                Option<Vec<u8>>,
                chrono::DateTime<chrono::Utc>,
            ),
        >(sql)
        .bind(query.device_id.as_deref())
        .bind(query.command.as_deref())
        .bind(start_time)
        .bind(end_time)
        .bind(query.limit as i64)
        .bind(query.offset as i64)
        .fetch_all(pool)
        .await
        .map_err(|e| std::io::Error::other(format!("Database query failed: {}", e)))?;

        // 处理查询结果
        for row in rows {
            let params: HashMap<String, String> = match row.2 {
                serde_json::Value::Object(map) => {
                    map.into_iter().map(|(k, v)| (k, v.to_string())).collect()
                }
                _ => HashMap::new(),
            };

            let protocol_data = crate::protocols::base::ProtocolData {
                device_id: row.0,
                command: row.1,
                params,
                raw_data: row.3.unwrap_or_default(),
                timestamp: row.4.into(),
            };

            result.push(protocol_data);
        }

        Ok(result)
    }

    // 从内存缓冲区查询数据
    async fn query_from_buffer(
        &self,
        query: &DataQuery,
        limit: usize,
    ) -> Vec<crate::protocols::base::ProtocolData> {
        let mut result = Vec::new();

        let buffer = self.data_buffer.read().await;
        for item in &buffer.data {
            let mut match_query = true;

            // 检查设备ID
            if let Some(device_id) = &query.device_id {
                if item.device_id != *device_id {
                    match_query = false;
                }
            }

            // 检查时间范围
            if let Some(start_time) = &query.start_time {
                if item.timestamp < *start_time {
                    match_query = false;
                }
            }
            if let Some(end_time) = &query.end_time {
                if item.timestamp > *end_time {
                    match_query = false;
                }
            }

            // 检查命令
            if let Some(command) = &query.command {
                if item.command != *command {
                    match_query = false;
                }
            }

            if match_query {
                result.push(item.clone());
                if result.len() >= limit {
                    break;
                }
            }
        }
        drop(buffer);

        result
    }
}

// 数据查询
pub struct DataQuery {
    pub device_id: Option<String>,
    pub protocol: Option<String>,
    pub start_time: Option<std::time::SystemTime>,
    pub end_time: Option<std::time::SystemTime>,
    pub command: Option<String>,
    pub limit: usize,
    pub offset: usize,
}

impl Default for DataQuery {
    fn default() -> Self {
        Self {
            device_id: None,
            protocol: None,
            start_time: None,
            end_time: None,
            command: None,
            limit: 100,
            offset: 0,
        }
    }
}
