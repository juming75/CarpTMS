//! 称重微服务模块
//!
//! ## 服务拆分说明
//!
//! 5万辆车的动态称重数据处理压力大，将称重功能拆分为：
//!
//! 1. **weighing-api** - CRUD 操作服务
//! 2. **weighing-stats** - 统计查询服务  
//! 3. **weighing-batch** - 批处理服务（高频物联网数据）
//!
//! ## 当前状态
//!
//! 架构已创建，完整微服务化需要：
//! - 创建独立的数据访问层
//! - 配置服务发现
//! - 实现 gRPC 通信

pub mod gateway;

/// 微服务配置
#[derive(Debug, Clone)]
pub struct WeighingMicroserviceConfig {
    /// API 服务端口
    pub api_port: u16,
    /// 统计服务端口
    pub stats_port: u16,
    /// 批处理服务端口
    pub batch_port: u16,
    /// 最大并发连接数
    pub max_connections: usize,
    /// 缓存 TTL（秒）
    pub cache_ttl_secs: u64,
}

impl Default for WeighingMicroserviceConfig {
    fn default() -> Self {
        Self {
            api_port: 8083,
            stats_port: 8084,
            batch_port: 8085,
            max_connections: 1000,
            cache_ttl_secs: 300,
        }
    }
}
