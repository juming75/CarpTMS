use chrono::NaiveDateTime;
use serde_json::Value;

use crate::errors::AppResult;
use crate::schemas::SyncStatus;

#[async_trait::async_trait]
pub trait SyncRepository: Send + Sync {
    /// 上传数据
    async fn upload_data(&self, data: &Value) -> AppResult<()>;
    
    /// 下载数据
    async fn download_data(&self, table_name: &str, last_sync_time: Option<NaiveDateTime>) -> AppResult<Vec<Value>>;
    
    /// 保存同步状态
    async fn save_sync_status(&self, status: &SyncStatus) -> AppResult<()>;
    
    /// 获取同步状态
    async fn get_sync_status(&self, sync_id: &str) -> AppResult<SyncStatus>;
    
    /// 获取同步历史
    async fn get_sync_history(&self) -> AppResult<Vec<SyncStatus>>;
    
    /// 清理同步历史
    async fn clean_sync_history(&self) -> AppResult<()>;
}