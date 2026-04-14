use chrono::Utc;
use log::info;

use std::sync::Arc;

use crate::domain::repositories::sync::SyncRepository;
use crate::errors::AppResult;
use crate::schemas::{SyncRequest, SyncResponse, SyncStatus};

pub struct SyncUseCases {
    sync_repository: Arc<dyn SyncRepository>,
}

impl SyncUseCases {
    pub fn new(sync_repository: Arc<dyn SyncRepository>) -> Self {
        Self {
            sync_repository,
        }
    }

    /// 执行数据同步
    pub async fn execute_sync(&self, request: SyncRequest) -> AppResult<SyncResponse> {
        info!("Executing sync: {:?}", request);

        // 生成同步ID
        let sync_id = format!("sync_{}_{}", request.sync_type, Utc::now().timestamp_millis());

        // 记录同步开始 - 使用匹配数据库表结构的新字段
        let sync_status = SyncStatus {
            id: None,
            sync_type: request.sync_type.clone(),
            source_type: "legacy".to_string(),
            status: "running".to_string(),
            start_time: Utc::now(),
            end_time: None,
            total_records: 0,
            processed_records: 0,
            failed_records: 0,
            error_message: Some("Sync started".to_string()),
            last_sync_time: None,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };

        // 保存同步状态
        self.sync_repository.save_sync_status(&sync_status).await?;

        // 执行同步操作
        let (synced_count, failed_count) = match request.sync_type.as_str() {
            "upload" => self.handle_upload(&request).await?,
            "download" => self.handle_download(&request).await?,
            "full_sync" => self.handle_full_sync(&request).await?,
            _ => return Err(anyhow::anyhow!("Invalid sync type").into()),
        };

        // 更新同步状态
        let final_status = SyncStatus {
            id: None,
            sync_type: request.sync_type,
            source_type: "legacy".to_string(),
            status: if failed_count == 0 { "completed" } else { "completed_with_errors" }.to_string(),
            start_time: sync_status.start_time,
            end_time: Some(Utc::now()),
            total_records: 0,
            processed_records: synced_count as i32,
            failed_records: failed_count as i32,
            error_message: Some(format!("Sync completed. Synced: {}, Failed: {}", synced_count, failed_count)),
            last_sync_time: Some(Utc::now()),
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };

        // 保存最终状态
        self.sync_repository.save_sync_status(&final_status).await?;

        Ok(SyncResponse {
            status: final_status.status,
            synced_count: synced_count as i32,
            message: final_status.error_message.unwrap(),
            sync_id: Some(sync_id),
        })
    }

    /// 处理上传同步
    async fn handle_upload(&self, request: &SyncRequest) -> AppResult<(usize, usize)> {
        info!("Handling upload sync");
        
        let mut synced_count = 0;
        let mut failed_count = 0;

        if let Some(data) = &request.data {
            for item in data {
                match self.sync_repository.upload_data(item).await {
                    Ok(_) => synced_count += 1,
                    Err(e) => {
                        info!("Failed to upload data: {:?}", e);
                        failed_count += 1;
                    }
                }
            }
        }

        Ok((synced_count, failed_count))
    }

    /// 处理下载同步
    async fn handle_download(&self, request: &SyncRequest) -> AppResult<(usize, usize)> {
        info!("Handling download sync");
        
        let mut synced_count = 0;
        let mut failed_count = 0;

        for table in &request.tables {
            match self.sync_repository.download_data(table, request.last_sync_time).await {
                Ok(_) => synced_count += 1,
                Err(e) => {
                    info!("Failed to download data for table {}: {:?}", table, e);
                    failed_count += 1;
                }
            }
        }

        Ok((synced_count, failed_count))
    }

    /// 处理全量同步
    async fn handle_full_sync(&self, request: &SyncRequest) -> AppResult<(usize, usize)> {
        info!("Handling full sync");
        
        // 先下载数据
        let (downloaded, download_failed) = self.handle_download(request).await?;
        
        // 再上传数据
        let (uploaded, upload_failed) = self.handle_upload(request).await?;

        Ok((downloaded + uploaded, download_failed + upload_failed))
    }

    /// 获取同步状态
    pub async fn get_sync_status(&self, sync_id: &str) -> AppResult<SyncStatus> {
        info!("Getting sync status for: {}", sync_id);
        
        self.sync_repository.get_sync_status(sync_id).await
    }

    /// 获取同步历史
    pub async fn get_sync_history(&self) -> AppResult<Vec<SyncStatus>> {
        info!("Getting sync history");
        
        self.sync_repository.get_sync_history().await
    }

    /// 取消同步任务
    pub async fn cancel_sync(&self, sync_id: &str) -> AppResult<SyncStatus> {
        info!("Cancelling sync: {}", sync_id);
        
        let mut status = self.sync_repository.get_sync_status(sync_id).await?;
        
        status.status = "cancelled".to_string();
        status.end_time = Some(Utc::now());
        status.error_message = Some("Sync cancelled".to_string());
        status.updated_at = Some(Utc::now());
        
        self.sync_repository.save_sync_status(&status).await?;
        
        Ok(status)
    }

    /// 清理同步历史
    pub async fn clean_sync_history(&self) -> AppResult<()> {
        info!("Cleaning sync history");
        
        self.sync_repository.clean_sync_history().await
    }
}