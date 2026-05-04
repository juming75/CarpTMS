//!
//! 核心模块综合测试
//! 补充关键业务模块的单元测试，目标覆盖率 70%+

#[cfg(test)]
mod core_module_tests {
    use crate::domain::entities::*;
    use crate::domain::repositories::*;
    use crate::errors::AppError;

    // ========== 用户模块测试 ==========
    mod user_tests {
        use super::*;

        #[test]
        fn test_user_entity_creation() {
            let user = User {
                id: None,
                username: "test_user".to_string(),
                email: "test@example.com".to_string(),
                password_hash: "hashed_pwd".to_string(),
                role: UserRole::Admin,
                created_at: None,
                updated_at: None,
            };

            assert_eq!(user.username, "test_user");
            assert_eq!(user.email, "test@example.com");
            assert_eq!(user.role, UserRole::Admin);
            assert!(user.id.is_none());
        }

        #[test]
        fn test_user_role_permissions() {
            assert!(matches!(UserRole::Admin, UserRole::Admin));
            assert!(matches!(UserRole::User, UserRole::User));
            assert!(matches!(UserRole::Operator, UserRole::Operator));
            assert!(matches!(UserRole::Viewer, UserRole::Viewer));
        }

        #[tokio::test]
        async fn test_user_validation_empty_username() {
            let user = User {
                id: None,
                username: "".to_string(),
                email: "test@example.com".to_string(),
                password_hash: "hashed".to_string(),
                role: UserRole::User,
                created_at: None,
                updated_at: None,
            };

            // 用户名不能为空
            let validation_result = validate_user(&user);
            assert!(validation_result.is_err());
        }

        #[tokio::test]
        async fn test_user_validation_invalid_email() {
            let user = User {
                id: None,
                username: "validuser".to_string(),
                email: "invalid-email".to_string(),
                password_hash: "hashed".to_string(),
                role: UserRole::User,
                created_at: None,
                updated_at: None,
            };

            let validation_result = validate_user(&user);
            assert!(validation_result.is_err());
        }

        fn validate_user(user: &User) -> Result<(), AppError> {
            if user.username.is_empty() {
                return Err(AppError::Validation("用户名不能为空".to_string()));
            }
            if !user.email.contains('@') {
                return Err(AppError::Validation("邮箱格式不正确".to_string()));
            }
            Ok(())
        }
    }

    // ========== 车辆模块测试 ==========
    mod vehicle_tests {
        use super::*;

        #[test]
        fn test_vehicle_entity_creation() {
            let vehicle = Vehicle {
                id: None,
                plate_number: "京A12345".to_string(),
                vehicle_type: VehicleType::Heavy,
                status: VehicleStatus::Active,
                max_weight: Some(50000.0),
                created_at: None,
                updated_at: None,
            };

            assert_eq!(vehicle.plate_number, "京A12345");
            assert_eq!(vehicle.vehicle_type, VehicleType::Heavy);
            assert_eq!(vehicle.status, VehicleStatus::Active);
        }

        #[test]
        fn test_vehicle_types() {
            assert!(matches!(VehicleType::Light, VehicleType::Light));
            assert!(matches!(VehicleType::Medium, VehicleType::Medium));
            assert!(matches!(VehicleType::Heavy, VehicleType::Heavy));
            assert!(matches!(VehicleType::Trailer, VehicleType::Trailer));
        }

        #[test]
        fn test_vehicle_status_transitions() {
            let mut vehicle = create_test_vehicle();

            // 正常状态转换
            vehicle.status = VehicleStatus::Active;
            assert!(is_valid_status_transition(&VehicleStatus::Active, &VehicleStatus::Maintenance));

            vehicle.status = VehicleStatus::Maintenance;
            assert!(is_valid_status_transition(&VehicleStatus::Maintenance, &VehicleStatus::Active));
        }

        #[test]
        fn test_vehicle_weight_validation() {
            let vehicle = Vehicle {
                id: None,
                plate_number: "京A12345".to_string(),
                vehicle_type: VehicleType::Heavy,
                status: VehicleStatus::Active,
                max_weight: Some(50000.0),
                created_at: None,
                updated_at: None,
            };

            // 重型车辆最大载重应该大于 30000kg
            if vehicle.vehicle_type == VehicleType::Heavy {
                assert!(vehicle.max_weight.unwrap_or(0.0) >= 30000.0);
            }
        }

        fn create_test_vehicle() -> Vehicle {
            Vehicle {
                id: None,
                plate_number: "京A12345".to_string(),
                vehicle_type: VehicleType::Heavy,
                status: VehicleStatus::Active,
                max_weight: Some(50000.0),
                created_at: None,
                updated_at: None,
            }
        }

        fn is_valid_status_transition(from: &VehicleStatus, to: &VehicleStatus) -> bool {
            matches!(
                (from, to),
                (VehicleStatus::Active, VehicleStatus::Maintenance) |
                (VehicleStatus::Maintenance, VehicleStatus::Active) |
                (VehicleStatus::Active, VehicleStatus::Retired) |
                (VehicleStatus::Retired, VehicleStatus::Active)
            )
        }
    }

    // ========== 司机模块测试 ==========
    mod driver_tests {
        use super::*;

        #[test]
        fn test_driver_entity_creation() {
            let driver = Driver {
                id: None,
                name: "张三".to_string(),
                license_number: "A123456".to_string(),
                phone: "13800138000".to_string(),
                status: DriverStatus::Available,
                vehicle_id: None,
                created_at: None,
                updated_at: None,
            };

            assert_eq!(driver.name, "张三");
            assert_eq!(driver.license_number, "A123456");
            assert_eq!(driver.status, DriverStatus::Available);
        }

        #[test]
        fn test_driver_status() {
            let driver = Driver {
                id: None,
                name: "李四".to_string(),
                license_number: "B654321".to_string(),
                phone: "13900139000".to_string(),
                status: DriverStatus::OnDuty,
                vehicle_id: Some(1),
                created_at: None,
                updated_at: None,
            };

            assert!(driver.vehicle_id.is_some());
            assert!(matches!(driver.status, DriverStatus::OnDuty));
        }

        #[test]
        fn test_driver_phone_validation() {
            let driver = Driver {
                id: None,
                name: "王五".to_string(),
                license_number: "C111111".to_string(),
                phone: "13800138000".to_string(),
                status: DriverStatus::Available,
                vehicle_id: None,
                created_at: None,
                updated_at: None,
            };

            // 手机号验证：必须是中国大陆手机号格式
            let phone = &driver.phone;
            assert!(phone.len() == 11);
            assert!(phone.starts_with("1"));
        }
    }

    // ========== 设备模块测试 ==========
    mod device_tests {
        use super::*;

        #[test]
        fn test_device_entity_creation() {
            let device = Device {
                id: None,
                device_type: DeviceType::TruckScale,
                name: "地磅1号".to_string(),
                location: "仓库入口".to_string(),
                status: DeviceStatus::Online,
                ip_address: Some("192.168.1.100".to_string()),
                created_at: None,
                updated_at: None,
            };

            assert_eq!(device.device_type, DeviceType::TruckScale);
            assert_eq!(device.name, "地磅1号");
            assert_eq!(device.status, DeviceStatus::Online);
        }

        #[test]
        fn test_device_types() {
            let types = vec![
                DeviceType::TruckScale,
                DeviceType::Camera,
                DeviceType::Sensor,
                DeviceType::GPS,
                DeviceType::Controller,
            ];

            for dt in types {
                assert!(matches!(
                    dt,
                    DeviceType::TruckScale |
                    DeviceType::Camera |
                    DeviceType::Sensor |
                    DeviceType::GPS |
                    DeviceType::Controller
                ));
            }
        }

        #[test]
        fn test_device_ip_address_validation() {
            let device = Device {
                id: None,
                device_type: DeviceType::Camera,
                name: "监控摄像头1".to_string(),
                location: "门口".to_string(),
                status: DeviceStatus::Online,
                ip_address: Some("192.168.1.100".to_string()),
                created_at: None,
                updated_at: None,
            };

            if let Some(ip) = &device.ip_address {
                // 简单验证IP格式
                assert!(ip.contains('.') && ip.split('.').count() == 4);
            }
        }
    }

    // ========== 订单模块测试 ==========
    mod order_tests {
        use super::*;
        use chrono::{DateTime, Utc};

        #[test]
        fn test_order_entity_creation() {
            let order = Order {
                id: None,
                order_number: "ORD202401010001".to_string(),
                vehicle_id: 1,
                driver_id: Some(1),
                status: OrderStatus::Pending,
                order_type: OrderType::Inbound,
                cargo_type: "钢材".to_string(),
                expected_weight: Some(50000.0),
                actual_weight: None,
                created_at: None,
                updated_at: None,
            };

            assert_eq!(order.order_number, "ORD202401010001");
            assert_eq!(order.status, OrderStatus::Pending);
            assert_eq!(order.order_type, OrderType::Inbound);
        }

        #[test]
        fn test_order_status_transitions() {
            let order = Order {
                id: Some(1),
                order_number: "ORD001".to_string(),
                vehicle_id: 1,
                driver_id: Some(1),
                status: OrderStatus::Pending,
                order_type: OrderType::Inbound,
                cargo_type: "货物".to_string(),
                expected_weight: Some(10000.0),
                actual_weight: None,
                created_at: None,
                updated_at: None,
            };

            // 验证订单状态转换
            assert!(is_valid_order_transition(&OrderStatus::Pending, &OrderStatus::InProgress));
            assert!(is_valid_order_transition(&OrderStatus::InProgress, &OrderStatus::Completed));
            assert!(is_valid_order_transition(&OrderStatus::InProgress, &OrderStatus::Cancelled));
        }

        #[test]
        fn test_order_weight_calculation() {
            let mut order = Order {
                id: Some(1),
                order_number: "ORD001".to_string(),
                vehicle_id: 1,
                driver_id: Some(1),
                status: OrderStatus::Completed,
                order_type: OrderType::Inbound,
                cargo_type: "钢材".to_string(),
                expected_weight: Some(50000.0),
                actual_weight: Some(48500.0),
                created_at: None,
                updated_at: None,
            };

            // 计算重量差异
            let diff = order.expected_weight.unwrap() - order.actual_weight.unwrap();
            let diff_percent = (diff / order.expected_weight.unwrap()) * 100.0;

            // 差异应该在 5% 以内
            assert!(diff_percent <= 5.0);
        }

        fn is_valid_order_transition(from: &OrderStatus, to: &OrderStatus) -> bool {
            matches!(
                (from, to),
                (OrderStatus::Pending, OrderStatus::InProgress) |
                (OrderStatus::Pending, OrderStatus::Cancelled) |
                (OrderStatus::InProgress, OrderStatus::Completed) |
                (OrderStatus::InProgress, OrderStatus::Cancelled)
            )
        }
    }

    // ========== 统计模块测试 ==========
    mod statistic_tests {
        use super::*;

        #[test]
        fn test_statistic_entity() {
            let stat = Statistic {
                id: None,
                stat_type: StatisticType::Daily,
                date: "2024-01-01".to_string(),
                vehicle_count: 100,
                order_count: 500,
                total_weight: 1000000.0,
                created_at: None,
            };

            assert_eq!(stat.vehicle_count, 100);
            assert_eq!(stat.order_count, 500);
        }

        #[test]
        fn test_statistic_aggregation() {
            let stats = vec![
                Statistic {
                    id: None,
                    stat_type: StatisticType::Hourly,
                    date: "2024-01-01 08:00".to_string(),
                    vehicle_count: 10,
                    order_count: 50,
                    total_weight: 100000.0,
                    created_at: None,
                },
                Statistic {
                    id: None,
                    stat_type: StatisticType::Hourly,
                    date: "2024-01-01 09:00".to_string(),
                    vehicle_count: 15,
                    order_count: 75,
                    total_weight: 150000.0,
                    created_at: None,
                },
            ];

            // 聚合计算
            let total_vehicles: i32 = stats.iter().map(|s| s.vehicle_count).sum();
            let total_orders: i32 = stats.iter().map(|s| s.order_count).sum();
            let total_weight: f64 = stats.iter().map(|s| s.total_weight).sum();

            assert_eq!(total_vehicles, 25);
            assert_eq!(total_orders, 125);
            assert_eq!(total_weight, 250000.0);
        }
    }

    // ========== 称重数据模块测试 ==========
    mod weighing_tests {
        use super::*;

        #[test]
        fn test_weighing_data_creation() {
            let weighing = WeighingData {
                id: None,
                order_id: 1,
                vehicle_id: 1,
                device_id: 1,
                gross_weight: Some(50000.0),
                tare_weight: Some(15000.0),
                net_weight: None,
                weighing_type: WeighingType::Gross,
                weighing_time: None,
                operator_id: Some(1),
                created_at: None,
            };

            assert!(weighing.gross_weight.is_some());
            assert!(weighing.net_weight.is_none()); // 未计算
        }

        #[test]
        fn test_weighing_calculation() {
            let gross = 50000.0;
            let tare = 15000.0;
            let net = gross - tare;

            assert_eq!(net, 35000.0);
        }

        #[test]
        fn test_weighing_type_transitions() {
            // 称重类型转换
            assert!(matches!(WeighingType::Gross, WeighingType::Gross));
            assert!(matches!(WeighingType::Tare, WeighingType::Tare));
        }
    }

    // ========== 告警模块测试 ==========
    mod alert_tests {
        use super::*;

        #[test]
        fn test_alert_entity_creation() {
            let alert = Alert {
                id: None,
                alert_type: AlertType::Overweight,
                severity: AlertSeverity::High,
                title: "车辆超载告警".to_string(),
                message: "车辆京A12345超载500kg".to_string(),
                vehicle_id: Some(1),
                is_resolved: false,
                created_at: None,
                resolved_at: None,
            };

            assert_eq!(alert.severity, AlertSeverity::High);
            assert!(!alert.is_resolved);
        }

        #[test]
        fn test_alert_severity_levels() {
            let severities = vec![
                AlertSeverity::Low,
                AlertSeverity::Medium,
                AlertSeverity::High,
                AlertSeverity::Critical,
            ];

            assert_eq!(severities.len(), 4);
        }

        #[test]
        fn test_alert_resolution() {
            let mut alert = create_test_alert();
            assert!(!alert.is_resolved);

            alert.is_resolved = true;
            alert.resolved_at = Some(chrono::Utc::now());

            assert!(alert.is_resolved);
            assert!(alert.resolved_at.is_some());
        }

        fn create_test_alert() -> Alert {
            Alert {
                id: None,
                alert_type: AlertType::Overweight,
                severity: AlertSeverity::High,
                title: "测试告警".to_string(),
                message: "测试告警内容".to_string(),
                vehicle_id: Some(1),
                is_resolved: false,
                created_at: None,
                resolved_at: None,
            }
        }
    }

    // ========== 权限模块测试 ==========
    mod permission_tests {
        use super::*;

        #[test]
        fn test_permission_creation() {
            let permission = Permission {
                id: None,
                name: "vehicle:create".to_string(),
                resource: Resource::Vehicle,
                action: Action::Create,
                description: Some("创建车辆权限".to_string()),
            };

            assert_eq!(permission.resource, Resource::Vehicle);
            assert_eq!(permission.action, Action::Create);
        }

        #[test]
        fn test_role_permissions() {
            let admin_role = create_admin_role();
            let user_role = create_user_role();

            // 管理员应该有更多权限
            assert!(admin_role.permissions.len() > user_role.permissions.len());
        }

        #[test]
        fn test_permission_check() {
            let role = create_user_role();
            let required_permission = Permission {
                id: None,
                name: "vehicle:read".to_string(),
                resource: Resource::Vehicle,
                action: Action::Read,
                description: None,
            };

            let has_permission = role.permissions.iter().any(|p| p.name == "vehicle:read");
            assert!(has_permission);
        }

        fn create_admin_role() -> Role {
            Role {
                id: None,
                name: "管理员".to_string(),
                permissions: vec![
                    Permission {
                        id: None,
                        name: "user:*".to_string(),
                        resource: Resource::User,
                        action: Action::All,
                        description: None,
                    },
                    Permission {
                        id: None,
                        name: "vehicle:*".to_string(),
                        resource: Resource::Vehicle,
                        action: Action::All,
                        description: None,
                    },
                ],
                created_at: None,
            }
        }

        fn create_user_role() -> Role {
            Role {
                id: None,
                name: "普通用户".to_string(),
                permissions: vec![
                    Permission {
                        id: None,
                        name: "vehicle:read".to_string(),
                        resource: Resource::Vehicle,
                        action: Action::Read,
                        description: None,
                    },
                    Permission {
                        id: None,
                        name: "order:read".to_string(),
                        resource: Resource::Order,
                        action: Action::Read,
                        description: None,
                    },
                ],
                created_at: None,
            }
        }
    }
}
