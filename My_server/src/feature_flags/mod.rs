//! Feature Flags 模块
//! 实现 A/B 测试框架，支持功能灰度验证

use log::info;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 特性标志状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FeatureStatus {
    /// 完全禁用
    Disabled,
    /// 完全启用
    Enabled,
    /// 部分启用（基于用户比例）
    Partial(f64), // 0.0 到 1.0 之间的比例
}

/// 特性标志配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureFlag {
    /// 特性名称
    pub name: String,
    /// 特性描述
    pub description: String,
    /// 特性状态
    pub status: FeatureStatus,
    /// 适用的用户组（可选）
    pub user_groups: Option<Vec<String>>,
    /// 适用的环境（可选）
    pub environments: Option<Vec<String>>,
    /// 开始时间（可选）
    pub start_time: Option<String>,
    /// 结束时间（可选）
    pub end_time: Option<String>,
}

/// Feature Flags 管理器
pub struct FeatureFlagManager {
    flags: Arc<RwLock<HashMap<String, FeatureFlag>>>,
}

impl Default for FeatureFlagManager {
    fn default() -> Self {
        Self::new()
    }
}

impl FeatureFlagManager {
    /// 创建新的 Feature Flags 管理器
    pub fn new() -> Self {
        let mut flags = HashMap::new();

        // 添加默认特性标志
        flags.insert(
            "new_ui".to_string(),
            FeatureFlag {
                name: "new_ui".to_string(),
                description: "新用户界面".to_string(),
                status: FeatureStatus::Disabled,
                user_groups: None,
                environments: None,
                start_time: None,
                end_time: None,
            },
        );

        flags.insert(
            "advanced_reporting".to_string(),
            FeatureFlag {
                name: "advanced_reporting".to_string(),
                description: "高级报表功能".to_string(),
                status: FeatureStatus::Disabled,
                user_groups: None,
                environments: None,
                start_time: None,
                end_time: None,
            },
        );

        flags.insert(
            "real_time_tracking".to_string(),
            FeatureFlag {
                name: "real_time_tracking".to_string(),
                description: "实时跟踪功能".to_string(),
                status: FeatureStatus::Enabled,
                user_groups: None,
                environments: None,
                start_time: None,
                end_time: None,
            },
        );

        Self {
            flags: Arc::new(RwLock::new(flags)),
        }
    }

    /// 获取特性标志状态
    pub async fn get_flag(&self, name: &str) -> Option<FeatureFlag> {
        let flags = self.flags.read().await;
        flags.get(name).cloned()
    }

    /// 设置特性标志状态
    pub async fn set_flag(&self, flag: FeatureFlag) -> Result<(), String> {
        let flag_name = flag.name.clone();
        let mut flags = self.flags.write().await;
        flags.insert(flag_name.clone(), flag);
        info!("特性标志更新: {:?}", flags.get(&flag_name));
        Ok(())
    }

    /// 检查用户是否启用了某个特性
    pub async fn is_enabled(&self, feature_name: &str, user_id: Option<u64>) -> bool {
        let flags = self.flags.read().await;

        match flags.get(feature_name) {
            Some(flag) => match flag.status {
                FeatureStatus::Enabled => true,
                FeatureStatus::Disabled => false,
                FeatureStatus::Partial(ratio) => {
                    if let Some(user_id) = user_id {
                        let hash = user_id as f64 / u64::MAX as f64;
                        hash < ratio
                    } else {
                        rand::random::<f64>() < ratio
                    }
                }
            },
            None => false,
        }
    }

    /// 获取所有特性标志
    pub async fn get_all_flags(&self) -> HashMap<String, FeatureFlag> {
        let flags = self.flags.read().await;
        flags.clone()
    }

    /// 批量更新特性标志
    pub async fn update_flags(&self, new_flags: Vec<FeatureFlag>) -> Result<(), String> {
        let count = new_flags.len();
        let mut flags = self.flags.write().await;

        for flag in new_flags {
            flags.insert(flag.name.clone(), flag);
        }

        info!("批量更新特性标志，共更新 {} 个", count);
        Ok(())
    }

    /// 加载特性标志配置
    pub async fn load_config(&self, config: HashMap<String, FeatureFlag>) {
        let mut flags = self.flags.write().await;
        *flags = config;
        info!("从配置加载特性标志，共 {} 个", flags.len());
    }
}
