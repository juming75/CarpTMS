//! / GPS 轨迹导出测试

#[cfg(test)]
mod tests {
    use carptms::bff::export::{ExportFormat, ReportExportService};
    use carptms::bff::models::GpsTrackPoint;
    use chrono::{DateTime, Utc, TimeZone};

    fn create_test_gps_track() -> Vec<GpsTrackPoint> {
        vec![
            GpsTrackPoint {
                latitude: 39.9042,
                longitude: 116.4074,
                altitude: Some(50.0),
                speed: 60.0,
                direction: 90,
                gps_time: Utc.with_ymd_and_hms(2024, 1, 15, 8, 0, 0).unwrap(),
            },
            GpsTrackPoint {
                latitude: 39.9045,
                longitude: 116.4080,
                altitude: Some(51.0),
                speed: 65.0,
                direction: 45,
                gps_time: Utc.with_ymd_and_hms(2024, 1, 15, 8, 5, 0).unwrap(),
            },
            GpsTrackPoint {
                latitude: 39.9050,
                longitude: 116.4090,
                altitude: Some(52.0),
                speed: 70.0,
                direction: 30,
                gps_time: Utc.with_ymd_and_hms(2024, 1, 15, 8, 10, 0).unwrap(),
            },
        ]
    }

    #[test]
    fn test_export_gps_track_csv() {
        let track_points = create_test_gps_track();
        
        let result = ReportExportService::export_gps_track_report(
            &track_points,
            1,
            "京A12345",
            Utc.with_ymd_and_hms(2024, 1, 15, 8, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2024, 1, 15, 8, 10, 0).unwrap(),
            ExportFormat::Csv,
        );
        
        assert!(result.is_ok());
        let data = result.unwrap();
        let csv_content = String::from_utf8_lossy(&data);
        
        assert!(csv_content.contains("39.9042"));
        assert!(csv_content.contains("116.4074"));
        assert!(csv_content.contains("60.00"));
    }

    #[test]
    fn test_export_gps_track_json() {
        let track_points = create_test_gps_track();
        
        let result = ReportExportService::export_gps_track_report(
            &track_points,
            1,
            "京A12345",
            Utc.with_ymd_and_hms(2024, 1, 15, 8, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2024, 1, 15, 8, 10, 0).unwrap(),
            ExportFormat::Json,
        );
        
        assert!(result.is_ok());
        let data = result.unwrap();
        let json_content = String::from_utf8_lossy(&data);
        
        assert!(json_content.contains("latitude"));
        assert!(json_content.contains("longitude"));
    }

    #[test]
    fn test_export_gps_track_excel() {
        let track_points = create_test_gps_track();
        
        let result = ReportExportService::export_gps_track_report(
            &track_points,
            1,
            "京A12345",
            Utc.with_ymd_and_hms(2024, 1, 15, 8, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2024, 1, 15, 8, 10, 0).unwrap(),
            ExportFormat::Excel,
        );
        
        assert!(result.is_ok());
        let data = result.unwrap();
        
        // Excel 文件以 PK (ZIP) 格式开头
        assert!(!data.is_empty());
        assert_eq!(data[0], 0x50); // 'P'
        assert_eq!(data[1], 0x4B); // 'K'
    }

    #[test]
    fn test_gps_track_point_creation() {
        let point = GpsTrackPoint {
            latitude: 39.9042,
            longitude: 116.4074,
            altitude: Some(50.0),
            speed: 60.0,
            direction: 90,
            gps_time: Utc::now(),
        };
        
        assert_eq!(point.latitude, 39.9042);
        assert_eq!(point.longitude, 116.4074);
        assert_eq!(point.altitude, Some(50.0));
        assert_eq!(point.speed, 60.0);
        assert_eq!(point.direction, 90);
    }

    #[test]
    fn test_gps_track_point_without_altitude() {
        let point = GpsTrackPoint {
            latitude: 39.9042,
            longitude: 116.4074,
            altitude: None,
            speed: 60.0,
            direction: 90,
            gps_time: Utc::now(),
        };
        
        assert!(point.altitude.is_none());
    }
}
