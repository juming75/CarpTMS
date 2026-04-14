//! / 消息协议单元测试
use crate::truck_scale::handlers::user_handler::UserInfo as HandlerUserInfo;
use crate::truck_scale::handlers::vehicle_handler::VehicleInfo as HandlerVehicleInfo;
use crate::truck_scale::protocol::message_protocol::*;
use crate::truck_scale::transformers::MessageTransformer;
use chrono::Utc;

#[cfg(test)]
mod message_protocol_tests {
    use super::*;

    #[test]
    fn test_message_header_default() {
        let header = MessageHeader::default();
        assert!(header.message_id.len() > 0);
        assert_eq!(header.message_type, MessageType::Notification);
        assert!(header.timestamp <= Utc::now());
        assert_eq!(header.session_id, None);
        assert_eq!(header.user_id, None);
        assert_eq!(header.device_id, None);
        assert_eq!(header.priority, MessagePriority::default());
        assert_eq!(header.version, "1.0");
    }

    #[test]
    fn test_message_header_builder() {
        let header = MessageHeader::new(MessageType::Login)
            .with_session_id("session123".to_string())
            .with_user_id("user456".to_string())
            .with_device_id("device789".to_string())
            .with_protocol_type("BSJ".to_string())
            .with_priority(MessagePriority::High);

        assert_eq!(header.message_type, MessageType::Login);
        assert_eq!(header.session_id, Some("session123".to_string()));
        assert_eq!(header.user_id, Some("user456".to_string()));
        assert_eq!(header.device_id, Some("device789".to_string()));
        assert_eq!(header.protocol_type, Some("BSJ".to_string()));
        assert_eq!(header.priority, MessagePriority::High);
    }

    #[test]
    fn test_login_message_creation() {
        let message = UnifiedMessage::login(
            "test_user".to_string(),
            "test_pass".to_string(),
            "truck_scale".to_string(),
        );

        assert_eq!(message.header.message_type, MessageType::Login);
        match &message.body {
            MessageBody::Login(req) => {
                assert_eq!(req.username, "test_user");
                assert_eq!(req.password, "test_pass");
                assert_eq!(req.client_type, "truck_scale");
            }
            _ => panic!("Expected Login message body"),
        }
    }

    #[test]
    fn test_login_response_message() {
        let message = UnifiedMessage::login_response(
            true,
            Some("user123".to_string()),
            Some("测试用户".to_string()),
        );

        assert_eq!(message.header.message_type, MessageType::LoginResponse);
        match &message.body {
            MessageBody::LoginResponse(resp) => {
                assert!(resp.success);
                assert_eq!(resp.user_id, Some("user123".to_string()));
                assert_eq!(resp.user_name, Some("测试用户".to_string()));
            }
            _ => panic!("Expected LoginResponse message body"),
        }
    }

    #[test]
    fn test_heartbeat_message() {
        let message = UnifiedMessage::heartbeat("session123".to_string());

        assert_eq!(message.header.message_type, MessageType::Heartbeat);
        assert_eq!(message.header.session_id, Some("session123".to_string()));
        match &message.body {
            MessageBody::Heartbeat(hb) => {
                assert_eq!(hb.status, "ok");
                assert!(hb.timestamp <= Utc::now());
            }
            _ => panic!("Expected Heartbeat message body"),
        }
    }

    #[test]
    fn test_query_vehicle_message() {
        let message = MessageTransformer::create_query_vehicle_message(
            Some("vehicle123".to_string()),
            None,
            None,
            None,
            None,
            None,
        );

        assert_eq!(message.header.message_type, MessageType::QueryVehicle);
        match &message.body {
            MessageBody::QueryVehicle(req) => {
                assert_eq!(req.vehicle_id, Some("vehicle123".to_string()));
            }
            _ => panic!("Expected QueryVehicle message body"),
        }
    }

    #[test]
    fn test_query_user_message() {
        let message = MessageTransformer::create_query_user_message(
            None,
            None,
            Some("group123".to_string()),
            None,
            None,
            None,
            None,
        );

        assert_eq!(message.header.message_type, MessageType::QueryUser);
        match &message.body {
            MessageBody::QueryUser(req) => {
                assert_eq!(req.group_id, Some("group123".to_string()));
            }
            _ => panic!("Expected QueryUser message body"),
        }
    }

    #[test]
    fn test_message_serialization() {
        let original = UnifiedMessage::login(
            "test_user".to_string(),
            "test_pass".to_string(),
            "web".to_string(),
        );

        let json = original.to_json().unwrap();
        assert!(json.contains("test_user"));
        assert!(json.contains("web"));

        let deserialized = UnifiedMessage::from_json(&json).unwrap();
        assert_eq!(original.header.message_id, deserialized.header.message_id);
        assert_eq!(
            original.header.message_type,
            deserialized.header.message_type
        );
    }

    #[test]
    fn test_message_serialization_bytes() {
        let original = UnifiedMessage::heartbeat("session123".to_string());

        let bytes = original.to_bytes().unwrap();
        assert!(!bytes.is_empty());

        let deserialized = UnifiedMessage::from_bytes(&bytes).unwrap();
        assert_eq!(original.header.session_id, deserialized.header.session_id);
    }

    #[test]
    fn test_message_with_extensions() {
        let mut message =
            UnifiedMessage::login("test".to_string(), "pass".to_string(), "client".to_string());

        use std::collections::HashMap;
        let mut extensions = HashMap::new();
        extensions.insert(
            "custom_field".to_string(),
            serde_json::json!("custom_value"),
        );
        message.extensions = Some(extensions);

        let json = message.to_json().unwrap();
        assert!(json.contains("custom_field"));
    }

    #[test]
    fn test_error_message() {
        let message = MessageTransformer::create_error_message(
            4001,
            "Invalid credentials".to_string(),
            Some("User not found".to_string()),
            None,
        );

        assert_eq!(message.header.message_type, MessageType::Error);
        match &message.body {
            MessageBody::Error(err) => {
                assert_eq!(err.error_code, 4001);
                assert_eq!(err.error_message, "Invalid credentials");
                assert_eq!(err.details, Some("User not found".to_string()));
            }
            _ => panic!("Expected Error message body"),
        }
    }

    #[test]
    fn test_notification_message() {
        let notification = NotificationMessage {
            notification_type: "alarm".to_string(),
            title: "设备报警".to_string(),
            content: "设备123发生超载报警".to_string(),
            data: Some(serde_json::json!({"weight": 5000})),
        };

        let message = UnifiedMessage::new(
            MessageType::Notification,
            MessageBody::Notification(notification),
        );

        assert_eq!(message.header.message_type, MessageType::Notification);
        let json = message.to_json().unwrap();
        assert!(json.contains("设备报警"));
        assert!(json.contains("超载报警"));
    }

    #[test]
    fn test_data_report_message() {
        let data = serde_json::json!({
            "weight": 4500.0,
            "status": "normal",
            "timestamp": "2026-02-08T10:30:00Z"
        });

        let message = MessageTransformer::create_data_report_message(
            "weighing".to_string(),
            "device123".to_string(),
            data.clone(),
            Some("session456".to_string()),
            Some("BSJ".to_string()),
        );

        assert_eq!(message.header.message_type, MessageType::DataReport);
        assert_eq!(message.header.session_id, Some("session456".to_string()));
        assert_eq!(message.header.device_id, Some("device123".to_string()));
        assert_eq!(message.header.protocol_type, Some("BSJ".to_string()));

        match &message.body {
            MessageBody::DataReport(report) => {
                assert_eq!(report.report_type, "weighing");
                assert_eq!(report.device_id, "device123");
            }
            _ => panic!("Expected DataReport message body"),
        }
    }

    #[test]
    fn test_vehicle_info_creation() {
        let vehicle_info = VehicleInfo {
            vehicle_id: "v123".to_string(),
            plate_no: "粤A12345".to_string(),
            terminal_no: Some("t678".to_string()),
            sim_no: Some("13800138000".to_string()),
            group_id: Some("g999".to_string()),
            driver_name: Some("张三".to_string()),
            driver_tel: None,
            vehicle_type: Some("货车".to_string()),
            vehicle_brand: None,
            vehicle_model: None,
            max_weight: Some(50000.0),
            tare_weight: Some(10000.0),
            status: Some(0),
        };

        let json = serde_json::to_string(&vehicle_info).unwrap();
        assert!(json.contains("粤A12345"));
        assert!(json.contains("张三"));
    }

    #[test]
    fn test_user_info_creation() {
        let user_info = UserInfo {
            user_id: "u123".to_string(),
            user_name: "testuser".to_string(),
            real_name: Some("李四".to_string()),
            user_type: 3,
            group_id: Some("g999".to_string()),
            company: Some("公司A".to_string()),
            mobile: Some("13900139000".to_string()),
            email: None,
            status: 0,
            permissions: Some(vec!["vehicle:view".to_string(), "vehicle:edit".to_string()]),
        };

        let json = serde_json::to_string(&user_info).unwrap();
        assert!(json.contains("testuser"));
        assert!(json.contains("vehicle:view"));
    }

    #[test]
    fn test_message_priority_values() {
        assert_eq!(MessagePriority::Low as i32, 0);
        assert_eq!(MessagePriority::Normal as i32, 1);
        assert_eq!(MessagePriority::High as i32, 2);
    }

    #[test]
    fn test_all_message_types() {
        let types = vec![
            MessageType::Login,
            MessageType::LoginResponse,
            MessageType::Logout,
            MessageType::Heartbeat,
            MessageType::QueryVehicle,
            MessageType::VehicleData,
            MessageType::QueryUser,
            MessageType::UserData,
            MessageType::QueryVehicleGroup,
            MessageType::VehicleGroupData,
            MessageType::QueryUserGroup,
            MessageType::UserGroupData,
            MessageType::DataReport,
            MessageType::DataReportResponse,
            MessageType::Error,
            MessageType::Notification,
        ];

        for msg_type in types {
            let header = MessageHeader::new(msg_type);
            assert_eq!(header.message_type, msg_type);
        }
    }

    #[test]
    fn test_vehicle_data_response() {
        let vehicles = vec![HandlerVehicleInfo {
            vehicle_id: "v1".to_string(),
            plate_no: "粤A11111".to_string(),
            terminal_no: String::new(),
            sim_no: String::new(),
            engine_no: String::new(),
            frame_no: String::new(),
            owner_name: String::new(),
            owner_tel: String::new(),
            owner_address: String::new(),
            vehicle_type: String::new(),
            vehicle_color: String::new(),
            vehicle_brand: String::new(),
            vehicle_model: String::new(),
            group_id: String::new(),
            driver_name: String::new(),
            driver_tel: String::new(),
            driver_license: String::new(),
            max_weight: 0.0,
            tare_weight: 0.0,
            rated_weight: 0.0,
            length: 0.0,
            width: 0.0,
            height: 0.0,
            fuel_type: String::new(),
            manufacturer: String::new(),
            manufacture_date: String::new(),
            registration_date: String::new(),
            insurance_expire_date: String::new(),
            annual_inspection_date: String::new(),
            remark: String::new(),
            status: 0,
            create_time: String::new(),
            update_time: String::new(),
            create_by: String::new(),
            update_by: String::new(),
        }];

        let message = MessageTransformer::create_vehicle_data_message(
            vehicles,
            Some(1),
            Some(1),
            Some(20),
            None,
        );

        assert_eq!(message.header.message_type, MessageType::VehicleData);
        match &message.body {
            MessageBody::VehicleData(data) => {
                assert_eq!(data.total, Some(1));
                assert_eq!(data.page, Some(1));
                assert_eq!(data.page_size, Some(20));
                assert_eq!(data.vehicles.len(), 1);
            }
            _ => panic!("Expected VehicleData message body"),
        }
    }

    #[test]
    fn test_user_data_response() {
        let users = vec![HandlerUserInfo {
            user_id: "u1".to_string(),
            user_name: "user1".to_string(),
            password: String::new(),
            real_name: String::new(),
            user_type: 3,
            group_id: String::new(),
            company: String::new(),
            department: String::new(),
            tel: String::new(),
            mobile: String::new(),
            email: String::new(),
            address: String::new(),
            permission: String::new(),
            veh_group_list: String::new(),
            status: 0,
            expiration_time: String::new(),
            title: String::new(),
            id_card: String::new(),
            id_card_expire_date: String::new(),
            education: String::new(),
            birth_date: String::new(),
            gender: 0,
            avatar: String::new(),
            signature: String::new(),
            last_login_time: String::new(),
            last_login_ip: String::new(),
            login_count: 0,
            remark: String::new(),
            create_time: String::new(),
            update_time: String::new(),
            create_by: String::new(),
            update_by: String::new(),
        }];

        let message =
            MessageTransformer::create_user_data_message(users, Some(1), Some(1), Some(20), None);

        assert_eq!(message.header.message_type, MessageType::UserData);
        match &message.body {
            MessageBody::UserData(data) => {
                assert_eq!(data.total, Some(1));
                assert_eq!(data.users.len(), 1);
            }
            _ => panic!("Expected UserData message body"),
        }
    }
}
