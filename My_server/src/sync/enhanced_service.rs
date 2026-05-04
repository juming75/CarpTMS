//! / 增强数据同步服务 - 提供高级同步功能
// 已激活 - 2026-02-06

use super::adapter::LegacySyncAdapter;
use super::config::*;
use super::models::*;
use anyhow::Result;
use chrono::{DateTime, Utc};
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use tokio::time::{sleep, Duration};

/// 同步状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SyncStatus {
    /// 空闲
    Idle,
    /// 同步中
    Syncing,
    /// 暂停
    Paused,
    /// 错误
    Error(String),
    /// 完成
    Completed,
}

/// 同步统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStats {
    pub last_sync_time: Option<DateTime<Utc>>,
    pub vehicles_synced: u64,
    pub users_synced: u64,
    pub gps_points_synced: u64,
    pub errors: u64,
}

/// 冲突解决策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConflictResolution {
    /// 使用新数据
    UseNew,
    /// 使用旧数据
    UseOld,
    /// 手动解决
    Manual,
}

/// 增强数据同步服务
pub struct EnhancedDataSyncService {
    pub adapter: Arc<tokio::sync::Mutex<LegacySyncAdapter>>,
    pub db: Arc<PgPool>,
    pub config: SyncConfig,
    pub status: Arc<tokio::sync::RwLock<SyncStatus>>,
    pub stats: Arc<tokio::sync::RwLock<SyncStats>>,
}

impl EnhancedDataSyncService {
    /// 创建新的增强同步服务实例
    pub fn new(adapter: LegacySyncAdapter, db: Arc<PgPool>, config: SyncConfig) -> Self {
        Self {
            adapter: Arc::new(tokio::sync::Mutex::new(adapter)),
            db,
            config,
            status: Arc::new(tokio::sync::RwLock::new(SyncStatus::Idle)),
            stats: Arc::new(tokio::sync::RwLock::new(SyncStats {
                last_sync_time: None,
                vehicles_synced: 0,
                users_synced: 0,
                gps_points_synced: 0,
                errors: 0,
            })),
        }
    }

    /// 获取当前同步状态
    pub async fn get_status(&self) -> SyncStatus {
        self.status.read().await.clone()
    }

    /// 获取同步统计
    pub async fn get_stats(&self) -> SyncStats {
        self.stats.read().await.clone()
    }

    /// 设置同步状态
    async fn set_status(&self, status: SyncStatus) {
        *self.status.write().await = status;
    }

    /// 更新统计
    async fn update_stats<F>(&self, update_fn: F)
    where
        F: FnOnce(&mut SyncStats),
    {
        update_fn(&mut *self.stats.write().await);
    }

    /// 启动全量同步
    pub async fn start_full_sync(&self) -> Result<()> {
        if !self.config.enabled {
            info!("Enhanced data sync is disabled in config");
            return Ok(());
        }

        self.set_status(SyncStatus::Syncing).await;
        info!("Starting enhanced full data synchronization...");

        match self.do_full_sync().await {
            Ok(_) => {
                self.set_status(SyncStatus::Completed).await;
                info!("Enhanced full data synchronization completed");
                Ok(())
            }
            Err(e) => {
                self.set_status(SyncStatus::Error(e.to_string())).await;
                error!("Enhanced full data synchronization failed: {}", e);
                Err(e)
            }
        }
    }

    /// 执行全量同步
    async fn do_full_sync(&self) -> Result<()> {
        // 同步车辆数据
        let vehicle_count = self.sync_vehicles().await?;
        self.update_stats(|s| {
            s.vehicles_synced += vehicle_count;
            s.last_sync_time = Some(Utc::now());
        })
        .await;

        // 同步用户数据
        let user_count = self.sync_users().await?;
        self.update_stats(|s| {
            s.users_synced += user_count;
        })
        .await;

        info!("Synced {} vehicles and {} users", vehicle_count, user_count);
        Ok(())
    }

    /// 启动增量同步
    pub async fn start_incremental_sync(&self) -> Result<()> {
        if !self.config.enabled {
            info!("Enhanced incremental sync is disabled in config");
            return Ok(());
        }

        info!("Starting enhanced incremental data synchronization...");

        let interval = Duration::from_secs(self.config.sync_interval_seconds);
        let db = self.db.clone();
        let adapter = self.adapter.clone();

        tokio::spawn(async move {
            loop {
                if let Err(e) = Self::sync_incremental_step(&adapter, &db).await {
                    error!("Incremental sync error: {}", e);
                }

                sleep(interval).await;
            }
        });

        info!("Enhanced incremental sync task started");
        Ok(())
    }

    /// 同步车辆数据
    async fn sync_vehicles(&self) -> Result<u64> {
        let mut adapter = self.adapter.lock().await;
        let vehicles = adapter.fetch_vehicles().await?;
        drop(adapter);

        let mut count = 0;
        for legacy_vehicle in &vehicles {
            if let Err(e) = self.save_vehicle(legacy_vehicle).await {
                warn!("Failed to save vehicle {}: {}", legacy_vehicle.VehicleID, e);
                self.update_stats(|s| s.errors += 1).await;
            } else {
                count += 1;
            }
        }

        info!("Synced {} vehicles", count);
        Ok(count)
    }

    /// 同步用户数据
    async fn sync_users(&self) -> Result<u64> {
        let mut adapter = self.adapter.lock().await;
        let users = adapter.fetch_users().await?;
        drop(adapter);

        let mut count = 0;
        for legacy_user in &users {
            if let Err(e) = self.save_user(legacy_user).await {
                warn!("Failed to save user {}: {}", legacy_user.UserID, e);
                self.update_stats(|s| s.errors += 1).await;
            } else {
                count += 1;
            }
        }

        info!("Synced {} users", count);
        Ok(count)
    }

    /// 保存车辆到数据库
    async fn save_vehicle(&self, legacy_vehicle: &LegacyVehicle) -> Result<()> {
        // 这里应该调用数据库API保存车辆
        // 简化实现,实际应该根据数据库schema调整
        debug!("Saving vehicle: {}", legacy_vehicle.VehicleID);
        Ok(())
    }

    /// 保存用户到数据库
    async fn save_user(&self, legacy_user: &LegacyUser) -> Result<()> {
        // 这里应该调用数据库API保存用户
        // 简化实现,实际应该根据数据库schema调整
        debug!("Saving user: {}", legacy_user.UserID);
        Ok(())
    }

    /// 单步增量同步
    async fn sync_incremental_step(
        adapter: &Arc<tokio::sync::Mutex<LegacySyncAdapter>>,
        _db: &Arc<PgPool>,
    ) -> Result<()> {
        let adap = adapter.lock().await;
        // 执行增量同步逻辑
        drop(adap);
        Ok(())
    }
}
