//! Application 层单元测试

#[cfg(test)]
mod tests {
    use crate::application::commands::*;
    use crate::application::queries::*;
    use crate::application::{Pagination, Sorting, CommandResult, QueryResult};
    use chrono::NaiveDateTime;

    // ============= Command Tests =============

    #[test]
    fn test_command_response_success() {
        let response = CommandResponse::success(123);
        assert!(response.success);
        assert_eq!(response.affected_id, Some(123));
        assert!(response.message.is_none());
    }

    #[test]
    fn test_command_response_success_with_message() {
        let response = CommandResponse::success_with_message(456, "操作成功");
        assert!(response.success);
        assert_eq!(response.affected_id, Some(456));
        assert_eq!(response.message, Some("操作成功".to_string()));
    }

    #[test]
    fn test_command_response_failure() {
        let response = CommandResponse::failure("操作失败");
        assert!(!response.success);
        assert!(response.affected_id.is_none());
        assert_eq!(response.message, Some("操作失败".to_string()));
    }

    // ============= Query Tests =============

    #[test]
    fn test_paged_result() {
        let items = vec![1, 2, 3];
        let result = PagedResult::new(items.clone(), 100, 1, 10);
        
        assert_eq!(result.items, items);
        assert_eq!(result.total, 100);
        assert_eq!(result.page, 1);
        assert_eq!(result.page_size, 10);
        assert_eq!(result.total_pages, 10);
        assert!(result.has_next());
        assert!(!result.has_prev());
    }

    #[test]
    fn test_paged_result_navigation() {
        let items = vec!["a", "b"];
        let result = PagedResult::new(items, 50, 3, 10);
        
        assert_eq!(result.page, 3);
        assert!(result.has_next());
        assert!(result.has_prev());
        
        let last_page = PagedResult::new(vec!["z"], 51, 6, 10);
        assert!(!last_page.has_next());
        assert!(last_page.has_prev());
    }

    // ============= Pagination Tests =============

    #[test]
    fn test_pagination_default() {
        let pagination = Pagination::default();
        assert_eq!(pagination.page, 1);
        assert_eq!(pagination.page_size, 20);
    }

    #[test]
    fn test_pagination_new() {
        let pagination = Pagination::new(5, 50);
        assert_eq!(pagination.page, 5);
        assert_eq!(pagination.page_size, 50);
    }

    #[test]
    fn test_pagination_boundaries() {
        // 页码不能小于 1
        let pagination = Pagination::new(0, 10);
        assert_eq!(pagination.page, 1);
        
        // 页大小限制在 1-100 之间
        let pagination = Pagination::new(1, 0);
        assert_eq!(pagination.page_size, 1);
        
        let pagination = Pagination::new(1, 200);
        assert_eq!(pagination.page_size, 100);
    }

    #[test]
    fn test_pagination_offset() {
        let pagination = Pagination::new(3, 20);
        assert_eq!(pagination.offset(), 40);
        
        let pagination = Pagination::new(1, 10);
        assert_eq!(pagination.offset(), 0);
    }

    // ============= Sorting Tests =============

    #[test]
    fn test_sorting_default() {
        let sorting = Sorting::default();
        assert_eq!(sorting.field, "id");
        assert!(!sorting.ascending);
    }

    // ============= CommandResult Tests =============

    #[test]
    fn test_command_result_success() {
        let result: CommandResult<i32> = CommandResult::success(42);
        assert!(result.success);
        assert_eq!(result.data, Some(42));
        assert!(result.error.is_none());
    }

    #[test]
    fn test_command_result_error() {
        let result: CommandResult<String> = CommandResult::error("发生错误");
        assert!(!result.success);
        assert!(result.data.is_none());
        assert_eq!(result.error, Some("发生错误".to_string()));
    }

    // ============= QueryResult Tests =============

    #[test]
    fn test_query_result_success() {
        let result: QueryResult<Vec<i32>> = QueryResult::success(vec![1, 2, 3]);
        assert!(result.success);
        assert_eq!(result.data, Some(vec![1, 2, 3]));
        assert!(result.total.is_none());
        assert!(result.error.is_none());
    }

    #[test]
    fn test_query_result_success_with_total() {
        let result: QueryResult<Vec<i32>> = QueryResult::success_with_total(vec![1, 2], 100);
        assert!(result.success);
        assert_eq!(result.total, Some(100));
    }

    #[test]
    fn test_query_result_error() {
        let result: QueryResult<String> = QueryResult::error("查询失败");
        assert!(!result.success);
        assert!(result.data.is_none());
        assert_eq!(result.error, Some("查询失败".to_string()));
    }

    // ============= CreateVehicleCommand Tests =============

    fn create_test_vehicle_command() -> CreateVehicleCommand {
        CreateVehicleCommand {
            vehicle_name: "测试车辆".to_string(),
            license_plate: "京A12345".to_string(),
            vehicle_type: "货车".to_string(),
            vehicle_color: "白色".to_string(),
            vehicle_brand: "东风".to_string(),
            vehicle_model: "EQ1090".to_string(),
            engine_no: "ENG123".to_string(),
            frame_no: "FRA123".to_string(),
            register_date: NaiveDateTime::parse_from_str("2020-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap(),
            inspection_date: NaiveDateTime::parse_from_str("2024-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap(),
            insurance_date: NaiveDateTime::parse_from_str("2024-06-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap(),
            seating_capacity: 2,
            load_capacity: 5000.0,
            vehicle_length: 6.0,
            vehicle_width: 2.0,
            vehicle_height: 2.5,
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
        }
    }

    #[test]
    fn test_create_vehicle_command_type() {
        let cmd = create_test_vehicle_command();
        assert_eq!(CreateVehicleCommand::command_type(), "create_vehicle");
    }

    #[test]
    fn test_create_vehicle_command_to_vehicle_create() {
        let cmd = create_test_vehicle_command();
        let vehicle_create = cmd.to_vehicle_create();
        
        assert_eq!(vehicle_create.vehicle_name, "测试车辆");
        assert_eq!(vehicle_create.license_plate, "京A12345");
        assert_eq!(vehicle_create.load_capacity, 5000.0);
    }

    // ============= GetVehicleQuery Tests =============

    #[test]
    fn test_get_vehicle_query_type() {
        assert_eq!(GetVehicleQuery::query_type(), "get_vehicle");
    }

    #[test]
    fn test_get_vehicle_query_creation() {
        let query = GetVehicleQuery { vehicle_id: 123 };
        assert_eq!(query.vehicle_id, 123);
    }
}
