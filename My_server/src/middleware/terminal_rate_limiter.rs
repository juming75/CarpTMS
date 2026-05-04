//! / 终端设备限流中间件 - 防止终端设备 DDOS 攻击
//! 
//! 针对大规模车载终端（5.5万+）的专门限流，保护系统免受恶意终端或被劫持终端的攻击
//! 
//! 防护措施：
//! 1. 基于终端编号的请求频率限制
//! 2. 基于 IP 的终端访问频率限制（适应 NAT 网络场景）
//! 3. 异常行为检测（短时间内大量请求）
//! 4. 自动封禁恶意终端

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use futures::future::{ok, Future, Ready};
use log::{debug, info, warn};
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// 终端限流配置 - 针对大规模终端网络优化
#[derive(Debug, Clone)]
pub struct TerminalRateLimitConfig {
    /// 终端请求时间窗口（秒）
    pub window_seconds: u64,
    /// 每个终端最大请求数（正常流量）
    pub max_requests_per_window: u32,
    /// 每个 IP 最大终端连接数（适应 NAT/车队网络）
    pub max_terminals_per_ip: u32,
    /// 封禁持续时间（秒）
    pub ban_duration_seconds: u64,
    /// 触发封禁的异常阈值（短时间内请求数超过此值视为攻击）
    pub anomaly_threshold: u32,
    /// 异常检测时间窗口（秒）
    pub anomaly_window_seconds: u64,
    /// 单终端每秒最大请求数（瞬时限流）
    pub max_requests_per_second: u32,
    /// 豁免的终端编号前缀（如测试终端）
    pub exempt_prefixes: Vec<String>,
}

impl Default for TerminalRateLimitConfig {
    fn default() -> Self {
        Self {
            window_seconds: 60,
            max_requests_per_window: 50,         // 每终端每分钟50次请求（5.5万终端规模）
            max_terminals_per_ip: 27500,         // 每个IP最多27500个终端（适应大型NAT/车队网络）
            ban_duration_seconds: 300,           // 封禁5分钟
            anomaly_threshold: 500,               // 5分钟内超过500次请求视为异常
            anomaly_window_seconds: 300,
            max_requests_per_second: 10,         // 每秒最多10次请求
            exempt_prefixes: vec![
                "TEST_".to_string(),
                "DEV_".to_string(),
                "DEMO_".to_string(),
            ],
        }
    }
}

/// 终端请求记录
#[derive(Debug, Clone)]
struct TerminalRecord {
    /// 请求时间戳
    timestamps: Vec<Instant>,
    /// 最后封禁时间
    banned_until: Option<Instant>,
    /// 请求总数（累计）
    total_requests: u64,
    /// 最后请求时间（用于瞬时速限）
    last_request_time: Option<Instant>,
}

/// 终端限流存储
pub struct TerminalRateLimitStore {
    config: TerminalRateLimitConfig,
    /// 终端请求记录: terminal_no -> TerminalRecord
    terminal_records: Arc<RwLock<HashMap<String, TerminalRecord>>>,
    /// IP 到终端的映射: ip -> Vec<terminal_no>
    ip_terminals: Arc<RwLock<HashMap<String, Vec<String>>>>,
    /// 被封禁的 IP: ip -> banned_until
    banned_ips: Arc<RwLock<HashMap<String, Instant>>>,
}

impl TerminalRateLimitStore {
    pub fn new(config: TerminalRateLimitConfig) -> Self {
        Self {
            config,
            terminal_records: Arc::new(RwLock::new(HashMap::new())),
            ip_terminals: Arc::new(RwLock::new(HashMap::new())),
            banned_ips: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 检查终端是否被封禁
    pub async fn is_terminal_banned(&self, terminal_no: &str) -> bool {
        // 检查豁免前缀
        if self.is_exempt(terminal_no) {
            return false;
        }

        let records = self.terminal_records.read().await;
        if let Some(record) = records.get(terminal_no) {
            if let Some(banned_until) = record.banned_until {
                return Instant::now() < banned_until;
            }
        }
        false
    }

    /// 检查 IP 是否被封禁
    pub async fn is_ip_banned(&self, ip: &str) -> bool {
        let banned = self.banned_ips.read().await;
        if let Some(banned_until) = banned.get(ip) {
            return Instant::now() < *banned_until;
        }
        false
    }

    /// 检查终端是否豁免
    fn is_exempt(&self, terminal_no: &str) -> bool {
        self.config.exempt_prefixes.iter()
            .any(|prefix| terminal_no.starts_with(prefix))
    }

    /// 检查请求是否允许
    /// 返回 (是否允许, 剩余请求数, 封禁类型)
    pub async fn check_request(
        &self,
        terminal_no: &str,
        ip: &str,
    ) -> (bool, u32, Option<BanType>) {
        let now = Instant::now();
        let window = Duration::from_secs(self.config.window_seconds);
        let anomaly_window = Duration::from_secs(self.config.anomaly_window_seconds);

        // 检查豁免前缀
        let is_exempt = self.is_exempt(terminal_no);

        // ========== 1. 检查终端请求频率 ==========
        {
            let mut records = self.terminal_records.write().await;
            let record = records.entry(terminal_no.to_string())
                .or_insert_with(|| TerminalRecord {
                    timestamps: Vec::new(),
                    banned_until: None,
                    total_requests: 0,
                    last_request_time: None,
                });

            // 清理过期时间戳（异常窗口外）
            record.timestamps.retain(|&t| now.duration_since(t) < anomaly_window);

            // 检查是否已封禁
            if let Some(banned_until) = record.banned_until {
                if now < banned_until {
                    let remaining = record.timestamps.iter()
                        .filter(|&&t| now.duration_since(t) < window)
                        .count() as u32;
                    return (false, 0, Some(BanType::Terminal));
                } else {
                    // 封禁已过期，清除
                    record.banned_until = None;
                }
            }

            // ========== 1.1 瞬时速限检查（每秒请求数）==========
            if !is_exempt {
                if let Some(last_time) = record.last_request_time {
                    if now.duration_since(last_time) < Duration::from_secs(1) {
                        // 计算当前秒内的请求数
                        let recent_count = record.timestamps.iter()
                            .filter(|&&t| now.duration_since(t) < Duration::from_secs(1))
                            .count() as u32;
                        
                        if recent_count >= self.config.max_requests_per_second {
                            debug!(
                                "Terminal {} exceeded per-second limit: {}/{}",
                                terminal_no, recent_count, self.config.max_requests_per_second
                            );
                            // 不封禁，但拒绝请求
                            return (false, 0, None);
                        }
                    }
                }
            }

            // ========== 1.2 窗口限速检查 ==========
            record.timestamps.retain(|&t| now.duration_since(t) < window);
            let current_window_requests = record.timestamps.len() as u32;
            let max_requests = if is_exempt {
                self.config.max_requests_per_window * 10
            } else {
                self.config.max_requests_per_window
            };

            if current_window_requests >= max_requests {
                debug!(
                    "Terminal {} exceeded window limit: {}/{}",
                    terminal_no, current_window_requests, max_requests
                );
                // 不封禁，但拒绝请求
                return (false, 0, None);
            }

            // ========== 1.3 异常行为检测 ==========
            let total_in_window = record.timestamps.len() as u32;
            if total_in_window >= self.config.anomaly_threshold && !is_exempt {
                // 触发封禁
                let ban_until = now + Duration::from_secs(self.config.ban_duration_seconds);
                record.banned_until = Some(ban_until);
                record.timestamps.clear();

                warn!(
                    "Terminal {} banned for {}s due to anomaly: {} requests in {}s",
                    terminal_no,
                    self.config.ban_duration_seconds,
                    total_in_window,
                    self.config.anomaly_window_seconds
                );

                return (false, 0, Some(BanType::Terminal));
            }

            // 记录请求
            record.timestamps.push(now);
            record.total_requests += 1;
            record.last_request_time = Some(now);

            let remaining = max_requests - current_window_requests - 1;
            return (true, remaining, None);
        }

        // ========== 2. 检查 IP 终端数量 ==========
        {
            let mut ip_terminals = self.ip_terminals.write().await;
            let terminals = ip_terminals.entry(ip.to_string())
                .or_insert_with(Vec::new);

            let current_count = terminals.len() as u32;
            let max_terminals = if is_exempt {
                self.config.max_terminals_per_ip * 10
            } else {
                self.config.max_terminals_per_ip
            };

            // 添加当前终端到列表（如果不在列表中）
            if !terminals.contains(&terminal_no.to_string()) {
                if current_count >= max_terminals && !is_exempt {
                    warn!(
                        "IP {} exceeded terminal limit: {}/{}",
                        ip, current_count, max_terminals
                    );
                    // 清理过期终端
                    terminals.retain(|t| !self.terminal_records.read().await.contains_key(t));
                    return (false, 0, Some(BanType::Ip));
                }
                terminals.push(terminal_no.to_string());
            }
        }

        (true, self.config.max_requests_per_window, None)
    }

    /// 主动封禁终端
    pub async fn ban_terminal(&self, terminal_no: &str, duration_secs: u64) {
        let mut records = self.terminal_records.write().await;
        if let Some(record) = records.get_mut(terminal_no) {
            record.banned_until = Some(Instant::now() + Duration::from_secs(duration_secs));
            record.timestamps.clear();
            info!("Terminal {} manually banned for {}s", terminal_no, duration_secs);
        }
    }

    /// 主动封禁 IP
    pub async fn ban_ip(&self, ip: &str, duration_secs: u64) {
        let mut banned = self.banned_ips.write().await;
        banned.insert(ip.to_string(), Instant::now() + Duration::from_secs(duration_secs));
        info!("IP {} manually banned for {}s", ip, duration_secs);
    }

    /// 解除封禁
    pub async fn unban(&self, identifier: &str) -> bool {
        // 尝试作为终端编号解除
        {
            let mut records = self.terminal_records.write().await;
            if let Some(record) = records.get_mut(identifier) {
                record.banned_until = None;
                info!("Terminal {} unbanned", identifier);
                return true;
            }
        }

        // 尝试作为 IP 解除
        {
            let mut banned = self.banned_ips.write().await;
            if banned.remove(identifier).is_some() {
                info!("IP {} unbanned", identifier);
                return true;
            }
        }

        false
    }

    /// 获取统计信息
    pub async fn get_stats(&self) -> TerminalRateLimitStats {
        let records = self.terminal_records.read().await;
        let banned = self.banned_ips.read().await;
        let ip_terminals = self.ip_terminals.read().await;

        let now = Instant::now();
        let window = Duration::from_secs(self.config.window_seconds);

        let mut active_terminals = 0;
        let mut banned_terminals = 0;
        let mut active_ips = 0;
        let mut banned_ips = 0;
        let mut total_requests_window = 0u64;

        for (terminal, record) in records.iter() {
            if let Some(banned_until) = record.banned_until {
                if now < banned_until {
                    banned_terminals += 1;
                }
            }
            let window_requests = record.timestamps.iter()
                .filter(|&&t| now.duration_since(t) < window)
                .count() as u64;
            if window_requests > 0 {
                active_terminals += 1;
                total_requests_window += window_requests;
            }
        }

        for (_, banned_until) in banned.iter() {
            if now < *banned_until {
                banned_ips += 1;
            }
        }

        for (_, terminals) in ip_terminals.iter() {
            if !terminals.is_empty() {
                active_ips += 1;
            }
        }

        TerminalRateLimitStats {
            active_terminals,
            banned_terminals,
            total_terminals: records.len(),
            active_ips,
            banned_ips,
            total_ips: banned.len(),
            requests_in_window: total_requests_window,
        }
    }
}

/// 封禁类型
#[derive(Debug, Clone, Copy)]
pub enum BanType {
    Terminal,
    Ip,
}

/// 统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalRateLimitStats {
    pub active_terminals: usize,
    pub banned_terminals: usize,
    pub total_terminals: usize,
    pub active_ips: usize,
    pub banned_ips: usize,
    pub total_ips: usize,
    pub requests_in_window: u64,
}

/// 限流中间件
pub struct TerminalRateLimitMiddleware {
    store: Arc<TerminalRateLimitStore>,
}

impl TerminalRateLimitMiddleware {
    pub fn new(config: TerminalRateLimitConfig) -> Self {
        Self {
            store: Arc::new(TerminalRateLimitStore::new(config)),
        }
    }

    pub fn store(&self) -> Arc<TerminalRateLimitStore> {
        self.store.clone()
    }

    /// 从请求中提取终端编号
    fn extract_terminal_no(req: &ServiceRequest) -> Option<String> {
        // 尝试从路径参数获取
        if let Some(terminal_no) = req.match_info().get("terminal_no") {
            return Some(terminal_no.to_string());
        }

        // 尝试从查询参数获取
        if let Some(terminal_no) = req.query_string().split('&')
            .find(|s| s.starts_with("terminal_no="))
            .and_then(|s| s.strip_prefix("terminal_no="))
        {
            return Some(terminal_no.to_string());
        }

        // 尝试从请求体获取（需要解析 JSON）
        // 这里简化处理，实际可能需要异步解析

        None
    }

    /// 从请求中获取客户端 IP
    fn get_client_ip(req: &ServiceRequest) -> String {
        req.connection_info()
            .realip_remote_addr()
            .unwrap_or("unknown")
            .to_string()
    }
}

impl<S, B> Transform<S, ServiceRequest> for TerminalRateLimitMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = TerminalRateLimitMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(TerminalRateLimitMiddlewareService {
            service,
            store: self.store.clone(),
        })
    }
}

pub struct TerminalRateLimitMiddlewareService<S> {
    service: S,
    store: Arc<TerminalRateLimitStore>,
}

impl<S, B> Service<ServiceRequest> for TerminalRateLimitMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let store = self.store.clone();

        Box::pin(async move {
            // 提取终端编号
            let terminal_no = match TerminalRateLimitMiddleware::extract_terminal_no(&req) {
                Some(t) => t,
                None => {
                    // 没有终端编号，跳过限流检查
                    return self.service.call(req).await;
                }
            };

            let client_ip = TerminalRateLimitMiddleware::get_client_ip(&req);

            // 检查 IP 是否封禁
            if store.is_ip_banned(&client_ip).await {
                warn!("Banned IP {} attempted connection", client_ip);
                return Err(actix_web::error::ErrorTooManyRequests(
                    "IP temporarily banned due to suspicious activity"
                ));
            }

            // 检查请求
            let (allowed, remaining, ban_type) = store.check_request(&terminal_no, &client_ip).await;

            if !allowed {
                match ban_type {
                    Some(BanType::Terminal) => {
                        warn!(
                            "Terminal {} rate limited. IP: {}, Remaining: {}",
                            terminal_no, client_ip, remaining
                        );
                    }
                    Some(BanType::Ip) => {
                        warn!(
                            "IP {} rate limited. Terminal: {}, Reason: too many terminals",
                            client_ip, terminal_no
                        );
                    }
                    None => {
                        debug!(
                            "Terminal {} request rejected (rate limit). IP: {}, Remaining: {}",
                            terminal_no, client_ip, remaining
                        );
                    }
                }

                return Err(actix_web::error::ErrorTooManyRequests(
                    "Too many requests, please slow down"
                ));
            }

            // 允许请求，继续处理
            self.service.call(req).await
        })
    }
}
