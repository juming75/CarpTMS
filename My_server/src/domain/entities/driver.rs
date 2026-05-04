//! 司机领域实体

use chrono::NaiveDate;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, FromRow)]
pub struct Driver {
    pub driver_id: i32,
    pub driver_name: String,
    pub license_number: String,
    pub phone_number: Option<String>,
    pub email: Option<String>,
    pub status: i32,
    pub create_time: NaiveDateTime,
    pub update_time: Option<NaiveDateTime>,
    pub license_no: Option<String>,
    pub license_type: Option<String>,
    pub license_expiry: Option<NaiveDate>,
    pub id_card: Option<String>,
    pub address: Option<String>,
    pub emergency_contact: Option<String>,
    pub emergency_phone: Option<String>,
    pub hire_date: Option<NaiveDate>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DriverCreateRequest {
    pub driver_name: String,
    pub license_number: Option<String>,
    pub phone_number: Option<String>,
    pub email: Option<String>,
    pub status: Option<i32>,
    pub license_no: Option<String>,
    pub license_type: Option<String>,
    pub license_expiry: Option<NaiveDate>,
    pub id_card: Option<String>,
    pub address: Option<String>,
    pub emergency_contact: Option<String>,
    pub emergency_phone: Option<String>,
    pub hire_date: Option<NaiveDate>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DriverUpdateRequest {
    pub driver_name: Option<String>,
    pub license_number: Option<String>,
    pub phone_number: Option<String>,
    pub email: Option<String>,
    pub status: Option<i32>,
    pub license_no: Option<String>,
    pub license_type: Option<String>,
    pub license_expiry: Option<NaiveDate>,
    pub id_card: Option<String>,
    pub address: Option<String>,
    pub emergency_contact: Option<String>,
    pub emergency_phone: Option<String>,
    pub hire_date: Option<NaiveDate>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DriverQuery {
    pub page: Option<i32>,
    pub page_size: Option<i32>,
    pub driver_name: Option<String>,
    pub license_number: Option<String>,
    pub phone_number: Option<String>,
    pub status: Option<i32>,
}

impl Driver {
    pub fn new(
        driver_id: i32,
        driver_name: String,
        license_number: String,
        phone_number: Option<String>,
        email: Option<String>,
        status: i32,
    ) -> Self {
        Self {
            driver_id,
            driver_name,
            license_number,
            phone_number,
            email,
            status,
            create_time: chrono::Local::now().naive_local(),
            update_time: None,
            license_no: None,
            license_type: None,
            license_expiry: None,
            id_card: None,
            address: None,
            emergency_contact: None,
            emergency_phone: None,
            hire_date: None,
        }
    }

    pub fn update(
        &mut self,
        name: Option<String>,
        license_number: Option<String>,
        phone_number: Option<String>,
        email: Option<String>,
        status: Option<i32>,
    ) {
        if let Some(name) = name {
            self.driver_name = name;
        }
        if license_number.is_some() {
            self.license_number = license_number.unwrap_or_default();
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
        self.update_time = Some(chrono::Local::now().naive_local());
    }

    pub fn can_delete(&self) -> bool {
        true
    }
}
