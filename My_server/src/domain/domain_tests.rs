//! 领域模型单元测试

#[cfg(test)]
mod tests {
    use crate::config::ArchitectureMode;
    use crate::domain::entities::vehicle::{Vehicle, VehicleCreate, VehicleQuery, VehicleUpdate};
    use chrono::NaiveDateTime;

    // ============= Vehicle Tests =============

    fn create_test_vehicle() -> Vehicle {
        Vehicle {
            vehicle_id: 1,
            vehicle_name: "测试车辆".to_string(),
            license_plate: "京A12345".to_string(),
            vehicle_type: "货车".to_string(),
            vehicle_color: "白色".to_string(),
            vehicle_brand: "东风".to_string(),
            vehicle_model: "EQ1090".to_string(),
            engine_no: "ENG123456".to_string(),
            frame_no: "FRA123456".to_string(),
            register_date: NaiveDateTime::parse_from_str(
                "2020-01-01 00:00:00",
                "%Y-%m-%d %H:%M:%S",
            )
            .unwrap(),
            inspection_date: NaiveDateTime::parse_from_str(
                "2024-01-01 00:00:00",
                "%Y-%m-%d %H:%M:%S",
            )
            .unwrap(),
            insurance_date: NaiveDateTime::parse_from_str(
                "2024-06-01 00:00:00",
                "%Y-%m-%d %H:%M:%S",
            )
            .unwrap(),
            seating_capacity: 2,
            load_capacity: 5000.0,
            vehicle_length: 6.0,
            vehicle_width: 2.0,
            vehicle_height: 2.5,
            device_id: Some("DEV001".to_string()),
            terminal_type: Some("GPS".to_string()),
            communication_type: Some("4G".to_string()),
            sim_card_no: Some("13800138000".to_string()),
            install_date: None,
            install_address: Some("北京".to_string()),
            install_technician: Some("张三".to_string()),
            own_no: Some("OWN001".to_string()),
            own_name: Some("李四".to_string()),
            own_phone: Some("13900139000".to_string()),
            own_id_card: Some("110101199001011234".to_string()),
            own_address: Some("北京市朝阳区".to_string()),
            own_email: Some("test@example.com".to_string()),
            group_id: 1,
            operation_status: 1,
            operation_route: Some("北京-上海".to_string()),
            operation_area: Some("华北".to_string()),
            operation_company: Some("测试物流公司".to_string()),
            driver_name: Some("王五".to_string()),
            driver_phone: Some("13700137000".to_string()),
            driver_license_no: Some("BJ123456".to_string()),
            purchase_price: Some(200000.0),
            annual_fee: Some(5000.0),
            insurance_fee: Some(3000.0),
            remark: Some("测试备注".to_string()),
            status: 1,
            create_time: NaiveDateTime::parse_from_str("2024-01-01 00:00:00", "%Y-%m-%d %H:%M:%S")
                .unwrap(),
            update_time: None,
            create_user_id: 1,
            update_user_id: None,
        }
    }

    #[test]
    fn test_vehicle_entity_creation() {
        let vehicle = create_test_vehicle();

        assert_eq!(vehicle.vehicle_id, 1);
        assert_eq!(vehicle.vehicle_name, "测试车辆");
        assert_eq!(vehicle.license_plate, "京A12345");
        assert_eq!(vehicle.vehicle_type, "货车");
        assert_eq!(vehicle.status, 1);
    }

    #[test]
    fn test_vehicle_create_entity() {
        let vehicle_create = VehicleCreate {
            vehicle_name: "新车辆".to_string(),
            license_plate: "京B88888".to_string(),
            vehicle_type: "货车".to_string(),
            vehicle_color: "蓝色".to_string(),
            vehicle_brand: "解放".to_string(),
            vehicle_model: "J6".to_string(),
            engine_no: "ENG999".to_string(),
            frame_no: "FRA999".to_string(),
            register_date: NaiveDateTime::parse_from_str(
                "2023-01-01 00:00:00",
                "%Y-%m-%d %H:%M:%S",
            )
            .unwrap(),
            inspection_date: NaiveDateTime::parse_from_str(
                "2024-01-01 00:00:00",
                "%Y-%m-%d %H:%M:%S",
            )
            .unwrap(),
            insurance_date: NaiveDateTime::parse_from_str(
                "2024-06-01 00:00:00",
                "%Y-%m-%d %H:%M:%S",
            )
            .unwrap(),
            seating_capacity: 3,
            load_capacity: 8000.0,
            vehicle_length: 8.0,
            vehicle_width: 2.5,
            vehicle_height: 3.0,
            device_id: None,
            terminal_type: None,
            communication_type: None,
            sim_card_no: None,
            install_date: None,
            install_address: None,
            install_technician: None,
            own_no: None,
            own_name: None,
            own_phone: None,
            own_id_card: None,
            own_address: None,
            own_email: None,
            group_id: 1,
            operation_status: 1,
            operation_route: None,
            operation_area: None,
            operation_company: None,
            driver_name: None,
            driver_phone: None,
            driver_license_no: None,
            purchase_price: None,
            annual_fee: None,
            insurance_fee: None,
            remark: None,
            status: 1,
            create_user_id: 1,
        };

        assert_eq!(vehicle_create.vehicle_name, "新车辆");
        assert_eq!(vehicle_create.license_plate, "京B88888");
        assert_eq!(vehicle_create.load_capacity, 8000.0);
    }

    #[test]
    fn test_vehicle_update_entity() {
        let mut vehicle_update = VehicleUpdate::default();
        vehicle_update.vehicle_name = Some("更新车辆名".to_string());
        vehicle_update.status = Some(2);

        assert_eq!(vehicle_update.vehicle_name, Some("更新车辆名".to_string()));
        assert_eq!(vehicle_update.status, Some(2));
        assert!(vehicle_update.license_plate.is_none());
    }

    #[test]
    fn test_vehicle_query_entity() {
        let query = VehicleQuery {
            page: Some(1),
            page_size: Some(10),
            vehicle_name: Some("测试".to_string()),
            license_plate: None,
            vehicle_type: None,
            status: Some(1),
        };

        assert_eq!(query.page, Some(1));
        assert_eq!(query.page_size, Some(10));
        assert_eq!(query.vehicle_name, Some("测试".to_string()));
    }

    // ============= Architecture Mode Tests =============

    #[test]
    fn test_architecture_mode_display() {
        let monolith = ArchitectureMode::MonolithDDD;
        assert_eq!(monolith.to_string(), "monolith_ddd");

        let micro_ddd = ArchitectureMode::MicroDDD;
        assert_eq!(micro_ddd.to_string(), "micro_ddd");
    }

    #[test]
    fn test_architecture_mode_helpers() {
        let monolith = ArchitectureMode::MonolithDDD;
        assert!(monolith.is_monolith());
        assert!(!monolith.is_microservice());
        assert!(monolith.is_ddd());

        let micro_ddd = ArchitectureMode::MicroDDD;
        assert!(!micro_ddd.is_monolith());
        assert!(micro_ddd.is_microservice());
        assert!(micro_ddd.is_ddd());
    }

    // ============= Config Tests =============

    #[test]
    fn test_architecture_config_defaults() {
        use crate::config::ArchitectureConfig;

        let config = ArchitectureConfig::default();
        assert_eq!(config.mode, ArchitectureMode::MonolithDDD);
    }

    #[test]
    fn test_architecture_config_monolith_ddd() {
        use crate::config::ArchitectureConfig;

        let config = ArchitectureConfig::monolith_ddd();
        assert_eq!(config.mode, ArchitectureMode::MonolithDDD);
        assert!(config.enable_event_driven);
        assert!(config.persist_domain_events);
        assert!(!config.bounded_contexts.is_empty());
    }

    #[test]
    fn test_architecture_config_micro_ddd() {
        use crate::config::ArchitectureConfig;

        let config = ArchitectureConfig::micro_ddd();
        assert_eq!(config.mode, ArchitectureMode::MicroDDD);
        assert!(config.enable_service_discovery);
        assert!(config.enable_event_driven);
        assert!(!config.bounded_contexts.is_empty());
        assert!(!config.microservices.is_empty());
    }

    #[test]
    fn test_architecture_config_validation() {
        use crate::config::ArchitectureConfig;

        // Monolith mode - should be valid without service_name
        let config = ArchitectureConfig::monolith_ddd();
        assert!(config.validate().is_ok());

        // Micro mode without service_name - should fail
        let mut micro_config = ArchitectureConfig::micro_ddd();
        assert!(micro_config.validate().is_err());

        // Micro mode with service_name - should be valid
        micro_config.service_name = Some("test-service".to_string());
        assert!(micro_config.validate().is_ok());
    }
}
