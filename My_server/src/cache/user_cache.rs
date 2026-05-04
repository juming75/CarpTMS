//! / 用户缓存模块
// 提供用户相关数据的高性能缓存

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{debug, info};

use super::CacheManager;

/// 用户缓存服务
pub struct UserCache {
    cache: CacheManager,
}

impl UserCache {
    /// 创建新的用户缓存实例
    pub fn new(cache: CacheManager) -> Self {
        Self { cache }
    }

    /// 从环境变量创建 (异步)
    pub async fn from_env() -> Result<Self> {
        Ok(Self {
            cache: CacheManager::from_env().await?,
        })
    }

    /// 设置用户信息缓存 (TTL: 30分钟)
    pub async fn set_user_info<T>(&self, user_id: i32, user_info: &T) -> Result<()>
    where
        T: Serialize,
    {
        let key = format!("user:{}:info", user_id);
        let data = serde_json::to_vec(user_info)?;
        self.cache
            .set(&key, &data, Some(Duration::from_secs(1800)))
            .await?;
        debug!("Cached user info: {}", user_id);
        Ok(())
    }

    /// 获取用户信息缓存
    pub async fn get_user_info<T>(&self, user_id: i32) -> Result<Option<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        let key = format!("user:{}:info", user_id);
        match self.cache.get(&key).await? {
            Some(data) => {
                let user_info: T = serde_json::from_slice(&data)?;
                Ok(Some(user_info))
            }
            None => Ok(None),
        }
    }

    /// 设置用户权限缓存 (TTL: 10分钟)
    pub async fn set_permissions<T>(&self, user_id: i32, permissions: &T) -> Result<()>
    where
        T: Serialize,
    {
        let key = format!("user:{}:permissions", user_id);
        let data = serde_json::to_vec(permissions)?;
        self.cache
            .set(&key, &data, Some(Duration::from_secs(600)))
            .await?;
        debug!("Cached permissions for user: {}", user_id);
        Ok(())
    }

    /// 获取用户权限缓存
    pub async fn get_permissions<T>(&self, user_id: i32) -> Result<Option<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        let key = format!("user:{}:permissions", user_id);
        match self.cache.get(&key).await? {
            Some(data) => {
                let permissions: T = serde_json::from_slice(&data)?;
                Ok(Some(permissions))
            }
            None => Ok(None),
        }
    }

    /// 设置用户会话缓存 (TTL: 24小时)
    pub async fn set_session<T>(&self, session_id: &str, session_data: &T) -> Result<()>
    where
        T: Serialize,
    {
        let key = format!("session:{}", session_id);
        let data = serde_json::to_vec(session_data)?;
        self.cache
            .set(&key, &data, Some(Duration::from_secs(86400)))
            .await?;
        debug!("Cached session: {}", session_id);
        Ok(())
    }

    /// 获取用户会话缓存
    pub async fn get_session<T>(&self, session_id: &str) -> Result<Option<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        let key = format!("session:{}", session_id);
        match self.cache.get(&key).await? {
            Some(data) => {
                let session: T = serde_json::from_slice(&data)?;
                Ok(Some(session))
            }
            None => Ok(None),
        }
    }

    /// 删除用户会话缓存
    pub async fn delete_session(&self, session_id: &str) -> Result<()> {
        let key = format!("session:{}", session_id);
        self.cache.del(&key).await?;
        info!("Deleted session: {}", session_id);
        Ok(())
    }

    /// 删除用户所有缓存
    pub async fn delete_user_cache(&self, user_id: i32) -> Result<()> {
        let pattern = format!("user:{}:*", user_id);
        self.cache.del_pattern(&pattern).await?;
        info!("Deleted cache for user: {}", user_id);
        Ok(())
    }

    /// 刷新用户信息缓存TTL
    pub async fn refresh_user_info(&self, user_id: i32) -> Result<bool> {
        let key = format!("user:{}:info", user_id);
        let data = self.cache.get(&key).await?;
        if let Some(d) = data {
            self.cache
                .set(&key, &d, Some(Duration::from_secs(1800)))
                .await?;
            debug!("Refreshed user info cache: {}", user_id);
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

/// 用户缓存配置
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct UserCacheConfig {
    pub info_ttl: Duration,
    pub permissions_ttl: Duration,
    pub session_ttl: Duration,
}

impl Default for UserCacheConfig {
    fn default() -> Self {
        Self {
            info_ttl: Duration::from_secs(1800),
            permissions_ttl: Duration::from_secs(600),
            session_ttl: Duration::from_secs(86400),
        }
    }
}
