//! 司机用例集成测试
//!
//! 独立的集成测试，不依赖内嵌测试模块

use chrono::Local;
use std::sync::Arc;
use carptms::domain::use_cases::driver::DriverUseCases;
use carptms::domain::entities::driver::{
    Driver, DriverCreateRequest, DriverQuery, DriverUpdateRequest,
};
use carptms::domain::use_cases::driver::repository::DriverRepository;

#[allow(dead_code)]
struct MockDriverRepo {
    drivers: Vec<Driver>,
    has_related_data: bool,
}

#[async_trait::async_trait]
impl DriverRepository for MockDriverRepo {
    async fn find_all(
        &self,
        _page: i32,
        _page_size: i32,
        _query: DriverQuery,
    ) -> Result<(Vec<Driver>, i64), anyhow::Error> {
        Ok((self.drivers.clone(), self.drivers.len() as i64))
    }

    async fn find_by_id(&self, driver_id: i32) -> Result<Option<Driver>, anyhow::Error> {
        Ok(self
            .drivers
            .iter()
            .find(|d| d.driver_id == driver_id)
            .cloned())
    }

    async fn create(&self, driver: DriverCreateRequest) -> Result<Driver, anyhow::Error> {
        let now = Local::now().naive_local();
        let new_driver = Driver {
            driver_id: self.drivers.len() as i32 + 1,
            driver_name: driver.driver_name,
            license_number: driver.license_number.unwrap_or_default(),
            phone_number: driver.phone_number,
            email: driver.email,
            status: driver.status.unwrap_or(0),
            create_time: now,
            update_time: None,
            license_no: driver.license_no,
            license_type: driver.license_type,
            license_expiry: driver.license_expiry,
            id_card: driver.id_card,
            address: driver.address,
            emergency_contact: driver.emergency_contact,
            emergency_phone: driver.emergency_phone,
            hire_date: driver.hire_date,
        };
        Ok(new_driver)
    }

    async fn update(
        &self,
        driver_id: i32,
        driver: DriverUpdateRequest,
    ) -> Result<Driver, anyhow::Error> {
        if let Some(mut existing_driver) = self.find_by_id(driver_id).await? {
            if let Some(driver_name) = driver.driver_name {
                existing_driver.driver_name = driver_name;
            }
            if driver.license_number.is_some() {
                existing_driver.license_number = driver.license_number.unwrap_or_default();
            }
            if driver.phone_number.is_some() {
                existing_driver.phone_number = driver.phone_number;
            }
            if driver.email.is_some() {
                existing_driver.email = driver.email;
            }
            if let Some(status) = driver.status {
                existing_driver.status = status;
            }
            existing_driver.update_time = Some(Local::now().naive_local());

            Ok(existing_driver)
        } else {
            Err(anyhow::anyhow!("Driver not found"))
        }
    }

    async fn delete(&self, driver_id: i32) -> Result<(), anyhow::Error> {
        if self.drivers.iter().any(|d| d.driver_id == driver_id) {
            Ok(())
        } else {
            Err(anyhow::anyhow!("Driver not found"))
        }
    }

    async fn has_related_data(&self, _driver_id: i32) -> Result<bool, anyhow::Error> {
        Ok(self.has_related_data)
    }

    async fn exists(&self, driver_id: i32) -> Result<bool, anyhow::Error> {
        Ok(self.drivers.iter().any(|d| d.driver_id == driver_id))
    }

    async fn count_by_name(
        &self,
        name: &str,
        exclude_id: Option<i32>,
    ) -> Result<i64, anyhow::Error> {
        Ok(self
            .drivers
            .iter()
            .filter(|d| {
                d.driver_name == name && exclude_id.map(|id| d.driver_id != id).unwrap_or(true)
            })
            .count() as i64)
    }

    async fn count_vehicles(&self, _driver_id: i32) -> Result<i64, anyhow::Error> {
        Ok(0)
    }

    async fn count_orders(&self, _driver_id: i32) -> Result<i64, anyhow::Error> {
        Ok(0)
    }
}

#[tokio::test]
async fn test_get_drivers() {
    let now = Local::now().naive_local();
    let drivers = vec![
        Driver {
            driver_id: 1,
            driver_name: "Test Driver 1".to_string(),
            license_number: "123456".to_string(),
            phone_number: Some("13800138000".to_string()),
            email: Some("driver1@example.com".to_string()),
            status: 1,
            create_time: now,
            update_time: None,
            license_no: None,
            license_type: None,
            license_expiry: None,
            id_card: None,
            address: None,
            emergency_contact: None,
            emergency_phone: None,
            hire_date: None,
        },
        Driver {
            driver_id: 2,
            driver_name: "Test Driver 2".to_string(),
            license_number: "654321".to_string(),
            phone_number: Some("13900139000".to_string()),
            email: Some("driver2@example.com".to_string()),
            status: 1,
            create_time: now,
            update_time: None,
            license_no: None,
            license_type: None,
            license_expiry: None,
            id_card: None,
            address: None,
            emergency_contact: None,
            emergency_phone: None,
            hire_date: None,
        },
    ];

    let mock_repo = Arc::new(MockDriverRepo {
        drivers: drivers.clone(),
        has_related_data: false,
    });

    let use_cases = DriverUseCases::new(mock_repo);

    let query = DriverQuery {
        page: None,
        page_size: None,
        driver_name: None,
        license_number: None,
        phone_number: None,
        status: None,
    };
    let result: Result<(Vec<Driver>, i64), anyhow::Error> = use_cases.get_drivers(query).await;

    assert!(result.is_ok());
    let (result_drivers, result_total) = result.unwrap();
    assert_eq!(result_drivers, drivers);
    assert_eq!(result_total, 2);
}

#[tokio::test]
async fn test_get_driver() {
    let now = Local::now().naive_local();
    let driver = Driver {
        driver_id: 1,
        driver_name: "Test Driver".to_string(),
        license_number: "123456".to_string(),
        phone_number: Some("13800138000".to_string()),
        email: Some("driver@example.com".to_string()),
        status: 1,
        create_time: now,
        update_time: None,
        license_no: None,
        license_type: None,
        license_expiry: None,
        id_card: None,
        address: None,
        emergency_contact: None,
        emergency_phone: None,
        hire_date: None,
    };

    let mock_repo = Arc::new(MockDriverRepo {
        drivers: vec![driver.clone()],
        has_related_data: false,
    });

    let use_cases = DriverUseCases::new(mock_repo);
    let result: Result<Option<Driver>, anyhow::Error> = use_cases.get_driver(1).await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Some(driver));
}

#[tokio::test]
async fn test_create_driver_success() {
    let mock_repo = Arc::new(MockDriverRepo {
        drivers: vec![],
        has_related_data: false,
    });

    let use_cases = DriverUseCases::new(mock_repo);

    let request = DriverCreateRequest {
        driver_name: "New Driver".to_string(),
        license_number: Some("123456".to_string()),
        phone_number: Some("13800138000".to_string()),
        email: Some("newdriver@example.com".to_string()),
        status: Some(1),
        license_no: None,
        license_type: None,
        license_expiry: None,
        id_card: None,
        address: None,
        emergency_contact: None,
        emergency_phone: None,
        hire_date: None,
    };

    let result: Result<Driver, anyhow::Error> = use_cases.create_driver(request).await;

    assert!(result.is_ok());
    let created_driver = result.unwrap();
    assert_eq!(created_driver.driver_name, "New Driver");
}

#[tokio::test]
async fn test_update_driver() {
    let now = Local::now().naive_local();
    let existing_driver = Driver {
        driver_id: 1,
        driver_name: "Old Name".to_string(),
        license_number: "123456".to_string(),
        phone_number: Some("13800138000".to_string()),
        email: Some("old@example.com".to_string()),
        status: 1,
        create_time: now,
        update_time: None,
        license_no: None,
        license_type: None,
        license_expiry: None,
        id_card: None,
        address: None,
        emergency_contact: None,
        emergency_phone: None,
        hire_date: None,
    };

    let mock_repo = Arc::new(MockDriverRepo {
        drivers: vec![existing_driver],
        has_related_data: false,
    });

    let use_cases = DriverUseCases::new(mock_repo);

    let request = DriverUpdateRequest {
        driver_name: Some("New Name".to_string()),
        license_number: Some("654321".to_string()),
        phone_number: Some("13900139000".to_string()),
        email: Some("new@example.com".to_string()),
        status: Some(0),
        license_no: None,
        license_type: None,
        license_expiry: None,
        id_card: None,
        address: None,
        emergency_contact: None,
        emergency_phone: None,
        hire_date: None,
    };

    let result: Result<Option<Driver>, anyhow::Error> = use_cases.update_driver(1, request).await;

    assert!(result.is_ok());
    let updated_driver = result.unwrap().unwrap();
    assert_eq!(updated_driver.driver_name, "New Name");
    assert_eq!(updated_driver.status, 0);
}

#[tokio::test]
async fn test_delete_driver_success() {
    let now = Local::now().naive_local();
    let driver = Driver {
        driver_id: 1,
        driver_name: "Test Driver".to_string(),
        license_number: "123456".to_string(),
        phone_number: Some("13800138000".to_string()),
        email: Some("driver@example.com".to_string()),
        status: 1,
        create_time: now,
        update_time: None,
        license_no: None,
        license_type: None,
        license_expiry: None,
        id_card: None,
        address: None,
        emergency_contact: None,
        emergency_phone: None,
        hire_date: None,
    };

    let mock_repo = Arc::new(MockDriverRepo {
        drivers: vec![driver],
        has_related_data: false,
    });

    let use_cases = DriverUseCases::new(mock_repo);
    let result: Result<bool, anyhow::Error> = use_cases.delete_driver(1).await;

    assert!(result.is_ok());
    assert!(result.unwrap());
}

#[tokio::test]
async fn test_delete_driver_with_related_data() {
    let now = Local::now().naive_local();
    let driver = Driver {
        driver_id: 1,
        driver_name: "Test Driver".to_string(),
        license_number: "123456".to_string(),
        phone_number: Some("13800138000".to_string()),
        email: Some("driver@example.com".to_string()),
        status: 1,
        create_time: now,
        update_time: None,
        license_no: None,
        license_type: None,
        license_expiry: None,
        id_card: None,
        address: None,
        emergency_contact: None,
        emergency_phone: None,
        hire_date: None,
    };

    let mock_repo = Arc::new(MockDriverRepo {
        drivers: vec![driver],
        has_related_data: true,
    });

    let use_cases = DriverUseCases::new(mock_repo);
    let result: Result<bool, anyhow::Error> = use_cases.delete_driver(1).await;

    assert!(result.is_err());
}
