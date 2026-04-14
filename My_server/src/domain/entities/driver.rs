//! 司机领域实体

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// 司机实体
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, FromRow)]
pub struct Driver {
    pub driver_id: i32,
    pub driver_name: String,
    pub license_number: Option<String>,
    pub phone_number: Option<String>,
    pub email: Option<String>,
    pub status: i16,
    pub create_time: DateTime<Utc>,
    pub update_time: Option<DateTime<Utc>>,
}

/// 司机创建请求
#[derive(Debug, Clone, Deserialize)]
pub struct DriverCreateRequest {
    pub driver_name: String,
    pub license_number: Option<String>,
    pub phone_number: Option<String>,
    pub email: Option<String>,
    pub status: i16,
}

/// 司机更新请求
#[derive(Debug, Clone, Deserialize)]
pub struct DriverUpdateRequest {
    pub driver_name: Option<String>,
    pub license_number: Option<String>,
    pub phone_number: Option<String>,
    pub email: Option<String>,
    pub status: Option<i16>,
}

/// 司机查询参数
#[derive(Debug, Clone, Deserialize)]
pub struct DriverQuery {
    pub page: Option<i32>,
    pub page_size: Option<i32>,
    pub driver_name: Option<String>,
    pub license_number: Option<String>,
    pub status: Option<i16>,
}

impl Driver {
    /// 创建新司机
    pub fn new(
        driver_id: i32,
        driver_name: String,
        license_number: Option<String>,
        phone_number: Option<String>,
        email: Option<String>,
        status: i16,
    ) -> Self {
        Self {
            driver_id,
            driver_name,
            license_number,
            phone_number,
            email,
            status,
            create_time: Utc::now(),
            update_time: None,
        }
    }

    /// 更新司机信息
    pub fn update(
        &mut self,
        name: Option<String>,
        license_number: Option<String>,
        phone_number: Option<String>,
        email: Option<String>,
        status: Option<i16>,
    ) {
        if let Some(name) = name {
            self.driver_name = name;
        }
        if license_number.is_some() {
            self.license_number = license_number;
        }
        if phone_number.is_some() {
            self.phone_number = phone_number;
        }
        if email.is_some() {
            self.email = email;
        }
        if let Some(status) = status {
            self.status = status;
        }
        self.update_time = Some(Utc::now());
    }

    /// 检查司机是否可以删除
    pub fn can_delete(&self) -> bool {
        // 可以根据业务规则添加更多检查
        true
    }
}
