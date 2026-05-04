// Vehicle 聚合根
// 封装车辆业务规则，确保数据一致性

use chrono::NaiveDateTime;

use crate::errors::AppError;

/// 车辆聚合根
#[derive(Debug, Clone)]
pub struct VehicleAggregate {
    pub vehicle_id: i32,
    pub vehicle_name: String,
    pub license_plate: String,
    pub vehicle_type: String,
    pub register_date: NaiveDateTime,
    pub inspection_date: NaiveDateTime,
    pub insurance_date: NaiveDateTime,
    pub status: i32,
}

impl VehicleAggregate {
    /// 创建车辆聚合根
    #[allow(clippy::too_many_arguments)]
    pub fn create(
        vehicle_id: i32,
        vehicle_name: String,
        license_plate: String,
        vehicle_type: String,
        register_date: NaiveDateTime,
        inspection_date: NaiveDateTime,
        insurance_date: NaiveDateTime,
        status: i32,
    ) -> Result<Self, AppError> {
        // 业务规则：车辆名称不能为空
        if vehicle_name.is_empty() {
            return Err(AppError::validation("车辆名称不能为空"));
        }

        // 业务规则：车牌号不能为空
        if license_plate.is_empty() {
            return Err(AppError::validation("车牌号不能为空"));
        }

        // 业务规则：年检日期不能早于注册日期
        if inspection_date < register_date {
            return Err(AppError::validation("年检日期不能早于注册日期"));
        }

        // 业务规则：保险日期不能早于注册日期
        if insurance_date < register_date {
            return Err(AppError::validation("保险日期不能早于注册日期"));
        }

        Ok(Self {
            vehicle_id,
            vehicle_name,
            license_plate,
            vehicle_type,
            register_date,
            inspection_date,
            insurance_date,
            status,
        })
    }

    /// 检查是否需要年检
    pub fn needs_inspection(&self, current_date: NaiveDateTime) -> bool {
        self.inspection_date < current_date
    }

    /// 检查是否需要续保
    pub fn needs_insurance(&self, current_date: NaiveDateTime) -> bool {
        self.insurance_date < current_date
    }

    /// 检查是否逾期未检（超过30天）
    pub fn is_overdue_inspection(&self, current_date: NaiveDateTime) -> bool {
        let thirty_days = chrono::Duration::days(30);
        self.inspection_date + thirty_days < current_date
    }
}
