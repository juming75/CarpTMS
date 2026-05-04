//! 高可用架构模块
//!
//! 提供高可用架构设计支持：
//! - 多机房部署
//! - 负载均衡
//! - 集群化设计
//! - 故障自动转移

use chrono::Utc;
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tokio::time::timeout;

/// 节点状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeStatus {
    /// 健康
    Healthy,
    /// 降级
    Degraded,
    /// 不健康
    Unhealthy,
    /// 维护中
    Maintenance,
}

/// 节点信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterNode {
    /// 节点ID
    pub node_id: String,
    /// 节点地址
    pub address: String,
    /// 节点角色
    pub role: NodeRole,
    /// 节点状态
    pub status: NodeStatus,
    /// 最后心跳时间
    pub last_heartbeat: String,
    /// 负载（0-100）
    pub load_percent: u8,
    /// 机房标识
    pub datacenter: String,
}

/// 节点角色
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeRole {
    /// 网关节点（无状态）
    Gateway,
    /// 业务处理节点
    Business,
    /// 数据存储节点
    Storage,
    /// 负载均衡器
    LoadBalancer,
}

/// 集群配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterConfig {
    /// 心跳间隔（秒）
    pub heartbeat_interval: u64,
    /// 健康检查超时（秒）
    pub health_check_timeout: u64,
    /// 故障转移阈值
    pub failure_threshold: u32,
    /// 负载均衡策略
    pub load_balance_strategy: LoadBalanceStrategy,
    /// 是否启用多机房
    pub multi_datacenter_enabled: bool,
}

impl Default for ClusterConfig {
    fn default() -> Self {
        Self {
            heartbeat_interval: 5,
            health_check_timeout: 10,
            failure_threshold: 3,
            load_balance_strategy: LoadBalanceStrategy::RoundRobin,
            multi_datacenter_enabled: true,
        }
    }
}

/// 负载均衡策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoadBalanceStrategy {
    /// 轮询
    RoundRobin,
    /// 加权轮询
    WeightedRoundRobin,
    /// 最少连接
    LeastConnections,
    /// 基于延迟
    LatencyBased,
}

/// 集群管理器
pub struct ClusterManager {
    /// 集群节点
    nodes: Arc<RwLock<HashMap<String, ClusterNode>>>,
    /// 集群配置
    config: Arc<RwLock<ClusterConfig>>,
    /// 故障计数
    failure_counts: Arc<RwLock<HashMap<String, u32>>>,
    /// 当前轮询索引
    round_robin_index: Arc<RwLock<usize>>,
    /// 统计信息
    stats: Arc<RwLock<ClusterStats>>,
}

/// 集群统计信息
#[derive(Debug, Clone, Default)]
pub struct ClusterStats {
    /// 总请求数
    pub total_requests: u64,
    /// 故障转移次数
    pub failover_count: u64,
    /// 当前健康节点数
    pub healthy_nodes: u64,
    /// 当前不健康节点数
    pub unhealthy_nodes: u64,
}

impl ClusterManager {
    /// 创建新的集群管理器
    pub fn new() -> Self {
        Self {
            nodes: Arc::new(RwLock::new(HashMap::new())),
            config: Arc::new(RwLock::new(ClusterConfig::default())),
            failure_counts: Arc::new(RwLock::new(HashMap::new())),
            round_robin_index: Arc::new(RwLock::new(0)),
            stats: Arc::new(RwLock::new(ClusterStats::default())),
        }
    }

    /// 使用自定义配置创建集群管理器
    pub fn with_config(config: ClusterConfig) -> Self {
        Self {
            nodes: Arc::new(RwLock::new(HashMap::new())),
            config: Arc::new(RwLock::new(config)),
            failure_counts: Arc::new(RwLock::new(HashMap::new())),
            round_robin_index: Arc::new(RwLock::new(0)),
            stats: Arc::new(RwLock::new(ClusterStats::default())),
        }
    }

    /// 注册节点
    pub async fn register_node(&self, node: ClusterNode) {
        let mut nodes = self.nodes.write().await;
        nodes.insert(node.node_id.clone(), node);
        
        let mut failure_counts = self.failure_counts.write().await;
        failure_counts.insert(node.node_id.clone(), 0);
        
        info!("Cluster node registered: {}", node.node_id);
    }

    /// 注销节点
    pub async fn unregister_node(&self, node_id: &str) {
        let mut nodes = self.nodes.write().await;
        nodes.remove(node_id);
        
        let mut failure_counts = self.failure_counts.write().await;
        failure_counts.remove(node_id);
        
        info!("Cluster node unregistered: {}", node_id);
    }

    /// 更新节点心跳
    pub async fn update_heartbeat(&self, node_id: &str) {
        let mut nodes = self.nodes.write().await;
        if let Some(node) = nodes.get_mut(node_id) {
            node.last_heartbeat = Utc::now().to_rfc3339();
            node.status = NodeStatus::Healthy;
            
            // 重置故障计数
            let mut failure_counts = self.failure_counts.write().await;
            if let Some(count) = failure_counts.get_mut(node_id) {
                *count = 0;
            }
        }
    }

    /// 检查节点健康状态
    pub async fn check_node_health(&self, node_id: &str) -> bool {
        debug!("Checking health for node {}", node_id);
        
        // 尝试连接节点
        let config = self.config.read().await;
        let nodes = self.nodes.read().await;
        
        if let Some(node) = nodes.get(node_id) {
            let result = timeout(
                Duration::from_secs(config.health_check_timeout),
                tokio::net::TcpStream::connect(&node.address),
            ).await;
            
            match result {
                Ok(Ok(_)) => true,
                _ => false,
            }
        } else {
            false
        }
    }

    /// 执行故障转移
    pub async fn perform_failover(&self, failed_node_id: &str) -> Option<String> {
        warn!("Performing failover for node {}", failed_node_id);
        
        let nodes = self.nodes.read().await;
        let failed_node = nodes.get(failed_node_id)?;
        
        // 查找同机房的健康节点
        let backup_node = nodes.values()
            .filter(|n| {
                n.datacenter == failed_node.datacenter
                    && n.role == failed_node.role
                    && matches!(n.status, NodeStatus::Healthy)
                    && n.node_id != failed_node_id
            })
            .min_by_key(|n| n.load_percent);
        
        if let Some(backup) = backup_node {
            info!(
                "Failover from {} to {} in datacenter {}",
                failed_node_id, backup.node_id, failed_node.datacenter
            );
            
            let mut stats = self.stats.write().await;
            stats.failover_count += 1;
            
            Some(backup.node_id.clone())
        } else {
            // 如果没有同机房的节点，选择任意健康节点
            let any_backup = nodes.values()
                .filter(|n| {
                    matches!(n.status, NodeStatus::Healthy)
                        && n.role == failed_node.role
                        && n.node_id != failed_node_id
                })
                .min_by_key(|n| n.load_percent);
            
            if let Some(backup) = any_backup {
                info!(
                    "Cross-datacenter failover from {} to {}",
                    failed_node_id, backup.node_id
                );
                
                let mut stats = self.stats.write().await;
                stats.failover_count += 1;
                
                Some(backup.node_id.clone())
            } else {
                warn!("No backup node available for failover");
                None
            }
        }
    }

    /// 获取下一个节点（负载均衡）
    pub async fn get_next_node(&self) -> Option<String> {
        let config = self.config.read().await;
        let nodes = self.nodes.read().await;
        
        let healthy_nodes: Vec<&ClusterNode> = nodes.values()
            .filter(|n| matches!(n.status, NodeStatus::Healthy))
            .collect();
        
        if healthy_nodes.is_empty() {
            return None;
        }
        
        match config.load_balance_strategy {
            LoadBalanceStrategy::RoundRobin => {
                let mut index = self.round_robin_index.write().await;
                let node = healthy_nodes.get(*index % healthy_nodes.len())?;
                *index = (*index + 1) % healthy_nodes.len();
                Some(node.node_id.clone())
            }
            LoadBalanceStrategy::LeastConnections => {
                healthy_nodes.iter()
                    .min_by_key(|n| n.load_percent)
                    .map(|n| n.node_id.clone())
            }
            _ => healthy_nodes.first().map(|n| n.node_id.clone()),
        }
    }

    /// 更新节点负载
    pub async fn update_node_load(&self, node_id: &str, load_percent: u8) {
        let mut nodes = self.nodes.write().await;
        if let Some(node) = nodes.get_mut(node_id) {
            node.load_percent = load_percent;
            
            // 根据负载更新状态
            node.status = if load_percent < 70 {
                NodeStatus::Healthy
            } else if load_percent < 90 {
                NodeStatus::Degraded
            } else {
                NodeStatus::Unhealthy
            };
        }
    }

    /// 获取集群统计信息
    pub async fn get_stats(&self) -> ClusterStats {
        let nodes = self.nodes.read().await;
        let mut stats = self.stats.write().await;
        
        stats.healthy_nodes = nodes.values()
            .filter(|n| matches!(n.status, NodeStatus::Healthy))
            .count() as u64;
        stats.unhealthy_nodes = nodes.values()
            .filter(|n| matches!(n.status, NodeStatus::Unhealthy))
            .count() as u64;
        
        stats.clone()
    }

    /// 获取所有节点信息
    pub async fn get_all_nodes(&self) -> Vec<ClusterNode> {
        let nodes = self.nodes.read().await;
        nodes.values().cloned().collect()
    }
}

/// 创建集群管理器实例
pub fn create_cluster_manager() -> ClusterManager {
    ClusterManager::new()
}

/// 创建带自定义配置的集群管理器
pub fn create_cluster_manager_with_config(config: ClusterConfig) -> ClusterManager {
    ClusterManager::with_config(config)
}
