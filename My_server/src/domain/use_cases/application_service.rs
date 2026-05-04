//! 应用服务接口

use anyhow::Result;

/// 应用服务接口
#[async_trait::async_trait]
pub trait ApplicationService: Send + Sync {
    /// 获取服务名称
    fn name(&self) -> &str;

    /// 初始化服务
    fn initialize(&self) -> Result<()>;

    /// 执行服务操作
    async fn execute(&self, input: serde_json::Value) -> Result<serde_json::Value>;
}
