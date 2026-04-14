//! / 消息转换器单元测试
use crate::truck_scale::handlers::user_group_handler::UserGroupInfo as HandlerUserGroupInfo;
use crate::truck_scale::handlers::user_handler::UserInfo as HandlerUserInfo;
use crate::truck_scale::handlers::vehicle_group_handler::VehicleGroupInfo as HandlerVehicleGroupInfo;
use crate::truck_scale::handlers::vehicle_handler::VehicleInfo as HandlerVehicleInfo;
use crate::truck_scale::protocol::message_protocol::{
    UserGroupInfo, UserInfo, VehicleGroupInfo, VehicleInfo,
};
use crate::truck_scale::transformers::message_transformer::*;

#[cfg(test)]
mod message_transformer_tests {
    use super::*;

    #[test]
    fn test_vehicle_info_from_handler() {
        let handler_info = HandlerVehicleInfo {
            vehicle_id: "v123".to_string(),
            plate_no: "粤A12345".to_string(),
            terminal_no: "t678".to_string(),
            sim_no: "13800138000".to_string(),
            engine_no: "e999".to_string(),
            frame_no: "f111".to_string(),
            owner_name: "owner".to_string(),
            owner_tel: "13900139000".to_string(),
            owner_address: "address".to_string(),
            vehicle_type: "货车".to_string(),
            vehicle_color: "红色".to_string(),
            vehicle_brand: "品牌A".to_string(),
            vehicle_model: "型号B".to_string(),
            group_id: "g888".to_string(),
            driver_name: "张三".to_string(),
            driver_tel: "13800138000".to_string(),
            driver_license: "license".to_string(),
            max_weight: 50000.0,
            tare_weight: 10000.0,
            rated_weight: 45000.0,
            length: 10.0,
            width: 2.5,
            height: 3.5,
            fuel_type: "柴油".to_string(),
            manufacturer: "制造商".to_string(),
            manufacture_date: "2020-01-01".to_string(),
            registration_date: "2020-02-01".to_string(),
            insurance_expire_date: "2025-02-01".to_string(),
            annual_inspection_date: "2025-01-01".to_string(),
            remark: "备注".to_string(),
            status: 0,
            create_time: "2024-01-01".to_string(),
            update_time: "2024-02-01".to_string(),
            create_by: "admin".to_string(),
            update_by: "admin".to_string(),
        };

        let protocol_info = MessageTransformer::vehicle_info_from_handler(&handler_info);

        assert_eq!(protocol_info.vehicle_id, "v123");
        assert_eq!(protocol_info.plate_no, "粤A12345");
        assert_eq!(protocol_info.terminal_no, Some("t678".to_string()));
        assert_eq!(protocol_info.sim_no, Some("13800138000".to_string()));
        assert_eq!(protocol_info.group_id, Some("g888".to_string()));
        assert_eq!(protocol_info.driver_name, Some("张三".to_string()));
        assert_eq!(protocol_info.driver_tel, Some("13800138000".to_string()));
        assert_eq!(protocol_info.vehicle_type, Some("货车".to_string()));
        assert_eq!(protocol_info.vehicle_brand, Some("品牌A".to_string()));
        assert_eq!(protocol_info.vehicle_model, Some("型号B".to_string()));
        assert_eq!(protocol_info.max_weight, Some(50000.0));
        assert_eq!(protocol_info.tare_weight, Some(10000.0));
        assert_eq!(protocol_info.status, Some(0));
    }

    #[test]
    fn test_vehicle_info_to_handler() {
        let protocol_info = VehicleInfo {
            vehicle_id: "v123".to_string(),
            plate_no: "粤A12345".to_string(),
            terminal_no: Some("t678".to_string()),
            sim_no: Some("13800138000".to_string()),
            group_id: Some("g888".to_string()),
            driver_name: Some("张三".to_string()),
            driver_tel: Some("13800138000".to_string()),
            vehicle_type: Some("货车".to_string()),
            vehicle_brand: Some("品牌A".to_string()),
            vehicle_model: Some("型号B".to_string()),
            max_weight: Some(50000.0),
            tare_weight: Some(10000.0),
            status: Some(0),
        };

        let handler_info = MessageTransformer::vehicle_info_to_handler(&protocol_info);

        assert_eq!(handler_info.vehicle_id, "v123");
        assert_eq!(handler_info.plate_no, "粤A12345");
        assert_eq!(handler_info.terminal_no, "t678");
        assert_eq!(handler_info.sim_no, "13800138000");
        assert_eq!(handler_info.group_id, "g888");
        assert_eq!(handler_info.driver_name, "张三");
        assert_eq!(handler_info.driver_tel, "13800138000");
        assert_eq!(handler_info.vehicle_type, "货车");
        assert_eq!(handler_info.vehicle_brand, "品牌A");
        assert_eq!(handler_info.vehicle_model, "型号B");
        assert_eq!(handler_info.max_weight, 50000.0);
        assert_eq!(handler_info.tare_weight, 10000.0);
        assert_eq!(handler_info.status, 0);
    }

    #[test]
    fn test_vehicle_info_from_handler_empty_fields() {
        let handler_info = HandlerVehicleInfo {
            vehicle_id: "v123".to_string(),
            plate_no: "粤A12345".to_string(),
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
        };

        let protocol_info = MessageTransformer::vehicle_info_from_handler(&handler_info);

        assert_eq!(protocol_info.terminal_no, None);
        assert_eq!(protocol_info.sim_no, None);
        assert_eq!(protocol_info.group_id, None);
        assert_eq!(protocol_info.driver_name, None);
        assert_eq!(protocol_info.driver_tel, None);
        assert_eq!(protocol_info.vehicle_type, None);
        assert_eq!(protocol_info.vehicle_brand, None);
        assert_eq!(protocol_info.vehicle_model, None);
        assert_eq!(protocol_info.max_weight, None);
        assert_eq!(protocol_info.tare_weight, None);
    }

    #[test]
    fn test_user_info_from_handler() {
        let handler_info = HandlerUserInfo {
            user_id: "u123".to_string(),
            user_name: "testuser".to_string(),
            password: "hashed_password".to_string(),
            real_name: "李四".to_string(),
            user_type: 3,
            group_id: "g888".to_string(),
            company: "公司A".to_string(),
            department: "部门B".to_string(),
            tel: "07551234567".to_string(),
            mobile: "13900139000".to_string(),
            email: "test@example.com".to_string(),
            address: "深圳市".to_string(),
            permission: "vehicle:view,vehicle:edit,user:view".to_string(),
            veh_group_list: "g888,g889".to_string(),
            status: 0,
            expiration_time: "2026-12-31".to_string(),
            title: "经理".to_string(),
            id_card: "123456789012345678".to_string(),
            id_card_expire_date: "2030-12-31".to_string(),
            education: "本科".to_string(),
            birth_date: "1990-01-01".to_string(),
            gender: 1,
            avatar: "avatar.jpg".to_string(),
            signature: "签名".to_string(),
            last_login_time: "2026-02-08T10:00:00Z".to_string(),
            last_login_ip: "192.168.1.100".to_string(),
            login_count: 100,
            remark: "备注".to_string(),
            create_time: "2024-01-01".to_string(),
            update_time: "2024-02-01".to_string(),
            create_by: "admin".to_string(),
            update_by: "admin".to_string(),
        };

        let protocol_info = MessageTransformer::user_info_from_handler(&handler_info);

        assert_eq!(protocol_info.user_id, "u123");
        assert_eq!(protocol_info.user_name, "testuser");
        assert_eq!(protocol_info.real_name, Some("李四".to_string()));
        assert_eq!(protocol_info.user_type, 3);
        assert_eq!(protocol_info.group_id, Some("g888".to_string()));
        assert_eq!(protocol_info.company, Some("公司A".to_string()));
        assert_eq!(protocol_info.mobile, Some("13900139000".to_string()));
        assert_eq!(protocol_info.email, Some("test@example.com".to_string()));
        assert_eq!(protocol_info.status, 0);
        assert_eq!(
            protocol_info.permissions,
            Some(vec![
                "vehicle:view".to_string(),
                "vehicle:edit".to_string(),
                "user:view".to_string(),
            ])
        );
    }

    #[test]
    fn test_user_info_to_handler() {
        let protocol_info = UserInfo {
            user_id: "u123".to_string(),
            user_name: "testuser".to_string(),
            real_name: Some("李四".to_string()),
            user_type: 3,
            group_id: Some("g888".to_string()),
            company: Some("公司A".to_string()),
            mobile: Some("13900139000".to_string()),
            email: Some("test@example.com".to_string()),
            status: 0,
            permissions: Some(vec!["vehicle:view".to_string(), "vehicle:edit".to_string()]),
        };

        let handler_info = MessageTransformer::user_info_to_handler(&protocol_info);

        assert_eq!(handler_info.user_id, "u123");
        assert_eq!(handler_info.user_name, "testuser");
        assert_eq!(handler_info.real_name, "李四");
        assert_eq!(handler_info.user_type, 3);
        assert_eq!(handler_info.group_id, "g888");
        assert_eq!(handler_info.company, "公司A");
        assert_eq!(handler_info.mobile, "13900139000");
        assert_eq!(handler_info.email, "test@example.com");
        assert_eq!(handler_info.status, 0);
        assert_eq!(handler_info.permission, "vehicle:view,vehicle:edit");
    }

    #[test]
    fn test_user_info_permissions_parsing() {
        let handler_info = HandlerUserInfo {
            user_id: "u123".to_string(),
            user_name: "testuser".to_string(),
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
            permission: "vehicle:view,vehicle:edit,,user:view,".to_string(),
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
        };

        let protocol_info = MessageTransformer::user_info_from_handler(&handler_info);

        assert_eq!(
            protocol_info.permissions,
            Some(vec![
                "vehicle:view".to_string(),
                "vehicle:edit".to_string(),
                "user:view".to_string(),
            ])
        );
    }

    #[test]
    fn test_user_info_empty_permissions() {
        let handler_info = HandlerUserInfo {
            user_id: "u123".to_string(),
            user_name: "testuser".to_string(),
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
        };

        let protocol_info = MessageTransformer::user_info_from_handler(&handler_info);

        assert_eq!(protocol_info.permissions, None);
    }

    #[test]
    fn test_vehicle_group_info_from_handler() {
        let handler_info = HandlerVehicleGroupInfo {
            group_id: "g123".to_string(),
            parent_id: "g0".to_string(),
            group_name: "深圳车组".to_string(),
            contact_people: "张三".to_string(),
            contact_tel: "13900139000".to_string(),
        };

        let protocol_info = MessageTransformer::vehicle_group_info_from_handler(&handler_info);

        assert_eq!(protocol_info.group_id, "g123");
        assert_eq!(protocol_info.parent_id, Some("g0".to_string()));
        assert_eq!(protocol_info.group_name, "深圳车组");
        assert_eq!(protocol_info.contact_people, Some("张三".to_string()));
        assert_eq!(protocol_info.contact_tel, Some("13900139000".to_string()));
    }

    #[test]
    fn test_vehicle_group_info_to_handler() {
        let protocol_info = VehicleGroupInfo {
            group_id: "g123".to_string(),
            parent_id: Some("g0".to_string()),
            group_name: "深圳车组".to_string(),
            contact_people: Some("张三".to_string()),
            contact_tel: Some("13900139000".to_string()),
            children: None,
        };

        let handler_info = MessageTransformer::vehicle_group_info_to_handler(&protocol_info);

        assert_eq!(handler_info.group_id, "g123");
        assert_eq!(handler_info.parent_id, "g0");
        assert_eq!(handler_info.group_name, "深圳车组");
        assert_eq!(handler_info.contact_people, "张三");
        assert_eq!(handler_info.contact_tel, "13900139000");
    }

    #[test]
    fn test_vehicle_group_info_empty_parent() {
        let handler_info = HandlerVehicleGroupInfo {
            group_id: "g123".to_string(),
            parent_id: String::new(),
            group_name: "根车组".to_string(),
            contact_people: String::new(),
            contact_tel: String::new(),
        };

        let protocol_info = MessageTransformer::vehicle_group_info_from_handler(&handler_info);

        assert_eq!(protocol_info.parent_id, None);
    }

    #[test]
    fn test_user_group_info_from_handler() {
        let handler_info = HandlerUserGroupInfo {
            group_id: "g123".to_string(),
            group_name: "管理员组".to_string(),
            user_type: 1,
            permission: "*".to_string(),
        };

        let protocol_info = MessageTransformer::user_group_info_from_handler(&handler_info);

        assert_eq!(protocol_info.group_id, "g123");
        assert_eq!(protocol_info.group_name, "管理员组");
        assert_eq!(protocol_info.user_type, 1);
        assert_eq!(protocol_info.permission, Some("*".to_string()));
    }

    #[test]
    fn test_user_group_info_to_handler() {
        let protocol_info = UserGroupInfo {
            group_id: "g123".to_string(),
            group_name: "管理员组".to_string(),
            user_type: 1,
            permission: Some("*".to_string()),
            user_count: None,
        };

        let handler_info = MessageTransformer::user_group_info_to_handler(&protocol_info);

        assert_eq!(handler_info.group_id, "g123");
        assert_eq!(handler_info.group_name, "管理员组");
        assert_eq!(handler_info.user_type, 1);
        assert_eq!(handler_info.permission, "*");
    }

    #[test]
    fn test_message_transformer_default() {
        let _transformer = MessageTransformer::default();
        let _ = MessageTransformer::new();
        // 测试默认构造函数不 panic
    }

    #[test]
    fn test_vehicle_info_zero_weights() {
        let handler_info = HandlerVehicleInfo {
            vehicle_id: "v123".to_string(),
            plate_no: "粤A12345".to_string(),
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
        };

        let protocol_info = MessageTransformer::vehicle_info_from_handler(&handler_info);

        assert_eq!(protocol_info.max_weight, None);
        assert_eq!(protocol_info.tare_weight, None);
    }
}
