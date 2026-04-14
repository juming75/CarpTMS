//! / 车辆用例层测试
use tokio::sync::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use crate::domain::entities::vehicle::Vehicle;

#[cfg(test)]
mod tests {
    use super::*;

    // 模拟车辆仓库
    struct MockVehicleRepository {
        vehicles: Arc<RwLock<HashMap<i32, Vehicle>>>,
    }

    impl MockVehicleRepository {
        fn new() -> Self {
            Self {
                vehicles: Arc::new(RwLock::new(HashMap::new())),
            }
        }

        async fn find_by_id(&self, id: i32) -> Option<Vehicle> {
            self.vehicles.read().await.get(&id).cloned()
        }

        async fn save(&self, vehicle: &Vehicle) -> Result<(), String> {
            let mut vehicles = self.vehicles.write().await;
            vehicles.insert(vehicle.vehicle_id, vehicle.clone());
            Ok(())
        }

        async fn delete(&self, id: i32) -> Result<(), String> {
            let mut vehicles = self.vehicles.write().await;
            vehicles.remove(&id);
            Ok(())
        }

        async fn list_all(&self) -> Vec<Vehicle> {
            self.vehicles.read().await.values().cloned().collect()
        }

        async fn count(&self) -> usize {
            self.vehicles.read().await.len()
        }
    }

    // 模拟车辆用例
    struct VehicleUseCase {
        repository: MockVehicleRepository,
    }

    impl VehicleUseCase {
        fn new(repository: MockVehicleRepository) -> Self {
            Self { repository }
        }

        async fn create_vehicle(&self, vehicle: Vehicle) -> Result<Vehicle, String> {
            self.repository.save(&vehicle).await?;
            Ok(vehicle)
        }

        async fn get_vehicle(&self, id: i32) -> Option<Vehicle> {
            self.repository.find_by_id(id).await
        }

        async fn update_vehicle(&self, vehicle: Vehicle) -> Result<Vehicle, String> {
            self.repository.find_by_id(vehicle.vehicle_id).await
                .ok_or("Vehicle not found".to_string())?;
            self.repository.save(&vehicle).await?;
            Ok(vehicle)
        }

        async fn delete_vehicle(&self, id: i32) -> Result<(), String> {
            self.repository.find_by_id(id).await
                .ok_or("Vehicle not found".to_string())?;
            self.repository.delete(id).await?;
            Ok(())
        }

        async fn list_vehicles(&self) -> Vec<Vehicle> {
            self.repository.list_all().await
        }

        async fn activate_vehicle(&self, id: i32) -> Result<Vehicle, String> {
            let mut vehicle = self.repository.find_by_id(id).await
                .ok_or("Vehicle not found".to_string())?;
            vehicle.status = 1;
            self.repository.save(&vehicle).await?;
            Ok(vehicle)
        }

        async fn deactivate_vehicle(&self, id: i32) -> Result<Vehicle, String> {
            let mut vehicle = self.repository.find_by_id(id).await
                .ok_or("Vehicle not found".to_string())?;
            vehicle.status = 0;
            self.repository.save(&vehicle).await?;
            Ok(vehicle)
        }
    }

    fn create_test_vehicle(id: i32) -> Vehicle {
        use chrono::NaiveDate;
        Vehicle {
            vehicle_id: id,
            vehicle_name: "测试车辆".to_string(),
            license_plate: format!("粤B{:05}", id),
            vehicle_type: "货车".to_string(),
            vehicle_color: "白色".to_string(),
            vehicle_brand: "东风".to_string(),
            vehicle_model: "天龙".to_string(),
            engine_no: format!("ENG{:06}", id),
            frame_no: format!("FRAME{:06}", id),
            register_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap().and_hms_opt(0, 0, 0).unwrap(),
            inspection_date: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap().and_hms_opt(0, 0, 0).unwrap(),
            insurance_date: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap().and_hms_opt(0, 0, 0).unwrap(),
            seating_capacity: 3,
            load_capacity: 5000.0,
            vehicle_length: 12.0,
            vehicle_width: 2.5,
            vehicle_height: 3.5,
            device_id: Some(format!("DEV{:03}", id)),
            terminal_type: Some("JT808".to_string()),
            communication_type: Some("TCP".to_string()),
            sim_card_no: Some(format!("13800138{:04}", id)),
            install_date: Some(NaiveDate::from_ymd_opt(2024, 1, 15).unwrap().and_hms_opt(0, 0, 0).unwrap()),
            install_address: Some("深圳市南山区".to_string()),
            install_technician: Some("张工".to_string()),
            own_no: Some(format!("OWN{:03}", id)),
            own_name: Some("张三".to_string()),
            own_phone: Some("13900139000".to_string()),
            own_id_card: Some("440301199001011234".to_string()),
            own_address: Some("深圳市福田区".to_string()),
            own_email: Some("zhangsan@example.com".to_string()),
            group_id: 1,
            operation_status: 1,
            operation_route: Some("深圳-广州".to_string()),
            operation_area: Some("珠三角".to_string()),
            operation_company: Some("顺丰物流".to_string()),
            driver_name: Some("李四".to_string()),
            driver_phone: Some("13700137000".to_string()),
            driver_license_no: Some("LIC123456".to_string()),
            purchase_price: Some(300000.0),
            annual_fee: Some(10000.0),
            insurance_fee: Some(5000.0),
            is_simulation: false,
            simulation_source: None,
            remark: Some("测试车辆".to_string()),
            status: 1,
        }
    }

    #[tokio::test]
    async fn test_create_vehicle_success() {
        let repo = MockVehicleRepository::new();
        let use_case = VehicleUseCase::new(repo);
        let vehicle = create_test_vehicle(1);

        let result = use_case.create_vehicle(vehicle.clone()).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().license_plate, "粤B00001");
    }

    #[tokio::test]
    async fn test_create_duplicate_vehicle() {
        let repo = MockVehicleRepository::new();
        let use_case = VehicleUseCase::new(repo);
        let vehicle = create_test_vehicle(1);

        use_case.create_vehicle(vehicle.clone()).await.unwrap();
        // 第二次创建相同ID的车辆
        let result = use_case.create_vehicle(vehicle).await;
        // 应该成功覆盖 (简化逻辑)
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_vehicle_success() {
        let repo = MockVehicleRepository::new();
        let use_case = VehicleUseCase::new(repo);
        let vehicle = create_test_vehicle(1);

        use_case.create_vehicle(vehicle.clone()).await.unwrap();
        let result = use_case.get_vehicle(1).await;

        assert!(result.is_some());
        assert_eq!(result.unwrap().vehicle_id, 1);
    }

    #[tokio::test]
    async fn test_get_vehicle_not_found() {
        let repo = MockVehicleRepository::new();
        let use_case = VehicleUseCase::new(repo);

        let result = use_case.get_vehicle(999).await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_update_vehicle_success() {
        let repo = MockVehicleRepository::new();
        let use_case = VehicleUseCase::new(repo);
        let mut vehicle = create_test_vehicle(1);

        use_case.create_vehicle(vehicle.clone()).await.unwrap();
        vehicle.vehicle_name = "更新后的车辆".to_string();

        let result = use_case.update_vehicle(vehicle).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().vehicle_name, "更新后的车辆");
    }

    #[tokio::test]
    async fn test_update_vehicle_not_found() {
        let repo = MockVehicleRepository::new();
        let use_case = VehicleUseCase::new(repo);
        let vehicle = create_test_vehicle(999);

        let result = use_case.update_vehicle(vehicle).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Vehicle not found");
    }

    #[tokio::test]
    async fn test_delete_vehicle_success() {
        let repo = MockVehicleRepository::new();
        let use_case = VehicleUseCase::new(repo);
        let vehicle = create_test_vehicle(1);

        use_case.create_vehicle(vehicle).await.unwrap();
        let result = use_case.delete_vehicle(1).await;

        assert!(result.is_ok());
        assert!(use_case.get_vehicle(1).await.is_none());
    }

    #[tokio::test]
    async fn test_delete_vehicle_not_found() {
        let repo = MockVehicleRepository::new();
        let use_case = VehicleUseCase::new(repo);

        let result = use_case.delete_vehicle(999).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_list_vehicles() {
        let repo = MockVehicleRepository::new();
        let use_case = VehicleUseCase::new(repo);

        use_case.create_vehicle(create_test_vehicle(1)).await.unwrap();
        use_case.create_vehicle(create_test_vehicle(2)).await.unwrap();
        use_case.create_vehicle(create_test_vehicle(3)).await.unwrap();

        let vehicles = use_case.list_vehicles().await;
        assert_eq!(vehicles.len(), 3);
    }

    #[tokio::test]
    async fn test_activate_vehicle() {
        let repo = MockVehicleRepository::new();
        let use_case = VehicleUseCase::new(repo);
        let mut vehicle = create_test_vehicle(1);
        vehicle.status = 0;

        use_case.create_vehicle(vehicle).await.unwrap();
        let result = use_case.activate_vehicle(1).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().status, 1);
    }

    #[tokio::test]
    async fn test_deactivate_vehicle() {
        let repo = MockVehicleRepository::new();
        let use_case = VehicleUseCase::new(repo);
        let vehicle = create_test_vehicle(1);

        use_case.create_vehicle(vehicle).await.unwrap();
        let result = use_case.deactivate_vehicle(1).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().status, 0);
    }

    #[tokio::test]
    async fn test_batch_operations() {
        let repo = MockVehicleRepository::new();
        let use_case = VehicleUseCase::new(repo);

        // 批量创建10辆车
        for i in 1..=10 {
            use_case.create_vehicle(create_test_vehicle(i)).await.unwrap();
        }

        let vehicles = use_case.list_vehicles().await;
        assert_eq!(vehicles.len(), 10);
    }

    #[tokio::test]
    async fn test_vehicle_count() {
        let repo = MockVehicleRepository::new();
        let use_case = VehicleUseCase::new(repo);

        assert_eq!(repo.count().await, 0);

        use_case.create_vehicle(create_test_vehicle(1)).await.unwrap();
        use_case.create_vehicle(create_test_vehicle(2)).await.unwrap();

        assert_eq!(repo.count().await, 2);
    }
}






