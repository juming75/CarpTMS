//! 地磅处理器集成测试

#[cfg(test)]
mod tests {
    use crate::truck_scale::handlers::recycle_handler::RecycledVehicle;
    use crate::truck_scale::handlers::permission_handler::UserPermissions;

    // ============= RecycleHandler Tests =============

    #[test]
    fn test_recycled_vehicle_structure() {
        let vehicle = RecycledVehicle {
            vehicle_id: "V001".to_string(),
            plate_no: "京A12345".to_string(),
            delete_time: "2024-01-01 12:00:00".to_string(),
            delete_by: "admin".to_string(),
        };

        assert_eq!(vehicle.vehicle_id, "V001");
        assert_eq!(vehicle.plate_no, "京A12345");
    }

    #[test]
    fn test_recycled_vehicles_list() {
        let vehicles = vec![
            RecycledVehicle {
                vehicle_id: "V001".to_string(),
                plate_no: "京A12345".to_string(),
                delete_time: "2024-01-01 12:00:00".to_string(),
                delete_by: "admin".to_string(),
            },
            RecycledVehicle {
                vehicle_id: "V002".to_string(),
                plate_no: "京B67890".to_string(),
                delete_time: "2024-01-02 12:00:00".to_string(),
                delete_by: "admin".to_string(),
            },
        ];

        assert_eq!(vehicles.len(), 2);
        assert!(vehicles[0].delete_time < vehicles[1].delete_time);
    }

    // ============= PermissionHandler Tests =============

    #[test]
    fn test_user_permissions_structure() {
        let perms = UserPermissions {
            user_id: "U001".to_string(),
            user_name: "testuser".to_string(),
            real_name: "测试用户".to_string(),
            permission: "read,write".to_string(),
            veh_group_list: "G001,G002".to_string(),
        };

        assert_eq!(perms.user_id, "U001");
        assert!(perms.permission.contains("read"));
    }

    #[test]
    fn test_user_permissions_parsing() {
        let perms = UserPermissions {
            user_id: "U001".to_string(),
            user_name: "admin".to_string(),
            real_name: "管理员".to_string(),
            permission: "admin,read,write,delete".to_string(),
            veh_group_list: "G001,G002,G003".to_string(),
        };

        let perm_list: Vec<&str> = perms.permission.split(',').collect();
        assert_eq!(perm_list.len(), 4);
        assert!(perm_list.contains(&"admin"));
        assert!(perm_list.contains(&"delete"));
    }

    #[test]
    fn test_vehicle_groups_parsing() {
        let perms = UserPermissions {
            user_id: "U001".to_string(),
            user_name: "operator".to_string(),
            real_name: "操作员".to_string(),
            permission: "read,write".to_string(),
            veh_group_list: "G001,G002,G003,G004,G005".to_string(),
        };

        let groups: Vec<&str> = perms.veh_group_list.split(',').collect();
        assert_eq!(groups.len(), 5);
    }
}
