//! 车辆用例集成测试
//!
//! 独立的集成测试，不依赖内嵌测试模块

use std::sync::Arc;
use carptms::domain::use_cases::vehicle::VehicleUseCases;
use carptms::domain::use_cases::vehicle::repository::VehicleRepository;
use carptms::domain::entities::vehicle::{Vehicle, VehicleCreate, VehicleQuery, VehicleUpdate};
use chrono::NaiveDateTime;

#[allow(dead_code)]
struct MockVehicleRepo {
    vehicles: Vec<Vehicle>,
    has_related_data: bool,
}

#[async_trait::async_trait]
impl VehicleRepository for MockVehicleRepo {
    async fn get_vehicles(
        &self,
        _query: VehicleQuery,
    ) -> Result<(Vec<Vehicle>, i64), anyhow::Error> {
        Ok((self.vehicles.clone(), self.vehicles.len() as i64))
    }

    async fn get_vehicle(&self, vehicle_id: i32) -> Result<Option<Vehicle>, anyhow::Error> {
        Ok(self
            .vehicles
            .iter()
            .find(|v| v.vehicle_id == vehicle_id)
            .cloned())
    }

    async fn get_vehicles_batch(&self, vehicle_ids: &[i32]) -> Result<Vec<Vehicle>, anyhow::Error> {
        Ok(self
            .vehicles
            .iter()
            .filter(|v| vehicle_ids.contains(&v.vehicle_id) && v.status == 1)
            .cloned()
            .collect())
    }

    async fn create_vehicle(&self, vehicle: VehicleCreate) -> Result<Vehicle, anyhow::Error> {
        let now =
            NaiveDateTime::parse_from_str("2026-01-13 10:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
        Ok(Vehicle {
            vehicle_id: self.vehicles.len() as i32 + 1,
            vehicle_name: vehicle.vehicle_name,
            license_plate: vehicle.license_plate,
            vehicle_type: vehicle.vehicle_type,
            vehicle_color: vehicle.vehicle_color,
            vehicle_brand: vehicle.vehicle_brand,
            vehicle_model: vehicle.vehicle_model,
            engine_no: vehicle.engine_no,
            frame_no: vehicle.frame_no,
            register_date: vehicle.register_date,
            inspection_date: vehicle.inspection_date,
            insurance_date: vehicle.insurance_date,
            seating_capacity: vehicle.seating_capacity,
            load_capacity: vehicle.load_capacity,
            vehicle_length: vehicle.vehicle_length,
            vehicle_width: vehicle.vehicle_width,
            vehicle_height: vehicle.vehicle_height,
            vehicle_weight: vehicle.vehicle_weight,
            max_load: vehicle.max_load,
            emergency_contact: vehicle.emergency_contact,
            emergency_phone: vehicle.emergency_phone,
            description: vehicle.description,
            vehicle_group_id: vehicle.vehicle_group_id,
            install_device_id: vehicle.install_device_id,
            status: vehicle.status,
            create_time: now,
            update_time: None,
            create_user_id: vehicle.create_user_id,
            update_user_id: None,
        })
    }

    async fn update_vehicle(
        &self,
        vehicle_id: i32,
        vehicle: VehicleUpdate,
    ) -> Result<Option<Vehicle>, anyhow::Error> {
        if let Some(mut existing) = self.get_vehicle(vehicle_id).await? {
            if let Some(name) = vehicle.vehicle_name {
                existing.vehicle_name = name;
            }
            if let Some(plate) = vehicle.license_plate {
                existing.license_plate = plate;
            }
            if let Some(status) = vehicle.status {
                existing.status = status;
            }
            Ok(Some(existing))
        } else {
            Ok(None)
        }
    }

    async fn delete_vehicle(&self, vehicle_id: i32) -> Result<bool, anyhow::Error> {
        Ok(self.vehicles.iter().any(|v| v.vehicle_id == vehicle_id))
    }

    async fn has_related_data(&self, _vehicle_id: i32) -> Result<bool, anyhow::Error> {
        Ok(self.has_related_data)
    }

    async fn exists(&self, vehicle_id: i32) -> Result<bool, anyhow::Error> {
        Ok(self.vehicles.iter().any(|v| v.vehicle_id == vehicle_id))
    }

    async fn count_by_plate(
        &self,
        plate: &str,
        exclude_id: Option<i32>,
    ) -> Result<i64, anyhow::Error> {
        Ok(self
            .vehicles
            .iter()
            .filter(|v| {
                v.license_plate == plate && exclude_id.map(|id| v.vehicle_id != id).unwrap_or(true)
            })
            .count() as i64)
    }

    async fn get_vehicle_stats(&self, _vehicle_id: i32) -> Result<Option<Vehicle>, anyhow::Error> {
        Ok(None)
    }

    async fn update_location(
        &self,
        _vehicle_id: i32,
        _lat: f64,
        _lon: f64,
        _speed: f32,
        _heading: f32,
    ) -> Result<(), anyhow::Error> {
        Ok(())
    }

    async fn update_status(&self, _vehicle_id: i32, _status: i16) -> Result<(), anyhow::Error> {
        Ok(())
    }

    async fn batch_get_vehicle_info(
        &self,
        _vehicle_ids: &[i32],
    ) -> Result<Vec<Vehicle>, anyhow::Error> {
        Ok(vec![])
    }
}

#[tokio::test]
async fn test_get_vehicles() {
    let now = NaiveDateTime::parse_from_str("2026-01-13 10:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
    let vehicles = vec![Vehicle {
        vehicle_id: 1,
        vehicle_name: "测试车辆".to_string(),
        license_plate: "京A12345".to_string(),
        vehicle_type: "货车".to_string(),
        vehicle_color: "白色".to_string(),
        vehicle_brand: "东风".to_string(),
        vehicle_model: "EQ1090".to_string(),
        engine_no: None,
        frame_no: None,
        register_date: None,
        inspection_date: None,
        insurance_date: None,
        seating_capacity: Some(3),
        load_capacity: Some(10.0),
        vehicle_length: None,
        vehicle_width: None,
        vehicle_height: None,
        vehicle_weight: None,
        max_load: None,
        emergency_contact: None,
        emergency_phone: None,
        description: None,
        vehicle_group_id: None,
        install_device_id: None,
        status: 1,
        create_time: now,
        update_time: None,
        create_user_id: 1,
        update_user_id: None,
    }];

    let mock_repo = Arc::new(MockVehicleRepo {
        vehicles: vehicles.clone(),
        has_related_data: false,
    });
    let use_cases = VehicleUseCases::new(mock_repo);

    let query = VehicleQuery::default();
    let result = use_cases.get_vehicles(query).await;

    assert!(result.is_ok());
    let (result_vehicles, total) = result.unwrap();
    assert_eq!(result_vehicles, vehicles);
    assert_eq!(total, 1);
}

#[tokio::test]
async fn test_get_vehicle_by_id() {
    let now = NaiveDateTime::parse_from_str("2026-01-13 10:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
    let vehicle = Vehicle {
        vehicle_id: 1,
        vehicle_name: "测试车辆".to_string(),
        license_plate: "京A12345".to_string(),
        vehicle_type: "货车".to_string(),
        vehicle_color: "白色".to_string(),
        vehicle_brand: "东风".to_string(),
        vehicle_model: "EQ1090".to_string(),
        engine_no: None,
        frame_no: None,
        register_date: None,
        inspection_date: None,
        insurance_date: None,
        seating_capacity: Some(3),
        load_capacity: Some(10.0),
        vehicle_length: None,
        vehicle_width: None,
        vehicle_height: None,
        vehicle_weight: None,
        max_load: None,
        emergency_contact: None,
        emergency_phone: None,
        description: None,
        vehicle_group_id: None,
        install_device_id: None,
        status: 1,
        create_time: now,
        update_time: None,
        create_user_id: 1,
        update_user_id: None,
    };

    let mock_repo = Arc::new(MockVehicleRepo {
        vehicles: vec![vehicle.clone()],
        has_related_data: false,
    });
    let use_cases = VehicleUseCases::new(mock_repo);

    let result = use_cases.get_vehicle(1).await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Some(vehicle));
}

#[tokio::test]
async fn test_create_vehicle_success() {
    let vehicle_create = VehicleCreate {
        vehicle_name: "新车辆".to_string(),
        license_plate: "京B67890".to_string(),
        vehicle_type: "货车".to_string(),
        vehicle_color: "蓝色".to_string(),
        vehicle_brand: "解放".to_string(),
        vehicle_model: "J6".to_string(),
        engine_no: None,
        frame_no: None,
        register_date: None,
        inspection_date: None,
        insurance_date: None,
        seating_capacity: Some(3),
        load_capacity: Some(15.0),
        vehicle_length: None,
        vehicle_width: None,
        vehicle_height: None,
        vehicle_weight: None,
        max_load: None,
        emergency_contact: None,
        emergency_phone: None,
        description: None,
        vehicle_group_id: None,
        install_device_id: None,
        status: 1,
        create_user_id: 1,
    };

    let mock_repo = Arc::new(MockVehicleRepo {
        vehicles: vec![],
        has_related_data: false,
    });
    let use_cases = VehicleUseCases::new(mock_repo);

    let result = use_cases.create_vehicle(vehicle_create).await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap().license_plate, "京B67890");
}

#[tokio::test]
async fn test_create_vehicle_empty_plate() {
    let vehicle_create = VehicleCreate {
        vehicle_name: "测试车辆".to_string(),
        license_plate: "".to_string(),
        vehicle_type: "货车".to_string(),
        vehicle_color: "白色".to_string(),
        vehicle_brand: "东风".to_string(),
        vehicle_model: "EQ1090".to_string(),
        engine_no: None,
        frame_no: None,
        register_date: None,
        inspection_date: None,
        insurance_date: None,
        seating_capacity: Some(3),
        load_capacity: Some(10.0),
        vehicle_length: None,
        vehicle_width: None,
        vehicle_height: None,
        vehicle_weight: None,
        max_load: None,
        emergency_contact: None,
        emergency_phone: None,
        description: None,
        vehicle_group_id: None,
        install_device_id: None,
        status: 1,
        create_user_id: 1,
    };

    let mock_repo = Arc::new(MockVehicleRepo {
        vehicles: vec![],
        has_related_data: false,
    });
    let use_cases = VehicleUseCases::new(mock_repo);

    let result: Result<Vehicle, anyhow::Error> = use_cases.create_vehicle(vehicle_create).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_delete_vehicle_with_related_data() {
    let now = NaiveDateTime::parse_from_str("2026-01-13 10:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
    let vehicle = Vehicle {
        vehicle_id: 1,
        vehicle_name: "测试车辆".to_string(),
        license_plate: "京A12345".to_string(),
        vehicle_type: "货车".to_string(),
        vehicle_color: "白色".to_string(),
        vehicle_brand: "东风".to_string(),
        vehicle_model: "EQ1090".to_string(),
        engine_no: None,
        frame_no: None,
        register_date: None,
        inspection_date: None,
        insurance_date: None,
        seating_capacity: Some(3),
        load_capacity: Some(10.0),
        vehicle_length: None,
        vehicle_width: None,
        vehicle_height: None,
        vehicle_weight: None,
        max_load: None,
        emergency_contact: None,
        emergency_phone: None,
        description: None,
        vehicle_group_id: None,
        install_device_id: None,
        status: 1,
        create_time: now,
        update_time: None,
        create_user_id: 1,
        update_user_id: None,
    };

    let mock_repo = Arc::new(MockVehicleRepo {
        vehicles: vec![vehicle],
        has_related_data: true,
    });
    let use_cases = VehicleUseCases::new(mock_repo);

    let result = use_cases.delete_vehicle(1).await;

    assert!(result.is_err());
}
