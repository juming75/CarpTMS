//! / 动态速率限制配置管理器
use super::rate_limiter::{LimitKeyType, RateLimiterConfig, StorageMode};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

// 动态配置管理器
#[derive(Clone)]
pub struct DynamicRateLimiterConfig {
    config: Arc<RwLock<RateLimiterConfig>>,
}

impl DynamicRateLimiterConfig {
    pub fn new(initial_config: RateLimiterConfig) -> Self {
        Self {
            config: Arc::new(RwLock::new(initial_config)),
        }
    }

    // 获取当前配置的克隆
    pub async fn get_config(&self) -> RateLimiterConfig {
        self.config.read().await.clone()
    }

    // 更新限流限制
    pub async fn update_limit(&self, new_limit: u32) {
        let mut config = self.config.write().await;
        let window_secs = config.window_size.as_secs_f64();
        config.limit = new_limit;
        config.burst = new_limit;
        config.refill_rate = new_limit as f64 / window_secs;
    }

    // 更新时间窗口
    pub async fn update_window(&self, new_window: Duration) {
        let mut config = self.config.write().await;
        config.window_size = new_window;
        config.refill_rate = config.limit as f64 / new_window.as_secs_f64();
    }

    // 更新令牌补充速率
    pub async fn update_refill_rate(&self, new_rate: f64) {
        let mut config = self.config.write().await;
        config.refill_rate = new_rate;
    }

    // 更新突发容量
    pub async fn update_burst(&self, new_burst: u32) {
        let mut config = self.config.write().await;
        config.burst = new_burst;
    }

    // 更新存储模式
    pub async fn update_storage_mode(&self, new_mode: StorageMode) {
        let mut config = self.config.write().await;
        config.storage_mode = new_mode;
    }

    // 更新限流键类型
    pub async fn update_key_type(&self, new_type: LimitKeyType) {
        let mut config = self.config.write().await;
        config.key_type = new_type;
    }

    // 添加豁免路径
    pub async fn add_exempt_path(&self, path: String) {
        let mut config = self.config.write().await;
        config.exempt_paths.push(path);
    }

    // 移除豁免路径
    pub async fn remove_exempt_path(&self, path: &str) {
        let mut config = self.config.write().await;
        config.exempt_paths.retain(|p| p != path);
    }

    // 批量更新配置
    pub async fn update_config(&self, updates: RateLimitUpdates) {
        let mut config = self.config.write().await;

        if let Some(limit) = updates.limit {
            config.limit = limit;
            config.burst = limit;
        }

        if let Some(window) = updates.window_size {
            config.window_size = window;
        }

        if let Some(refill_rate) = updates.refill_rate {
            config.refill_rate = refill_rate;
        }

        if let Some(burst) = updates.burst {
            config.burst = burst;
        }

        if let Some(storage_mode) = updates.storage_mode {
            config.storage_mode = storage_mode;
        }

        if let Some(key_type) = updates.key_type {
            config.key_type = key_type;
        }

        // 更新refill_rate确保一致性
        if updates.limit.is_some() || updates.window_size.is_some() {
            let window_secs = config.window_size.as_secs_f64();
            if window_secs > 0.0 {
                config.refill_rate = config.limit as f64 / window_secs;
            }
        }
    }

    // 获取当前配置摘要
    pub async fn get_summary(&self) -> ConfigSummary {
        let config = self.config.read().await;
        ConfigSummary {
            limit: config.limit,
            window_size: config.window_size,
            burst: config.burst,
            refill_rate: config.refill_rate,
            storage_mode: config.storage_mode,
            key_type: config.key_type,
            exempt_paths_count: config.exempt_paths.len(),
        }
    }
}

// 部分配置更新
#[derive(Debug, Clone, Default)]
pub struct RateLimitUpdates {
    pub limit: Option<u32>,
    pub window_size: Option<Duration>,
    pub burst: Option<u32>,
    pub refill_rate: Option<f64>,
    pub storage_mode: Option<StorageMode>,
    pub key_type: Option<LimitKeyType>,
}

// 配置摘要
#[derive(Debug, Clone)]
pub struct ConfigSummary {
    pub limit: u32,
    pub window_size: Duration,
    pub burst: u32,
    pub refill_rate: f64,
    pub storage_mode: StorageMode,
    pub key_type: LimitKeyType,
    pub exempt_paths_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[actix_rt::test]
    async fn test_dynamic_config_update() {
        let config = RateLimiterConfig::default();
        let dynamic_config = DynamicRateLimiterConfig::new(config);

        // 更新限制
        dynamic_config.update_limit(50).await;
        let summary = dynamic_config.get_summary().await;
        assert_eq!(summary.limit, 50);

        // 更新窗口
        dynamic_config.update_window(Duration::from_secs(30)).await;
        let summary = dynamic_config.get_summary().await;
        assert_eq!(summary.window_size, Duration::from_secs(30));
    }

    #[actix_rt::test]
    async fn test_exempt_paths() {
        let config = RateLimiterConfig::default();
        let dynamic_config = DynamicRateLimiterConfig::new(config);

        // 添加豁免路径
        dynamic_config
            .add_exempt_path("/api/test".to_string())
            .await;
        let summary = dynamic_config.get_summary().await;
        assert_eq!(summary.exempt_paths_count, 1);

        // 移除豁免路径
        dynamic_config.remove_exempt_path("/api/test").await;
        let summary = dynamic_config.get_summary().await;
        assert_eq!(summary.exempt_paths_count, 0);
    }
}
