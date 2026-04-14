//! / DDD设计模式测试

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vehicle_entity_creation() {
        let vehicle = VehicleEntity::new(
            "V123".to_string(),
            "车牌号123".to_string(),
            "truck".to_string(),
        );
        assert_eq!(vehicle.id(), "V123");
        assert_eq!(vehicle.plate_number, "车牌号123");
    }

    #[test]
    fn test_aggregate_root_creation() {
        let vehicle = VehicleEntity::new(
            "V123".to_string(),
            "车牌号123".to_string(),
            "truck".to_string(),
        );
        let aggregate = VehicleAggregate::new(vehicle);
        assert_eq!(aggregate.vehicle().id(), "V123");
    }

    #[test]
    fn test_aggregate_add_event() {
        let vehicle = VehicleEntity::new(
            "V123".to_string(),
            "车牌号123".to_string(),
            "truck".to_string(),
        );
        let mut aggregate = VehicleAggregate::new(vehicle);
        
        aggregate.add_event(DomainEvent::VehicleOnline {
            vehicle_id: "V123".to_string(),
            timestamp: chrono::Utc::now(),
        });
        
        assert_eq!(aggregate.events().len(), 1);
    }

    #[test]
    fn test_aggregate_clear_events() {
        let vehicle = VehicleEntity::new(
            "V123".to_string(),
            "车牌号123".to_string(),
            "truck".to_string(),
        );
        let mut aggregate = VehicleAggregate::new(vehicle);
        
        aggregate.add_event(DomainEvent::VehicleOnline {
            vehicle_id: "V123".to_string(),
            timestamp: chrono::Utc::now(),
        });
        
        let events = aggregate.take_events();
        assert_eq!(events.len(), 1);
        assert_eq!(aggregate.events().len(), 0);
    }

    #[test]
    fn test_repository_trait() {
        // 测试Repository trait定义
        let _repo: Box<dyn Repository<VehicleEntity>> = Box::new(MockVehicleRepository::new());
    }

    #[test]
    fn test_domain_service() {
        let service = VehicleDomainService::new();
        let result = service.validate_vehicle_id("V123");
        assert!(result.is_ok());
        
        let result = service.validate_vehicle_id("");
        assert!(result.is_err());
    }

    // Mock实现用于测试
    struct MockVehicleRepository;

    impl MockVehicleRepository {
        fn new() -> Self {
            Self
        }
    }

    impl Repository<VehicleEntity> for MockVehicleRepository {
        async fn find_by_id(&self, _id: &str) -> Result<Option<VehicleEntity>, AppError> {
            Ok(None)
        }

        async fn save(&self, _entity: &VehicleEntity) -> Result<(), AppError> {
            Ok(())
        }

        async fn delete(&self, _id: &str) -> Result<(), AppError> {
            Ok(())
        }
    }
}






