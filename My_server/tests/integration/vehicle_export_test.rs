//! / 报表导出集成测试

#[cfg(test)]
mod tests {
    use carptms::bff::export::{ExportFormat, ReportExportService};
    use carptms::bff::models::*;
    use chrono::{DateTime, Utc, TimeZone};

    fn create_test_vehicle_operation_report() -> VehicleOperationReport {
        VehicleOperationReport {
            start_time: Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(),
            end_time: Utc.with_ymd_and_hms(2024, 1, 31, 23, 59, 59).unwrap(),
            summary: VehicleSummary {
                total_vehicles: 2,
                total_mileage: 1500.5,
                average_speed: 65.0,
                max_speed: 120.0,
                total_track_points: 500,
                online_vehicles: 2,
            },
            vehicles: vec![
                VehicleData {
                    vehicle_id: 1,
                    license_plate: "京A12345".to_string(),
                    driver_name: Some("张三".to_string()),
                    total_mileage: 800.25,
                    average_speed: 60.0,
                    max_speed: 100.0,
                    online_duration: 200.0,
                    track_point_count: 250,
                    fuel_consumption: Some(80.5),
                },
                VehicleData {
                    vehicle_id: 2,
                    license_plate: "京B67890".to_string(),
                    driver_name: Some("李四".to_string()),
                    total_mileage: 700.25,
                    average_speed: 70.0,
                    max_speed: 120.0,
                    online_duration: 180.0,
                    track_point_count: 250,
                    fuel_consumption: Some(70.0),
                },
            ],
        }
    }

    fn create_test_weighing_report() -> WeighingStatisticsReport {
        WeighingStatisticsReport {
            start_time: Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(),
            end_time: Utc.with_ymd_and_hms(2024, 1, 31, 23, 59, 59).unwrap(),
            summary: WeighingSummary {
                total_weighings: 2,
                total_net_weight: 100.0,
                average_net_weight: 50.0,
            },
            weighings: vec![
                WeighingData {
                    vehicle_id: 1,
                    license_plate: "京A12345".to_string(),
                    gross_weight: 50.0,
                    tare_weight: 20.0,
                    net_weight: 30.0,
                    weighing_time: Utc.with_ymd_and_hms(2024, 1, 15, 10, 30, 0).unwrap(),
                    location: Some("1号地磅".to_string()),
                    operator: Some("操作员A".to_string()),
                    notes: None,
                },
                WeighingData {
                    vehicle_id: 2,
                    license_plate: "京B67890".to_string(),
                    gross_weight: 40.0,
                    tare_weight: 20.0,
                    net_weight: 20.0,
                    weighing_time: Utc.with_ymd_and_hms(2024, 1, 16, 14, 0, 0).unwrap(),
                    location: Some("2号地磅".to_string()),
                    operator: Some("操作员B".to_string()),
                    notes: Some("超限".to_string()),
                },
            ],
        }
    }

    #[test]
    fn test_export_format_parsing() {
        assert_eq!(ExportFormat::parse_format("excel"), Some(ExportFormat::Excel));
        assert_eq!(ExportFormat::parse_format("xlsx"), Some(ExportFormat::Excel));
        assert_eq!(ExportFormat::parse_format("csv"), Some(ExportFormat::Csv));
        assert_eq!(ExportFormat::parse_format("json"), Some(ExportFormat::Json));
        assert_eq!(ExportFormat::parse_format("unknown"), None);
    }

    #[test]
    fn test_export_format_content_type() {
        assert_eq!(ExportFormat::Excel.content_type(), 
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet");
        assert_eq!(ExportFormat::Csv.content_type(), "text/csv; charset=utf-8");
        assert_eq!(ExportFormat::Json.content_type(), "application/json");
    }

    #[test]
    fn test_export_format_extension() {
        assert_eq!(ExportFormat::Excel.extension(), ".xlsx");
        assert_eq!(ExportFormat::Csv.extension(), ".csv");
        assert_eq!(ExportFormat::Json.extension(), ".json");
    }

    #[test]
    fn test_export_vehicle_operation_csv() {
        let report = create_test_vehicle_operation_report();
        
        let result = ReportExportService::export_vehicle_operation_report(
            &report, 
            ExportFormat::Csv
        );
        
        assert!(result.is_ok());
        let data = result.unwrap();
        let csv_content = String::from_utf8_lossy(&data);
        
        assert!(csv_content.contains("京A12345"));
        assert!(csv_content.contains("京B67890"));
        assert!(csv_content.contains("800.25"));
        assert!(csv_content.contains("700.25"));
    }

    #[test]
    fn test_export_vehicle_operation_json() {
        let report = create_test_vehicle_operation_report();
        
        let result = ReportExportService::export_vehicle_operation_report(
            &report, 
            ExportFormat::Json
        );
        
        assert!(result.is_ok());
        let data = result.unwrap();
        let json_content = String::from_utf8_lossy(&data);
        
        assert!(json_content.contains("京A12345"));
        assert!(json_content.contains("total_mileage"));
    }

    #[test]
    fn test_export_vehicle_operation_excel() {
        let report = create_test_vehicle_operation_report();
        
        let result = ReportExportService::export_vehicle_operation_report(
            &report, 
            ExportFormat::Excel
        );
        
        assert!(result.is_ok());
        let data = result.unwrap();
        
        // Excel 文件以 PK (ZIP) 格式开头
        assert!(!data.is_empty());
        assert_eq!(data[0], 0x50); // 'P'
        assert_eq!(data[1], 0x4B); // 'K'
    }

    #[test]
    fn test_export_weighing_csv() {
        let report = create_test_weighing_report();
        
        let result = ReportExportService::export_weighing_statistics_report(
            &report, 
            ExportFormat::Csv
        );
        
        assert!(result.is_ok());
        let data = result.unwrap();
        let csv_content = String::from_utf8_lossy(&data);
        
        assert!(csv_content.contains("京A12345"));
        assert!(csv_content.contains("30.00")); // 净重
    }

    #[test]
    fn test_export_weighing_excel() {
        let report = create_test_weighing_report();
        
        let result = ReportExportService::export_weighing_statistics_report(
            &report, 
            ExportFormat::Excel
        );
        
        assert!(result.is_ok());
        let data = result.unwrap();
        
        // Excel 文件以 PK (ZIP) 格式开头
        assert!(!data.is_empty());
    }

    #[test]
    fn test_export_html_not_supported() {
        let report = create_test_vehicle_operation_report();
        
        let result = ReportExportService::export_vehicle_operation_report(
            &report, 
            ExportFormat::Html
        );
        
        assert!(result.is_err());
    }
}
