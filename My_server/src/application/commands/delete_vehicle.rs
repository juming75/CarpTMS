//! 删除车辆命令

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::{Command, CommandHandler, CommandResponse};
use crate::domain::repositories::{SqlxVehicleRepository, VehicleRepository};
use crate::errors::{AppError, AppResult};

/// 删除车辆命令
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteVehicleCommand {
    /// 车辆ID
    pub vehicle_id: i32,
}

impl Command for DeleteVehicleCommand {
    fn command_type() -> &'static str {
        "delete_vehicle"
    }
}

/// 删除车辆命令处理器
pub struct DeleteVehicleCommandHandler {
    pool: sqlx::PgPool,
    repository: SqlxVehicleRepository,
}

impl DeleteVehicleCommandHandler {
    /// 创建处理器实例
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self {
            pool,
            repository: SqlxVehicleRepository::new(),
        }
    }
}

#[async_trait]
impl CommandHandler<DeleteVehicleCommand> for DeleteVehicleCommandHandler {
    async fn handle(&self, command: DeleteVehicleCommand) -> AppResult<CommandResponse> {
        // 验证命令
        if command.vehicle_id <= 0 {
            return Err(AppError::validation_error("车辆ID无效", None));
        }

        // 检查车辆是否存在
        let existing = self
            .repository
            .find_by_id(&self.pool, command.vehicle_id)
            .await?;

        if existing.is_none() {
            return Err(AppError::not_found_error("车辆不存在".to_string()));
        }

        // 调用仓储删除车辆
        let deleted = self.repository.delete(&self.pool, command.vehicle_id).await?;

        if deleted {
            Ok(CommandResponse::success_with_message(
                command.vehicle_id,
                "车辆删除成功",
            ))
        } else {
            Err(AppError::db_error("车辆删除失败", None))
        }
    }
}
