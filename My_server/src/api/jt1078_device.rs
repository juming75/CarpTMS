//! JT/T 1078 设备注册与管理模块
//!
//! 实现JT1078流媒体平台的设备注册、管理和推流参数分配功能
//! 核心流程：
//! 1. 设备通过SIM卡号注册到平台
//! 2. 平台管理设备信息和状态
//! 3. 业务系统查询推流参数
//! 4. 终端设备按参数推流

use actix_web::{web, HttpResponse, Result};
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::{Duration, Instant};

/// 设备信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    /// 设备ID（平台生成）
    pub device_id: String,
    /// SIM卡号（设备唯一标识）
    pub sim_number: String,
    /// 设备名称
    pub device_name: String,
    /// 设备类型（JTT1078）
    pub device_type: String,
    /// 通道数量
    pub channel_count: u8,
    /// 通道列表
    pub channels: Vec<ChannelInfo>,
    /// 在线状态
    pub online: bool,
    /// 注册时间
    pub registered_at: String,
    /// 最后活跃时间
    pub last_active: String,
    /// IP地址
    pub ip_address: Option<String>,
    /// 所属组织
    pub organization: Option<String>,
}

/// 通道信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelInfo {
    /// 通道号
    pub channel_id: u8,
    /// 通道名称
    pub channel_name: String,
    /// 是否在线
    pub online: bool,
    /// 视频编码格式
    pub video_codec: String,
    /// 音频编码格式
    pub audio_codec: Option<String>,
    /// 分辨率
    pub resolution: Option<String>,
    /// 帧率
    pub framerate: Option<u8>,
    /// 码率 (kbps)
    pub bitrate: Option<u32>,
}

/// 推流参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamParams {
    /// 推流服务器IP
    pub receive_ip: String,
    /// 推流端口
    pub receive_port: u16,
    /// 推流协议（TCP/UDP）
    pub protocol: String,
    /// FLV播放地址
    pub flv_url: String,
    /// HLS播放地址
    pub hls_url: String,
    /// WebSocket播放地址
    pub ws_url: String,
    /// 推流密钥（鉴权用）
    pub stream_key: String,
    /// 过期时间（秒）
    pub expires_in: u64,
}

/// 设备注册请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceRegisterRequest {
    /// SIM卡号
    pub sim_number: String,
    /// 设备名称
    pub device_name: String,
    /// 设备类型
    pub device_type: String,
    /// 通道数量
    pub channel_count: Option<u8>,
    /// 设备IP
    pub ip_address: Option<String>,
    /// 组织标识
    pub organization: Option<String>,
}

/// 设备注册响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceRegisterResponse {
    /// 是否成功
    pub success: bool,
    /// 消息
    pub message: String,
    /// 设备ID
    pub device_id: Option<String>,
    /// 设备信息
    pub device: Option<DeviceInfo>,
}

/// 推流参数查询请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamQueryRequest {
    /// SIM卡号
    pub sim: String,
    /// 通道号（逗号分隔，如"1,2,3"）
    pub channel: String,
}

/// 推流参数查询响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamQueryResponse {
    /// 状态码
    pub code: u16,
    /// 消息
    pub message: String,
    /// 推流参数
    pub data: Option<HashMap<String, StreamParams>>,
}

/// 设备管理器配置
#[derive(Debug, Clone)]
pub struct DeviceManagerConfig {
    /// 流媒体服务器IP
    pub stream_server_ip: String,
    /// TCP推流端口
    pub tcp_stream_port: u16,
    /// UDP推流端口
    pub udp_stream_port: u16,
    /// HTTP-FLV服务端口
    pub http_flv_port: u16,
    /// HLS服务端口
    pub hls_port: u16,
    /// WebSocket服务端口
    pub ws_port: u16,
    /// 推流参数过期时间（秒）
    pub stream_key_ttl: u64,
    /// 设备心跳超时时间（秒）
    pub heartbeat_timeout: u64,
}

impl Default for DeviceManagerConfig {
    fn default() -> Self {
        Self {
            stream_server_ip: "127.0.0.1".to_string(),
            tcp_stream_port: 1078,
            udp_stream_port: 9788,
            http_flv_port: 8081,
            hls_port: 8084,
            ws_port: 8083,
            stream_key_ttl: 3600,
            heartbeat_timeout: 300,
        }
    }
}

/// 推流参数缓存（带过期时间）
struct StreamKeyCache {
    params: StreamParams,
    created_at: Instant,
}

/// 设备管理器
/// 管理设备注册、推流参数分配和设备状态监控
pub struct DeviceManager {
    /// 设备列表（SIM卡号 -> 设备信息）
    devices: Arc<RwLock<HashMap<String, DeviceInfo>>>,
    /// 推流参数缓存（SIM卡号:通道号 -> 推流参数）
    stream_keys: Arc<RwLock<HashMap<String, StreamKeyCache>>>,
    /// 配置
    config: DeviceManagerConfig,
    /// 设备计数器（用于生成设备ID）
    device_counter: Arc<RwLock<u64>>,
}

impl DeviceManager {
    /// 创建新的设备管理器
    pub fn new(config: DeviceManagerConfig) -> Self {
        Self {
            devices: Arc::new(RwLock::new(HashMap::new())),
            stream_keys: Arc::new(RwLock::new(HashMap::new())),
            config,
            device_counter: Arc::new(RwLock::new(0)),
        }
    }

    /// 生成设备ID
    async fn generate_device_id(&self) -> String {
        let mut counter = self.device_counter.write().await;
        *counter += 1;
        format!("DEV{:08X}", counter)
    }

    /// 生成推流密钥
    fn generate_stream_key(sim: &str, channel: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        format!("{}:{}:{}", sim, channel, Instant::now().elapsed().as_nanos()).hash(&mut hasher);
        format!("{:016X}", hasher.finish())
    }

    /// 注册新设备
    pub async fn register_device(&self, request: DeviceRegisterRequest) -> Result<DeviceInfo, String> {
        let mut devices = self.devices.write().await;

        // 检查设备是否已注册
        if let Some(device) = devices.get(&request.sim_number) {
            info!("Device {} already registered, updating info", request.sim_number);
            let mut updated_device = device.clone();
            updated_device.device_name = request.device_name;
            updated_device.last_active = chrono::Utc::now().to_rfc3339();
            updated_device.ip_address = request.ip_address;
            
            devices.insert(request.sim_number.clone(), updated_device.clone());
            return Ok(updated_device);
        }

        // 生成设备ID
        let device_id = self.generate_device_id().await;
        let now = chrono::Utc::now().to_rfc3339();
        let channel_count = request.channel_count.unwrap_or(1);

        // 创建通道列表
        let channels: Vec<ChannelInfo> = (1..=channel_count)
            .map(|ch| ChannelInfo {
                channel_id: ch,
                channel_name: format!("通道{}", ch),
                online: false,
                video_codec: "H264".to_string(),
                audio_codec: Some("G711A".to_string()),
                resolution: Some("1280x720".to_string()),
                framerate: Some(25),
                bitrate: Some(2048),
            })
            .collect();

        // 创建设备信息
        let device = DeviceInfo {
            device_id,
            sim_number: request.sim_number.clone(),
            device_name: request.device_name,
            device_type: request.device_type,
            channel_count,
            channels,
            online: false,
            registered_at: now.clone(),
            last_active: now,
            ip_address: request.ip_address,
            organization: request.organization,
        };

        // 保存设备
        devices.insert(request.sim_number, device.clone());

        info!("Device registered: sim={}, device_id={}", request.sim_number, device.device_id);
        Ok(device)
    }

    /// 更新设备在线状态
    pub async fn update_device_status(&self, sim_number: &str, online: bool) -> Result<(), String> {
        let mut devices = self.devices.write().await;
        if let Some(device) = devices.get_mut(sim_number) {
            device.online = online;
            device.last_active = chrono::Utc::now().to_rfc3339();
            
            // 更新所有通道状态
            for channel in &mut device.channels {
                channel.online = online;
            }

            info!("Device {} status updated: online={}", sim_number, online);
            Ok(())
        } else {
            Err(format!("Device {} not found", sim_number))
        }
    }

    /// 更新设备IP地址（设备推流时调用）
    pub async fn update_device_ip(&self, sim_number: &str, ip: &str) -> Result<(), String> {
        let mut devices = self.devices.write().await;
        if let Some(device) = devices.get_mut(sim_number) {
            device.ip_address = Some(ip.to_string());
            device.last_active = chrono::Utc::now().to_rfc3339();
            Ok(())
        } else {
            Err(format!("Device {} not found", sim_number))
        }
    }

    /// 获取设备信息
    pub async fn get_device(&self, sim_number: &str) -> Option<DeviceInfo> {
        let devices = self.devices.read().await;
        devices.get(sim_number).cloned()
    }

    /// 获取所有设备列表
    pub async fn list_devices(&self) -> Vec<DeviceInfo> {
        let devices = self.devices.read().await;
        devices.values().cloned().collect()
    }

    /// 获取在线设备列表
    pub async fn list_online_devices(&self) -> Vec<DeviceInfo> {
        let devices = self.devices.read().await;
        devices.values()
            .filter(|d| d.online)
            .cloned()
            .collect()
    }

    /// 查询推流参数
    /// 对应接口：/jtt-cam-query-by-bg.do?sim={sim}&channel={channel}
    pub async fn query_stream_params(
        &self,
        sim: &str,
        channels: &str,
    ) -> Result<HashMap<String, StreamParams>, String> {
        // 检查设备是否存在
        let devices = self.devices.read().await;
        if !devices.contains_key(sim) {
            return Err(format!("Device {} not registered", sim));
        }
        drop(devices);

        let mut result = HashMap::new();
        let channel_list: Vec<&str> = channels.split(',').collect();

        for &channel in &channel_list {
            let cache_key = format!("{}:{}", sim, channel);
            
            // 检查缓存是否有效
            {
                let stream_keys = self.stream_keys.read().await;
                if let Some(cached) = stream_keys.get(&cache_key) {
                    if cached.created_at.elapsed() < Duration::from_secs(self.config.stream_key_ttl) {
                        result.insert(channel.to_string(), cached.params.clone());
                        continue;
                    }
                }
            }

            // 生成新的推流参数
            let stream_key = Self::generate_stream_key(sim, channel);
            let flv_url = format!(
                "http://{}/live/{}_{}.flv?key={}",
                self.config.stream_server_ip,
                sim,
                channel,
                stream_key
            );
            let hls_url = format!(
                "http://{}/hls/{}_{}/index.m3u8?key={}",
                self.config.stream_server_ip,
                sim,
                channel,
                stream_key
            );
            let ws_url = format!(
                "ws://{}/ws/video/{}/{}?key={}",
                self.config.stream_server_ip,
                sim,
                channel,
                stream_key
            );

            let params = StreamParams {
                receive_ip: self.config.stream_server_ip.clone(),
                receive_port: self.config.udp_stream_port,
                protocol: "UDP".to_string(),
                flv_url,
                hls_url,
                ws_url,
                stream_key,
                expires_in: self.config.stream_key_ttl,
            };

            // 缓存推流参数
            {
                let mut stream_keys = self.stream_keys.write().await;
                stream_keys.insert(
                    cache_key.clone(),
                    StreamKeyCache {
                        params: params.clone(),
                        created_at: Instant::now(),
                    },
                );
            }

            result.insert(channel.to_string(), params);
        }

        info!(
            "Stream params queried: sim={}, channels={}",
            sim, channels
        );

        Ok(result)
    }

    /// 验证推流密钥
    pub async fn verify_stream_key(
        &self,
        sim: &str,
        channel: &str,
        key: &str,
    ) -> bool {
        let cache_key = format!("{}:{}", sim, channel);
        let stream_keys = self.stream_keys.read().await;

        if let Some(cached) = stream_keys.get(&cache_key) {
            if cached.params.stream_key == key
                && cached.created_at.elapsed() < Duration::from_secs(self.config.stream_key_ttl)
            {
                return true;
            }
        }
        false
    }

    /// 清理过期的推流参数
    pub async fn cleanup_expired_keys(&self) {
        let mut stream_keys = self.stream_keys.write().await;
        let ttl = Duration::from_secs(self.config.stream_key_ttl);
        let before_count = stream_keys.len();

        stream_keys.retain(|_, v| v.created_at.elapsed() < ttl);

        let removed = before_count - stream_keys.len();
        if removed > 0 {
            info!("Cleaned up {} expired stream keys", removed);
        }
    }

    /// 清理离线的设备
    pub async fn cleanup_offline_devices(&self) {
        let mut devices = self.devices.write().await;
        let timeout = Duration::from_secs(self.config.heartbeat_timeout);
        let mut removed_count = 0;

        devices.retain(|sim, device| {
            if let Ok(last_active) = chrono::DateTime::parse_from_rfc3339(&device.last_active) {
                let elapsed = chrono::Utc::now().signed_duration_since(last_active);
                if elapsed.num_seconds() > self.config.heartbeat_timeout as i64 {
                    info!("Removing offline device: {}", sim);
                    removed_count += 1;
                    false
                } else {
                    true
                }
            } else {
                true
            }
        });

        if removed_count > 0 {
            info!("Cleaned up {} offline devices", removed_count);
        }
    }

    /// 获取设备统计信息
    pub async fn get_statistics(&self) -> DeviceStatistics {
        let devices = self.devices.read().await;
        let online_count = devices.values().filter(|d| d.online).count();
        let stream_keys = self.stream_keys.read().await;

        DeviceStatistics {
            total_devices: devices.len(),
            online_devices: online_count,
            offline_devices: devices.len() - online_count,
            active_streams: stream_keys.len(),
        }
    }
}

/// 设备统计信息
#[derive(Debug, Clone, Serialize)]
pub struct DeviceStatistics {
    /// 设备总数
    pub total_devices: usize,
    /// 在线设备数
    pub online_devices: usize,
    /// 离线设备数
    pub offline_devices: usize,
    /// 活跃流数
    pub active_streams: usize,
}

/// 设备管理器状态（用于actix-web应用数据）
pub type DeviceManagerState = Arc<DeviceManager>;

// ============= API 路由处理函数 =============

/// 设备登录
/// POST /api/v1/jt1078/login
pub async fn device_login(
    device_manager: web::Data<DeviceManagerState>,
    form: web::Form<LoginRequest>,
) -> HttpResponse {
    info!("Device login attempt: username={}", form.username);

    // 尝试使用SIM卡号查找设备
    if let Some(device) = device_manager.get_device(&form.username).await {
        // 设备已注册，更新状态
        let _ = device_manager.update_device_status(&form.username, true).await;
        
        HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "message": "Login successful",
            "device": device
        }))
    } else {
        // 设备未注册，返回错误
        HttpResponse::NotFound().json(serde_json::json!({
            "success": false,
            "message": "Device not registered"
        }))
    }
}

/// 登录请求
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

/// 注册设备
/// POST /api/v1/jt1078/device/register
pub async fn register_device(
    device_manager: web::Data<DeviceManagerState>,
    body: web::Json<DeviceRegisterRequest>,
) -> HttpResponse {
    match device_manager.register_device(body.into_inner()).await {
        Ok(device) => HttpResponse::Ok().json(DeviceRegisterResponse {
            success: true,
            message: "Device registered successfully".to_string(),
            device_id: Some(device.device_id.clone()),
            device: Some(device),
        }),
        Err(e) => HttpResponse::InternalServerError().json(DeviceRegisterResponse {
            success: false,
            message: e,
            device_id: None,
            device: None,
        }),
    }
}

/// 查询设备列表
/// GET /api/v1/jt1078/devices
pub async fn list_devices(
    device_manager: web::Data<DeviceManagerState>,
    query: web::Query<ListDevicesQuery>,
) -> HttpResponse {
    let devices = if query.online_only {
        device_manager.list_online_devices().await
    } else {
        device_manager.list_devices().await
    };

    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "total": devices.len(),
        "devices": devices
    }))
}

/// 设备列表查询参数
#[derive(Debug, Deserialize)]
pub struct ListDevicesQuery {
    #[serde(default)]
    pub online_only: bool,
}

/// 获取设备详情
/// GET /api/v1/jt1078/device/{sim_number}
pub async fn get_device(
    device_manager: web::Data<DeviceManagerState>,
    path: web::Path<String>,
) -> HttpResponse {
    let sim_number = path.into_inner();

    match device_manager.get_device(&sim_number).await {
        Some(device) => HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "device": device
        })),
        None => HttpResponse::NotFound().json(serde_json::json!({
            "success": false,
            "message": format!("Device {} not found", sim_number)
        })),
    }
}

/// 查询推流参数
/// GET /api/v1/jt1078/stream/query?sim={sim}&channel={channel}
/// 兼容旧接口：/jtt-cam-query-by-bg.do
pub async fn query_stream_params(
    device_manager: web::Data<DeviceManagerState>,
    query: web::Query<StreamQueryRequest>,
) -> HttpResponse {
    match device_manager
        .query_stream_params(&query.sim, &query.channel)
        .await
    {
        Ok(params) => HttpResponse::Ok().json(StreamQueryResponse {
            code: 200,
            message: "success".to_string(),
            data: Some(params),
        }),
        Err(e) => HttpResponse::NotFound().json(StreamQueryResponse {
            code: 404,
            message: e,
            data: None,
        }),
    }
}

/// 兼容旧版接口
/// GET /jtt-cam-query-by-bg.do
pub async fn legacy_query_stream_params(
    device_manager: web::Data<DeviceManagerState>,
    query: web::Query<StreamQueryRequest>,
) -> HttpResponse {
    info!(
        "Legacy stream query: sim={}, channel={}",
        query.sim, query.channel
    );
    query_stream_params(device_manager, query).await
}

/// 更新设备状态（心跳）
/// POST /api/v1/jt1078/device/{sim_number}/heartbeat
pub async fn device_heartbeat(
    device_manager: web::Data<DeviceManagerState>,
    path: web::Path<String>,
) -> HttpResponse {
    let sim_number = path.into_inner();

    match device_manager.update_device_status(&sim_number, true).await {
        Ok(()) => HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "message": "Heartbeat received"
        })),
        Err(e) => HttpResponse::NotFound().json(serde_json::json!({
            "success": false,
            "message": e
        })),
    }
}

/// 获取设备统计
/// GET /api/v1/jt1078/statistics
pub async fn get_statistics(
    device_manager: web::Data<DeviceManagerState>,
) -> HttpResponse {
    let stats = device_manager.get_statistics().await;
    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "statistics": stats
    }))
}

/// 配置设备管理路由
pub fn configure_device_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1/jt1078")
            .route("/login", web::post().to(device_login))
            .route("/device/register", web::post().to(register_device))
            .route("/devices", web::get().to(list_devices))
            .route("/device/{sim_number}", web::get().to(get_device))
            .route("/device/{sim_number}/heartbeat", web::post().to(device_heartbeat))
            .route("/stream/query", web::get().to(query_stream_params))
            .route("/statistics", web::get().to(get_statistics)),
    )
    // 兼容旧版接口
    .service(
        web::resource("/jtt-cam-query-by-bg.do")
            .route(web::get().to(legacy_query_stream_params)),
    );
}
