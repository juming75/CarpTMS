// WebSocket 差分压缩优化
// 对重复数据使用差分压缩，减少传输数据量

use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::infrastructure::message_router::UnifiedMessage;

/// 差分消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeltaMessage {
    /// 差分类型
    pub delta_type: DeltaType,
    /// 差分数据
    pub delta: Value,
    /// 完整消息的哈希（用于校验）
    pub hash: String,
}

/// 差分类型
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum DeltaType {
    /// 首次发送：发送完整消息
    Full,
    /// 仅位置变化
    LocationDelta,
    /// 仅传感器数据变化
    SensorDelta,
    /// 仅状态变化
    StatusDelta,
    /// 多个字段变化
    MultiFieldDelta,
}

/// 差分压缩器配置
#[derive(Debug, Clone)]
pub struct DiffCompressorConfig {
    /// 缓存大小（每个会话）
    pub cache_size: usize,
    /// 差分阈值（位置变化超过这个值才发送）
    pub location_threshold: f64, // 单位：度
    /// 启用压缩的最小字段数
    pub min_compress_fields: usize,
}

impl Default for DiffCompressorConfig {
    pub default() -> Self {
        Self {
            cache_size: 100,
            location_threshold: 0.0001, // 约11米
            min_compress_fields: 2,
        }
    }
}

/// 差分压缩器
pub struct DiffCompressor {
    config: DiffCompressorConfig,
    last_messages: Arc<RwLock<HashMap<String, CachedMessage>>>,
}

/// 缓存的消息
#[derive(Debug, Clone)]
struct CachedMessage {
    message: UnifiedMessage,
    hash: String,
    timestamp: i64,
}

impl DiffCompressor {
    /// 创建新的差分压缩器
    pub fn new(config: DiffCompressorConfig) -> Self {
        info!("Creating diff compressor: cache_size={}, threshold={}",
              config.cache_size, config.location_threshold);
        
        Self {
            config,
            last_messages: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 压缩消息
    pub async fn compress(&self, session_id: String, msg: &UnifiedMessage) -> DeltaMessage {
        let current_hash = Self::compute_hash(msg);
        
        let last = {
            let cache = self.last_messages.read().await;
            cache.get(&session_id).cloned()
        };

        match last {
            Some(cached) => {
                if cached.hash == current_hash {
                    // 消息完全相同，不需要发送
                    debug!("Message for session {} is identical to previous, skipping", session_id);
                    DeltaMessage {
                        delta_type: DeltaType::Full,
                        delta: json!(null),
                        hash: current_hash,
                    }
                } else {
                    // 计算差分
                    let delta = Self::calculate_delta(&cached.message, msg);
                    
                    // 更新缓存
                    self.update_cache(session_id, msg, current_hash).await;
                    
                    delta
                }
            }
            None => {
                // 首次消息，发送完整消息
                debug!("First message for session {}, sending full message", session_id);
                
                self.update_cache(session_id, msg, current_hash).await;
                
                DeltaMessage {
                    delta_type: DeltaType::Full,
                    delta: serde_json::to_value(msg).unwrap_or(json!(null)),
                    hash: current_hash,
                }
            }
        }
    }

    /// 计算消息哈希
    fn compute_hash(msg: &UnifiedMessage) -> String {
        // 简单实现：对关键字段进行哈希
        let key = format!("{}-{}-{}-{:.6}-{:.6}",
                         msg.device_id,
                         msg.message_type,
                         msg.timestamp,
                         msg.data.get("latitude").and_then(|v| v.as_f64()).unwrap_or(0.0),
                         msg.data.get("longitude").and_then(|v| v.as_f64()).unwrap_or(0.0));
        format!("{:x}", md5::compute(key.as_bytes()))
    }

    /// 计算差分
    fn calculate_delta(prev: &UnifiedMessage, curr: &UnifiedMessage) -> DeltaMessage {
        // 检查位置变化
        let prev_lat = prev.data.get("latitude").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let prev_lon = prev.data.get("longitude").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let curr_lat = curr.data.get("latitude").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let curr_lon = curr.data.get("longitude").and_then(|v| v.as_f64()).unwrap_or(0.0);

        let lat_diff = (curr_lat - prev_lat).abs();
        let lon_diff = (curr_lon - prev_lon).abs();

        if lat_diff < 0.0001 && lon_diff < 0.0001 {
            // 位置变化很小，检查其他字段
            let delta_fields = Self::collect_changed_fields(prev, curr);
            
            if delta_fields.is_empty() {
                // 几乎没有变化
                DeltaMessage {
                    delta_type: DeltaType::MultiFieldDelta,
                    delta: json!({}),
                    hash: Self::compute_hash(curr),
                }
            } else if delta_fields.len() == 1 {
                // 单个字段变化
                DeltaMessage {
                    delta_type: DeltaType::StatusDelta,
                    delta: json!(delta_fields),
                    hash: Self::compute_hash(curr),
                }
            } else {
                // 多个字段变化
                DeltaMessage {
                    delta_type: DeltaType::MultiFieldDelta,
                    delta: json!(delta_fields),
                    hash: Self::compute_hash(curr),
                }
            }
        } else {
            // 位置有明显变化
            DeltaMessage {
                delta_type: DeltaType::LocationDelta,
                delta: json!({
                    "latitude": curr_lat,
                    "longitude": curr_lon,
                    "altitude": curr.data.get("altitude"),
                    "speed": curr.data.get("speed"),
                    "direction": curr.data.get("direction"),
                }),
                hash: Self::compute_hash(curr),
            }
        }
    }

    /// 收集变化的字段
    fn collect_changed_fields(prev: &UnifiedMessage, curr: &UnifiedMessage) -> HashMap<String, Value> {
        let mut changed = HashMap::new();

        // 比较数据字段
        if let (Some(prev_data), Some(curr_data)) = (prev.data.as_object(), curr.data.as_object()) {
            for (key, curr_value) in curr_data {
                if let Some(prev_value) = prev_data.get(key) {
                    if prev_value != curr_value {
                        changed.insert(key.clone(), curr_value.clone());
                    }
                } else {
                    changed.insert(key.clone(), curr_value.clone());
                }
            }
        }

        changed
    }

    /// 更新缓存
    async fn update_cache(&self, session_id: String, msg: &UnifiedMessage, hash: String) {
        let mut cache = self.last_messages.write().await;

        // 检查缓存大小
        if cache.len() >= self.config.cache_size {
            // 简单的LRU策略：删除最旧的
            if let Some(oldest_key) = cache.iter()
                .min_by_key(|(_, cached)| cached.timestamp)
                .map(|(k, _)| k.clone())
            {
                cache.remove(&oldest_key);
                debug!("Removed oldest cache entry for session {}", oldest_key);
            }
        }

        let cached = CachedMessage {
            message: msg.clone(),
            hash,
            timestamp: chrono::Utc::now().timestamp(),
        };

        cache.insert(session_id, cached);
    }

    /// 清理会话缓存
    pub async fn clear_session(&self, session_id: &str) {
        let mut cache = self.last_messages.write().await;
        cache.remove(session_id);
        debug!("Cleared cache for session {}", session_id);
    }

    /// 获取统计信息
    pub async fn get_stats(&self) -> DiffCompressorStats {
        let cache = self.last_messages.read().await;
        DiffCompressorStats {
            cached_sessions: cache.len(),
            cache_size: self.config.cache_size,
            location_threshold: self.config.location_threshold,
        }
    }
}

/// 差分压缩器统计信息
#[derive(Debug, Clone, serde::Serialize)]
pub struct DiffCompressorStats {
    pub cached_sessions: usize,
    pub cache_size: usize,
    pub location_threshold: f64,
}

/// 客户端差分合并器
pub struct DeltaMerger {
    last_full_messages: HashMap<String, UnifiedMessage>,
}

impl DeltaMerger {
    pub fn new() -> Self {
        Self {
            last_full_messages: HashMap::new(),
        }
    }

    /// 合并差分消息
    pub fn merge(&mut self, session_id: String, delta: DeltaMessage) -> Option<UnifiedMessage> {
        match delta.delta_type {
            DeltaType::Full => {
                // 完整消息，直接使用
                if let Ok(msg) = serde_json::from_value(delta.delta) {
                    self.last_full_messages.insert(session_id, msg.clone());
                    Some(msg)
                } else {
                    warn!("Failed to parse full message from delta");
                    None
                }
            }
            DeltaType::LocationDelta | DeltaType::SensorDelta | 
            DeltaType::StatusDelta | DeltaType::MultiFieldDelta => {
                // 差分消息，需要与上一条消息合并
                if let Some(last_msg) = self.last_full_messages.get(&session_id) {
                    // 合并差分
                    let merged = self.merge_delta(last_msg, &delta);
                    self.last_full_messages.insert(session_id, merged.clone());
                    Some(merged)
                } else {
                    warn!("No previous message for session {}, cannot merge delta", session_id);
                    None
                }
            }
        }
    }

    /// 合并差分到完整消息
    fn merge_delta(&self, base: &UnifiedMessage, delta: &DeltaMessage) -> UnifiedMessage {
        let mut merged = base.clone();

        // 应用差分
        if let Some(delta_data) = delta.delta.as_object() {
            for (key, value) in delta_data {
                if let Some(data_map) = merged.data.as_object_mut() {
                    data_map.insert(key.clone(), value.clone());
                }
            }
        }

        merged
    }

    /// 清理会话
    pub fn clear_session(&mut self, session_id: &str) {
        self.last_full_messages.remove(session_id);
    }
}
