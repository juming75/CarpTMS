//! 司机应用服务

use std::sync::Arc;

use crate::domain::entities::driver::{Driver, DriverCreateRequest, DriverUpdateRequest};
use crate::domain::use_cases::driver::DriverRepository;
use crate::errors::AppError;

/// 司机应用服务
pub struct DriverService {
    repository: Arc<dyn DriverRepository>,
}

impl DriverService {
    /// 创建新司机服务
    pub fn new(repository: Arc<dyn DriverRepository>) -> Self {
        Self { repository }
    }

    /// 获取司机列表
    pub async fn get_drivers(
        &self,
        page: i32,
        page_size: i32,
        driver_name: Option<String>,
        license_number: Option<String>,
        status: Option<i16>,
    ) -> Result<(Vec<Driver>, i64), AppError> {
        let query = crate::domain::entities::driver::DriverQuery {
            page: Some(page),
            page_size: Some(page_size),
            driver_name,
            license_number,
            status,
        };
        self.repository.find_all(page, page_size, query).await.map_err(|e| AppError::internal_error(e.to_string().as_str(), None))
    }

    /// 获取司机详情
    pub async fn get_driver(&self, driver_id: i32) -> Result<Option<Driver>, AppError> {
        self.repository.find_by_id(driver_id).await.map_err(|e| AppError::internal_error(e.to_string().as_str(), None))
    }

    /// 创建司机
    pub async fn create_driver(&self, request: DriverCreateRequest) -> Result<Driver, AppError> {
        // 检查司机名称是否已存在
        let existing_count = self.repository.count_by_name(&request.driver_name, None).await.map_err(|e| AppError::internal_error(e.to_string().as_str(), None))?;
        if existing_count > 0 {
            return Err(AppError::business_error("Driver name already exists", None));
        }

        self.repository.create(request).await.map_err(|e| AppError::internal_error(e.to_string().as_str(), None))
    }

    /// 更新司机
    pub async fn update_driver(
        &self,
        driver_id: i32,
        request: DriverUpdateRequest,
    ) -> Result<Driver, AppError> {
        // 检查司机是否存在
        if !self.repository.exists(driver_id).await.map_err(|e| AppError::internal_error(e.to_string().as_str(), None))? {
            return Err(AppError::not_found_error("Driver not found".to_string()));
        }

        // 检查司机名称是否已被其他司机使用
        if let Some(ref driver_name) = request.driver_name {
            if !driver_name.is_empty() {
                let duplicate_count = self.repository.count_by_name(driver_name, Some(driver_id)).await.map_err(|e| AppError::internal_error(e.to_string().as_str(), None))?;
                if duplicate_count > 0 {
                    return Err(AppError::business_error("Driver name already exists", None));
                }
            }
        }

        self.repository.update(driver_id, request).await.map_err(|e| AppError::internal_error(e.to_string().as_str(), None))
    }

    /// 删除司机
    pub async fn delete_driver(&self, driver_id: i32) -> Result<(), AppError> {
        // 检查司机是否存在
        if !self.repository.exists(driver_id).await.map_err(|e| AppError::internal_error(e.to_string().as_str(), None))? {
            return Err(AppError::not_found_error("Driver not found".to_string()));
        }

        // 检查司机是否有关联的车辆
        let vehicle_count = self.repository.count_vehicles(driver_id).await.map_err(|e| AppError::internal_error(e.to_string().as_str(), None))?;
        if vehicle_count > 0 {
            return Err(AppError::business_error(
                "Cannot delete driver with associated vehicles",
                None,
            ));
        }

        // 检查司机是否有关联的订单
        let order_count = self.repository.count_orders(driver_id).await.map_err(|e| AppError::internal_error(e.to_string().as_str(), None))?;
        if order_count > 0 {
            return Err(AppError::business_error(
                "Cannot delete driver with associated orders",
                None,
            ));
        }

        self.repository.delete(driver_id).await.map_err(|e| AppError::internal_error(e.to_string().as_str(), None))
    }
}
