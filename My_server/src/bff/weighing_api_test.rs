//! 称重数据 API 集成测试

#[cfg(test)]
mod tests {
    use crate::schemas::WeighingDataResponse;
    use chrono::NaiveDateTime;

    #[test]
    fn test_weighing_data_response_id_is_i64() {
        let response = WeighingDataResponse {
            id: 12345678901234i64,
            vehicle_id: 1,
            device_id: 1,
            weighing_time: NaiveDateTime::parse_from_str("2024-01-01 12:00:00", "%Y-%m-%d %H:%M:%S").unwrap(),
            gross_weight: 5000.0,
            tare_weight: 2000.0,
            net_weight: 3000.0,
            axle_count: 3,
            speed: None,
            lane_no: 1,
            site_id: 1,
            status: 0,
            create_time: NaiveDateTime::parse_from_str("2024-01-01 12:00:00", "%Y-%m-%d %H:%M:%S").unwrap(),
            update_time: None,
        };
        assert!(response.id > i32::MAX as i64);
    }

    #[test]
    fn test_weighing_data_json_roundtrip() {
        let response = WeighingDataResponse {
            id: 1i64,
            vehicle_id: 1,
            device_id: 1,
            weighing_time: NaiveDateTime::parse_from_str("2024-01-01 12:00:00", "%Y-%m-%d %H:%M:%S").unwrap(),
            gross_weight: 5000.0,
            tare_weight: 2000.0,
            net_weight: 3000.0,
            axle_count: 3,
            speed: None,
            lane_no: 1,
            site_id: 1,
            status: 0,
            create_time: NaiveDateTime::parse_from_str("2024-01-01 12:00:00", "%Y-%m-%d %H:%M:%S").unwrap(),
            update_time: None,
        };

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: WeighingDataResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(response.id, deserialized.id);
    }
}
