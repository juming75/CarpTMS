// VehicleGroup 聚合根
// 封装车组业务规则

use crate::errors::AppError;

/// 车组聚合根
#[derive(Debug, Clone)]
pub struct VehicleGroupAggregate {
    pub group_id: i32,
    pub group_name: String,
    pub parent_id: Option<i32>,
    pub description: Option<String>,
    pub vehicle_count: i64,
}

impl VehicleGroupAggregate {
    /// 创建车组聚合根
    pub fn create(
        group_id: i32,
        group_name: String,
        parent_id: Option<i32>,
        description: Option<String>,
    ) -> Result<Self, AppError> {
        // 业务规则：车组名称不能为空
        if group_name.is_empty() {
            return Err(AppError::validation("车组名称不能为空"));
        }

        // 业务规则：车组名称长度限制
        if group_name.len() > 100 {
            return Err(AppError::validation("车组名称不能超过100个字符"));
        }

        Ok(Self {
            group_id,
            group_name,
            parent_id,
            description,
            vehicle_count: 0,
        })
    }

    /// 更新车组
    pub fn update(
        &mut self,
        group_name: Option<String>,
        description: Option<String>,
    ) -> Result<(), AppError> {
        if let Some(name) = group_name {
            if name.is_empty() {
                return Err(AppError::validation("车组名称不能为空"));
            }
            if name.len() > 100 {
                return Err(AppError::validation("车组名称不能超过100个字符"));
            }
            self.group_name = name;
        }

        if let Some(desc) = description {
            self.description = Some(desc);
        }

        Ok(())
    }

    /// 增加车辆数量
    pub fn add_vehicle(&mut self) {
        self.vehicle_count += 1;
    }

    /// 减少车辆数量
    pub fn remove_vehicle(&mut self) {
        if self.vehicle_count > 0 {
            self.vehicle_count -= 1;
        }
    }
}
